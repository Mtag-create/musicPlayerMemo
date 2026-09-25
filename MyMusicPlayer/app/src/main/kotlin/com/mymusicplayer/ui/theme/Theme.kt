package com.mymusicplayer.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Brush

private val ClubDarkColorScheme = darkColorScheme(
    primary = Color(0xFF0A2A6B),      // アイコンのメタリックブルー
    secondary = Color(0xFF7FB8FF),    // 発光部分の明るいスカイブルー
    background = Color(0xFF0A0A0F),   // ほぼ黒(アイコンの背景に合わせる)
    surface = Color(0xFF16161F),
    onPrimary = Color(0xFF00102B),
    onBackground = Color(0xFFEDEDF2),
    onSurface = Color(0xFFEDEDF2)
)

@Composable
fun MyMusicPlayerTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = ClubDarkColorScheme,
        content = content
    )
}