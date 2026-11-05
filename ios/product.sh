#!/usr/bin/env bash
# Write Seokdok.xcodeproj for a product: project.yml as it is for 釋讀, and with
# the values of products/<id>.json for another. The target names stay, since the
# bundle IDs, the App Group, the icon and the names are what tell them apart.
#
#   ios/product.sh [seokdok|eumdok]
set -euo pipefail
id="${1:-seokdok}"
cd "$(dirname "$0")"
if [ "$id" = seokdok ]; then
  exec xcodegen
fi
product="../products/$id.json"
get() { jq -r "$1" "$product"; }
capital="$(get .latin)"
spec="project.$id.yml"
trap 'rm -f "$spec"' EXIT
has_kun=false
jq -e '.scripts | index("seokdok")' "$product" >/dev/null && has_kun=true
sed -e "s/^\( *\)PRODUCT_BUNDLE_IDENTIFIER: li\.mkpo\.seokdok\.keyboard/\1PRODUCT_BUNDLE_IDENTIFIER: $(get .ios.keyboard_bundle_id)/" \
    -e "s/^\( *\)PRODUCT_BUNDLE_IDENTIFIER: li\.mkpo\.seokdok\.uitests/\1PRODUCT_BUNDLE_IDENTIFIER: $(get .ios.bundle_id).uitests/" \
    -e "s/^\( *\)PRODUCT_BUNDLE_IDENTIFIER: li\.mkpo\.seokdok$/\1PRODUCT_BUNDLE_IDENTIFIER: $(get .ios.bundle_id)/" \
    -e "s/group\.li\.mkpo\.seokdok/$(get .ios.app_group)/" \
    -e "s/ASSETCATALOG_COMPILER_APPICON_NAME: AppIcon/ASSETCATALOG_COMPILER_APPICON_NAME: AppIcon$capital/" \
    -e "s/CFBundleDisplayName: 釋讀/CFBundleDisplayName: $(get .hanja)/" \
    -e "s/SeokdokProductHanja: 釋讀/SeokdokProductHanja: $(get .hanja)/" \
    -e "s/SeokdokProductDir: seokdok/SeokdokProductDir: $(get .config_dir)/" \
    -e "s|SeokdokProductGuide: .*|SeokdokProductGuide: $(get .guide)|" \
    -e "s/SeokdokProductHasKun: true/SeokdokProductHasKun: $has_kun/" \
    project.yml >"$spec"
xcodegen --spec "$spec"
