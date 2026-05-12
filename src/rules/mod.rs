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

/// The core muncher macro for generating JSON-like markup.
///
/// Primarily invoked by the `rsj!` macro to handle recursive nesting, 
/// attribute-like fields, loops, and conditional content.
#[macro_export]
macro_rules! json_muncher {
    // 1. EMPTY ROOT CASE - Matches obj {} or arr {} with nothing inside
    ($m:expr, $d:expr, $tag:ident, [], ) => {{
        let tag_name = stringify!($tag);
        if tag_name == "arr" || tag_name == "list" { "[]" } else { "{}" }
    }};

    // 2. TERMINATION - Final assembly
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*], ) => {{
        let mut fields = Vec::new();
        $(
            let child_str = format!("{}", $children);
            if !child_str.trim().is_empty() { fields.push(child_str); }
        )*

        let current_indent = match $m { 2 => "  ".repeat($d), 4 => "    ".repeat($d), _ => String::new() };
        let nl = if $m > 0 { "\n" } else { "" };
        
        let tag_name = stringify!($tag);
        let (open, close) = if tag_name == "arr" || tag_name == "list" { ("[", "]") } else { ("{", "}") };

        if fields.is_empty() {
            format!("{}{}", open, close)
        } else {
            // Ensure join with the newline and comma correctly
            let inner = fields.join(&format!(",{}", nl));
            format!("{}{}{}{}{}{}", open, nl, inner, nl, current_indent, close)
        }
    }};

    // 3. FOR LOOPS
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*], for $var:pat in $collection:expr => { $it:ident { $($ic:tt)* } } $($rest:tt)*) => {{
        let mut items = Vec::new();
        let child_indent = match $m { 2 => "  ".repeat($d + 1), 4 => "    ".repeat($d + 1), _ => String::new() };
        for $var in $collection {
            let inner = $crate::json_muncher!($m, $d + 1, $it, [], $($ic)*).trim().to_string();
            items.push(format!("{}{}", child_indent, inner));
        }
        let joined_items = items.join(&format!(",{}", if $m > 0 { "\n" } else { "" }));
        $crate::json_muncher!($m, $d, $tag, [$($children,)* joined_items.to_string()], $($rest)*)
    }};

    // 4. IF / ELSE
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*], if $cond:expr => { $key:tt : $it:ident { $($ic:tt)* } } else { $e_key:tt : $e_it:ident { $($e_ic:tt)* } } $($rest:tt)*) => {{
        let child_indent = match $m { 2 => "  ".repeat($d + 1), 4 => "    ".repeat($d + 1), _ => String::new() };
        let result: String = if $cond {
            let inner = $crate::json_muncher!($m, $d + 1, $it, [], $($ic)*).trim().to_string();
            format!("{}\"{}\": {}", child_indent, stringify!($key).trim_matches('"'), inner)
        } else {
            let inner = $crate::json_muncher!($m, $d + 1, $e_it, [], $($e_ic)*).trim().to_string();
            format!("{}\"{}\": {}", child_indent, stringify!($e_key).trim_matches('"'), inner)
        };
        $crate::json_muncher!($m, $d, $tag, [$($children,)* result], $($rest)*)
    }};

    // 5. OPTIONAL IF
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*], if $cond:expr => { $key:tt : $it:ident { $($ic:tt)* } } $($rest:tt)*) => {{
        let mut result = String::new();
        if $cond {
            let child_indent = match $m { 2 => "  ".repeat($d + 1), 4 => "    ".repeat($d + 1), _ => String::new() };
            let inner = $crate::json_muncher!($m, $d + 1, $it, [], $($ic)*).trim().to_string();
            result = format!("{}\"{}\": {}", child_indent, stringify!($key).trim_matches('"'), inner);
        }
        $crate::json_muncher!($m, $d, $tag, [$($children,)* result], $($rest)*)
    }};

    // 6. EMPTY COLLECTIONS (Keyed)
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*], $inner_key:tt : $inner_tag:ident {} $($rest:tt)*) => {{
        let child_indent = match $m { 2 => "  ".repeat($d + 1), 4 => "    ".repeat($d + 1), _ => String::new() };
        let tag_str = stringify!($inner_tag);
        let brackets = if tag_str == "arr" || tag_str == "list" { "[]" } else { "{}" };
        let val = format!("{}\"{}\": {}", child_indent, stringify!($inner_key).trim_matches('"'), brackets);
        $crate::json_muncher!($m, $d, $tag, [$($children,)* val.to_string()], $($rest)*)
    }};

    // 7. EMPTY ANONYMOUS COLLECTIONS (For `arr { obj {} }`)
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*], $inner_tag:ident {} $($rest:tt)*) => {{
        let child_indent = match $m { 2 => "  ".repeat($d + 1), 4 => "    ".repeat($d + 1), _ => String::new() };
        let tag_str = stringify!($inner_tag);
        let brackets = if tag_str == "arr" || tag_str == "list" { "[]" } else { "{}" };
        let val = format!("{}{}", child_indent, brackets);
        $crate::json_muncher!($m, $d, $tag, [$($children,)* val.to_string()], $($rest)*)
    }};

    // 8. NESTED OBJECT/ARRAY (With Key)
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*], $inner_key:tt : $inner_tag:ident { $($inner_content:tt)* } $($rest:tt)*) => {{
        let child_indent = match $m { 2 => "  ".repeat($d + 1), 4 => "    ".repeat($d + 1), _ => String::new() };
        let inner = $crate::json_muncher!($m, $d + 1, $inner_tag, [], $($inner_content)*).trim().to_string();
        let val = format!("{}\"{}\": {}", child_indent, stringify!($inner_key).trim_matches('"'), inner);
        $crate::json_muncher!($m, $d, $tag, [$($children,)* val], $($rest)*)
    }};

    // 9. ANONYMOUS NESTED OBJECT/ARRAY (For `arr { obj { k:v } }`)
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*], $inner_tag:ident { $($inner_content:tt)* } $($rest:tt)*) => {{
        let child_indent = match $m { 2 => "  ".repeat($d + 1), 4 => "    ".repeat($d + 1), _ => String::new() };
        let inner = $crate::json_muncher!($m, $d + 1, $inner_tag, [], $($inner_content)*).trim().to_string();
        let val = format!("{}{}", child_indent, inner);
        $crate::json_muncher!($m, $d, $tag, [$($children,)* val], $($rest)*)
    }};

    // 10. STANDARD FIELDS
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*], $key:tt : $val:expr, $($rest:tt)+) => {{
        let child_indent = match $m { 2 => "  ".repeat($d + 1), 4 => "    ".repeat($d + 1), _ => String::new() };
        let f = $crate::rules::format_json_field(stringify!($key), &format!("{}", $val), &child_indent);
        $crate::json_muncher!($m, $d, $tag, [$($children,)* f], $($rest)*)
    }};
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*], $key:tt : $val:expr) => {{
        let child_indent = match $m { 2 => "  ".repeat($d + 1), 4 => "    ".repeat($d + 1), _ => String::new() };
        let f = $crate::rules::format_json_field(stringify!($key), &format!("{}", $val), &child_indent);
        $crate::json_muncher!($m, $d, $tag, [$($children,)* f], )
    }};

    // 11. LITERALS / BRACED
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*], $text:literal $($rest:tt)*) => {{
        let child_indent = match $m { 2 => "  ".repeat($d + 1), 4 => "    ".repeat($d + 1), _ => String::new() };
        $crate::json_muncher!($m, $d, $tag, [$($children,)* format!("{}\"{}\"", child_indent, $text)], $($rest)*)
    }};
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*], { $text:expr } $($rest:tt)*) => {{
        let child_indent = match $m { 2 => "  ".repeat($d + 1), 4 => "    ".repeat($d + 1), _ => String::new() };
        let val = format!("{}{}", child_indent, $text);
        $crate::json_muncher!($m, $d, $tag, [$($children,)* val], $($rest)*)
    }};

    // 12. CLEANUP
    ($m:expr, $d:expr, $tag:ident, [$($children:expr),*], , $($rest:tt)*) => {
        $crate::json_muncher!($m, $d, $tag, [$($children),*], $($rest)*)
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
///     code: 200,
///     activity: arr {},
///     if is_logged_in => { 
///         user: obj { 
///             name: "Ahmed", 
///             role: "admin",
///             age: 24
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
///             id: 101,
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