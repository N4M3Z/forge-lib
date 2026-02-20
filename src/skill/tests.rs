use super::*;
use crate::sidecar::SidecarConfig;
use std::fs;
use tempfile::TempDir;

fn make_skill_dir(root: &Path, name: &str, md: &str, yaml: Option<&str>) -> PathBuf {
    let dir = root.join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("SKILL.md"), md).unwrap();
    if let Some(y) = yaml {
        fs::write(dir.join("SKILL.yaml"), y).unwrap();
    }
    dir
}

fn config_with_allowlist(dir: &Path, yaml: &str) -> SidecarConfig {
    fs::write(dir.join("defaults.yaml"), yaml).unwrap();
    SidecarConfig::load(dir)
}

// ─── extract_skill_meta ───

#[test]
fn extract_meta_from_skill_md_only() {
    let dir = TempDir::new().unwrap();
    let skill = make_skill_dir(
        dir.path(),
        "Demo",
        "---\nname: Demo\ndescription: A demo skill\n---\n# Demo\n",
        None,
    );
    let meta = extract_skill_meta(&skill).unwrap();
    assert_eq!(meta.name, "Demo");
    assert_eq!(meta.description, "A demo skill");
    assert!(meta.claude_fields.is_empty());
}

#[test]
fn extract_meta_with_claude_fields() {
    let dir = TempDir::new().unwrap();
    let skill = make_skill_dir(
        dir.path(),
        "WikiLink",
        "---\nname: WikiLink\ndescription: Add wikilinks\n---\n# WikiLink\n",
        Some("claude:\n    argument-hint: \"[path]\"\n"),
    );
    let meta = extract_skill_meta(&skill).unwrap();
    assert_eq!(meta.name, "WikiLink");
    assert_eq!(
        meta.claude_fields.get("argument-hint"),
        Some(&"[path]".to_string())
    );
}

#[test]
fn extract_meta_with_bool_claude_field() {
    let dir = TempDir::new().unwrap();
    let skill = make_skill_dir(
        dir.path(),
        "Hidden",
        "---\nname: Hidden\ndescription: Hidden skill\n---\n",
        Some("claude:\n    disable-model-invocation: true\n"),
    );
    let meta = extract_skill_meta(&skill).unwrap();
    assert_eq!(
        meta.claude_fields.get("disable-model-invocation"),
        Some(&"true".to_string())
    );
}

#[test]
fn extract_meta_missing_name_returns_none() {
    let dir = TempDir::new().unwrap();
    let skill = make_skill_dir(
        dir.path(),
        "NoName",
        "---\ndescription: No name\n---\n",
        None,
    );
    assert!(extract_skill_meta(&skill).is_none());
}

#[test]
fn extract_meta_no_skill_md_returns_none() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("Empty");
    fs::create_dir_all(&path).unwrap();
    assert!(extract_skill_meta(&path).is_none());
}

#[test]
fn extract_meta_yaml_without_claude_key() {
    let dir = TempDir::new().unwrap();
    let skill = make_skill_dir(
        dir.path(),
        "Old",
        "---\nname: Old\ndescription: Old format\n---\n",
        Some("providers:\n  claude:\n    enabled: true\n"),
    );
    let meta = extract_skill_meta(&skill).unwrap();
    assert!(meta.claude_fields.is_empty());
}

#[test]
fn extract_meta_corrupt_yaml_ignored() {
    let dir = TempDir::new().unwrap();
    let skill = make_skill_dir(
        dir.path(),
        "Bad",
        "---\nname: Bad\ndescription: Bad yaml\n---\n",
        Some("{{{{ invalid yaml !!!!"),
    );
    let meta = extract_skill_meta(&skill).unwrap();
    assert!(meta.claude_fields.is_empty());
}

// ─── plan_skill_install ───

#[test]
fn plan_copy_when_in_allowlist() {
    let dir = TempDir::new().unwrap();
    let config = config_with_allowlist(dir.path(), "skills:\n    claude:\n        Demo:\n");
    let meta = SkillMeta {
        name: "Demo".into(),
        description: "d".into(),
        claude_fields: BTreeMap::new(),
    };
    let action = plan_skill_install(
        &meta,
        Path::new("/src"),
        Provider::Claude,
        Path::new("/dst"),
        "workspace",
        &config,
    );
    assert!(
        matches!(action, SkillInstallAction::Copy { ref skill_name, .. } if skill_name == "Demo")
    );
}

#[test]
fn plan_skipped_when_not_in_allowlist() {
    let dir = TempDir::new().unwrap();
    let config = config_with_allowlist(dir.path(), "skills:\n    claude:\n        Other:\n");
    let meta = SkillMeta {
        name: "Demo".into(),
        description: "d".into(),
        claude_fields: BTreeMap::new(),
    };
    let action = plan_skill_install(
        &meta,
        Path::new("/src"),
        Provider::Claude,
        Path::new("/dst"),
        "workspace",
        &config,
    );
    assert!(matches!(action, SkillInstallAction::Skipped { .. }));
}

#[test]
fn plan_skipped_when_empty_allowlist() {
    let config = SidecarConfig::default();
    let meta = SkillMeta {
        name: "Demo".into(),
        description: "d".into(),
        claude_fields: BTreeMap::new(),
    };
    let action = plan_skill_install(
        &meta,
        Path::new("/src"),
        Provider::Claude,
        Path::new("/dst"),
        "workspace",
        &config,
    );
    assert!(matches!(action, SkillInstallAction::Skipped { .. }));
}

#[test]
fn plan_gemini_returns_cli_action() {
    let dir = TempDir::new().unwrap();
    let config = config_with_allowlist(dir.path(), "skills:\n    gemini:\n        Demo:\n");
    let meta = SkillMeta {
        name: "Demo".into(),
        description: "d".into(),
        claude_fields: BTreeMap::new(),
    };
    let action = plan_skill_install(
        &meta,
        Path::new("/src"),
        Provider::Gemini,
        Path::new("/dst"),
        "user",
        &config,
    );
    assert!(matches!(action, SkillInstallAction::GeminiCli { ref scope, .. } if scope == "user"));
}

#[test]
fn plan_gemini_scope_from_config() {
    let dir = TempDir::new().unwrap();
    let config = config_with_allowlist(
        dir.path(),
        "skills:\n    gemini:\n        Demo:\n            scope: workspace\n",
    );
    let meta = SkillMeta {
        name: "Demo".into(),
        description: "d".into(),
        claude_fields: BTreeMap::new(),
    };
    let action = plan_skill_install(
        &meta,
        Path::new("/src"),
        Provider::Gemini,
        Path::new("/dst"),
        "user",
        &config,
    );
    assert!(
        matches!(action, SkillInstallAction::GeminiCli { ref scope, .. } if scope == "workspace")
    );
}

#[test]
fn plan_copy_carries_claude_fields() {
    let dir = TempDir::new().unwrap();
    let config = config_with_allowlist(dir.path(), "skills:\n    claude:\n        WikiLink:\n");
    let mut fields = BTreeMap::new();
    fields.insert("argument-hint".into(), "[path]".into());
    let meta = SkillMeta {
        name: "WikiLink".into(),
        description: "d".into(),
        claude_fields: fields,
    };
    let action = plan_skill_install(
        &meta,
        Path::new("/src"),
        Provider::Claude,
        Path::new("/dst"),
        "workspace",
        &config,
    );
    match action {
        SkillInstallAction::Copy {
            ref claude_fields, ..
        } => {
            assert_eq!(
                claude_fields.get("argument-hint"),
                Some(&"[path]".to_string())
            );
        }
        _ => panic!("expected Copy"),
    }
}

// ─── plan_skills_from_dir ───

#[test]
fn plan_from_dir_with_allowlist() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("skills");

    make_skill_dir(
        &root,
        "Alpha",
        "---\nname: Alpha\ndescription: first\n---\n# Alpha\n",
        None,
    );
    make_skill_dir(
        &root,
        "Beta",
        "---\nname: Beta\ndescription: second\n---\n# Beta\n",
        None,
    );
    make_skill_dir(
        &root,
        "Gamma",
        "---\nname: Gamma\ndescription: third\n---\n# Gamma\n",
        None,
    );

    let config = config_with_allowlist(
        dir.path(),
        "skills:\n    claude:\n        Alpha:\n        Gamma:\n",
    );

    let actions = plan_skills_from_dir(
        &root,
        Provider::Claude,
        Path::new("/dst"),
        "workspace",
        &config,
    )
    .unwrap();

    let copy_names: Vec<&str> = actions
        .iter()
        .filter_map(|a| match a {
            SkillInstallAction::Copy { skill_name, .. } => Some(skill_name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(copy_names, vec!["Alpha", "Gamma"]);

    let skipped: Vec<&str> = actions
        .iter()
        .filter_map(|a| match a {
            SkillInstallAction::Skipped { skill_name, .. } => Some(skill_name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(skipped, vec!["Beta"]);
}

#[test]
fn plan_from_dir_no_skill_yaml_needed() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("skills");
    make_skill_dir(
        &root,
        "Simple",
        "---\nname: Simple\ndescription: A simple skill\n---\n# Simple\n",
        None,
    );

    let config = config_with_allowlist(dir.path(), "skills:\n    claude:\n        Simple:\n");

    let actions = plan_skills_from_dir(
        &root,
        Provider::Claude,
        Path::new("/dst"),
        "workspace",
        &config,
    )
    .unwrap();

    assert_eq!(actions.len(), 1);
    assert!(
        matches!(&actions[0], SkillInstallAction::Copy { skill_name, .. } if skill_name == "Simple")
    );
}

#[test]
fn plan_from_dir_empty() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let actions = plan_skills_from_dir(
        dir.path(),
        Provider::Claude,
        Path::new("/dst"),
        "workspace",
        &config,
    )
    .unwrap();
    assert!(actions.is_empty());
}

#[test]
fn plan_from_dir_missing_returns_empty() {
    let config = SidecarConfig::default();
    let actions = plan_skills_from_dir(
        Path::new("/nonexistent"),
        Provider::Claude,
        Path::new("/dst"),
        "workspace",
        &config,
    )
    .unwrap();
    assert!(actions.is_empty());
}

// ─── merge_claude_fields ───

#[test]
fn merge_empty_fields_returns_original() {
    let md = "---\nname: Demo\ndescription: d\n---\n# Demo\n";
    let result = merge_claude_fields(md, &BTreeMap::new());
    assert_eq!(result, md);
}

#[test]
fn merge_adds_fields_to_frontmatter() {
    let md = "---\nname: Demo\ndescription: d\n---\n# Demo\n";
    let mut fields = BTreeMap::new();
    fields.insert("argument-hint".into(), "[path]".into());
    let result = merge_claude_fields(md, &fields);
    assert!(result.contains("argument-hint: [path]"));
    assert!(result.contains("name: Demo"));
    assert!(result.contains("# Demo"));
}

#[test]
fn merge_does_not_duplicate_existing_fields() {
    let md = "---\nname: Demo\ndescription: d\nargument-hint: existing\n---\n# Demo\n";
    let mut fields = BTreeMap::new();
    fields.insert("argument-hint".into(), "[path]".into());
    let result = merge_claude_fields(md, &fields);
    assert_eq!(result.matches("argument-hint").count(), 1);
    assert!(result.contains("argument-hint: existing"));
}

#[test]
fn merge_multiple_fields() {
    let md = "---\nname: Demo\ndescription: d\n---\n# Demo\n";
    let mut fields = BTreeMap::new();
    fields.insert("argument-hint".into(), "[args]".into());
    fields.insert("disable-model-invocation".into(), "true".into());
    let result = merge_claude_fields(md, &fields);
    assert!(result.contains("argument-hint: [args]"));
    assert!(result.contains("disable-model-invocation: true"));
}

#[test]
fn merge_no_frontmatter_wraps() {
    let md = "# Demo\nSome content\n";
    let mut fields = BTreeMap::new();
    fields.insert("argument-hint".into(), "[args]".into());
    let result = merge_claude_fields(md, &fields);
    assert!(result.starts_with("---\n"));
    assert!(result.contains("argument-hint: [args]"));
    assert!(result.contains("# Demo"));
}

// ─── execute_skill_copy ───

#[test]
fn execute_copy_creates_and_copies() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("src_skill");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("SKILL.md"), "# Test").unwrap();
    fs::write(src.join("helper.sh"), "#!/bin/bash").unwrap();

    let dst = dir.path().join("dst");
    execute_skill_copy(&src, "TestSkill", &dst).unwrap();

    assert!(dst.join("TestSkill").join("SKILL.md").exists());
    assert!(dst.join("TestSkill").join("helper.sh").exists());
}

#[test]
fn execute_copy_replaces_existing() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("src_skill");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("SKILL.md"), "# New").unwrap();

    let dst = dir.path().join("dst");
    let existing = dst.join("TestSkill");
    fs::create_dir_all(&existing).unwrap();
    fs::write(existing.join("SKILL.md"), "# Old").unwrap();

    execute_skill_copy(&src, "TestSkill", &dst).unwrap();
    let content = fs::read_to_string(dst.join("TestSkill").join("SKILL.md")).unwrap();
    assert_eq!(content, "# New");
}

// ─── execute_skill_copy: symlink guard ───

#[test]
fn execute_copy_rejects_symlink() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("src_skill");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("SKILL.md"), "# Test").unwrap();

    let dst = dir.path().join("dst");
    fs::create_dir_all(&dst).unwrap();
    let real_target = dir.path().join("real_target");
    fs::create_dir_all(&real_target).unwrap();
    std::os::unix::fs::symlink(&real_target, dst.join("TestSkill")).unwrap();

    let result = execute_skill_copy(&src, "TestSkill", &dst);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("symlink"));
}

// ─── clean_orphaned_skills ───

#[test]
fn orphan_skill_removes_renamed() {
    let dir = TempDir::new().unwrap();
    let dst = dir.path();

    crate::manifest::update(dst, "forge-council", &["OldCouncil".to_string()]).unwrap();

    let old_deployed = dst.join("OldCouncil");
    fs::create_dir_all(&old_deployed).unwrap();
    fs::write(old_deployed.join("SKILL.md"), "# Old").unwrap();

    let current = vec!["NewCouncil".to_string()];
    let removed = clean_orphaned_skills(dst, "forge-council", &current, false).unwrap();
    assert_eq!(removed, vec!["OldCouncil"]);
    assert!(!dst.join("OldCouncil").exists());
}

#[test]
fn orphan_skill_keeps_current() {
    let dir = TempDir::new().unwrap();
    let dst = dir.path();

    crate::manifest::update(dst, "forge-council", &["Council".to_string()]).unwrap();
    let deployed = dst.join("Council");
    fs::create_dir_all(&deployed).unwrap();
    fs::write(deployed.join("SKILL.md"), "# Council").unwrap();

    let current = vec!["Council".to_string()];
    let removed = clean_orphaned_skills(dst, "forge-council", &current, false).unwrap();
    assert!(removed.is_empty());
    assert!(dst.join("Council").exists());
}

#[test]
fn orphan_skill_dry_run_preserves() {
    let dir = TempDir::new().unwrap();
    let dst = dir.path();

    crate::manifest::update(dst, "forge-council", &["OldSkill".to_string()]).unwrap();
    let deployed = dst.join("OldSkill");
    fs::create_dir_all(&deployed).unwrap();

    let removed = clean_orphaned_skills(dst, "forge-council", &[], true).unwrap();
    assert_eq!(removed, vec!["OldSkill"]);
    assert!(dst.join("OldSkill").exists());
}

#[test]
fn orphan_skill_empty_module_skips() {
    let dir = TempDir::new().unwrap();
    let removed = clean_orphaned_skills(dir.path(), "", &[], false).unwrap();
    assert!(removed.is_empty());
}

// ─── Skill Generation (Codex wrappers) ───

#[test]
fn generate_uses_claude_name() {
    let content = "---\nclaude.name: Dev\ntitle: Developer\nclaude.description: A dev\n---\nBody\n";
    let result = generate_skill_from_agent(content, "Dev.md").unwrap();
    assert_eq!(result.agent_name, "Dev");
}

#[test]
fn generate_falls_back_to_title() {
    let content = "---\ntitle: Helper\ndescription: A helper\n---\nBody\n";
    let result = generate_skill_from_agent(content, "Helper.md").unwrap();
    assert_eq!(result.agent_name, "Helper");
}

#[test]
fn generate_missing_name_returns_none() {
    let content = "---\ndescription: No name\n---\nBody\n";
    assert!(generate_skill_from_agent(content, "test.md").is_none());
}

#[test]
fn generate_default_description() {
    let content = "---\nclaude.name: Agent\n---\nBody\n";
    let result = generate_skill_from_agent(content, "Agent.md").unwrap();
    assert!(result.skill_md.contains("Specialist skill"));
    assert!(result.skill_yaml.contains("Specialist skill"));
}

#[test]
fn generate_from_agents_dir() {
    let dir = TempDir::new().unwrap();
    let agents = dir.path().join("agents");
    fs::create_dir_all(&agents).unwrap();
    fs::write(
        agents.join("Dev.md"),
        "---\nclaude.name: Dev\nclaude.description: Developer\n---\nDev body\n",
    )
    .unwrap();
    fs::write(
        agents.join("Tester.md"),
        "---\nclaude.name: Tester\nclaude.description: QA\n---\nTest body\n",
    )
    .unwrap();

    let results = generate_skills_from_agents_dir(&agents).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].agent_name, "Dev");
    assert_eq!(results[1].agent_name, "Tester");
}

#[test]
fn generate_from_missing_dir() {
    let results = generate_skills_from_agents_dir(Path::new("/nonexistent")).unwrap();
    assert!(results.is_empty());
}

#[test]
fn format_skill_md_structure() {
    let md = format_agent_skill_md("Agent", "A specialist", "Do things.\n", "Agent.md");
    assert!(md.starts_with("---\n"));
    assert!(md.contains("name: Agent"));
    assert!(md.contains("description: \"A specialist\""));
    assert!(md.contains("# Agent"));
    assert!(md.contains("Generated from agents/Agent.md"));
    assert!(md.contains("Do things."));
}

#[test]
fn format_skill_yaml_codex_only() {
    let yaml = format_agent_skill_yaml("Agent", "A specialist", "Agent.md");
    assert!(yaml.contains("name: Agent"));
    let lines: Vec<&str> = yaml.lines().collect();
    let claude_enabled = lines.iter().position(|l| l.contains("claude:")).unwrap();
    assert!(lines[claude_enabled + 1].contains("enabled: false"));
    let codex_enabled = lines.iter().position(|l| l.contains("codex:")).unwrap();
    assert!(lines[codex_enabled + 1].contains("enabled: true"));
}

#[test]
fn format_skill_yaml_escapes_quotes() {
    let yaml = format_agent_skill_yaml("Agent", "A \"quoted\" desc", "Agent.md");
    assert!(yaml.contains("A \\\"quoted\\\" desc"));
}
