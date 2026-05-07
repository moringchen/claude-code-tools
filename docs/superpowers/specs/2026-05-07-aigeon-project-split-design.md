# Aigeon project split-out design

## Context

The current `claudeBoard` project lives under the larger `claudetools` repository at `tools/claudeBoard`. The goal is to split it into a standalone project under `/Users/moringchen/workspace/ai/tools/Aigeon`, rebrand the project and app name to `Aigeon`, and publish it as a new public GitHub repository named `Aigeon`.

The user wants this to be a practical extraction, not a product rewrite. The intended outcome is:
- create a standalone local project directory at `/Users/moringchen/workspace/ai/tools/Aigeon`
- preserve current functionality and structure as the starting point
- remove obvious `claudeBoard` / `Claude` project identity from package/app/repo surfaces
- initialize a new independent git repository
- create and push to a new public GitHub repository `Aigeon`

## Goals

- Separate the project physically and version-control-wise from `claudetools`
- Rebrand the app and project identity to `Aigeon`
- Keep the current codebase behavior as intact as possible during extraction
- Produce a clean first public repository that can build and test independently

## Non-goals

- Do not redesign the product during extraction
- Do not rewrite major subsystems just to remove historical implementation traces
- Do not alter the original `claudetools/tools/claudeBoard` project as part of the split
- Do not choose an alternate GitHub repository name unless `Aigeon` is unavailable and the user approves a fallback

## Recommended approach

Use a **copy, detach, rebrand, validate, publish** workflow:

1. Copy `tools/claudeBoard` to `/Users/moringchen/workspace/ai/tools/Aigeon`
2. Remove inherited git metadata so the new directory becomes an independent repository
3. Rebrand project identity surfaces to `Aigeon`
4. Run focused validation to confirm the split project still installs, tests, and starts cleanly
5. Initialize git, create a first clean commit, create the new public GitHub repository, add SSH remote, and push

This minimizes behavior risk while still producing a clean standalone public project.

## Existing structure to reuse

- Current project root structure under `tools/claudeBoard`
- Existing frontend build/test pipeline in `package.json`
- Existing Tauri app configuration in `src-tauri/tauri.conf.json`
- Existing Rust/Tauri source tree and test suites
- Existing docs/README structure as the base for rebranded documentation

## Detailed design

### 1. Local extraction

Create a new sibling directory:
- source: `/Users/moringchen/workspace/ai/tools/claudetools/tools/claudeBoard`
- destination: `/Users/moringchen/workspace/ai/tools/Aigeon`

The destination should be treated as a standalone project root. It must not keep the source repository’s `.git` directory or any assumptions that it still lives inside the `claudetools` mono-repo.

### 2. Repository independence

After copying:
- remove inherited git metadata in the copied directory
- initialize a fresh git repository in `Aigeon`
- ensure all future commits belong only to the new standalone repository

This guarantees the split project has its own clean public history rather than being a nested fragment of the original repository.

### 3. Rebrand identity surfaces

The split project should standardize visible identity to `Aigeon` across these layers:

#### Filesystem and repository
- local directory name: `Aigeon`
- GitHub repository name: `Aigeon`

#### Node / frontend metadata
- `package.json` package name
- any project name references in scripts or npm-facing metadata
- README titles and setup wording

#### Tauri / application metadata
- `src-tauri/tauri.conf.json`
  - `productName`
  - app window `title`
  - bundle/application `identifier`
- any app-facing labels or names used by the desktop shell

#### Rust / daemon / binary naming
- rename strongly bound `claude_board` / `claude_boardd` binary naming where it affects build, run, scripts, or logs
- update any references in shell commands, startup scripts, and tests so the split project is internally consistent

#### In-app copy and docs
- user-facing `claudeBoard` references in UI summaries, logs where appropriate, and documentation
- keep behavior-oriented terms intact; only rebrand project/app identity, not unrelated domain terms

### 4. Scope rule for renaming

This extraction should distinguish between:
- **identity names that must change**: project/app/repo/package/bundle/binary/documentation branding
- **internal domain concepts that may stay temporarily**: implementation details not user-facing and not required for immediate independence

That keeps the split manageable. The default is: if a name affects packaging, startup, repository identity, user-facing branding, or obvious public presentation, rename it now.

### 5. Validation before publishing

Before creating the GitHub repository, validate the split project in isolation:

- dependency install works in the new directory
- frontend tests pass
- targeted Rust / Tauri tests still pass
- app startup/build commands still resolve after renaming

The goal is not exhaustive product QA at this stage, but confidence that the standalone repo is internally coherent and publishable.

### 6. GitHub public repository creation

Once the local split is validated:
- create a new **public** GitHub repository named `Aigeon`
- configure the local repo to use SSH remote format
- expected remote form: `git@github.com:moringchen/Aigeon.git`
- push the initial standalone history to that repository

If GitHub creation fails because:
- `Aigeon` is unavailable, or
- `gh` is not authenticated / lacks permission,

stop and surface that explicitly instead of silently choosing a different repository identity.

## Execution phases

### Phase 1 — Copy and detach
- copy project directory to `/Users/moringchen/workspace/ai/tools/Aigeon`
- remove inherited `.git`
- verify destination root contents look complete

### Phase 2 — Rebrand critical identity
- update package/app/bundle/window/repo-facing names to `Aigeon`
- update daemon/binary/script references that would otherwise break startup or publishing
- update README / README_CN titles and setup text

### Phase 3 — Validate standalone project
- run install/build/test commands from the new directory
- fix any pathing or naming regressions caused by the extraction

### Phase 4 — Initialize and publish
- initialize new git repo
- create first standalone commit
- create public GitHub repository `Aigeon`
- add SSH remote and push

## Key files likely to change

- `/Users/moringchen/workspace/ai/tools/Aigeon/package.json`
- `/Users/moringchen/workspace/ai/tools/Aigeon/README.md`
- `/Users/moringchen/workspace/ai/tools/Aigeon/README_CN.md`
- `/Users/moringchen/workspace/ai/tools/Aigeon/src-tauri/tauri.conf.json`
- any Rust manifest/bin declarations under `/Users/moringchen/workspace/ai/tools/Aigeon/src-tauri/`
- startup scripts or shell commands that reference `claude_board` / `claude_boardd`
- any tests that lock old product/app names

## Error handling

- If destination `/Users/moringchen/workspace/ai/tools/Aigeon` already exists, stop and decide whether to replace, rename, or reuse it before modifying anything
- If GitHub repo `Aigeon` is unavailable, stop and ask for a fallback name
- If `gh` authentication is missing, stop after local repo preparation and ask the user to authenticate
- If renaming binaries/scripts breaks startup, fix those consistency issues before creating the public repo

## Testing strategy

### Local validation
- run frontend tests from the new `Aigeon` directory
- run targeted Rust/Tauri tests affected by renaming
- run build/start commands that prove the new project path and names are coherent

### Repository validation
- confirm the new repo has no inherited `.git` history from `claudetools`
- confirm `git remote -v` points only to the new SSH GitHub remote
- confirm the default branch and first push succeed

## Acceptance criteria

- `/Users/moringchen/workspace/ai/tools/Aigeon` exists as a standalone project
- the copied project no longer depends on the old repo’s git metadata
- public-facing project/app/package identity is renamed to `Aigeon`
- the standalone project passes focused validation in its new location
- a new public GitHub repository `Aigeon` exists and the local standalone repo is pushed to it via SSH
