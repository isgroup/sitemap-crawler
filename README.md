# Sitemap Crawler

A high-performance parallel sitemap crawler written in Rust that analyzes XML sitemaps and downloads web pages concurrently.

## Features

- **Sitemap Analysis**: Supports both individual sitemaps and sitemap index files
- **Parallel Processing**: Configurable number of concurrent threads for optimal performance
- **Flexible Output**: Save downloaded pages as files or generate JSON metadata reports
- **Collision Handling**: Automatic filename collision resolution with underscore suffixes
- **Progress Tracking**: Real-time progress reporting to stderr
- **Configurable Timeout**: Set custom timeout values for page requests
- **Error Handling**: Comprehensive error reporting and recovery

## Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))

### Build from Source

```bash
git clone <repository-url>
cd sitemap-crawler
cargo build --release
```

The compiled binary will be available at `target/release/sitemap-crawler`.

## Usage

### Basic Usage

```bash
# Analyze a sitemap and generate JSON report
./target/release/sitemap-crawler https://example.com/sitemap.xml

# Save actual page files
./target/release/sitemap-crawler https://example.com/sitemap.xml --save-files

# Use custom output directory
./target/release/sitemap-crawler https://example.com/sitemap.xml --output ./crawled-pages

# Configure parallel threads and timeout
./target/release/sitemap-crawler https://example.com/sitemap.xml --threads 20 --timeout 60
```

### Command Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `sitemap_url` | URL of the sitemap to analyze | Required |
| `--threads` | Number of concurrent threads for requests | 10 |
| `--output` | Output directory for results | "output" |
| `--save-files` | Save downloaded pages as files | false |
| `--timeout` | Timeout in seconds for page requests | 30 |

### Examples

#### Basic Crawling with JSON Output
```bash
./target/release/sitemap-crawler https://example.com/sitemap.xml
```
This will:
- Create an `output/` directory
- Generate `output/results.json` with metadata for each page
- Show progress on stderr

#### High-Performance Crawling
```bash
./target/release/sitemap-crawler https://example.com/sitemap.xml \
  --threads 50 \
  --timeout 10 \
  --output ./fast-crawl
```

#### Save All Pages as Files
```bash
./target/release/sitemap-crawler https://example.com/sitemap.xml \
  --save-files \
  --output ./downloaded-pages
```

## Output Format

### JSON Report Structure

When not using `--save-files`, the crawler generates a `results.json` file with the following structure:

```json
[
  {
    "url": "https://example.com/page1",
    "status_code": 200,
    "content_length": 1024,
    "mime_type": "text/html; charset=utf-8",
    "error": null
  },
  {
    "url": "https://example.com/page2",
    "status_code": 404,
    "content_length": 0,
    "mime_type": "text/html",
    "error": "Request failed: 404 Not Found"
  }
]
```

### File Naming Convention

When using `--save-files`, pages are saved with filenames derived from their URLs:
- Slashes (`/`) are replaced with underscores (`_`)
- Invalid filename characters are replaced with underscores
- Collisions are resolved by appending `_2`, `_3`, etc.

Examples:
- `https://example.com/page` → `example.com_page`
- `https://example.com/blog/post` → `example.com_blog_post`
- Collision: `example.com_page_2`

## Sitemap Support

The crawler supports:
- **XML Sitemaps**: Standard sitemap format with `<urlset>` and `<url>` elements
- **Sitemap Index**: Index files containing multiple sitemap references
- **Nested Sitemaps**: Automatically follows and processes all referenced sitemaps

## Performance Considerations

- **Thread Count**: Start with 10-20 threads and adjust based on target server capacity
- **Timeout**: Lower timeouts (5-15s) for faster crawling, higher (30-60s) for reliability
- **Memory Usage**: Scales with number of URLs and concurrent threads
- **Network**: Respects server response times and implements proper error handling

## Error Handling

The crawler handles various error conditions gracefully:
- Network timeouts and connection failures
- Invalid XML sitemap formats
- HTTP error responses (4xx, 5xx)
- File system errors when saving pages
- URL parsing errors

All errors are reported in the JSON output and logged to stderr.

## Development

### Running Tests
```bash
cargo test
```

### Development Build
```bash
cargo build
./target/debug/sitemap-crawler --help
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

## License

This software is proprietary and confidential. All rights reserved.

**Commercial License**

This software is licensed for commercial use only. Unauthorized copying, distribution, modification, or use of this software is strictly prohibited without explicit written permission from the copyright holder.

For licensing inquiries, please contact: [your-email@example.com]

Copyright © 2025 [Your Company Name]. All rights reserved.

## Contributing

[Add contribution guidelines here]
