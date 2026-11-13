"""Search an existing `cargo enshwick types` Rust catalogue, without a game.

Only reads generated text. Does not open executables, archives, saves or memory.
Output contains derived game metadata: redirect it into git-ignored gamedata/.
"""

import argparse
import hashlib
import json
from pathlib import Path
import re


HEADER = "//! The game's own types, written by `cargo enshwick types`"
RECORD = re.compile(
    r"^[ \t]*/// `(?P<name>keen::[^`]+)`(?P<description>[^\n]*)",
    re.MULTILINE,
)
FIELD = re.compile(r"/// `([^`]+)`, of `([^`]+)`, at (0x[0-9a-f]+)\.")


def inspect(text, queries=(), limit=20):
    if not text.startswith(HEADER):
        raise ValueError("expected an existing cargo enshwick types Rust catalogue")
    # Type documentation uses a qualified keen:: name. Field documentation
    # starts with a field name, so it cannot start another record.
    matches = list(RECORD.finditer(text))
    if not matches:
        raise ValueError("no type records found; the generator format may have changed")
    records = []
    counts = {"struct": 0, "enum": 0, "alias": 0}
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
        block = text[match.end():end]
        description = match["description"].strip()
        if ": an enum stored as" in description:
            kind = "enum"
        elif ", another name for" in description:
            kind = "alias"
        elif re.match(r": \d+ bytes, aligned to \d+\.", description):
            kind = "struct"
        else:
            raise ValueError(f"unrecognized type documentation: {match['name']}")
        counts[kind] += 1
        record = {"name": match["name"], "kind": kind, "description": description}
        record["fields"] = [
            {"name": name, "type": ty, "offset": offset}
            for name, ty, offset in FIELD.findall(block)
        ]
        record["has_padding_warning"] = "Left in padding" in description
        records.append(record)
    selected = [
        record for record in records
        if not queries or any(q.casefold() in record["name"].casefold() for q in queries)
    ]
    return {
        "provenance": "\n".join(line for line in text.splitlines()[:5] if line.startswith("//!")),
        "counts": counts,
        "total_types": len(records),
        "structs_with_padding_warnings": sum(r["has_padding_warning"] for r in records),
        "matched": len(selected),
        "truncated": len(selected) > limit,
        "types": selected[:limit],
        "scope": "Generated declarations only; not live values or proof of runtime readability.",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("catalogue", type=Path)
    parser.add_argument("--query", action="append", default=[], help="type-name substring; repeat for OR")
    parser.add_argument("--limit", type=int, default=20)
    args = parser.parse_args()
    if args.limit < 1:
        parser.error("--limit must be positive")
    if args.catalogue.suffix != ".rs":
        parser.error("catalogue must be an existing generated .rs file")
    try:
        raw = args.catalogue.read_bytes()
        result = inspect(raw.decode("utf-8-sig"), args.query, args.limit)
    except (OSError, UnicodeError, ValueError) as error:
        parser.error(str(error))
    result["sha256"] = hashlib.sha256(raw).hexdigest()
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
