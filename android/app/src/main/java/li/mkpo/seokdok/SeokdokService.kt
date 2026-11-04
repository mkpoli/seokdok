package li.mkpo.seokdok

import android.inputmethodservice.InputMethodService
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import android.text.InputType
import android.text.SpannableString
import android.text.Spanned
import android.text.style.BackgroundColorSpan
import android.text.style.UnderlineSpan
import android.util.Log
import android.view.KeyEvent
import android.view.View
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputMethodManager

/**
 * The 釋讀 keyboard. Keys go to the Rust session; the text being composed
 * is the editor's composing text (setComposingText), confirmed text is
 * committed, and the candidates fill the strip above the keys.
 */
class SeokdokService : InputMethodService(), KeyboardListener {
    private lateinit var session: Session
    private var keyboard: KeyboardView? = null
    private var state = KeyboardView.State()
    /** Composing text is in the editor. */
    private var composing = false
    /** Password, number, e-mail and URL fields: keys go straight through. */
    private var passthrough = false
    private var lastShiftTap = 0L
    /** The dictionaries are on disk and the engine has loaded them. */
    private val loaded: Boolean get() = session.attached && Session.ready
    private val timer = Handler(Looper.getMainLooper())
    private val pollReady = object : Runnable {
        override fun run() {
            if (loaded) refresh() else timer.postDelayed(this, 200)
        }
    }

    override fun onCreate() {
        super.onCreate()
        session = Session()
        // The first run (and each update) copies the dictionaries, about 20 MB;
        // that happens off the main thread. The strip shows "loading" and keys
        // are held back until the native session exists.
        val started = SystemClock.elapsedRealtime()
        Thread({
            val dict = Files.dictDir(this).path
            val config = Files.configDir(this).path
            val copied = SystemClock.elapsedRealtime() - started
            session.attach(dict, config)
            Log.i(TAG, "dictionaries ready after $copied ms, session attached")
            timer.post {
                session.sync()
                if (keyboard != null) refresh()
                if (!loaded) timer.post(pollReady)
            }
        }, "seokdok-dict").start()
    }

    override fun onDestroy() {
        timer.removeCallbacksAndMessages(null)
        session.close()
        super.onDestroy()
    }

    override fun onCreateInputView(): View {
        val view = KeyboardView(this, this)
        keyboard = view
        return view
    }

    /** In landscape the system would replace the app with its own light extract editor, also at night. */
    override fun onEvaluateFullscreenMode(): Boolean = false

    override fun onStartInput(attribute: EditorInfo, restarting: Boolean) {
        super.onStartInput(attribute, restarting)
        if (!restarting) {
            session.reset()
            composing = false
        }
        val cls = attribute.inputType and InputType.TYPE_MASK_CLASS
        val variation = attribute.inputType and InputType.TYPE_MASK_VARIATION
        val numeric = cls == InputType.TYPE_CLASS_NUMBER || cls == InputType.TYPE_CLASS_PHONE || cls == InputType.TYPE_CLASS_DATETIME
        val latinText = cls == InputType.TYPE_CLASS_TEXT && variation in LATIN_VARIATIONS
        passthrough = numeric || latinText
        state = KeyboardView.State(symbols = numeric)
    }

    override fun onStartInputView(info: EditorInfo, restarting: Boolean) {
        super.onStartInputView(info, restarting)
        session.sync()
        state = state.copy(shift = false, capsLock = false, enterLabel = enterLabel(info), globe = offersSwitching())
        refresh()
        if (!loaded) timer.post(pollReady)
    }

    override fun onFinishInputView(finishingInput: Boolean) {
        timer.removeCallbacks(pollReady)
        commitComposition()
        super.onFinishInputView(finishingInput)
    }

    override fun onFinishInput() {
        commitComposition()
        super.onFinishInput()
    }

    override fun onUpdateSelection(
        oldSelStart: Int,
        oldSelEnd: Int,
        newSelStart: Int,
        newSelEnd: Int,
        candidatesStart: Int,
        candidatesEnd: Int,
    ) {
        super.onUpdateSelection(oldSelStart, oldSelEnd, newSelStart, newSelEnd, candidatesStart, candidatesEnd)
        // The cursor left the composing text (a tap elsewhere): keep the text
        // as shown and start afresh.
        if (composing && candidatesEnd >= 0 && (newSelStart != candidatesEnd || newSelEnd != candidatesEnd)) {
            session.reset()
            currentInputConnection?.finishComposingText()
            composing = false
            refresh()
        }
    }

    private fun commitComposition() {
        if (!composing) return
        session.flush()
        render()
    }

    /** Write the session's view into the editor and the strip. */
    private fun render() {
        val ic = currentInputConnection
        val commit = session.takeCommit()
        val runs = session.preedit()
        if (ic != null) {
            ic.beginBatchEdit()
            if (commit != null) {
                // Replaces the composing text, if any.
                ic.commitText(commit, 1)
                composing = false
            }
            if (runs.isNotEmpty()) {
                ic.setComposingText(styled(runs), 1)
                composing = true
            } else if (composing) {
                ic.setComposingText("", 1)
                ic.finishComposingText()
                composing = false
            }
            ic.endBatchEdit()
        }
        refresh(runs)
    }

    private fun refresh(runs: List<Session.Run>? = null) {
        val kb = keyboard ?: return
        val preedit = runs ?: session.preedit()
        state = state.copy(mode = if (passthrough) Session.LATIN else session.mode, composing = preedit.isNotEmpty())
        kb.update(state)
        kb.show(session.candidates(), preedit, loading = !loaded && !passthrough)
    }

    /** The preedit underlined, the segment being converted highlighted. */
    private fun styled(runs: List<Session.Run>): CharSequence {
        val s = SpannableString(runs.joinToString("") { it.text })
        s.setSpan(UnderlineSpan(), 0, s.length, Spanned.SPAN_INCLUSIVE_INCLUSIVE or Spanned.SPAN_COMPOSING)
        var at = 0
        for (r in runs) {
            if (r.attr == 2 && r.text.isNotEmpty()) {
                s.setSpan(BackgroundColorSpan(getColor(R.color.preedit_focus)), at, at + r.text.length, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
            }
            at += r.text.length
        }
        return s
    }

    private fun feed(kind: Int, ch: Char? = null, shift: Boolean = false): Boolean =
        !passthrough && session.key(kind, ch, shift)

    private fun commitText(text: String) {
        currentInputConnection?.commitText(text, 1)
    }

    // KeyboardListener

    override fun letter(key: Char) {
        val shifted = state.shift || state.capsLock
        if (!feed(Native.LETTER, key, shifted)) commitText((if (shifted) key.uppercaseChar() else key).toString())
        if (state.shift && !state.capsLock) state = state.copy(shift = false)
        render()
    }

    override fun printable(c: Char) {
        if (!feed(Native.PRINTABLE, c)) commitText(c.toString())
        render()
    }

    override fun space() {
        if (!feed(Native.SPACE)) commitText(" ")
        render()
    }

    override fun enter() {
        if (!feed(Native.ENTER)) {
            val info = currentInputEditorInfo
            val action = info?.let { it.imeOptions and EditorInfo.IME_MASK_ACTION } ?: EditorInfo.IME_ACTION_NONE
            val noAction = info == null || info.imeOptions and EditorInfo.IME_FLAG_NO_ENTER_ACTION != 0
            if (info?.actionLabel != null) {
                // The editor's own action, labelled by the editor.
                currentInputConnection?.performEditorAction(info.actionId)
            } else if (!noAction && action != EditorInfo.IME_ACTION_NONE && action != EditorInfo.IME_ACTION_UNSPECIFIED) {
                currentInputConnection?.performEditorAction(action)
            } else {
                sendKeyChar('\n')
            }
        }
        render()
    }

    override fun backspace() {
        if (!feed(Native.BACKSPACE)) sendDownUpKeyEvents(KeyEvent.KEYCODE_DEL)
        render()
    }

    override fun shift() {
        val now = SystemClock.uptimeMillis()
        state = when {
            state.capsLock -> state.copy(shift = false, capsLock = false)
            state.shift && now - lastShiftTap < DOUBLE_TAP_MS -> state.copy(shift = false, capsLock = true)
            else -> state.copy(shift = !state.shift)
        }
        lastShiftTap = now
        refresh()
    }

    override fun toggleMode() {
        if (passthrough) return
        session.key(Native.TOGGLE)
        render()
    }

    override fun toggleSymbols() {
        state = state.copy(symbols = !state.symbols, shift = false, capsLock = false)
        refresh()
    }

    /** Candidate [index] on the shown page, picked as its digit key. */
    override fun pick(index: Int) {
        if (index !in 0 until 9) return
        session.key(Native.PRINTABLE, '1' + index)
        render()
    }

    override fun page(forward: Boolean) {
        session.key(if (forward) Native.PAGE_DOWN else Native.PAGE_UP)
        render()
    }

    override fun segment(forward: Boolean, resize: Boolean) {
        session.key(if (forward) Native.RIGHT else Native.LEFT, shift = resize)
        render()
    }

    override fun nextInputMethod() {
        if (Build.VERSION.SDK_INT >= 28) {
            switchToNextInputMethod(false)
        } else {
            @Suppress("DEPRECATION")
            imm().switchToNextInputMethod(window.window?.attributes?.token, false)
        }
    }

    override fun pickInputMethod() {
        imm().showInputMethodPicker()
    }

    private fun imm() = getSystemService(INPUT_METHOD_SERVICE) as InputMethodManager

    private fun offersSwitching(): Boolean =
        if (Build.VERSION.SDK_INT >= 28) {
            shouldOfferSwitchingToNextInputMethod()
        } else {
            @Suppress("DEPRECATION")
            imm().shouldOfferSwitchingToNextInputMethod(window.window?.attributes?.token)
        }

    private fun enterLabel(info: EditorInfo): String {
        if (info.imeOptions and EditorInfo.IME_FLAG_NO_ENTER_ACTION != 0) return "↵"
        return when (info.imeOptions and EditorInfo.IME_MASK_ACTION) {
            EditorInfo.IME_ACTION_GO, EditorInfo.IME_ACTION_SEND -> "→"
            EditorInfo.IME_ACTION_SEARCH -> "⌕"
            EditorInfo.IME_ACTION_NEXT -> "⇥"
            EditorInfo.IME_ACTION_DONE -> "✓"
            else -> "↵"
        }
    }

    private companion object {
        const val TAG = "Seokdok"
        const val DOUBLE_TAP_MS = 300L
        val LATIN_VARIATIONS = setOf(
            InputType.TYPE_TEXT_VARIATION_PASSWORD,
            InputType.TYPE_TEXT_VARIATION_VISIBLE_PASSWORD,
            InputType.TYPE_TEXT_VARIATION_WEB_PASSWORD,
            InputType.TYPE_TEXT_VARIATION_EMAIL_ADDRESS,
            InputType.TYPE_TEXT_VARIATION_WEB_EMAIL_ADDRESS,
            InputType.TYPE_TEXT_VARIATION_URI,
        )
    }
}
