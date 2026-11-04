package li.mkpo.seokdok

/** The Kotlin face of the Rust session. */
class Session {
    /** 0 until [attach] has run (the dictionaries are still being copied) and after [close]. */
    @Volatile
    private var handle = 0L
    private var closed = false
    private var privateField = false

    /** Create the native session once the dictionaries are on disk; any thread. */
    @Synchronized
    fun attach(dictDir: String, configDir: String) {
        if (closed || handle != 0L) return
        val created = Native.create(dictDir, configDir)
        Native.setPrivateField(created, privateField)
        handle = created
    }

    /** The native session exists; keys are used only after this. */
    val attached: Boolean get() = handle != 0L

    /** A preedit run; [attr] is 0 input, 1 converted, 2 focused segment. */
    data class Run(val text: String, val attr: Int)

    data class Candidate(val text: String, val note: String)

    data class Candidates(val items: List<Candidate>, val selected: Int, val position: Int, val total: Int)

    /** Re-read the settings. */
    fun sync() {
        if (handle != 0L) Native.sync(handle)
    }

    /** The field asks not to be learned from. Kept for a session attached later. */
    @Synchronized
    fun setPrivateField(privateField: Boolean) {
        this.privateField = privateField
        if (handle != 0L) Native.setPrivateField(handle, privateField)
    }

    /** Feed a key; true when the session used it. */
    fun key(kind: Int, ch: Char? = null, shift: Boolean = false): Boolean =
        // Before attach the key is swallowed rather than typed as a raw letter.
        handle == 0L || Native.key(handle, kind, ch?.code ?: 0, shift)

    /** Offer a hardware key to the `[input.keys]` lists; true when it was used. */
    fun modeKey(name: String?, mods: Int, state: Int): Boolean =
        handle != 0L && Native.modeKey(handle, name, mods, state)

    /** Commit whatever is being composed. */
    fun flush() {
        if (handle != 0L) Native.flush(handle)
    }

    /** Forget the composition (the text changed under it). */
    fun reset() {
        if (handle != 0L) Native.reset(handle)
    }

    fun takeCommit(): String? = if (handle == 0L) null else Native.takeCommit(handle)

    fun preedit(): List<Run> {
        if (handle == 0L) return emptyList()
        val texts = Native.preeditTexts(handle)
        val attrs = Native.preeditAttrs(handle)
        return texts.indices.map { Run(texts[it], attrs.getOrElse(it) { 0 }) }
    }

    fun candidates(): Candidates? {
        if (handle == 0L) return null
        val flat = Native.candidates(handle)
        if (flat.isEmpty()) return null
        val items = (0 until flat.size / 2).map { Candidate(flat[2 * it], flat[2 * it + 1]) }
        val info = Native.candidateInfo(handle)
        return Candidates(items, info[0], info[1], info[2])
    }

    /** 0 釋讀, 1 Hangul, 2 Latin. */
    val mode: Int get() = if (handle == 0L) HANGUL else Native.mode(handle)

    /** 0 釋讀, 1 國漢文混用, 2 韓㐎 專用. */
    val script: Int get() = Native.script()

    /** The on-screen mode key: 釋 → 混 → 한 → A → 釋. */
    fun modeControl() {
        if (handle != 0L) Native.modeControl(handle)
    }

    @Synchronized
    fun close() {
        closed = true
        if (handle != 0L) {
            Native.free(handle)
            handle = 0L
        }
    }

    companion object {
        const val SEOKDOK = 0
        const val HANGUL = 1
        const val LATIN = 2

        /** The jamo a key types (upper case = Shift layer), for key labels. */
        fun jamo(key: Char): String? {
            val j = Native.keyJamo(key.code)
            return if (j == 0) null else String(Character.toChars(j))
        }

        val ready: Boolean get() = Native.ready()
    }
}
