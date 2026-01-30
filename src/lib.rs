#![doc(html_logo_url = "https://github.com/dr-montasir/crator/raw/HEAD/logo.svg")]
#![doc = r"<div align='center'><a href='https://github.com/dr-montasir/crator' target='_blank'><img src='https://github.com/dr-montasir/crator/raw/HEAD/logo.svg' alt='crator' width='80' height='auto' /></a><br><a href='https://github.com/dr-montasir/crator' target='_blank'>CRATOR</a><br><br><b>This library offers core functions to retrieve crate metadata from crates.io via raw TCP/TLS connections, process the JSON response, and present the data in a user-friendly format.</b></div>"]

//! A high-performance, lightweight library for fetching crate metadata from [crates.io](https://crates.io).
//! 
//! This library implements a custom, dependency-free asynchronous executor 
//! to manage networking. It intentionally avoids heavy runtimes like `tokio`, 
//! relying instead on the [Standard Library](https://doc.rust-lang.org) 
//! and [native-tls](https://docs.rs) for secure HTTPS connections.
//!
//! ### Key Features
//! - **Zero-Dependency JSON Extraction**: Custom parsing logic without `serde_json`.
//! - **Custom Async Runtime**: Built-in "Spin-then-Yield" executor—no `tokio` or `async-std` required.
//! - **Minimal Footprint**: Only one external dependency (`native-tls`) for secure HTTPS.
//! - **Deep Path Support**: Robust dot-notation extraction (e.g., `metadata.stats.0.count`).
//! - **Human-Readable Formatting**: Compacts large numbers (e.g., `56000` -> `56k`)

#![doc = include_str!("../README.md")]

use std::error::Error;
use std::future::Future;
use std::hint;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
pub use std::time::Instant;
use std::{thread, str, sync::Arc};
pub use native_tls::TlsConnector;

/// A lightweight, zero-dependency JSON extractor designed for maximum performance.
/// 
/// This struct provides a "fast-path" for parsing JSON response bodies without
/// the overhead of full serialization libraries like [Serde](https://serde.rs). 
/// It operates directly on string slices, making it ideal for high-speed CLI tools.
///
/// # Dependencies
/// - Uses only the Rust Standard Library (`std`).
/// - Network operations are handled by [native-tls](https://docs.rs).
pub struct Json;

impl Json {
    /// Extracts a value from a JSON string using dot-notation.
    ///
    /// This method is format-agnostic, handling both minified and "pretty-printed"
    /// JSON by ignoring whitespace and tracking bracket depth to handle nested objects.
    ///
    /// # Path Syntax
    /// - **Keys**: `metadata.version`
    /// - **Arrays**: `releases.0.v`
    ///
    /// # Performance
    /// Operates in O(N) time with minimal heap allocations. 
    ///
    /// # Returns
    /// Returns the value as an owned `String`, or `"N/A"` if the key is not found.
    /// 
    /// # Example
    /// ```rust
    /// use crator::Json;
    /// 
    /// let body = r#"{"stats": {"downloads": 56000}}"#;
    /// let val = Json::extract(body, "stats.downloads");
    /// assert_eq!(val, "56000");
    /// ```
    pub fn extract(body: &str, path: &str) -> String {
        let mut current_body = body.to_string();
        for part in path.split('.') {
            let next = if let Ok(idx) = part.parse::<usize>() {
                Self::get_array_index(&current_body, idx)
            } else {
                Self::get_key_value(&current_body, part)
            };
            if next == "N/A" { return "N/A".to_string(); }
            current_body = next;
        }
        // Auto-unquote if the final result is a string
        if current_body.starts_with('"') && current_body.ends_with('"') {
            return current_body[1..current_body.len() - 1].to_string();
        }
        current_body
    }

    fn get_key_value(body: &str, key: &str) -> String {
        let pattern = format!("\"{}\"", key);
        if let Some(key_idx) = body.find(&pattern) {
            let after_key = &body[key_idx + pattern.len()..];
            // Skip the colon and find the value
            if let Some(colon_idx) = after_key.find(':') {
                return Self::slice_until_boundary(&after_key[colon_idx + 1..]);
            }
        }
        "N/A".to_string()
    }

    fn get_array_index(body: &str, target: usize) -> String {
        let trimmed = body.trim_start();
        if !trimmed.starts_with('[') { return "N/A".to_string(); }
        let mut content = &trimmed[1..];
        for i in 0..=target {
            content = content.trim_start();
            let val = Self::slice_until_boundary(content);
            if i == target { return val; }
            let val_len = val.len();
            if val_len == 0 { break; }
            content = &content[val_len..].trim_start();
            if content.starts_with(',') { content = &content[1..]; } 
            else { break; }
        }
        "N/A".to_string()
    }

    fn slice_until_boundary(data: &str) -> String {
        let s = data.trim_start();
        if s.is_empty() { return "".to_string(); }
        let bytes = s.as_bytes();
        let (mut d_obj, mut d_arr, mut q) = (0, 0, false);
        for (i, &b) in bytes.iter().enumerate() {
            match b {
                b'"' if i == 0 || bytes[i-1] != b'\\' => q = !q,
                _ if q => continue, // Ignore everything inside quotes
                b'{' => d_obj += 1,
                b'}' => { if d_obj == 0 { return s[..i].trim().to_string(); } d_obj -= 1; }
                b'[' => d_arr += 1,
                b']' => { if d_arr == 0 { return s[..i].trim().to_string(); } d_arr -= 1; }
                b',' if d_obj == 0 && d_arr == 0 => return s[..i].trim().to_string(),
                _ if d_obj == 0 && d_arr == 0 && b.is_ascii_whitespace() && i > 0 => return s[..i].trim().to_string(),
                _ => {}
            }
        }
        s.trim_matches(|c| c == ',' || c == '}' || c == ']').trim().to_string()
    }

    /// Attempts to parse the extracted value as an `i64`. 
    /// Returns `0` if extraction or parsing fails.
    pub fn extract_int(body: &str, path: &str) -> i64 {
        Self::extract(body, path).parse::<i64>().unwrap_or(0)
    }

    /// Attempts to parse the extracted value as a `u64`. 
    /// Returns `0` if extraction or parsing fails.
    pub fn extract_u64(body: &str, path: &str) -> u64 {
        Self::extract(body, path).parse::<u64>().unwrap_or(0)
    }

    /// Attempts to parse the extracted value as an `f64`. 
    /// Returns `0.0` if extraction or parsing fails.
    pub fn extract_float(body: &str, path: &str) -> f64 {
        Self::extract(body, path).parse::<f64>().unwrap_or(0.0)
    }

    /// Attempts to parse the extracted value as a `bool`. 
    /// Returns `true` if the extracted value is "true" (case-insensitive).
    pub fn extract_bool(body: &str, path: &str) -> bool {
        Self::extract(body, path).to_lowercase() == "true"
    }
}

/// A minimal, thread-safe Waker implementation that performs no action.
/// 
/// This is used by the internal executor to satisfy the `Context` requirements 
/// of the `Future::poll` method. Since this library uses a high-performance 
/// polling strategy, a functional wake-up notification is not required.
struct NoopWake;

impl std::task::Wake for NoopWake {
    /// No-op: The executor polls continuously until completion.
    fn wake(self: Arc<Self>) {}
}

/// A high-performance, single-threaded executor for running Futures to completion.
///
/// This runtime uses a hybrid "Spin-then-Yield" strategy:
/// 1. **Spinning**: For the first 150,000 iterations, it uses [`hint::spin_loop`](https://doc.rust-lang.org) 
///    to minimize latency for near-instant tasks.
/// 2. **Yielding**: If the task remains pending, it calls [`thread::yield_now`](https://doc.rust-lang.org) 
///    to allow the OS to schedule other threads, preventing 100% CPU starvation.
///
/// # Safety
/// Uses a thread-safe [`Waker`](https://doc.rust-lang.org) backed by an `Arc<NoopWake>`, 
/// ensuring full compliance with the [Rust Future Trait](https://doc.rust-lang.org) 
/// without the overhead of a complex event loop.
pub fn execute<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    
    // Completely safe Waker using Arc
    let waker = Waker::from(Arc::new(NoopWake));
    let mut cx = Context::from_waker(&waker);
    
    let mut spins = 0u64;
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => {
                // Efficiency: Spin briefly before yielding to the OS
                if spins < 150_000 {
                    hint::spin_loop();
                    spins += 1;
                } else {
                    thread::yield_now();
                    spins = 0;
                }
            }
        }
    }
}

/// Represents the essential metadata of a crate retrieved from crates.io.
/// 
/// This structure holds both human-readable strings for display and 
/// raw numeric values for programmatic use.
pub struct CrateInfo {
    /// The latest version of the crate (e.g., "1.5.0").
    pub latest: String,
    /// Human-readable download count (e.g., "56k").
    pub downloads: String,
    /// The exact total number of downloads.
    pub total_downloads: u64,
    /// The total number of versions ever published.
    pub versions: u64,
    /// The software license (e.g., "MIT OR Apache-2.0").
    pub license: String,
    /// ISO 8601 formatted creation timestamp.
    pub created_at: String,
    /// ISO 8601 formatted timestamp of the last update.
    pub updated_at: String,
}

/// Formats large numbers into human-readable strings.
///
/// Examples:
/// - `format_number(950)` -> `"950"`
/// - `format_number(1500)` -> `"1.5k"`
/// - `format_number(10000)` -> `"10k"`
/// - `format_number(250000)` -> `"250k"`
/// - `format_number(2_500_000)` -> `"3M"`
///
/// # Arguments
/// * `n` - The number to format
///
/// # Returns
/// * A `String` representing the formatted number
pub fn format_number(n: u64) -> String {
    if n < 1000 {
        n.to_string()
    } else if n < 10_000 {
        let fractional = (n as f64 / 1000.0 * 10.0).round() / 10.0;
        if fractional.fract() > 0.0 {
            format!("{:.1}k", fractional)
        } else {
            format!("{:.0}k", fractional)
        }
    } else if n < 100_000 {
        // Simplified range: handles 10k through 999k with consistent rounding
        let value = (n + 500) / 1000;
        format!("{}k", value)
    } else if n < 1_000_000 {
        let value = (n + 500) / 1000;
        format!("{}k", value)
    } else {
        // Million range
        let value = (n + 500_000) / 1_000_000;
        format!("{}M", value)
    }
}

/// Fetches crate data from the crates.io API given a crate name.
///
/// This function performs a secure HTTPS request using `native-tls` and parses the 
/// results using a high-performance, zero-dependency JSON extractor.
///
/// # Arguments
/// * `crate_name` - The name of the crate to fetch info for (e.g., "mathlab").
///
/// # Returns
/// * `Result<CrateInfo, Box<dyn Error>>` containing the crate's metadata.
///
/// # Examples
///
/// ```rust
/// use crator::{crate_data, execute};
///
/// fn main() {
///     let crate_name = "mathlab";
///     
///     // Use the built-in lightweight executor to run the fetch
///     let info = execute(async move {
///         crate_data(crate_name).await
///     }).expect("Failed to fetch crate data");
///
///     println!("Latest: v{}, Downloads: {}", info.latest, info.downloads);
/// }
/// ```
/// 
/// ```rust
/// use crator::*;
/// 
/// fn main() {
///     let crate_name = "fluxor";
///     let start = Instant::now();
/// 
///     // Work happens here...
///     let info = execute(crate_data(crate_name)).expect("Failed to get crate info");
/// 
///     // ...then print the timing!
///     println!("🦀 Fetching [{}] done in {:?}", crate_name, start.elapsed());
/// 
///     println!("Version:  v{}", info.latest);
///     println!("Total:    {}", info.downloads);
/// } 
/// ```
/// 
/// ```rust
/// use crator::*;
/// 
/// fn main() {
///     let crate_name = "mathlab";
///     let start = Instant::now();
/// 
///     // 1. Run the custom executor (This is the heavy lifting)
///     let result = execute(crate_data(crate_name));
/// 
///     // 2. Measure and print the timing AFTER it's done
///     println!("🦀 Fetching [{}] done in {:?}", crate_name, start.elapsed());
/// 
///     // 3. Match the result to display the metadata
///     match result {
///         Ok(info) => {
///             println!("Latest:             v{}", info.latest);
///             println!("Downloads:          {}", info.downloads);
///             println!("Total Downloads:    {}", info.total_downloads);
///             println!("Versions:           {}", info.versions);
///             println!("Created At:         {}", info.created_at);
///             println!("Updated At:         {}", info.updated_at);
///             println!("License:            {}", info.license);
///         }
///         Err(e) => eprintln!("❌ Error: {}", e),
///     }
/// } 
/// ```
pub async fn crate_data(crate_name: &str) -> Result<CrateInfo, Box<dyn Error>> {
    let host = "crates.io";
    let path = format!("/api/v1/crates/{}", crate_name);

    let connector = TlsConnector::new()?;
    let stream = TcpStream::connect(format!("{}:443", host))?;
    let mut tls_stream = connector.connect(host, stream)?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: crator_safe/1.0\r\nConnection: close\r\n\r\n",
        path, host
    );

    tls_stream.write_all(request.as_bytes())?;
    let mut response = Vec::new();
    tls_stream.read_to_end(&mut response)?;

    let full_res = String::from_utf8_lossy(&response);
    let body = full_res.split("\r\n\r\n").nth(1).unwrap_or("");

    let latest = Json::extract(body, "max_version");
    let total_downloads = Json::extract_u64(body, "downloads");
    // Get total number of versions
    let versions = Json::extract_u64(body, "versions");
    let license = Json::extract(body, "license");
    let created_at = Json::extract(body, "created_at");
    let updated_at = Json::extract(body, "updated_at");

    Ok(CrateInfo { latest, downloads: format_number(total_downloads), total_downloads: total_downloads, versions: versions, license, created_at, updated_at})
}