use crate::deploy::deploy_agents_from_dir;
use crate::deploy::provider::Provider;
use crate::parse;
use crate::sidecar::SidecarConfig;
use std::fs;
use std::path::Path;

pub struct Check {
    pub desc: String,
    pub passed: bool,
}

impl Check {
    fn pass(desc: impl Into<String>) -> Self {
        Self {
            desc: desc.into(),
            passed: true,
        }
    }
    fn fail(desc: impl Into<String>) -> Self {
        Self {
            desc: desc.into(),
            passed: false,
        }
    }
}

pub struct Suite {
    pub name: String,
    pub checks: Vec<Check>,
}

impl Suite {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            checks: Vec::new(),
        }
    }

    fn assert_file_exists(&mut self, desc: &str, path: &Path) {
        self.checks.push(if path.is_file() {
            Check::pass(desc)
        } else {
            Check::fail(desc)
        });
    }

    fn assert_not_empty(&mut self, desc: &str, value: &str) {
        self.checks.push(if value.is_empty() {
            Check::fail(desc)
        } else {
            Check::pass(desc)
        });
    }

    fn assert_eq(&mut self, desc: &str, expected: &str, actual: &str) {
        self.checks.push(if expected == actual {
            Check::pass(desc)
        } else {
            Check::fail(desc)
        });
    }

    fn assert_contains(&mut self, desc: &str, haystack: &str, needle: &str) {
        self.checks.push(if haystack.contains(needle) {
            Check::pass(desc)
        } else {
            Check::fail(desc)
        });
    }

    fn assert_match(&mut self, desc: &str, value: &str, pattern: &str) {
        let re = regex::Regex::new(pattern).unwrap();
        self.checks.push(if re.is_match(value) {
            Check::pass(desc)
        } else {
            Check::fail(desc)
        });
    }

    pub fn passed(&self) -> usize {
        self.checks.iter().filter(|c| c.passed).count()
    }

    pub fn failed(&self) -> usize {
        self.checks.iter().filter(|c| !c.passed).count()
    }
}

// --- Suite 1: Module Structure ---

pub fn validate_structure(root: &Path) -> Suite {
    let mut s = Suite::new("Module Structure");

    let yaml_path = root.join("module.yaml");
    s.assert_file_exists("module.yaml exists", &yaml_path);

    if let Ok(content) = fs::read_to_string(&yaml_path) {
        for key in &["name", "version", "description"] {
            let val = yaml_value(&content, key);
            s.assert_not_empty(&format!("module.yaml has {key}"), &val);
        }
    }

    let pjson_path = root.join(".claude-plugin/plugin.json");
    s.assert_file_exists("plugin.json exists", &pjson_path);

    if let Ok(content) = fs::read_to_string(&pjson_path) {
        let valid = serde_json::from_str::<serde_json::Value>(&content).is_ok();
        s.checks.push(if valid {
            Check::pass("plugin.json is valid JSON")
        } else {
            Check::fail("plugin.json is not valid JSON")
        });
    }

    s.assert_file_exists("lib/Makefile exists", &root.join("lib/Makefile"));

    s
}

// --- Suite 2: Agent Frontmatter ---

fn read_agents(agents_dir: &Path) -> Vec<(String, String)> {
    let Ok(entries) = fs::read_dir(agents_dir) else {
        return Vec::new();
    };
    let mut agents: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .filter_map(|e| {
            let name = e.path().file_stem()?.to_string_lossy().to_string();
            let content = fs::read_to_string(e.path()).ok()?;
            Some((name, content))
        })
        .collect();
    agents.sort_by(|a, b| a.0.cmp(&b.0));
    agents
}

fn roster_names(defaults_content: &str) -> Vec<String> {
    let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(defaults_content) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    if let Some(agents) = yaml.get("agents") {
        for section in &["council", "standalone"] {
            if let Some(serde_yaml::Value::Sequence(list)) = agents.get(section) {
                for item in list {
                    if let Some(s) = item.as_str() {
                        names.push(s.to_string());
                    }
                }
            }
        }
    }
    names
}

fn council_names(defaults_content: &str) -> Vec<String> {
    let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(defaults_content) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    if let Some(serde_yaml::Value::Mapping(councils)) = yaml.get("councils") {
        for (key, _) in councils {
            if let Some(s) = key.as_str() {
                names.push(s.to_string());
            }
        }
    }
    names
}

fn council_roles(defaults_content: &str, council_name: &str) -> Vec<String> {
    let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(defaults_content) else {
        return Vec::new();
    };
    let mut roles = Vec::new();
    if let Some(council) = yaml.get("councils").and_then(|c| c.get(council_name)) {
        if let Some(serde_yaml::Value::Sequence(list)) = council.get("roles") {
            for item in list {
                if let Some(s) = item.as_str() {
                    roles.push(s.to_string());
                }
            }
        }
    }
    roles
}

fn has_config_block(defaults_content: &str, agent_name: &str) -> bool {
    let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(defaults_content) else {
        return false;
    };
    let Some(block) = yaml.get(agent_name) else {
        return false;
    };
    block.get("model").is_some() && block.get("tools").is_some()
}

fn check_agent_body_conventions(s: &mut Suite, agents: &[(String, String)]) {
    let required_sections = [
        "## Role",
        "## Expertise",
        "## Instructions",
        "## Output Format",
        "## Constraints",
    ];
    for (_, content) in agents {
        let name = parse::fm_value(content, "claude.name").unwrap_or_default();
        let body = parse::fm_body(content);
        for heading in &required_sections {
            s.assert_contains(&format!("{name}: has '{heading}'"), body, heading);
        }
    }

    for (_, content) in agents {
        let name = parse::fm_value(content, "claude.name").unwrap_or_default();
        let body = parse::fm_body(content);
        s.assert_contains(&format!("{name}: honesty clause (say so)"), body, "say so");
    }

    for (_, content) in agents {
        let name = parse::fm_value(content, "claude.name").unwrap_or_default();
        let body = parse::fm_body(content);
        s.assert_contains(
            &format!("{name}: team clause (SendMessage)"),
            body,
            "SendMessage",
        );
    }

    for (_, content) in agents {
        let name = parse::fm_value(content, "claude.name").unwrap_or_default();
        let body = parse::fm_body(content);
        s.assert_contains(
            &format!("{name}: shipped-with marker"),
            body,
            "Shipped with forge-",
        );
    }
}

pub fn validate_agent_frontmatter(root: &Path) -> Suite {
    let mut s = Suite::new("Agent Frontmatter");
    let agents_dir = root.join("agents");
    let agents = read_agents(&agents_dir);

    let defaults_content = fs::read_to_string(root.join("defaults.yaml")).unwrap_or_default();
    let roster = roster_names(&defaults_content);

    s.assert_eq(
        &format!(
            "agent_count_matches_roster (files={}, roster={})",
            agents.len(),
            roster.len()
        ),
        &roster.len().to_string(),
        &agents.len().to_string(),
    );

    let required_keys = [
        "title",
        "description",
        "claude.name",
        "claude.model",
        "claude.description",
        "claude.tools",
    ];

    for (name, content) in &agents {
        for key in &required_keys {
            let val = parse::fm_value(content, key).unwrap_or_default();
            s.assert_not_empty(&format!("{name} has {key}"), &val);
        }
    }

    for (name, content) in &agents {
        let claude_name = parse::fm_value(content, "claude.name").unwrap_or_default();
        s.assert_eq(
            &format!("{name}: filename matches claude.name"),
            name,
            &claude_name,
        );
    }

    for (_, content) in &agents {
        let name = parse::fm_value(content, "claude.name").unwrap_or_default();
        s.assert_match(
            &format!("{name} is PascalCase"),
            &name,
            r"^[A-Z][a-zA-Z0-9]+$",
        );
    }

    let valid_models = ["sonnet", "opus", "haiku", "fast", "strong"];
    for (_, content) in &agents {
        let name = parse::fm_value(content, "claude.name").unwrap_or_default();
        let model = parse::fm_value(content, "claude.model").unwrap_or_default();
        let is_valid = valid_models.contains(&model.as_str());
        s.checks.push(if is_valid {
            Check::pass(format!("{name}: model '{model}' is valid"))
        } else {
            Check::fail(format!("{name}: model '{model}' is not valid"))
        });
    }

    for (_, content) in &agents {
        let name = parse::fm_value(content, "claude.name").unwrap_or_default();
        let desc = parse::fm_value(content, "claude.description").unwrap_or_default();
        s.assert_contains(
            &format!("{name}: description has USE WHEN"),
            &desc,
            "USE WHEN",
        );
    }

    check_agent_body_conventions(&mut s, &agents);

    s
}

// --- Suite 3: Defaults Consistency ---

pub fn validate_defaults(root: &Path) -> Suite {
    let mut s = Suite::new("Defaults Consistency");
    let agents_dir = root.join("agents");
    let defaults_content = fs::read_to_string(root.join("defaults.yaml")).unwrap_or_default();

    let roster = roster_names(&defaults_content);

    for name in &roster {
        s.assert_file_exists(
            &format!("roster agent {name} exists"),
            &agents_dir.join(format!("{name}.md")),
        );
    }

    let councils = council_names(&defaults_content);
    for council_name in &councils {
        let roles = council_roles(&defaults_content, council_name);
        for role in &roles {
            let found = roster.iter().any(|r| r == role);
            s.checks.push(if found {
                Check::pass(format!(
                    "council '{council_name}' role '{role}' is in roster"
                ))
            } else {
                Check::fail(format!(
                    "council '{council_name}' role '{role}' is in roster"
                ))
            });
        }
    }

    for name in &roster {
        let has = has_config_block(&defaults_content, name);
        s.checks.push(if has {
            Check::pass(format!("{name} has config block (model + tools)"))
        } else {
            Check::fail(format!(
                "{name} missing config block (model + tools) in defaults.yaml"
            ))
        });
    }

    s
}

// --- Suite 4: Skill Integrity ---

fn read_skill_dirs(skills_dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(skills_dir) else {
        return Vec::new();
    };
    let mut names: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}

fn yaml_value(content: &str, key: &str) -> String {
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix(key) {
            if let Some(val) = rest.strip_prefix(':') {
                let val = val.trim();
                let val = val.trim_matches('"').trim_matches('\'');
                if !val.is_empty() {
                    return val.to_string();
                }
            }
        }
    }
    String::new()
}

pub fn validate_skills(root: &Path) -> Suite {
    let mut s = Suite::new("Skill Integrity");
    let skills_dir = root.join("skills");
    let skill_names = read_skill_dirs(&skills_dir);

    for name in &skill_names {
        let dir = skills_dir.join(name);
        s.assert_file_exists(&format!("{name} has SKILL.md"), &dir.join("SKILL.md"));
        s.assert_file_exists(&format!("{name} has SKILL.yaml"), &dir.join("SKILL.yaml"));
    }

    for name in &skill_names {
        let yaml_path = skills_dir.join(name).join("SKILL.yaml");
        let Ok(content) = fs::read_to_string(&yaml_path) else {
            continue;
        };
        for key in &["name", "description"] {
            let val = yaml_value(&content, key);
            s.assert_not_empty(&format!("{name} SKILL.yaml has {key}"), &val);
        }
    }

    for name in &skill_names {
        let yaml_path = skills_dir.join(name).join("SKILL.yaml");
        let Ok(content) = fs::read_to_string(&yaml_path) else {
            continue;
        };
        let yaml_name = yaml_value(&content, "name");
        s.assert_eq(
            &format!("{name}: SKILL.yaml name matches directory"),
            name,
            &yaml_name,
        );
    }

    for name in &skill_names {
        let md_path = skills_dir.join(name).join("SKILL.md");
        let Ok(content) = fs::read_to_string(&md_path) else {
            continue;
        };
        let fm_name = parse::fm_value(&content, "name").unwrap_or_default();
        let fm_desc = parse::fm_value(&content, "description").unwrap_or_default();
        s.assert_not_empty(&format!("{name} SKILL.md has name"), &fm_name);
        s.assert_not_empty(&format!("{name} SKILL.md has description"), &fm_desc);
    }

    s
}

/// Content-level checks that emit warnings, not failures.
/// These patterns are valuable but need proper scoping (e.g., agent-team
/// checks should only apply to council modules). Tracked as backlog item.
pub fn warn_skill_content(root: &Path) -> Suite {
    let mut s = Suite::new("Skill Content (warnings)");
    let skills_dir = root.join("skills");
    let skill_names = read_skill_dirs(&skills_dir);

    for name in &skill_names {
        if name == "Demo" {
            continue;
        }
        let md_path = skills_dir.join(name).join("SKILL.md");
        let Ok(content) = fs::read_to_string(&md_path) else {
            continue;
        };
        let body = parse::fm_body(&content);
        s.assert_contains(&format!("{name}: has Gate Check"), body, "Gate Check");
        s.assert_contains(
            &format!("{name}: has Sequential Fallback"),
            body,
            "Sequential Fallback",
        );
    }

    s
}

// --- Suite 5: Deploy Parity ---

fn count_md_files(dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .count()
}

fn provider_label(path: &Path) -> String {
    let s = path.to_string_lossy();
    if s.contains(".gemini") {
        ".gemini".to_string()
    } else if s.contains(".codex") {
        ".codex".to_string()
    } else {
        ".claude".to_string()
    }
}

fn sorted_md_entries(dir: &Path) -> Vec<std::fs::DirEntry> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    files.sort_by_key(std::fs::DirEntry::file_name);
    files
}

fn check_synced_from(s: &mut Suite, provider_dirs: &[(&std::path::PathBuf, Provider)]) {
    for (dst, _) in provider_dirs {
        let label = provider_label(dst);
        for entry in sorted_md_entries(dst) {
            let name = entry
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let content = fs::read_to_string(entry.path()).unwrap_or_default();
            let has_source = parse::fm_value(&content, "source").is_some()
                || content.lines().any(|l| l.starts_with("# synced-from:"));
            s.checks.push(if has_source {
                Check::pass(format!("{label}/{name} has source"))
            } else {
                Check::fail(format!("{label}/{name} missing source field"))
            });
        }
    }
}

fn check_body_matches_source(s: &mut Suite, claude_dst: &Path, agents_dir: &Path) {
    for entry in sorted_md_entries(claude_dst) {
        let name = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let source_path = agents_dir.join(format!("{name}.md"));
        if !source_path.is_file() {
            continue;
        }

        let source_content = fs::read_to_string(&source_path).unwrap_or_default();
        let source_body = parse::fm_body(&source_content).trim_end_matches('\n');

        let deployed_content = fs::read_to_string(entry.path()).unwrap_or_default();
        let deployed_body = extract_deployed_body(&deployed_content).trim_end_matches('\n');

        s.checks.push(if source_body == deployed_body {
            Check::pass(format!("{name}: deployed body matches source"))
        } else {
            Check::fail(format!("{name}: deployed body differs from source"))
        });
    }
}

fn check_gemini_formatting(s: &mut Suite, gemini_dst: &Path) {
    let slug_re = regex::Regex::new(r"^[a-z][a-z0-9-]*$").unwrap();
    let claude_tools = [
        "Read",
        "Write",
        "Edit",
        "Grep",
        "Glob",
        "Bash",
        "WebSearch",
        "WebFetch",
    ];

    for entry in sorted_md_entries(gemini_dst) {
        let filename = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let content = fs::read_to_string(entry.path()).unwrap_or_default();

        let gemini_name = parse::fm_value(&content, "name").unwrap_or_default();
        s.checks.push(if slug_re.is_match(&gemini_name) {
            Check::pass(format!(
                "{filename}: gemini name '{gemini_name}' is slugified"
            ))
        } else {
            Check::fail(format!(
                "{filename}: gemini name '{gemini_name}' is not slugified"
            ))
        });

        let has_unmapped = content.lines().any(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("- ")
                .is_some_and(|val| claude_tools.contains(&val.trim()))
        });

        s.checks.push(if has_unmapped {
            Check::fail(format!(
                "{filename}: unmapped Claude tool name found in Gemini frontmatter"
            ))
        } else {
            Check::pass(format!(
                "{filename}: no unmapped Claude tool names in Gemini frontmatter"
            ))
        });
    }
}

fn check_model_resolved(s: &mut Suite, provider_dirs: &[(&std::path::PathBuf, Provider)]) {
    for (dst, _) in provider_dirs {
        let label = provider_label(dst);
        for entry in sorted_md_entries(dst) {
            let name = entry
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let content = fs::read_to_string(entry.path()).unwrap_or_default();
            let model = parse::fm_value(&content, "model").unwrap_or_default();
            let resolved = model != "fast" && model != "strong";
            s.checks.push(if resolved {
                Check::pass(format!("{label}/{name}: model '{model}' resolved"))
            } else {
                Check::fail(format!("{label}/{name}: model '{model}' not resolved"))
            });
        }
    }
}

pub fn validate_deploy_parity(root: &Path) -> Suite {
    let mut s = Suite::new("Deploy Parity");
    let agents_dir = root.join("agents");

    if !agents_dir.is_dir() {
        return s;
    }

    let config = SidecarConfig::load(root);

    let Ok(tmp) = tempfile::tempdir() else {
        return s;
    };

    let claude_dst = tmp.path().join(".claude/agents");
    let gemini_dst = tmp.path().join(".gemini/agents");
    let codex_dst = tmp.path().join(".codex/agents");

    let provider_dirs: Vec<_> = vec![
        (&claude_dst, Provider::Claude),
        (&gemini_dst, Provider::Gemini),
        (&codex_dst, Provider::Codex),
    ];

    for (dst, provider) in &provider_dirs {
        let _ = fs::create_dir_all(dst);
        let _ = deploy_agents_from_dir(&agents_dir, dst, *provider, &config, false, "");
    }

    let claude_count = count_md_files(&claude_dst);
    let gemini_count = count_md_files(&gemini_dst);
    let codex_count = count_md_files(&codex_dst);

    s.assert_eq(
        &format!("claude count ({claude_count}) == gemini count ({gemini_count})"),
        &claude_count.to_string(),
        &gemini_count.to_string(),
    );
    s.assert_eq(
        &format!("claude count ({claude_count}) == codex count ({codex_count})"),
        &claude_count.to_string(),
        &codex_count.to_string(),
    );

    check_synced_from(&mut s, &provider_dirs);
    check_body_matches_source(&mut s, &claude_dst, &agents_dir);
    check_gemini_formatting(&mut s, &gemini_dst);
    check_model_resolved(&mut s, &provider_dirs);

    s
}

fn extract_deployed_body(content: &str) -> &str {
    let body = parse::fm_body(content);
    // Legacy format: strip "# synced-from:" line from body
    let body = body
        .strip_prefix("# synced-from:")
        .map_or(body, |rest| rest.find('\n').map_or("", |i| &rest[i + 1..]));
    body.strip_prefix('\n').unwrap_or(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn structure_missing_files() {
        let dir = tempdir().unwrap();
        let suite = validate_structure(dir.path());
        assert!(suite.failed() > 0);
    }

    #[test]
    fn structure_valid_module() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::write(
            root.join("module.yaml"),
            "name: test\nversion: 0.1.0\ndescription: A test module\n",
        )
        .unwrap();
        fs::create_dir_all(root.join(".claude-plugin")).unwrap();
        fs::write(
            root.join(".claude-plugin/plugin.json"),
            r#"{"name":"test"}"#,
        )
        .unwrap();
        fs::create_dir_all(root.join("lib")).unwrap();
        fs::write(root.join("lib/Makefile"), "build:\n").unwrap();

        let suite = validate_structure(root);
        assert_eq!(suite.failed(), 0);
        assert_eq!(suite.passed(), 7);
    }

    #[test]
    fn roster_extraction() {
        let yaml = "agents:\n  council:\n    - Dev\n    - QA\n  standalone:\n    - Ops\n";
        let names = roster_names(yaml);
        assert_eq!(names, vec!["Dev", "QA", "Ops"]);
    }

    #[test]
    fn council_extraction() {
        let yaml =
            "councils:\n  dev:\n    roles:\n      - Dev\n      - QA\n  ops:\n    roles:\n      - Ops\n";
        let names = council_names(yaml);
        assert_eq!(names, vec!["dev", "ops"]);
        assert_eq!(council_roles(yaml, "dev"), vec!["Dev", "QA"]);
        assert_eq!(council_roles(yaml, "ops"), vec!["Ops"]);
    }

    #[test]
    fn config_block_detection() {
        let yaml = "Developer:\n  model: sonnet\n  tools:\n    - Read\n";
        assert!(has_config_block(yaml, "Developer"));
        assert!(!has_config_block(yaml, "Missing"));
    }

    #[test]
    fn deployed_body_extraction() {
        let content = "---\nname: Test\n---\n# synced-from: Test.md\n\nBody here.\n";
        assert_eq!(extract_deployed_body(content), "Body here.\n");
    }

    #[test]
    fn deployed_body_no_synced_from() {
        let content = "---\nname: Test\n---\nPlain body.\n";
        assert_eq!(extract_deployed_body(content), "Plain body.\n");
    }

    #[test]
    fn skill_dirs_empty() {
        let dir = tempdir().unwrap();
        let names = read_skill_dirs(dir.path());
        assert!(names.is_empty());
    }

    #[test]
    fn yaml_value_basic() {
        let content = "name: TestSkill\ndescription: A test\nargument-hint: test\n";
        assert_eq!(yaml_value(content, "name"), "TestSkill");
        assert_eq!(yaml_value(content, "description"), "A test");
        assert_eq!(yaml_value(content, "argument-hint"), "test");
        assert_eq!(yaml_value(content, "missing"), "");
    }
}
