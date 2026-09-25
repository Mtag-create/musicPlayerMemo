package com.mymusicplayer.ui.screen

import android.widget.Toast
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.mymusicplayer.player.PlayerViewModel
import com.mymusicplayer.storage.MusicStorage

/**
 * ライブラリ内の全曲一覧画面。
 * - 曲名タップ: Rust側の「現在の曲」として即座に設定(setCurrentTrack)し、その場で再生開始してPlayer画面へ戻る
 * - 「コピー」ボタン: 曲名をクリップボードにコピーする
 * - 「次に再生」ボタン: 今すぐ割り込まず、今の曲が終わったタイミングでこの曲を再生するよう予約する
 */
@Composable
fun TrackListScreen(
    onNavigateBack: () -> Unit,
    viewModel: PlayerViewModel
) {
    val availableTracks by viewModel.availableTracks.collectAsState()
    val clipboardManager = LocalClipboardManager.current
    val context = LocalContext.current

    Column(modifier = Modifier.fillMaxSize().padding(24.dp)) {
        IconButton(onClick = onNavigateBack) {
            Icon(Icons.Filled.ArrowBack, contentDescription = "Back")
        }
        Text("曲一覧", style = MaterialTheme.typography.titleLarge, color = MaterialTheme.colorScheme.onBackground)

        LazyColumn(
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxSize()
        ) {
            items(availableTracks) { id ->
                val title = MusicStorage.displayTitle(id)

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = title,
                        modifier = Modifier
                            .weight(1f)
                            .clickable {
                                viewModel.selectTrackManually(id)
                                onNavigateBack()
                            }
                            .padding(vertical = 10.dp),
                        fontSize = 16.sp,
                        color = MaterialTheme.colorScheme.onSurface
                    )

                    TextButton(onClick = {
                        clipboardManager.setText(AnnotatedString(title))
                        Toast.makeText(context, "コピーしました", Toast.LENGTH_SHORT).show()
                    }) {
                        Text("コピー", fontSize = 12.sp, color = MaterialTheme.colorScheme.secondary)
                    }

                    IconButton(onClick = {
                        viewModel.queueTrackNext(id)
                        Toast.makeText(context, "次に再生予約: $title", Toast.LENGTH_SHORT).show()
                    }) {
                        Icon(
                            Icons.Filled.PlayArrow,
                            contentDescription = "次に再生",
                            tint = MaterialTheme.colorScheme.secondary
                        )
                    }
                }
            }
        }
    }
}