#!/usr/bin/env bash
# Write a product's fcitx5 files from products/<id>.json:
#   conf.sh <id> addon|im
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
product="$here/../../products/${1:?usage: conf.sh <id> addon|im}.json"
sed -e "s/@ID@/$(jq -r .fcitx5.addon "$product")/" \
    -e "s/@LATIN@/$(jq -r .latin "$product")/" \
    -e "s/@HANJA@/$(jq -r .hanja "$product")/" \
    -e "s/@GLYPH@/$(jq -r .glyph "$product")/" \
    "$here/addon/${2:?usage: conf.sh <id> addon|im}.conf.in"
