// トップレベルのビルド設定。
//
// AGP 9.0以降はビルトインKotlinサポートが既定で有効なため、
// org.jetbrains.kotlin.android プラグインは適用しない。
// Composeコンパイラプラグインのみ、AGPが同梱するKotlin Gradle Plugin
// (AGP 9.1.1が要求する最小バージョン 2.2.10)に合わせて適用する。
plugins {
    id("com.android.application") version "9.1.1" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.2.10" apply false
}
