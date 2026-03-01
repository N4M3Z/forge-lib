# GEMINI.md

This file provides instructional context for the Gemini AI agent when working with the **forge-lib** codebase.

## Project Overview

**forge-lib** is the core shared library and CLI toolkit for the Forge Framework. It provides essential utilities for frontmatter parsing, markdown processing, agent deployment, and skill installation used across all Forge modules.

### Core Responsibilities
- **Frontmatter Parsing:** Extracting and validating YAML frontmatter from markdown files.
- **Markdown Processing:** Stripping metadata and headings for clean consumption by AI providers.
- **Agent Deployment:** Deploying agent configurations to platform-specific directories (Claude, Gemini, Codex, OpenCode).
- **Skill Installation:** Generating and routing skill wrappers for modular AI capabilities.
- **Module Validation:** Enforcing Forge conventions through automated structural and metadata checks.

## Building and Testing

The project uses a `Makefile` to manage the Rust toolchain and binary symlinking.

### Key Commands
- `make build`: Compiles all Rust binaries in release mode and symlinks them to the `bin/` directory.
- `make test`: Runs the full test suite (~240 tests), including library and integration tests.
- `make lint`: Executes `cargo fmt` and `clippy` (pedantic) to ensure code quality.
- `make check`: Diagnostic to verify that all expected binaries are present in the `bin/` directory.
- `make clean`: Removes build artifacts and the `bin/` directory.

## Architecture & Modules

The library is organized into seven functional modules:

| Module | Key Functions / Responsibilities |
| :--- | :--- |
| `dci` | `extract_dci_lines`, `extract_bash_block_lines`, `validate_dci` — DCI and bash block validation. |
| `parse` | `fm_value`, `fm_body`, `fm_list`, `split_frontmatter` — Low-level YAML extraction. |
| `strip` | `strip_front`, `strip_front_keep` — Clean markdown by removing metadata and H1 headings. |
| `sidecar` | `SidecarConfig::load` — Loading agent and skill YAML configurations. |
| `deploy` | `deploy_agents_from_dir`, `clean_agents` — Multi-platform agent distribution. |
| `skill` | `plan_skills_from_dir`, `generate_skills_from_agents_dir` — Skill lifecycle management. |
| `validate` | `validate_structure`, `validate_agent_frontmatter` — Forge convention enforcement. |

## CLI Binaries

Binaries are located in `src/bin/` and symlinked to `bin/` after running `make build`.

| Binary | Purpose |
| :--- | :--- |
| `strip-front` | Strips YAML frontmatter and H1 heading from a markdown file for clean input. |
| `install-agents` | Deploys agent files to provider-specific directories (Claude, Gemini, Codex, OpenCode). |
| `install-skills` | Installs skills with provider-specific routing and wrapper generation. |
| `validate-module` | Runs a convention test suite against a Forge module to ensure compliance. |

## Submodule Integration

Forge modules consume `forge-lib` as a git submodule, typically located at `lib/`.

### Consumption Pattern
1.  **Add Submodule:** `git submodule add https://github.com/N4M3Z/forge-lib.git lib`
2.  **Initialize/Build:** `make -C lib build`
3.  **Reference Binaries:** Use `lib/bin/install-agents`, etc., in the parent module's `Makefile`.

### Makefile Integration Example
```makefile
LIB_DIR = lib
INSTALL_AGENTS := $(LIB_DIR)/bin/install-agents

$(INSTALL_AGENTS):
	@$(MAKE) -C $(LIB_DIR) build

deploy: $(INSTALL_AGENTS)
	@$(INSTALL_AGENTS) agents --scope workspace
```

## Development Conventions

- **Error Handling:** Uses `Option<T>` or `Result<T, String>`. Avoids complex error enums for simplicity across the framework.
- **Safety:** `#![forbid(unsafe_code)]` is strictly enforced.
- **I/O Separation:** Library functions in `src/` are generally pure/logic-focused; I/O is handled by the thin CLI wrappers in `src/bin/`.
- **Testing:** Unit tests are located in `src/<module>/tests.rs`, and integration tests in `tests/`.
