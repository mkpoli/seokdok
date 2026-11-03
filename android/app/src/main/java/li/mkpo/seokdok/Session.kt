package li.mkpo.seokdok

/** The Kotlin face of the Rust session. */
class Session(dictDir: String, configDir: String) {
    private var handle = Native.create(dictDir, configDir)

    /** A preedit run; [attr] is 0 input, 1 converted, 2 focused segment. */
    data class Run(val text: String, val attr: Int)

    data class Candidate(val text: String, val note: String)

    data class Candidates(val items: List<Candidate>, val selected: Int, val position: Int, val total: Int)

    /** Re-read the settings. */
    fun sync() = Native.sync(handle)

    /** Feed a key; true when the session used it. */
    fun key(kind: Int, ch: Char? = null, shift: Boolean = false): Boolean =
        Native.key(handle, kind, ch?.code ?: 0, shift)

    /** Commit whatever is being composed. */
    fun flush() = Native.flush(handle)

    /** Forget the composition (the text changed under it). */
    fun reset() = Native.reset(handle)

    fun takeCommit(): String? = Native.takeCommit(handle)

    fun preedit(): List<Run> {
        val texts = Native.preeditTexts(handle)
        val attrs = Native.preeditAttrs(handle)
        return texts.indices.map { Run(texts[it], attrs.getOrElse(it) { 0 }) }
    }

    fun candidates(): Candidates? {
        val flat = Native.candidates(handle)
        if (flat.isEmpty()) return null
        val items = (0 until flat.size / 2).map { Candidate(flat[2 * it], flat[2 * it + 1]) }
        val info = Native.candidateInfo(handle)
        return Candidates(items, info[0], info[1], info[2])
    }

    /** 0 釋讀, 1 Hangul, 2 Latin. */
    val mode: Int get() = Native.mode(handle)

    fun close() {
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
