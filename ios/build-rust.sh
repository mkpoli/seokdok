#!/usr/bin/env bash
# Build the Rust core for iOS and wrap it as ios/SeokdokCore.xcframework
# (static library + seokdok.h + a module map, imported in Swift as
# `import SeokdokCore`). The libraries cross-compile on any host; creating the
# XCFramework needs Xcode (macOS).
#
#   rustup target add aarch64-apple-ios aarch64-apple-ios-sim
#   ios/build-rust.sh [seokdok|eumdok]
set -euo pipefail
id="${1:-seokdok}"
features=()
[ "$id" = seokdok ] || features=(--features "$id")
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/.." && pwd)"
targets=(aarch64-apple-ios aarch64-apple-ios-sim)
# Match the deployment target in project.yml.
export IPHONEOS_DEPLOYMENT_TARGET=16.0
for t in "${targets[@]}"; do
  cargo build --release --target "$t" -p seokdok-ffi --manifest-path "$root/Cargo.toml" ${features[@]+"${features[@]}"}
done

headers="$root/target/ios-headers"
rm -rf "$headers"
mkdir -p "$headers"
cp "$root/crates/seokdok-ffi/include/seokdok.h" "$headers/"
cat > "$headers/module.modulemap" <<'MAP'
module SeokdokCore {
    header "seokdok.h"
    export *
}
MAP

if ! command -v xcodebuild >/dev/null; then
  echo "built the libraries in $root/target/*/release; the XCFramework needs Xcode" >&2
  exit 1
fi
out="$here/SeokdokCore.xcframework"
rm -rf "$out"
args=()
for t in "${targets[@]}"; do
  args+=(-library "$root/target/$t/release/libseokdok_ffi.a" -headers "$headers")
done
xcodebuild -create-xcframework "${args[@]}" -output "$out"
echo "$out"
