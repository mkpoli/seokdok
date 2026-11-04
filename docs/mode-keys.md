# Mode keys

What each platform's Korean tools use, what the system keeps for itself, and
what 釋讀 binds by default. Sources were read on 2026-11-04. The lists are
`input.keys` in `crates/seokdok-session/src/config.rs`; the defaults are
`ModeKeys::for_platform` in `crates/seokdok-session/src/mode_keys.rs`.

## Existing tools and the defaults

| Platform | What the existing tools do | 釋讀 default | Source |
|---|---|---|---|
| Windows, Microsoft IME | Right Alt switches Korean and English; right Ctrl converts to Hanja | `RAlt` tap, `RCtrl` tap | [1] |
| Windows, keyboard drivers | The driver maps keys to 한/영 and 한자: `kbd101a` right Alt, right Ctrl; `kbd101b` right Ctrl, right Alt; `kbd101c` Shift+Space, Ctrl+Space; `kbd103` dedicated keys | `Hangul`, `Hanja`, which these all produce. Shift+Space stays 띄어쓰기 unless the type 3 driver turns it into 한/영 first | [2], [3], [4] |
| Windows, Microsoft Japanese IME | 半角/全角 turns Japanese input on and off; 変換 reconverts, 無変換 cycles kana, unless the ATOK template or a key assignment changes them | `Zenkaku` toggles. `Muhenkan` to Latin, `Henkan` back, then converts | [5], [7], [8] |
| Windows, ImeOn and ImeOff keys | LANG1, LANG2: scan 0xF2, 0xF1, VK_IME_ON, VK_IME_OFF | `ImeOn` back to Korean, `ImeOff` to Latin | [4], [6] |
| macOS, system | Control-Space, Control-Option-Space, the Fn or Globe key, and Caps Lock (an option in Input Sources) switch input sources | Not bound: the system takes them. The three 釋讀 modes are input sources, so these reach them | [9] |
| macOS, Apple Korean | Option-Return opens the Hanja candidates | `Alt+Enter` | [10] |
| macOS, JIS keyboards | 英数 and かな | `Eisu` to Latin, `Kana` back. Korean keyboards send 한/영 and 한자 on the same USB usages, so a Korean PC keyboard reaches these two entries | [11] |
| iOS and iPadOS | Control-Space or the Switch Keyboard key moves between keyboards. Overriding `pressesBegan` in a keyboard extension is reported not to be called; Apple's documentation does not say either way | All lists empty; the on-screen mode key remains | [12], [13] |
| Linux, ibus-hangul | `switch-keys` Hangul, Shift+space; `hanja-keys` Hangul_Hanja, F9 | `Hangul`, `Shift+Space`; `Hanja`, `F9` | [14] |
| Linux, fcitx5-hangul | `HanjaModeToggleKey` Hangul_Hanja, F9; no switching key of its own | The same | [15] |
| Linux, fcitx5 | With more than one input method in the group, trigger keys Ctrl+Space, Zenkaku_Hankaku, Hangul and alternate trigger Shift_L act before the engine | Not fought: in a group of one these reach the engine, which binds `Hangul` and `Zenkaku`. Hangul_Hanja activates only from another input method, so it reaches the engine while 釋讀 is active | [16] |
| Linux, xkeyboard-config | `kr(ralt_hangul)` and `kr(rctrl_hanja)` make right Alt and right Ctrl produce Hangul and Hangul_Hanja | Nothing extra: they arrive as `Hangul`, `Hanja` | [17] |
| Android, system | `PhoneWindowManager` takes `KEYCODE_LANGUAGE_SWITCH` and Ctrl+Space to switch the hardware layout | Not bound | [18] |
| Android, key codes | `Generic.kl` maps Linux `KEY_HANGEUL` (122) to `KANA` and `KEY_HANJA` (123) to `EISU`, and has `ZENKAKU_HANKAKU`, `HENKAN`, `MUHENKAN`, `KATAKANA_HIRAGANA` | `KANA` is `Hangul` and `EISU` is `Hanja` here | [19], [20] |
| Android, Samsung | Shift+Space, Ctrl+Space and left Alt+Shift switch language; “Change language shortcut” in the physical keyboard settings turns them on and off | `Shift+Space` is bound. Whether it reaches an input method while Samsung's shortcut is on is uncertain | [21] |
| Android, Gboard | Uncertain. No Google page documents a hardware shortcut. One forum user reports Shift+Space and Shift+Ctrl did nothing, and a Unity issue reports Ctrl+Space or Alt+Shift | Same as Android above | [22], [23] |

## Decisions

**The script chord.** `script_cycle` rotates `conversion.script`:
釋讀 → 國漢文混用 → 韓㐎 專用 → 釋讀. No other tool has this. It is left Ctrl and left Alt tapped
together, on every platform with a hardware keyboard: a chord of two modifiers
and no other key cannot be an application shortcut, because a shortcut needs a
key besides its modifiers. It produces no text on any layout, and it differs
from AltGr, which Windows reports as left Ctrl with right Alt. Where a 한/영 key
exists, Shift+한/영 does the same.

The rotation goes through `Config::update_file` and the session follows only
after the file is written, so a config.toml that does not parse changes nothing.
Every application follows the file, and the spelling changes on the running
engine with no dictionary load. The chord always lands in 釋讀 mode, committing
any composition first: from Latin or plain 한글 it goes to the next script, so
the icon shows the change. A held key rotates once. The logic is
`crates/seokdok-session/src/script_cycle.rs`.

Plain 한글 is reached by the menu on Windows, the input source on macOS, or a
key added to `hangul`. Latin has no other on-screen key on Android and iOS, so
their mode key has a fourth stop: 釋 → 混 → 한 → A → 釋. A keyboard that cannot
write its settings, such as iOS without Full Access and a writable folder,
moves between Korean and Latin instead.

macOS has no icon for the script, because its three input sources are the
modes. The input method's menu names the script.

**Shift+Space.** It already means 띄어쓰기 while composing and previous
candidate in a conversion. While 釋讀 is composing or converting it keeps those
meanings. Anywhere else, on Linux and Android, it toggles Korean and Latin as
ibus-hangul does. That includes plain 한글 while composing, where Space itself
types the space, so the toggle commits. Windows does not bind it, because its
Korean precedent is a driver choice that arrives as 한/영.

**Left Alt.** A tap of either Alt used to toggle. On Windows and Linux left Alt
tapped alone opens application menus, and the Microsoft IME uses the right one,
so only right Alt toggles there by default. On macOS Option tapped alone does
nothing else, so either side still toggles.

**AltGr.** Windows reports left Ctrl with right Alt, a different chord from
`RAlt`. On Linux AltGr is `ISO_Level3_Shift`, which has no name in the lists.

**Mid-composition.** A switch commits as `Session::set_mode` does. A switch to
the current mode does nothing and leaves the composition alone. 韓㐎 專用 has
no 釋讀 to tell from 한글, so `seokdok` and `hangul` leave Latin and do nothing
else. `script_cycle` commits even when the mode stays, because the script changes.

**変換.** It sits in both `korean` and `convert`: out of Latin first, then
convert. Idle in Korean it stays the application's key.

**Preserved keys.** `ITfKeystrokeMgr::PreserveKey` registers a key by virtual
key and modifiers [24]. The key event sink already handles these keys, including
with no document in focus, and modifier taps need key-up tracking that a
preserved key would not replace. The sink stays the single path, so
`OnTestKeyDown` and `OnKeyDown` cannot disagree about a key.

## The Windows quick menu

The input indicator on the taskbar shows the icon of one button: 釋 converts to
釋讀, 混 to 國漢文混用, 한 types Hangul without converting, A passes keys through.
A left click toggles conversion and plain Hangul, or Hangul and Latin under
韓㐎 專用. A right click opens the menu:
the script, the input mode, `candidate_layout` and `old_ha`, then Settings….
The script entries name the `script_cycle` keys, and 釋讀 and 한글 name their own
lists, which are empty by default.
`windows.menu_items` in config.toml, or the settings app, adds the other
settings or takes these away. Choosing a script also leaves plain Hangul and
Latin, so the icon always shows the choice.

A setting chosen there goes to config.toml through `Config::save_to` and applies
at the next key. Other applications load their own copy of the DLL, so they
compare the file's timestamp and length at every key and on focus. Spelling
options change on the running engine with no dictionary load. A config.toml
that does not parse is left alone and the menu does nothing.

**One button.** Since Windows 8 a language-bar item is ignored unless its GUID
is `GUID_LBI_INPUTMODE`, and only the first such item is shown [26], [27]. Those
pages do not mention Windows 11, and no Microsoft page found says the rule
changed. Separate buttons for 釋, 混 and 한 would therefore not appear, and were
not built. The style follows Microsoft's sample and Mozc: `TF_LBI_STYLE_BTN_BUTTON`
with `TF_LBI_STYLE_SHOWNINTRAY` [28], [29]. Microsoft documents `InitMenu` only
for `TF_LBI_STYLE_BTN_MENU` items [30], and Mozc draws its menu with
`TrackPopupMenu` on `TF_LBI_CLK_RIGHT`, as this one does.

## Uncertain

- How Windows 11 draws the quick menu and the icon: nothing on Windows has run
  it. Microsoft's pages on the indicator date from Windows 8, and how the
  Windows 11 taskbar treats `TF_LBI_CLK_RIGHT` rests on Mozc's code.
- Whether an application in an AppContainer can read or write config.toml in
  `%APPDATA%`, which following the file and the menu both need.
- The virtual key a Japanese 半角/全角 gives under a Korean profile. The sink
  takes scan 0x29 as 半角/全角 when `GetKeyboardType(0)` is 7, Japanese [25], and
  that value under a Korean layout has not been checked on hardware.
- Whether every Korean IMM32 application delivers modifier-only key events to
  the sink. Right Alt on a type 1 driver arrives as 한/영 and does not depend on it.
- macOS, Android and iOS key handling has been compiled, not typed on.

## Sources

1. Microsoft Learn, [Korean IME](https://learn.microsoft.com/en-us/globalization/input/korean-ime), 2024-06-19.
2. Michael S. Kaplan, [The [underdocumented] world of the Korean keyboard layout](https://archives.miloush.net/michkap/archive/2012/04/30/10298801.html), 2012-04-30. A Microsoft employee's blog, on an archive; old, and it calls the layouts undocumented.
3. KB927824, [Windows may not use the correct keyboard layout…](https://mskb.pkisolutions.com/kb/927824). A third-party mirror of the KB: `LayerDriver KOR` values. It does not name the keys.
4. Microsoft Learn, [Virtual-Key Codes](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes), 2025-10-06.
5. Microsoft Support, [Microsoft Japanese IME](https://support.microsoft.com/windows/microsoft-japanese-ime-da40471d-6b91-4042-ae8b-713a96476916).
6. Microsoft Learn, [Keyboard Japan ImeOn/ImeOff implementation](https://learn.microsoft.com/en-us/windows-hardware/design/component-guidelines/keyboard-japan-ime).
7. MDN, [Keyboard event code values](https://developer.mozilla.org/en-US/docs/Web/API/UI_Events/Keyboard_event_code_values): scan codes 0x70, 0x79, 0x7B, 0x29, 0xE0F1, 0xE0F2.
8. MDN, [Keyboard event key values](https://developer.mozilla.org/en-US/docs/Web/API/UI_Events/Keyboard_event_key_values): `VK_OEM_AUTO` 0xF3, `VK_OEM_ENLW` 0xF4, `VK_OEM_COPY` 0xF2 on Japanese keyboards. Microsoft's virtual-key table lists 0xE9-F5 only as OEM specific.
9. Apple, [Write in another language on Mac](https://support.apple.com/guide/mac-help/mchlp1406/mac) and [Change Input Sources settings on Mac](https://support.apple.com/guide/mac-help/change-input-sources-settings-mchl84525d76/mac).
10. Apple, [Korean Input Method User Guide](https://support.apple.com/guide/korean-input-method/korean-hangul-input-method-user-guide-him001/mac).
11. Apple Developer, [`UIKeyboardHIDUsage.keyboardLANG1`](https://developer.apple.com/documentation/uikit/uikeyboardhidusage/keyboardlang1) and [`keyboardLANG2`](https://developer.apple.com/documentation/uikit/uikeyboardhidusage/keyboardlang2). Written for UIKit; that AppKit's key codes follow them is an inference.
12. Apple, [Switch between keyboards with iPad](https://support.apple.com/guide/ipad/switch-between-keyboards-ipaddd28d7ed/ipados).
13. Nicolapps, [Is it possible to listen to hardware keyboard events from a Custom Keyboard extension?](https://developer.apple.com/forums/thread/650883), Apple Developer Forums, June 2020. One developer's report, no replies.
14. ibus-hangul, [`org.freedesktop.ibus.engine.hangul.gschema.xml`](https://github.com/libhangul/ibus-hangul/blob/main/data/org.freedesktop.ibus.engine.hangul.gschema.xml), `main`.
15. fcitx5-hangul, [`src/engine.h`](https://github.com/fcitx/fcitx5-hangul/blob/master/src/engine.h), `master`.
16. fcitx5, [`globalconfig.cpp`](https://github.com/fcitx/fcitx5/blob/master/src/lib/fcitx/globalconfig.cpp) and [`instance.cpp`](https://github.com/fcitx/fcitx5/blob/master/src/lib/fcitx/instance.cpp), `master`. The behaviour of Hangul and Hanja in a two-item group is also checked by `crates/seokdok-fcitx5/e2e.py` on fcitx5 5.1.7.
17. freedesktop.org, [Bug 84404, update Korean layouts](https://bugs.freedesktop.org/show_bug.cgi?id=84404). Option names in current xkeyboard-config not confirmed.
18. AOSP, [`PhoneWindowManager.java`](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/services/core/java/com/android/server/policy/PhoneWindowManager.java), `main`.
19. AOSP, [`data/keyboards/Generic.kl`](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/data/keyboards/Generic.kl), `main`.
20. Linux, [`input-event-codes.h`](https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h).
21. Samsung, [How to use the supported features in Samsung DeX](https://www.samsung.com/uk/support/apps-services/how-to-use-the-supported-features-in-samsung-dex/). Describes DeX; a [2019 community thread](https://r1.community.samsung.com/t5/%ED%83%9C%EB%B8%94%EB%A6%BF/dex-mode-and-shift-ctrl-space-change-it-to-meta-space/td-p/3049628) reports the setting from One UI 2.
22. Android Central Forums, [External keyboard language switch shortcut not working](https://forums.androidcentral.com/threads/external-keyboard-language-switch-shortcut-not-working.979187). One user's report.
23. Unity Issue Tracker, [Android language toggle with an external keyboard](https://issuetracker.unity3d.com/issues/android-legacy-tmp-and-ui-toolkit-input-field-language-toggle-to-some-non-latin-alphabet-languages-does-not-work-with-external-keyboard).
24. Microsoft Learn, [ITfKeystrokeMgr::PreserveKey](https://learn.microsoft.com/en-us/windows/win32/api/msctf/nf-msctf-itfkeystrokemgr-preservekey).
25. Microsoft Learn, [GetKeyboardType](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getkeyboardtype).
26. Microsoft Learn, [TF_LANGBARITEMINFO](https://learn.microsoft.com/en-us/windows/win32/api/ctfutb/ns-ctfutb-tf_langbariteminfo), 2018-12-05, read 2026-11-04.
27. Microsoft Learn, [ITfLangBarItemMgr::AddItem](https://learn.microsoft.com/en-us/windows/win32/api/ctfutb/nf-ctfutb-itflangbaritemmgr-additem), 2018-12-05, read 2026-11-04, and [Third-party input method editors](https://learn.microsoft.com/en-us/windows/win32/w8cookbook/third-party-input-method-editors), 2018-05-31, written for Windows 8.
28. Microsoft, [SampleIME `LanguageBar.cpp`](https://github.com/microsoft/Windows-classic-samples/blob/main/Samples/IME/cpp/SampleIME/LanguageBar.cpp), `main`.
29. Mozc, [`tip_lang_bar.cc`](https://github.com/google/mozc/blob/master/src/win32/tip/tip_lang_bar.cc) and [`tip_lang_bar_menu.cc`](https://github.com/google/mozc/blob/master/src/win32/tip/tip_lang_bar_menu.cc), `master`. Another input method's code, which shows what it does and does not state that Windows promises it.
30. Microsoft Learn, [ITfLangBarItemButton::InitMenu](https://learn.microsoft.com/en-us/windows/win32/api/ctfutb/nf-ctfutb-itflangbaritembutton-initmenu).
