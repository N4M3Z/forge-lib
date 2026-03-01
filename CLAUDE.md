# CLAUDE.md

Shared Rust library consumed as a git submodule at `lib/`. Not a Claude Code plugin -- no hooks, no plugin.json. Ships one skill (BuildSystem) for fallback installation without Rust.

## API Surface

Seven library modules:

| Module | Key Functions |
|--------|--------------|
| `dci` | `extract_dci_lines`, `extract_bash_block_lines`, `validate_dci` |
| `parse` | `fm_value`, `fm_body`, `fm_list`, `split_frontmatter` |
| `strip` | `strip_front`, `strip_front_keep` |
| `sidecar` | `SidecarConfig::load`, `agent_value`, `skill_value` |
| `deploy` | `deploy_agents_from_dir`, `clean_agents`, `scope_dirs` |
| `skill` | `plan_skills_from_dir`, `generate_skills_from_agents_dir`, `get_council_roles` |
| `validate` | `validate_structure`, `validate_agent_frontmatter`, `validate_skills`, `validate_deploy_parity` |

## CLI Binaries

| Binary | Purpose |
|--------|---------|
| `strip-front` | Strip YAML frontmatter and H1 heading from markdown |
| `install-agents` | Deploy agent markdown to Claude/Gemini/Codex/OpenCode directories |
| `install-skills` | Install skills with provider-specific routing and wrapper generation |
| `validate-module` | Convention test suite for forge modules |

All binaries support `--version` and `--help`. All support all providers (Claude, Gemini, Codex, OpenCode).

## Skills

| Skill | What it does |
|-------|-------------|
| **BuildSystem** | Fallback installation without Rust — per-provider adapters (ClaudeAdapter, GeminiAdapter, CodexAdapter) |

## Shell Fallback Scripts

When Rust is unavailable, per-provider scripts install skills/agents/rules by copying files:

| Script | Provider | Installs |
|--------|----------|----------|
| `scripts/claude/install.sh` | Claude Code | Skills + agents + rules |
| `scripts/gemini/install.sh` | Gemini CLI | Skills (kebab-cased) |
| `scripts/codex/install.sh` | Codex CLI | Skills |

## Build & Test

```bash
make build    # cargo build --release + symlink to bin/
make test     # cargo test
make lint     # cargo fmt --check + clippy pedantic
make check    # verify bin/ symlinks
make clean    # cargo clean + rm bin/
```

## Conventions

- Error handling: `Option<T>` / `Result<T, String>` -- no custom error enums
- `unsafe` forbidden (`#![forbid(unsafe_code)]`)
- Clippy pedantic warnings enabled
- Pure core + thin CLI wrapper: library functions do no I/O, binaries handle it
- `serde_yaml` for all YAML parsing
- Test pattern: `mod.rs` + sibling `tests.rs` for unit tests, `tests/` for integration

## Consuming as Submodule

```bash
git submodule add https://github.com/N4M3Z/forge-lib.git lib
make -C lib build
```

Reference binaries at `lib/bin/install-agents`, `lib/bin/validate-module`, etc.

```makefile
LIB_DIR = $(or $(FORGE_LIB),lib)
INSTALL_AGENTS := $(LIB_DIR)/bin/install-agents

$(INSTALL_AGENTS):
	@$(MAKE) -C $(LIB_DIR) build
```

## Updating

```bash
git -C lib pull
make -C lib build
git add lib
git commit -m "chore: update forge-lib submodule"
```
