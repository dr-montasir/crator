#![doc(html_logo_url = "https://github.com/dr-montasir/crator/raw/HEAD/logo.svg")]
#![doc = r"<div align='center'><a href='https://github.com/dr-montasir/crator' target='_blank'><img src='https://github.com/dr-montasir/crator/raw/HEAD/logo.svg' alt='crator' width='80' height='auto' /></a><br><a href='https://github.com/dr-montasir/crator' target='_blank'>CRATOR</a><br><br><b>This library offers core functions to retrieve crate metadata from crates.io via raw TCP/TLS connections, process the JSON response, and present the data in a user-friendly format.</b></div>"]

/// This library provides asynchronous functions to fetch crate information from crates.io,
/// including the latest version and download count. It uses Tokio for asynchronous networking,
/// TLS for secure connections, and serde_json for JSON parsing.
use tokio::net::TcpStream; // Asynchronous TCP stream
use tokio_native_tls::TlsConnector; // Tokio-compatible TLS connector
use native_tls::TlsConnector as NativeTlsConnector; // Native TLS connector
use std::error::Error; // Error trait for error handling
use serde_json::Value; // JSON value type
use std::str; // String utilities

/// Represents the essential information about a crate: its latest version and total downloads.
pub struct CrateInfo {
    /// The latest version of the crate
    pub latest: String,
    /// Formatted number of downloads
    pub downloads: String,
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
        let value = (n + 500) / 1000;
        format!("{}k", value)
    } else if n < 1_000_000 {
        let value = (n + 500) / 1000;
        format!("{}k", value)
    } else {
        let value = (n + 500_000) / 1_000_000;
        format!("{}M", value)
    }
}

/// Performs an HTTP GET request over a raw TCP connection with TLS to fetch JSON data from the provided URL.
///
/// # Arguments
/// * `url` - The URL to fetch data from.
///
/// # Returns
/// * `Result<Value, Box<dyn Error>>` containing the parsed JSON response on success.
///
/// # Errors
/// Returns an error if URL parsing, network connection, TLS handshake, or JSON parsing fails.
async fn fetch_crate_data(url: &str) -> Result<Value, Box<dyn Error>> {
    let url_parts = url::Url::parse(url)?;
    let host = url_parts.host_str().ok_or("Invalid host")?;
    let port = url_parts.port_or_known_default().ok_or("Invalid port")?;
    let path = url_parts.path();

    let stream = TcpStream::connect((host, port)).await?;

    let tls_connector = TlsConnector::from(NativeTlsConnector::new()?);
    let domain = host;
    let mut tls_stream = tls_connector.connect(domain, stream).await?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: my_rust_client\r\nConnection: close\r\n\r\n",
        path, host
    );

    use tokio::io::AsyncWriteExt;
    tls_stream.write_all(request.as_bytes()).await?;

    use tokio::io::AsyncReadExt;
    let mut response = Vec::new();
    tls_stream.read_to_end(&mut response).await?;

    let response_str = String::from_utf8_lossy(&response);
    if let Some(pos) = response_str.find("\r\n\r\n") {
        let body = &response_str[pos + 4..];
        let json_value: Value = serde_json::from_str(body)?;
        Ok(json_value)
    } else {
        Err("Invalid HTTP response".into())
    }
}

/// Fetches crate data from crates.io API given a crate name.
///
/// # Arguments
/// * `crate_name` - The name of the crate to fetch info for.
///
/// # Returns
/// * `Result<CrateInfo, Box<dyn Error>>` containing the crate's latest version and formatted downloads.
///
/// # Errors
/// Returns an error if network request or JSON parsing fails.
/// 
/// # Examples
/// 
/// ## (a) Fn main {}
/// 
/// ### Example (1)
/// 
/// ```
/// use crator::crate_data;
/// use tokio::runtime::Runtime;
///
/// fn main() {
///     // Create a new Tokio runtime
///     let rt = Runtime::new().unwrap();
///     let crate_name = "crator";
///     let crate_info = rt.block_on(async {
///         crate_data(crate_name).await
///     }).unwrap();
///     println!("Latest version: {}", crate_info.latest);
///     println!("Downloads: {}", crate_info.downloads);
/// }
/// ```
/// 
/// ### Example (2)
/// 
/// ```
/// use crator::crate_data;
/// use tokio::runtime::Runtime;
///
/// fn main() {
///     // Create a new Tokio runtime
///     let rt = Runtime::new().unwrap();
///     let crate_name = "crator";
///     let crate_info = rt.block_on(async {
///         crate_data(crate_name).await
///     }).expect("Failed to get crate info");
///     println!("Latest version: {}", crate_info.latest);
///     println!("Downloads: {}", crate_info.downloads);
/// }
/// ```
/// 
/// ### Example (3)
/// 
/// ```
/// use crator::crate_data;
/// use tokio::runtime::Runtime;
///
/// fn main() {
///     // Create a new Tokio runtime
///     let rt = Runtime::new().unwrap();
///     let crate_name = "crator";
///     let crate_info = rt.block_on(async {
///         crate_data(crate_name).await
///     }).unwrap_or_else(|err| {
///         eprintln!("Error fetching crate data: {}", err);
///         std::process::exit(1);
///     });
///     println!("Latest version: {}", crate_info.latest);
///     println!("Downloads: {}", crate_info.downloads);
/// }
/// ```
/// 
/// ### Example (4)
/// 
/// ```
/// use crator::crate_data;
/// use tokio::runtime::Runtime;
///
/// fn main() {
///     // Create a new Tokio runtime
///     let rt = Runtime::new().unwrap();
///     let crate_name = "crator";
///     let crate_info = match rt.block_on(async {
///         match crate_data(crate_name).await {
///             Ok(info) => Ok(info),
///             Err(err) => {
///                 eprintln!("Error fetching crate data: {}", err);
///                 Err(err)
///             }
///         }
///     }) {
///         Ok(info) => info,
///         Err(_) => {
///             // Handle error, e.g., exit or default
///             return;
///         }
///     };
///     println!("Latest version: {}", crate_info.latest);
///     println!("Downloads: {}", crate_info.downloads);
/// }
/// ```
/// 
/// ## (b) Async fn main {}
/// 
/// ### Example (5)
/// 
/// ```
/// use crator::crate_data;
///
/// #[tokio::main]
/// async fn main() {
///     let crate_name = "crator";
///     let crate_info = crate_data(crate_name).await.unwrap();
///     println!("Latest version: {}", crate_info.latest);
///     println!("Downloads: {}", crate_info.downloads);
/// }
/// ```
/// 
/// ### Example (6)
/// 
/// ```
/// use crator::crate_data;
///
/// #[tokio::main]
/// async fn main() {
///     let crate_name = "crator";
///     let crate_info = crate_data(crate_name).await.expect("Failed to get crate info");
///     println!("Latest version: {}", crate_info.latest);
///     println!("Downloads: {}", crate_info.downloads);
/// }
/// ```
/// 
/// ### Example (7)
/// 
/// ```
/// use crator::crate_data;
///
/// #[tokio::main]
/// async fn main() {
///     let crate_name = "crator";
///     let crate_info = crate_data(crate_name).await.unwrap_or_else(|err| {
///         eprintln!("Error fetching crate data: {}", err);
///         std::process::exit(1);
///     });
///     println!("Latest version: {}", crate_info.latest);
///     println!("Downloads: {}", crate_info.downloads);
/// }
/// ```
/// 
/// ### Example (8)
/// 
/// ```
/// use crator::crate_data;
///
/// #[tokio::main]
/// async fn main() {
///     let crate_name = "crator";
///     let crate_info = match crate_data(crate_name).await {
///         Ok(info) => info,
///         Err(err) => {
///             eprintln!("Error fetching crate data: {}", err);
///             return;
///         }
///     };
///     println!("Latest version: {}", crate_info.latest);
///     println!("Downloads: {}", crate_info.downloads);
/// }
/// ```
pub async fn crate_data(crate_name: &str) -> Result<CrateInfo, Box<dyn Error>> {
    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);
    let json_value = fetch_crate_data(&url).await?;

    let latest = json_value["crate"]["max_version"]
        .as_str()
        .unwrap_or("N/A")
        .to_string();

    let downloads = json_value["crate"]["downloads"]
        .as_u64()
        .unwrap_or(0);

    Ok(CrateInfo { latest, downloads: format_number(downloads) })
}