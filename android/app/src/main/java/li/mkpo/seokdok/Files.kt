package li.mkpo.seokdok

import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import java.io.File

/** Where the session reads its files from. */
object Files {
    /** config.toml, custom.json and user.tsv. */
    fun configDir(context: Context): File = File(context.filesDir, "config").apply { mkdirs() }

    /**
     * The dictionaries, copied out of the APK's assets on first use and again
     * after each install or update (the engine reads them as files).
     */
    @Synchronized
    fun dictDir(context: Context): File {
        val dir = File(context.filesDir, "dict")
        val stamp = File(dir, ".installed")
        val want = installStamp(context)
        if (stamp.isFile && stamp.readText() == want) return dir

        val tmp = File(context.filesDir, "dict.tmp")
        tmp.deleteRecursively()
        tmp.mkdirs()
        val assets = context.assets
        for (name in assets.list("dict").orEmpty()) {
            assets.open("dict/$name").use { input ->
                File(tmp, name).outputStream().use { input.copyTo(it) }
            }
        }
        File(tmp, ".installed").writeText(want)
        dir.deleteRecursively()
        if (!tmp.renameTo(dir)) error("cannot move $tmp to $dir")
        return dir
    }

    private fun installStamp(context: Context): String {
        val pm = context.packageManager
        val info = if (Build.VERSION.SDK_INT >= 33) {
            pm.getPackageInfo(context.packageName, PackageManager.PackageInfoFlags.of(0))
        } else {
            @Suppress("DEPRECATION")
            pm.getPackageInfo(context.packageName, 0)
        }
        return info.lastUpdateTime.toString()
    }
}
