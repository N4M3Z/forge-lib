# GeminiAdapter

Fallback installation for Gemini CLI.

## Detection

```bash
gemini --version
```

## Install

```bash
bash Modules/forge-lib/scripts/gemini/install.sh
```

Scope options: `--scope workspace` (default), `--scope user`.

```bash
bash Modules/forge-lib/scripts/gemini/install.sh --scope user
bash Modules/forge-lib/scripts/gemini/install.sh --vault-skills Vaults/Personal/Orchestration/Skills
```

## What Gets Installed

| Content | Destination | Format |
|---------|-------------|--------|
| Skills | `.gemini/skills/<kebab-name>/` | Directory (SKILL.md + companions), SKILL.yaml excluded |

## What's NOT Installed (Requires Rust)

| Content | Why |
|---------|-----|
| Agents | Gemini agent frontmatter requires tool name remapping (Read → read_file, Bash → run_shell_command) and YAML list formatting. The Rust `install-agents` binary handles this. |
| Rules | Gemini has no rules directory. Project instructions go in `GEMINI.md` with `@file.md` import syntax. Run `gemini init` to generate. |

## Provider Notes

- Skill names are kebab-cased (BuildSystem → build-system)
- Skills follow the [Agent Skills](https://agentskills.io) standard
- Tool names differ from Claude: `Read` → `read_file`, `Write` → `write_file`, `Edit` → `replace`, `Grep` → `grep_search`, `Bash` → `run_shell_command`, `WebSearch` → `google_web_search`
- Project context loaded via `GEMINI.md` hierarchy: global (`~/.gemini/GEMINI.md`) → workspace → just-in-time
- Settings in `.gemini/settings.json`

## Verification

```bash
ls .gemini/skills/ | wc -l
gemini skills list 2>/dev/null || echo "gemini CLI not available"
```
