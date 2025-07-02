use anyhow::{anyhow, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use quick_xml::de::from_str;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;
use url::Url;

#[derive(Parser)]
#[command(name = "sitemap-crawler")]
#[command(about = "Un crawler che analizza sitemap e scarica pagine in parallelo")]
struct Args {
    /// URL della sitemap da analizzare
    sitemap_url: String,
    
    /// Numero di thread per le richieste parallele
    #[arg(long, default_value = "10")]
    threads: usize,
    
    /// Cartella di output
    #[arg(long, default_value = "output")]
    output: String,
    
    /// Salva i file invece di creare solo il JSON
    #[arg(long)]
    save_files: bool,
}

#[derive(Debug, Deserialize)]
struct Urlset {
    #[serde(rename = "url", default)]
    urls: Vec<UrlEntry>,
}

#[derive(Debug, Deserialize)]
struct UrlEntry {
    loc: String,
}

#[derive(Debug, Deserialize)]
struct SitemapIndex {
    #[serde(rename = "sitemap", default)]
    sitemaps: Vec<SitemapEntry>,
}

#[derive(Debug, Deserialize)]
struct SitemapEntry {
    loc: String,
}

#[derive(Debug, Serialize)]
struct PageResult {
    url: String,
    status_code: u16,
    content_length: usize,
    mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn fetch_sitemap(client: &Client, url: &str) -> Result<String> {
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(anyhow!("Failed to fetch sitemap: {}", response.status()));
    }
    Ok(response.text().await?)
}

async fn parse_sitemap_urls(client: &Client, sitemap_url: &str) -> Result<Vec<String>> {
    let content = fetch_sitemap(client, sitemap_url).await?;
    let mut all_urls = Vec::new();
    
    // Prova prima a parsare come sitemap index
    if let Ok(sitemap_index) = from_str::<SitemapIndex>(&content) {
        eprintln!("Trovata sitemap index con {} sitemap", sitemap_index.sitemaps.len());
        
        for sitemap_entry in sitemap_index.sitemaps {
            match parse_single_sitemap(client, &sitemap_entry.loc).await {
                Ok(mut urls) => {
                    eprintln!("Estratti {} URL da {}", urls.len(), sitemap_entry.loc);
                    all_urls.append(&mut urls);
                }
                Err(e) => {
                    eprintln!("Errore nel parsing della sitemap {}: {}", sitemap_entry.loc, e);
                }
            }
        }
    } else {
        // Prova a parsare come singola sitemap
        all_urls = parse_single_sitemap(client, sitemap_url).await?;
    }
    
    Ok(all_urls)
}

async fn parse_single_sitemap(client: &Client, sitemap_url: &str) -> Result<Vec<String>> {
    let content = fetch_sitemap(client, sitemap_url).await?;
    
    let urlset: Urlset = from_str(&content)
        .map_err(|e| anyhow!("Failed to parse sitemap XML: {}", e))?;
    
    Ok(urlset.urls.into_iter().map(|entry| entry.loc).collect())
}

fn url_to_filename(url: &str, used_names: &mut HashSet<String>) -> String {
    let parsed_url = Url::parse(url).unwrap_or_else(|_| Url::parse("http://example.com").unwrap());
    
    let mut filename = format!("{}{}", 
        parsed_url.host_str().unwrap_or("unknown"),
        parsed_url.path()
    );
    
    // Sostituisci gli slash con underscore
    filename = filename.replace('/', "_");
    
    // Rimuovi caratteri non validi per i nomi file
    filename = filename.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' { c } else { '_' })
        .collect();
    
    // Gestisci le collisioni
    let mut final_filename = filename.clone();
    let mut counter = 2;
    
    while used_names.contains(&final_filename) {
        final_filename = format!("{}_{}", filename, counter);
        counter += 1;
    }
    
    used_names.insert(final_filename.clone());
    final_filename
}

async fn fetch_page(client: &Client, url: &str, output_dir: &str, save_files: bool, used_names: Arc<tokio::sync::Mutex<HashSet<String>>>) -> PageResult {
    match client.get(url).send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let mime_type = response
                .headers()
                .get("content-type")
                .and_then(|ct| ct.to_str().ok())
                .unwrap_or("unknown")
                .to_string();
            
            match response.bytes().await {
                Ok(content) => {
                    let content_length = content.len();
                    
                    if save_files {
                        let mut names_guard = used_names.lock().await;
                        let filename = url_to_filename(url, &mut *names_guard);
                        drop(names_guard);
                        
                        let file_path = Path::new(output_dir).join(&filename);
                        if let Err(e) = fs::write(&file_path, &content) {
                            return PageResult {
                                url: url.to_string(),
                                status_code,
                                content_length,
                                mime_type,
                                error: Some(format!("Failed to save file: {}", e)),
                            };
                        }
                    }
                    
                    PageResult {
                        url: url.to_string(),
                        status_code,
                        content_length,
                        mime_type,
                        error: None,
                    }
                }
                Err(e) => PageResult {
                    url: url.to_string(),
                    status_code,
                    content_length: 0,
                    mime_type,
                    error: Some(format!("Failed to read response body: {}", e)),
                },
            }
        }
        Err(e) => PageResult {
            url: url.to_string(),
            status_code: 0,
            content_length: 0,
            mime_type: "unknown".to_string(),
            error: Some(format!("Request failed: {}", e)),
        },
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Crea la cartella di output
    fs::create_dir_all(&args.output)?;
    
    let client = Client::new();
    
    eprintln!("Analizzando sitemap: {}", args.sitemap_url);
    
    // Estrai tutti gli URL dalla sitemap
    let urls = parse_sitemap_urls(&client, &args.sitemap_url).await?;
    eprintln!("Trovati {} URL totali da processare", urls.len());
    
    // Setup progress bar
    let progress = ProgressBar::new(urls.len() as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    
    // Semaforo per limitare il numero di richieste concorrenti
    let semaphore = Arc::new(Semaphore::new(args.threads));
    let used_names = Arc::new(tokio::sync::Mutex::new(HashSet::new()));
    
    // Processa tutti gli URL in parallelo
    let mut tasks = Vec::new();
    
    for url in urls {
        let client = client.clone();
        let output_dir = args.output.clone();
        let save_files = args.save_files;
        let semaphore = semaphore.clone();
        let used_names = used_names.clone();
        let progress = progress.clone();
        
        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let result = fetch_page(&client, &url, &output_dir, save_files, used_names).await;
            progress.inc(1);
            result
        });
        
        tasks.push(task);
    }
    
    // Attendi tutti i task
    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await?);
    }
    
    progress.finish_with_message("Completato!");
    
    // Salva i risultati in JSON
    let json_path = Path::new(&args.output).join("results.json");
    let json_content = serde_json::to_string_pretty(&results)?;
    fs::write(&json_path, json_content)?;
    
    eprintln!("Risultati salvati in: {}", json_path.display());
    eprintln!("Processati {} URL", results.len());
    
    // Statistiche
    let successful = results.iter().filter(|r| r.error.is_none()).count();
    let failed = results.len() - successful;
    eprintln!("Successi: {}, Errori: {}", successful, failed);
    
    Ok(())
}
