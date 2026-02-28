# AGENTS.md -- forge-lib

> Shared Rust library and CLI binaries for the Forge ecosystem. Provides
> frontmatter parsing, agent deployment, skill installation, and module
> validation. Consumed as a git submodule -- not a standalone plugin.

## Build / Install / Verify

```bash
make build    # cargo build --release + symlink to bin/
make test     # cargo test (~240 tests)
make lint     # cargo fmt --check + clippy pedantic
make check    # verify bin/ symlinks
make clean    # cargo clean + rm bin/
```

No plugin installation -- forge-lib is a dependency pulled in as a git submodule
at `lib/` by other Forge modules.

## Project Structure

```
src/
  parse/       # fm_value, fm_body, fm_list, split_frontmatter
  strip/       # strip_front, strip_front_keep
  sidecar/     # SidecarConfig::load, agent_value, skill_value
  deploy/      # deploy_agents_from_dir, clean_agents, scope_dirs
  skill/       # plan_skills_from_dir, generate_skills_from_agents_dir
  validate/    # validate_structure, validate_agent_frontmatter, validate_skills
  bin/
    strip-front.rs       # Strip YAML frontmatter and H1 heading from markdown
    install-agents.rs    # Deploy agent markdown to Claude/Gemini/Codex/OpenCode
    install-skills.rs    # Install skills with provider-specific routing
    validate-module.rs   # Convention test suite for forge modules (5 suites)
tests/                   # Integration tests
bin/                     # Symlinked binaries (created by make build)
Cargo.toml
Makefile
module.yaml              # Module metadata (name, version)
```

## CLI Binaries

All binaries support `--version` and `--help`. All support all providers
(Claude, Gemini, Codex, OpenCode).

| Binary | Purpose |
|--------|---------|
| `strip-front` | Strip YAML frontmatter and H1 heading from markdown |
| `install-agents` | Deploy agent markdown to provider-specific directories (Codex renders `.toml` with inline `developer_instructions`) |
| `install-skills` | Install skills with provider routing and wrapper generation |
| `validate-module` | Convention test suite for forge modules (5 suites) |

### validate-module Suites

```bash
bin/validate-module path/to/module
```

Five suites: structure (required files), agent frontmatter (YAML correctness),
defaults consistency (roster vs files), skill integrity (SKILL.yaml + SKILL.md),
deploy parity (installed matches source).

## Consuming as Submodule

```bash
git submodule add https://github.com/N4M3Z/forge-lib.git lib
make -C lib build
```

Reference binaries at `lib/bin/install-agents`, `lib/bin/validate-module`, etc.

### Makefile Integration

```makefile
LIB_DIR = $(or $(FORGE_LIB),lib)
INSTALL_AGENTS := $(LIB_DIR)/bin/install-agents

$(INSTALL_AGENTS):
	@$(MAKE) -C $(LIB_DIR) build

install: $(INSTALL_AGENTS)
	@$(INSTALL_AGENTS) agents --scope workspace
```

## Development Conventions

- **Error handling**: `Option<T>` / `Result<T, String>` -- no custom error enums
- **Safety**: `#![forbid(unsafe_code)]` strictly enforced
- **I/O separation**: Library functions are pure (no I/O), binaries are thin CLI wrappers
- **Clippy pedantic**: All warnings enabled
- **YAML**: `serde_yaml` for all parsing
- **Testing**: Unit tests in `src/<module>/tests.rs`, integration tests in `tests/`

### Test Fixtures

- `tests/fixtures/codex/` contains canonical Codex TOML render outputs and
  corresponding prompt markdown sources used by deploy tests.
- Keep these fixtures in-repo. They are golden references for Codex rendering
  behavior and protect against format regressions.

## Git Conventions

Conventional Commits: `type: description`. Lowercase, no trailing period, no scope.

Types: `feat`, `fix`, `docs`

```
feat: add validate module convention test suite
fix: correct provider enum kebab-case for Gemini
docs: update README with validate-module binary
```
