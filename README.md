<div align="center">
  <br>
  <a href="https://github.com/dr-montasir/crator">
      <img src="logo.svg" width="100">
  </a>
  <br>

[<img alt="github" src="https://img.shields.io/badge/github-dr%20montasir%20/%20crator-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="22">](https://github.com/dr-montasir/crator)[<img alt="crates.io" src="https://img.shields.io/crates/v/crator.svg?style=for-the-badge&color=fc8d62&logo=rust" height="22">](https://crates.io/crates/crator)[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-crator-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="22">](https://docs.rs/crator)[<img alt="license" src="https://img.shields.io/badge/license-apache_2.0-4a98f7.svg?style=for-the-badge&labelColor=555555&logo=apache" height="22">](https://choosealicense.com/licenses/apache-2.0)[<img alt="downloads" src="https://img.shields.io/crates/d/crator.svg?style=for-the-badge&labelColor=555555&logo=&color=428600" height="22">](https://crates.io/crates/crator)

  <h1>CRATOR</h1>

  <p>
    This Rust library provides asynchronous functions to fetch crate information from <a href="https://crates.io" target="_blank">crates.io</a>, including the latest version and total download count. It leverages Tokio for async networking, TLS for secure connections, and serde_json for JSON parsing.
  </p>
</div>

---

## Features

- Fetch crate data asynchronously
- Proper error handling
- Human-readable number formatting
- Extensive usage examples

## About

This library offers core functions to retrieve crate metadata from crates.io via raw TCP/TLS connections, process the JSON response, and present the data in a user-friendly format.

## Installation

To include `crator` in your Rust project, run:

```shell
cargo add crator
```

**Note:** If you are creating a new project or your project does not already include `tokio` as a dependency, you also need to add `tokio` to enable asynchronous features:

```shell
cargo add tokio
```

Or try

```shell
cargo add tokio --features full
```

**This ensures that the project has the necessary asynchronous runtime support to use functions like `crate_data`.**

## Key Components

- **CrateInfo**: Struct holding the latest version and download count.
- **format_number**: Function to convert large numbers into compact, human-readable strings.
- **crate_data**: Async function to fetch and parse crate info from crates.io API.

## Example Usage

Below are various ways to call `crate_data`. These include different error handling approaches, concurrency patterns, and usage in both synchronous and asynchronous contexts.

### Example 1:  Basic usage with an explicit runtime and `unwrap()`

```rust
use crator::crate_data;
use tokio::runtime::Runtime;

fn main() {
    // Create a new Tokio runtime
    let rt = Runtime::new().unwrap();
    let crate_name = "crator";
    let crate_info = rt.block_on(async {
        crate_data(crate_name).await
    }).unwrap();
    println!(
        "Latest: v{}, Downloads: {}, Total Downloads: {}, Versions: {}, License: {}",
        crate_info.latest, crate_info.downloads, crate_info.total_downloads, crate_info.versions, crate_info.license
    );
    // Result (e.g.):
    // crate_info.latest: v0.1.0
    // crate_info.downloads: 5.9k
    // crate_info.downloads: 11
    // crate_info.versions: 1
    // crate_info.license: MIT OR Apache-2.0
    // Latest: v0.1.0, Downloads: 11, Versions: 1, License: MIT OR Apache-2.0
}
```

### Example 2:  Basic usage with an explicit runtime and `expect()`

```rust
use crator::crate_data;
use tokio::runtime::Runtime;

fn main() {
    // Create a new Tokio runtime
    let rt = Runtime::new().unwrap();
    let crate_name = "fluxor";
    let crate_info = rt.block_on(async {
        crate_data(crate_name).await
    }).expect("Failed to get crate info");
    println!("Latest version: {}", crate_info.latest);
    println!("Downloads: {}", crate_info.downloads);
    println!("Versions: {}", crate_info.versions);
    println!("Crate Health Index: {}", crate_info.total_downloads / crate_info.versions);
    println!("License: {}", crate_info.license);
}
```

### Example 3:  Basic usage with an explicit runtime and `unwrap_or_else()`

```rust
use crator::crate_data;
use tokio::runtime::Runtime;

fn main() {
    // Create a new Tokio runtime
    let rt = Runtime::new().unwrap();
    let crate_name = "serde";
    let crate_info = rt.block_on(async {
        crate_data(crate_name).await
    }).unwrap_or_else(|err| {
        eprintln!("Error fetching crate data: {}", err);
        std::process::exit(1);
    });
    println!(
        "Latest: v{}, Downloads: {}, Total Downloads: {}, Versions: {}, License: {}",
        crate_info.latest, crate_info.downloads, crate_info.total_downloads, crate_info.versions, crate_info.license
    );
}
```

### Example 4:  Basic usage with an explicit runtime and `match`

```rust
use crator::crate_data;
use tokio::runtime::Runtime;

fn main() {
    // Create a new Tokio runtime
    let rt = Runtime::new().unwrap();
    let crate_name = "tokio";
    let crate_info = match rt.block_on(async {
        match crate_data(crate_name).await {
            Ok(info) => Ok(info),
            Err(err) => {
                eprintln!("Error fetching crate data: {}", err);
                Err(err)
            }
        }
    }) {
        Ok(info) => info,
        Err(_) => {
            // Handle error, e.g., exit or default
            return;
        }
    };
    println!(
        "Latest: v{}, Downloads: {}, Total Downloads: {}, Versions: {}, License: {}",
        crate_info.latest, crate_info.downloads, crate_info.total_downloads, crate_info.versions, crate_info.license
    );
}
```

### Example 5: Basic usage with `tokio::main` and `unwrap()`

```rust
use crator::crate_data;

#[tokio::main]
async fn main() {
    let crate_name = "crator";
    let info = crate_data(crate_name).await.unwrap();
    println!(
        "Latest: v{}, Downloads: {}, Total Downloads: {}, Versions: {}, License: {}",
        info.latest, info.downloads, info.total_downloads, info.versions, info.license
    );
}
```

### Example 6: Basic usage with `tokio::main` and `expect()`

```rust
use crator::crate_data;

#[tokio::main]
async fn main() {
    let crate_name = "fluxor";
    let info = crate_data(crate_name).await.expect("Failed to fetch crate info");
    println!(
        "Latest: v{}, Downloads: {}, Total Downloads: {}, Versions: {}, License: {}",
        info.latest, info.downloads, info.total_downloads, info.versions, info.license
    );
}
```

### Example 7: Basic usage with `tokio::main` and `unwrap_or_else()`

```rust
use crator::crate_data;

#[tokio::main]
async fn main() {
    let crate_name = "serde";
    let crate_info = crate_data(crate_name).await.unwrap_or_else(|err| {
        eprintln!("Error fetching crate data: {}", err);
        std::process::exit(1);
    });
    println!(
        "Latest: v{}, Downloads: {}, Total Downloads: {}, Versions: {}, License: {}",
        crate_info.latest, crate_info.downloads, crate_info.total_downloads, crate_info.versions, , crate_info.license
    );
}
```

### Example 8: Basic usage with `tokio::main` and `match`

```rust
use crator::crate_data;

#[tokio::main]
async fn main() {
    let crate_name = "tokio";
    let crate_info = match crate_data(crate_name).await {
        Ok(info) => info,
        Err(err) => {
            eprintln!("Error fetching crate data: {}", err);
            return;
        }
    };
    println!(
        "Latest: v{}, Downloads: {}, Total Downloads: {}, Versions: {}, License: {} Created At: {}, Updated At: {}", 
        crate_info.latest, crate_info.downloads, crate_info.total_downloads, crate_info.versions, crate_info.license, crate_info.created_at, crate_info.updated_at
    );
}
```

## License

This project is licensed under the MIT License or Apache 2.0 License.