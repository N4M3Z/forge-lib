//! YAML CLI — query arbitrary YAML files from shell scripts.
//!
//! Dot-path notation for nested access:
//!   yaml value  <file> <path> [default]    # scalar extraction
//!   yaml list   <file> <path>              # array → one item per line
//!   yaml map    <file> <path>              # mapping → key\tvalue per line
//!   yaml keys   <file> <path>              # mapping → keys only
//!   yaml env    <file|dir> <path> [--prefix P] # mapping → shell export lines
//!   yaml nested <file> <parent> <child> [default]  # legacy (use value with dot-path)
//!
//! Path examples:
//!   .agents                    → top-level key
//!   .skills.claude             → nested key
//!   .skills.claude.DebateCouncil.scope → deep nesting
//!   .modules[0]                → array index
//!   .modules[0].name           → array index + nested key
//!   agents                     → leading dot is optional

use serde_yaml::{Mapping, Value};
use std::{env, fs, process};

#[cfg(test)]
mod tests;

// --- Path parsing ---

enum PathSegment {
    Key(String),
    Index(usize),
}

fn parse_path(path: &str) -> Vec<PathSegment> {
    let path = path.strip_prefix('.').unwrap_or(path);
    if path.is_empty() {
        return vec![];
    }

    let mut segments = Vec::new();
    for part in path.split('.') {
        if let Some(bracket) = part.find('[') {
            let key = &part[..bracket];
            if !key.is_empty() {
                segments.push(PathSegment::Key(key.to_string()));
            }
            // Parse all [N] suffixes: field[0][1]
            let mut rest = &part[bracket..];
            while let Some(start) = rest.find('[') {
                if let Some(end) = rest.find(']') {
                    if let Ok(idx) = rest[start + 1..end].parse::<usize>() {
                        segments.push(PathSegment::Index(idx));
                    }
                    rest = &rest[end + 1..];
                } else {
                    break;
                }
            }
        } else {
            segments.push(PathSegment::Key(part.to_string()));
        }
    }
    segments
}

fn walk(doc: &Value, segments: &[PathSegment]) -> Option<Value> {
    let mut current = doc.clone();
    for seg in segments {
        current = match seg {
            PathSegment::Key(k) => current.get(k.as_str())?.clone(),
            PathSegment::Index(i) => current.get(*i)?.clone(),
        };
    }
    Some(current)
}

// --- Helpers ---

fn load(path: &str) -> Value {
    let Ok(content) = fs::read_to_string(path) else {
        return Value::Mapping(Mapping::default());
    };
    serde_yaml::from_str(&content).unwrap_or(Value::Mapping(Mapping::default()))
}

fn as_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        _ => format!("{v:?}"),
    }
}

fn strip_quotes(s: &str) -> &str {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn print_value(v: &Value) {
    match v {
        Value::String(_) | Value::Number(_) | Value::Bool(_) => {
            let s = as_str(v);
            println!("{}", strip_quotes(&s));
        }
        Value::Null | Value::Tagged(_) => {}
        Value::Sequence(items) => {
            for item in items {
                let s = as_str(item);
                let s = strip_quotes(&s);
                if !s.is_empty() {
                    println!("{s}");
                }
            }
        }
        Value::Mapping(map) => {
            for (k, v) in map {
                let key = as_str(k);
                let val = as_str(v);
                let val = strip_quotes(&val);
                if !key.is_empty() {
                    println!("{key}\t{val}");
                }
            }
        }
    }
}

// --- Commands ---

fn cmd_value(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: yaml value <file> <path> [default]");
        process::exit(1);
    }
    let doc = load(&args[0]);
    let segments = parse_path(&args[1]);
    let default = args.get(2).map_or("", |s| s.as_str());

    match walk(&doc, &segments) {
        Some(Value::String(_) | Value::Number(_) | Value::Bool(_)) => {
            print_value(&walk(&doc, &segments).unwrap());
        }
        _ => println!("{default}"),
    }
}

fn cmd_list(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: yaml list <file> <path>");
        process::exit(1);
    }
    let doc = load(&args[0]);
    let segments = parse_path(&args[1]);

    if let Some(Value::Sequence(items)) = walk(&doc, &segments) {
        for item in &items {
            let s = as_str(item);
            let s = strip_quotes(&s);
            if !s.is_empty() {
                println!("{s}");
            }
        }
    }
}

fn cmd_map(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: yaml map <file> <path>");
        process::exit(1);
    }
    let doc = load(&args[0]);
    let segments = parse_path(&args[1]);

    if let Some(Value::Mapping(map)) = walk(&doc, &segments) {
        for (k, v) in &map {
            let key = as_str(k);
            if let Value::Sequence(items) = v {
                for item in items {
                    let val = as_str(item);
                    let val = strip_quotes(&val);
                    if !val.is_empty() {
                        println!("{key}\t{val}");
                    }
                }
            } else {
                let val = as_str(v);
                let val = strip_quotes(&val);
                if !key.is_empty() && !val.is_empty() {
                    println!("{key}\t{val}");
                }
            }
        }
    }
}

fn cmd_keys(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: yaml keys <file> <path>");
        process::exit(1);
    }
    let doc = load(&args[0]);
    let segments = parse_path(&args[1]);

    if let Some(Value::Mapping(map)) = walk(&doc, &segments) {
        for k in map.keys() {
            let key = as_str(k);
            if !key.is_empty() {
                println!("{key}");
            }
        }
    }
}

fn cmd_get(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: yaml get <file> <path> [default]");
        process::exit(1);
    }
    let doc = load(&args[0]);
    let segments = parse_path(&args[1]);
    let default = args.get(2).map_or("", |s| s.as_str());

    match walk(&doc, &segments) {
        Some(ref v) => print_value(v),
        None => {
            if !default.is_empty() {
                println!("{default}");
            }
        }
    }
}

// --- env ---

/// Deep merge overlay onto base. Overlay values win for scalars;
/// mappings are merged recursively.
fn deep_merge(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Mapping(base_map), Value::Mapping(overlay_map)) => {
            for (k, v) in overlay_map {
                if let Some(base_val) = base_map.get_mut(k) {
                    deep_merge(base_val, v);
                } else {
                    base_map.insert(k.clone(), v.clone());
                }
            }
        }
        (base, overlay) => {
            *base = overlay.clone();
        }
    }
}

fn shell_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

fn flatten_env(mapping: &Mapping, prefix: &str) -> Vec<String> {
    let mut output = Vec::new();
    for (k, v) in mapping {
        let key_part = as_str(k).to_uppercase().replace('-', "_");
        let full_key = format!("{prefix}_{key_part}");
        match v {
            Value::String(_) | Value::Number(_) | Value::Bool(_) => {
                let raw = as_str(v);
                let val = strip_quotes(&raw);
                output.push(format!("export {full_key}='{}'", shell_escape(val)));
            }
            Value::Mapping(nested) => {
                output.extend(flatten_env(nested, &full_key));
            }
            _ => {}
        }
    }
    output
}

fn cmd_env(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: yaml env <file|dir> <path> [--prefix PREFIX]");
        process::exit(1);
    }
    let source = &args[0];
    let doc = if std::path::Path::new(source).is_dir() {
        let dir = source.trim_end_matches('/');
        let mut base = load(&format!("{dir}/defaults.yaml"));
        let overlay = load(&format!("{dir}/config.yaml"));
        deep_merge(&mut base, &overlay);
        base
    } else {
        load(source)
    };
    let segments = parse_path(&args[1]);

    let mut prefix = "FORGE";
    let remaining = &args[2..];
    let mut i = 0;
    while i < remaining.len() {
        if remaining[i] == "--prefix" {
            if i + 1 < remaining.len() {
                prefix = &remaining[i + 1];
                i += 2;
                continue;
            }
        }
        i += 1;
    }

    let value = walk(&doc, &segments).unwrap_or(Value::Null);
    if let Value::Mapping(ref map) = value {
        for line in flatten_env(map, prefix) {
            println!("{line}");
        }
    }
}

// Legacy: `yaml nested <file> <parent> <child> [default]`
fn cmd_nested(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: yaml nested <file> <parent> <child> [default]");
        process::exit(1);
    }
    let path = format!("{}.{}", args[1], args[2]);
    let mut new_args = vec![args[0].clone(), path];
    if let Some(d) = args.get(3) {
        new_args.push(d.clone());
    }
    cmd_value(&new_args);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: yaml <command> <file> <path> [...]");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  get    <file> <path> [default]   Auto-detect type and print");
        eprintln!("  value  <file> <path> [default]   Extract scalar (default if missing)");
        eprintln!("  list   <file> <path>             Print array items, one per line");
        eprintln!("  map    <file> <path>             Print mapping as key\\tvalue lines");
        eprintln!("  keys   <file> <path>             Print mapping keys, one per line");
        eprintln!("  env    <file|dir> <path> [--prefix P]  Flatten mapping to shell exports");
        eprintln!("  nested <file> <p> <c> [default]  Legacy: same as value with <p>.<c>");
        eprintln!();
        eprintln!("Paths: .field.subfield, .array[0], .deep.path[1].key");
        process::exit(1);
    }

    let cmd = args[1].as_str();
    let rest = &args[2..];

    match cmd {
        "get" => cmd_get(rest),
        "value" => cmd_value(rest),
        "list" => cmd_list(rest),
        "map" => cmd_map(rest),
        "keys" => cmd_keys(rest),
        "env" => cmd_env(rest),
        "nested" => cmd_nested(rest),
        _ => {
            eprintln!("Unknown command: {cmd}");
            eprintln!("Commands: get, value, list, map, keys, env, nested");
            process::exit(1);
        }
    }
}
