"""Delete all non-.svelte files and empty directories in svelte@5.48.0-tests/."""
import os
from pathlib import Path

root = Path(__file__).parent / "svelte@5.48.0-tests"
deleted = 0

for f in root.rglob("*"):
    if f.is_file() and f.suffix != ".svelte":
        f.unlink()
        deleted += 1

# Remove empty directories
for d in sorted(root.rglob("*"), reverse=True):
    if d.is_dir() and not any(d.iterdir()):
        d.rmdir()

print(f"Deleted {deleted} files")
