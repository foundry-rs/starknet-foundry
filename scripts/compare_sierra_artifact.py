#!/usr/bin/env python3

import json
import sys
from pathlib import Path


def normalize_debug_info(debug_info: dict[str, object]) -> dict[str, object]:
    normalized = dict(debug_info)

    for key in ("type_names", "libfunc_names", "user_func_names"):
        if key in normalized:
            normalized[key] = sorted(normalized[key], key=lambda item: item[0])

    normalized.pop("annotations", None)
    return normalized


def load_normalized(path: str) -> object:
    with Path(path).open() as file:
        data = json.load(file)

    if "sierra_program_debug_info" in data:
        data["sierra_program_debug_info"] = normalize_debug_info(
            data["sierra_program_debug_info"]
        )

    return data


def main() -> int:
    if len(sys.argv) != 3:
        print(
            "Usage: compare_sierra_artifact.py <expected-sierra.json> <actual-sierra.json>",
            file=sys.stderr,
        )
        return 1

    expected = load_normalized(sys.argv[1])
    actual = load_normalized(sys.argv[2])

    if expected != actual:
        print(
            "Sierra artifacts differ after normalizing debug info ordering and paths.",
            file=sys.stderr,
        )
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
