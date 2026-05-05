#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import sys

REPO_ROOT = Path(__file__).resolve().parents[3]

CHECKS = {
    "tools/claudeBoard/README.md": [
        "macOS",
        "Windows 10",
        "global hooks",
        "当前无任务",
    ],
    "tools/claudeBoard/README_CN.md": [
        "macOS",
        "Windows 11",
        "全局 hooks",
        "当前无任务",
    ],
    "README.md": [
        "claudeBoard",
        "tools/claudeBoard/",
    ],
    "README_CN.md": [
        "claudeBoard",
        "tools/claudeBoard/",
    ],
}


def main() -> int:
    missing: list[str] = []

    for relative_path, fragments in CHECKS.items():
        file_path = REPO_ROOT / relative_path
        try:
            content = file_path.read_text(encoding="utf-8")
        except FileNotFoundError:
            missing.append(f"{relative_path}: file not found")
            continue

        for fragment in fragments:
            if fragment not in content:
                missing.append(f"{relative_path}: missing '{fragment}'")

    if missing:
        for item in missing:
            print(item)
        return 1

    print("Documentation checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
