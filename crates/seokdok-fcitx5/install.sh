#!/usr/bin/env bash
# Install the 釋讀 input method for fcitx5.
#
#   ./install.sh              for this user (~/.local), no root needed
#   sudo ./install.sh --system   for every user (/usr)
#   ./install.sh --uninstall  (add --system to remove the system install)
#
# Run it from the unpacked release folder (lib/, share/, dict/).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
system=0
uninstall=0
for a in "$@"; do
  case "$a" in
    --system) system=1 ;;
    --uninstall) uninstall=1 ;;
    *) echo "unknown option: $a" >&2; exit 2 ;;
  esac
done

if [[ $system = 1 ]]; then
  multiarch="$(gcc -print-multiarch 2>/dev/null || echo x86_64-linux-gnu)"
  libdir="/usr/lib/$multiarch/fcitx5"
  [[ -d "$libdir" ]] || libdir="/usr/lib/fcitx5"
  share="/usr/share"
  library="libseokdok"
else
  libdir="${XDG_DATA_HOME:-$HOME/.local/share}/seokdok/lib"
  share="${XDG_DATA_HOME:-$HOME/.local/share}"
  library="$libdir/libseokdok"
fi

if [[ $uninstall = 1 ]]; then
  rm -f "$libdir/libseokdok.so" "$share/fcitx5/addon/seokdok.conf" "$share/fcitx5/inputmethod/seokdok.conf"
  rm -rf "$share/seokdok/dict"
  rmdir "$share/seokdok/lib" "$share/seokdok" 2>/dev/null || true
  echo "Removed. Restart fcitx5 (fcitx5 -rd) to unload it."
  exit 0
fi

install -Dm755 "$here/lib/libseokdok.so" "$libdir/libseokdok.so"
install -d "$share/fcitx5/addon" "$share/fcitx5/inputmethod" "$share/seokdok/dict"
sed "s|^Library=.*|Library=$library|" "$here/share/fcitx5/addon/seokdok.conf" >"$share/fcitx5/addon/seokdok.conf"
install -m644 "$here/share/fcitx5/inputmethod/seokdok.conf" "$share/fcitx5/inputmethod/seokdok.conf"
install -m644 "$here"/dict/* "$share/seokdok/dict/"

echo "Installed to $share. Next:"
echo "  1. Restart fcitx5:   fcitx5 -rd"
echo "  2. Add 釋讀 in fcitx5-configtool (Input Method > uncheck 'Only Show Current Language' > 釋讀)."
