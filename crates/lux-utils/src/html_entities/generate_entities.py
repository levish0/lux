"""
Generate html_entities_data.rs from svelte@5.50.0-entities.js.

Usage:
    python generate_entities.py

Reads from: svelte@5.50.0-entities.js (same directory)
Writes to:  html_entities_data.rs (same directory)
"""

import re
import os

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
ENTITIES_JS = os.path.join(SCRIPT_DIR, "svelte@5.50.0-entities.js")
OUTPUT_FILE = os.path.join(SCRIPT_DIR, "html_entities_data.rs")


def main():
    with open(ENTITIES_JS, "r") as f:
        content = f.read()

    entries = []
    for m in re.finditer(r"(?:'([^']+)'|([a-zA-Z]+)):\s*(\d+)", content):
        name = m.group(1) or m.group(2)
        code = int(m.group(3))
        entries.append((name, code))

    print(f"Parsed {len(entries)} entities from {ENTITIES_JS}")

    lines = [f'    "{name}" => {code},' for name, code in entries]
    entries_str = "\n".join(lines)

    output = f"""use phf::phf_map;

#[rustfmt::skip]
pub static ENTITIES: phf::Map<&'static str, u32> = phf_map! {{
{entries_str}
}};
"""

    with open(OUTPUT_FILE, "w") as f:
        f.write(output)

    print(f"Generated {OUTPUT_FILE} ({len(entries)} entries)")


if __name__ == "__main__":
    main()
