use super::*;
use crate::sidecar::SidecarConfig;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn write_yaml(dir: &Path, filename: &str, content: &str) {
    fs::write(dir.join(filename), content).unwrap();
}

// ─── Provider Fixture ───

#[test]
fn fixture_provider_transforms_agent_for_each_platform() {
    assert_eq!(
        Provider::Claude.format_name("SecurityArchitect"),
        "SecurityArchitect"
    );
    assert_eq!(
        Provider::Codex.format_name("SecurityArchitect"),
        "SecurityArchitect"
    );
    assert_eq!(
        Provider::Gemini.format_name("SecurityArchitect"),
        "security-architect"
    );

    assert_eq!(Provider::Claude.map_tools("Read, Bash"), "Read, Bash");
    assert_eq!(
        Provider::Gemini.map_tools("Read, Bash"),
        "read_file, run_shell_command"
    );
}

// ─── Provider: format_name ───

#[test]
fn format_name_claude_identity() {
    assert_eq!(Provider::Claude.format_name("DevOps"), "DevOps");
}

#[test]
fn format_name_codex_identity() {
    assert_eq!(Provider::Codex.format_name("DevOps"), "DevOps");
}

#[test]
fn format_name_opencode_kebab() {
    assert_eq!(
        Provider::OpenCode.format_name("DocumentationWriter"),
        "documentation-writer"
    );
}

#[test]
fn format_name_gemini_pascal_case() {
    assert_eq!(
        Provider::Gemini.format_name("DocumentationWriter"),
        "documentation-writer"
    );
}

#[test]
fn format_name_gemini_single_word() {
    assert_eq!(Provider::Gemini.format_name("Dev"), "dev");
}

#[test]
fn format_name_gemini_already_lowercase() {
    assert_eq!(Provider::Gemini.format_name("test"), "test");
}

#[test]
fn format_name_gemini_with_numbers() {
    assert_eq!(
        Provider::Gemini.format_name("Model3Config"),
        "model3-config"
    );
}

#[test]
fn format_name_gemini_with_spaces() {
    assert_eq!(Provider::Gemini.format_name("My Agent"), "my-agent");
}

#[test]
fn format_name_gemini_with_underscores() {
    assert_eq!(Provider::Gemini.format_name("my_agent"), "my-agent");
}

#[test]
fn format_name_gemini_empty() {
    assert_eq!(Provider::Gemini.format_name(""), "");
}

#[test]
fn format_name_gemini_consecutive_uppercase() {
    assert_eq!(Provider::Gemini.format_name("HTTPClient"), "httpclient");
}

// ─── Provider: from_str ───

#[test]
fn from_str_claude() {
    assert_eq!(Provider::from_str("claude"), Some(Provider::Claude));
}

#[test]
fn from_str_gemini_case_insensitive() {
    assert_eq!(Provider::from_str("GEMINI"), Some(Provider::Gemini));
}

#[test]
fn from_str_codex() {
    assert_eq!(Provider::from_str("codex"), Some(Provider::Codex));
}

#[test]
fn from_str_opencode() {
    assert_eq!(Provider::from_str("opencode"), Some(Provider::OpenCode));
}

#[test]
fn from_str_invalid() {
    assert_eq!(Provider::from_str("openai"), None);
}

// ─── Provider: from_path ───

#[test]
fn from_path_gemini() {
    assert_eq!(
        Provider::from_path(Path::new("/home/.gemini/agents")),
        Provider::Gemini
    );
}

#[test]
fn from_path_codex() {
    assert_eq!(
        Provider::from_path(Path::new("/home/.codex/agents")),
        Provider::Codex
    );
}

#[test]
fn from_path_opencode() {
    assert_eq!(
        Provider::from_path(Path::new("/home/.opencode/agents")),
        Provider::OpenCode
    );
}

#[test]
fn from_path_claude_default() {
    assert_eq!(
        Provider::from_path(Path::new("/home/.claude/agents")),
        Provider::Claude
    );
}

// ─── Provider: map_tool ───

#[test]
fn map_tool_gemini_read() {
    assert_eq!(Provider::Gemini.map_tool("Read"), "read_file");
}

#[test]
fn map_tool_gemini_write() {
    assert_eq!(Provider::Gemini.map_tool("Write"), "write_file");
}

#[test]
fn map_tool_gemini_edit() {
    assert_eq!(Provider::Gemini.map_tool("Edit"), "replace");
}

#[test]
fn map_tool_gemini_grep() {
    assert_eq!(Provider::Gemini.map_tool("Grep"), "grep_search");
}

#[test]
fn map_tool_gemini_bash() {
    assert_eq!(Provider::Gemini.map_tool("Bash"), "run_shell_command");
}

#[test]
fn map_tool_gemini_websearch() {
    assert_eq!(Provider::Gemini.map_tool("WebSearch"), "google_web_search");
}

#[test]
fn map_tool_gemini_webfetch() {
    assert_eq!(Provider::Gemini.map_tool("WebFetch"), "web_fetch");
}

#[test]
fn map_tool_gemini_unknown() {
    assert_eq!(Provider::Gemini.map_tool("CustomTool"), "customtool");
}

#[test]
fn map_tool_claude_identity() {
    assert_eq!(Provider::Claude.map_tool("Read"), "Read");
}

#[test]
fn map_tool_opencode_identity() {
    assert_eq!(Provider::OpenCode.map_tool("Read"), "Read");
    assert_eq!(Provider::OpenCode.map_tool("Bash"), "Bash");
}

// ─── Provider: map_tools ───

#[test]
fn map_tools_gemini_multiple() {
    assert_eq!(
        Provider::Gemini.map_tools("Read, Write, Bash"),
        "read_file, write_file, run_shell_command"
    );
}

#[test]
fn map_tools_claude_identity() {
    assert_eq!(
        Provider::Claude.map_tools("Read, Write, Bash"),
        "Read, Write, Bash"
    );
}

// ─── Provider: agent_extension ───

#[test]
fn agent_extension_codex_toml() {
    assert_eq!(Provider::Codex.agent_extension(), "toml");
}

#[test]
fn agent_extension_claude_md() {
    assert_eq!(Provider::Claude.agent_extension(), "md");
}

#[test]
fn agent_extension_gemini_md() {
    assert_eq!(Provider::Gemini.agent_extension(), "md");
}

#[test]
fn agent_extension_opencode_md() {
    assert_eq!(Provider::OpenCode.agent_extension(), "md");
}

// ─── Provider: as_str ───

#[test]
fn as_str_roundtrip() {
    assert_eq!(Provider::Claude.as_str(), "claude");
    assert_eq!(Provider::Gemini.as_str(), "gemini");
    assert_eq!(Provider::Codex.as_str(), "codex");
    assert_eq!(Provider::OpenCode.as_str(), "opencode");
}

// ─── Deploy Fixture ───

#[test]
fn fixture_full_deploy_pipeline() {
    let agent_content = "\
---
claude.name: SecurityArchitect
claude.model: sonnet
claude.description: System architect
claude.tools:
  - Read
  - Bash
---
You are a security architect.
";
    let config = SidecarConfig::default();
    let dir = TempDir::new().unwrap();

    let claude_dir = dir.path().join("claude");
    let result = deploy_agent(
        agent_content,
        "SecurityArchitect.md",
        &claude_dir,
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    let deployed = fs::read_to_string(claude_dir.join("SecurityArchitect.md")).unwrap();
    assert!(deployed.contains("name: SecurityArchitect"));
    assert!(deployed.contains("tools: Read, Bash"));
    assert!(deployed.contains("source: SecurityArchitect.md"));

    let gemini_dir = dir.path().join("gemini");
    let result = deploy_agent(
        agent_content,
        "SecurityArchitect.md",
        &gemini_dir,
        Provider::Gemini,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    let deployed = fs::read_to_string(gemini_dir.join("SecurityArchitect.md")).unwrap();
    assert!(deployed.contains("name: security-architect"));
    assert!(deployed.contains("- read_file"));
    assert!(deployed.contains("- run_shell_command"));
    assert!(deployed.contains("kind: local"));
}

// ─── format_agent_output ───

fn make_meta() -> AgentMeta {
    AgentMeta {
        name: "SecurityArchitect".into(),
        display_name: "SecurityArchitect".into(),
        model: "sonnet".into(),
        description: "System architect".into(),
        tools: Some("Read, Bash".into()),
        skills: Vec::new(),
        source_file: "SecurityArchitect.md".into(),
        source: "SecurityArchitect.md".into(),
        reasoning_effort: None,
    }
}

#[test]
fn format_claude_with_model_and_tools() {
    let meta = make_meta();
    let output = format_agent_output(&meta, "Body text.\n", Provider::Claude, true);
    assert!(output.primary.contains("name: SecurityArchitect\n"));
    assert!(output.primary.contains("model: sonnet\n"));
    assert!(output.primary.contains("tools: Read, Bash\n"));
    assert!(output.primary.contains("source: SecurityArchitect.md"));
    assert!(!output.primary.contains("# synced-from:"));
    assert!(output.primary.contains("Body text.\n"));
    assert!(output.prompt_file.is_none());
}

#[test]
fn format_claude_without_model() {
    let meta = make_meta();
    let output = format_agent_output(&meta, "Body.\n", Provider::Claude, false);
    assert!(!output.primary.contains("model:"));
    assert!(output.primary.contains("name: SecurityArchitect"));
}

#[test]
fn format_claude_without_tools() {
    let mut meta = make_meta();
    meta.tools = None;
    let output = format_agent_output(&meta, "Body.\n", Provider::Claude, true);
    assert!(!output.primary.contains("tools:"));
}

#[test]
fn format_gemini_with_mapped_tools() {
    let meta = AgentMeta {
        name: "SecurityArchitect".into(),
        display_name: "security-architect".into(),
        model: "gemini-2.0-flash".into(),
        description: "System architect".into(),
        tools: Some("Read, Bash".into()),
        skills: Vec::new(),
        source_file: "SecurityArchitect.md".into(),
        source: "SecurityArchitect.md".into(),
        reasoning_effort: None,
    };
    let output = format_agent_output(&meta, "Body.\n", Provider::Gemini, true);
    assert!(output.primary.contains("name: security-architect\n"));
    assert!(output.primary.contains("kind: local\n"));
    assert!(output.primary.contains("model: gemini-2.0-flash\n"));
    assert!(output.primary.contains("  - read_file\n"));
    assert!(output.primary.contains("  - run_shell_command\n"));
    assert!(output.prompt_file.is_none());
}

#[test]
fn format_gemini_without_model() {
    let meta = AgentMeta {
        name: "Dev".into(),
        display_name: "dev".into(),
        model: "gemini-2.0-flash".into(),
        description: "Developer".into(),
        tools: Some("Read".into()),
        skills: Vec::new(),
        source_file: "Dev.md".into(),
        source: "Dev.md".into(),
        reasoning_effort: None,
    };
    let output = format_agent_output(&meta, "Body.\n", Provider::Gemini, false);
    assert!(!output.primary.contains("model:"));
    assert!(output.primary.contains("kind: local"));
}

#[test]
fn format_codex_toml_output() {
    let mut meta = make_meta();
    meta.reasoning_effort = Some("low".into());
    let output = format_agent_output(&meta, "Body.\n", Provider::Codex, true);
    assert!(output.primary.contains("# source: SecurityArchitect.md"));
    assert!(output
        .primary
        .contains("description = \"System architect\""));
    assert!(output.primary.contains("model = \"sonnet\""));
    assert!(output.primary.contains("model_reasoning_effort = \"low\""));
    assert!(output
        .primary
        .contains("developer_instructions = \"\"\"\nBody.\n\n\"\"\""));
    assert!(!output.primary.contains("---"));
    assert!(output.prompt_file.is_none());
}

#[test]
fn format_codex_no_reasoning_effort() {
    let meta = make_meta();
    let output = format_agent_output(&meta, "Body.\n", Provider::Codex, true);
    assert!(!output.primary.contains("model_reasoning_effort"));
    assert!(output
        .primary
        .contains("description = \"System architect\""));
    assert!(output.primary.contains("developer_instructions = \"\"\""));
    assert!(output.prompt_file.is_none());
}

#[test]
fn format_codex_without_model() {
    let meta = make_meta();
    let output = format_agent_output(&meta, "Body.\n", Provider::Codex, false);
    assert!(!output.primary.contains("model ="));
    assert!(output
        .primary
        .contains("description = \"System architect\""));
}

#[test]
fn format_source_always_present() {
    let meta = make_meta();
    let claude = format_agent_output(&meta, "B.\n", Provider::Claude, true);
    let gemini = format_agent_output(
        &AgentMeta {
            display_name: "security-architect".into(),
            ..make_meta()
        },
        "B.\n",
        Provider::Gemini,
        true,
    );
    let codex = format_agent_output(&meta, "B.\n", Provider::Codex, true);
    assert!(claude.primary.contains("source: SecurityArchitect.md"));
    assert!(gemini.primary.contains("source: SecurityArchitect.md"));
    assert!(codex.primary.contains("# source: SecurityArchitect.md"));
    assert!(!claude.primary.contains("# synced-from:"));
    assert!(!gemini.primary.contains("# synced-from:"));
}

#[test]
fn format_body_preserved() {
    let meta = make_meta();
    let body = "## Role\n\nYou review architecture.\n\n## Constraints\n\nBe thorough.\n";
    let output = format_agent_output(&meta, body, Provider::Claude, true);
    assert!(output.primary.contains(body));
}

#[test]
fn format_codex_body_in_developer_instructions() {
    let meta = make_meta();
    let body = "## Role\n\nYou review architecture.\n\n## Constraints\n\nBe thorough.\n";
    let output = format_agent_output(&meta, body, Provider::Codex, true);
    assert!(output.primary.contains("developer_instructions = \"\"\""));
    assert!(output.primary.contains("## Role"));
    assert!(output.primary.contains("## Constraints"));
    assert!(output.prompt_file.is_none());
}

// ─── skills rendering ───

#[test]
fn format_claude_with_skills() {
    let mut meta = make_meta();
    meta.skills = vec!["Git".into(), "SecretScan".into()];
    let output = format_agent_output(&meta, "Body.\n", Provider::Claude, true);
    assert!(output.primary.contains("skills:\n  - Git\n  - SecretScan\n"));
}

#[test]
fn format_claude_without_skills() {
    let meta = make_meta();
    let output = format_agent_output(&meta, "Body.\n", Provider::Claude, true);
    assert!(!output.primary.contains("skills:"));
}

#[test]
fn format_gemini_with_skills() {
    let mut meta = make_meta();
    meta.display_name = "security-architect".into();
    meta.skills = vec!["Git".into()];
    let output = format_agent_output(&meta, "Body.\n", Provider::Gemini, true);
    assert!(output.primary.contains("skills:\n  - Git\n"));
}

#[test]
fn format_codex_ignores_skills() {
    let mut meta = make_meta();
    meta.skills = vec!["Git".into()];
    let output = format_agent_output(&meta, "Body.\n", Provider::Codex, true);
    assert!(!output.primary.contains("skills"));
}

#[test]
fn format_codex_matches_data_analyst_fixture() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/codex");
    let expected = fs::read_to_string(fixture_dir.join("DataAnalyst.toml")).unwrap();
    let body = fs::read_to_string(fixture_dir.join("DataAnalyst.prompt.md")).unwrap();
    let meta = AgentMeta {
        name: "DataAnalyst".into(),
        display_name: "DataAnalyst".into(),
        model: "gpt-5.3".into(),
        description: "Data analyst — success metrics, KPIs, measurement strategies, business impact analysis, data-driven evaluation. USE WHEN metrics design, success criteria, impact analysis, data strategy.".into(),
        tools: None,
        skills: Vec::new(),
        source_file: "DataAnalyst.md".into(),
        source: "forge-council/agents/DataAnalyst.md".into(),
        reasoning_effort: Some("low".into()),
    };

    let output = format_agent_output(&meta, &body, Provider::Codex, true);
    assert_eq!(output.primary, expected);
    assert!(output.prompt_file.is_none());
}

#[test]
fn extract_skills_from_config() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "agents:\n  Developer:\n    model: sonnet\n    tools: Read\n    skills:\n      - Git\n      - RustDevelopment\n",
    );
    let config = SidecarConfig::load(dir.path());
    let content = "---\nname: Developer\ndescription: Dev\nversion: 0.3.0\n---\nBody.\n";
    let meta =
        extract_agent_meta(content, "Developer.md", Provider::Claude, &config, "").unwrap();
    assert_eq!(meta.skills, vec!["Git", "RustDevelopment"]);
}

#[test]
fn extract_skills_from_frontmatter_fallback() {
    let config = SidecarConfig::default();
    let content = "---\nclaude.name: Developer\nclaude.skills:\n  - Git\n  - DefensiveProgramming\n---\nBody.\n";
    let meta =
        extract_agent_meta(content, "Developer.md", Provider::Claude, &config, "").unwrap();
    assert_eq!(meta.skills, vec!["Git", "DefensiveProgramming"]);
}

#[test]
fn extract_no_skills_returns_empty() {
    let config = SidecarConfig::default();
    let content = "---\nname: Developer\ndescription: Dev\n---\nBody.\n";
    let meta =
        extract_agent_meta(content, "Developer.md", Provider::Claude, &config, "").unwrap();
    assert!(meta.skills.is_empty());
}

// ─── extract_agent_meta ───

#[test]
fn extract_basic_meta() {
    let content = "\
---
claude.name: Developer
claude.model: sonnet
claude.description: Senior developer
claude.tools:
  - Read
  - Write
---
Body.
";
    let config = SidecarConfig::default();
    let meta = extract_agent_meta(content, "Developer.md", Provider::Claude, &config, "").unwrap();
    assert_eq!(meta.name, "Developer");
    assert_eq!(meta.display_name, "Developer");
    assert_eq!(meta.model, "sonnet");
    assert_eq!(meta.description, "Senior developer");
    assert_eq!(meta.tools, Some("Read, Write".into()));
}

#[test]
fn extract_template_returns_none() {
    let content = "---\nclaude.name: Foo\n---\nBody.\n";
    let config = SidecarConfig::default();
    assert!(
        extract_agent_meta(content, "_TemplateFoo.md", Provider::Claude, &config, "").is_none()
    );
}

#[test]
fn extract_missing_name_returns_none() {
    let content = "---\nclaude.model: sonnet\n---\nBody.\n";
    let config = SidecarConfig::default();
    assert!(extract_agent_meta(content, "Foo.md", Provider::Claude, &config, "").is_none());
}

#[test]
fn extract_defaults_model_to_sonnet() {
    let content = "---\nclaude.name: Tester\n---\nBody.\n";
    let config = SidecarConfig::default();
    let meta = extract_agent_meta(content, "Tester.md", Provider::Claude, &config, "").unwrap();
    assert_eq!(meta.model, "sonnet");
}

#[test]
fn extract_gemini_formats_display_name() {
    let content = "---\nclaude.name: SecurityArchitect\n---\nBody.\n";
    let config = SidecarConfig::default();
    let meta = extract_agent_meta(
        content,
        "SecurityArchitect.md",
        Provider::Gemini,
        &config,
        "",
    )
    .unwrap();
    assert_eq!(meta.name, "SecurityArchitect");
    assert_eq!(meta.display_name, "security-architect");
}

#[test]
fn extract_plain_name_fallback() {
    let content = "\
---
name: TheOpponent
description: \"Devil's advocate -- stress-tests ideas. USE WHEN critical analysis.\"
version: 0.3.0
---
Body.
";
    let config = SidecarConfig::default();
    let meta =
        extract_agent_meta(content, "TheOpponent.md", Provider::Claude, &config, "").unwrap();
    assert_eq!(meta.name, "TheOpponent");
    assert_eq!(
        meta.description,
        "Devil's advocate -- stress-tests ideas. USE WHEN critical analysis."
    );
    assert_eq!(meta.model, "sonnet");
    assert!(meta.tools.is_none());
}

#[test]
fn extract_plain_name_with_sidecar_override() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        concat!(
            "agents:\n  TheOpponent:\n    model: strong\n    tools: Read, Grep, Glob, WebSearch\n",
            "providers:\n  claude:\n    fast: claude-sonnet-4-6\n    strong: claude-opus-4-6\n",
        ),
    );
    let content = "\
---
name: TheOpponent
description: \"Devil's advocate. USE WHEN critical analysis.\"
version: 0.3.0
---
Body.
";
    let config = SidecarConfig::load(dir.path());
    let meta =
        extract_agent_meta(content, "TheOpponent.md", Provider::Claude, &config, "").unwrap();
    assert_eq!(meta.name, "TheOpponent");
    assert_eq!(meta.model, "claude-opus-4-6");
    assert_eq!(meta.tools, Some("Read, Grep, Glob, WebSearch".into()));
}

// ─── deploy_agent ───

fn agent_fixture() -> String {
    "\
---
claude.name: Developer
claude.model: sonnet
claude.description: Senior developer
claude.tools:
  - Read
  - Write
---
You are a developer.
"
    .to_string()
}

#[test]
fn deploy_basic() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let result = deploy_agent(
        &agent_fixture(),
        "Developer.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    assert!(dir.path().join("Developer.md").exists());
}

#[test]
fn deploy_template_skip() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let result = deploy_agent(
        &agent_fixture(),
        "_TemplateAgent.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::SkippedTemplate)));
}

#[test]
fn deploy_user_protection() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    fs::write(
        dir.path().join("Developer.md"),
        "User-created agent content.\n",
    )
    .unwrap();
    let result = deploy_agent(
        &agent_fixture(),
        "Developer.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::SkippedUserOwned)));
}

#[test]
fn deploy_synced_overwrite() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    fs::write(
        dir.path().join("Developer.md"),
        "# synced-from: Developer.md\nOld content.\n",
    )
    .unwrap();
    let result = deploy_agent(
        &agent_fixture(),
        "Developer.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    let content = fs::read_to_string(dir.path().join("Developer.md")).unwrap();
    assert!(content.contains("You are a developer."));
}

#[test]
fn deploy_no_name() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let content = "---\nclaude.model: sonnet\n---\nBody.\n";
    let result = deploy_agent(
        content,
        "Unnamed.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::SkippedNoName)));
}

#[test]
fn deploy_invalid_name() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let content = "---\nclaude.name: ../evil\n---\nBody.\n";
    let result = deploy_agent(
        content,
        "Evil.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(result.is_err());
}

#[test]
fn deploy_dry_run() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let result = deploy_agent(
        &agent_fixture(),
        "Developer.md",
        dir.path(),
        Provider::Claude,
        &config,
        true,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    assert!(!dir.path().join("Developer.md").exists());
}

#[test]
fn deploy_symlink_rejected() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let target = dir.path().join("target.md");
    fs::write(&target, "target").unwrap();
    std::os::unix::fs::symlink(&target, dir.path().join("Developer.md")).unwrap();
    let result = deploy_agent(
        &agent_fixture(),
        "Developer.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(result.is_err());
}

// ─── deploy_agents_from_dir ───

#[test]
fn deploy_from_dir_multiple() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    fs::write(
        src.path().join("Developer.md"),
        "---\nclaude.name: Developer\n---\nDev body.\n",
    )
    .unwrap();
    fs::write(
        src.path().join("Tester.md"),
        "---\nclaude.name: Tester\n---\nTest body.\n",
    )
    .unwrap();
    let config = SidecarConfig::default();
    let results =
        deploy_agents_from_dir(src.path(), dst.path(), Provider::Claude, &config, false, "")
            .unwrap();
    assert_eq!(results.len(), 2);
    assert!(dst.path().join("Developer.md").exists());
    assert!(dst.path().join("Tester.md").exists());
}

#[test]
fn deploy_from_dir_missing_src() {
    let dst = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let results = deploy_agents_from_dir(
        Path::new("/nonexistent"),
        dst.path(),
        Provider::Claude,
        &config,
        false,
        "",
    )
    .unwrap();
    assert!(results.is_empty());
}

// ─── clean_agents ───

#[test]
fn clean_removes_synced() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    fs::write(
        src.path().join("Developer.md"),
        "---\nclaude.name: Developer\n---\nBody.\n",
    )
    .unwrap();
    fs::write(
        dst.path().join("Developer.md"),
        "# synced-from: Developer.md\nDeployed content.\n",
    )
    .unwrap();
    let removed = clean_agents(src.path(), dst.path(), Provider::Claude, false).unwrap();
    assert_eq!(removed, vec!["Developer"]);
    assert!(!dst.path().join("Developer.md").exists());
}

#[test]
fn clean_protects_user_created() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    fs::write(
        src.path().join("Developer.md"),
        "---\nclaude.name: Developer\n---\nBody.\n",
    )
    .unwrap();
    fs::write(dst.path().join("Developer.md"), "User-created agent.\n").unwrap();
    let removed = clean_agents(src.path(), dst.path(), Provider::Claude, false).unwrap();
    assert!(removed.is_empty());
    assert!(dst.path().join("Developer.md").exists());
}

#[test]
fn clean_dry_run() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    fs::write(
        src.path().join("Developer.md"),
        "---\nclaude.name: Developer\n---\nBody.\n",
    )
    .unwrap();
    fs::write(
        dst.path().join("Developer.md"),
        "# synced-from: Developer.md\nContent.\n",
    )
    .unwrap();
    let removed = clean_agents(src.path(), dst.path(), Provider::Claude, true).unwrap();
    assert_eq!(removed, vec!["Developer"]);
    assert!(dst.path().join("Developer.md").exists());
}

#[test]
fn clean_missing_dst() {
    let src = TempDir::new().unwrap();
    let removed = clean_agents(
        src.path(),
        Path::new("/nonexistent"),
        Provider::Claude,
        false,
    )
    .unwrap();
    assert!(removed.is_empty());
}

// ─── new format (name + config-driven model/tools) ───

fn config_with_agents(yaml: &str) -> SidecarConfig {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("defaults.yaml"), yaml).unwrap();
    SidecarConfig::load(dir.path())
}

#[test]
fn extract_new_format_from_config() {
    let config = config_with_agents(
        "agents:\n  Developer:\n    model: sonnet\n    tools: Read, Write, Bash\n",
    );
    let content = "\
---
name: Developer
description: \"Senior developer — implementation quality. USE WHEN code review.\"
version: 0.3.0
---
You are a developer.
";
    let meta = extract_agent_meta(content, "Developer.md", Provider::Claude, &config, "").unwrap();
    assert_eq!(meta.name, "Developer");
    assert_eq!(meta.model, "sonnet");
    assert_eq!(
        meta.description,
        "Senior developer — implementation quality. USE WHEN code review."
    );
    assert_eq!(meta.tools, Some("Read, Write, Bash".into()));
}

#[test]
fn extract_new_format_no_config_defaults() {
    let config = SidecarConfig::default();
    let content = "\
---
name: Tester
description: QA specialist
version: 0.3.0
---
Body.
";
    let meta = extract_agent_meta(content, "Tester.md", Provider::Claude, &config, "").unwrap();
    assert_eq!(meta.name, "Tester");
    assert_eq!(meta.model, "sonnet");
    assert_eq!(meta.description, "QA specialist");
    assert_eq!(meta.tools, None);
}

#[test]
fn extract_new_format_gemini_model_resolution() {
    let config = config_with_agents(concat!(
        "agents:\n  Opponent:\n    model: strong\n    tools: Read, Grep, Glob\n",
        "providers:\n  gemini:\n    fast: gemini-2.0-flash\n    strong: gemini-2.5-pro\n",
    ));
    let content =
        "---\nname: Opponent\ndescription: Devil's advocate\nversion: 0.3.0\n---\nBody.\n";
    let meta = extract_agent_meta(content, "Opponent.md", Provider::Gemini, &config, "").unwrap();
    assert_eq!(meta.model, "gemini-2.5-pro");
    assert_eq!(meta.display_name, "opponent");
}

#[test]
fn deploy_new_format_full_pipeline() {
    let cfg_dir = TempDir::new().unwrap();
    fs::write(
        cfg_dir.path().join("defaults.yaml"),
        "agents:\n  Developer:\n    model: sonnet\n    tools: Read, Write\n",
    )
    .unwrap();
    let config = SidecarConfig::load(cfg_dir.path());

    let content = "\
---
name: Developer
description: Senior developer specialist
version: 0.3.0
---
You are a developer.
";
    let dst = TempDir::new().unwrap();
    let result = deploy_agent(
        content,
        "Developer.md",
        dst.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    let deployed = fs::read_to_string(dst.path().join("Developer.md")).unwrap();
    assert!(deployed.contains("name: Developer"));
    assert!(deployed.contains("model: sonnet"));
    assert!(deployed.contains("tools: Read, Write"));
    assert!(deployed.contains("source: Developer.md"));
    assert!(deployed.contains("You are a developer."));
}

#[test]
fn deploy_new_format_from_dir() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    fs::write(
        src.path().join("Developer.md"),
        "---\nname: Developer\ndescription: Dev\nversion: 0.3.0\n---\nDev body.\n",
    )
    .unwrap();
    fs::write(
        src.path().join("Tester.md"),
        "---\nname: Tester\ndescription: QA\nversion: 0.3.0\n---\nTest body.\n",
    )
    .unwrap();

    let cfg_dir = TempDir::new().unwrap();
    fs::write(
        cfg_dir.path().join("defaults.yaml"),
        "agents:\n  Developer:\n    model: sonnet\n    tools: Read, Write\n  Tester:\n    model: sonnet\n    tools: Read, Bash\n",
    )
    .unwrap();
    let config = SidecarConfig::load(cfg_dir.path());

    let results =
        deploy_agents_from_dir(src.path(), dst.path(), Provider::Claude, &config, false, "")
            .unwrap();
    assert_eq!(results.len(), 2);
    assert!(dst.path().join("Developer.md").exists());
    assert!(dst.path().join("Tester.md").exists());
}

#[test]
fn clean_new_format() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    fs::write(
        src.path().join("Developer.md"),
        "---\nname: Developer\ndescription: Dev\nversion: 0.3.0\n---\nBody.\n",
    )
    .unwrap();
    fs::write(
        dst.path().join("Developer.md"),
        "# synced-from: Developer.md\nDeployed content.\n",
    )
    .unwrap();
    let removed = clean_agents(src.path(), dst.path(), Provider::Claude, false).unwrap();
    assert_eq!(removed, vec!["Developer"]);
    assert!(!dst.path().join("Developer.md").exists());
}

// ─── Codex deploy ───

#[test]
fn deploy_codex_writes_inline_toml() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let content = "---\nname: Developer\ndescription: Senior dev\nversion: 0.3.0\n---\nYou are a developer.\n";
    let result = deploy_agent(
        content,
        "Developer.md",
        dir.path(),
        Provider::Codex,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    assert!(dir.path().join("Developer.toml").exists());
    assert!(!dir.path().join("Developer.prompt.md").exists());
    let toml = fs::read_to_string(dir.path().join("Developer.toml")).unwrap();
    assert!(toml.contains("description = \"Senior dev\""));
    assert!(toml.contains("developer_instructions = \"\"\""));
    assert!(toml.contains("You are a developer."));
}

#[test]
fn deploy_codex_overwrite_with_source() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    fs::write(
        dir.path().join("Developer.toml"),
        "# source: Developer.md\ndescription = \"Old\"\n",
    )
    .unwrap();
    let content =
        "---\nname: Developer\ndescription: Updated dev\nversion: 0.3.0\n---\nNew body.\n";
    let result = deploy_agent(
        content,
        "Developer.md",
        dir.path(),
        Provider::Codex,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    let toml = fs::read_to_string(dir.path().join("Developer.toml")).unwrap();
    assert!(toml.contains("description = \"Updated dev\""));
}

#[test]
fn deploy_codex_overwrite_removes_legacy_prompt_file() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    fs::write(
        dir.path().join("Developer.toml"),
        "# source: Developer.md\ndescription = \"Old\"\n",
    )
    .unwrap();
    fs::write(dir.path().join("Developer.prompt.md"), "Legacy prompt.\n").unwrap();
    let content =
        "---\nname: Developer\ndescription: Updated dev\nversion: 0.3.0\n---\nNew body.\n";
    let result = deploy_agent(
        content,
        "Developer.md",
        dir.path(),
        Provider::Codex,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    assert!(!dir.path().join("Developer.prompt.md").exists());
}

#[test]
fn deploy_codex_skips_user_owned_toml() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    fs::write(
        dir.path().join("Developer.toml"),
        "description = \"My custom agent\"\n",
    )
    .unwrap();
    let content = "---\nname: Developer\ndescription: Dev\nversion: 0.3.0\n---\nBody.\n";
    let result = deploy_agent(
        content,
        "Developer.md",
        dir.path(),
        Provider::Codex,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::SkippedUserOwned)));
}

#[test]
fn clean_codex_removes_toml_and_prompt() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    fs::write(
        src.path().join("Developer.md"),
        "---\nname: Developer\n---\nBody.\n",
    )
    .unwrap();
    fs::write(
        dst.path().join("Developer.toml"),
        "# source: Developer.md\ndescription = \"Dev\"\n",
    )
    .unwrap();
    fs::write(dst.path().join("Developer.prompt.md"), "Body.\n").unwrap();
    let removed = clean_agents(src.path(), dst.path(), Provider::Codex, false).unwrap();
    assert_eq!(removed, vec!["Developer"]);
    assert!(!dst.path().join("Developer.toml").exists());
    assert!(!dst.path().join("Developer.prompt.md").exists());
}

// ─── reasoning_effort extraction ───

#[test]
fn extract_reasoning_effort_from_agent_config() {
    let config = config_with_agents(concat!(
        "agents:\n  Developer:\n    model: fast\n    tools: Read\n    reasoning_effort: high\n",
        "providers:\n  codex:\n    fast: gpt-5.1-codex-mini\n    strong: o4-mini\n",
        "    reasoning_effort:\n      fast: low\n      strong: medium\n",
    ));
    let content = "---\nname: Developer\ndescription: Dev\nversion: 0.3.0\n---\nBody.\n";
    let meta = extract_agent_meta(content, "Developer.md", Provider::Codex, &config, "").unwrap();
    assert_eq!(meta.reasoning_effort, Some("high".into()));
}

#[test]
fn extract_reasoning_effort_tier_fallback() {
    let config = config_with_agents(concat!(
        "agents:\n  Developer:\n    model: fast\n    tools: Read\n",
        "providers:\n  codex:\n    fast: gpt-5.1-codex-mini\n    strong: o4-mini\n",
        "    reasoning_effort:\n      fast: low\n      strong: medium\n",
    ));
    let content = "---\nname: Developer\ndescription: Dev\nversion: 0.3.0\n---\nBody.\n";
    let meta = extract_agent_meta(content, "Developer.md", Provider::Codex, &config, "").unwrap();
    assert_eq!(meta.reasoning_effort, Some("low".into()));
    assert_eq!(meta.model, "gpt-5.1-codex-mini");
}

#[test]
fn extract_reasoning_effort_none_without_config() {
    let config = SidecarConfig::default();
    let content = "---\nname: Developer\ndescription: Dev\nversion: 0.3.0\n---\nBody.\n";
    let meta = extract_agent_meta(content, "Developer.md", Provider::Claude, &config, "").unwrap();
    assert_eq!(meta.reasoning_effort, None);
}

// ─── source prefix ───

#[test]
fn extract_source_prefix_produces_full_path() {
    let config = SidecarConfig::default();
    let content = "---\nname: Dev\ndescription: Developer\nversion: 0.3.0\n---\nBody.\n";
    let meta = extract_agent_meta(
        content,
        "Dev.md",
        Provider::Claude,
        &config,
        "forge-council/agents",
    )
    .unwrap();
    assert_eq!(meta.source, "forge-council/agents/Dev.md");
    assert_eq!(meta.source_file, "Dev.md");
}

#[test]
fn deploy_source_in_frontmatter() {
    let dst = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let content = "---\nname: Dev\ndescription: Developer\nversion: 0.3.0\n---\nBody.\n";
    let result = deploy_agent(
        content,
        "Dev.md",
        dst.path(),
        Provider::Claude,
        &config,
        false,
        "forge-council/agents",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    let deployed = fs::read_to_string(dst.path().join("Dev.md")).unwrap();
    assert!(deployed.contains("source: forge-council/agents/Dev.md"));
    assert!(!deployed.contains("# synced-from:"));
}

#[test]
fn deploy_overwrite_new_format_source() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    fs::write(
        dir.path().join("Developer.md"),
        "---\nname: Developer\nsource: Developer.md\n---\nOld.\n",
    )
    .unwrap();
    let result = deploy_agent(
        &agent_fixture(),
        "Developer.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    let content = fs::read_to_string(dir.path().join("Developer.md")).unwrap();
    assert!(content.contains("You are a developer."));
}

// ─── scope_dirs ───

fn default_providers() -> Vec<String> {
    vec![
        "claude".into(),
        "gemini".into(),
        "codex".into(),
        "opencode".into(),
    ]
}

#[test]
fn scope_user() {
    let home = Path::new("/home/user");
    let providers = default_providers();
    let dirs = scope_dirs("user", home, &providers).unwrap();
    assert_eq!(dirs.len(), 4);
    assert_eq!(dirs[0], home.join(".claude/agents"));
    assert_eq!(dirs[1], home.join(".gemini/agents"));
    assert_eq!(dirs[2], home.join(".codex/agents"));
    assert_eq!(dirs[3], home.join(".opencode/agents"));
}

#[test]
fn scope_workspace() {
    let home = Path::new("/home/user");
    let providers = default_providers();
    let dirs = scope_dirs("workspace", home, &providers).unwrap();
    assert_eq!(dirs.len(), 4);
    assert_eq!(dirs[0], PathBuf::from(".claude/agents"));
    assert_eq!(dirs[3], PathBuf::from(".opencode/agents"));
}

#[test]
fn scope_all() {
    let home = Path::new("/home/user");
    let providers = default_providers();
    let dirs = scope_dirs("all", home, &providers).unwrap();
    assert_eq!(dirs.len(), 8);
}

#[test]
fn scope_project() {
    let home = Path::new("/home/user");
    let providers = default_providers();
    let dirs = scope_dirs("project", home, &providers).unwrap();
    assert_eq!(dirs.len(), 4);
    // Project key is CWD with / replaced by -
    let key = std::env::current_dir()
        .unwrap()
        .to_string_lossy()
        .replace('/', "-");
    assert_eq!(dirs[0], home.join(format!(".claude/projects/{key}/agents")));
    assert_eq!(dirs[1], home.join(format!(".gemini/projects/{key}/agents")));
    assert_eq!(dirs[2], home.join(format!(".codex/projects/{key}/agents")));
    assert_eq!(
        dirs[3],
        home.join(format!(".opencode/projects/{key}/agents"))
    );
}

#[test]
fn scope_subset_providers() {
    let home = Path::new("/home/user");
    let providers = vec!["claude".into(), "gemini".into()];
    let dirs = scope_dirs("user", home, &providers).unwrap();
    assert_eq!(dirs.len(), 2);
    assert_eq!(dirs[0], home.join(".claude/agents"));
    assert_eq!(dirs[1], home.join(".gemini/agents"));
}

#[test]
fn scope_invalid() {
    let providers = default_providers();
    assert!(scope_dirs("bogus", Path::new("/tmp"), &providers).is_err());
}

// ─── toml_escape ───

#[test]
fn toml_escape_quotes_and_backslashes() {
    assert_eq!(toml_escape(r#"say "hello""#), r#"say \"hello\""#);
    assert_eq!(toml_escape(r"path\to\file"), r"path\\to\\file");
    assert_eq!(
        toml_escape(r#"mixed "quote" and \back"#),
        r#"mixed \"quote\" and \\back"#
    );
}

#[test]
fn toml_escape_no_special_chars() {
    assert_eq!(toml_escape("plain text"), "plain text");
}

// ─── format_codex_config_block ───

#[test]
fn format_codex_config_block_single_agent() {
    let entries = vec![CodexConfigEntry {
        name: "DataAnalyst".into(),
        description: "Data analyst specialist".into(),
    }];
    let block = format_codex_config_block(&entries, "forge-council/agents");
    assert!(block.contains("# BEGIN forge-council agents"));
    assert!(block.contains("# Generated by install-agents (forge-council/agents)"));
    assert!(block.contains("[agents.DataAnalyst]"));
    assert!(block.contains("description = \"Data analyst specialist\""));
    assert!(block.contains("config_file = \"agents/DataAnalyst.toml\""));
    assert!(block.contains("# END forge-council agents"));
}

#[test]
fn format_codex_config_block_multiple_agents() {
    let entries = vec![
        CodexConfigEntry {
            name: "DataAnalyst".into(),
            description: "Data analyst".into(),
        },
        CodexConfigEntry {
            name: "SecurityArchitect".into(),
            description: "Security architect".into(),
        },
    ];
    let block = format_codex_config_block(&entries, "test");
    let da_pos = block.find("[agents.DataAnalyst]").unwrap();
    let sa_pos = block.find("[agents.SecurityArchitect]").unwrap();
    assert!(da_pos < sa_pos);
    assert!(block.contains("config_file = \"agents/SecurityArchitect.toml\""));
}

#[test]
fn format_codex_config_block_escapes_description() {
    let entries = vec![CodexConfigEntry {
        name: "Test".into(),
        description: r#"Agent with "quotes" and \backslash"#.into(),
    }];
    let block = format_codex_config_block(&entries, "");
    assert!(block.contains(r#"description = "Agent with \"quotes\" and \\backslash""#));
}

// ─── strip_managed_block ───

#[test]
fn strip_managed_block_basic() {
    let content = "\
[features]
multi_agent = true

# BEGIN forge-council agents
[agents.Foo]
description = \"Foo\"
# END forge-council agents
";
    let stripped = strip_managed_block(content, CODEX_BLOCK_BEGIN, CODEX_BLOCK_END);
    assert!(!stripped.contains("agents.Foo"));
    assert!(!stripped.contains("BEGIN forge-council"));
    assert!(stripped.contains("multi_agent = true"));
}

#[test]
fn strip_managed_block_no_block_present() {
    let content = "[features]\nmulti_agent = true\n";
    let stripped = strip_managed_block(content, CODEX_BLOCK_BEGIN, CODEX_BLOCK_END);
    assert!(stripped.contains("multi_agent = true"));
}

// ─── write_codex_config_block ───

#[test]
fn write_codex_config_preserves_existing() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.toml");
    fs::write(&config_path, "[features]\nmulti_agent = true\n").unwrap();

    let entries = vec![CodexConfigEntry {
        name: "Dev".into(),
        description: "Developer".into(),
    }];
    write_codex_config_block(&config_path, &entries, "test", false).unwrap();

    let result = fs::read_to_string(&config_path).unwrap();
    assert!(result.contains("multi_agent = true"));
    assert!(result.contains("[agents.Dev]"));
    assert!(result.contains("# BEGIN forge-council agents"));
}

#[test]
fn write_codex_config_replaces_managed_block() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.toml");
    let initial = "\
[features]
multi_agent = true

# BEGIN forge-council agents
[agents.OldAgent]
description = \"Old\"
config_file = \"agents/OldAgent.toml\"
# END forge-council agents
";
    fs::write(&config_path, initial).unwrap();

    let entries = vec![CodexConfigEntry {
        name: "NewAgent".into(),
        description: "New".into(),
    }];
    write_codex_config_block(&config_path, &entries, "test", false).unwrap();

    let result = fs::read_to_string(&config_path).unwrap();
    assert!(result.contains("[agents.NewAgent]"));
    assert!(!result.contains("OldAgent"));
    assert_eq!(
        result.matches("BEGIN forge-council agents").count(),
        1,
        "should have exactly one managed block"
    );
}

#[test]
fn write_codex_config_creates_new_file() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("sub").join("config.toml");

    let entries = vec![CodexConfigEntry {
        name: "Dev".into(),
        description: "Developer".into(),
    }];
    write_codex_config_block(&config_path, &entries, "test", false).unwrap();

    assert!(config_path.exists());
    let result = fs::read_to_string(&config_path).unwrap();
    assert!(result.contains("[agents.Dev]"));
}

// ─── clean_codex_config_block ───

#[test]
fn clean_codex_config_block_removes_managed() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.toml");
    let content = "\
[features]
multi_agent = true

# BEGIN forge-council agents
[agents.Dev]
description = \"Dev\"
# END forge-council agents
";
    fs::write(&config_path, content).unwrap();

    clean_codex_config_block(&config_path, false).unwrap();

    let result = fs::read_to_string(&config_path).unwrap();
    assert!(!result.contains("agents.Dev"));
    assert!(!result.contains("BEGIN forge-council"));
    assert!(result.contains("multi_agent = true"));
}

#[test]
fn clean_codex_config_block_noop_when_missing() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.toml");
    // File doesn't exist — should be a no-op
    clean_codex_config_block(&config_path, false).unwrap();
    assert!(!config_path.exists());
}

// ─── clean_orphaned_agents ───

#[test]
fn orphan_removes_renamed_agent() {
    let dst = TempDir::new().unwrap();
    crate::manifest::update(dst.path(), "forge-council", &["OldName".to_string()]).unwrap();
    fs::write(
        dst.path().join("OldName.md"),
        "---\nname: OldName\nsource: forge-council/agents/OldName.md\n---\nOld body.\n",
    )
    .unwrap();
    let removed = clean_orphaned_agents(
        dst.path(),
        "forge-council",
        &["NewName".to_string()],
        Provider::Claude,
        false,
    )
    .unwrap();
    assert_eq!(removed, vec!["OldName"]);
    assert!(!dst.path().join("OldName.md").exists());
}

#[test]
fn orphan_keeps_current_agent() {
    let dst = TempDir::new().unwrap();
    crate::manifest::update(dst.path(), "forge-council", &["Developer".to_string()]).unwrap();
    fs::write(
        dst.path().join("Developer.md"),
        "---\nname: Developer\nsource: forge-council/agents/Developer.md\n---\nBody.\n",
    )
    .unwrap();
    let removed = clean_orphaned_agents(
        dst.path(),
        "forge-council",
        &["Developer".to_string()],
        Provider::Claude,
        false,
    )
    .unwrap();
    assert!(removed.is_empty());
    assert!(dst.path().join("Developer.md").exists());
}

#[test]
fn orphan_dry_run_preserves_file() {
    let dst = TempDir::new().unwrap();
    crate::manifest::update(dst.path(), "forge-council", &["Old".to_string()]).unwrap();
    fs::write(dst.path().join("Old.md"), "---\nname: Old\n---\nBody.\n").unwrap();
    let removed =
        clean_orphaned_agents(dst.path(), "forge-council", &[], Provider::Claude, true).unwrap();
    assert_eq!(removed, vec!["Old"]);
    assert!(dst.path().join("Old.md").exists());
}

#[test]
fn orphan_codex_removes_prompt_companion() {
    let dst = TempDir::new().unwrap();
    crate::manifest::update(dst.path(), "forge-council", &["Old".to_string()]).unwrap();
    fs::write(
        dst.path().join("Old.toml"),
        "# source: forge-council/agents/Old.md\ndescription = \"Old\"\n",
    )
    .unwrap();
    fs::write(dst.path().join("Old.prompt.md"), "Old body.\n").unwrap();
    let removed =
        clean_orphaned_agents(dst.path(), "forge-council", &[], Provider::Codex, false).unwrap();
    assert_eq!(removed, vec!["Old"]);
    assert!(!dst.path().join("Old.toml").exists());
    assert!(!dst.path().join("Old.prompt.md").exists());
}

#[test]
fn orphan_empty_module_skips() {
    let dst = TempDir::new().unwrap();
    let removed = clean_orphaned_agents(dst.path(), "", &[], Provider::Claude, false).unwrap();
    assert!(removed.is_empty());
}

#[test]
fn orphan_missing_dst_dir() {
    let removed = clean_orphaned_agents(
        Path::new("/nonexistent"),
        "forge-council",
        &[],
        Provider::Claude,
        false,
    )
    .unwrap();
    assert!(removed.is_empty());
}

// ─── Lifecycle: deploy → rename → orphan clean ───

#[test]
fn orphan_lifecycle_deploy_rename_clean() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let prefix = "forge-council/agents";
    let module = "forge-council";

    // Step 1: Deploy "OldName" agent
    let content = "---\nname: OldName\ndescription: Original\nversion: 0.1.0\n---\nBody.\n";
    fs::write(src.path().join("OldName.md"), content).unwrap();
    let results = deploy_agents_from_dir(
        src.path(),
        dst.path(),
        Provider::Claude,
        &config,
        false,
        prefix,
    )
    .unwrap();
    assert_eq!(results.len(), 1);
    assert!(dst.path().join("OldName.md").exists());

    // Record in manifest
    crate::manifest::update(dst.path(), module, &["OldName".to_string()]).unwrap();

    // Step 2: Rename source to "NewName" (remove OldName, add NewName)
    fs::remove_file(src.path().join("OldName.md")).unwrap();
    let new_content = "---\nname: NewName\ndescription: Renamed\nversion: 0.2.0\n---\nBody.\n";
    fs::write(src.path().join("NewName.md"), new_content).unwrap();

    // Step 3: Deploy again (NewName)
    let results = deploy_agents_from_dir(
        src.path(),
        dst.path(),
        Provider::Claude,
        &config,
        false,
        prefix,
    )
    .unwrap();
    assert_eq!(results.len(), 1);
    assert!(dst.path().join("NewName.md").exists());
    // OldName still exists (deploy doesn't clean)
    assert!(dst.path().join("OldName.md").exists());

    // Step 4: Orphan clean removes OldName
    let installed = vec!["NewName".to_string()];
    let removed =
        clean_orphaned_agents(dst.path(), module, &installed, Provider::Claude, false).unwrap();
    assert_eq!(removed, vec!["OldName"]);
    assert!(!dst.path().join("OldName.md").exists());
    assert!(dst.path().join("NewName.md").exists());

    // Step 5: Update manifest
    crate::manifest::update(dst.path(), module, &installed).unwrap();
    assert_eq!(crate::manifest::read(dst.path(), module), installed);
}
