#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR=${0:A:h}
REPO_ROOT=${SCRIPT_DIR:h:h}
INSTALLER="$SCRIPT_DIR/install.sh"
SKILL_SOURCE="$SCRIPT_DIR/SKILL.md"
GUIDELINES_SOURCE="$SCRIPT_DIR/karpathy-guidelines.md"
TOOL_README_EN="$SCRIPT_DIR/README.md"
TOOL_README_CN="$SCRIPT_DIR/README_CN.md"
ROOT_README_EN="$REPO_ROOT/README.md"
ROOT_README_CN="$REPO_ROOT/README_CN.md"
UPSTREAM_DIR="$SCRIPT_DIR/upstream"
UPSTREAM_SKILL="$UPSTREAM_DIR/skills/karpathy-guidelines/SKILL.md"
EXPECTED_INSTALL_SNIPPET='curl -fsSL https://raw.githubusercontent.com/moringchen/claude-code-tools/main/tools/karpahere/install.sh | bash'
EXPECTED_SKILL_DIR_SUFFIX='/.claude/skills/karpahere'

authoritative_files=(
  "$INSTALLER"
  "$SKILL_SOURCE"
  "$GUIDELINES_SOURCE"
  "$TOOL_README_EN"
  "$TOOL_README_CN"
  "$UPSTREAM_DIR/README.md"
  "$UPSTREAM_DIR/README.zh.md"
  "$UPSTREAM_DIR/EXAMPLES.md"
  "$UPSTREAM_DIR/CURSOR.md"
  "$UPSTREAM_DIR/CLAUDE.md"
  "$UPSTREAM_SKILL"
)

assert_eq() {
  local expected="$1"
  local actual="$2"
  local message="$3"

  if [[ "$actual" != "$expected" ]]; then
    print -u2 -- "$message"
    print -u2 -- "expected: $expected"
    print -u2 -- "actual:   $actual"
    exit 1
  fi
}

assert_true() {
  local condition="$1"
  local message="$2"

  if ! eval "$condition"; then
    print -u2 -- "$message"
    exit 1
  fi
}

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local message="$3"

  if [[ "$haystack" != *"$needle"* ]]; then
    print -u2 -- "$message"
    print -u2 -- "missing: $needle"
    exit 1
  fi
}

assert_not_contains() {
  local haystack="$1"
  local needle="$2"
  local message="$3"

  if [[ "$haystack" == *"$needle"* ]]; then
    print -u2 -- "$message"
    print -u2 -- "unexpected: $needle"
    exit 1
  fi
}

run_capture() {
  local stdout_file stderr_file
  stdout_file=$(mktemp)
  stderr_file=$(mktemp)

  set +e
  "$@" >"$stdout_file" 2>"$stderr_file"
  RUN_STATUS=$?
  set -e

  RUN_STDOUT=$(<"$stdout_file")
  RUN_STDERR=$(<"$stderr_file")
  rm -f "$stdout_file" "$stderr_file"
}

typeset -g RUN_STATUS=0
typeset -g RUN_STDOUT=''
typeset -g RUN_STDERR=''

for file_path in $authoritative_files; do
  assert_true "[[ -f '$file_path' ]]" "required file should exist: $file_path"
done

assert_true '[[ -x "$INSTALLER" ]]' 'install.sh should be executable'

README_TOOL_EN_CONTENTS=$(<"$TOOL_README_EN")
README_TOOL_CN_CONTENTS=$(<"$TOOL_README_CN")
README_ROOT_EN_CONTENTS=$(<"$ROOT_README_EN")
README_ROOT_CN_CONTENTS=$(<"$ROOT_README_CN")
SKILL_SOURCE_CONTENTS=$(<"$SKILL_SOURCE")
GUIDELINES_SOURCE_CONTENTS=$(<"$GUIDELINES_SOURCE")
UPSTREAM_SKILL_CONTENTS=$(<"$UPSTREAM_SKILL")

assert_contains "$README_TOOL_EN_CONTENTS" "$EXPECTED_INSTALL_SNIPPET" 'tool README.md should include the curl-based install snippet'
assert_contains "$README_TOOL_CN_CONTENTS" "$EXPECTED_INSTALL_SNIPPET" 'tool README_CN.md should include the curl-based install snippet'
assert_contains "$README_ROOT_EN_CONTENTS" "$EXPECTED_INSTALL_SNIPPET" 'root README.md should include the karpahere install snippet'
assert_contains "$README_ROOT_CN_CONTENTS" "$EXPECTED_INSTALL_SNIPPET" 'root README_CN.md should include the karpahere install snippet'

assert_contains "$README_TOOL_EN_CONTENTS" '/help' 'tool README.md should describe /help verification'
assert_contains "$README_TOOL_CN_CONTENTS" '/help' 'tool README_CN.md should describe /help verification'
assert_contains "$README_ROOT_EN_CONTENTS" 'karpahere' 'root README.md should list karpahere'
assert_contains "$README_ROOT_CN_CONTENTS" 'karpahere' 'root README_CN.md should list karpahere'

assert_contains "$SKILL_SOURCE_CONTENTS" 'name: karpahere' 'SKILL.md should declare the karpahere skill name'
assert_contains "$SKILL_SOURCE_CONTENTS" '<!-- karpahere:start -->' 'SKILL.md should mention the start marker'
assert_contains "$SKILL_SOURCE_CONTENTS" '<!-- karpahere:end -->' 'SKILL.md should mention the end marker'
assert_contains "$SKILL_SOURCE_CONTENTS" 'karpathy-guidelines.md' 'SKILL.md should reference the vendored guidelines file'
assert_not_contains "$SKILL_SOURCE_CONTENTS" '.claude/plugins/cache' 'SKILL.md should not depend on the Claude plugin cache'

assert_contains "$GUIDELINES_SOURCE_CONTENTS" '# Karpathy Guidelines' 'karpathy-guidelines.md should contain the vendored guidelines heading'
assert_contains "$UPSTREAM_SKILL_CONTENTS" '# Karpathy Guidelines' 'upstream skill copy should preserve the original heading'

sandbox_home=$(mktemp -d)
run_capture env HOME="$sandbox_home" bash "$INSTALLER"
assert_eq '0' "$RUN_STATUS" 'install.sh should succeed in a sandbox HOME'
assert_eq '' "$RUN_STDERR" 'install.sh should not write stderr on success'

installed_skill_dir="$sandbox_home/.claude/skills/karpahere"
installed_skill="$installed_skill_dir/SKILL.md"
installed_guidelines="$installed_skill_dir/karpathy-guidelines.md"

assert_true '[[ -d "$installed_skill_dir" ]]' 'install.sh should create the karpahere skill directory'
assert_true '[[ -f "$installed_skill" ]]' 'install.sh should install SKILL.md'
assert_true '[[ -f "$installed_guidelines" ]]' 'install.sh should install karpathy-guidelines.md'

installed_skill_contents=$(<"$installed_skill")
installed_guidelines_contents=$(<"$installed_guidelines")

assert_eq "$SKILL_SOURCE_CONTENTS" "$installed_skill_contents" 'installed SKILL.md should match the repo source file'
assert_eq "$GUIDELINES_SOURCE_CONTENTS" "$installed_guidelines_contents" 'installed karpathy-guidelines.md should match the repo source file'
assert_not_contains "$installed_skill_contents" '.claude/plugins/cache' 'installed SKILL.md should not mention the Claude plugin cache'
assert_contains "$installed_skill_contents" 'karpathy-guidelines.md' 'installed SKILL.md should point at the installed vendored guidelines file'
assert_contains "$RUN_STDOUT" "$installed_skill_dir" 'install.sh should report the installed skill directory'
assert_contains "$RUN_STDOUT" '/karpahere' 'install.sh should mention /karpahere in its success output'

run_capture env HOME="$sandbox_home" bash "$INSTALLER"
assert_eq '0' "$RUN_STATUS" 'install.sh should be idempotent on a second run'
assert_eq '' "$RUN_STDERR" 'install.sh should not write stderr on a second successful run'
installed_skill_contents_after_second_run=$(<"$installed_skill")
installed_guidelines_contents_after_second_run=$(<"$installed_guidelines")
assert_eq "$SKILL_SOURCE_CONTENTS" "$installed_skill_contents_after_second_run" 'installed SKILL.md should remain unchanged after a second run'
assert_eq "$GUIDELINES_SOURCE_CONTENTS" "$installed_guidelines_contents_after_second_run" 'installed guidelines should remain unchanged after a second run'

piped_home=$(mktemp -d)
run_capture env HOME="$piped_home" bash -lc 'cat "$1" | bash' bash "$INSTALLER"
assert_eq '0' "$RUN_STATUS" 'install.sh should support curl-style piping through bash'
assert_true '[[ -f "$piped_home/.claude/skills/karpahere/SKILL.md" ]]' 'piped install should create the installed skill file'

rm -rf "$sandbox_home" "$piped_home"

print 'ok'
