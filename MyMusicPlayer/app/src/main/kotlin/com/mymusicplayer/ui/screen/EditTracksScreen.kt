package com.mymusicplayer.ui.screen

import android.widget.Toast
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.mymusicplayer.storage.MusicEditor
import com.mymusicplayer.storage.MusicImporter
import androidx.compose.material3.OutlinedTextField
import com.mymusicplayer.player.PlayerViewModel

/**
 * 曲の編集画面(設定から開く)。filesDir内の曲だけを表示し、削除できる。
 * 反映(選曲対象や曲一覧)は、アプリの再起動後。
 */
@Composable
fun EditTracksScreen(onNavigateBack: () -> Unit, viewModel: PlayerViewModel) {
    val context = LocalContext.current
    var tracks by remember { mutableStateOf(MusicEditor.listEditableTracks(context)) }
    var deleteTarget by remember { mutableStateOf<MusicEditor.EditableTrack?>(null) }
    var renameTarget by remember { mutableStateOf<MusicEditor.EditableTrack?>(null) }
    Column(modifier = Modifier.fillMaxSize().padding(24.dp)) {
        IconButton(onClick = onNavigateBack) {
            Icon(Icons.Filled.ArrowBack, contentDescription = "Back")
        }
        Text("曲の編集", style = MaterialTheme.typography.titleLarge, color = MaterialTheme.colorScheme.onBackground)

        if (tracks.isEmpty()) {
            Text(
                "編集できる曲がありません",
                modifier = Modifier.padding(top = 16.dp),
                color = MaterialTheme.colorScheme.onBackground
            )
        }

        LazyColumn(modifier = Modifier.fillMaxWidth().fillMaxSize()) {
            items(tracks, key = { it.key }) { track ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = track.baseName,
                        modifier = Modifier.weight(1f).padding(vertical = 10.dp),
                        fontSize = 16.sp,
                        color = MaterialTheme.colorScheme.onSurface
                    )

                    if (track.destination == MusicImporter.Destination.LIVE) {
                        Text(
                            text = "SET",
                            fontSize = 11.sp,
                            color = MaterialTheme.colorScheme.secondary
                        )
                    }

                    TextButton(onClick = { renameTarget = track }) {
                        Text("名前変更", fontSize = 12.sp, color = MaterialTheme.colorScheme.secondary)
                    }

                    TextButton(onClick = { deleteTarget = track }) {
                        Text("削除", fontSize = 12.sp, color = MaterialTheme.colorScheme.error)
                    }
                }
            }
        }
    }

    renameTarget?.let { target ->
        var newName by remember(target) { mutableStateOf(target.baseName) }

        AlertDialog(
            onDismissRequest = { renameTarget = null },
            title = { Text("名前変更") },
            text = {
                Column {
                    Text(
                        if (target.destination == MusicImporter.Destination.MUSIC) {
                            ".mp3 と .mmz の名前を、まとめて変更します"
                        } else {
                            ".mp3 の名前を変更します"
                        }
                    )
                    OutlinedTextField(
                        value = newName,
                        onValueChange = { newName = it },
                        singleLine = true,
                        modifier = Modifier.padding(top = 16.dp)
                    )
                }
            },
            confirmButton = {
                TextButton(onClick = {
                    val result = MusicEditor.rename(target, newName)
                    if (result == MusicEditor.RenameResult.SUCCESS) {
                        tracks = MusicEditor.listEditableTracks(context)
                        viewModel.initRustLibrary()
                        renameTarget = null
                    }
                    val message = when (result) {
                        MusicEditor.RenameResult.SUCCESS -> "名前を変更しました"
                        MusicEditor.RenameResult.EMPTY -> "名前が空です"
                        MusicEditor.RenameResult.INVALID_NAME -> "使えない文字、または長すぎる名前です"
                        MusicEditor.RenameResult.UNCHANGED -> "名前が同じです"
                        MusicEditor.RenameResult.ALREADY_EXISTS -> "同じ名前の曲がすでにあります"
                        MusicEditor.RenameResult.FAILED -> "名前を変更できませんでした"
                    }
                    Toast.makeText(context, message, Toast.LENGTH_SHORT).show()
                }) { Text("変更") }
            },
            dismissButton = {
                TextButton(onClick = { renameTarget = null }) { Text("キャンセル") }
            }
        )
    }

    deleteTarget?.let { target ->
        var confirmText by remember(target) { mutableStateOf("") }

        AlertDialog(
            onDismissRequest = { deleteTarget = null },
            title = { Text("削除") },
            text = {
                Column {
                    Text("「${target.baseName}」を削除しますか?\n${target.files.joinToString("\n") { it.name }}")
                    OutlinedTextField(
                        value = confirmText,
                        onValueChange = { confirmText = it },
                        label = { Text("確認のため「削除」と入力") },
                        singleLine = true,
                        modifier = Modifier.padding(top = 16.dp)
                    )
                }
            },
            confirmButton = {
                TextButton(
                    enabled = confirmText.trim() == "削除",
                    onClick = {
                        val ok = MusicEditor.delete(target)
                        tracks = MusicEditor.listEditableTracks(context)
                        viewModel.initRustLibrary()
                        deleteTarget = null
                        Toast.makeText(
                            context,
                            if (ok) "削除しました" else "一部のファイルを削除できませんでした",
                            Toast.LENGTH_SHORT
                        ).show()
                    }
                ) { Text("削除", color = MaterialTheme.colorScheme.error) }
            },
            dismissButton = {
                TextButton(onClick = { deleteTarget = null }) { Text("キャンセル") }
            }
        )
    }
}