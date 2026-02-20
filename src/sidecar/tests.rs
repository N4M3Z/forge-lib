use super::*;
use std::fs;
use tempfile::TempDir;

fn write_yaml(dir: &Path, filename: &str, content: &str) {
    fs::write(dir.join(filename), content).unwrap();
}

// --- ModelTiers ---

#[test]
fn default_tiers() {
    let tiers = ModelTiers::default();
    assert_eq!(tiers.fast, "sonnet");
    assert_eq!(tiers.strong, "opus");
}

// --- resolve_model ---

#[test]
fn resolve_fast_alias() {
    let global = ModelTiers::default();
    assert_eq!(resolve_model("fast", &global, &global), "sonnet");
}

#[test]
fn resolve_strong_alias() {
    let global = ModelTiers::default();
    assert_eq!(resolve_model("strong", &global, &global), "opus");
}

#[test]
fn resolve_fast_default_to_provider() {
    let global = ModelTiers::default();
    let gemini = ModelTiers {
        fast: "gemini-2.0-flash".into(),
        strong: "gemini-2.5-pro".into(),
    };
    assert_eq!(
        resolve_model("sonnet", &global, &gemini),
        "gemini-2.0-flash"
    );
    assert_eq!(resolve_model("fast", &global, &gemini), "gemini-2.0-flash");
}

#[test]
fn resolve_strong_default_to_provider() {
    let global = ModelTiers::default();
    let gemini = ModelTiers {
        fast: "gemini-2.0-flash".into(),
        strong: "gemini-2.5-pro".into(),
    };
    assert_eq!(resolve_model("opus", &global, &gemini), "gemini-2.5-pro");
    assert_eq!(resolve_model("strong", &global, &gemini), "gemini-2.5-pro");
}

#[test]
fn resolve_same_provider_noop() {
    let global = ModelTiers::default();
    assert_eq!(resolve_model("sonnet", &global, &global), "sonnet");
    assert_eq!(resolve_model("opus", &global, &global), "opus");
}

#[test]
fn resolve_passthrough() {
    let global = ModelTiers::default();
    assert_eq!(
        resolve_model("gemini-2.5-pro", &global, &global),
        "gemini-2.5-pro"
    );
}

#[test]
fn resolve_empty_string() {
    let global = ModelTiers::default();
    assert_eq!(resolve_model("", &global, &global), "");
}

// --- SidecarConfig::load ---

#[test]
fn load_defaults_yaml() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "shared:\n  models:\n    fast: haiku\n    strong: sonnet\n",
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "haiku");
    assert_eq!(tiers.strong, "sonnet");
}

#[test]
fn load_config_overrides_defaults() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "shared:\n  models:\n    fast: haiku\n    strong: sonnet\n",
    );
    write_yaml(
        dir.path(),
        "config.yaml",
        "shared:\n  models:\n    fast: sonnet\n",
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "sonnet");
    assert_eq!(tiers.strong, "sonnet");
}

#[test]
fn load_missing_dir_returns_defaults() {
    let config = SidecarConfig::load(Path::new("/nonexistent/path/that/wont/exist"));
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "sonnet");
    assert_eq!(tiers.strong, "opus");
}

#[test]
fn load_corrupt_yaml_returns_defaults() {
    let dir = TempDir::new().unwrap();
    write_yaml(dir.path(), "defaults.yaml", "{{{{invalid yaml!!!!}}}}");
    let config = SidecarConfig::load(dir.path());
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "sonnet");
    assert_eq!(tiers.strong, "opus");
}

#[test]
fn load_empty_yaml_returns_defaults() {
    let dir = TempDir::new().unwrap();
    write_yaml(dir.path(), "defaults.yaml", "");
    let config = SidecarConfig::load(dir.path());
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "sonnet");
    assert_eq!(tiers.strong, "opus");
}

#[test]
fn load_yml_extension() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yml",
        "shared:\n  models:\n    fast: haiku\n    strong: sonnet\n",
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "haiku");
    assert_eq!(tiers.strong, "sonnet");
}

#[test]
fn load_yaml_takes_priority_over_yml() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "shared:\n  models:\n    fast: sonnet\n",
    );
    write_yaml(
        dir.path(),
        "defaults.yml",
        "shared:\n  models:\n    fast: haiku\n",
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "sonnet");
}

// --- provider_tiers ---

#[test]
fn provider_specific_override() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        concat!(
            "shared:\n  models:\n    fast: sonnet\n    strong: opus\n",
            "providers:\n  gemini:\n    models:\n      - gemini-2.0-flash\n      - gemini-2.5-pro\n",
            "    fast: gemini-2.0-flash\n    strong: gemini-2.5-pro\n",
        ),
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.provider_tiers("gemini");
    assert_eq!(tiers.fast, "gemini-2.0-flash");
    assert_eq!(tiers.strong, "gemini-2.5-pro");
}

#[test]
fn provider_partial_override_falls_back_to_global() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        concat!(
            "shared:\n  models:\n    fast: sonnet\n    strong: opus\n",
            "providers:\n  claude:\n    fast: claude-sonnet-4-6\n",
        ),
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.provider_tiers("claude");
    assert_eq!(tiers.fast, "claude-sonnet-4-6");
    assert_eq!(tiers.strong, "opus");
}

#[test]
fn provider_missing_falls_back_to_global() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "shared:\n  models:\n    fast: haiku\n    strong: sonnet\n",
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.provider_tiers("nonexistent");
    assert_eq!(tiers.fast, "haiku");
    assert_eq!(tiers.strong, "sonnet");
}

// --- is_model_whitelisted ---

#[test]
fn whitelist_model_present() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "providers:\n  claude:\n    whitelist:\n      - sonnet\n      - opus\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert!(config.is_model_whitelisted("claude", "sonnet"));
}

#[test]
fn whitelist_model_absent() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "providers:\n  claude:\n    whitelist:\n      - sonnet\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert!(!config.is_model_whitelisted("claude", "haiku"));
}

#[test]
fn whitelist_missing_allows_all() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "providers:\n  claude:\n    models:\n    fast: sonnet\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert!(config.is_model_whitelisted("claude", "anything"));
}

#[test]
fn whitelist_empty_denies_all() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "providers:\n  claude:\n    whitelist: []\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert!(!config.is_model_whitelisted("claude", "sonnet"));
}

#[test]
fn whitelist_no_provider_allows_all() {
    let config = SidecarConfig::default();
    assert!(config.is_model_whitelisted("anything", "any_model"));
}

// --- agent_value ---

#[test]
fn agent_specific_model() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "agents:\n  Opponent:\n    model: strong\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(
        config.agent_value("Opponent", "model"),
        Some("strong".into())
    );
}

#[test]
fn agent_specific_tools() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "agents:\n  Developer:\n    tools: Read, Write, Bash\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(
        config.agent_value("Developer", "tools"),
        Some("Read, Write, Bash".into())
    );
}

#[test]
fn agent_missing_returns_none() {
    let config = SidecarConfig::default();
    assert_eq!(config.agent_value("NonExistent", "model"), None);
}

// --- skill_value ---

#[test]
fn skill_scope_from_skills_section() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "skills:\n    DeveloperCouncil:\n        scope: workspace\n    DebateCouncil:\n        scope: user\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(
        config.skill_value("DeveloperCouncil", "scope"),
        Some("workspace".into())
    );
    assert_eq!(
        config.skill_value("DebateCouncil", "scope"),
        Some("user".into())
    );
}

#[test]
fn skill_scope_from_root_level_fallback() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "DeveloperCouncil:\n  scope: workspace\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(
        config.skill_value("DeveloperCouncil", "scope"),
        Some("workspace".into())
    );
}

#[test]
fn skill_value_missing_returns_none() {
    let config = SidecarConfig::default();
    assert_eq!(config.skill_value("NonExistent", "scope"), None);
}

// --- flat YAML structure (forge-council style) ---

#[test]
fn flat_global_tiers() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "models:\n  fast: haiku\n  strong: sonnet\n",
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "haiku");
    assert_eq!(tiers.strong, "sonnet");
}

#[test]
fn flat_provider_tiers() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        concat!(
            "models:\n  fast: sonnet\n  strong: opus\n",
            "gemini:\n  fast: gemini-2.0-flash\n  strong: gemini-2.5-pro\n",
        ),
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.provider_tiers("gemini");
    assert_eq!(tiers.fast, "gemini-2.0-flash");
    assert_eq!(tiers.strong, "gemini-2.5-pro");
}

#[test]
fn flat_agent_value() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "Developer:\n  model: fast\n  tools: Read, Write, Bash\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(
        config.agent_value("Developer", "model"),
        Some("fast".into())
    );
    assert_eq!(
        config.agent_value("Developer", "tools"),
        Some("Read, Write, Bash".into())
    );
}

#[test]
fn flat_whitelist() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "gemini:\n  models:\n    - gemini-2.0-flash\n    - gemini-2.5-pro\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert!(config.is_model_whitelisted("gemini", "gemini-2.0-flash"));
    assert!(!config.is_model_whitelisted("gemini", "sonnet"));
}

#[test]
fn nested_takes_priority_over_flat() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        concat!(
            "shared:\n  models:\n    fast: nested-fast\n",
            "models:\n  fast: flat-fast\n",
        ),
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "nested-fast");
}

// --- merge_values ---

#[test]
fn merge_overlay_wins_for_scalars() {
    let base: Value = serde_yaml::from_str("key: base").unwrap();
    let overlay: Value = serde_yaml::from_str("key: overlay").unwrap();
    let merged = merge_values(base, overlay);
    let result = merged
        .as_mapping()
        .unwrap()
        .get(Value::String("key".into()))
        .unwrap();
    assert_eq!(result, &Value::String("overlay".into()));
}

#[test]
fn merge_preserves_base_keys() {
    let base: Value = serde_yaml::from_str("a: 1\nb: 2").unwrap();
    let overlay: Value = serde_yaml::from_str("b: 3").unwrap();
    let merged = merge_values(base, overlay);
    let map = merged.as_mapping().unwrap();
    assert_eq!(
        map.get(Value::String("a".into())),
        Some(&Value::Number(1.into()))
    );
    assert_eq!(
        map.get(Value::String("b".into())),
        Some(&Value::Number(3.into()))
    );
}

#[test]
fn merge_deep_nested() {
    let base: Value =
        serde_yaml::from_str("shared:\n  models:\n    fast: haiku\n    strong: opus").unwrap();
    let overlay: Value = serde_yaml::from_str("shared:\n  models:\n    fast: sonnet").unwrap();
    let merged = merge_values(base, overlay);
    let fast = navigate(&merged, &["shared", "models", "fast"]).unwrap();
    let strong = navigate(&merged, &["shared", "models", "strong"]).unwrap();
    assert_eq!(fast, Value::String("sonnet".into()));
    assert_eq!(strong, Value::String("opus".into()));
}

// --- provider_reasoning_effort ---

#[test]
fn reasoning_effort_reads_tier() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        concat!(
            "providers:\n  codex:\n    reasoning_effort:\n",
            "      fast: low\n      strong: medium\n",
        ),
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(
        config.provider_reasoning_effort("codex", "fast"),
        Some("low".into())
    );
    assert_eq!(
        config.provider_reasoning_effort("codex", "strong"),
        Some("medium".into())
    );
}

#[test]
fn reasoning_effort_missing_returns_none() {
    let config = SidecarConfig::default();
    assert_eq!(config.provider_reasoning_effort("codex", "fast"), None);
}

#[test]
fn reasoning_effort_flat_fallback() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "codex:\n  reasoning_effort:\n    fast: low\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(
        config.provider_reasoning_effort("codex", "fast"),
        Some("low".into())
    );
}

// --- provider_skills ---

#[test]
fn provider_skills_returns_map_keys() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "skills:\n    claude:\n        DebateCouncil:\n        Demo:\n        DeveloperCouncil:\n            scope: workspace\n",
    );
    let config = SidecarConfig::load(dir.path());
    let skills = config.provider_skills("claude");
    assert_eq!(skills, vec!["DebateCouncil", "Demo", "DeveloperCouncil"]);
}

#[test]
fn provider_skills_missing_provider_returns_empty() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "skills:\n    claude:\n        Demo:\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert!(config.provider_skills("gemini").is_empty());
}

#[test]
fn provider_skills_missing_skills_key_returns_empty() {
    let dir = TempDir::new().unwrap();
    write_yaml(dir.path(), "defaults.yaml", "agents:\n    Foo:\n");
    let config = SidecarConfig::load(dir.path());
    assert!(config.provider_skills("claude").is_empty());
}

#[test]
fn provider_skills_empty_config_returns_empty() {
    let config = SidecarConfig::default();
    assert!(config.provider_skills("claude").is_empty());
}

#[test]
fn provider_skills_null_values_are_keys() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "skills:\n    codex:\n        CleanText:\n        Summarize:\n",
    );
    let config = SidecarConfig::load(dir.path());
    let skills = config.provider_skills("codex");
    assert_eq!(skills, vec!["CleanText", "Summarize"]);
}

// --- provider_skill_value ---

#[test]
fn provider_skill_value_reads_nested_config() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "skills:\n    claude:\n        DeveloperCouncil:\n            scope: workspace\n            roles:\n                - Dev\n                - QA\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(
        config.provider_skill_value("claude", "DeveloperCouncil", "scope"),
        Some("workspace".into())
    );
}

#[test]
fn provider_skill_value_null_entry_returns_none() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "skills:\n    claude:\n        Demo:\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(config.provider_skill_value("claude", "Demo", "scope"), None);
}

#[test]
fn provider_skill_value_missing_skill_returns_none() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "skills:\n    claude:\n        Demo:\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(
        config.provider_skill_value("claude", "NonExistent", "scope"),
        None
    );
}

#[test]
fn provider_skill_value_config_override() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "skills:\n    claude:\n        Council:\n            scope: workspace\n",
    );
    write_yaml(
        dir.path(),
        "config.yaml",
        "skills:\n    claude:\n        Council:\n            scope: user\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(
        config.provider_skill_value("claude", "Council", "scope"),
        Some("user".into())
    );
}

// --- proptest ---

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn load_never_panics(content in ".*") {
            let dir = TempDir::new().unwrap();
            write_yaml(dir.path(), "defaults.yaml", &content);
            let config = SidecarConfig::load(dir.path());
            // Should always produce valid tiers, never panic
            let tiers = config.global_tiers();
            prop_assert!(!tiers.fast.is_empty());
            prop_assert!(!tiers.strong.is_empty());
        }

        #[test]
        fn resolve_model_never_panics(model in "\\PC{0,100}") {
            let global = ModelTiers::default();
            let _ = resolve_model(&model, &global, &global);
        }
    }
}
