package com.mymusicplayer.ui.screen

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.List
import androidx.compose.material.icons.filled.Loop
import androidx.compose.material.icons.filled.Menu
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.mymusicplayer.player.PlayerViewModel
import com.mymusicplayer.player.WAVE_HISTORY_MAX
import com.mymusicplayer.storage.MusicStorage

@Composable
fun PlayerScreen(
    onNavigateToSettings: () -> Unit,
    onNavigateToHistory: () -> Unit,
    onNavigateToTrackList: () -> Unit,
    viewModel: PlayerViewModel = viewModel()
) {
    LaunchedEffect(Unit) {
        viewModel.initRustLibrary()
    }

    val selectedTrackId by viewModel.selectedTrackId.collectAsState()
    val autoDjEnabled by viewModel.autoDjEnabled.collectAsState()
    val waveHistory by viewModel.waveHistory.collectAsState()

    // 直近の選曲で実際に採用された4軸の重要度(トレンド分析結果×設定画面の重み)を比較し、
    // 最も高いものを「現在最も重視されている軸」として明るく表示する。
    // 初回選曲(ランダム選出)直後などimportanceがまだ無い間は全て同率(明るい)として扱う。
    val latestPoint = waveHistory.lastOrNull()
    val importances = listOf(
        latestPoint?.energyImportance ?: 0f,
        latestPoint?.bpmImportance ?: 0f,
        latestPoint?.brightnessImportance ?: 0f,
        latestPoint?.keyImportance ?: 0f
    )
    val maxImportance = importances.max()

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.End
        ) {
            IconButton(onClick = onNavigateToHistory) {
                Icon(Icons.Filled.List, contentDescription = "History")
            }
            IconButton(onClick = onNavigateToTrackList) {
                Icon(Icons.Filled.Menu, contentDescription = "Track List")
            }
            IconButton(onClick = onNavigateToSettings) {
                Icon(Icons.Filled.Settings, contentDescription = "Settings")
            }
        }

        Box(
            modifier = Modifier
                .fillMaxWidth()
                .weight(0.75f),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = selectedTrackId?.let { MusicStorage.displayTitle(it) } ?: "-",
                style = MaterialTheme.typography.headlineMedium.copy(fontWeight = FontWeight.Bold),
                color = MaterialTheme.colorScheme.onBackground
            )
        }

        val isPlaying by viewModel.isPlaying.collectAsState()
        val currentPositionMs by viewModel.currentPositionMs.collectAsState()
        val durationMs by viewModel.durationMs.collectAsState()

        // ドラッグ中は実際の再生位置(currentPositionMs)に引っ張られないよう、
        // 操作中だけローカルのsliderPosition(0f〜1f)を使う。指を離したらシークして解除する。
        var sliderPosition by remember { mutableStateOf<Float?>(null) }
        val displayedFraction = sliderPosition
            ?: if (durationMs > 0) (currentPositionMs.toFloat() / durationMs.toFloat()).coerceIn(0f, 1f) else 0f

        val loopEnabled by viewModel.loopEnabled.collectAsState()

        // シークバー右上の小さなループアイコン。タップで現在の曲のループ再生をオン/オフする。
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.End
        ) {
            IconButton(
                onClick = { viewModel.toggleLoop() },
                modifier = Modifier.height(28.dp)
            ) {
                Icon(
                    Icons.Filled.Loop,
                    contentDescription = "ループ再生",
                    tint = emphasisColor(MaterialTheme.colorScheme.secondary, loopEnabled),
                    modifier = Modifier.height(18.dp)
                )
            }
        }

        Slider(
            value = displayedFraction,
            onValueChange = { sliderPosition = it },
            onValueChangeFinished = {
                val fraction = sliderPosition ?: 0f
                viewModel.seekTo((fraction * durationMs).toLong())
                sliderPosition = null
            },
            enabled = durationMs > 0,
            modifier = Modifier.fillMaxWidth()
        )

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text(
                text = formatDurationMs((displayedFraction * durationMs).toLong()),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onBackground
            )
            Text(
                text = formatDurationMs(durationMs),
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onBackground
            )
        }

        Spacer(modifier = Modifier.height(8.dp))

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Button(
                onClick = {
                    when {
                        !autoDjEnabled -> viewModel.startAutoDj()
                        isPlaying -> viewModel.pause()
                        else -> viewModel.play()
                    }
                },
                modifier = Modifier.weight(1f),
                colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.primary)
            ) {
                Text(if (autoDjEnabled && isPlaying) "一時停止" else "スタート")
            }

            Button(
                onClick = { viewModel.stop() },
                enabled = autoDjEnabled,
                modifier = Modifier.weight(1f)
            ) {
                Text("停止")
            }
        }

        Spacer(modifier = Modifier.height(24.dp))

        Column(
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f)
        ) {
            MetricWaveChart(
                label = "Energy",
                values = waveHistory.map { it.energy },
                color = emphasisColor(MaterialTheme.colorScheme.secondary, importances[0] == maxImportance)
            )
            Spacer(modifier = Modifier.height(12.dp))
            MetricWaveChart(
                label = "BPM",
                values = waveHistory.map { it.bpm },
                color = emphasisColor(MaterialTheme.colorScheme.secondary, importances[1] == maxImportance)
            )
            Spacer(modifier = Modifier.height(12.dp))
            MetricWaveChart(
                label = "Key",
                values = waveHistory.map { it.keyPitchClass.toFloat() },
                color = emphasisColor(MaterialTheme.colorScheme.secondary, importances[3] == maxImportance)
            )
            Spacer(modifier = Modifier.height(12.dp))
            MetricWaveChart(
                label = "Brightness",
                values = waveHistory.map { it.brightness },
                color = emphasisColor(MaterialTheme.colorScheme.secondary, importances[2] == maxImportance)
            )
        }
    }
}

/** ミリ秒を"分:秒"形式の文字列に変換する(例: 125000L -> "2:05")。 */
private fun formatDurationMs(ms: Long): String {
    val totalSeconds = (ms / 1000).coerceAtLeast(0)
    val minutes = totalSeconds / 60
    val seconds = totalSeconds % 60
    return "%d:%02d".format(minutes, seconds)
}

/**
 * 現在最も重視されている指標(重みが最大のもの)は明るい色(secondary)のまま、
 * それ以外は同じ色相を少し暗くして表示する。
 * secondaryはテーマ上「発光部分の明るいスカイブルー」として定義されており、
 * ほぼ黒に近い背景でも視認できる明るさがあるため基準色に使う。
 */
private fun emphasisColor(base: Color, isEmphasized: Boolean): Color {
    if (isEmphasized) return base
    val dimFactor = 0.45f
    return Color(
        red = base.red * dimFactor,
        green = base.green * dimFactor,
        blue = base.blue * dimFactor,
        alpha = base.alpha
    )
}

/**
 * 直近WAVE_HISTORY_MAX曲(WavePoint)の1指標分を、水平スクロール型の折れ線グラフとして描画する。
 * 表示領域の横幅は常にWAVE_HISTORY_MAX曲分のスロット幅として扱い、
 * 新しい曲は常に右端のスロットに追加され、曲数がWAVE_HISTORY_MAXを超えると
 * (waveHistory側で古いものが自動的に切り捨てられるため)最古の曲は左端からフレームアウトする。
 * データが埋まりきっていない間は右詰めで表示され、左側は空白になる。
 * 値のレンジは表示中のデータ内でのmin/maxに正規化する(指標ごとにスケールが違うため)。
 */
@Composable
private fun MetricWaveChart(
    label: String,
    values: List<Float>,
    color: Color
) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onBackground
        )
        Canvas(
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp)
        ) {
            if (values.size < 2) return@Canvas

            val slotWidth = size.width / WAVE_HISTORY_MAX
            // 右詰め: 最新の曲が常に右端のスロットに来るよう、スロットオフセットを計算する
            val slotOffset = WAVE_HISTORY_MAX - values.size

            val minValue = values.min()
            val maxValue = values.max()
            val range = (maxValue - minValue).let { if (it > 0.0001f) it else 1f }

            fun xFor(index: Int) = (slotOffset + index + 0.5f) * slotWidth

            val path = Path()
            values.forEachIndexed { index, value ->
                val x = xFor(index)
                val y = size.height - ((value - minValue) / range) * size.height
                if (index == 0) path.moveTo(x, y) else path.lineTo(x, y)
            }

            drawPath(path = path, color = color, style = Stroke(width = 4f))
        }
    }
}