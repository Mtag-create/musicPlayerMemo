package com.mymusicplayer.storage

import android.content.Context
import androidx.media3.common.MediaItem
import com.mymusicplayer.BuildConfig
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

    sealed class AudioSource {
        data class Asset(val assetPath: String) : AudioSource()
        data class LocalFile(val file: File) : AudioSource()
    }

    data class TrackFiles(
        val id: String,
        val audioSource: AudioSource,
        val mmzBytesProvider: () -> ByteArray
    )

    fun listAvailableTracks(context: Context): List<TrackFiles> {
        return if (BuildConfig.DEBUG) {
            listFromLocalStorage(context)
        } else {
            listFromAssets(context)
        }
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

    /** ファイル名の拡張子(.wav)を除いた、表示用のタイトルを返す。 */
    fun displayTitle(id: String): String = id.removeSuffix(".wav")

    private fun listFromAssets(context: Context): List<TrackFiles> {
        val files = context.assets.list(MUSIC_ASSET_DIR) ?: emptyArray()
        val wavFiles = files.filter { it.endsWith(".wav") }

        return wavFiles.mapNotNull { wavFile ->
            val baseName = wavFile.removeSuffix(".wav")
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

    private fun listFromLocalStorage(context: Context): List<TrackFiles> {
        val dir = File(context.filesDir, DEBUG_MUSIC_DIR)
        val files = dir.listFiles() ?: emptyArray()
        val wavFiles = files.filter { it.name.endsWith(".wav") }

        return wavFiles.mapNotNull { wavFile ->
            val baseName = wavFile.name.removeSuffix(".wav")
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
}