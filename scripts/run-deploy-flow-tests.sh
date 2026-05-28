#!/usr/bin/env bash
# Run full deploy-flow integration tests (temp sandboxes, not in repo).
# Usage:
#   ./scripts/run-deploy-flow-tests.sh              # all deploy_flow tests
#   ./scripts/run-deploy-flow-tests.sh bethesda     # engine family
#   ./scripts/run-deploy-flow-tests.sh skyrimse     # profile id substring

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/src-tauri"

FILTER="${1:-deploy_flow}"

if [[ "$FILTER" == "deploy_flow" ]]; then
  echo "Running all deploy-flow tests..."
  cargo test deploy_flow -- --nocapture
elif [[ "$FILTER" == deploy_flow_* ]]; then
  cargo test "$FILTER" -- --nocapture
elif [[ "$FILTER" =~ ^(bethesda|data|kcd|cyberpunk|bg3|mods|mod_root|stardew|bepinex|subnautica|marvel|unreal|mod_path|game_root)$ ]]; then
  cargo test "deploy_flow_engine_${FILTER}" -- --nocapture
else
  echo "Running tests matching profile: $FILTER"
  cargo test deploy_flow -- "$FILTER" --nocapture
fi
