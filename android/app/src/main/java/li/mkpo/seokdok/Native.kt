package li.mkpo.seokdok

/**
 * The Rust session (crates/seokdok-jni over crates/seokdok-ffi). A handle is
 * the session pointer; free it once.
 */
object Native {
    init {
        System.loadLibrary("seokdok_jni")
    }

    // Key kinds, as in crates/seokdok-ffi/include/seokdok.h.
    const val LETTER = 1
    const val PRINTABLE = 2
    const val SPACE = 3
    const val ENTER = 4
    const val ESC = 5
    const val BACKSPACE = 6
    const val TAB = 7
    const val LEFT = 8
    const val RIGHT = 9
    const val UP = 10
    const val DOWN = 11
    const val PAGE_UP = 12
    const val PAGE_DOWN = 13
    const val CONVERT = 14
    const val OTHER = 16

    // Key states and modifier flags for modeKey, as in seokdok.h.
    const val KEY_UP = 0
    const val KEY_DOWN = 1
    const val KEY_REPEAT = 2
    const val MOD_CTRL = 1
    const val MOD_SHIFT = 2
    const val MOD_ALT = 4
    const val MOD_SUPER = 8

    @JvmStatic external fun create(dictDir: String, configDir: String): Long
    @JvmStatic external fun free(handle: Long)
    @JvmStatic external fun sync(handle: Long)
    @JvmStatic external fun key(handle: Long, kind: Int, ch: Int, shift: Boolean): Boolean
    /** Offer a key to the `[input.keys]` lists; [name] is null for a key they cannot name. */
    @JvmStatic external fun modeKey(handle: Long, name: String?, mods: Int, state: Int): Boolean
    @JvmStatic external fun flush(handle: Long)
    @JvmStatic external fun reset(handle: Long)
    @JvmStatic external fun takeCommit(handle: Long): String?
    @JvmStatic external fun preeditTexts(handle: Long): Array<String>
    @JvmStatic external fun preeditAttrs(handle: Long): IntArray
    /** Text, note, text, note, … for the current page. */
    @JvmStatic external fun candidates(handle: Long): Array<String>
    /** Selected index, 1-based position, total, horizontal (0/1). */
    @JvmStatic external fun candidateInfo(handle: Long): IntArray
    @JvmStatic external fun mode(handle: Long): Int
    /** The on-screen mode key: 釋 → 混 → 한 → A → 釋. */
    @JvmStatic external fun modeControl(handle: Long)
    /** 0 釋讀, 1 國漢文混用, 2 韓㐎 專用. */
    @JvmStatic external fun script(): Int
    @JvmStatic external fun keyJamo(key: Int): Int
    @JvmStatic external fun ready(): Boolean
    @JvmStatic external fun loadConfig(dir: String): String
    @JvmStatic external fun saveConfig(dir: String, toml: String): Boolean
    /** Reload the dictionaries and custom.json, if the keyboard loaded them in this process. */
    @JvmStatic external fun reload(dictDir: String)
}
