# karpahere

Install a reusable `/karpahere` slash into your local Claude Code skills directory.

## What it does

`/karpahere` appends a single marked Karpathy guidelines block to the current project's `.claude/CLAUDE.md`.

- Installs to `~/.claude/skills/karpahere/`
- Vendors the guidelines payload locally
- Skips insertion if the existing file already contains both marker comments
- Does not depend on a versioned Claude plugin cache path

## Karpathy-inspired principles

The vendored guidelines are based on the coding habits Andrej Karpathy has repeatedly pointed out as failure modes for LLMs: making silent assumptions, overengineering simple work, editing unrelated code, and acting without crisp success criteria.

The installed block pushes Claude Code toward four habits:

- Think before coding
- Prefer the simplest solution that solves the actual request
- Make surgical, task-scoped changes
- Work against verifiable success criteria instead of vague intent

## Benefits

Using `/karpahere` helps make project guidance more consistent across repositories and usually leads to:

- fewer unnecessary edits in diffs
- fewer overcomplicated implementations
- more clarifying questions before code is written
- cleaner, easier-to-review task outputs
- better alignment between implementation and the original request

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/karpahere/install.sh | bash
```

## Installed files

The installer writes:

- `~/.claude/skills/karpahere/SKILL.md`
- `~/.claude/skills/karpahere/karpathy-guidelines.md`

## How it works

When you invoke `/karpahere`, the installed skill:

1. Targets the current project's `.claude/CLAUDE.md`
2. Creates `.claude/` and `CLAUDE.md` if missing
3. Checks for these markers before writing:
   - `<!-- karpahere:start -->`
   - `<!-- karpahere:end -->`
4. Appends the vendored Karpathy guidelines only when that marked block is absent

## Verify after install

1. Start a new Claude Code session
2. Run `/help`
3. Confirm `/karpahere` appears
4. Invoke `/karpahere` in a test project and confirm `.claude/CLAUDE.md` gets one marked block

## Upstream reference docs

This tool also keeps copies of the upstream Markdown files under `upstream/` so the source material travels with the maintained tool.

The vendored snapshot was imported from the installed `andrej-karpathy-skills@karpathy-skills` plugin content observed during migration, including the `1.0.0` cache snapshot used as the source at import time.

## Maintenance note

`SKILL.md` and `karpathy-guidelines.md` are the checked-in source of truth. `install.sh` embeds matching copies so the documented `curl -fsSL ... | bash` flow stays self-contained, and `test_karpahere.zsh` guards against drift.

## License

MIT License - same as the main project.
