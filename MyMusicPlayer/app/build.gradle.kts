plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "com.mymusicplayer"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.mymusicplayer"
        minSdk = 35
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

dependencies {
    // Compose BOMで各Composeライブラリのバージョンを一括管理する
    val composeBom = platform("androidx.compose:compose-bom:2026.06.00")
    implementation(composeBom)

    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.activity:activity-compose:1.13.0")
    implementation("androidx.navigation:navigation-compose:2.9.7")
    implementation("androidx.media3:media3-exoplayer:1.10.0")
    implementation("androidx.media3:media3-common:1.10.0")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.10.0")
    implementation("androidx.media3:media3-session:1.10.0")
    implementation("org.msgpack:msgpack-core:0.9.11")

    debugImplementation("androidx.compose.ui:ui-tooling")
    implementation("androidx.compose.material:material-icons-extended")
    // Media3・ViewModel+StateFlow関連の依存は、実際に使用するPhase
    // (Phase2以降)で必要になった時点で追加する(Phase0時点では未追加)
}

// --- Rust(algorithm crate)のビルド統合 ---
// algorithm/はMyMusicPlayer/と同じ親フォルダ内にある兄弟フォルダとして扱う
val algorithmDir = file("../../algorithm")
val jniLibsDir = file("src/main/jniLibs")

val isWindows = System.getProperty("os.name").lowercase().contains("windows")

val cargoNdkBuild = tasks.register<Exec>("cargoNdkBuild") {
    workingDir = algorithmDir
    val args = listOf(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-o", jniLibsDir.absolutePath,
        "build", "--release"
    )
    if (isWindows) {
        commandLine(listOf("cmd", "/c") + args)
    } else {
        commandLine(args)
    }
}

tasks.named("preBuild") {
    dependsOn(cargoNdkBuild)
}
