# ClaudeAdapter

Fallback installation for Claude Code.

## Detection

```bash
claude --version
```

## Install

```bash
bash Modules/forge-lib/scripts/claude/install.sh
```

Scope options: `--scope workspace` (default), `--scope user`, `--scope project`.

```bash
bash Modules/forge-lib/scripts/claude/install.sh --scope user
bash Modules/forge-lib/scripts/claude/install.sh --vault-skills Vaults/Personal/Orchestration/Skills
```

Or via Make:

```bash
make install-shell
```

## What Gets Installed

| Content | Destination | Format |
|---------|-------------|--------|
| Skills | `.claude/skills/<Name>/` | Directory (SKILL.md + companions), SKILL.yaml excluded |
| Agents | `.claude/agents/<Name>.md` | Markdown with YAML frontmatter |
| Rules | `.claude/rules/<Name>.md` | Individual markdown files, auto-loaded per session |

## Provider Notes

- Names are PascalCase
- Rules are Claude-specific — other providers use context files instead
- Settings live in `.claude/settings.json` (hooks, permissions, env)

## Verification

```bash
ls .claude/skills/ | wc -l
ls .claude/agents/ | wc -l
ls .claude/rules/ | wc -l
```
