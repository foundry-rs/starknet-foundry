#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "Usage: compare_sierra_artifact.sh <expected-sierra.json> <actual-sierra.json>" >&2
  exit 1
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
filter_path="${script_dir}/compare_sierra_artifact.jq"

expected="$(jq -cS -f "${filter_path}" "$1")"
actual="$(jq -cS -f "${filter_path}" "$2")"

if [[ "${expected}" != "${actual}" ]]; then
  echo "Sierra artifacts differ after normalizing debug info ordering and paths." >&2
  exit 1
fi
