use forge_lib::codex;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

struct Args {
    codex_dir: PathBuf,
    module_root: PathBuf,
}

fn parse_args() -> Result<Args, ExitCode> {
    let args: Vec<String> = env::args().collect();
    let mut codex_dir: Option<PathBuf> = None;
    let mut module_root = PathBuf::from(".");
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--version" => {
                println!("install-codex-agent-config {}", env!("CARGO_PKG_VERSION"));
                return Err(ExitCode::SUCCESS);
            }
            "--module-root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --module-root requires a value");
                    return Err(ExitCode::from(1));
                }
                module_root = PathBuf::from(&args[i]);
            }
            "-h" | "--help" => {
                println!("Usage: install-codex-agent-config <codex-dir> [--module-root <path>]");
                return Err(ExitCode::SUCCESS);
            }
            arg if arg.starts_with('-') => {
                eprintln!("Error: unknown flag {arg}");
                return Err(ExitCode::from(1));
            }
            value => {
                codex_dir = Some(PathBuf::from(value));
            }
        }
        i += 1;
    }

    let Some(codex_dir) = codex_dir else {
        eprintln!("Error: codex directory required.");
        eprintln!("Usage: install-codex-agent-config <codex-dir> [--module-root <path>]");
        return Err(ExitCode::from(1));
    };

    Ok(Args {
        codex_dir,
        module_root,
    })
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(code) => return code,
    };

    match codex::render_codex_agent_config(&args.module_root, &args.codex_dir) {
        Ok(result) => {
            println!(
                "Generated Codex role config in {}",
                result.agents_dir.display()
            );
            println!("Updated Codex config: {}", result.config_file.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(1)
        }
    }
}
