#!/usr/bin/env uv run
# /// script
# dependencies = [
#   "python-frontmatter",
# ]
# ///

import frontmatter
import glob

for path in glob.glob("rules/*/rules.md"):
    # Split by path separator and extract the slug
    slug = path.split("/")[1]
    post = frontmatter.load(path)
    
    print(f"\n{slug}:")
    for item in post.get("when", []):
        print(f"- {item}")
