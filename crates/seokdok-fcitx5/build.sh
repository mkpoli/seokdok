#!/usr/bin/env bash
# Build the fcitx5 addon: the Rust core as a static library, linked into
# libseokdok.so with g++ against the fcitx5 headers.
#
#   crates/seokdok-fcitx5/build.sh            uses pkg-config Fcitx5Core
#   FCITX5_INCLUDE=/path/include/Fcitx5 crates/seokdok-fcitx5/build.sh
#
# Output: target/fcitx5/libseokdok.so
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
out="$root/target/fcitx5"
mkdir -p "$out"

cargo build --release -p seokdok-fcitx5 --manifest-path "$root/Cargo.toml"

if [[ -n "${FCITX5_INCLUDE:-}" ]]; then
  cflags="-I$FCITX5_INCLUDE/Core -I$FCITX5_INCLUDE/Config -I$FCITX5_INCLUDE/Utils"
  libdir="${FCITX5_LIBDIR:-/usr/lib/$(gcc -print-multiarch)}"
  libs="$libdir/libFcitx5Core.so.7 $libdir/libFcitx5Utils.so.2 $libdir/libFcitx5Config.so.6"
else
  cflags="$(pkg-config --cflags Fcitx5Core Fcitx5Utils)"
  libs="$(pkg-config --libs Fcitx5Core Fcitx5Utils)"
fi

# shellcheck disable=SC2086
g++ -std=c++17 -O2 -fPIC -shared -fvisibility=hidden -Wall -Wextra -Wno-unused-parameter \
  $cflags -o "$out/libseokdok.so" "$here/addon/seokdok.cpp" \
  -Wl,--whole-archive "$root/target/release/libseokdok_fcitx5.a" -Wl,--no-whole-archive \
  -Wl,--exclude-libs,ALL $libs -lpthread -ldl -lm
strip --strip-unneeded "$out/libseokdok.so"
echo "$out/libseokdok.so"
