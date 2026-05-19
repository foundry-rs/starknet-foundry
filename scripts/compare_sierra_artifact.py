#!/usr/bin/env python3

import json
import sys
from pathlib import Path


def normalize_path(path: str) -> str:
    for marker in ("/starkgate-contracts/", "/registry/std/"):
        if marker in path:
            return path.split(marker, 1)[1]
    return path


def normalize_code_location(location: list[object]) -> tuple[object, ...]:
    path, span, is_user_code = location
    return (
        normalize_path(path),
        span["start"]["line"],
        span["start"]["col"],
        span["end"]["line"],
        span["end"]["col"],
        is_user_code,
    )


def normalize_code_location_value(location: list[object]) -> list[object]:
    path, span, is_user_code = location
    return [normalize_path(path), span, is_user_code]


def normalize_debug_info(debug_info: dict[str, object]) -> dict[str, object]:
    normalized = dict(debug_info)

    for key in ("type_names", "libfunc_names", "user_func_names"):
        if key in normalized:
            normalized[key] = sorted(normalized[key], key=lambda item: item[0])

    annotations = dict(normalized.get("annotations", {}))

    profiler = dict(annotations.get("github.com/software-mansion/cairo-profiler", {}))
    if "statements_functions" in profiler:
        profiler["statements_functions"] = {
            statement_idx: sorted(functions)
            for statement_idx, functions in profiler["statements_functions"].items()
        }
    if profiler:
        annotations["github.com/software-mansion/cairo-profiler"] = profiler

    coverage = dict(annotations.get("github.com/software-mansion/cairo-coverage", {}))
    if "statements_code_locations" in coverage:
        coverage["statements_code_locations"] = {
            statement_idx: sorted(
                [normalize_code_location_value(location) for location in locations],
                key=normalize_code_location,
            )
            for statement_idx, locations in coverage["statements_code_locations"].items()
        }
    if coverage:
        annotations["github.com/software-mansion/cairo-coverage"] = coverage

    normalized["annotations"] = annotations
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
