import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import javax.inject.Inject

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

/** The repository root: the Cargo workspace and `dict/`. */
val repoRoot: File = rootProject.projectDir.parentFile

android {
    namespace = "li.mkpo.seokdok"
    compileSdk = 35
    // r28 links with 16 KB pages by default (Android 15 devices).
    ndkVersion = "28.0.13004108"

    defaultConfig {
        applicationId = "li.mkpo.seokdok"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
    }

    // The release key comes from the environment (set by the CI workflow from
    // repository secrets). Without SEOKDOK_KEYSTORE, `assembleRelease` yields an
    // unsigned APK, which cannot be installed.
    val keystore: String? = System.getenv("SEOKDOK_KEYSTORE")
    signingConfigs {
        if (keystore != null) {
            create("release") {
                storeFile = file(keystore)
                // A PKCS12 keystore has one password, for the store and its key.
                storePassword = System.getenv("SEOKDOK_KEYSTORE_PASSWORD")
                keyAlias = System.getenv("SEOKDOK_KEY_ALIAS")
                keyPassword = storePassword
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            if (keystore != null) signingConfig = signingConfigs.getByName("release")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

}

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_17)
    }
}

/** Builds `libseokdok_jni.so` for every ABI with cargo-ndk, laid out as jniLibs. */
abstract class CargoNdk : DefaultTask() {
    @get:InputFiles
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val sources: ConfigurableFileCollection

    @get:Internal
    abstract val workspace: DirectoryProperty

    @get:Internal
    abstract val ndkDirectory: DirectoryProperty

    @get:Input
    abstract val abis: ListProperty<String>

    @get:Input
    abstract val platform: Property<Int>

    @get:OutputDirectory
    abstract val outputDir: DirectoryProperty

    @get:Inject
    abstract val execOps: ExecOperations

    @TaskAction
    fun build() {
        val out = outputDir.get().asFile
        out.deleteRecursively()
        execOps.exec {
            workingDir = workspace.get().asFile
            val args = mutableListOf("cargo", "ndk", "--platform", platform.get().toString(), "--output-dir", out.path)
            abis.get().forEach { args += listOf("--target", it) }
            // Release even for debug APKs: an unoptimised engine loads the
            // dictionaries many times slower.
            args += listOf("build", "--release", "-p", "seokdok-jni")
            commandLine(args)
            environment("ANDROID_NDK_HOME", ndkDirectory.get().asFile.path)
            // 16 KB pages (Android 15 devices): align the library's segments.
            val pages = "-C link-arg=-Wl,-z,max-page-size=16384"
            environment("CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS", pages)
            environment("CARGO_TARGET_X86_64_LINUX_ANDROID_RUSTFLAGS", pages)
        }
    }
}

/** Copies the dictionaries in `dict/` into the APK's assets under `dict/`. */
abstract class DictAssets : DefaultTask() {
    @get:InputFiles
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val dictionaries: ConfigurableFileCollection

    @get:OutputDirectory
    abstract val outputDir: DirectoryProperty

    @TaskAction
    fun copy() {
        val dir = outputDir.get().asFile.resolve("dict")
        outputDir.get().asFile.deleteRecursively()
        dir.mkdirs()
        dictionaries.files.forEach { it.copyTo(dir.resolve(it.name), overwrite = true) }
    }
}

val dictAssets = tasks.register<DictAssets>("dictAssets") {
    dictionaries.from(fileTree(repoRoot.resolve("dict")) {
        include("*.tsv", "LICENSE*")
        exclude("user*.tsv")
    })
    outputDir.set(layout.buildDirectory.dir("generated/seokdok/assets"))
}

androidComponents {
    onVariants { variant ->
        val cargo = tasks.register<CargoNdk>("cargoNdk${variant.name.replaceFirstChar(Char::uppercase)}") {
            sources.from(
                fileTree(repoRoot.resolve("crates")) { include("**/*.rs", "**/Cargo.toml") },
                repoRoot.resolve("Cargo.toml"),
                repoRoot.resolve("Cargo.lock"),
            )
            workspace.set(repoRoot)
            // The pinned NDK; sdkComponents.ndkDirectory has no value without an
            // externalNativeBuild block.
            ndkDirectory.set(sdkComponents.sdkDirectory.map { it.dir("ndk/${android.ndkVersion}") })
            abis.set(listOf("arm64-v8a", "armeabi-v7a", "x86_64"))
            platform.set(26)
            outputDir.set(layout.buildDirectory.dir("generated/seokdok/jniLibs/${variant.name}"))
        }
        variant.sources.jniLibs?.addGeneratedSourceDirectory(cargo, CargoNdk::outputDir)
        variant.sources.assets?.addGeneratedSourceDirectory(dictAssets, DictAssets::outputDir)
    }
}
