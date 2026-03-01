---
name: BuildSystem
description: "Install forge skills and agents without the Rust toolchain — provider-specific fallback adapters. USE WHEN rust not installed, cargo fails, can't compile, install without rust, manual install, shell bootstrap, provider setup, build system."
---

# BuildSystem

Fallback installation for the forge ecosystem when the Rust toolchain is unavailable. Each adapter covers a single AI coding tool provider with provider-accurate instructions.

## When This Fires

- `cargo build` fails or `rustc` is missing
- Agent can only run shell scripts (no compilation)
- First-time setup on a machine without Rust
- CI/CD environment without the Rust toolchain

## Provider Adapters

| Adapter | Provider | Trigger | Content |
|---------|----------|---------|---------|
| ClaudeAdapter | Claude Code | "claude", "claude code" | @ClaudeAdapter.md |
| GeminiAdapter | Gemini CLI | "gemini", "gemini cli" | @GeminiAdapter.md |
| CodexAdapter | Codex CLI | "codex", "openai codex" | @CodexAdapter.md |

## Content Mapping

Each provider has different conventions for loading instructions:

| Content | Claude Code | Gemini CLI | Codex CLI |
|---------|------------|------------|-----------|
| Skills | `.claude/skills/` (directories) | `.gemini/skills/` (directories) | `.codex/skills/` (directories) |
| Agents | `.claude/agents/*.md` | `.gemini/agents/*.md` (kebab-case) | `.codex/agents/*.toml` |
| Rules | `.claude/rules/*.md` (auto-loaded) | `GEMINI.md` (`@` imports) | `AGENTS.md` (single file) |
| Config | `.claude/settings.json` | `settings.json` | `.codex/config.toml` |
| Naming | PascalCase | kebab-case | PascalCase |

## What's Missing Without Rust

These features require compiled binaries:

| Feature | Binary | Impact |
|---------|--------|--------|
| Hook dispatch | `dispatch` | No event routing to modules |
| TLP access control | `safe-read`, `tlp-guard` | No file classification enforcement |
| Reflection enforcement | `insight`, `reflect` | No session insight capture |
| DCI | `dispatch skill-load` | No dynamic context injection |
| Template merging | `build-templates` | No journal templates |
| Module validation | `validate-module` | No convention test suite |

Skills, agents, and rules load and work. Runtime hooks and enforcement do not.

## Full Install

```bash
make deps && make install
```
