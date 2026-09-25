package com.mymusicplayer.player

import android.content.Intent
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService

class PlaybackService : MediaSessionService() {
    private lateinit var musicPlayer: MusicPlayer
    private lateinit var mediaSession: MediaSession

    override fun onCreate() {
        super.onCreate()
        musicPlayer = MusicPlayer(this)
        mediaSession = MediaSession.Builder(this, musicPlayer.player).build()
    }

    override fun onGetSession(controllerInfo: MediaSession.ControllerInfo): MediaSession {
        return mediaSession
    }

    /**
     * タスクが最近使ったアプリ一覧から消された(=アプリを閉じた)ときに呼ばれる。
     * ホームボタンやロック画面への移行ではこのコールバックは呼ばれないため、
     * ここで停止させても「アプリを閉じていない間はバックグラウンド/ロック画面で再生を続ける」動作は変わらない。
     * 誤操作防止のため、再生中かどうかに関わらずここで確実に停止・サービス終了する。
     */
    override fun onTaskRemoved(rootIntent: Intent?) {
        mediaSession.player.apply {
            pause()
            stop()
        }
        stopSelf()
        super.onTaskRemoved(rootIntent)
    }

    override fun onDestroy() {
        mediaSession.run {
            player.release()
            release()
        }
        super.onDestroy()
    }
}