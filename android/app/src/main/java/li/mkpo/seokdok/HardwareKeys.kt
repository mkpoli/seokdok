package li.mkpo.seokdok

import android.view.KeyEvent

/**
 * Android key codes in the vocabulary of the `[input.keys]` lists.
 *
 * Android's Generic.kl gives the Linux codes KEY_HANGEUL and KEY_HANJA the
 * names KANA and EISU, so those two key codes are 한/영 and 한자 here.
 * KEYCODE_LANGUAGE_SWITCH and Ctrl+Space never arrive: PhoneWindowManager takes
 * them to switch the hardware keyboard layout.
 */
object HardwareKeys {
    private val MODIFIERS = mapOf(
        KeyEvent.KEYCODE_CTRL_LEFT to "LCtrl",
        KeyEvent.KEYCODE_CTRL_RIGHT to "RCtrl",
        KeyEvent.KEYCODE_SHIFT_LEFT to "LShift",
        KeyEvent.KEYCODE_SHIFT_RIGHT to "RShift",
        KeyEvent.KEYCODE_ALT_LEFT to "LAlt",
        KeyEvent.KEYCODE_ALT_RIGHT to "RAlt",
        KeyEvent.KEYCODE_META_LEFT to "LSuper",
        KeyEvent.KEYCODE_META_RIGHT to "RSuper",
    )

    private val NAMED = mapOf(
        KeyEvent.KEYCODE_KANA to "Hangul",
        KeyEvent.KEYCODE_EISU to "Hanja",
        KeyEvent.KEYCODE_ZENKAKU_HANKAKU to "Zenkaku",
        KeyEvent.KEYCODE_HENKAN to "Henkan",
        KeyEvent.KEYCODE_MUHENKAN to "Muhenkan",
        KeyEvent.KEYCODE_KATAKANA_HIRAGANA to "Kana",
        KeyEvent.KEYCODE_SPACE to "Space",
        KeyEvent.KEYCODE_ENTER to "Enter",
        KeyEvent.KEYCODE_NUMPAD_ENTER to "Enter",
        KeyEvent.KEYCODE_TAB to "Tab",
        KeyEvent.KEYCODE_ESCAPE to "Esc",
    )

    fun isModifier(keyCode: Int): Boolean = keyCode in MODIFIERS

    /** The key's name in the lists, or null when they cannot name it. */
    fun name(keyCode: Int): String? = MODIFIERS[keyCode] ?: NAMED[keyCode] ?: when (keyCode) {
        in KeyEvent.KEYCODE_A..KeyEvent.KEYCODE_Z -> ('A' + (keyCode - KeyEvent.KEYCODE_A)).toString()
        in KeyEvent.KEYCODE_0..KeyEvent.KEYCODE_9 -> ('0' + (keyCode - KeyEvent.KEYCODE_0)).toString()
        in KeyEvent.KEYCODE_F1..KeyEvent.KEYCODE_F12 -> "F" + (keyCode - KeyEvent.KEYCODE_F1 + 1)
        else -> null
    }

    fun mods(event: KeyEvent): Int {
        var mods = 0
        if (event.isCtrlPressed) mods = mods or Native.MOD_CTRL
        if (event.isShiftPressed) mods = mods or Native.MOD_SHIFT
        if (event.isAltPressed) mods = mods or Native.MOD_ALT
        if (event.isMetaPressed) mods = mods or Native.MOD_SUPER
        return mods
    }
}
