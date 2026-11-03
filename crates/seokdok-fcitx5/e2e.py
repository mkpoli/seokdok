"""End-to-end check of the fcitx5 addon over fcitx5's D-Bus frontend.

Run inside a private bus with an isolated fcitx5 (see e2e.sh). Types romaja
keys as a 두벌식 keyboard would, converts with Space, and checks what the
client receives.
"""
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

im = dbus.Interface(bus.get_object("org.fcitx.Fcitx5", "/org/freedesktop/portal/inputmethod"),
                    "org.fcitx.Fcitx.InputMethod1")
path, _ = im.CreateInputContext([("program", "e2e")])
ic = dbus.Interface(bus.get_object("org.fcitx.Fcitx5", path), "org.fcitx.Fcitx.InputContext1")
ctl = dbus.Interface(bus.get_object("org.fcitx.Fcitx5", "/controller"), "org.fcitx.Fcitx.Controller1")

commits, preedit, cands = [], [""], [[]]
ic.connect_to_signal("CommitString", lambda s: commits.append(str(s)))
ic.connect_to_signal("UpdateFormattedPreedit", lambda runs, cur: preedit.__setitem__(0, "".join(str(t) for t, _ in runs)))
ic.connect_to_signal("UpdateClientSideUI",
                     lambda pre, cur, aup, adn, cand, idx, lay, hp, hn: cands.__setitem__(0, [str(t) for _, t in cand]))

loop = GLib.MainLoop()


def pump(ms=150):
    GLib.timeout_add(ms, loop.quit)
    loop.run()


# Preedit (bit 1) and client-side input panel (bit 39) capabilities.
ic.SetCapability(dbus.UInt64((1 << 1) | (1 << 39)))
ic.FocusIn()
ctl.SetCurrentIM("seokdok")
pump(300)
assert ctl.CurrentInputMethod() == "seokdok", ctl.CurrentInputMethod()

# Let the dictionaries load.
pump(3000)

SYMS = {" ": 0x20, "\n": 0xff0d, "\b": 0xff08}


def key(ch, state=0, release=False):
    sym = SYMS.get(ch, ord(ch))
    return bool(ic.ProcessKeyEvent(dbus.UInt32(sym), dbus.UInt32(0), dbus.UInt32(state), release, dbus.UInt32(0)))


def type_(s):
    for c in s:
        key(c)
        key(c, release=True)
    pump()


failures = 0


def check(name, got, want):
    global failures
    ok = got == want
    failures += not ok
    print(("ok  " if ok else "FAIL"), name, repr(got), "" if ok else f"(want {want!r})")


type_("gksmfdmf")
check("preedit while typing", preedit[0], "하늘을")
type_(" ")
check("conversion preedit", preedit[0], "天을")
type_("\n")
check("commit", commits[-1:], ["天을"])
check("preedit cleared", preedit[0], "")

# A second Space opens the list; a digit picks from it.
type_("tkfka ")
type_(" ")
first = list(cands[0])
check("second space opens the list", len(first) > 1, True)
type_("2")
check("digit picks the second candidate", commits[-1:], first[1:2])

# Alt tapped alone switches to Latin; letters then pass through.
ic.ProcessKeyEvent(dbus.UInt32(0xffe9), dbus.UInt32(0), dbus.UInt32(0), False, dbus.UInt32(0))
ic.ProcessKeyEvent(dbus.UInt32(0xffe9), dbus.UInt32(0), dbus.UInt32(8), True, dbus.UInt32(0))
pump()
check("latin passes keys through", key("g"), False)

print("all passed" if not failures else f"{failures} failed")
sys.exit(1 if failures else 0)
