use forge_lib::deploy::provider::Provider;
use forge_lib::manifest;
use forge_lib::sidecar::SidecarConfig;
use forge_lib::skill::{self, SkillInstallAction};
use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

struct Args {
    skills_dir: String,
    provider: Provider,
    scope: String,
    dry_run: bool,
    clean: bool,
    dst_override: Option<String>,
    agents_dir: String,
    include_agent_wrappers: bool,
}

fn parse_args() -> Result<Args, ExitCode> {
    let args: Vec<String> = env::args().collect();
    let mut skills_dir: Option<String> = None;
    let mut provider_str: Option<String> = None;
    let mut scope = "workspace".to_string();
    let mut dry_run = false;
    let mut clean = false;
    let mut dst_override: Option<String> = None;
    let mut agents_dir = "agents".to_string();
    let mut include_agent_wrappers = false;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--version" => {
                println!("install-skills {}", env!("CARGO_PKG_VERSION"));
                return Err(ExitCode::SUCCESS);
            }
            "--provider" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --provider requires a value");
                    return Err(ExitCode::from(1));
                }
                provider_str = Some(args[i].clone());
            }
            "--scope" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --scope requires a value");
                    return Err(ExitCode::from(1));
                }
                scope.clone_from(&args[i]);
            }
            "--dst" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --dst requires a value");
                    return Err(ExitCode::from(1));
                }
                dst_override = Some(args[i].clone());
            }
            "--agents-dir" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --agents-dir requires a value");
                    return Err(ExitCode::from(1));
                }
                agents_dir.clone_from(&args[i]);
            }
            "--dry-run" => dry_run = true,
            "--clean" => clean = true,
            "--include-agent-wrappers" => include_agent_wrappers = true,
            "-h" | "--help" => {
                println!(
                    "Usage: install-skills <skills-dir> --provider claude|gemini|codex \
                     [--scope user|workspace] [--dry-run] [--clean] [--dst <path>] \
                     [--agents-dir <path>] [--include-agent-wrappers]"
                );
                return Err(ExitCode::SUCCESS);
            }
            arg if arg.starts_with('-') => {
                eprintln!("Error: unknown flag {arg}");
                return Err(ExitCode::from(1));
            }
            _ => {
                skills_dir = Some(args[i].clone());
            }
        }
        i += 1;
    }

    let Some(skills_dir) = skills_dir else {
        eprintln!("Error: skills directory required.");
        eprintln!(
            "Usage: install-skills <skills-dir> --provider claude|gemini|codex \
             [--scope user|workspace] [--dry-run] [--clean] [--dst <path>]"
        );
        return Err(ExitCode::from(1));
    };

    let Some(ref prov) = provider_str else {
        eprintln!("Error: --provider is required.");
        return Err(ExitCode::from(1));
    };

    let Some(provider) = Provider::from_str(prov) else {
        eprintln!("Error: invalid provider {prov:?}: use claude, gemini, or codex");
        return Err(ExitCode::from(1));
    };

    Ok(Args {
        skills_dir,
        provider,
        scope,
        dry_run,
        clean,
        dst_override,
        agents_dir,
        include_agent_wrappers,
    })
}

fn read_module_name(input_dir: &Path) -> Option<String> {
    let module_root = input_dir.parent()?;
    let content = std::fs::read_to_string(module_root.join("module.yaml")).ok()?;
    forge_lib::parse::module_name(&content)
}

fn project_key() -> Result<String, String> {
    let cwd = env::current_dir().map_err(|e| format!("failed to get cwd: {e}"))?;
    Ok(cwd.to_string_lossy().replace('/', "-"))
}

fn resolve_dst(provider: Provider, scope: &str) -> Result<PathBuf, String> {
    let home = env::var("HOME").unwrap_or_default();
    let provider_dir = match provider {
        Provider::Claude => ".claude",
        Provider::Gemini => ".gemini",
        Provider::Codex => ".codex",
    };

    match scope {
        "user" => Ok(PathBuf::from(format!("{home}/{provider_dir}/skills"))),

        "project" => {
            let key = project_key()?;
            Ok(PathBuf::from(format!(
                "{home}/{provider_dir}/projects/{key}/skills"
            )))
        }

        "workspace" => Ok(PathBuf::from(format!("{provider_dir}/skills"))),

        other => Err(format!(
            "invalid scope: {other} (use user, project, or workspace)"
        )),
    }
}

fn clean_module_skills(dst_dir: &Path, module_name: &str, dry_run: bool) {
    if !dst_dir.is_dir() || module_name.is_empty() {
        return;
    }
    let previous = manifest::read(dst_dir, module_name);
    for name in &previous {
        let path = dst_dir.join(name);
        if path.is_dir() {
            if dry_run {
                println!("[dry-run] Would clean: {name}");
            } else {
                let _ = std::fs::remove_dir_all(&path);
            }
        }
    }
}

fn execute_action(action: &SkillInstallAction, dry_run: bool) -> Result<(), String> {
    match action {
        SkillInstallAction::Copy {
            skill_name,
            src_dir,
            dst_dir,
            claude_fields,
        } => {
            if dry_run {
                println!(
                    "[dry-run] Would install skill: {skill_name} -> {}",
                    dst_dir.display()
                );
            } else {
                skill::execute_skill_copy(src_dir, skill_name, dst_dir)?;
                if !claude_fields.is_empty() {
                    let md_path = dst_dir.join(skill_name).join("SKILL.md");
                    if let Ok(content) = std::fs::read_to_string(&md_path) {
                        let merged = skill::merge_claude_fields(&content, claude_fields);
                        std::fs::write(&md_path, &merged)
                            .map_err(|e| format!("failed to write {}: {e}", md_path.display()))?;
                    }
                }
                println!("Installed skill: {skill_name} -> {}", dst_dir.display());
            }
        }
        SkillInstallAction::GeminiCli {
            skill_name,
            skill_dir,
            scope,
        } => {
            if dry_run {
                println!("[dry-run] Would install Gemini skill: {skill_name} (scope: {scope})");
            } else {
                println!("Installing Gemini skill: {skill_name} (scope: {scope})...");
                let status = Command::new("gemini")
                    .args([
                        "skills",
                        "install",
                        &skill_dir.to_string_lossy(),
                        "--scope",
                        scope,
                    ])
                    .status()
                    .map_err(|e| format!("failed to run gemini CLI: {e}"))?;
                if !status.success() {
                    return Err(format!(
                        "gemini skills install failed for {skill_name} (exit {})",
                        status.code().unwrap_or(-1)
                    ));
                }
            }
        }
        SkillInstallAction::Skipped { .. } => {}
    }
    Ok(())
}

fn generate_and_plan_wrappers(
    agents_dir: &Path,
    _provider: Provider,
    dst_dir: &Path,
    _scope: &str,
    _config: &SidecarConfig,
) -> Result<(Vec<SkillInstallAction>, Option<tempfile::TempDir>), String> {
    let generated = skill::generate_skills_from_agents_dir(agents_dir)?;
    if generated.is_empty() {
        return Ok((Vec::new(), None));
    }

    let tmp_dir = tempfile::tempdir().map_err(|e| format!("failed to create temp dir: {e}"))?;

    let mut actions = Vec::new();
    for gen in &generated {
        let skill_dir = tmp_dir.path().join(&gen.agent_name);
        std::fs::create_dir_all(&skill_dir)
            .map_err(|e| format!("failed to create {}: {e}", skill_dir.display()))?;
        std::fs::write(skill_dir.join("SKILL.md"), &gen.skill_md)
            .map_err(|e| format!("failed to write SKILL.md: {e}"))?;
        std::fs::write(skill_dir.join("SKILL.yaml"), &gen.skill_yaml)
            .map_err(|e| format!("failed to write SKILL.yaml: {e}"))?;

        actions.push(SkillInstallAction::Copy {
            skill_name: gen.agent_name.clone(),
            src_dir: skill_dir,
            dst_dir: dst_dir.to_path_buf(),
            claude_fields: BTreeMap::new(),
        });
    }

    Ok((actions, Some(tmp_dir)))
}

fn run(args: &Args) -> ExitCode {
    let skills_path = Path::new(&args.skills_dir);
    if !skills_path.is_dir() {
        eprintln!("Error: not a directory: {}", args.skills_dir);
        return ExitCode::from(1);
    }

    let dst_dir = match &args.dst_override {
        Some(dst) => PathBuf::from(dst),
        None => match resolve_dst(args.provider, &args.scope) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error: {e}");
                return ExitCode::from(1);
            }
        },
    };

    let module_root = skills_path.parent().unwrap_or(Path::new("."));
    let config = SidecarConfig::load(module_root);

    let module_name = read_module_name(skills_path).unwrap_or_default();

    if args.clean {
        clean_module_skills(&dst_dir, &module_name, args.dry_run);
    }

    let mut actions = match skill::plan_skills_from_dir(
        skills_path,
        args.provider,
        &dst_dir,
        &args.scope,
        &config,
    ) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::from(1);
        }
    };

    let mut _wrapper_tmpdir = None;
    if args.include_agent_wrappers && args.provider != Provider::Gemini {
        let agents_path = Path::new(&args.agents_dir);
        match generate_and_plan_wrappers(agents_path, args.provider, &dst_dir, &args.scope, &config)
        {
            Ok((extra, tmpdir)) => {
                actions.extend(extra);
                _wrapper_tmpdir = tmpdir;
            }
            Err(e) => {
                eprintln!("Error generating agent wrappers: {e}");
                return ExitCode::from(1);
            }
        }
    }

    for action in &actions {
        if let Err(e) = execute_action(action, args.dry_run) {
            eprintln!("Error: {e}");
            return ExitCode::from(1);
        }
    }

    if !module_name.is_empty() && args.provider != Provider::Gemini {
        let installed: Vec<String> = actions
            .iter()
            .filter_map(|a| match a {
                SkillInstallAction::Copy { skill_name, .. } => Some(skill_name.clone()),
                _ => None,
            })
            .collect();

        match skill::clean_orphaned_skills(&dst_dir, &module_name, &installed, args.dry_run) {
            Ok(orphans) => {
                for name in &orphans {
                    if args.dry_run {
                        println!("[dry-run] Would remove orphaned skill: {name}");
                    } else {
                        println!("Removed orphaned skill: {name}");
                    }
                }
            }
            Err(e) => eprintln!("Warning: skill orphan scan failed: {e}"),
        }

        if !args.dry_run {
            if let Err(e) = manifest::update(&dst_dir, &module_name, &installed) {
                eprintln!("Warning: manifest update failed: {e}");
            }
        }
    }

    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    match parse_args() {
        Ok(ref args) => run(args),
        Err(code) => code,
    }
}
