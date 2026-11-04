package li.mkpo.seokdok

import android.content.Context
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.RectF
import android.graphics.Typeface
import android.os.Handler
import android.os.Looper
import android.util.TypedValue
import android.view.HapticFeedbackConstants
import android.view.MotionEvent
import android.view.View
import android.view.ViewConfiguration

/**
 * One key: the main label and an optional corner hint (the other Shift
 * layer). A key fires on release; with [repeats] it fires
 * on press and keeps firing while held; [onLong] replaces the release when
 * the key is held.
 */
class KeyView(context: Context, private val style: Style = Style.LETTER) : View(context) {
    enum class Style { LETTER, FUNCTION, ACCENT }

    var main: String = ""
        set(v) { field = v; contentDescription = v; invalidate() }
    var hint: String? = null
        set(v) { field = v; invalidate() }
    /** Drawn as switched on (Shift held, Caps Lock). */
    var on = false
        set(v) { field = v; invalidate() }

    var onTap: (() -> Unit)? = null
    var onLong: (() -> Unit)? = null
    var repeats = false

    private val density = resources.displayMetrics.density
    private val gapX = 2.5f * density
    private val gapY = 4f * density
    private val radius = 6f * density
    private val rect = RectF()
    private val bg = Paint(Paint.ANTI_ALIAS_FLAG)
    private val shadow = Paint(Paint.ANTI_ALIAS_FLAG).apply { color = context.getColor(R.color.key_shadow) }
    private val mainPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { textAlign = Paint.Align.CENTER }
    private val hintPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        textAlign = Paint.Align.RIGHT
        textSize = sp(11f)
    }

    private var down = false
    private var longFired = false
    private val timer = Handler(Looper.getMainLooper())
    private val repeat = object : Runnable {
        override fun run() {
            onTap?.invoke()
            timer.postDelayed(this, REPEAT_MS)
        }
    }
    private val long = Runnable {
        longFired = true
        performHapticFeedback(HapticFeedbackConstants.LONG_PRESS)
        onLong?.invoke()
    }

    init {
        isClickable = true
        isFocusable = false
    }

    override fun onDraw(canvas: Canvas) {
        rect.set(gapX, gapY, width - gapX, height - gapY)
        val base = when (style) {
            Style.LETTER -> R.color.key
            Style.FUNCTION -> R.color.key_function
            Style.ACCENT -> R.color.key_accent
        }
        bg.color = context.getColor(if (down) R.color.key_pressed else if (on) R.color.key else base)
        rect.offset(0f, density)
        canvas.drawRoundRect(rect, radius, radius, shadow)
        rect.offset(0f, -density)
        canvas.drawRoundRect(rect, radius, radius, bg)

        mainPaint.color = context.getColor(
            when {
                style == Style.ACCENT -> R.color.key_text_accent
                else -> R.color.key_text
            },
        )
        val wide = main.length > 2
        mainPaint.textSize = sp(if (style == Style.LETTER && !wide) 22f else 15f)
        mainPaint.typeface = if (on) Typeface.DEFAULT_BOLD else Typeface.DEFAULT
        val y = rect.centerY() - (mainPaint.descent() + mainPaint.ascent()) / 2
        canvas.drawText(main, rect.centerX(), y, mainPaint)

        hint?.let {
            hintPaint.color = context.getColor(R.color.key_hint)
            canvas.drawText(it, rect.right - 4 * density, rect.top - hintPaint.ascent() + 1 * density, hintPaint)
        }
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                down = true
                longFired = false
                invalidate()
                performHapticFeedback(HapticFeedbackConstants.KEYBOARD_TAP)
                if (repeats) {
                    onTap?.invoke()
                    timer.postDelayed(repeat, ViewConfiguration.getLongPressTimeout().toLong())
                } else if (onLong != null) {
                    timer.postDelayed(long, ViewConfiguration.getLongPressTimeout().toLong())
                }
            }
            MotionEvent.ACTION_UP -> {
                val fire = down && !repeats && !longFired
                release()
                if (fire) performClick()
            }
            MotionEvent.ACTION_CANCEL -> release()
        }
        return true
    }

    override fun performClick(): Boolean {
        super.performClick()
        onTap?.invoke()
        return true
    }

    override fun onDetachedFromWindow() {
        release()
        super.onDetachedFromWindow()
    }

    private fun release() {
        down = false
        timer.removeCallbacks(repeat)
        timer.removeCallbacks(long)
        invalidate()
    }

    private fun sp(v: Float) = TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_SP, v, resources.displayMetrics)

    private companion object {
        const val REPEAT_MS = 50L
    }
}
