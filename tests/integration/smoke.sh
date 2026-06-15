#!/usr/bin/env bash

set -u

fail=0
for dir in "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"/fixtures/*/; do
  about="$dir/about.json"
  title=$(jq -r .title "$about")
  args=(--data-binary "@$dir/body.xml")
  while read -r header; do args+=(-H "$header"); done < <(jq -r '.headers[]' "$about" | envsubst)

  if curl -s "${args[@]}" "$(jq -r .url "$about" | envsubst)" | grep -qF "$(jq -r .target "$about")"; then
    echo "  ✓ $title"
  else
    echo "  ✗ $title"; fail=1
  fi
done

[ "$fail" -eq 0 ]
