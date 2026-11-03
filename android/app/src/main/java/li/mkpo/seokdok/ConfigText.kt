package li.mkpo.seokdok

/**
 * The settings as the flat TOML the session hands out ([Native.loadConfig]):
 * `[section]` headers and `key = value` lines, every key present. Values
 * are kept as written, so keys this screen does not show survive a save.
 * crates/seokdok-session/src/config.rs documents every key.
 */
class ConfigText(text: String) {
    private val values = LinkedHashMap<String, String>()

    init {
        var section = ""
        for (raw in text.lines()) {
            val line = raw.trim()
            if (line.isEmpty() || line.startsWith("#")) continue
            if (line.startsWith("[") && line.endsWith("]")) {
                section = line.substring(1, line.length - 1).trim()
                continue
            }
            val eq = line.indexOf('=')
            if (eq < 0) continue
            values["$section.${line.substring(0, eq).trim()}"] = line.substring(eq + 1).trim()
        }
    }

    fun string(key: String): String? = values[key]?.removeSurrounding("\"")

    fun bool(key: String): Boolean? = values[key]?.toBooleanStrictOrNull()

    fun int(key: String): Int? = values[key]?.toIntOrNull()

    fun set(key: String, value: String) {
        values[key] = "\"" + value.replace("\\", "\\\\").replace("\"", "\\\"") + "\""
    }

    fun set(key: String, value: Boolean) {
        values[key] = value.toString()
    }

    fun set(key: String, value: Int) {
        values[key] = value.toString()
    }

    override fun toString(): String = buildString {
        for ((section, entries) in values.entries.groupBy { it.key.substringBefore('.') }) {
            if (section.isNotEmpty()) append("[").append(section).append("]\n")
            for ((k, v) in entries) append(k.substringAfter('.')).append(" = ").append(v).append('\n')
            append('\n')
        }
    }
}
