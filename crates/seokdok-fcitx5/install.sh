#!/usr/bin/env bash
# Install the input method of this release folder for fcitx5.
#
#   ./install.sh              for this user (~/.local), no root needed
#   sudo ./install.sh --system   for every user (/usr)
#   ./install.sh --uninstall  (add --system to remove the system install)
#
# Run it from the unpacked release folder (lib/, share/, dict/).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=/dev/null
. "$here/product"
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
  library="lib$id"
else
  libdir="${XDG_DATA_HOME:-$HOME/.local/share}/$id/lib"
  share="${XDG_DATA_HOME:-$HOME/.local/share}"
  library="$libdir/lib$id"
fi

if [[ $uninstall = 1 ]]; then
  rm -f "$libdir/lib$id.so" "$share/fcitx5/addon/$id.conf" "$share/fcitx5/inputmethod/$id.conf"
  rm -rf "${share:?}/$id/dict"
  rmdir "$share/$id/lib" "$share/$id" 2>/dev/null || true
  echo "Removed. Restart fcitx5 (fcitx5 -rd) to unload it."
  exit 0
fi

install -Dm755 "$here/lib/lib$id.so" "$libdir/lib$id.so"
install -d "$share/fcitx5/addon" "$share/fcitx5/inputmethod" "$share/$id/dict"
sed "s|^Library=.*|Library=$library|" "$here/share/fcitx5/addon/$id.conf" >"$share/fcitx5/addon/$id.conf"
install -m644 "$here/share/fcitx5/inputmethod/$id.conf" "$share/fcitx5/inputmethod/$id.conf"
install -m644 "$here"/dict/* "$share/$id/dict/"

echo "Installed to $share. Next:"
echo "  1. Restart fcitx5:   fcitx5 -rd"
echo "  2. Add $hanja in fcitx5-configtool (Input Method > uncheck 'Only Show Current Language' > $hanja)."
