<div align="center">
    <br>
    <a href="https://github.com/dr-montasir/crator">
        <img src="logo.svg" width="100">
    </a>
    <br>
    <a href="https://github.com/dr-montasir/crator" target="_blank">
            <img alt="github" src="https://img.shields.io/badge/github-dr%20montasir%20/%20crator-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="22">
    </a>
    <a href="https://crates.io/crates/crator" target="_blank">
            <img alt="crates.io" src="https://img.shields.io/crates/v/crator.svg?style=for-the-badge&color=fc8d62&logo=rust" height="22">
    </a>
    <a href="https://docs.rs/crator" target="_blank">
            <img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-crator-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="22">
    </a>
    <a href="https://choosealicense.com/licenses/apache-2.0" target="_blank">
        <img alt="license" src="https://img.shields.io/badge/license-apache_2.0-4a98f7.svg?style=for-the-badge&labelColor=555555&logo=apache" height="22">
    </a>
    <a href="https://choosealicense.com/licenses/mit" target="_blank">
        <img alt="license" src="https://img.shields.io/badge/license-mit-4a98f7.svg?style=for-the-badge&labelColor=555555&logo=apache" height="22">
    </a>
    <a href="https://crates.io/crates/crator" target="_blank">
            <img 
                alt="downloads" 
                src="https://img.shields.io/crates/d/crator.svg?style=for-the-badge&labelColor=555555&logo=&color=428600"
                height="22"
            >
    </a>
    <a href="https://deps.rs/crate/crator" target="_blank">
            <img 
                alt="Dependency Status" 
                src="https://deps.rs/crate/crator/latest/status.svg?style=for-the-badge"
                height="22"
            >
    </a>
    <h1>CRATOR</h1>
    <p>
        <b>High-performance Rust toolkit for HTTP/HTTPS requests, JSON extraction, and environment management</b>.
    </p>
</div>

---

## What is Rust Crator?

* A lightweight, high-performance, and synchronous `HTTP/HTTPS` client for `Rust`.

* A lightweight, zero-dependency `JSON extractor` designed for maximum performance and efficiency.

* High-performance functions to `fetch` and `interact` with structured <a href="https://crates.io" target="_blank">Crates.io API</a> `metadata`.

* Panic-free `utilities` for safely loading and managing `environment variables`.

## 1. [crator::Http;](#http)

## 2. [crator::Json;](#json)

## 3. [crator::{CrateInfo, crate_data};](#crateinfo)

## 4. [crator::{get_env, get_env_or};](#env)

---

## Installation

To include crator in your Rust project, run:

```shell
cargo add crator
```

Or add crator to your `Cargo.toml`:

```toml
[dependencies]
crator = "MAJOR.MINOR.PATCH" # replace with the actual version
```

---

## Http

A lightweight, high-performance, and synchronous `HTTP/HTTPS` client for `Rust`.

### Overview

`Crator::Http` provides a simple yet powerful `API` for executing `HTTP` requests with minimal dependencies. It features connection pooling, automatic retries, native `TLS` support, and full method coverage, making it ideal for high-performance network applications.

### Key Features

- **Connection Pooling:** Reuses `TCP/TLS` streams via a global agent for reduced latency.
- **Automatic Retries:** Handles transient network failures automatically.
- **Security:** Uses system-native `TLS` backends (`SChannel, OpenSSL, Secure Transport`).
- **Full Method Support:** Supports all standard `HTTP` verbs (`GET, POST, PUT, DELETE, etc`) and WebDAV extensions (`MOVE, LOCK, etc.`).

---

### Usage Examples

#### GET Request

```rust
use crator::Http;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = Http::get("https://jsonplaceholder.typicode.com/posts/1").send("")?;
    println!("Status: {}", response.status());
    println!("Body: {}", response.body());
    Ok(())
}
```

#### POST Request with JSON Body

```rust
use crator::Http;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_body = r#"{"title": "foo", "body": "bar", "userId": 1}"#;
    let response = Http::post("https://jsonplaceholder.typicode.com/posts")
        .header("Content-Type", "application/json".to_string())
        .send(json_body)?;
    println!("Response Status: {}", response.status());
    Ok(())
}
```

#### Handling Cookies

```rust
use crator::Http;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = Http::get("https://httpbin.org")
        .cookie("session_id", "12345")
        .cookie("theme", "dark")
        .send("")?;

    for cookie in response.get_cookies() {
        println!("Set-Cookie: {}", cookie);
    }
    Ok(())
}
```

#### HTTP Request Example: Fetching Crates.io Metadata

```rust
use crator::Http;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Construct the URL exactly as requested
    let crate_name = "crator"; 
    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);

    // Execute the GET request
    let response = Http::get(&url)
        .header("User-Agent", "crator-client/0.1.0".to_string())
        .timeout(10)
        .send("")?;

    // Check status and display the raw JSON body
    if response.status() == 200 {
        println!("Successfully retrieved metadata for: {}", url);
        println!("JSON Response: {}", response.body());
    } else {
        eprintln!("Failed to fetch data. Status: {}", response.status());
    }

    Ok(())
}
```

### Features

- **Request Methods:** `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`, `CONNECT`, `TRACE`, `COPY`, `MOVE`, `MKCOL`, `PROPFIND`, `LOCK`, `UNLOCK`.
- **Custom Headers & Cookies:** Easily add headers and cookies.
- **Timeouts & Redirects:** Set request timeouts; follow redirects automatically up to 5 times.
- **Persistent Connections:** Connection reuse via global agent.

### Internal Architecture

* Uses a global `Agent` for connection pooling.
* Supports both `HTTP` and `HTTPS` protocols.
* Handles transfer encodings including `chunked` transfer.
* Retries on common transient network `errors`.

---

## Json

A lightweight, zero-dependency `JSON extractor` designed for maximum performance and efficiency.

### Overview

`Crator::Json` provides a minimal and fast way to parse, navigate, and extract data from JSON strings in Rust. It features a simple enum-based structure, recursive data traversal, and no external dependencies, making it ideal for performance-critical applications.

### Key Features

- **Zero dependencies:** No external crates required.
- **Fast parsing:** Custom recursive parser for maximum speed.
- **Flexible navigation:** Recursive search for keys within JSON structures.
- **Type-safe accessors:** Retrieve data as strings or numbers.
- **Readable output:** Pretty-print JSON with indentation.
- **Indexing support:** Access objects and arrays using `[]` syntax.

---

### Usage Examples

#### Parsing JSON

```rust
use crator::Json;

fn main() {
    let json_str = r#"{"name": "Rust", "features": ["performance", "safety"], "version": 0.8}"#;
    let parsed = Json::from_str(json_str);

    // Accessing object properties
    if let Some(name) = parsed["name"].as_str() {
        println!("Name: {}", name);
    }

    // Accessing array length
    let features = &parsed["features"];
    println!("Number of features: {}", features.len());

    // Finding nested key
    let version = parsed.find("version");
    println!("Version: {}", version.as_f64().unwrap_or(0.0));
}
```

#### Pretty Printing JSON

```rust
use crator::Json;

let json_str = r#"{"name": "Rust", "features": ["performance", "safety"], "version": 0.8}"#;
let parsed = Json::from_str(json_str);

println!("{}", parsed.pretty_print(0));
```

#### Checking for Null Values

```rust
use crator::Json;

let json_str = r#"{"name": "Rust", "features": ["performance", "safety"], "version": 0.8}"#;
let parsed = Json::from_str(json_str);

if parsed["unknown"].is_null() {
    println!("Key not found or null");
}
```

### Features

- **Recursive search:** `find()` method searches for keys at any depth.
- **Type conversion:** `as_str()`, `as_f64()` for safe type retrieval.
- **Flexible navigation:** Index objects and arrays via [].
- **Pretty print:** Human-readable `JSON output`.
- **No dependencies:** Zero external crates.

### Internal Architecture

* Uses a custom recursive parser to convert `JSON` strings into a `enum-based` structure.
* Supports `nested` objects and arrays.
* Provides efficient traversal without allocations beyond the initial parse.
* Designed for maximum `speed` and minimal memory footprint.

---

## CrateInfo

High-performance functions to `fetch` and `interact` with structured <a href="https://crates.io" target="_blank">Crates.io API</a> `metadata`.

### Overview

`CrateInfo` provides high-level utilities for retrieving and processing `crate metadata` from <a href="https://crates.io" target="_blank">crates.io</a>. It simplifies fetching crate details, parsing `JSON` responses, and formatting large numeric data into human-readable strings. Built on top of the lightweight `Http` client and `Json` parser, it offers dependency-free, fast, and reliable access to crate information.

### Key Features

- **Efficient Data Fetching:** Uses the internal `Http` client for network requests.
- **Dependency-Free JSON Parsing:** Leverages the `Json` enum for minimal overhead.
- **Readable Data Formatting:** Converts large numbers into compact formats (`k`, `M`).
- **Structured Metadata:** Provides comprehensive crate statistics and details.

---

### Usage Examples

#### Fetching Crate Metadata

```rust
use crator::crate_data;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let crate_name = "mathlab";

    let info = crate_data(crate_name)?;

    println!("-----------------------------------");
    println!("Latest:    v{}", info.latest);
    println!("Versions:  {}", info.versions);
    println!("Downloads: {}", info.downloads);
    println!("total_downloads: {}", info.total_downloads);
    println!("License:   {}", info.license);
    println!("Created:   {}", info.created_at);
    println!("Updated:   {}", info.updated_at);
    println!("-----------------------------------");

    Ok(())
}

// -----------------------------------
// Latest:    v1.5.0
// Versions:  55
// Downloads: 56.3k
// total_downloads: 56326
// License:   MIT OR Apache-2.0
// Created:   2021-02-26T19:02:59.116360Z
// Updated:   2025-10-16T20:18:58.196131Z
// -----------------------------------
```

#### Formatting Large Numbers

```rust
use crator::format_number;

fn main() {
    println!("{}", format_number(950));       // "950"
    println!("{}", format_number(1500));      // "1.5k"
    println!("{}", format_number(2_500_000)); // "2.5M"
}
```

### Features

- Fetches detailed crate `info` from <a href="https://crates.io" target="_blank">crates.io</a>
- Parses and extracts specific `metadata` fields
- Formats large numbers into human-readable strings
- Easy integration with existing Rust projects

### Internal Architecture

* Uses the internal `Http` client for network requests.
* Parses JSON responses with a dependency-free `Json` enum.
* Handles common JSON structures returned by the crates.io API.
* Formats and presents data in a user-friendly manner.

---

## ENV

Panic-free `utilities` for safely loading and managing `environment variables`.

### Overview

`ENV` provides safe, straightforward functions to load environment variables from `.env` files and retrieve them with optional fallbacks or parsing. Designed to be panic-free, it ensures robust environment management suitable for both development and production environments.

### Key Features

- **Non-panicking environment loading:** Safely loads `.env` files without panics.
- **Flexible retrieval:** Fetch environment variables with or without defaults.
- **Typed parsing:** Convert environment variables into desired types.
- **Automatic initialization:** Internal `.env` loading for convenience.
- **Error transparency:** Returns explicit errors for missing or invalid variables.

---

### Usage Examples

#### Initializing Environment Variables

```rust
use crator::init_env;

fn main() {
    if let Err(e) = init_env() {
        eprintln!("Failed to load environment variables: {}", e);
    }
}
```

#### Fetching a Required Variable

```rust
use crator::get_env;

match get_env("API_KEY") {
    Ok(api_key) => println!("API Key: {}", api_key),
    Err(e) => eprintln!("Error: {}", e),
}
```

#### Fetching with Default Value

```rust
use crator::get_env_or;

let port = get_env_or("PORT", "8080");
println!("Server running on port: {}", port);
```

#### Parsing Environment Variables into Types

```rust
use crator::get_env_parse;

let port: u16 = get_env_parse("PORT").unwrap_or(8080);
let debug_mode: bool = get_env_parse("DEBUG").unwrap_or(false);
```

#### Fetching and Parsing with Default

```rust
use crator::get_env_parse_or;

let max_connections = get_env_parse_or("MAX_CONN", 100);
```

### Features

- Load environment variables from `.env` files safely
- Retrieve variables with optional defaults
- Parse variables into common types (`u16`, `bool`, etc.)
- No panics; returns errors explicitly
- Internal `.env` loading for convenience

### Internal Architecture

* Uses `dotenvy` for loading `.env` files.
* Wraps environment variable access with error handling.
* Provides parsing into any type that implements `FromStr`.
* Designed for safe, panic-free operation.

---

## License

This project is licensed under the MIT License or Apache 2.0 License.nnector, no other dependencies are required.