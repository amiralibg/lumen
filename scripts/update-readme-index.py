#!/usr/bin/env python3
"""Rewrite README.md's component index and counts from site/assets/content.js."""
import pathlib, re, subprocess
root = pathlib.Path(__file__).resolve().parent.parent
idx = subprocess.run(["node", "scripts/component-index.mjs"], cwd=root, capture_output=True, text=True, check=True).stdout.strip()
n = subprocess.run(["node", "scripts/component-index.mjs", "--count"], cwd=root, capture_output=True, text=True, check=True).stdout.strip()
p = root / "README.md"
s = p.read_text()
a = s.index("## Components\n\n") + len("## Components\n\n")
b = s.index("\n\nPlus building blocks")
s = s[:a] + idx + s[b:]
s = re.sub(r"\*\*\d+ components\*\*", f"**{n} components**", s)
p.write_text(s)
print(f"README: {n} components")
