#![doc(html_logo_url = "https://github.com/dr-montasir/crator/raw/HEAD/logo.svg")]
#![doc = r"<div align='center'><a href='https://github.com/dr-montasir/crator' target='_blank'><img src='https://github.com/dr-montasir/crator/raw/HEAD/logo.svg' alt='crator' width='80' height='auto' /></a><br><a href='https://github.com/dr-montasir/crator' target='_blank'>CRATOR</a><br><br><b>High-performance Rust toolkit for HTTP/HTTPS requests, JSON extraction, and environment management.</b></div>"]

#![doc = include_str!("../README.md")]

mod config;
mod http;
mod json;
mod info;

pub use config::{get_env, get_env_or, get_env_parse, get_env_parse_or, init_env};
pub use http::Http;
pub use json::Json;
pub use info::{CrateInfo, crate_data, format_number};