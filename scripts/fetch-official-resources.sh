#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
resources_dir="$root_dir/resources"

mkdir -p "$resources_dir"

curl -L --fail --silent --show-error \
  https://raw.githubusercontent.com/zanfranceschi/rinha-de-backend-2026/main/resources/normalization.json \
  -o "$resources_dir/normalization.json"

curl -L --fail --silent --show-error \
  https://raw.githubusercontent.com/zanfranceschi/rinha-de-backend-2026/main/resources/mcc_risk.json \
  -o "$resources_dir/mcc_risk.json"

curl -L --fail --silent --show-error \
  https://raw.githubusercontent.com/zanfranceschi/rinha-de-backend-2026/main/resources/references.json.gz \
  -o "$resources_dir/references.json.gz"
