//! # Configuration Module
//! 
//! This module provides panic-free utilities for managing environment variables 
//! using the [dotenvy](https://docs.rs) crate. 
//!
//! Functionality includes loading `.env` files into the system environment and 
//! retrieving values with optional fallbacks.

use dotenvy::dotenv;
use std::env;
use std::error::Error;

/// Initializes the environment by loading the `.env` file into `std::env`.
///
/// This function attempts to load variables from a `.env` file in the current or parent 
/// directories into the system environment. The absence of a `.env` file is ignored 
/// to support production environments where variables are set via the operating system.
///
/// # Returns
/// * `Ok(())` - Successfully processed the environment (even if no file was found).
/// * `Err(Box<dyn Error>)` - If a `.env` file exists but is corrupted or unreadable.
///
/// # Example
/// ```
/// use crator::init_env;
/// 
/// if let Err(e) = init_env() {
///     eprintln!("Initialization failed: {}", e);
/// }
/// ```
pub fn init_env() -> Result<(), Box<dyn Error>> {
    let _ = dotenv(); 
    Ok(())
}

/// Retrieves an environment variable by key.
///
/// This provides a safe interface for variables that must exist for the 
/// application to function, such as database credentials or API keys.
///
/// # Errors
/// Returns [std::env::VarError] if:
/// * The variable is not present in the environment.
/// * The variable contains invalid Unicode data.
/// 
/// # Example
/// ```
/// use crator::get_env;
/// 
/// match get_env("DATABASE_URL") {
///     Ok(val) => println!("Database URL: {}", val),
///     Err(e) => eprintln!("Missing required variable: {}", e),
/// }
/// ```
pub fn get_env(key: &str) -> Result<String, env::VarError> {
    env::var(key)
}

/// Retrieves an environment variable or returns a provided default value.
///
/// This function is intended for non-critical configuration settings. It attempts 
/// to load the `.env` file internally to ensure local variables are available if 
/// [init_env] was not previously called.
///
/// # Note
/// While this function handles internal initialization, calling [init_env] once 
/// at application startup is more efficient for multiple lookups to avoid 
/// repeated file system operations.
///
/// # Example
/// ```
/// use crator::get_env_or;
/// 
/// // Returns the String "8080" if the "PORT" environment variable is missing
/// let port = get_env_or("PORT", "8080");
///
/// // Returns the String "info" if the "LOG_LEVEL" environment variable is missing
/// let level = get_env_or("LOG_LEVEL", "info");
/// ```
pub fn get_env_or(key: &str, default: &str) -> String {
    let _ = dotenv(); 
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Retrieves an environment variable and attempts to parse it into the requested type.
///
/// This function attempts to load the `.env` file internally if it has not been 
/// loaded yet.
///
/// # Errors
/// Returns an error if:
/// * The variable is not present.
/// * The variable contains invalid Unicode.
/// * The value cannot be parsed into type `T`.
///
/// # Example
/// ```
/// use crator::get_env_parse;
/// 
/// let port: u16 = get_env_parse("PORT").unwrap_or(8080);
/// let debug_mode: bool = get_env_parse("DEBUG").unwrap_or(false);
/// ```
pub fn get_env_parse<T>(key: &str) -> Result<T, Box<dyn Error>> 
where 
    T: std::str::FromStr,
    T::Err: Error + 'static,
{
    let _ = dotenv(); 
    let val = env::var(key)?;
    let parsed = val.parse::<T>()?;
    Ok(parsed)
}

/// Retrieves an environment variable and parses it, returning a default value on failure.
///
/// This function attempts to load the `.env` file internally if it has not been 
/// loaded yet.
///
/// # Example
/// ```
/// use crator::get_env_parse_or;
/// let port = get_env_parse_or("PORT", 8080);
/// ```
pub fn get_env_parse_or<T>(key: &str, default: T) -> T 
where 
    T: std::str::FromStr,
{
    let _ = dotenv(); 

    env::var(key)
        .ok()
        .and_then(|val| val.parse::<T>().ok())
        .unwrap_or(default)
}