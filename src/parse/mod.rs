use regex::Regex;
use serde_yaml::Value;
use std::sync::OnceLock;

const MAX_CONTENT_SIZE: usize = 256 * 1024;

fn agent_name_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Z][a-zA-Z0-9]{2,50}$").expect("valid regex"))
}

pub fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    if content.len() > MAX_CONTENT_SIZE {
        return None;
    }
    if !content.starts_with("---") {
        return None;
    }
    let after_first = &content[3..];
    let after_first = after_first.strip_prefix('\n').unwrap_or(after_first);
    if let Some(rest) = after_first.strip_prefix("---") {
        let body = rest.strip_prefix('\n').unwrap_or(rest);
        return Some(("", body));
    }
    let end = after_first.find("\n---")?;
    let yaml = &after_first[..end];
    let rest = &after_first[end + 4..];
    let body = rest.strip_prefix('\n').unwrap_or(rest);
    Some((yaml, body))
}

pub fn fm_value(content: &str, key: &str) -> Option<String> {
    let (yaml_text, _) = split_frontmatter(content)?;
    let value: Value = serde_yaml::from_str(yaml_text).ok()?;
    let mapping = value.as_mapping()?;
    let key_value = mapping.get(Value::String(key.to_string()))?;
    match key_value {
        Value::String(s) => Some(s.clone()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::Null => None,
        _ => Some(serde_yaml::to_string(key_value).ok()?.trim().to_string()),
    }
}

pub fn fm_list(content: &str, key: &str) -> Option<String> {
    let (yaml_text, _) = split_frontmatter(content)?;
    let value: Value = serde_yaml::from_str(yaml_text).ok()?;
    let mapping = value.as_mapping()?;
    let key_value = mapping.get(Value::String(key.to_string()))?;
    match key_value {
        Value::Sequence(seq) => {
            let items: Vec<String> = seq
                .iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    Value::Number(n) => Some(n.to_string()),
                    Value::Bool(b) => Some(b.to_string()),
                    _ => None,
                })
                .collect();
            if items.is_empty() {
                None
            } else {
                Some(items.join(", "))
            }
        }
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

pub fn fm_body(content: &str) -> &str {
    if let Some((_, body)) = split_frontmatter(content) {
        body
    } else {
        content
    }
}

pub fn validate_agent_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("agent name is empty".to_string());
    }
    if !agent_name_regex().is_match(name) {
        return Err(format!(
            "agent name {name:?} does not match ^[A-Z][a-zA-Z0-9]{{2,50}}$"
        ));
    }
    Ok(())
}

pub fn extract_source_field(content: &str) -> Option<String> {
    // TOML format: # source: comment on first line
    if let Some(first_line) = content.lines().next() {
        if let Some(source) = first_line.strip_prefix("# source:") {
            let source = source.trim();
            if !source.is_empty() {
                return Some(source.to_string());
            }
        }
    }
    // Frontmatter format: source: in YAML frontmatter
    fm_value(content, "source")
}

pub fn module_name(content: &str) -> Option<String> {
    fm_value(content, "name").or_else(|| {
        content.lines().find_map(|l| {
            l.strip_prefix("name:")
                .map(|v| v.trim().trim_matches('"').trim_matches('\'').to_string())
        })
    })
}

pub fn is_synced_from(content: &str, expected_source: &str) -> bool {
    if let Some(source) = extract_source_field(content) {
        if source == expected_source || source.ends_with(&format!("/{expected_source}")) {
            return true;
        }
    }
    // Legacy format: # synced-from: in body
    let expected_comment = format!("# synced-from: {expected_source}");
    let body = fm_body(content);
    let first_line = body.lines().next().unwrap_or("");
    first_line == expected_comment
}

#[cfg(test)]
mod tests;
