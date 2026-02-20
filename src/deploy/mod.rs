pub mod provider;

use crate::parse;
use crate::sidecar::{resolve_model, SidecarConfig};
use provider::Provider;
use std::env;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

pub struct AgentMeta {
    pub name: String,
    pub display_name: String,
    pub model: String,
    pub description: String,
    pub tools: Option<String>,
    pub source_file: String,
    pub source: String,
    pub reasoning_effort: Option<String>,
}

pub struct AgentOutput {
    pub primary: String,
    pub prompt_file: Option<(String, String)>,
}

#[derive(Debug, PartialEq)]
pub enum DeployResult {
    Deployed,
    SkippedTemplate,
    SkippedUserOwned,
    SkippedNoName,
}

pub fn format_agent_output(
    meta: &AgentMeta,
    body: &str,
    provider: Provider,
    model_allowed: bool,
) -> AgentOutput {
    let mut out = String::new();

    match provider {
        Provider::Codex => {
            let _ = writeln!(out, "# source: {}", meta.source);
            let _ = writeln!(out, "description = \"{}\"", toml_escape(&meta.description));
            if model_allowed {
                let _ = writeln!(out, "model = \"{}\"", toml_escape(&meta.model));
            }
            if let Some(ref effort) = meta.reasoning_effort {
                let _ = writeln!(out, "model_reasoning_effort = \"{effort}\"");
            }
            let prompt_filename = format!("{}.prompt.md", meta.name);
            let instructions_path = format!("agents/{prompt_filename}");
            let _ = writeln!(
                out,
                "model_instructions_file = \"{}\"",
                toml_escape(&instructions_path)
            );

            let mut prompt_body = body.to_string();
            if !prompt_body.ends_with('\n') {
                prompt_body.push('\n');
            }

            return AgentOutput {
                primary: out,
                prompt_file: Some((prompt_filename, prompt_body)),
            };
        }
        Provider::Gemini => {
            out.push_str("---\n");
            let _ = writeln!(out, "name: {}", meta.display_name);
            let _ = writeln!(out, "description: {}", meta.description);
            out.push_str("kind: local\n");
            if model_allowed {
                let _ = writeln!(out, "model: {}", meta.model);
            }
            if let Some(ref tools) = meta.tools {
                let mapped = provider.map_tools(tools);
                out.push_str("tools:\n");
                for tool in mapped.split(", ") {
                    let _ = writeln!(out, "  - {tool}");
                }
            }
        }
        Provider::Claude => {
            out.push_str("---\n");
            let _ = writeln!(out, "name: {}", meta.display_name);
            let _ = writeln!(out, "description: {}", meta.description);
            if model_allowed {
                let _ = writeln!(out, "model: {}", meta.model);
            }
            if let Some(ref tools) = meta.tools {
                let _ = writeln!(out, "tools: {tools}");
            }
        }
    }

    let _ = writeln!(out, "source: {}", meta.source);
    out.push_str("---\n");
    out.push_str(body);
    if !body.ends_with('\n') {
        out.push('\n');
    }

    AgentOutput {
        primary: out,
        prompt_file: None,
    }
}

pub fn extract_agent_meta(
    content: &str,
    filename: &str,
    provider: Provider,
    config: &SidecarConfig,
    source_prefix: &str,
) -> Option<AgentMeta> {
    if filename.starts_with("_Template") || filename.starts_with("Template") {
        return None;
    }

    let name =
        parse::fm_value(content, "name").or_else(|| parse::fm_value(content, "claude.name"))?;
    if name.is_empty() {
        return None;
    }

    // Config is primary source for model/tools; frontmatter is legacy fallback
    let model_tier = config
        .agent_value(&name, "model")
        .or_else(|| parse::fm_value(content, "claude.model"))
        .unwrap_or_else(|| "sonnet".into());

    let description = parse::fm_value(content, "description")
        .or_else(|| parse::fm_value(content, "claude.description"))
        .or_else(|| config.agent_value(&name, "description"))
        .unwrap_or_else(|| "Specialist agent".into());

    let tools = config
        .agent_value(&name, "tools")
        .or_else(|| parse::fm_list(content, "claude.tools"))
        .or_else(|| parse::fm_value(content, "claude.tools"));

    let global = config.global_tiers();
    let provider_tiers = config.provider_tiers(provider.as_str());
    let model = resolve_model(&model_tier, &global, &provider_tiers);

    let reasoning_effort = config
        .agent_value(&name, "reasoning_effort")
        .or_else(|| config.provider_reasoning_effort(provider.as_str(), &model_tier));

    let display_name = provider.format_name(&name);

    let source = if source_prefix.is_empty() {
        filename.to_string()
    } else {
        format!("{source_prefix}/{filename}")
    };

    Some(AgentMeta {
        name,
        display_name,
        model,
        description,
        tools,
        source_file: filename.to_string(),
        source,
        reasoning_effort,
    })
}

pub fn deploy_agent(
    content: &str,
    filename: &str,
    dst_dir: &Path,
    provider: Provider,
    config: &SidecarConfig,
    dry_run: bool,
    source_prefix: &str,
) -> Result<DeployResult, String> {
    if filename.starts_with("_Template") || filename.starts_with("Template") {
        return Ok(DeployResult::SkippedTemplate);
    }

    let Some(meta) = extract_agent_meta(content, filename, provider, config, source_prefix) else {
        return Ok(DeployResult::SkippedNoName);
    };

    parse::validate_agent_name(&meta.name)?;

    let ext = provider.agent_extension();
    let out_path = dst_dir.join(format!("{}.{ext}", meta.name));

    if out_path.is_symlink() {
        return Err(format!("destination is a symlink: {}", out_path.display()));
    }

    if out_path.exists() {
        let existing = std::fs::read_to_string(&out_path)
            .map_err(|e| format!("failed to read {}: {e}", out_path.display()))?;
        if !parse::is_synced_from(&existing, filename) {
            return Ok(DeployResult::SkippedUserOwned);
        }
    }

    let model_allowed = config.is_model_whitelisted(provider.as_str(), &meta.model);
    let body = parse::fm_body(content);
    let output = format_agent_output(&meta, body, provider, model_allowed);

    if !dry_run {
        std::fs::create_dir_all(dst_dir)
            .map_err(|e| format!("failed to create {}: {e}", dst_dir.display()))?;
        std::fs::write(&out_path, &output.primary)
            .map_err(|e| format!("failed to write {}: {e}", out_path.display()))?;
        if let Some((ref prompt_filename, ref prompt_content)) = output.prompt_file {
            let prompt_path = dst_dir.join(prompt_filename);
            std::fs::write(&prompt_path, prompt_content)
                .map_err(|e| format!("failed to write {}: {e}", prompt_path.display()))?;
        }
    }

    Ok(DeployResult::Deployed)
}

pub fn deploy_agents_from_dir(
    src_dir: &Path,
    dst_dir: &Path,
    provider: Provider,
    config: &SidecarConfig,
    dry_run: bool,
    source_prefix: &str,
) -> Result<Vec<(String, DeployResult)>, String> {
    if !src_dir.is_dir() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(src_dir)
        .map_err(|e| format!("failed to read {}: {e}", src_dir.display()))?;

    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    files.sort_by_key(std::fs::DirEntry::file_name);

    let mut results = Vec::new();
    for entry in files {
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().to_string();
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let result = deploy_agent(
            &content,
            &filename,
            dst_dir,
            provider,
            config,
            dry_run,
            source_prefix,
        )?;
        results.push((filename, result));
    }

    Ok(results)
}

pub fn clean_agents(
    src_dir: &Path,
    dst_dir: &Path,
    provider: Provider,
    dry_run: bool,
) -> Result<Vec<String>, String> {
    if !src_dir.is_dir() || !dst_dir.is_dir() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(src_dir)
        .map_err(|e| format!("failed to read {}: {e}", src_dir.display()))?;

    let ext = provider.agent_extension();
    let mut removed = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "md") {
            let filename = entry.file_name().to_string_lossy().to_string();
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("failed to read {}: {e}", path.display()))?;

            let name = match parse::fm_value(&content, "name")
                .or_else(|| parse::fm_value(&content, "claude.name"))
            {
                Some(n) if !n.is_empty() => n,
                _ => continue,
            };

            let dst_path = dst_dir.join(format!("{name}.{ext}"));
            if dst_path.exists() {
                let existing = std::fs::read_to_string(&dst_path)
                    .map_err(|e| format!("failed to read {}: {e}", dst_path.display()))?;
                if parse::is_synced_from(&existing, &filename) {
                    if !dry_run {
                        std::fs::remove_file(&dst_path)
                            .map_err(|e| format!("failed to remove {}: {e}", dst_path.display()))?;
                    }
                    if provider == Provider::Codex {
                        let prompt_path = dst_dir.join(format!("{name}.prompt.md"));
                        if prompt_path.exists() && !dry_run {
                            let _ = std::fs::remove_file(&prompt_path);
                        }
                    }
                    removed.push(name);
                }
            }
        }
    }

    Ok(removed)
}

pub fn clean_orphaned_agents(
    dst_dir: &Path,
    module_name: &str,
    current_agents: &[String],
    provider: Provider,
    dry_run: bool,
) -> Result<Vec<String>, String> {
    if module_name.is_empty() {
        return Ok(Vec::new());
    }

    let previous = crate::manifest::read(dst_dir, module_name);
    let ext = provider.agent_extension();
    let mut removed = Vec::new();

    for name in &previous {
        if current_agents.contains(name) {
            continue;
        }
        let path = dst_dir.join(format!("{name}.{ext}"));
        if !path.exists() {
            continue;
        }
        if !dry_run {
            std::fs::remove_file(&path)
                .map_err(|e| format!("failed to remove {}: {e}", path.display()))?;
            if provider == Provider::Codex {
                let prompt_path = dst_dir.join(format!("{name}.prompt.md"));
                if prompt_path.exists() {
                    let _ = std::fs::remove_file(&prompt_path);
                }
            }
        }
        removed.push(name.clone());
    }

    Ok(removed)
}

fn project_key() -> Result<String, String> {
    let cwd = env::current_dir().map_err(|e| format!("failed to get cwd: {e}"))?;
    Ok(cwd.to_string_lossy().replace('/', "-"))
}

pub fn scope_dirs(scope: &str, home: &Path) -> Result<Vec<PathBuf>, String> {
    let user_dirs = vec![
        home.join(".claude/agents"),
        home.join(".gemini/agents"),
        home.join(".codex/agents"),
    ];
    let workspace_dirs = vec![
        PathBuf::from(".claude/agents"),
        PathBuf::from(".gemini/agents"),
        PathBuf::from(".codex/agents"),
    ];

    match scope {
        "user" => Ok(user_dirs),
        "workspace" => Ok(workspace_dirs),
        "project" => {
            let key = project_key()?;
            Ok(vec![
                home.join(format!(".claude/projects/{key}/agents")),
                home.join(format!(".gemini/projects/{key}/agents")),
                home.join(format!(".codex/projects/{key}/agents")),
            ])
        }
        "all" => {
            let mut all = user_dirs;
            all.extend(workspace_dirs);
            Ok(all)
        }
        other => Err(format!(
            "invalid scope {other:?}: use user, workspace, project, or all"
        )),
    }
}

// ─── Codex config.toml managed block ───

const CODEX_BLOCK_BEGIN: &str = "# BEGIN forge-council agents";
const CODEX_BLOCK_END: &str = "# END forge-council agents";

pub struct CodexConfigEntry {
    pub name: String,
    pub description: String,
}

pub fn format_codex_config_block(entries: &[CodexConfigEntry], source_prefix: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{CODEX_BLOCK_BEGIN}");
    let _ = writeln!(out, "# Generated by install-agents ({source_prefix})");
    for entry in entries {
        let _ = writeln!(out);
        let _ = writeln!(out, "[agents.{}]", entry.name);
        let _ = writeln!(out, "description = \"{}\"", toml_escape(&entry.description));
        let _ = writeln!(
            out,
            "config_file = \"agents/{}.toml\"",
            toml_escape(&entry.name)
        );
    }
    let _ = writeln!(out, "{CODEX_BLOCK_END}");
    out
}

pub fn strip_managed_block(content: &str, begin: &str, end: &str) -> String {
    let mut output = String::new();
    let mut skip = false;
    for line in content.lines() {
        if line == begin {
            skip = true;
            continue;
        }
        if line == end {
            skip = false;
            continue;
        }
        if !skip {
            output.push_str(line);
            output.push('\n');
        }
    }
    // Trim trailing blank lines left by removed block
    while output.ends_with("\n\n") {
        output.pop();
    }
    output
}

pub fn write_codex_config_block(
    config_path: &Path,
    entries: &[CodexConfigEntry],
    source_prefix: &str,
    dry_run: bool,
) -> Result<(), String> {
    let existing = std::fs::read_to_string(config_path).unwrap_or_default();
    let stripped = strip_managed_block(&existing, CODEX_BLOCK_BEGIN, CODEX_BLOCK_END);

    let block = format_codex_config_block(entries, source_prefix);

    let mut rendered = String::new();
    if !stripped.is_empty() {
        rendered.push_str(&stripped);
        if !stripped.ends_with('\n') {
            rendered.push('\n');
        }
        rendered.push('\n');
    }
    rendered.push_str(&block);

    if !dry_run {
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
        }
        std::fs::write(config_path, &rendered)
            .map_err(|e| format!("failed to write {}: {e}", config_path.display()))?;
    }

    Ok(())
}

pub fn clean_codex_config_block(config_path: &Path, dry_run: bool) -> Result<(), String> {
    let Ok(existing) = std::fs::read_to_string(config_path) else {
        return Ok(());
    };

    if !existing.contains(CODEX_BLOCK_BEGIN) {
        return Ok(());
    }

    let stripped = strip_managed_block(&existing, CODEX_BLOCK_BEGIN, CODEX_BLOCK_END);

    if !dry_run {
        std::fs::write(config_path, &stripped)
            .map_err(|e| format!("failed to write {}: {e}", config_path.display()))?;
    }

    Ok(())
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests;
