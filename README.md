# forge-lib

Shared Rust library and CLI binaries for Forge modules. Provides frontmatter parsing, agent deployment, skill installation, and markdown processing.

## Build

```bash
make build    # cargo build --release + symlinks to bin/
make test     # cargo test (222 tests)
make lint     # cargo fmt --check + clippy
make check    # verify bin/ has all binaries
make clean    # cargo clean + rm bin/
```

Binaries are symlinked into `bin/` for submodule consumers. The `bin/` directory is gitignored and lazily populated on first `make build`.

## Usage

### As a submodule

Add forge-lib as a git submodule at `lib/`:

```bash
cd your-module/
git submodule add https://github.com/N4M3Z/forge-lib.git lib
```

Reference binaries at `lib/bin/`:

```makefile
LIB_DIR = lib
INSTALL_AGENTS := $(LIB_DIR)/bin/install-agents

# Auto-build when binaries are missing
$(INSTALL_AGENTS):
	@$(MAKE) -C $(LIB_DIR) build

install-agents: $(INSTALL_AGENTS)
	@$(INSTALL_AGENTS) agents --scope workspace
```

### Library crate

```toml
[dependencies]
forge-lib = { path = "lib" }
```

Five modules: `parse` (frontmatter), `strip` (markdown processing), `sidecar` (YAML config), `deploy` (agent deployment), `skill` (skill installation).

## CLI Binaries

| Binary | Purpose |
|--------|---------|
| `strip-front` | Strip YAML frontmatter and H1 heading from markdown |
| `install-agents` | Deploy agent markdown files to Claude/Gemini/Codex directories |
| `install-skills` | Install skills with provider-specific routing and wrapper generation |
| `install-codex-agent-config` | Generate Codex role TOML files and managed config block |

## Updating forge-lib

All Forge modules include forge-lib as a git submodule at `lib/`. When forge-lib is updated:

```bash
cd your-module/
git -C lib pull
make -C lib build
git add lib
git commit -m "chore: update forge-lib submodule"
```

## Dependencies

- Rust (cargo) for building
- git for submodule management
