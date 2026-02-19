use super::*;
use std::fs;
use tempfile::TempDir;

fn write(path: &std::path::Path, content: &str) {
    fs::write(path, content).unwrap();
}

#[test]
fn render_codex_agent_config_writes_toml_and_config_block() {
    let tmp = TempDir::new().unwrap();
    let module_root = tmp.path().join("module");
    let codex_root = tmp.path().join("codex");
    fs::create_dir_all(module_root.join("agents")).unwrap();
    fs::create_dir_all(codex_root.join("agents")).unwrap();

    write(
        &module_root.join("defaults.yaml"),
        r#"
agents:
  DataAnalyst:
    model: fast
providers:
  codex:
    fast: gpt-5.1-codex-mini
    strong: gpt-5.3-codex
"#,
    );
    write(
        &module_root.join("agents/DataAnalyst.md"),
        r#"---
name: DataAnalyst
description: "Data analyst -- analysis."
version: 0.1.0
---
Prompt body
"#,
    );
    write(&codex_root.join("config.toml"), "foo = 1\n");
    write(&codex_root.join("agents/legacy.prompt.md"), "old");
    write(&codex_root.join("agents/forge-council-agents.toml"), "old");

    let result = render_codex_agent_config(&module_root, &codex_root).unwrap();
    assert!(result.agents_dir.ends_with("codex/agents"));
    assert!(result.config_file.ends_with("codex/config.toml"));

    let agent_toml = fs::read_to_string(codex_root.join("agents/DataAnalyst.toml")).unwrap();
    assert!(agent_toml.contains("model = \"gpt-5.1-codex-mini\""));
    assert!(agent_toml.contains("model_reasoning_effort = \"low\""));
    assert!(agent_toml.contains("prompt_file = \"DataAnalyst.md\""));

    assert!(!codex_root.join("agents/legacy.prompt.md").exists());
    assert!(!codex_root.join("agents/forge-council-agents.toml").exists());

    let config = fs::read_to_string(codex_root.join("config.toml")).unwrap();
    assert!(config.contains("foo = 1"));
    assert!(config.contains("# BEGIN forge-council agents"));
    assert!(config.contains("[agents.DataAnalyst]"));
    assert!(config.contains("config_file = \"agents/DataAnalyst.toml\""));
    assert!(config.contains("# END forge-council agents"));
}

#[test]
fn render_codex_agent_config_replaces_existing_managed_block() {
    let tmp = TempDir::new().unwrap();
    let module_root = tmp.path().join("module");
    let codex_root = tmp.path().join("codex");
    fs::create_dir_all(module_root.join("agents")).unwrap();
    fs::create_dir_all(codex_root.join("agents")).unwrap();

    write(
        &module_root.join("defaults.yaml"),
        r#"
agents:
  SoftwareDeveloper:
    model: strong
providers:
  codex:
    fast: gpt-5.1-codex-mini
    strong: gpt-5.3-codex
"#,
    );
    write(
        &module_root.join("agents/SoftwareDeveloper.md"),
        r#"---
name: SoftwareDeveloper
description: "Dev -- writes code."
version: 0.1.0
---
Prompt body
"#,
    );
    write(
        &codex_root.join("config.toml"),
        r#"foo = 1
# BEGIN forge-council agents
[agents.Obsolete]
description = "obsolete"
# END forge-council agents
bar = 2
"#,
    );

    render_codex_agent_config(&module_root, &codex_root).unwrap();

    let config = fs::read_to_string(codex_root.join("config.toml")).unwrap();
    assert!(!config.contains("[agents.Obsolete]"));
    assert!(config.contains("foo = 1"));
    assert!(config.contains("bar = 2"));
    assert!(config.contains("[agents.SoftwareDeveloper]"));
}
