#!/usr/bin/env bash
set -euo pipefail

SKILL_DIR="$HOME/.claude/skills/karpahere"
SKILL_PATH="$SKILL_DIR/SKILL.md"
GUIDELINES_PATH="$SKILL_DIR/karpathy-guidelines.md"

mkdir -p "$SKILL_DIR"

cat >"$SKILL_PATH" <<'EOF'
---
name: karpahere
description: Add the Karpathy guidelines block to the current project's .claude/CLAUDE.md if it is not already present.
tools:
  - Read
  - Write
  - Edit
---

# karpahere

When invoked, copy the full contents of:

`~/.claude/skills/karpahere/karpathy-guidelines.md`

into the current project's `.claude/CLAUDE.md`.

## Required behavior

1. Treat the current Claude Code working directory as the target project root.
2. The target file is `.claude/CLAUDE.md` under that root.
3. If the `.claude` directory does not exist, create it.
4. If `.claude/CLAUDE.md` does not exist, create it.
5. Before writing, check whether the target file already contains both of these markers:
   - `<!-- karpahere:start -->`
   - `<!-- karpahere:end -->`
6. If that marked block already exists, do not change the file. Report that insertion was skipped.
7. Otherwise append a new block to the end of the file in this exact shape:

```md
<!-- karpahere:start -->
[paste the source file content verbatim here]
<!-- karpahere:end -->
```

## Constraints

- Preserve the source file content verbatim.
- Do not modify any existing content outside the inserted block.
- If the target file is non-empty, separate the appended block with a blank line.
- Report the final target path and whether the operation changed the file or was skipped.
EOF

cat >"$GUIDELINES_PATH" <<'EOF'
# Karpathy Guidelines

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.
EOF

printf 'Installed /karpahere into %s\n' "$SKILL_DIR"
printf 'Installed skill file to %s\n' "$SKILL_PATH"
printf 'Installed vendored guidelines to %s\n' "$GUIDELINES_PATH"
printf '%s\n' 'Start a new Claude Code session, then use /help to confirm /karpahere is available.'
