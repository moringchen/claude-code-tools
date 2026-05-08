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
