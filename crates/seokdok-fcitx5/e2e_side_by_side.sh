#!/usr/bin/env bash
# Run e2e_side_by_side.py against a private fcitx5 that loads 釋讀 and 音讀 together.
# Build first: crates/seokdok-fcitx5/build.sh seokdok; crates/seokdok-fcitx5/build.sh eumdok
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/data/fcitx5/addon" "$tmp/data/fcitx5/inputmethod" "$tmp/config/fcitx5"
for id in seokdok eumdok; do
  "$here/conf.sh" "$id" addon | sed "s|^Library=.*|Library=$root/target/fcitx5/lib$id|" >"$tmp/data/fcitx5/addon/$id.conf"
  "$here/conf.sh" "$id" im >"$tmp/data/fcitx5/inputmethod/$id.conf"
  mkdir -p "$tmp/data/$id"
  ln -s "$root/dict" "$tmp/data/$id/dict"
done
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

[Groups/0/Items/2]
Name=eumdok
Layout=

[GroupOrder]
0=Default
PROFILE

export HOME="$tmp" XDG_CONFIG_HOME="$tmp/config" XDG_DATA_HOME="$tmp/data"
export XDG_DATA_DIRS="$tmp/data:/usr/local/share:/usr/share"
unset DISPLAY WAYLAND_DISPLAY
dbus-run-session -- bash -c '
  fcitx5 --disable=all --enable=dbus,dbusfrontend,seokdok,eumdok,keyboard >"$0/fcitx5.log" 2>&1 &
  python3 "$1"; status=$?
  kill %1 2>/dev/null; wait 2>/dev/null
  [ $status = 0 ] || tail -30 "$0/fcitx5.log"
  exit $status' "$tmp" "$here/e2e_side_by_side.py"
