package com.mymusicplayer.player

import android.content.Context
import androidx.media3.common.MediaItem
import androidx.media3.exoplayer.ExoPlayer

/**
 * Media3(ExoPlayer)をラップする再生エンジン。
 * Phase2の機能1時点では「アセットからの読み込み」のみを扱う。
 */
class MusicPlayer(context: Context) {
    val player: ExoPlayer = ExoPlayer.Builder(context).build()

    /**
     * assets/ 配下のファイルを読み込む。
     * @param assetPath assets/ からの相対パス(例: "music/sample.wav")
     */
    fun loadFromAssets(assetPath: String) {
        val mediaItem = MediaItem.fromUri("asset:///$assetPath")
        player.setMediaItem(mediaItem)
        player.prepare()
    }

    fun release() {
        player.release()
    }
}