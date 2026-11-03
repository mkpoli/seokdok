#!/usr/bin/env bash
# Run e2e.py against a private fcitx5 that loads the freshly built addon.
# Build first with crates/seokdok-fcitx5/build.sh.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/data/fcitx5/addon" "$tmp/data/fcitx5/inputmethod" "$tmp/config/fcitx5" "$tmp/data/seokdok"
sed "s|^Library=.*|Library=$root/target/fcitx5/libseokdok|" "$here/addon/seokdok-addon.conf" >"$tmp/data/fcitx5/addon/seokdok.conf"
cp "$here/addon/seokdok-im.conf" "$tmp/data/fcitx5/inputmethod/seokdok.conf"
ln -s "$root/dict" "$tmp/data/seokdok/dict"
cat > "$tmp/config/fcitx5/profile" <<'PROFILE'
[Groups/0]
Name=Default
Default Layout=us
DefaultIM=seokdok

[Groups/0/Items/0]
Name=keyboard-us
Layout=

[Groups/0/Items/1]
Name=seokdok
Layout=

[GroupOrder]
0=Default
PROFILE

export HOME="$tmp" XDG_CONFIG_HOME="$tmp/config" XDG_DATA_HOME="$tmp/data"
export XDG_DATA_DIRS="$tmp/data:/usr/local/share:/usr/share"
unset DISPLAY WAYLAND_DISPLAY
dbus-run-session -- bash -c '
  fcitx5 --disable=all --enable=dbus,dbusfrontend,seokdok,keyboard >"$0/fcitx5.log" 2>&1 &
  python3 "$1"; status=$?
  kill %1 2>/dev/null; wait 2>/dev/null
  [ $status = 0 ] || tail -30 "$0/fcitx5.log"
  exit $status' "$tmp" "$here/e2e.py"
