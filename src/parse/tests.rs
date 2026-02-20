use super::*;

// --- split_frontmatter ---

#[test]
fn split_basic() {
    let content = "---\ntitle: Hello\n---\nBody text";
    let (fm, body) = split_frontmatter(content).unwrap();
    assert_eq!(fm, "title: Hello");
    assert_eq!(body, "Body text");
}

#[test]
fn split_no_frontmatter() {
    assert!(split_frontmatter("Just plain text").is_none());
}

#[test]
fn split_empty_body() {
    let content = "---\ntitle: Hello\n---\n";
    let (fm, body) = split_frontmatter(content).unwrap();
    assert_eq!(fm, "title: Hello");
    assert!(body.is_empty());
}

#[test]
fn split_rejects_oversized_content() {
    let big = format!("---\ntitle: x\n---\n{}", "x".repeat(256 * 1024));
    assert!(split_frontmatter(&big).is_none());
}

#[test]
fn split_unclosed_frontmatter() {
    let content = "---\ntitle: Hello\nno closing delimiter";
    assert!(split_frontmatter(content).is_none());
}

#[test]
fn split_empty_frontmatter() {
    let content = "---\n---\nBody";
    let (fm, body) = split_frontmatter(content).unwrap();
    assert_eq!(fm, "");
    assert_eq!(body, "Body");
}

#[test]
fn split_multiline_frontmatter() {
    let content = "---\ntitle: Hello\nauthor: World\ntags:\n  - one\n  - two\n---\nBody";
    let (fm, body) = split_frontmatter(content).unwrap();
    assert!(fm.contains("title: Hello"));
    assert!(fm.contains("  - two"));
    assert_eq!(body, "Body");
}

// --- fm_value ---

#[test]
fn value_simple_key() {
    let content = "---\ntitle: Hello World\n---\nBody";
    assert_eq!(fm_value(content, "title"), Some("Hello World".into()));
}

#[test]
fn value_dotted_key() {
    let content = "---\nclaude.name: SecurityArchitect\nclaude.model: sonnet\n---\n";
    assert_eq!(
        fm_value(content, "claude.name"),
        Some("SecurityArchitect".into())
    );
    assert_eq!(fm_value(content, "claude.model"), Some("sonnet".into()));
}

#[test]
fn value_dotted_key_no_wildcard() {
    // Regression: shell awk uses `.` as regex wildcard, matching `claudeXname:`
    // serde_yaml uses exact string keys — no wildcard issue
    let content = "---\nclaudeXname: Wrong\nclaude.name: Right\n---\n";
    assert_eq!(fm_value(content, "claude.name"), Some("Right".into()));
}

#[test]
fn value_quoted_double() {
    let content = "---\ndescription: \"A quoted value\"\n---\n";
    assert_eq!(
        fm_value(content, "description"),
        Some("A quoted value".into())
    );
}

#[test]
fn value_quoted_single() {
    let content = "---\ndescription: 'Single quoted'\n---\n";
    assert_eq!(
        fm_value(content, "description"),
        Some("Single quoted".into())
    );
}

#[test]
fn value_missing_key() {
    let content = "---\ntitle: Hello\n---\n";
    assert_eq!(fm_value(content, "missing"), None);
}

#[test]
fn value_no_frontmatter() {
    assert_eq!(fm_value("Just text", "title"), None);
}

#[test]
fn value_with_colon_in_value() {
    let content = "---\nurl: \"https://example.com\"\n---\n";
    assert_eq!(fm_value(content, "url"), Some("https://example.com".into()));
}

#[test]
fn value_boolean() {
    let content = "---\ndraft: true\n---\n";
    assert_eq!(fm_value(content, "draft"), Some("true".into()));
}

#[test]
fn value_number() {
    let content = "---\npriority: 42\n---\n";
    assert_eq!(fm_value(content, "priority"), Some("42".into()));
}

#[test]
fn value_null_returns_none() {
    let content = "---\nempty:\n---\n";
    assert_eq!(fm_value(content, "empty"), None);
}

// --- fm_list ---

#[test]
fn list_yaml_sequence() {
    let content = "---\nclaude.tools:\n  - Read\n  - Write\n  - Bash\n---\n";
    assert_eq!(
        fm_list(content, "claude.tools"),
        Some("Read, Write, Bash".into())
    );
}

#[test]
fn list_string_fallback() {
    let content = "---\nclaude.tools: Read, Write, Bash\n---\n";
    assert_eq!(
        fm_list(content, "claude.tools"),
        Some("Read, Write, Bash".into())
    );
}

#[test]
fn list_missing_key() {
    let content = "---\ntitle: Hello\n---\n";
    assert_eq!(fm_list(content, "tools"), None);
}

#[test]
fn list_empty_sequence() {
    let content = "---\nclaude.tools: []\n---\n";
    assert_eq!(fm_list(content, "claude.tools"), None);
}

#[test]
fn list_terminated_by_next_key() {
    let content = "---\nclaude.tools:\n  - Read\n  - Write\nclaude.model: sonnet\n---\n";
    assert_eq!(fm_list(content, "claude.tools"), Some("Read, Write".into()));
}

#[test]
fn list_single_item() {
    let content = "---\ntags:\n  - one\n---\n";
    assert_eq!(fm_list(content, "tags"), Some("one".into()));
}

// --- fm_body ---

#[test]
fn body_basic() {
    let content = "---\ntitle: Hello\n---\nBody text here";
    assert_eq!(fm_body(content), "Body text here");
}

#[test]
fn body_no_frontmatter() {
    let content = "Just plain text";
    assert_eq!(fm_body(content), "Just plain text");
}

#[test]
fn body_empty_after_frontmatter() {
    let content = "---\ntitle: Hello\n---\n";
    assert_eq!(fm_body(content), "");
}

#[test]
fn body_multiline() {
    let content = "---\ntitle: Hello\n---\nLine 1\nLine 2\nLine 3";
    assert_eq!(fm_body(content), "Line 1\nLine 2\nLine 3");
}

#[test]
fn body_preserves_leading_blank_lines() {
    let content = "---\ntitle: Hello\n---\n\n\nBody after blanks";
    assert_eq!(fm_body(content), "\n\nBody after blanks");
}

// --- validate_agent_name ---

#[test]
fn name_valid_pascal_case() {
    assert!(validate_agent_name("SecurityArchitect").is_ok());
}

#[test]
fn name_valid_short() {
    assert!(validate_agent_name("Dev").is_ok());
}

#[test]
fn name_too_short() {
    assert!(validate_agent_name("AB").is_err());
}

#[test]
fn name_too_long() {
    let long = format!("A{}", "a".repeat(51));
    assert!(validate_agent_name(&long).is_err());
}

#[test]
fn name_starts_lowercase() {
    assert!(validate_agent_name("developer").is_err());
}

#[test]
fn name_contains_slash() {
    assert!(validate_agent_name("A/evil").is_err());
}

#[test]
fn name_contains_dotdot() {
    assert!(validate_agent_name("A..evil").is_err());
}

#[test]
fn name_contains_spaces() {
    assert!(validate_agent_name("My Agent").is_err());
}

#[test]
fn name_empty() {
    assert!(validate_agent_name("").is_err());
}

#[test]
fn name_with_numbers() {
    assert!(validate_agent_name("Model3Config").is_ok());
}

// --- is_synced_from ---

#[test]
fn synced_from_first_line_match() {
    let content = "# synced-from: Agent.md\n\nBody content";
    assert!(is_synced_from(content, "Agent.md"));
}

#[test]
fn synced_from_after_frontmatter() {
    let content = "---\nname: Test\n---\n# synced-from: Agent.md\n\nBody";
    assert!(is_synced_from(content, "Agent.md"));
}

#[test]
fn synced_from_body_injection_rejected() {
    let content = "---\nname: Test\n---\nRegular body.\n\n# synced-from: Agent.md\n\nMore body";
    assert!(!is_synced_from(content, "Agent.md"));
}

#[test]
fn synced_from_missing_header() {
    let content = "---\nname: Test\n---\nBody";
    assert!(!is_synced_from(content, "Agent.md"));
}

#[test]
fn synced_from_different_source() {
    let content = "# synced-from: Other.md\n\nBody";
    assert!(!is_synced_from(content, "Agent.md"));
}

#[test]
fn synced_from_empty_content() {
    assert!(!is_synced_from("", "Agent.md"));
}

#[test]
fn synced_from_frontmatter_source_exact() {
    let content = "---\nname: Agent\nsource: Agent.md\n---\nBody";
    assert!(is_synced_from(content, "Agent.md"));
}

#[test]
fn synced_from_frontmatter_source_with_prefix() {
    let content = "---\nname: Agent\nsource: forge-council/agents/Agent.md\n---\nBody";
    assert!(is_synced_from(content, "Agent.md"));
}

#[test]
fn synced_from_source_frontmatter_wrong_file() {
    let content = "---\nname: TheOpponent\nsource: forge-council/agents/TheOpponent.md\n---\nBody";
    assert!(!is_synced_from(content, "Other.md"));
}

// --- is_synced_from: TOML source comment ---

#[test]
fn synced_from_toml_source_comment() {
    let content = "# source: Agent.md\ndescription = \"Dev\"\n";
    assert!(is_synced_from(content, "Agent.md"));
}

#[test]
fn synced_from_toml_source_with_prefix() {
    let content = "# source: forge-council/agents/Agent.md\ndescription = \"Dev\"\n";
    assert!(is_synced_from(content, "Agent.md"));
}

#[test]
fn synced_from_toml_source_wrong_file() {
    let content = "# source: Other.md\ndescription = \"Dev\"\n";
    assert!(!is_synced_from(content, "Agent.md"));
}

// --- extract_source_field ---

#[test]
fn extract_source_from_frontmatter() {
    let content = "---\nname: Agent\nsource: forge-council/agents/Agent.md\n---\nBody";
    assert_eq!(
        extract_source_field(content),
        Some("forge-council/agents/Agent.md".into())
    );
}

#[test]
fn extract_source_from_toml_comment() {
    let content = "# source: forge-council/agents/Dev.md\ndescription = \"Dev\"\n";
    assert_eq!(
        extract_source_field(content),
        Some("forge-council/agents/Dev.md".into())
    );
}

#[test]
fn extract_source_none_when_missing() {
    let content = "---\nname: Agent\n---\nBody";
    assert_eq!(extract_source_field(content), None);
}

#[test]
fn extract_source_none_for_empty() {
    assert_eq!(extract_source_field(""), None);
}

#[test]
fn extract_source_bare_filename() {
    let content = "---\nname: Dev\nsource: Dev.md\n---\nBody";
    assert_eq!(extract_source_field(content), Some("Dev.md".into()));
}

#[test]
fn extract_source_user_content_no_source() {
    let content = "User-created agent content.\n";
    assert_eq!(extract_source_field(content), None);
}

// --- module_name ---

#[test]
fn module_name_plain_yaml() {
    assert_eq!(
        module_name("name: forge-council\n"),
        Some("forge-council".into())
    );
}

#[test]
fn module_name_frontmatter() {
    assert_eq!(
        module_name("---\nname: forge-council\n---\n"),
        Some("forge-council".into())
    );
}

#[test]
fn module_name_quoted() {
    assert_eq!(
        module_name("name: \"forge-council\"\n"),
        Some("forge-council".into())
    );
}

#[test]
fn module_name_single_quoted() {
    assert_eq!(
        module_name("name: 'forge-council'\n"),
        Some("forge-council".into())
    );
}

#[test]
fn module_name_missing() {
    assert_eq!(module_name("version: 0.1.0\n"), None);
}

#[test]
fn module_name_empty() {
    assert_eq!(module_name(""), None);
}

// --- proptest ---

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn fm_value_dotted_key_exact_match(
            prefix in "[a-z]{1,10}",
            suffix in "[a-z]{1,10}"
        ) {
            let key = format!("{prefix}.{suffix}");
            let wrong_key = format!("{prefix}X{suffix}");
            let content = format!("---\n{wrong_key}: Wrong\n{key}: Right\n---\n");
            // The exact key should find "Right", not "Wrong"
            prop_assert_eq!(fm_value(&content, &key), Some("Right".to_string()));
        }

        #[test]
        fn validate_agent_name_rejects_path_chars(
            name in "[A-Z][a-zA-Z0-9]{2,10}[/\\.]{1}[a-zA-Z]{1,5}"
        ) {
            // Any name containing /, \, or . should be rejected
            prop_assert!(validate_agent_name(&name).is_err());
        }
    }
}
