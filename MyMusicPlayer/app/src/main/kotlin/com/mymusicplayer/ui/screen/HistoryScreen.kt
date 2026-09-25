package com.mymusicplayer.ui.screen

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.mymusicplayer.player.PlayerViewModel
import com.mymusicplayer.storage.MusicStorage

/**
 * 選曲履歴専用の画面。
 * 以前はPlayerScreenに常時表示されていたが、
 * PlayerScreen側の表示領域をWave表示に使うため、SettingsScreenと同様に
 * 独立した画面へ切り出した。
 */
@Composable
fun HistoryScreen(
    onNavigateBack: () -> Unit,
    viewModel: PlayerViewModel
) {
    val selectionHistory by viewModel.selectionHistory.collectAsState()

    Column(modifier = Modifier.fillMaxSize().padding(24.dp)) {
        IconButton(onClick = onNavigateBack) {
            Icon(Icons.Filled.ArrowBack, contentDescription = "Back")
        }
        Text("履歴", style = MaterialTheme.typography.titleLarge, color = MaterialTheme.colorScheme.onBackground)

        LazyColumn(
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxSize()
        ) {
            items(selectionHistory.reversed()) { id ->
                Text(
                    text = MusicStorage.displayTitle(id),
                    modifier = Modifier.padding(vertical = 6.dp),
                    fontSize = 16.sp,
                    color = MaterialTheme.colorScheme.onSurface
                )
            }
        }
    }
}