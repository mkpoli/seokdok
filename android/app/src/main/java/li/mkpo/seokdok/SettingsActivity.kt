package li.mkpo.seokdok

import android.app.Activity
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import android.text.Editable
import android.text.InputType
import android.text.TextWatcher
import android.util.TypedValue
import android.view.View
import android.view.ViewGroup
import android.view.WindowInsets
import android.view.inputmethod.InputMethodManager
import android.widget.AdapterView
import android.widget.ArrayAdapter
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.Spinner
import android.widget.Switch
import android.widget.TextView
import android.widget.Toast
import java.io.File

/**
 * How to turn the keyboard on, and its settings, written as config.toml
 * where the keyboard's session reads it.
 */
class SettingsActivity : Activity() {
    private lateinit var config: ConfigText
    private lateinit var configDir: File
    private lateinit var customStatus: TextView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        configDir = Files.configDir(this)
        config = ConfigText(Native.loadConfig(configDir.path))

        val column = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dp(16), dp(8), dp(16), dp(24))
        }

        column.addView(header(R.string.section_enable))
        column.addView(body(getString(R.string.enable_help)))
        column.addView(button(R.string.enable_open) { startActivity(Intent(Settings.ACTION_INPUT_METHOD_SETTINGS)) })
        column.addView(button(R.string.enable_pick) {
            (getSystemService(INPUT_METHOD_SERVICE) as InputMethodManager).showInputMethodPicker()
        })
        column.addView(EditText(this).apply {
            hint = getString(R.string.try_hint)
            minLines = 2
        })

        column.addView(header(R.string.section_input))
        column.addView(choice(R.string.default_mode, R.array.mode_labels, "input.default_mode", listOf("seokdok", "hangul", "latin")))
        column.addView(toggle(R.string.space_converts, "input.space_converts"))
        column.addView(toggle(R.string.old_clusters, "input.old_clusters"))

        column.addView(header(R.string.section_conversion))
        column.addView(toggle(R.string.sino_only, "conversion.sino_only"))
        column.addView(choice(R.string.style, R.array.style_labels, "conversion.style", listOf("surface", "morpheme")))
        column.addView(choice(R.string.fragment, R.array.fragment_labels, "conversion.fragment", listOf("absorb", "letter")))
        column.addView(toggle(R.string.coda_always, "conversion.coda_always"))
        column.addView(toggle(R.string.resyllabify, "conversion.resyllabify"))
        column.addView(toggle(R.string.hada_kun, "conversion.hada_kun"))
        column.addView(number(R.string.max_candidates, "conversion.max_candidates"))

        column.addView(header(R.string.section_custom))
        column.addView(body(getString(R.string.custom_help)))
        customStatus = body("")
        column.addView(customStatus)
        column.addView(button(R.string.custom_import) {
            val pick = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
                addCategory(Intent.CATEGORY_OPENABLE)
                type = "*/*"
                putExtra(Intent.EXTRA_MIME_TYPES, arrayOf("application/json", "text/plain", "application/octet-stream"))
            }
            startActivityForResult(pick, PICK_CUSTOM)
        })
        column.addView(button(R.string.custom_remove) {
            File(configDir, CUSTOM).delete()
            customChanged()
        })
        showCustom()

        column.addView(body("https://seokdok.mkpo.li/").apply { setTextIsSelectable(true) })

        val scroll = ScrollView(this).apply { addView(column) }
        scroll.setOnApplyWindowInsetsListener { v, insets ->
            if (Build.VERSION.SDK_INT >= 30) {
                val bars = insets.getInsets(WindowInsets.Type.systemBars() or WindowInsets.Type.ime())
                v.setPadding(bars.left, bars.top, bars.right, bars.bottom)
            } else {
                @Suppress("DEPRECATION")
                v.setPadding(insets.systemWindowInsetLeft, insets.systemWindowInsetTop, insets.systemWindowInsetRight, insets.systemWindowInsetBottom)
            }
            insets
        }
        setContentView(scroll)
        save()
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        val uri = data?.data
        if (requestCode != PICK_CUSTOM || resultCode != RESULT_OK || uri == null) return
        val tmp = File(configDir, "$CUSTOM.tmp")
        val copied = runCatching {
            contentResolver.openInputStream(uri)!!.use { input -> tmp.outputStream().use { input.copyTo(it) } }
            tmp.renameTo(File(configDir, CUSTOM))
        }.getOrDefault(false)
        if (!copied) tmp.delete()
        customChanged()
    }

    /** Write config.toml; the keyboard reads it on the next text field. */
    private fun save() {
        if (!Native.saveConfig(configDir.path, config.toString())) {
            Toast.makeText(this, R.string.saved_failed, Toast.LENGTH_SHORT).show()
        }
    }

    private fun customChanged() {
        showCustom()
        Native.reload(Files.dictDir(this).path)
    }

    private fun showCustom() {
        customStatus.setText(if (File(configDir, CUSTOM).isFile) R.string.custom_present else R.string.custom_absent)
    }

    private fun toggle(label: Int, key: String) = Switch(this).apply {
        setText(label)
        isChecked = config.bool(key) ?: false
        setPadding(0, dp(10), 0, dp(10))
        setOnCheckedChangeListener { _, on ->
            config.set(key, on)
            save()
        }
    }

    private fun choice(label: Int, labels: Int, key: String, values: List<String>) = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        setPadding(0, dp(8), 0, dp(8))
        addView(body(getString(label)))
        addView(Spinner(this@SettingsActivity).apply {
            adapter = ArrayAdapter(
                this@SettingsActivity,
                android.R.layout.simple_spinner_dropdown_item,
                resources.getStringArray(labels),
            )
            setSelection(values.indexOf(config.string(key)).coerceAtLeast(0), false)
            onItemSelectedListener = object : AdapterView.OnItemSelectedListener {
                override fun onItemSelected(parent: AdapterView<*>?, view: View?, position: Int, id: Long) {
                    if (config.string(key) == values[position]) return
                    config.set(key, values[position])
                    save()
                }

                override fun onNothingSelected(parent: AdapterView<*>?) {}
            }
        })
    }

    private fun number(label: Int, key: String) = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        setPadding(0, dp(8), 0, dp(8))
        addView(body(getString(label)))
        addView(EditText(this@SettingsActivity).apply {
            inputType = InputType.TYPE_CLASS_NUMBER
            setText((config.int(key) ?: 1).toString())
            addTextChangedListener(object : TextWatcher {
                override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) {}
                override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {}
                override fun afterTextChanged(s: Editable?) {
                    val n = s?.toString()?.toIntOrNull()?.takeIf { it >= 1 } ?: return
                    config.set(key, n)
                    save()
                }
            })
        })
    }

    private fun header(text: Int) = TextView(this).apply {
        setText(text)
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 20f)
        setPadding(0, dp(20), 0, dp(6))
    }

    private fun body(text: String) = TextView(this).apply {
        this.text = text
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
        setPadding(0, dp(4), 0, dp(4))
    }

    private fun button(text: Int, action: () -> Unit) = Button(this).apply {
        setText(text)
        isAllCaps = false
        setOnClickListener { action() }
        layoutParams = LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT)
    }

    private fun dp(v: Int) = (v * resources.displayMetrics.density).toInt()

    private companion object {
        const val PICK_CUSTOM = 1
        const val CUSTOM = "custom.json"
    }
}
