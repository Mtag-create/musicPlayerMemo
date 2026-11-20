package com.mymusicplayer.storage

import android.content.Context
import androidx.media3.common.MediaItem
import java.io.File
import androidx.media3.common.MediaMetadata

/**
 * 音源(.wav)と解析サマリ(.mmz)へのアクセスを抽象化する層。
 *
 * Release: assets/music/ に内蔵された全曲を使う。
 * Debug: context.filesDir/music/ (アプリ専用領域、adb pushで配置)を使う。
 *        APKへの再パッケージ(3.56GB・約14分)を避けるための開発時専用の経路。
 *
 * MusicPlayer/PlayerViewModelは、この違いを意識する必要がない
 * (mediaItemFor()がAsset/LocalFileの差を吸収する)。
 */
object MusicStorage {
    private const val MUSIC_ASSET_DIR = "music"
    private const val DEBUG_MUSIC_DIR = "music"

    /**
     * ライブのフルセットなど、Auto DJの選曲対象にしない曲を置くフォルダ(music/とは別)。
     * .mmz(解析サマリ)は不要で、.wavを置くだけでよい。
     */
    private const val LIVE_DIR = "live_music"

    /** live_music/の曲のIDに付ける接頭辞。music/内の同名ファイルとIDが衝突しないようにする。 */
    private const val LIVE_ID_PREFIX = "$LIVE_DIR/"

    sealed class AudioSource {
        data class Asset(val assetPath: String) : AudioSource()
        data class LocalFile(val file: File) : AudioSource()
    }

    /**
     * @param mmzBytesProvider 解析サマリ(.mmz)の読み出し関数。live_music/の曲(手動再生専用)はnull。
     *
     * live_music/の曲は「手動再生専用」として扱う。
     * Rust側の選曲ライブラリには渡されないため、Auto DJの候補には一切入らず、
     * 曲一覧から手動で選んだとき(または「次に再生」予約したとき)だけ再生される。
     */
    data class TrackFiles(
        val id: String,
        val audioSource: AudioSource,
        val mmzBytesProvider: (() -> ByteArray)?
    ) {
        val manualOnly: Boolean get() = mmzBytesProvider == null
    }

    /** music/(Auto DJ対象、.mmz必須)とlive_music/(手動再生専用)の両方の曲を返す。 */
    /** music/(Auto DJ対象、.mmz必須)とlive_music/(手動再生専用)の両方の曲を返す。 */
    fun listAvailableTracks(context: Context): List<TrackFiles> {
        // filesDir(取り込み済み・adb push)を優先し、assetsと同じIDがあれば重複させない
        return (
                listFromLocalStorage(context) + listLiveFromLocalStorage(context) +
                        listFromAssets(context) + listLiveFromAssets(context)
                ).distinctBy { it.id }
    }

    /** AudioSourceの違いを吸収し、再生可能なMediaItemを返す。 */
    /** AudioSourceの違いを吸収し、再生可能なMediaItemを返す。タイトルは通知にも表示される。 */
    fun mediaItemFor(audioSource: AudioSource, title: String): MediaItem {
        val metadata = MediaMetadata.Builder().setTitle(title).build()

        val uri = when (audioSource) {
            is AudioSource.Asset -> {
                val parts = audioSource.assetPath.split("/")
                val encodedPath = parts.joinToString("/") { android.net.Uri.encode(it) }
                android.net.Uri.parse("asset:///$encodedPath")
            }
            is AudioSource.LocalFile -> android.net.Uri.fromFile(audioSource.file)
        }

        return MediaItem.Builder()
            .setUri(uri)
            .setMediaMetadata(metadata)
            .build()
    }

    /** フォルダの接頭辞と拡張子(.wav)を除いた、表示用のタイトルを返す。 */
    fun displayTitle(id: String): String = id.removePrefix(LIVE_ID_PREFIX).removeSuffix(".mp3")

    private fun listFromAssets(context: Context): List<TrackFiles> {
        val files = context.assets.list(MUSIC_ASSET_DIR) ?: emptyArray()
        val wavFiles = files.filter { it.endsWith(".mp3") }

        return wavFiles.mapNotNull { wavFile ->
            val baseName = wavFile.removeSuffix(".mp3")
            val mmzFile = "$baseName.mmz"
            if (files.contains(mmzFile)) {
                TrackFiles(
                    id = wavFile,
                    audioSource = AudioSource.Asset("$MUSIC_ASSET_DIR/$wavFile"),
                    mmzBytesProvider = {
                        context.assets.open("$MUSIC_ASSET_DIR/$mmzFile").use { it.readBytes() }
                    }
                )
            } else {
                null
            }
        }
    }

    private fun listLiveFromAssets(context: Context): List<TrackFiles> {
        val files = context.assets.list(LIVE_DIR) ?: emptyArray()
        return files.filter { it.endsWith(".mp3") }.map { wavFile ->
            TrackFiles(
                id = "$LIVE_ID_PREFIX$wavFile",
                audioSource = AudioSource.Asset("$LIVE_DIR/$wavFile"),
                mmzBytesProvider = null
            )
        }
    }

    private fun listFromLocalStorage(context: Context): List<TrackFiles> {
        val dir = File(context.filesDir, DEBUG_MUSIC_DIR)
        val files = dir.listFiles() ?: emptyArray()
        val wavFiles = files.filter { it.name.endsWith(".mp3") }

        return wavFiles.mapNotNull { wavFile ->
            val baseName = wavFile.name.removeSuffix(".mp3")
            val mmzFile = File(dir, "$baseName.mmz")
            if (mmzFile.exists()) {
                TrackFiles(
                    id = wavFile.name,
                    audioSource = AudioSource.LocalFile(wavFile),
                    mmzBytesProvider = { mmzFile.readBytes() }
                )
            } else {
                null
            }
        }
    }

    /** Debug用: filesDir/live_music/ を(無ければ作成して)読む。adb pushの置き場所になる。 */
    private fun listLiveFromLocalStorage(context: Context): List<TrackFiles> {
        val dir = File(context.filesDir, LIVE_DIR)
        dir.mkdirs()
        val wavFiles = (dir.listFiles() ?: emptyArray()).filter { it.name.endsWith(".mp3") }

        return wavFiles.map { wavFile ->
            TrackFiles(
                id = "$LIVE_ID_PREFIX${wavFile.name}",
                audioSource = AudioSource.LocalFile(wavFile),
                mmzBytesProvider = null
            )
        }
    }
}