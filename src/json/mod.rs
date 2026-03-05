//! # Json Module
//!
//! A lightweight, zero-dependency JSON extractor designed for maximum performance.
//! This module provides the [`Json`] enum to navigate and extract data from JSON strings.

use std::collections::HashMap;
use std::ops::Index;

/// Represents any valid JSON value.
/// 
/// This enum allows for recursive navigation of JSON objects and arrays.
#[derive(Debug, Clone)]
pub enum Json {
    Object(HashMap<String, Json>),
    Array(Vec<Json>),
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

impl Json {
    /// Returns `true` if the value is Null.
    pub fn is_null(&self) -> bool { matches!(self, Json::Null) }

    /// Returns the number of elements in an Array or keys in an Object.
    pub fn len(&self) -> usize {
        match self {
            Json::Array(v) => v.len(),
            Json::Object(m) => m.len(),
            _ => 0,
        }
    }

    /// Returns the raw string value, or a default string if the value is Null.
    ///
    /// # Example
    /// ```
    /// use crator::Json;
    /// let v = Json::from_str(r#"{"name": "Rust"}"#);
    /// assert_eq!(v["name"].unwrap_or("Unknown"), "Rust");
    /// assert_eq!(v["age"].unwrap_or("0"), "0");
    /// ```
    pub fn unwrap_or(&self, default: &str) -> String {
        if self.is_null() { default.to_string() } else { self.to_string_raw() }
    }

    /// Recursively searches for a key anywhere in the JSON structure.
    pub fn find(&self, key: &str) -> &Json {
        static NULL: Json = Json::Null;
        match self {
            Json::Object(m) => {
                if let Some(v) = m.get(key) { return v; }
                for v in m.values() {
                    let found = v.find(key);
                    if !found.is_null() { return found; }
                }
                &NULL
            }
            Json::Array(a) => {
                for v in a {
                    let found = v.find(key);
                    if !found.is_null() { return found; }
                }
                &NULL
            }
            _ => &NULL,
        }
    }

    /// Parses a JSON string into a `Json` enum.
    pub fn from_str(input: &str) -> Self {
        let mut chars = input.chars().peekable();
        Self::parse(&mut chars)
    }

    /// Returns the value as a string slice if it is a `Json::String`.
    pub fn as_str(&self) -> Option<&str> {
        if let Json::String(s) = self { Some(s) } else { None }
    }

    /// Returns the value as an f64 if it is a `Json::Number`.
    pub fn as_f64(&self) -> Option<f64> {
        if let Json::Number(n) = self { Some(*n) } else { None }
    }

    fn parse(chars: &mut std::iter::Peekable<std::str::Chars>) -> Self {
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() || c == ',' || c == ':' { chars.next(); } else { break; }
        }
        match chars.next() {
            Some('{') => {
                let mut map = HashMap::new();
                while let Some(&c) = chars.peek() {
                    if c == '}' { chars.next(); break; }
                    let key = match Self::parse(chars) { Json::String(s) => s, _ => String::new() };
                    map.insert(key, Self::parse(chars));
                }
                Json::Object(map)
            }
            Some('[') => {
                let mut vec = Vec::new();
                while let Some(&c) = chars.peek() {
                    if c == ']' { chars.next(); break; }
                    vec.push(Self::parse(chars));
                }
                Json::Array(vec)
            }
            Some('"') => {
                let mut s = String::new();
                while let Some(c) = chars.next() {
                    if c == '\\' { if let Some(esc) = chars.next() { s.push(esc); } }
                    else if c == '"' { break; }
                    else { s.push(c); }
                }
                Json::String(s)
            }
            Some('t') => { for _ in 0..3 { chars.next(); } Json::Bool(true) }
            Some('f') => { for _ in 0..4 { chars.next(); } Json::Bool(false) }
            Some('n') => { for _ in 0..3 { chars.next(); } Json::Null }
            Some(c) if c.is_digit(10) || c == '-' => {
                let mut s = c.to_string();
                while let Some(&next) = chars.peek() {
                    if next.is_digit(10) || next == '.' || next == 'e' || next == 'E' { 
                        s.push(chars.next().unwrap()); 
                    } else { break; }
                }
                Json::Number(s.parse().unwrap_or(0.0))
            }
            _ => Json::Null,
        }
    }

    /// Formats the JSON into a human-readable string with indentation.
    pub fn pretty_print(&self, indent: usize) -> String {
        let space = " ".repeat(indent);
        let inner_space = " ".repeat(indent + 4);
        match self {
            Json::Object(obj) => {
                if obj.is_empty() { return "{}".to_string(); }
                let mut items: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}\"{}\": {}", inner_space, k, v.pretty_print(indent + 4)))
                    .collect();
                items.sort();
                format!("{{\n{}\n{}}}", items.join(",\n"), space)
            }
            Json::Array(arr) => {
                if arr.is_empty() { return "[]".to_string(); }
                let items: Vec<String> = arr.iter()
                    .map(|v| format!("{}{}", inner_space, v.pretty_print(indent + 4)))
                    .collect();
                format!("[\n{}\n{}]", items.join(",\n"), space)
            }
            Json::String(s) => format!("\"{}\"", s),
            _ => self.to_string_raw(),
        }
    }

    /// Returns a string representation without the "Json::" enum wrappers.
    pub fn to_string_raw(&self) -> String {
        match self {
            Json::String(s) => s.clone(),
            Json::Number(n) => if n.fract() == 0.0 { (*n as i64).to_string() } else { n.to_string() },
            Json::Bool(b) => b.to_string(),
            Json::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string_raw_json()).collect();
                format!("[{}]", items.join(","))
            },
            Json::Object(obj) => {
                let mut pairs: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, v.to_string_raw_json()))
                    .collect();
                pairs.sort();
                format!("{{{}}}", pairs.join(","))
            },
            Json::Null => "null".to_string(),
        }
    }

    fn to_string_raw_json(&self) -> String {
        match self {
            Json::String(s) => format!("\"{}\"", s),
            _ => self.to_string_raw(),
        }
    }
}

impl Index<&str> for Json {
    type Output = Json;
    fn index(&self, key: &str) -> &Self::Output {
        static NULL: Json = Json::Null;
        if let Json::Object(m) = self { m.get(key).unwrap_or(&NULL) } else { &NULL }
    }
}

impl Index<usize> for Json {
    type Output = Json;
    fn index(&self, index: usize) -> &Self::Output {
        static NULL: Json = Json::Null;
        if let Json::Array(a) = self { a.get(index).unwrap_or(&NULL) } else { &NULL }
    }
}