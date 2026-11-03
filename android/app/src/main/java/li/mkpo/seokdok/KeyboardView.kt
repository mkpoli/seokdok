package li.mkpo.seokdok

import android.annotation.SuppressLint
import android.content.Context
import android.graphics.drawable.GradientDrawable
import android.text.TextUtils
import android.util.TypedValue
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.widget.HorizontalScrollView
import android.widget.LinearLayout
import android.widget.TextView

/** What the keys do; implemented by the service. */
interface KeyboardListener {
    fun letter(key: Char)
    fun printable(c: Char)
    fun space()
    fun enter()
    fun backspace()
    fun shift()
    fun toggleMode()
    fun toggleSymbols()
    fun pick(index: Int)
    fun page(forward: Boolean)
    /** Move the focus to the next segment, or with [resize] grow or shrink it. */
    fun segment(forward: Boolean, resize: Boolean)
    fun nextInputMethod()
    fun pickInputMethod()
}

/**
 * The candidate strip and the keys. Each letter key shows the jamo it types
 * and, in its corner, the one on the other Shift layer; old letters
 * (ㆍ ㅿ ㆁ ㆆ ᄼ …) are tinted.
 */
@SuppressLint("ViewConstructor")
class KeyboardView(context: Context, private val listener: KeyboardListener) : LinearLayout(context) {
    /** The keyboard's look for one moment. */
    data class State(
        val shift: Boolean = false,
        val capsLock: Boolean = false,
        val mode: Int = Session.SEOKDOK,
        val symbols: Boolean = false,
        val composing: Boolean = false,
        val enterLabel: String = "↵",
        val globe: Boolean = false,
    )

    private val density = resources.displayMetrics.density
    private val strip = LinearLayout(context)
    private val stripScroll = HorizontalScrollView(context)
    private val stripRow = LinearLayout(context)
    private val stripControls = LinearLayout(context)
    private val keys = LinearLayout(context)
    private val letterKeys = mutableListOf<Pair<Char, KeyView>>()
    private var state = State()
    private var built: Pair<Boolean, Boolean>? = null
    private var shiftKey: KeyView? = null
    private var modeKey: KeyView? = null
    private var spaceKey: KeyView? = null
    private var enterKey: KeyView? = null

    init {
        orientation = VERTICAL
        setBackgroundColor(context.getColor(R.color.kb_background))

        strip.orientation = HORIZONTAL
        strip.gravity = Gravity.CENTER_VERTICAL
        stripScroll.isHorizontalScrollBarEnabled = false
        stripRow.orientation = HORIZONTAL
        stripRow.gravity = Gravity.CENTER_VERTICAL
        stripScroll.addView(stripRow, ViewGroup.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.MATCH_PARENT))
        strip.addView(stripScroll, LayoutParams(0, ViewGroup.LayoutParams.MATCH_PARENT, 1f))
        stripControls.orientation = HORIZONTAL
        strip.addView(stripControls, LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.MATCH_PARENT))
        addView(strip, LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, dp(STRIP_DP)))

        keys.orientation = VERTICAL
        keys.setPadding(dp(2), 0, dp(2), dp(4))
        addView(keys, LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT))
        update(state)
    }

    /** Relabel the keys for the Shift state, the input mode and the layer. */
    fun update(s: State) {
        state = s
        if (built != (s.symbols to s.globe)) build(s)
        val latin = s.mode == Session.LATIN
        for ((ch, key) in letterKeys) {
            val up = ch.uppercaseChar()
            if (latin) {
                key.main = (if (s.shift || s.capsLock) up else ch).toString()
                key.hint = null
                key.old = false
                continue
            }
            val shifted = s.shift || s.capsLock
            val main = Session.jamo(if (shifted) up else ch) ?: ch.toString()
            val other = Session.jamo(if (shifted) ch else up)
            val hint = other.takeIf { it != main }
            key.main = shown(main)
            key.hint = hint?.let { shown(it) }
            key.old = isOld(main) || isOld(hint ?: "")
        }
        shiftKey?.on = s.shift || s.capsLock
        shiftKey?.main = if (s.capsLock) "⇪" else "⇧"
        modeKey?.main = when (s.mode) {
            Session.LATIN -> "A"
            Session.HANGUL -> "한"
            else -> "韓"
        }
        spaceKey?.main = when {
            s.composing && s.mode == Session.SEOKDOK -> "變換"
            s.mode == Session.LATIN -> "Latin"
            s.mode == Session.HANGUL -> "한글"
            else -> "釋讀"
        }
        enterKey?.main = if (s.composing) "確定" else s.enterLabel
    }

    /** Fill the strip: the candidates when the list is open, else the text being composed. */
    fun show(candidates: Session.Candidates?, preedit: List<Session.Run>, loading: Boolean) {
        stripRow.removeAllViews()
        stripControls.removeAllViews()
        if (candidates != null) {
            candidates.items.forEachIndexed { i, item ->
                stripRow.addView(candidateCell(item, i == candidates.selected) { listener.pick(i) })
            }
            stripRow.addView(stripText("${candidates.position}/${candidates.total}", 12f, R.color.strip_note))
            if (candidates.total > candidates.items.size) {
                stripControls.addView(control("‹") { listener.page(false) })
                stripControls.addView(control("›") { listener.page(true) })
            }
        } else {
            val text = if (loading && preedit.isEmpty()) context.getString(R.string.loading) else preedit.joinToString("") { it.text }
            stripRow.addView(stripText(text, 18f, R.color.strip_note))
        }
        // A converted phrase in several segments: walk them (hold to resize).
        if (preedit.count { it.attr != 0 } > 1) {
            stripControls.addView(control("◂", { listener.segment(false, true) }) { listener.segment(false, false) })
            stripControls.addView(control("▸", { listener.segment(true, true) }) { listener.segment(true, false) })
        }
        stripScroll.post { stripScroll.scrollTo(0, 0) }
    }

    private fun build(s: State) {
        built = s.symbols to s.globe
        keys.removeAllViews()
        letterKeys.clear()
        shiftKey = null
        if (s.symbols) {
            keys.addView(row(SYMBOLS[0].map { symbolKey(it) to 1f }))
            keys.addView(row(SYMBOLS[1].map { symbolKey(it) to 1f }))
            keys.addView(row(SYMBOLS[2].map { symbolKey(it) to 1f } + (backspaceKey() to 1.5f)))
        } else {
            keys.addView(row(LETTERS[0].map { letterKey(it) to 1f }))
            keys.addView(row(LETTERS[1].map { letterKey(it) to 1f }, inset = 0.5f))
            val shift = KeyView(context, KeyView.Style.FUNCTION).apply {
                main = "⇧"
                onTap = listener::shift
            }
            shiftKey = shift
            keys.addView(row(listOf(shift to 1.5f) + LETTERS[2].map { letterKey(it) to 1f } + (backspaceKey() to 1.5f)))
        }

        val bottom = mutableListOf<Pair<View, Float>>()
        bottom += KeyView(context, KeyView.Style.FUNCTION).apply {
            main = if (s.symbols) "가" else "123"
            onTap = listener::toggleSymbols
        } to 1.5f
        if (s.globe) {
            bottom += KeyView(context, KeyView.Style.FUNCTION).apply {
                main = "⇆"
                contentDescription = "Next input method"
                onTap = listener::nextInputMethod
                onLong = listener::pickInputMethod
            } to 1f
        }
        bottom += KeyView(context, KeyView.Style.FUNCTION).apply {
            onTap = listener::toggleMode
            modeKey = this
        } to 1.25f
        bottom += KeyView(context).apply {
            onTap = listener::space
            onLong = listener::pickInputMethod
            spaceKey = this
        } to (if (s.globe) 4f else 5f)
        bottom += symbolKey('.') to 1f
        bottom += KeyView(context, KeyView.Style.ACCENT).apply {
            onTap = listener::enter
            enterKey = this
        } to 1.75f
        keys.addView(row(bottom))
    }

    private fun letterKey(ch: Char) = KeyView(context).apply {
        onTap = { listener.letter(ch) }
        letterKeys += ch to this
    }

    private fun symbolKey(c: Char) = KeyView(context).apply {
        main = c.toString()
        onTap = { listener.printable(c) }
    }

    private fun backspaceKey() = KeyView(context, KeyView.Style.FUNCTION).apply {
        main = "⌫"
        contentDescription = "Backspace"
        repeats = true
        onTap = listener::backspace
    }

    /** A row of keys by weight; [inset] keeps that many key widths empty, half on each side. */
    private fun row(items: List<Pair<View, Float>>, inset: Float = 0f) = LinearLayout(context).apply {
        orientation = HORIZONTAL
        if (inset > 0) addView(View(context), LayoutParams(0, dp(KEY_DP), inset))
        for ((v, w) in items) addView(v, LayoutParams(0, dp(KEY_DP), w))
        if (inset > 0) addView(View(context), LayoutParams(0, dp(KEY_DP), inset))
    }

    private fun candidateCell(item: Session.Candidate, selected: Boolean, tap: () -> Unit) = LinearLayout(context).apply {
        orientation = VERTICAL
        gravity = Gravity.CENTER
        setPadding(dp(10), 0, dp(10), 0)
        if (selected) {
            background = GradientDrawable().apply {
                cornerRadius = 6f * density
                setColor(context.getColor(R.color.strip_selected))
            }
        }
        addView(stripText(item.text, 20f, R.color.strip_text, padded = false))
        if (item.note.isNotEmpty()) {
            addView(stripText(item.note, 10f, R.color.strip_note, padded = false).apply { maxWidth = dp(160) })
        }
        isClickable = true
        setOnClickListener {
            performHapticFeedback(android.view.HapticFeedbackConstants.KEYBOARD_TAP)
            tap()
        }
        layoutParams = LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.MATCH_PARENT).apply {
            setMargins(dp(2), dp(3), dp(2), dp(3))
        }
    }

    private fun stripText(text: String, sp: Float, color: Int, padded: Boolean = true) = TextView(context).apply {
        this.text = text
        setTextSize(TypedValue.COMPLEX_UNIT_SP, sp)
        setTextColor(context.getColor(color))
        maxLines = 1
        ellipsize = TextUtils.TruncateAt.END
        gravity = Gravity.CENTER_VERTICAL
        if (padded) setPadding(dp(10), 0, dp(10), 0)
    }

    private fun control(label: String, long: (() -> Unit)? = null, tap: () -> Unit) =
        KeyView(context, KeyView.Style.FUNCTION).apply {
            main = label
            onTap = tap
            onLong = long
            layoutParams = LayoutParams(dp(40), ViewGroup.LayoutParams.MATCH_PARENT)
        }

    /** An initial-only jamo (ᄼ) needs a vowel filler to show on its own. */
    private fun shown(j: String): String = if (j.isNotEmpty() && j[0].code in 0x1100..0x115F) j + "ᅠ" else j

    private fun isOld(j: String): Boolean {
        if (j.isEmpty()) return false
        return j[0] in "ㆍㅿㆁㆆ" || j[0].code in 0x1100..0x115F
    }

    private fun dp(v: Int) = (v * density).toInt()

    private companion object {
        const val STRIP_DP = 46
        const val KEY_DP = 52
        val LETTERS = listOf("qwertyuiop", "asdfghjkl", "zxcvbnm")
        val SYMBOLS = listOf("1234567890", "@#%&*-+=()", "~,?!'\"/:;")
    }
}
