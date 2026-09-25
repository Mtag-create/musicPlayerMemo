package com.mymusicplayer.ui.navigation

/**
 * ナビゲーション先の識別子を1箇所に集約する。
 * 実際の画面遷移ロジック(NavHost)はMainActivity.ktで定義する。
 */
sealed class Screen(val route: String) {
    object Player : Screen("player")
    object Settings : Screen("settings")
    object History : Screen("history")
    object TrackList : Screen("track_list")
}