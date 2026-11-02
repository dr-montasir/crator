//! # Rules Module
//!
//! High-performance, zero-dependency `rsj!` macro for declarative, 
//! JSX-like JSON generation with support for loops and conditionals.
//! 
//! This module provides the recursive "munching" logic to produce declarative, JSX-like 
//! JSON structures. It supports various indentation styles, conditional logic, and 
//! pattern-based iteration.

/// Formats a JSON key-value pair with leading indentation.
/// 
/// Automatically detects raw types (booleans, null, numbers) and nested 
/// structures (objects/arrays) to avoid redundant quoting of values.
pub fn format_json_field(k: &str, v: &str, indent: &str) -> String {
    let key = k.trim_matches('"');
    let val_str = v.trim();

    // 1. Determine if it's a raw type (bool, null, number, or nested)
    let is_raw = val_str == "true" || val_str == "false" || val_str == "null" || 
                 val_str.parse::<f64>().is_ok() || 
                 (val_str.starts_with('{') && val_str.ends_with('}')) || 
                 (val_str.starts_with('[') && val_str.ends_with(']'));

    // 2. Format
    let formatted = if is_raw { 
        format!("\"{}\": {}", key, val_str) 
    } else { 
        // Handles empty strings by wrapping them in quotes
        format!("\"{}\": \"{}\"", key, val_str) 
    };
    
    format!("{}{}", indent, formatted)
}

// Utils:

/// Escapes a raw string slice into a valid JSON string payload.
///
/// This function acts like the string serialization layer of JavaScript's `JSON.stringify()`.
/// It converts control characters (`\n`, `\t`, `\r`), inner quotes (`"`), and backslashes (`\`)
/// into their safe JSON escape sequences.
///
/// ### When to use:
/// Use this whenever you want to embed raw, multi-line blocks of code (JavaScript, HTML, GLSL, Bash scripts) 
/// as a value inside an `rsj!` object or array structure.
///
/// ### Performance:
/// Allocates a single `String` with pre-allocated capacity equal to the input length to minimize re-allocations.
pub fn to_str(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    
    for c in input.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            // Handle control characters if present
            _ if c.is_ascii_control() => {
                escaped.push_str(&format!("\\u{:04x}", c as u32));
            }
            _ => escaped.push(c),
        }
    }
    
    escaped
}

/// Unescapes a JSON-escaped string back into its original raw plaintext representation.
///
/// This function acts like the string deserialization layer of JavaScript's `JSON.parse()`.
/// It handles standard JSON escape codes (`\"`, `\\`, `\/`, `\b`, `\f`, `\n`, `\r`, `\t`) 
/// and fully decodes hex-encoded unicode escape characters (`\uXXXX`).
///
/// ### When to use:
/// Use this when pulling an embedded script or complex text block out of a processed JSON string
/// to restore it back into its raw, runnable layout.
pub fn to_raw(input: &str) -> String {
    let mut unescaped = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next_c) = chars.peek() {
                match next_c {
                    '"'  => { unescaped.push('"'); chars.next(); }
                    '\\' => { unescaped.push('\\'); chars.next(); }
                    '/'  => { unescaped.push('/'); chars.next(); }
                    'b'  => { unescaped.push('\x08'); chars.next(); }
                    'f'  => { unescaped.push('\x0c'); chars.next(); }
                    'n'  => { unescaped.push('\n'); chars.next(); }
                    'r'  => { unescaped.push('\r'); chars.next(); }
                    't'  => { unescaped.push('\t'); chars.next(); }
                    'u'  => {
                        chars.next(); // consume 'u'
                        // Extract next 4 hex hex-digits
                        let mut hex_str = String::new();
                        for _ in 0..4 {
                            if let Some(hc) = chars.next() {
                                hex_str.push(hc);
                            }
                        }
                        if let Ok(code_point) = u32::from_str_radix(&hex_str, 16) {
                            if let Some(decoded_char) = std::char::from_u32(code_point) {
                                unescaped.push(decoded_char);
                            }
                        }
                    }
                    _ => unescaped.push('\\'), // Standalone backslash if unmapped
                }
            } else {
                unescaped.push('\\');
            }
        } else {
            unescaped.push(c);
        }
    }

    unescaped
}

/// Wraps a string as a JSON string.
///
/// The value is escaped using [`to_str`] before being surrounded with double
/// quotes.
///
/// # Examples
///
/// ```
/// use crator::quote_json_string;
/// let json = quote_json_string("hello");
/// assert_eq!(json, "\"hello\"");
/// ```
pub fn quote_json_string(value: &str) -> String {
    format!("\"{}\"", to_str(value))
}

/// Converts a displayable value to a JSON representation.
///
/// After converting `value` to text and trimming surrounding whitespace, this
/// function returns the text as-is if it looks like a JSON boolean, `null`,
/// number, object, or array. Otherwise, it returns the text as a quoted JSON
/// string, escaped using [`to_str`].
///
/// This is a simple format check, not full JSON validation. For example,
/// strings enclosed in `{}` or `[]` are returned unchanged even if their
/// contents are not valid JSON.
///
/// # Examples
///
/// ```
/// use crator::value_to_json;
/// assert_eq!(value_to_json(&42), "42");
/// assert_eq!(value_to_json(&true), "true");
/// assert_eq!(value_to_json(&"hello"), "\"hello\"");
/// assert_eq!(value_to_json(&"[1, 2]"), "[1, 2]");
/// ```
pub fn value_to_json<T: std::fmt::Display>(value: &T) -> String {
    let text = value.to_string();
    let trimmed = text.trim();

    if trimmed == "true"
        || trimmed == "false"
        || trimmed == "null"
        || trimmed.parse::<f64>().is_ok()
        || ((trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']')))
    {
        trimmed.to_string()
    } else {
        quote_json_string(trimmed)
    }
}

/// The core muncher macro for generating JSON-like markup.
///
/// Primarily invoked by the `rsj!` macro to handle recursive nesting, 
/// attribute-like fields, loops, and conditional content.
#[macro_export]
macro_rules! json_muncher {
    // Empty root
    ($m:expr, $d:expr, $tag:ident, [],) => {{
        let tag_name = stringify!($tag);
        if tag_name == "arr" || tag_name == "list" {
            "[]"
        } else {
            "{}"
        }
    }};

    // Finish and assemble fields
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*],) => {{
        let mut fields = Vec::new();
        $(
            let child_str = format!("{}", $children);
            if !child_str.trim().is_empty() {
                fields.push(child_str);
            }
        )*

        let current_indent = match $m {
            2 => "  ".repeat($d),
            4 => "    ".repeat($d),
            _ => String::new(),
        };
        let nl = if $m > 0 { "\n" } else { "" };

        let tag_name = stringify!($tag);
        let (open, close) = if tag_name == "arr" || tag_name == "list" {
            ("[", "]")
        } else {
            ("{", "}")
        };

        if fields.is_empty() {
            format!("{}{}", open, close)
        } else {
            let inner = fields.join(&format!(",{}", nl));
            format!("{}{}{}{}{}{}", open, nl, inner, nl, current_indent, close)
        }
    }};

    // For loops
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        for $var:pat in $collection:expr => {
            $it:ident { $($ic:tt)* }
        } $($rest:tt)*
    ) => {{
        let mut items = Vec::new();
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        for $var in $collection {
            let inner = $crate::json_muncher!(
                $m, $d + 1, $it, [], $($ic)*
            )
            .trim()
            .to_string();

            items.push(format!("{}{}", child_indent, inner));
        }

        let joined_items =
            items.join(&format!(",{}", if $m > 0 { "\n" } else { "" }));

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* joined_items],
            $($rest)*
        )
    }};

    // If / else
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        if $cond:expr => {
            $key:tt : $it:ident { $($ic:tt)* }
        } else {
            $e_key:tt : $e_it:ident { $($e_ic:tt)* }
        } $($rest:tt)*
    ) => {{
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        let result = if $cond {
            let inner = $crate::json_muncher!(
                $m, $d + 1, $it, [], $($ic)*
            )
            .trim()
            .to_string();

            format!(
                "{}{}: {}",
                child_indent,
                $crate::rules::quote_json_string(
                    stringify!($key).trim_matches('"')
                ),
                inner
            )
        } else {
            let inner = $crate::json_muncher!(
                $m, $d + 1, $e_it, [], $($e_ic)*
            )
            .trim()
            .to_string();

            format!(
                "{}{}: {}",
                child_indent,
                $crate::rules::quote_json_string(
                    stringify!($e_key).trim_matches('"')
                ),
                inner
            )
        };

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* result],
            $($rest)*
        )
    }};

    // Optional if
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        if $cond:expr => {
            $key:tt : $it:ident { $($ic:tt)* }
        } $($rest:tt)*
    ) => {{
        let mut result = String::new();

        if $cond {
            let child_indent = match $m {
                2 => "  ".repeat($d + 1),
                4 => "    ".repeat($d + 1),
                _ => String::new(),
            };

            let inner = $crate::json_muncher!(
                $m, $d + 1, $it, [], $($ic)*
            )
            .trim()
            .to_string();

            result = format!(
                "{}{}: {}",
                child_indent,
                $crate::rules::quote_json_string(
                    stringify!($key).trim_matches('"')
                ),
                inner
            );
        }

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* result],
            $($rest)*
        )
    }};

    // Empty keyed collection
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        $inner_key:tt : $inner_tag:ident {} $($rest:tt)*
    ) => {{
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        let tag_name = stringify!($inner_tag);
        let brackets = if tag_name == "arr" || tag_name == "list" {
            "[]"
        } else {
            "{}"
        };

        let field = format!(
            "{}{}: {}",
            child_indent,
            $crate::rules::quote_json_string(
                stringify!($inner_key).trim_matches('"')
            ),
            brackets
        );

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* field],
            $($rest)*
        )
    }};

    // Empty anonymous collection
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        $inner_tag:ident {} $($rest:tt)*
    ) => {{
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        let tag_name = stringify!($inner_tag);
        let brackets = if tag_name == "arr" || tag_name == "list" {
            "[]"
        } else {
            "{}"
        };

        let field = format!("{}{}", child_indent, brackets);

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* field],
            $($rest)*
        )
    }};

    // Nested object or array with a key
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        $inner_key:tt : $inner_tag:ident {
            $($inner_content:tt)*
        } $($rest:tt)*
    ) => {{
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        let inner = $crate::json_muncher!(
            $m, $d + 1, $inner_tag, [], $($inner_content)*
        )
        .trim()
        .to_string();

        let field = format!(
            "{}{}: {}",
            child_indent,
            $crate::rules::quote_json_string(
                stringify!($inner_key).trim_matches('"')
            ),
            inner
        );

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* field],
            $($rest)*
        )
    }};

    // Anonymous nested object or array
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        $inner_tag:ident {
            $($inner_content:tt)*
        } $($rest:tt)*
    ) => {{
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        let inner = $crate::json_muncher!(
            $m, $d + 1, $inner_tag, [], $($inner_content)*
        )
        .trim()
        .to_string();

        let field = format!("{}{}", child_indent, inner);

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* field],
            $($rest)*
        )
    }};

    // String literal field, followed by more fields
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        $key:tt : $value:literal, $($rest:tt)+
    ) => {{
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        let value = $crate::rules::quote_json_string($value);
        let field = format!(
            "{}{}: {}",
            child_indent,
            $crate::rules::quote_json_string(
                stringify!($key).trim_matches('"')
            ),
            value
        );

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* field],
            $($rest)*
        )
    }};

    // String literal field, final field
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        $key:tt : $value:literal
    ) => {{
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        let value = $crate::rules::quote_json_string($value);
        let field = format!(
            "{}{}: {}",
            child_indent,
            $crate::rules::quote_json_string(
                stringify!($key).trim_matches('"')
            ),
            value
        );

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* field],
        )
    }};

    // Braced expression field, followed by more fields
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        $key:tt : { $value:expr }, $($rest:tt)+
    ) => {{
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        let value = $crate::rules::value_to_json(&$value);
        let field = format!(
            "{}{}: {}",
            child_indent,
            $crate::rules::quote_json_string(
                stringify!($key).trim_matches('"')
            ),
            value
        );

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* field],
            $($rest)*
        )
    }};

    // Braced expression field, final field
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        $key:tt : { $value:expr }
    ) => {{
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        let value = $crate::rules::value_to_json(&$value);
        let field = format!(
            "{}{}: {}",
            child_indent,
            $crate::rules::quote_json_string(
                stringify!($key).trim_matches('"')
            ),
            value
        );

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* field],
        )
    }};

    // Other expression field, followed by more fields
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        $key:tt : $value:expr, $($rest:tt)+
    ) => {{
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        let value = $crate::rules::value_to_json(&$value);
        let field = format!(
            "{}{}: {}",
            child_indent,
            $crate::rules::quote_json_string(
                stringify!($key).trim_matches('"')
            ),
            value
        );

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* field],
            $($rest)*
        )
    }};

    // Other expression field, final field
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        $key:tt : $value:expr
    ) => {{
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        let value = $crate::rules::value_to_json(&$value);
        let field = format!(
            "{}{}: {}",
            child_indent,
            $crate::rules::quote_json_string(
                stringify!($key).trim_matches('"')
            ),
            value
        );

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* field],
        )
    }};

    // Standalone string literal
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        $text:literal $($rest:tt)*
    ) => {{
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        let field = format!(
            "{}{}",
            child_indent,
            $crate::rules::quote_json_string($text)
        );

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* field],
            $($rest)*
        )
    }};

    // Braced standalone expression
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        { $text:expr } $($rest:tt)*
    ) => {{
        let child_indent = match $m {
            2 => "  ".repeat($d + 1),
            4 => "    ".repeat($d + 1),
            _ => String::new(),
        };

        let value = $crate::rules::value_to_json(&$text);
        let field = format!("{}{}", child_indent, value);

        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children,)* field],
            $($rest)*
        )
    }};

    // Ignore extra commas
    (
        $m:expr, $d:expr, $tag:ident, [$($children:expr),*],
        , $($rest:tt)*
    ) => {
        $crate::json_muncher!(
            $m, $d, $tag,
            [$($children),*],
            $($rest)*
        )
    };
}

/// A macro to generate JSON markup with declarative indentation styles.
///
/// Supports `lined` (minified), `tabed`/`btfy2` (2 spaces), and `btfy4` (4 spaces).
/// 
/// # Examples
/// 
/// ### 1. Advanced Conditional Logic and Loops
/// ```rust
/// use crator::rsj;
///
/// let items = vec!["Rust", "Forge", "Crator"];
/// let is_logged_in = true;
/// let has_premium = false;
///
/// let my_json = rsj!(tabed, obj {
///     status: "success",
///     code: 200 as u8,
///     activity: arr {},
///     if is_logged_in => { 
///         user: obj { 
///             name: "Ahmed", 
///             role: "admin",
///             age: 24 as u8
///         } 
///     } else { 
///         guest: obj { status: "anonymous" } 
///     },
///     if has_premium => { 
///         rewards: arr { "Badge", "Gift" } 
///     },
///     data: obj {
///         version: "2.1.0",
///         tags: arr {
///             for item in items => { obj { name: {item} } }
///         }
///     }
/// });
/// ```
/// 
/// ### 2. Complex Product Lists (Object and Array Roots)
/// ```rust
/// use crator::rsj;
///
/// let products = vec![
///     ("Laptop", 999.99, true),
///     ("Mouse", 25.50, false),
///     ("Keyboard", 75.00, true),
/// ];
///
/// // JSON Object root
/// let product_obj = rsj!(btfy4, obj {
///     store: "CratorTech",
///     inventory: arr {
///         for (name, price, available) in products.clone() => {
///             obj { 
///                 item: {name}, 
///                 price: {price}, 
///                 available: {available} 
///             }
///         }
///     }
/// });
///
/// // JSON Array root
/// let product_arr = rsj!(btfy4, arr {
///     for (name, price, available) in products => {
///         obj { 
///             item: {name}, 
///             price: {price}, 
///             available: {available} 
///         }
///     }
/// });
/// ```
/// 
/// ### 3. Smart Contract ABI Definitions (Deeply Nested Schemas)
/// ```rust
/// use crator::rsj;
/// 
/// let json_str = rsj!(tabed, arr {
///    obj {
///     inputs: arr {},
///     name: "greet",
///     outputs: arr {
///         "internalType": "string",
///         name: "",
///         type: "string"
///     },
///     stateMutability :"pure",
///     type: "function"
///    }
/// });
/// ```
/// 
/// ### 4. Flexible Keys (Spaces & Hyphens)
/// ```rust
/// use crator::rsj;
/// 
/// let is_active = true;
/// let advanced_json = rsj!(btfy2, obj {
///     "API Version": "1.0.0",     // Key with spaces
///     "x-api-key": "secret-123",  // Key with hyphens
/// 
///     if is_active => { 
///         "user-session": obj {   // Quoted key in conditional
///             id: 101 as u128,
///             status: "verified"
///         } 
///     },
/// 
///     data: obj {                 // Mixing quoted and unquoted
///         "Content-Type": "application/json",
///         tags: arr { "rust", "json", "crator" }
///     }
/// });
/// ```
/// 
/// ### 5. Numeric Values and Numeric Strings
/// ```rust
/// use crator::rsj;
/// fn main() {
///     let json = rsj!(tabed, obj {
///             str_int: "2",                   // JSON string: "2"
///             str_negative_int: "-2",         // JSON string: "-2"
///             str_float: "2.0",               // JSON string: "2.0"
///             str_negative_float: "-2.0",     // JSON string: "-2.0"
///             float_64: 2.0 as f64,           // JSON number: 2
///             float_32: 2.0 as f32,           // JSON number: 2
///             unsigned_8: 2.0 as u8,          // JSON number: 2
///             negative_float_32: -2.0 as f32, // JSON number: -2
///             negative_i8: -2 as i8,          // JSON number: -2
///             negative_float_64: -2.0 as f64  // JSON number: -2
///         }
///     );
///     println!("{}", json);
///     // {
///     //   "str_int": "2",
///     //   "str_negative_int": "-2",
///     //   "str_float": "2.0",
///     //   "str_negative_float": "-2.0",
///     //   "float_64": 2,
///     //   "float_32": 2,
///     //   "unsigned_8": 2,
///     //   "negative_float_32": -2,
///     //   "negative_i8": -2,
///     //   "negative_float_64": -2
///     // }
/// }
/// ```
#[macro_export]
macro_rules! rsj {
    ($m_name:ident, $tag:ident { $($content:tt)* }) => {{
        let m = match stringify!($m_name) {
            "lined" => 0,
            "tabed" | "btfy2" => 2,
            "btfy4" => 4,
            _ => 0,
        };
        $crate::json_muncher!(m, 0, $tag, [], $($content)*)
    }};
}