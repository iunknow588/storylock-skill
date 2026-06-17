#!/usr/bin/env python3
"""
Check repository text files for BOM and non-LF line endings.

Usage:
  python scripts/text/check_line_endings.py --root .
  python scripts/text/check_line_endings.py --root skill/docs --fail-on-crlf
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

SKIP_DIRS = {
    ".git",
    ".github",
    "__pycache__",
    ".pytest_cache",
    ".ruff_cache",
    ".vscode",
    "node_modules",
    "venv",
    ".venv",
    "runtime",
    "dist",
    "build",
    "coverage",
}
TEXT_EXT = {
    ".c",
    ".cc",
    ".cmd",
    ".cpp",
    ".css",
    ".csv",
    ".env",
    ".h",
    ".hpp",
    ".html",
    ".ini",
    ".java",
    ".js",
    ".json",
    ".jsx",
    ".md",
    ".mjs",
    ".ps1",
    ".py",
    ".rs",
    ".sh",
    ".sql",
    ".svg",
    ".toml",
    ".ts",
    ".tsx",
    ".txt",
    ".xml",
    ".yaml",
    ".yml",
}
TEXT_FILENAMES = {
    "Dockerfile",
    "LICENSE",
    "Makefile",
    "README",
    "README.md",
    "SKILL.md",
    ".editorconfig",
    ".gitattributes",
    ".gitignore",
}

DEFAULT_ROOT = Path(__file__).resolve().parents[2]


def is_text_file(path: Path) -> bool:
    return path.suffix.lower() in TEXT_EXT or path.name in TEXT_FILENAMES


def should_skip(path: Path) -> bool:
    for part in path.parts:
        if part in SKIP_DIRS:
            return True
    return not is_text_file(path)


def is_binary(data: bytes) -> bool:
    return b"\x00" in data[:8192]


def inspect_file(path: Path) -> tuple[bool, bool, bool]:
    data = path.read_bytes()
    has_bom = data.startswith(b"\xef\xbb\xbf")
    has_crlf = b"\r\n" in data
    has_lf = b"\n" in data
    return has_bom, has_crlf, has_lf


def main() -> None:
    parser = argparse.ArgumentParser(description="Check text files for BOM and CRLF line endings")
    parser.add_argument("--root", default=str(DEFAULT_ROOT), help="Root path to scan")
    parser.add_argument("--fail-on-bom", action="store_true", help="Exit non-zero if any BOM is found")
    parser.add_argument("--fail-on-crlf", action="store_true", help="Exit non-zero if any CRLF is found")
    parser.add_argument("--only-changed", action="store_true", help="Only print files with BOM or CRLF")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    if not root.exists():
        print(f"Root path not found: {root}")
        sys.exit(2)

    print(f"Checking text files under {root}")

    total = 0
    flagged = 0
    bom_count = 0
    crlf_count = 0

    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fname in filenames:
            path = Path(dirpath) / fname
            if should_skip(path):
                continue
            data = path.read_bytes()
            if is_binary(data):
                continue

            total += 1
            has_bom, has_crlf, has_lf = inspect_file(path)
            if has_bom:
                bom_count += 1
            if has_crlf:
                crlf_count += 1

            needs_attention = has_bom or has_crlf
            if needs_attention:
                flagged += 1

            if args.only_changed and not needs_attention:
                continue

            rel = path.relative_to(root)
            print(f"{rel}: BOM={has_bom}, CRLF={has_crlf}, LF={has_lf}")

    print("---")
    print(f"Scanned: {total}")
    print(f"Flagged: {flagged}")
    print(f"BOM files: {bom_count}")
    print(f"CRLF files: {crlf_count}")

    if (args.fail_on_bom and bom_count > 0) or (args.fail_on_crlf and crlf_count > 0):
        sys.exit(1)


if __name__ == "__main__":
    main()
