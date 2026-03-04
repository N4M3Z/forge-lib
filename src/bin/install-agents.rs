use forge_lib::deploy::provider::Provider;
use forge_lib::deploy::{self, CodexConfigEntry, DeployResult};
use forge_lib::manifest;
use forge_lib::parse;
use forge_lib::sidecar::SidecarConfig;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

struct Args {
    src_dir: String,
    scope: String,
    dry_run: bool,
    clean: bool,
    dst_override: Option<String>,
}

fn parse_args() -> Result<Args, ExitCode> {
    let args: Vec<String> = env::args().collect();
    let mut src_dir: Option<String> = None;
    let mut scope = "all".to_string();
    let mut dry_run = false;
    let mut clean = false;
    let mut dst_override: Option<String> = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--version" => {
                println!("install-agents {}", env!("CARGO_PKG_VERSION"));
                return Err(ExitCode::SUCCESS);
            }
            "--dry-run" => dry_run = true,
            "--clean" => clean = true,
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
            "-h" | "--help" => {
                println!(
                    "Usage: install-agents <agents-dir> [--scope user|workspace|project|all] \
                     [--dry-run] [--clean] [--dst <path>]"
                );
                return Err(ExitCode::SUCCESS);
            }
            arg if arg.starts_with('-') => {
                eprintln!("Error: unknown flag {arg}");
                return Err(ExitCode::from(1));
            }
            _ => {
                src_dir = Some(args[i].clone());
            }
        }
        i += 1;
    }

    let Some(src_dir) = src_dir else {
        eprintln!("Error: source directory required.");
        eprintln!(
            "Usage: install-agents <agents-dir> [--scope user|workspace|all] \
             [--dry-run] [--clean] [--dst <path>]"
        );
        return Err(ExitCode::from(1));
    };

    Ok(Args {
        src_dir,
        scope,
        dry_run,
        clean,
        dst_override,
    })
}

fn read_module_name(input_dir: &Path) -> Option<String> {
    let module_root = input_dir.parent()?;
    let content = std::fs::read_to_string(module_root.join("module.yaml")).ok()?;
    forge_lib::parse::module_name(&content)
}

fn sync_manifest(
    dst_dir: &Path,
    module_name: &str,
    installed: &[String],
    provider: Provider,
    dry_run: bool,
) {
    match deploy::clean_orphaned_agents(dst_dir, module_name, installed, provider, dry_run) {
        Ok(orphans) => {
            let ext = provider.agent_extension();
            for name in &orphans {
                if dry_run {
                    println!("[dry-run] Would remove orphan: {name}.{ext}");
                } else {
                    println!("Removed orphan: {name}.{ext}");
                }
            }
        }
        Err(e) => eprintln!("Warning: orphan scan failed: {e}"),
    }

    if !dry_run {
        if let Err(e) = manifest::update(dst_dir, module_name, installed) {
            eprintln!("Warning: manifest update failed: {e}");
        }
    }
}

fn sync_codex_config(
    dst_dir: &Path,
    src_path: &Path,
    config: &SidecarConfig,
    source_prefix: &str,
    dry_run: bool,
) -> Result<(), ExitCode> {
    let provider = Provider::Codex;
    let codex_root = dst_dir.parent().unwrap_or(dst_dir);
    let config_path = codex_root.join("config.toml");
    let entries = collect_codex_entries(src_path, provider, config, source_prefix);
    if let Err(e) = deploy::write_codex_config_block(&config_path, &entries, source_prefix, dry_run)
    {
        eprintln!("Error writing config.toml: {e}");
        return Err(ExitCode::from(1));
    }
    if dry_run {
        println!(
            "[dry-run] Would write config.toml with {} agent entries",
            entries.len()
        );
    } else {
        println!(
            "Updated {} with {} agent entries",
            config_path.display(),
            entries.len()
        );
    }
    Ok(())
}

fn run(args: &Args) -> ExitCode {
    let src_path = Path::new(&args.src_dir);
    if !src_path.is_dir() {
        eprintln!("Error: not a directory: {}", args.src_dir);
        return ExitCode::from(1);
    }

    let module_name = read_module_name(src_path).unwrap_or_default();
    let source_prefix = if module_name.is_empty() {
        String::new()
    } else {
        format!("{module_name}/{}", args.src_dir)
    };

    let module_root = src_path.parent().unwrap_or(Path::new("."));
    let config = SidecarConfig::load(module_root);

    let dirs = if let Some(ref dst) = args.dst_override {
        vec![PathBuf::from(dst)]
    } else {
        let home = env::var("HOME").unwrap_or_default();
        let providers = config.providers();
        match deploy::scope_dirs(&args.scope, Path::new(&home), &providers) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error: {e}");
                return ExitCode::from(1);
            }
        }
    };

    for dst_dir in &dirs {
        let provider = Provider::from_path(dst_dir);
        eprintln!("Targeting provider directory: {}", dst_dir.display());

        if args.clean {
            match deploy::clean_agents(src_path, dst_dir, provider, args.dry_run) {
                Ok(removed) => {
                    let ext = provider.agent_extension();
                    for name in &removed {
                        if args.dry_run {
                            println!("[dry-run] Would remove: {name}.{ext}");
                        } else {
                            println!("Removed: {name}.{ext}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    return ExitCode::from(1);
                }
            }

            if provider == Provider::Codex {
                let codex_root = dst_dir.parent().unwrap_or(dst_dir);
                let config_path = codex_root.join("config.toml");
                if let Err(e) = deploy::clean_codex_config_block(&config_path, args.dry_run) {
                    eprintln!("Error cleaning config.toml: {e}");
                    return ExitCode::from(1);
                }
                if args.dry_run {
                    println!("[dry-run] Would clean config.toml managed block");
                } else {
                    println!("Cleaned config.toml managed block");
                }
            }
        }

        let installed = match deploy_to_dir(
            src_path,
            dst_dir,
            provider,
            &config,
            args.dry_run,
            &source_prefix,
        ) {
            Ok(names) => names,
            Err(code) => return code,
        };

        if !module_name.is_empty() {
            sync_manifest(dst_dir, &module_name, &installed, provider, args.dry_run);
        }

        if provider == Provider::Codex {
            if let Err(code) =
                sync_codex_config(dst_dir, src_path, &config, &source_prefix, args.dry_run)
            {
                return code;
            }
        }
    }

    ExitCode::SUCCESS
}

fn deploy_to_dir(
    src_path: &Path,
    dst_dir: &Path,
    provider: Provider,
    config: &SidecarConfig,
    dry_run: bool,
    source_prefix: &str,
) -> Result<Vec<String>, ExitCode> {
    let results =
        deploy::deploy_agents_from_dir(src_path, dst_dir, provider, config, dry_run, source_prefix)
            .map_err(|e| {
                eprintln!("Error: {e}");
                ExitCode::from(1)
            })?;

    let ext = provider.agent_extension();
    let mut installed = Vec::new();
    for (filename, result) in &results {
        let name = filename.trim_end_matches(".md");
        match result {
            DeployResult::Deployed => {
                let source_path = src_path.join(filename);
                let deployed_name = std::fs::read_to_string(&source_path)
                    .ok()
                    .and_then(|content| {
                        deploy::extract_agent_meta(
                            &content,
                            filename,
                            provider,
                            config,
                            source_prefix,
                        )
                        .map(|meta| meta.name)
                    })
                    .unwrap_or_else(|| name.to_string());
                installed.push(deployed_name);
                if dry_run {
                    println!(
                        "[dry-run] Would install: {name}.{ext} to {}",
                        dst_dir.display()
                    );
                } else {
                    println!("Installed: {name}.{ext} to {}", dst_dir.display());
                }
            }
            DeployResult::SkippedUserOwned => {
                eprintln!("Warning: Skipping {name}.{ext} — user-created agent (no source field)");
            }
            DeployResult::SkippedTemplate | DeployResult::SkippedNoName => {}
        }
    }
    Ok(installed)
}

fn collect_codex_entries(
    src_dir: &Path,
    provider: Provider,
    config: &SidecarConfig,
    source_prefix: &str,
) -> Vec<CodexConfigEntry> {
    let Ok(rd) = std::fs::read_dir(src_dir) else {
        return Vec::new();
    };

    let mut files: Vec<_> = rd
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    files.sort_by_key(std::fs::DirEntry::file_name);

    let mut entries = Vec::new();
    for entry in files {
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().to_string();
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Some(meta) =
            deploy::extract_agent_meta(&content, &filename, provider, config, source_prefix)
        {
            if parse::validate_agent_name(&meta.name).is_ok() {
                entries.push(CodexConfigEntry {
                    name: meta.name,
                    description: meta.description,
                });
            }
        }
    }

    entries
}

fn main() -> ExitCode {
    match parse_args() {
        Ok(ref args) => run(args),
        Err(code) => code,
    }
}
