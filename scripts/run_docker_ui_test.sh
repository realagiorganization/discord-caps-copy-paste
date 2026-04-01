#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
image_tag="discord-caps-copy-paste-ui-test:local"

docker build -t "$image_tag" -f "$repo_root/tests/ui/Dockerfile" "$repo_root"
docker run --rm \
  --user "$(id -u):$(id -g)" \
  -e HOME=/tmp/dccp-home \
  -e CARGO_HOME=/tmp/dccp-cargo-home \
  -v "$repo_root:/workspace" \
  --workdir /workspace \
  "$image_tag" \
  /workspace/tests/ui/run_in_container.sh
