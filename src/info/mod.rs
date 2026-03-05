//! # Info Module
//!
//! Provides high-level functions to interact with the Crates.io API.
//!
//! This module handles fetching crate metadata, parsing the response into the 
//! [`CrateInfo`] struct, and formatting numeric data for human readability.
//!
//! It utilizes the internal [`Http`] client for networking and the [`Json`] 
//! parser for dependency-free data extraction.

use crate::{Http, Json};

/// Represents detailed statistics and metadata for a specific crate.
pub struct CrateInfo {
    /// The most recent version number (e.g., "1.0.2").
    pub latest: String,
    /// A formatted string of downloads (e.g., "1.5k" or "2.3M").
    pub downloads: String,
    /// The exact numeric count of total downloads.
    pub total_downloads: u64,
    /// The total number of versions published to crates.io.
    pub versions: u64,
    /// The license identifier (e.g., "MIT" or "Apache-2.0").
    pub license: String,
    /// The date the crate was first created.
    pub created_at: String,
    /// The date of the last metadata update.
    pub updated_at: String,
}

/// Formats large numbers into a human-readable "k" or "M" string.
///
/// # Examples
/// ```
/// use crator::format_number;
/// assert_eq!(format_number(999), "999");
/// assert_eq!(format_number(1500), "1.5k");
/// assert_eq!(format_number(2500000), "2.5M");
/// ```
pub fn format_number(n: u64) -> String {
    if n < 1000 {
        n.to_string()
    } else if n < 1_000_000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    }
}

/// Fetches crate metadata from the crates.io API.
///
/// # Errors
/// Returns an error if the network request fails, the User-Agent is rejected,
/// or the API returns an invalid JSON response.
///
/// # Example
/// ```no_run
/// use crator::crate_data;
/// 
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let info = crate_data("serde")?;
///     println!("Latest: v{}", info.latest);
///     Ok(())
/// }
/// ```
pub fn crate_data(crate_name: &str) -> Result<CrateInfo, Box<dyn std::error::Error>> {
    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);
    
    let res = Http::get(&url)
        .header("User-Agent", "crator-client/0.1.0".to_string())
        .send("")?;

    // Parse the JSON once
    let root = Json::from_str(res.body());

    // Use the "crate" object from the response
    let metadata = &root["crate"];

    let total_dl = metadata["downloads"].as_f64().unwrap_or(0.0) as u64;

    let license = root["versions"][0]["license"].unwrap_or("Unknown");

    Ok(CrateInfo {
        latest: metadata["max_version"].to_string_raw(),
        downloads: format_number(total_dl),
        total_downloads: total_dl,
        versions: metadata["versions"].len() as u64,
        license: license, 
        created_at: metadata["created_at"].to_string_raw(),
        updated_at: metadata["updated_at"].to_string_raw(),
    })
}