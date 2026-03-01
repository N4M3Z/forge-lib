# Install

> **For AI agents**: This guide covers installation of forge-lib. Follow the steps for your platform.

## Requirements

1. **Git** with submodule support
2. **Rust toolchain** — `cargo` and `rustc` for building binaries
3. **make** (POSIX) — macOS/Linux build orchestration

## Build

### As a submodule (most common)

Forge modules consume forge-lib as a git submodule at `lib/`:

```bash
cd your-module/
git submodule add https://github.com/N4M3Z/forge-lib.git lib
make -C lib build
```

Binaries are symlinked into `lib/bin/` for consumption by the parent module's Makefile.

### Standalone

Clone and build directly:

```bash
git clone https://github.com/N4M3Z/forge-lib.git
cd forge-lib
make build
```

This compiles all Rust binaries in release mode and symlinks them to `bin/`:

```bash
make check    # verify all binaries are present
```

### Windows PowerShell fallback

Use this if `make build` fails due to POSIX shell syntax:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
cargo build --release

# Verify binaries
.\target\release\strip-front.exe --version
.\target\release\install-agents.exe --version
.\target\release\install-skills.exe --version
.\target\release\validate-module.exe --version
.\target\release\yaml.exe --version
```

On Windows, reference binaries at `target\release\` directly — the `bin/` symlink step requires a POSIX shell.

## Platforms

- **macOS / Linux**: `make build` handles everything. Requires `cargo` on PATH.
- **Windows**: Prefer WSL or Git Bash for `make` targets. If staying in PowerShell, use the fallback commands above. Rust builds natively on Windows — `cargo build --release` works without modification.

## Configuration

`defaults.yaml` ships the skill roster keyed by provider:

```yaml
skills:
    claude:
        BuildSystem:
    gemini:
        BuildSystem:
```

Create `config.yaml` (gitignored) to override — same structure, only the fields you want to change.

## Updating

### As a submodule consumer

```bash
git -C lib pull
make -C lib build
git add lib
git commit -m "chore: update forge-lib submodule"
```

### Standalone

```bash
git pull
make clean
make build
```
