#!/usr/bin/env bash
# Build the Rust core for iOS and wrap it as ios/SeokdokCore.xcframework
# (static library + seokdok.h + a module map, imported in Swift as
# `import SeokdokCore`). The libraries cross-compile on any host; creating the
# XCFramework needs Xcode (macOS).
#
#   rustup target add aarch64-apple-ios aarch64-apple-ios-sim
#   ios/build-rust.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/.." && pwd)"
targets=(aarch64-apple-ios aarch64-apple-ios-sim)
for t in "${targets[@]}"; do
  cargo build --release --target "$t" -p seokdok-ffi --manifest-path "$root/Cargo.toml"
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

out="$here/SeokdokCore.xcframework"
rm -rf "$out"
args=()
for t in "${targets[@]}"; do
  args+=(-library "$root/target/$t/release/libseokdok_ffi.a" -headers "$headers")
done
xcodebuild -create-xcframework "${args[@]}" -output "$out"
echo "$out"
