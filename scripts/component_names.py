"""Lists the game's ECS component types from a `cargo enshwick types` dump.

Usage: python scripts/component_names.py gamedata/server-types.rs > gamedata/components.txt

A component is a type that derives, directly or through other types, from
`keen::ecs::Component`. The mod reads the list from `components.txt` in its
folder (`entity_components`). The list is derived from the game, so it stays
out of git like the dump.
"""

import re
import sys

STRUCT = re.compile(
    r"/// `(keen::[\w:]+)`: \d+ bytes[^\n]*\n"   # the game's name
    r"(?:[ \t]*#\[[^\n]*\]\n)*"                  # attributes
    r"[ \t]*pub struct (\w+) \{\n"               # the Rust name
    r"(?:[ \t]*///[^\n]*\n)?"                    # the base's doc line
    r"[ \t]*pub base: (\w+),"                    # the Rust name of the base
)


def components(text):
    base_of = {}   # Rust name -> Rust name of its base
    game_name = {}  # Rust name -> keen name
    for keen, rust, base in STRUCT.findall(text):
        base_of[rust] = base
        game_name[rust] = keen
    found = set()
    for rust in base_of:
        seen, cur = set(), rust
        while cur in base_of and cur not in seen:
            seen.add(cur)
            cur = base_of[cur]
            if cur == "Component":
                found.add(game_name[rust])
                break
    return sorted(found)


def main():
    with open(sys.argv[1], encoding="utf-8") as f:
        names = components(f.read())
    sys.stdout.write("\n".join(names) + "\n")
    print(f"{len(names)} component types", file=sys.stderr)


if __name__ == "__main__":
    main()
