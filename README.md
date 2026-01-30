<div align="center">
  <br>
  <a href="https://github.com/dr-montasir/crator">
      <img src="logo.svg" width="100">
  </a>
  <br>
[<img alt="github" src="https://img.shields.io/badge/github-dr%20montasir%20/%20crator-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="22">](https://github.com/dr-montasir/crator)[<img alt="crates.io" src="https://img.shields.io/crates/v/crator.svg?style=for-the-badge&color=fc8d62&logo=rust" height="22">](https://crates.io/crates/crator)[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-crator-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="22">](https://docs.rs/crator)[<img alt="license" src="https://img.shields.io/badge/license-apache_2.0-4a98f7.svg?style=for-the-badge&labelColor=555555&logo=apache" height="22">](https://choosealicense.com/licenses/apache-2.0)[<img alt="downloads" src="https://img.shields.io/crates/d/crator.svg?style=for-the-badge&labelColor=555555&logo=&color=428600" height="22">](https://crates.io/crates/crator)

  <h1>CRATOR</h1>

  <p>
    A high-performance, lightweight Rust library to fetch metadata from <a href="https://crates.io" target="_blank">crates.io</a>.
  </p>
</div>

---

## Features

- **Zero-Dependency JSON Extraction**: Custom parsing logic without `serde_json`.
- **Custom Async Runtime**: Built-in "Spin-then-Yield" executor—no `tokio` or `async-std` required.
- **Minimal Footprint**: Only one external dependency (`native-tls`) for secure HTTPS.
- **Deep Path Support**: Robust dot-notation extraction (e.g., `metadata.stats.0.count`).
- **Human-Readable Formatting**: Compacts large numbers (e.g., `56000` -> `56k`)

## About

`crator` is a high-performance utility designed for CLI tools where binary size and execution speed are critical. While most libraries rely on heavy asynchronous runtimes and full JSON serializers, `crator` achieves its minimal footprint by utilizing raw TCP/TLS streams and manual string-slice processing.

By bypassing the overhead of traditional frameworks, it offers a direct, ultra-fast path to retrieve crate metadata from [crates.io](https://crates.io), process the response, and present the data in a clean, user-friendly format.

## Installation

To include `crator` in your Rust project, run:

```shell
cargo add crator
```

Or add `crator` to your `Cargo.toml`. Since `crator` re-exports its TLS connector, no other dependencies are required.

```toml
[dependencies]
crator = "MAJOR.MINOR.PATCH"
```

## Key Components

- **`CrateInfo`**: Struct holding metadata like versions, download counts, and license info.
- **`crate_data`**: Async function that performs secure HTTPS requests to the crates.io API.
- **`Json`**: A zero-dependency utility for ultra-fast value extraction (100ns range).
- **`execute`**: A custom, lightweight "Spin-then-Yield" runtime for running futures.
- **`format_number`**: Function to convert large numbers into compact strings (e.g., `56k`).
- **`TlsConnector`**: Re-exported from `native-tls` for zero-config secure connections.
- **`Instant`**: Re-exported from `std::time` for easy high-precision benchmarking.

## Examples

```rust
use crator::{crate_data, execute};

fn main() {
    let crate_name = "mathlab";
    
    // Use the built-in lightweight executor to run the fetch
    let info = execute(async move {
        crate_data(crate_name).await
    }).expect("Failed to fetch crate data");

    println!("Latest: v{}, Downloads: {}", info.latest, info.downloads);
}
```



```rust
use crator::*;

fn main() {
    let crate_name = "fluxor";
    let start = Instant::now();

    // Work happens here...
    let info = execute(crate_data(crate_name)).expect("Failed to get crate info");

    // ...then print the timing!
    println!("🦀 Fetching [{}] done in {:?}", crate_name, start.elapsed());

    println!("Version:  v{}", info.latest);
    println!("Total:    {}", info.downloads);
} 
```



```rust
use crator::*;

fn main() {
    let crate_name = "mathlab";
    let start = Instant::now();

    // 1. Run the custom executor (This is the heavy lifting)
    let result = execute(crate_data(crate_name));

    // 2. Measure and print the timing AFTER it's done
    println!("🦀 Fetching [{}] done in {:?}", crate_name, start.elapsed());

    // 3. Match the result to display the metadata
    match result {
        Ok(info) => {
            println!("Latest:             v{}", info.latest);
            println!("Downloads:          {}", info.downloads);
            println!("Total Downloads:    {}", info.total_downloads);
            println!("Versions:           {}", info.versions);
            println!("Created At:         {}", info.created_at);
            println!("Updated At:         {}", info.updated_at);
            println!("License:            {}", info.license);
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }
}
```

## License

This project is licensed under the MIT License or Apache 2.0 License.nnector, no other dependencies are required.
