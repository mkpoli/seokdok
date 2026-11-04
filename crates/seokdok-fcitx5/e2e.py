"""End-to-end check of the fcitx5 addon over fcitx5's D-Bus frontend.

Run inside a private bus with an isolated fcitx5 (see e2e.sh). Types romaja
keys as a 두벌식 keyboard would, converts with Space, and checks what the
client receives.
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

# Keys by keysym, with their modifier state: Shift 1, Ctrl 4, Alt 8.
def sym(keysym, state=0, release=False):
    handled = ic.ProcessKeyEvent(dbus.UInt32(keysym), dbus.UInt32(0), dbus.UInt32(state), release, dbus.UInt32(0))
    pump()
    return bool(handled)

ALT_L, ALT_R, CTRL_L, SHIFT_L, SPACE, F9 = 0xffe9, 0xffea, 0xffe3, 0xffe1, 0x20, 0xffc6

# Left Alt tapped alone is the application's.
sym(ALT_L)
check("left Alt is not a mode key", sym(ALT_L, 8, True), False)

# Shift typed with a letter does not end a composition.
sym(SHIFT_L)
check("Shift keeps the composition", preedit[0], "")
type_("g")
sym(SHIFT_L)
sym(0x4b, 1)  # Shift+K
sym(SHIFT_L, 1, True)
check("Shift does not break the syllable", preedit[0] not in ("", "ㅎ"), True)
key_commit = len(commits)
type_("\n")
check("Enter commits", len(commits), key_commit + 1)

# F9 converts while composing, and the composing 釋讀 keeps Shift+Space.
type_("rkt")
check("F9 converts", sym(F9), True)
check("F9 opens no list yet", len(cands[0]), 0)
type_("\n")
type_("rk")
sym(SHIFT_L)
check("Shift+Space is a word break while composing", sym(SPACE, 1), True)
check("the word break stays in the composition", preedit[0].endswith(" "), True)
sym(SHIFT_L, 1, True)
type_("\n")

# Left Ctrl + left Alt tapped: 釋讀 → 國漢文混用 → 韓㐎 專用 → 釋讀.
def rotate():
    sym(CTRL_L)
    sym(ALT_L, 4)
    eaten = sym(ALT_L, 12, True)
    sym(CTRL_L, 4, True)
    return eaten


def script():
    path = os.path.join(os.environ["XDG_CONFIG_HOME"], "seokdok", "config.toml")
    with open(path, encoding="utf8") as f:
        return next(line.split('"')[1] for line in f if line.startswith("script ="))


def converted(keys):
    type_(keys + " ")
    shown = preedit[0]
    type_("\n")
    return shown


check("釋讀 converts the native word", converted("gksmf"), "天")
type_("rk")
check("the tap is eaten", rotate(), True)
check("the tap commits the composition", commits[-1:], ["가"])
check("the tap writes 國漢文混用", script(), "sino_only")
check("國漢文混用 leaves the native word", converted("gksmf"), "하늘")
rotate()
check("the tap writes 韓㐎 專用", script(), "hangul_only")
before = len(commits)
type_("rk ")
check("韓㐎 專用 commits on Space", commits[-1:], ["가 "])
check("one commit", len(commits), before + 1)

# Shift+Hangul does the same, from Latin too, and a held key rotates once.
HANGUL = 0xff31
sym(SHIFT_L)
check("Shift+Hangul is eaten", sym(HANGUL, 1), True)
check("Shift+Hangul writes 釋讀", script(), "seokdok")
check("its repeat is eaten", sym(HANGUL, 1), True)
check("its repeat does not rotate", script(), "seokdok")
sym(HANGUL, 1, True)
sym(SHIFT_L, 1, True)
check("釋讀 again converts the native word", converted("gksmf"), "天")

# Shift+Space with nothing composed: Korean ↔ Latin, and letters pass through.
sym(SHIFT_L)
check("Shift+Space toggles when idle", sym(SPACE, 1), True)
sym(SHIFT_L, 1, True)
check("latin passes keys through", key("g"), False)
# Right Alt tapped alone comes back.
sym(ALT_R)
check("the Right Alt tap is eaten", sym(ALT_R, 8, True), True)
check("back in Korean", key("g"), True)
type_("\n")

# The Hanja key reaches the engine while it is active, and converts.
type_("gksmf")
check("Hanja converts", sym(0xff34), True)
check("Hanja shows the conversion", preedit[0], "天")
type_("\n")

# With two input methods in the group, fcitx5 itself takes the Hangul key.
sym(0xff31)
sym(0xff31, 0, True)
check("fcitx5 switches away on Hangul", ctl.CurrentInputMethod(), "keyboard-us")
sym(0xff31)
sym(0xff31, 0, True)
check("and back", ctl.CurrentInputMethod(), "seokdok")

# AltGr is another keysym: it never switches.
ISO_LEVEL3 = 0xfe03
sym(ISO_LEVEL3)
check("AltGr tap is not eaten", sym(ISO_LEVEL3, 128, True), False)
check("still Korean", key("g"), True)
type_("\n")

# A candidate picked twice comes first; an e-mail field neither counts nor shows.
# (fcitx5 itself keeps password fields from the input method.)
PREEDIT_AND_PANEL = (1 << 1) | (1 << 39)
EMAIL = 1 << 7


def learned_file():
    return os.path.join(os.environ["XDG_CONFIG_HOME"], "seokdok", "learned.tsv")


def pick_second(keys):
    type_(keys + " ")
    type_(" ")
    second = cands[0][1]
    type_("2")
    return second


usual = converted("dkq")
ic.SetCapability(dbus.UInt64(PREEDIT_AND_PANEL | EMAIL))
pick_second("dkq")
pick_second("dkq")
check("an e-mail field does not reorder", converted("dkq"), usual)
pump(1500)
with open(learned_file(), encoding="utf8") as f:
    check("an e-mail field writes nothing", "압\t" in f.read(), False)
ic.SetCapability(dbus.UInt64(PREEDIT_AND_PANEL))
once = pick_second("dkq")
check("one pick does not reorder", converted("dkq"), usual)
pick_second("dkq")
pick_second("dkq")
check("two picks in a row put it first", converted("dkq"), once)
pump(1500)
with open(learned_file(), encoding="utf8") as f:
    check("the picks are on disk", once in f.read(), True)
os.remove(learned_file())
check("the file deleted, the order returns", converted("dkq"), usual)

print("all passed" if not failures else f"{failures} failed")
sys.exit(1 if failures else 0)
