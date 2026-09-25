package com.mymusicplayer.ui.screen

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.mymusicplayer.player.PlayerViewModel

@Composable
fun SettingsScreen(
    onNavigateBack: () -> Unit,
    viewModel: PlayerViewModel
) {
    Column(modifier = Modifier.fillMaxSize().padding(24.dp)) {
        IconButton(onClick = onNavigateBack) {
            Icon(Icons.Filled.ArrowBack, contentDescription = "Back")
        }
        Text("Settings", style = MaterialTheme.typography.titleLarge)

        MultiplierSlider("Energy", viewModel.energyMultiplier.collectAsState().value, viewModel::setEnergyMultiplier)
        MultiplierSlider("BPM", viewModel.bpmMultiplier.collectAsState().value, viewModel::setBpmMultiplier)
        MultiplierSlider("Key", viewModel.keyMultiplier.collectAsState().value, viewModel::setKeyMultiplier)
        MultiplierSlider("Brightness", viewModel.brightnessMultiplier.collectAsState().value, viewModel::setBrightnessMultiplier)

        VolumeSlider(viewModel.appVolume.collectAsState().value, viewModel::setAppVolume)

        LoopCountSetting(viewModel.loopCount.collectAsState().value, viewModel::setLoopCount)

        LibraryStatusSection(
            status = viewModel.libraryStatus.collectAsState().value,
            onReload = { viewModel.initRustLibrary() }
        )
    }
}

/**
 * 曲ライブラリ(Rust側にmmzを読み込ませた結果)の状態を表示する常設セクション。
 * 曲の入れ直し(adb push等)をした後、実際に何曲読み込めたかをここで確認できる。
 */
@Composable
private fun LibraryStatusSection(
    status: PlayerViewModel.LibraryStatus,
    onReload: () -> Unit
) {
    Column(modifier = Modifier.padding(top = 24.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text(
                "ライブラリ",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onBackground
            )
            Button(onClick = onReload) {
                Text("再読み込み")
            }
        }

        Spacer(modifier = Modifier.height(8.dp))

        when (status) {
            is PlayerViewModel.LibraryStatus.NotLoaded -> {
                Text("未読み込み", color = MaterialTheme.colorScheme.onBackground)
            }
            is PlayerViewModel.LibraryStatus.Loading -> {
                Text("読み込み中...", color = MaterialTheme.colorScheme.onBackground)
            }
            is PlayerViewModel.LibraryStatus.Loaded -> {
                val statusColor = if (status.success) {
                    MaterialTheme.colorScheme.secondary
                } else {
                    MaterialTheme.colorScheme.error
                }
                Text(
                    text = "読み込み曲数: ${status.trackCount}曲",
                    color = statusColor,
                    style = MaterialTheme.typography.bodyLarge
                )

                if (status.failedIds.isNotEmpty()) {
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "失敗: ${status.failedIds.size}件",
                        color = MaterialTheme.colorScheme.error
                    )
                    LazyColumn(modifier = Modifier.fillMaxWidth().height(120.dp)) {
                        items(status.failedIds) { failedId ->
                            Text(
                                text = failedId,
                                color = MaterialTheme.colorScheme.onBackground,
                                style = MaterialTheme.typography.bodySmall,
                                modifier = Modifier.padding(vertical = 2.dp)
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun VolumeSlider(value: Float, onValueChange: (Float) -> Unit) {
    Column(modifier = Modifier.padding(top = 24.dp)) {
        Text("音量: ${(value * 100).toInt()}%", color = MaterialTheme.colorScheme.onBackground)
        Slider(
            value = value,
            onValueChange = onValueChange,
            valueRange = 0f..1f
        )
    }
}

/**
 * ループ回数の入力欄。
 * -1(無限ループ)、または1以上の自然数のみを受け付ける。
 * 数字と先頭の"-"以外の文字はそもそも入力させず、
 * 途中入力などでまだ条件を満たさない値はViewModelには反映せず、エラー表示のみ行う。
 */
@Composable
private fun LoopCountSetting(value: Int, onValueChange: (Int) -> Unit) {
    var text by remember { mutableStateOf(value.toString()) }
    var isValid by remember { mutableStateOf(true) }

    Column(modifier = Modifier.padding(top = 24.dp)) {
        Text("ループ回数(-1で無限)", color = MaterialTheme.colorScheme.onBackground)
        OutlinedTextField(
            value = text,
            onValueChange = { input ->
                // 数字と先頭の"-"以外の文字は受け付けない
                val filtered = input.filter { it.isDigit() || it == '-' }
                val hasMisplacedMinus = filtered.count { it == '-' } > 1 || filtered.indexOf('-') > 0
                val sanitized = if (hasMisplacedMinus) text else filtered
                text = sanitized

                val parsed = sanitized.toIntOrNull()
                val valid = sanitized == "-1" || (parsed != null && parsed >= 1)
                isValid = valid
                if (valid && parsed != null) {
                    onValueChange(parsed)
                }
            },
            isError = !isValid,
            singleLine = true,
            modifier = Modifier.fillMaxWidth()
        )
        if (!isValid) {
            Text(
                text = "-1、または1以上の整数を入力してください",
                color = MaterialTheme.colorScheme.error,
                style = MaterialTheme.typography.bodySmall
            )
        }
    }
}

@Composable
private fun MultiplierSlider(label: String, value: Float, onValueChange: (Float) -> Unit) {
    Column(modifier = Modifier.padding(top = 16.dp)) {
        Text("$label: ${"%.1f".format(value)}", color = MaterialTheme.colorScheme.onBackground)
        Slider(
            value = value,
            onValueChange = onValueChange,
            valueRange = 0f..2f
        )
    }
}