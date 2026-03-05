//! # Crator::Http
//! 
//! A lightweight, high-performance, and synchronous HTTP/HTTPS client for Rust.
//! 
//! ## Overview
//! This module provides a fluent API for executing HTTP requests with minimal dependencies.
//! Key features include:
//! * **Connection Pooling**: Reuses TCP/TLS streams via a global agent to reduce latency.
//! * **Automatic Retries**: Heals broken or timed-out connections automatically.
//! * **Security**: Utilizes native system TLS backends (SChannel, OpenSSL, or Secure Transport).
//! * **Full Method Support**: Includes standard verbs (GET, POST) and WebDAV extensions (MOVE, LOCK).
//! 
//! ## Example Usage
//! 
//! ```rust,no_run
//! use crator::Http;
//! 
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let response = Http::post("https://api.example.com")
//!         .header("Content-Type", "application/json".to_string())
//!         .timeout(5)
//!         .send("{\"key\": \"value\"}")?;
//! 
//!     if response.status() == 200 {
//!         println!("Body: {}", response.body());
//!     }
//!     Ok(())
//! }
//! ```
//! 
//! ## Examples
//! 
//! ### GET Request - Fetching a Resource
//! ```rust,no_run
//! use crator::Http;
//! 
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Fetches a sample post from JSONPlaceholder
//!     let response = Http::get("https://jsonplaceholder.typicode.com/posts/1").send("")?;
//! 
//!     if response.status() == 200 {
//!         println!("Post Body: {}", response.body());
//!     }
//!     Ok(())
//! }
//! ```
//! 
//! ### POST Request - Creating Data
//! ```rust,no_run
//! use crator::Http;
//! 
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let json_body = r#"{"title": "foo", "body": "bar", "userId": 1}"#;
//!     let response = Http::post("https://jsonplaceholder.typicode.com/posts")
//!         .header("Content-Type", "application/json".to_string())
//!         .send(json_body)?;
//! 
//!     println!("Created ID: {}", response.status()); // Should be 201 Created
//!     Ok(())
//! }
//! ```
//! 
//! ### PUT Request - Updating a User
//! ```rust,no_run
//! use crator::Http;
//! 
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let update_data = r#"{"name": "morpheus", "job": "zion resident"}"#;
//!     let response = Http::put("https://reqres.in")
//!         .header("Content-Type", "application/json".to_string())
//!         .send(update_data)?;
//! 
//!     println!("Update Status: {}", response.status());
//!     Ok(())
//! }
//! ```
//! 
//! ### DELETE Request - Removing a Resource
//! ```rust,no_run
//! use crator::Http;
//! 
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let response = Http::delete("https://jsonplaceholder.typicode.com/posts/1").send("")?;
//!     println!("Deleted Status: {}", response.status()); // Should be 200 OK
//!     Ok(())
//! }
//! ```
//! 
//! ### Cookie Management - Sending and Receiving
//! 
//! This example demonstrates sending a cookie to a server and then inspecting 
//! the `Set-Cookie` headers returned by the response.
//! 
//! ```rust,no_run
//! use crator::Http;
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. Sending a cookie via the fluent API
//!     let response = Http::get("https://httpbin.org")
//!         .cookie("session_id", "12345")
//!         .cookie("theme", "dark")
//!         .send("")?;
//! 
//!     // 2. Retrieving cookies from a response (e.g., from a login endpoint)
//!     let login_res = Http::get("https://httpbin.org/set?auth=abc").send("")?;
//! 
//!     for cookie in login_res.get_cookies() {
//!         println!("Server set cookie: {}", cookie);
//!     }
//!     Ok(())
//! }
//! ```

// use std::io::{BufReader, Read, Write, ErrorKind};
use std::io::{Read, Write, ErrorKind};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::sync::{Mutex, LazyLock};
use native_tls::TlsConnector;

/// Global storage for persistent connections to enable pooling.
static GLOBAL_AGENT: LazyLock<Mutex<Agent>> = LazyLock::new(|| Mutex::new(Agent::new()));

/// Trait alias for streams supporting both Read and Write operations.
pub trait ReadWrite: Read + Write + Send {}
impl<T: Read + Write + Send> ReadWrite for T {}

/// Main entry point for creating HTTP requests.
pub struct Http;
impl Http {
    pub fn get(url: &str) -> RequestBuilder { RequestBuilder::new("GET", url) }
    pub fn post(url: &str) -> RequestBuilder { RequestBuilder::new("POST", url) }
    pub fn put(url: &str) -> RequestBuilder { RequestBuilder::new("PUT", url) }
    pub fn patch(url: &str) -> RequestBuilder { RequestBuilder::new("PATCH", url) }
    pub fn delete(url: &str) -> RequestBuilder { RequestBuilder::new("DELETE", url) }
    pub fn head(url: &str) -> RequestBuilder { RequestBuilder::new("HEAD", url) }
    pub fn options(url: &str) -> RequestBuilder { RequestBuilder::new("OPTIONS", url) }
    pub fn connect(url: &str) -> RequestBuilder { RequestBuilder::new("CONNECT", url) }
    pub fn trace(url: &str) -> RequestBuilder { RequestBuilder::new("TRACE", url) }
    pub fn copy(url: &str) -> RequestBuilder { RequestBuilder::new("COPY", url) }
    pub fn mov(url: &str) -> RequestBuilder { RequestBuilder::new("MOVE", url) }
    pub fn mkcol(url: &str) -> RequestBuilder { RequestBuilder::new("MKCOL", url) }
    pub fn propfind(url: &str) -> RequestBuilder { RequestBuilder::new("PROPFIND", url) }
    pub fn lock(url: &str) -> RequestBuilder { RequestBuilder::new("LOCK", url) }
    pub fn unlock(url: &str) -> RequestBuilder { RequestBuilder::new("UNLOCK", url) }
}

/// Managed connection state for a specific host.
pub struct Agent {
    stream: Option<Box<dyn ReadWrite>>,
    host: String,
}
impl Agent { fn new() -> Self { Self { stream: None, host: String::new() } } }

/// Data structure containing the raw HTTP response.
pub struct Response {
    pub raw: String,
}

impl Response {
    /// Returns the HTTP status code (e.g., 200, 404).
    pub fn status(&self) -> u16 {
        self.raw.lines().next()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// Returns the body portion of the HTTP response.
    pub fn body(&self) -> &str {
        self.raw.split("\r\n\r\n").nth(1).unwrap_or("")
    }

    /// Retrieves the value of a specific header if present.
    pub fn get_header(&self, name: &str) -> Option<&str> {
        let prefix = format!("{}: ", name).to_lowercase();
        self.raw.lines()
            .find(|l| l.to_lowercase().starts_with(&prefix))
            .map(|l| l[prefix.len()..].trim())
    }

    /// Extracts all `Set-Cookie` headers into a vector of strings.
    pub fn get_cookies(&self) -> Vec<String> {
        self.raw.lines()
            .filter(|l| l.to_lowercase().starts_with("set-cookie: "))
            .map(|l| l["set-cookie: ".len()..].split(';').next().unwrap_or("").to_string())
            .collect()
    }
}

/// Fluent builder for configuring and sending HTTP requests.
pub struct RequestBuilder {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    timeout: u64,
    redirects: u8,
}

impl RequestBuilder {
    fn new(method: &str, url: &str) -> Self {
        Self { 
            method: method.to_string(), 
            url: url.to_string(), 
            headers: Vec::new(),
            timeout: 10,
            redirects: 0,
        }
    }

    /// Adds a custom header to the request.
    pub fn header(mut self, key: &str, value: String) -> Self {
        self.headers.push((key.to_string(), value));
        self
    }

    /// Appends a `Cookie` header to the request.
    pub fn cookie(self, name: &str, value: &str) -> Self {
        self.header("Cookie", format!("{}={}", name, value))
    }

    /// Overrides the default 10-second timeout.
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout = seconds;
        self
    }

    /// Executes the request and returns a `Response`. Automatically retries on network failures.
    pub fn send(mut self, body: &str) -> Result<Response, Box<dyn std::error::Error>> {
        // Attempt 1
        let res_raw = match self.execute(body) {
            Ok(raw) => raw,
            Err(e) => {
                // On ANY error, clear the stream so the retry is 100% fresh
                if let Ok(mut agent) = GLOBAL_AGENT.lock() {
                    agent.stream = None; 
                }

                if is_retryable(&e) {
                    // Attempt 2 (Retry)
                    self.execute(body)?
                } else {
                    return Err(e);
                }
            }
        };

        let response = Response { raw: res_raw };

        // Handle Redirects
        let status = response.status();
        if (status >= 301 && status <= 308) && self.redirects < 5 {
            if let Some(new_url) = response.get_header("Location") {
                self.url = new_url.to_string();
                self.redirects += 1;
                return self.send(body); 
            }
        }

        Ok(response)
    }

    /// Core execution logic for dispatching the HTTP request.
    ///
    /// This method performs several low-level operations:
    /// 1.  **URL Parsing**: Extracts the host, path, and protocol (HTTP vs HTTPS).
    /// 2.  **Connection Management**: Checks the `GLOBAL_AGENT` for an existing 
    ///     warm connection. If the host has changed or the stream is empty, it 
    ///     performs DNS resolution and establishes a new TCP/TLS session.
    /// 3.  **Timeout Application**: Sets the socket-level connect and read 
    ///     timeouts based on the `RequestBuilder` configuration.
    /// 4.  **Request Formatting**: Serializes the method, path, and headers 
    ///     into a valid HTTP/1.1 wire format.
    /// 5.  **Transmission**: Writes the request head and body to the stream.
    /// 6.  **Header Processing**: Reads from the stream until the `\r\n\r\n` 
    ///     boundary is identified, extracting headers for transfer metadata.
    /// 7.  **Adaptive Body Ingestion**: 
    ///     - **Chunked**: Decodes `Transfer-Encoding: chunked` streams by 
    ///       iteratively reading hex-sized blocks.
    ///     - **Fixed-Length**: Reads exactly the number of bytes specified 
    ///       by the `Content-Length` header.
    ///     - **Stream**: Continues reading until the connection is closed 
    ///       if no length metadata is present.
    ///
    /// # Errors
    /// Returns `UnexpectedEof` if the connection drops prematurely or 
    /// `InvalidData` if headers exceed security limits or chunking is malformed.
    fn execute(&self, body: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Acquire lock on global agent to manage connection
        let mut agent = GLOBAL_AGENT.lock().map_err(|_| "Mutex Lock Failed")?;
        
        // Determine if protocol is HTTPS
        let is_https = self.url.starts_with("https://");
        
        // Clean URL to extract host and path
        let url_clean = self.url.replace("https://", "").replace("http://", "");
        let (host, path_part) = url_clean.split_once('/').unwrap_or((&url_clean, ""));
        let path = if path_part.is_empty() { "" } else { path_part };
        
        // Set default port based on protocol
        let port = if is_https { 443 } else { 80 };

        // Establish new connection if needed
        if agent.stream.is_none() || agent.host != host {
            let addr = format!("{}:{}", host, port).to_socket_addrs()?.next().ok_or("DNS Fail")?;
            let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(5))?;
            tcp.set_read_timeout(Some(Duration::from_secs(self.timeout)))?;
            
            // Wrap with TLS if HTTPS
            let stream: Box<dyn ReadWrite> = if is_https {
                let connector = TlsConnector::new()?;
                Box::new(connector.connect(host, tcp)?)
            } else {
                Box::new(tcp)
            };
            agent.stream = Some(stream);
            agent.host = host.to_string();
        }

        // Obtain mutable reference to the stream
        let stream = agent.stream.as_mut().unwrap();

        // Build HTTP request string
        let mut req_str = format!("{} /{} HTTP/1.1\r\nHost: {}\r\nConnection: keep-alive\r\n", self.method, path, host);
        for (k, v) in &self.headers {
            req_str.push_str(&format!("{}: {}\r\n", k, v));
        }
        if !body.is_empty() {
            req_str.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }
        req_str.push_str("\r\n");

        // Send request headers
        stream.write_all(req_str.as_bytes())?;
        // Send request body if present
        if !body.is_empty() {
            stream.write_all(body.as_bytes())?;
        }

        // Buffer for reading from stream
        let mut buffer = Vec::new();
        // String to accumulate headers content
        let mut temp_buf = [0u8; 1024];

        // Read headers
        let (headers_part, body_bytes) = loop {
            let n = stream.read(&mut temp_buf)?;
            if n == 0 {
                return Err(Box::new(std::io::Error::new(ErrorKind::UnexpectedEof, "Connection closed while reading headers")));
            }
            buffer.extend_from_slice(&temp_buf[..n]);
            
            // Convert buffer to string for header parsing
            let current_str = String::from_utf8_lossy(&buffer); // Update headers_str
            
            // Check if headers end is found
            if let Some(idx) = current_str.find("\r\n\r\n") {
                let end_idx = idx + 4;
                let headers = current_str[..end_idx].to_string();
                let body = buffer.split_off(end_idx);
                break (headers, body);
            }
            // Security limit for headers to prevent attacks
            if buffer.len() > 16384 {
                return Err(Box::new(std::io::Error::new(ErrorKind::InvalidData, "Headers too large")));
            }
        };

        // Parse headers for transfer info
        let mut content_length: Option<usize> = None;
        let mut is_chunked = false;

        for line in headers_part.lines() {
            let line_lower = line.to_lowercase();
            if let Some(val) = line_lower.strip_prefix("content-length:") {
                content_length = val.trim().parse().ok();
            } else if line_lower.starts_with("transfer-encoding") && line_lower.contains("chunked") {
                is_chunked = true;
            }
        }

        // Handle response body based on transfer encoding
        let mut full_body = Vec::new();
        if is_chunked {
            // For chunked transfer encoding, implement chunk decoding
            // Chain the remaining body bytes and stream for reading chunks
            let mut combined_reader = body_bytes.chain(stream);
            loop {
                // Read chunk size line
                let mut line = String::new();
                let mut b = [0u8; 1];
                // Read until CRLF
                while combined_reader.read_exact(&mut b).is_ok() {
                    line.push(b[0] as char);
                    if line.ends_with("\r\n") { break; }
                }
                
                let size_str = line.trim();
                if size_str.is_empty() { break; }
                let chunk_size = usize::from_str_radix(size_str, 16).unwrap_or(0);
                if chunk_size == 0 { break; }

                // Read chunk data
                let mut chunk_buf = vec![0u8; chunk_size];
                combined_reader.read_exact(&mut chunk_buf)?;
                full_body.extend_from_slice(&chunk_buf);
                
                // Read trailing CRLF after chunk
                let mut crlf = [0u8; 2];
                combined_reader.read_exact(&mut crlf)?;
            }
        } else if let Some(cl) = content_length {
            // Read fixed length body
            full_body = body_bytes;
            while full_body.len() < cl {
                let n = stream.read(&mut temp_buf)?;
                if n == 0 { break; }
                full_body.extend_from_slice(&temp_buf[..n]);
            }
        } else {
            full_body = body_bytes;
            loop {
                let n = stream.read(&mut temp_buf)?;
                if n == 0 { break; }
                full_body.extend_from_slice(&temp_buf[..n]);
            }
        }

        // Compose full response string
        let mut full_response = headers_part;
        full_response.push_str(&String::from_utf8_lossy(&full_body));
        Ok(full_response)
    }
}

/// Evaluates if an error is transient and suitable for an automatic retry.
fn is_retryable(e: &Box<dyn std::error::Error>) -> bool {
    if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
        return matches!(
            io_err.kind(), 
            ErrorKind::BrokenPipe | 
            ErrorKind::ConnectionAborted | 
            ErrorKind::ConnectionReset |
            ErrorKind::UnexpectedEof | 
            ErrorKind::TimedOut // catches OS Error 10060
        );
    }
    false
}