#!/usr/bin/env bash
# Build the Rust core for macOS as one universal static library (arm64 +
# x86_64) with seokdok.h and a module map, imported in Swift as
# `import SeokdokCore`:
#
#   macos/build/lib/libseokdok_ffi.a
#   macos/build/include/{seokdok.h,module.modulemap}
#
# The libraries cross-compile on any host. Joining them needs `lipo` (Xcode's
# command-line tools) or an `llvm-lipo` as new as the LLVM of rustc.
#
#   rustup target add aarch64-apple-darwin x86_64-apple-darwin
#   macos/build-rust.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/.." && pwd)"
targets=(aarch64-apple-darwin x86_64-apple-darwin)
# Keep in step with deploymentTarget in project.yml.
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-13.0}"
for t in "${targets[@]}"; do
  cargo build --release --target "$t" -p seokdok-ffi --manifest-path "$root/Cargo.toml"
done

lipo_bin=""
for c in lipo llvm-lipo; do
  if command -v "$c" >/dev/null 2>&1; then
    lipo_bin="$c"
    break
  fi
done
if [ -z "$lipo_bin" ]; then
  echo "lipo not found: install Xcode's command-line tools (xcode-select --install) or LLVM" >&2
  exit 1
fi

out="$here/build"
rm -rf "$out"
mkdir -p "$out/lib" "$out/include"
inputs=()
for t in "${targets[@]}"; do
  inputs+=("$root/target/$t/release/libseokdok_ffi.a")
done
"$lipo_bin" -create "${inputs[@]}" -output "$out/lib/libseokdok_ffi.a"
cp "$root/crates/seokdok-ffi/include/seokdok.h" "$out/include/"
cat > "$out/include/module.modulemap" <<'MAP'
module SeokdokCore {
    header "seokdok.h"
    export *
}
MAP
"$lipo_bin" -info "$out/lib/libseokdok_ffi.a"
echo "$out"
