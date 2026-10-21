#!/usr/bin/env bash
# Copies the personal API spec that the backend generates into openapi/.
#
#   scripts/sync-openapi.sh              # from a sibling checkout: ../visor
#   scripts/sync-openapi.sh PATH         # from a spec file or a visor checkout
#   scripts/sync-openapi.sh --remote     # from GitHub main (needs `gh` auth)
#
# Then run `cargo test --test contract` to see what the CLI must follow.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
target="$root/openapi/openapi.json"
spec_path="developer-docs/openapi.json"

case "${1:-}" in
  --remote)
    gh api -H "Accept: application/vnd.github.raw" \
      "repos/Visor-Finance/visor/contents/$spec_path?ref=main" > "$target.tmp"
    ;;
  *)
    source="${1:-$root/../visor}"
    [ -d "$source" ] && source="$source/$spec_path"
    cp "$source" "$target.tmp"
    ;;
esac

# Refuse to replace the spec with something that is not one.
python3 -c 'import json, sys; assert "paths" in json.load(open(sys.argv[1]))' "$target.tmp"
mv "$target.tmp" "$target"
echo "Updated $target"
