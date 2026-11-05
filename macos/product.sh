#!/usr/bin/env bash
# Write Seokdok.xcodeproj for a product: project.yml and Seokdok/Info.plist as they
# are for 釋讀, and with the values of products/<id>.json for another, so that it
# installs beside 釋讀 as <Name>.app with its own bundle ID, input sources and
# connection. The target name stays.
#
#   macos/product.sh [seokdok|eumdok]
set -euo pipefail
id="${1:-seokdok}"
cd "$(dirname "$0")"
if [ "$id" = seokdok ]; then
  exec xcodegen
fi
product="../products/$id.json"
get() { jq -r "$1" "$product"; }
name="$(get .macos.app_name)"
bundle="$(get .macos.bundle_id)"
hanja="$(get .hanja)"
mkdir -p build
sed -e "s/li\.mkpo\.inputmethod\.seokdok/$bundle/g" -e "s/釋讀/$hanja/g" -e "s/seokdok\.tiff/$id.tiff/g" Seokdok/Info.plist >"build/Info-$id.plist"
spec="project.$id.yml"
trap 'rm -f "$spec"' EXIT
sed -e "s/PRODUCT_NAME: Seokdok/PRODUCT_NAME: $name/" \
    -e "s/PRODUCT_BUNDLE_IDENTIFIER: li\.mkpo\.inputmethod\.seokdok/PRODUCT_BUNDLE_IDENTIFIER: $bundle/" \
    -e "s/ASSETCATALOG_COMPILER_APPICON_NAME: AppIcon/ASSETCATALOG_COMPILER_APPICON_NAME: AppIcon$name/" \
    -e "s#INFOPLIST_FILE: Seokdok/Info.plist#INFOPLIST_FILE: build/Info-$id.plist#" \
    project.yml >"$spec"
python3 - "$spec" "$bundle" "$hanja" <<'PY'
import sys
path, bundle, hanja = sys.argv[1:]
text = open(path, encoding="utf-8").read()
step = f"""      - name: Name the input sources
        basedOnDependencyAnalysis: false
        script: |
          for d in "$TARGET_BUILD_DIR/$UNLOCALIZED_RESOURCES_FOLDER_PATH"/*.lproj; do
            lang="$(basename "$d")"
            sed -e 's/li\\.mkpo\\.inputmethod\\.seokdok/{bundle}/g' -e 's/釋讀/{hanja}/g' "$PROJECT_DIR/Seokdok/Resources/$lang/InfoPlist.strings" > "$d/InfoPlist.strings"
            plutil -convert binary1 "$d/InfoPlist.strings"
          done
"""
mark = "    postCompileScripts:\n"
assert mark in text
text = text.replace(mark, mark + step, 1)
open(path, "w", encoding="utf-8").write(text)
PY
xcodegen --spec "$spec"
