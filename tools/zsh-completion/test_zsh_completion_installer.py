from __future__ import annotations

import os
import pathlib
import pty
import select
import shlex
import subprocess
import tempfile
import time
import unittest

SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
SETUP_SCRIPT = SCRIPT_DIR / "setup_claude_completion.sh"
HELP_FIXTURE = SCRIPT_DIR / "testdata" / "claude-help.txt"


class ZshCompletionInstallerTests(unittest.TestCase):
    maxDiff = None

    def run_installer(self, home: pathlib.Path) -> subprocess.CompletedProcess[str]:
        fake_bin = home / "bin"
        fake_bin.mkdir(parents=True, exist_ok=True)
        claude_path = fake_bin / "claude"
        fixture = shlex.quote(HELP_FIXTURE.read_text(encoding="utf-8"))
        claude_path.write_text(
            "#!/bin/sh\n"
            "if [ \"$1\" = \"--help\" ]; then\n"
            f"  printf '%s\\n' {fixture}\n"
            "  exit 0\n"
            "fi\n"
            "echo \"unexpected args: $*\" >&2\n"
            "exit 1\n",
            encoding="utf-8",
        )
        claude_path.chmod(0o755)

        env = os.environ.copy()
        env["HOME"] = str(home)
        env["PATH"] = f"{fake_bin}:{env['PATH']}"

        return subprocess.run(
            ["bash", str(SETUP_SCRIPT)],
            cwd=SCRIPT_DIR,
            env=env,
            text=True,
            capture_output=True,
        )

    def complete_and_capture(self, home: pathlib.Path, seed: str) -> tuple[str, int]:
        zshrc = home / ".zshrc"
        zshrc.write_text(
            zshrc.read_text(encoding="utf-8")
            + "\n"
            + "zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}' 'r:|=*' 'l:|=* r:|=*'\n"
            + "_capture_buffer_state() {\n"
            + "  print -r -- \"$BUFFER\" > \"$HOME/.buffer_state\"\n"
            + "  print -r -- \"$CURSOR\" > \"$HOME/.cursor_state\"\n"
            + "  zle accept-line\n"
            + "}\n"
            + "zle -N _capture_buffer_state\n"
            + "bindkey '^X^B' _capture_buffer_state\n",
            encoding="utf-8",
        )

        env = os.environ.copy()
        env["HOME"] = str(home)

        master, slave = pty.openpty()
        process = subprocess.Popen(
            ["/bin/zsh", "-i"],
            stdin=slave,
            stdout=slave,
            stderr=slave,
            env=env,
            close_fds=True,
        )
        os.close(slave)

        def drain(timeout: float = 0.5) -> str:
            end = time.time() + timeout
            output = b""
            while time.time() < end:
                ready, _, _ = select.select([master], [], [], 0.05)
                if not ready:
                    continue
                chunk = os.read(master, 4096)
                if not chunk:
                    break
                output += chunk
            return output.decode("utf-8", errors="replace")

        try:
            drain(1.0)
            os.write(master, seed.encode("utf-8") + b"\t\x18\x02")
            time.sleep(1.0)
            drain(1.0)
            buffer_value = (home / ".buffer_state").read_text(encoding="utf-8").rstrip("\n")
            cursor_value = int((home / ".cursor_state").read_text(encoding="utf-8").strip())
            return buffer_value, cursor_value
        finally:
            os.write(master, b"exit\n")
            time.sleep(0.2)
            drain(0.5)
            process.wait(timeout=5)

    def test_installer_generates_completion_from_help_fixture(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = pathlib.Path(tmp)
            result = self.run_installer(home)
            self.assertEqual(result.returncode, 0, result.stderr)

            completion = (home / ".zsh" / "completions" / "_claude").read_text(encoding="utf-8")
            self.assertIn("--bare", completion)
            self.assertIn("--name", completion)
            self.assertIn("xhigh", completion)
            self.assertIn("acceptEdits auto bypassPermissions default dontAsk plan", completion)
            self.assertNotIn("acceptEdits bypassPermissions default dontAsk plan auto", completion)
            self.assertIn("'plugin:Manage Claude Code plugins'", completion)
            self.assertIn("'plugins:Manage Claude Code plugins'", completion)
            self.assertNotIn("'plugin|plugins:Manage Claude Code plugins'", completion)
            self.assertIn(r"--system-prompt\[-file\]", completion)
            self.assertIn(r"\[DEPRECATED. Use --debug instead\]", completion)
            self.assertIn("_arguments -M 'm:{a-z}={A-Z}' -C", completion)

    def test_generated_completion_is_valid_zsh_syntax(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = pathlib.Path(tmp)
            result = self.run_installer(home)
            self.assertEqual(result.returncode, 0, result.stderr)

            syntax = subprocess.run(
                ["zsh", "-n", str(home / ".zsh" / "completions" / "_claude")],
                text=True,
                capture_output=True,
            )
            self.assertEqual(syntax.returncode, 0, syntax.stderr)

    def test_generated_completion_loads_without_bad_pattern(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = pathlib.Path(tmp)
            result = self.run_installer(home)
            self.assertEqual(result.returncode, 0, result.stderr)

            runtime = subprocess.run(
                [
                    "zsh",
                    "-fc",
                    "source \"$HOME/.zsh/completions/_claude\"; _arguments(){ :; }; _describe(){ :; }; _claude",
                ],
                env={**os.environ, "HOME": str(home)},
                text=True,
                capture_output=True,
            )
            self.assertEqual(runtime.returncode, 0, runtime.stderr)

    def test_installer_writes_claude_scoped_zshrc_block(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = pathlib.Path(tmp)
            zshrc = home / ".zshrc"
            zshrc.write_text(
                "# OPENSPEC:START\n"
                "fpath=(\"/Users/example/.zsh/completions\" $fpath)\n"
                "zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}' 'r:|=*' 'l:|=* r:|=*'\n"
                "autoload -Uz compinit\n"
                "compinit\n"
                "# OPENSPEC:END\n",
                encoding="utf-8",
            )

            result = self.run_installer(home)
            self.assertEqual(result.returncode, 0, result.stderr)

            updated = zshrc.read_text(encoding="utf-8")
            self.assertIn("# >>> Claude CLI zsh completion >>>", updated)
            self.assertIn("zstyle ':completion:*:*:claude:*' matcher-list 'm:{a-z}={A-Z}'", updated)
            self.assertIn('fpath=("$HOME/.zsh/completions" $fpath)', updated)
            self.assertIn("# OPENSPEC:START", updated)
            self.assertIn("zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}' 'r:|=*' 'l:|=* r:|=*'", updated)
            self.assertEqual(updated.count("# >>> Claude CLI zsh completion >>>"), 1)

    def test_installer_prefers_permission_mode_under_aggressive_global_matchers(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = pathlib.Path(tmp)
            result = self.run_installer(home)
            self.assertEqual(result.returncode, 0, result.stderr)

            buffer_value, cursor_value = self.complete_and_capture(home, "claude --permi")

            self.assertEqual(buffer_value, "claude --permission-mode ")
            self.assertEqual(cursor_value, len(buffer_value))

    def test_installer_moves_cursor_to_end_after_unique_option_completion(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = pathlib.Path(tmp)
            result = self.run_installer(home)
            self.assertEqual(result.returncode, 0, result.stderr)

            buffer_value, cursor_value = self.complete_and_capture(home, "claude --nam")

            self.assertEqual(buffer_value, "claude --name ")
            self.assertEqual(cursor_value, len(buffer_value))

    def test_installer_fails_when_claude_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            home = pathlib.Path(tmp)
            env = os.environ.copy()
            env["HOME"] = str(home)
            env["PATH"] = "/usr/bin:/bin"

            result = subprocess.run(
                ["bash", str(SETUP_SCRIPT)],
                cwd=SCRIPT_DIR,
                env=env,
                text=True,
                capture_output=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("claude was not found in PATH", result.stderr)


if __name__ == "__main__":
    unittest.main()
