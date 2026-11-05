"""釋讀 and 音讀 in one fcitx5, over its D-Bus frontend (see e2e_side_by_side.sh):
each converts in its own way, keeps its own folder, and 音讀's rotation has no 釋讀.
"""
import os
import sys
import time

import dbus
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib

DBusGMainLoop(set_as_default=True)
bus = dbus.SessionBus()
for _ in range(100):
    if bus.name_has_owner("org.fcitx.Fcitx5"):
        break
    time.sleep(0.1)
else:
    sys.exit("fcitx5 did not start")

im = dbus.Interface(bus.get_object("org.fcitx.Fcitx5", "/org/freedesktop/portal/inputmethod"), "org.fcitx.Fcitx.InputMethod1")
path, _ = im.CreateInputContext([("program", "e2e")])
ic = dbus.Interface(bus.get_object("org.fcitx.Fcitx5", path), "org.fcitx.Fcitx.InputContext1")
ctl = dbus.Interface(bus.get_object("org.fcitx.Fcitx5", "/controller"), "org.fcitx.Fcitx.Controller1")
preedit = [""]
ic.connect_to_signal("UpdateFormattedPreedit", lambda runs, cur: preedit.__setitem__(0, "".join(str(t) for t, _ in runs)))
ic.connect_to_signal("UpdateClientSideUI", lambda *a: None)
loop = GLib.MainLoop()


def pump(ms=150):
    GLib.timeout_add(ms, loop.quit)
    loop.run()


ic.SetCapability(dbus.UInt64((1 << 1) | (1 << 39)))
ic.FocusIn()
failures = []


def check(what, got, want):
    print(("ok  " if got == want else "FAIL"), what, repr(got))
    if got != want:
        failures.append(what)


def sym(s, state=0, release=False):
    r = bool(ic.ProcessKeyEvent(dbus.UInt32(s), dbus.UInt32(0), dbus.UInt32(state), release, dbus.UInt32(0)))
    pump(40)
    return r


def press(ch):
    s = {" ": 0x20, "\n": 0xff0d}.get(ch, ord(ch))
    sym(s)
    sym(s, 0, True)


def converted(keys):
    for ch in keys + " ":
        press(ch)
    shown = preedit[0]
    press("\n")
    return shown


def rotate():
    sym(0xffe3)
    sym(0xffe9, 4)
    sym(0xffe9, 12, True)
    sym(0xffe3, 4, True)


def script(folder):
    with open(os.path.join(os.environ["XDG_CONFIG_HOME"], folder, "config.toml"), encoding="utf8") as f:
        return next(line.split('"')[1] for line in f if line.startswith("script ="))


def use(name):
    ctl.SetCurrentIM(name)
    pump(300)
    check("the current input method", str(ctl.CurrentInputMethod()), name)
    pump(3000)


use("seokdok")
check("釋讀 writes the native word in Hanja", converted("gksmf"), "天")
use("eumdok")
check("音讀 leaves the native word", converted("gksmf"), "하늘")
check("音讀 writes the Sino-Korean word in Hanja", converted("rhdqn").strip(), "工夫")
rotate()
check("音讀 rotates to 韓㐎 專用", script("eumdok"), "hangul_only")
rotate()
check("音讀 rotates back, with no 釋讀", script("eumdok"), "sino_only")
check("音讀 still converts", converted("rhdqn").strip(), "工夫")
use("seokdok")
check("釋讀 kept its own settings", converted("gksmf"), "天")
sys.exit(1 if failures else 0)
