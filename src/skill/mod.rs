use crate::deploy::provider::Provider;
use crate::parse;
use crate::sidecar::SidecarConfig;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

// ─── Types ───

#[derive(Debug, Clone, PartialEq)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    pub claude_fields: BTreeMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub enum SkillInstallAction {
    Copy {
        skill_name: String,
        src_dir: PathBuf,
        dst_dir: PathBuf,
        claude_fields: BTreeMap<String, String>,
    },
    GeminiCli {
        skill_name: String,
        skill_dir: PathBuf,
        scope: String,
    },
    Skipped {
        skill_name: String,
        reason: String,
    },
}

#[derive(Debug, PartialEq)]
pub struct GeneratedSkill {
    pub agent_name: String,
    pub skill_md: String,
    pub skill_yaml: String,
}

// ─── Skill Meta Extraction ───

pub fn extract_skill_meta(skill_dir: &Path) -> Option<SkillMeta> {
    let md_path = skill_dir.join("SKILL.md");
    let content = std::fs::read_to_string(&md_path).ok()?;

    let name = parse::fm_value(&content, "name").filter(|n| !n.is_empty())?;
    let description = parse::fm_value(&content, "description").unwrap_or_else(|| "Skill".into());

    let claude_fields = read_claude_fields(&skill_dir.join("SKILL.yaml"));

    Some(SkillMeta {
        name,
        description,
        claude_fields,
    })
}

fn read_claude_fields(yaml_path: &Path) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();

    let Ok(content) = std::fs::read_to_string(yaml_path) else {
        return fields;
    };
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
        return fields;
    };

    let Some(claude) = value
        .as_mapping()
        .and_then(|m| m.get(serde_yaml::Value::String("claude".into())))
        .and_then(serde_yaml::Value::as_mapping)
    else {
        return fields;
    };

    for (k, v) in claude {
        if let Some(key) = k.as_str() {
            let val = match v {
                serde_yaml::Value::String(s) => s.clone(),
                serde_yaml::Value::Bool(b) => b.to_string(),
                serde_yaml::Value::Number(n) => n.to_string(),
                _ => continue,
            };
            fields.insert(key.to_string(), val);
        }
    }

    fields
}

// ─── Install Planning ───

pub fn plan_skill_install(
    meta: &SkillMeta,
    skill_dir: &Path,
    provider: Provider,
    dst_dir: &Path,
    default_scope: &str,
    config: &SidecarConfig,
) -> SkillInstallAction {
    let allowed = config.provider_skills(provider.as_str());
    if !allowed.iter().any(|s| s == &meta.name) {
        return SkillInstallAction::Skipped {
            skill_name: meta.name.clone(),
            reason: format!("not in {} allowlist", provider.as_str()),
        };
    }

    match provider {
        Provider::Gemini => {
            let scope = config
                .provider_skill_value(provider.as_str(), &meta.name, "scope")
                .unwrap_or_else(|| default_scope.to_string());
            SkillInstallAction::GeminiCli {
                skill_name: meta.name.clone(),
                skill_dir: skill_dir.to_path_buf(),
                scope,
            }
        }
        Provider::Claude | Provider::Codex => SkillInstallAction::Copy {
            skill_name: meta.name.clone(),
            src_dir: skill_dir.to_path_buf(),
            dst_dir: dst_dir.to_path_buf(),
            claude_fields: meta.claude_fields.clone(),
        },
    }
}

pub fn plan_skills_from_dir(
    root_dir: &Path,
    provider: Provider,
    dst_dir: &Path,
    default_scope: &str,
    config: &SidecarConfig,
) -> Result<Vec<SkillInstallAction>, String> {
    if !root_dir.is_dir() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(root_dir)
        .map_err(|e| format!("failed to read {}: {e}", root_dir.display()))?;

    let mut skill_dirs: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir() && e.path().join("SKILL.md").exists())
        .collect();
    skill_dirs.sort_by_key(std::fs::DirEntry::file_name);

    let mut actions = Vec::new();
    for entry in skill_dirs {
        let path = entry.path();
        let Some(meta) = extract_skill_meta(&path) else {
            continue;
        };
        actions.push(plan_skill_install(
            &meta,
            &path,
            provider,
            dst_dir,
            default_scope,
            config,
        ));
    }

    Ok(actions)
}

// ─── Skill Copy ───

pub fn execute_skill_copy(src_dir: &Path, skill_name: &str, dst_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst_dir)
        .map_err(|e| format!("failed to create {}: {e}", dst_dir.display()))?;

    let target = dst_dir.join(skill_name);
    if target.is_symlink() {
        return Err(format!("destination is a symlink: {}", target.display()));
    }
    if target.exists() {
        std::fs::remove_dir_all(&target)
            .map_err(|e| format!("failed to remove {}: {e}", target.display()))?;
    }

    copy_dir_recursive(src_dir, &target)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("failed to create {}: {e}", dst.display()))?;

    let entries =
        std::fs::read_dir(src).map_err(|e| format!("failed to read {}: {e}", src.display()))?;

    for entry in entries.filter_map(Result::ok) {
        let src_path = entry.path();
        let name = entry.file_name();
        if name == "SKILL.yaml" {
            continue;
        }
        let dst_path = dst.join(name);
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| {
                format!(
                    "failed to copy {} to {}: {e}",
                    src_path.display(),
                    dst_path.display()
                )
            })?;
        }
    }

    Ok(())
}

pub fn merge_claude_fields(skill_md: &str, fields: &BTreeMap<String, String>) -> String {
    if fields.is_empty() {
        return skill_md.to_string();
    }

    let Some((fm, body)) = parse::split_frontmatter(skill_md) else {
        // No frontmatter — wrap body with new frontmatter
        let mut out = String::from("---\n");
        for (k, v) in fields {
            let _ = writeln!(out, "{k}: {v}");
        }
        out.push_str("---\n");
        out.push_str(skill_md);
        return out;
    };

    let mut out = String::from("---\n");
    out.push_str(fm);
    if !fm.ends_with('\n') {
        out.push('\n');
    }
    for (k, v) in fields {
        // Only add if not already present in frontmatter
        let key_prefix = format!("{k}:");
        if !fm.lines().any(|line| line.starts_with(&key_prefix)) {
            let _ = writeln!(out, "{k}: {v}");
        }
    }
    out.push_str("---\n");
    out.push_str(body);

    out
}

// ─── Orphan Cleanup ───

pub fn clean_orphaned_skills(
    dst_dir: &Path,
    module_name: &str,
    current_skills: &[String],
    dry_run: bool,
) -> Result<Vec<String>, String> {
    if module_name.is_empty() {
        return Ok(Vec::new());
    }

    let previous = crate::manifest::read(dst_dir, module_name);
    let mut removed = Vec::new();

    for name in &previous {
        if current_skills.contains(name) {
            continue;
        }
        let path = dst_dir.join(name);
        if !path.is_dir() {
            continue;
        }
        if !dry_run {
            std::fs::remove_dir_all(&path)
                .map_err(|e| format!("failed to remove {}: {e}", path.display()))?;
        }
        removed.push(name.clone());
    }

    Ok(removed)
}

// ─── Skill Generation (Codex wrappers) ───

pub fn format_agent_skill_md(
    agent_name: &str,
    description: &str,
    body: &str,
    source_filename: &str,
) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    let _ = writeln!(out, "name: {agent_name}");
    let _ = writeln!(out, "description: \"{}\"", escape_yaml_string(description));
    let _ = writeln!(
        out,
        "argument-hint: \"[task, files, or question for {agent_name}]\""
    );
    out.push_str("---\n\n");
    let _ = writeln!(out, "# {agent_name}");
    out.push('\n');
    let _ = writeln!(
        out,
        "> Generated from agents/{source_filename}. Do not edit manually."
    );
    out.push('\n');
    out.push_str("Use the specialist guidance below to handle the user's request.\n\n");
    out.push_str(body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out
}

pub fn format_agent_skill_yaml(
    agent_name: &str,
    description: &str,
    source_filename: &str,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "name: {agent_name}");
    let _ = writeln!(out, "description: \"{}\"", escape_yaml_string(description));
    let _ = writeln!(
        out,
        "argument-hint: \"[task, files, or question for {agent_name}]\""
    );
    out.push_str("providers:\n");
    out.push_str("  claude:\n");
    out.push_str("    enabled: false\n");
    out.push_str("  gemini:\n");
    out.push_str("    enabled: false\n");
    out.push_str("  codex:\n");
    out.push_str("    enabled: true\n");
    out.push_str("generation:\n");
    out.push_str("  method: generated-from-agent\n");
    let _ = writeln!(out, "  agent: {agent_name}");
    let _ = writeln!(out, "  source: {source_filename}");
    out
}

fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn generate_skill_from_agent(content: &str, filename: &str) -> Option<GeneratedSkill> {
    let agent_name = parse::fm_value(content, "claude.name")
        .or_else(|| parse::fm_value(content, "title"))
        .filter(|n| !n.is_empty())?;

    let description = parse::fm_value(content, "claude.description")
        .or_else(|| parse::fm_value(content, "description"))
        .unwrap_or_else(|| "Specialist skill".into());

    let body = parse::fm_body(content);

    let skill_md = format_agent_skill_md(&agent_name, &description, body, filename);
    let skill_yaml = format_agent_skill_yaml(&agent_name, &description, filename);

    Some(GeneratedSkill {
        agent_name,
        skill_md,
        skill_yaml,
    })
}

pub fn generate_skills_from_agents_dir(agents_dir: &Path) -> Result<Vec<GeneratedSkill>, String> {
    if !agents_dir.is_dir() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(agents_dir)
        .map_err(|e| format!("failed to read {}: {e}", agents_dir.display()))?;

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
        if let Some(skill) = generate_skill_from_agent(&content, &filename) {
            results.push(skill);
        }
    }

    Ok(results)
}

// ─── Council roster helpers (used by validate module) ───

pub fn get_council_roles(config: &SidecarConfig, council: &str) -> Vec<String> {
    config
        .skill_value(council, "roles")
        .map(|s| {
            s.lines()
                .map(|l| l.trim().trim_start_matches("- ").to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
