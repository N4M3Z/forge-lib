use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use forge_lib::validate;

fn print_suite(suite: &validate::Suite) {
    println!("\n=== {} ===", suite.name);
    for check in &suite.checks {
        if check.passed {
            println!("  PASS: {}", check.desc);
        } else {
            println!("  FAIL: {}", check.desc);
        }
    }
    println!();
    println!("--- {} ---", suite.name);
    println!("  Passed: {}", suite.passed());
    println!("  Failed: {}", suite.failed());
    let failures: Vec<_> = suite
        .checks
        .iter()
        .filter(|c| !c.passed)
        .map(|c| &c.desc)
        .collect();
    if !failures.is_empty() {
        println!("  Failures:");
        for f in &failures {
            println!("    - {f}");
        }
    }
    println!();
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--version") {
        println!("validate-module {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    if args.iter().any(|a| a == "--help" || a == "-h") {
        eprintln!("Usage: validate-module [module-root]");
        eprintln!();
        eprintln!("Validates forge module structure, agents, defaults, skills, and deploy parity.");
        eprintln!("Defaults to current directory if no module-root is specified.");
        return ExitCode::SUCCESS;
    }

    let root = if args.len() > 1 && !args[1].starts_with('-') {
        PathBuf::from(&args[1])
    } else {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    if !root.is_dir() {
        eprintln!("Error: not a directory: {}", root.display());
        return ExitCode::from(1);
    }

    let suites = [
        validate::validate_structure(&root),
        validate::validate_agent_frontmatter(&root),
        validate::validate_defaults(&root),
        validate::validate_skills(&root),
        validate::validate_deploy_parity(&root),
    ];

    let mut total_fail = 0;
    for suite in &suites {
        print_suite(suite);
        total_fail += suite.failed();
    }

    let warnings = validate::warn_skill_content(&root);
    if !warnings.checks.is_empty() {
        println!("\n=== {} ===", warnings.name);
        for check in &warnings.checks {
            if check.passed {
                println!("  OK:   {}", check.desc);
            } else {
                println!("  WARN: {}", check.desc);
            }
        }
        if warnings.failed() > 0 {
            println!("\n  ({} warnings — not counted as failures)", warnings.failed());
        }
        println!();
    }

    if total_fail > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
