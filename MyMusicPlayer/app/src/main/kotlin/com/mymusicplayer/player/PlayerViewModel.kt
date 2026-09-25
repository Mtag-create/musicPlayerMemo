package com.mymusicplayer.player

import android.app.Application
import android.content.ComponentName
import androidx.lifecycle.AndroidViewModel
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.session.MediaController
import androidx.media3.session.SessionToken
import com.google.common.util.concurrent.MoreExecutors
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import com.mymusicplayer.storage.MusicStorage
import com.mymusicplayer.rust.RustBridge
import com.mymusicplayer.rust.InitLibraryRequest
import com.mymusicplayer.rust.MmzFileEntry
import com.mymusicplayer.rust.RustCodec
import com.mymusicplayer.rust.SelectNextRequest
import com.mymusicplayer.rust.SelectNextResponse
import com.mymusicplayer.rust.SetCurrentTrackRequest
import kotlinx.coroutines.withContext

enum class LoadState { NOT_LOADED, LOADING, READY }

/** Energy/BPM/Key/Brightnessの折れ線グラフ用、1曲分のサマリ値 */
data class WavePoint(
    val energy: Float,
    val bpm: Float,
    val keyPitchClass: Int,
    val brightness: Float,
    /** この曲を選んだ際に実際に使われた4軸の重要度(トレンド由来の係数×設定画面の重み)。
     *  初回選曲(ランダム選出)の場合はnull。 */
    val energyImportance: Float?,
    val bpmImportance: Float?,
    val brightnessImportance: Float?,
    val keyImportance: Float?
)

/** Waveグラフに表示する曲数。表示領域は常にこの本数分のスロット幅として扱う。 */
const val WAVE_HISTORY_MAX = 21

private val NOTE_NAMES = listOf("C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B")

/** "C Major" / "A Minor" のようなKey名から、グラフ表示用のピッチクラス(0-11)を取り出す */
private fun pitchClassOf(keyName: String?): Int {
    val root = keyName?.substringBefore(" ") ?: return 0
    val index = NOTE_NAMES.indexOf(root)
    return if (index >= 0) index else 0
}

class PlayerViewModel(application: Application) : AndroidViewModel(application) {

    private var controller: MediaController? = null

    private val _loadState = MutableStateFlow(LoadState.NOT_LOADED)
    val loadState: StateFlow<LoadState> = _loadState

    private val _isPlaying = MutableStateFlow(false)

    val isPlaying: StateFlow<Boolean> = _isPlaying

    private val _currentPositionMs = MutableStateFlow(0L)
    val currentPositionMs: StateFlow<Long> = _currentPositionMs

    private val _durationMs = MutableStateFlow(0L)
    val durationMs: StateFlow<Long> = _durationMs

    init {
        val sessionToken = SessionToken(
            application,
            ComponentName(application, PlaybackService::class.java)
        )
        val controllerFuture = MediaController.Builder(application, sessionToken).buildAsync()
        controllerFuture.addListener({
            controller = controllerFuture.get()
            controller?.volume = _appVolume.value
            controller?.addListener(object : Player.Listener {
                override fun onPlaybackStateChanged(playbackState: Int) {
                    when (playbackState) {
                        Player.STATE_READY -> _loadState.value = LoadState.READY
                        Player.STATE_BUFFERING -> _loadState.value = LoadState.LOADING
                        Player.STATE_ENDED -> {
                            if (_loopEnabled.value && (remainingLoops == -1 || remainingLoops > 0)) {
                                // ループ続行: 同じ曲の先頭に戻すだけで、Rust側の選曲は呼ばない
                                if (remainingLoops > 0) remainingLoops -= 1
                                controller?.seekTo(0)
                                controller?.play()
                            } else {
                                _loopEnabled.value = false
                                if (_autoDjEnabled.value) {
                                    viewModelScope.launch(Dispatchers.IO) {
                                        val queued = _queuedTrackId.value
                                        if (queued != null) {
                                            _queuedTrackId.value = null
                                            setCurrentTrackAndApply(queued)
                                        } else {
                                            playSelectedNext()
                                        }
                                    }
                                }
                            }
                        }
                        else -> Unit
                    }
                }

                override fun onIsPlayingChanged(isPlaying: Boolean) {
                    _isPlaying.value = isPlaying
                }
            })
        }, MoreExecutors.directExecutor())

        // シークバー用に再生位置と曲の長さを定期的にポーリングする。
        // MediaControllerはメインスレッドで扱う必要があるため、viewModelScope(Main)で実行する。
        viewModelScope.launch {
            while (true) {
                controller?.let { c ->
                    _currentPositionMs.value = c.currentPosition.coerceAtLeast(0L)
                    _durationMs.value = c.duration.let { if (it > 0) it else 0L }
                }
                delay(500)
            }
        }
    }

    fun loadTestTrack() {
        _loadState.value = LoadState.LOADING
        controller?.setMediaItem(MediaItem.fromUri("asset:///music/WakeMeUp.wav"))
        controller?.prepare()
    }

    fun play() { controller?.play() }
    fun stop() {
        _autoDjEnabled.value = false
        controller?.stop()
    }
    fun pause() { controller?.pause() }
    fun seekTo(positionMs: Long) { controller?.seekTo(positionMs) }

    private val _pingResult = MutableStateFlow<String?>(null)
    val pingResult: StateFlow<String?> = _pingResult

    /** ライブラリ読み込みの進行状況。Settings画面に常設表示するための状態。 */
    sealed class LibraryStatus {
        object NotLoaded : LibraryStatus()
        object Loading : LibraryStatus()
        data class Loaded(
            val success: Boolean,
            val trackCount: Int,
            val failedIds: List<String>
        ) : LibraryStatus()
    }

    private val _libraryStatus = MutableStateFlow<LibraryStatus>(LibraryStatus.NotLoaded)
    val libraryStatus: StateFlow<LibraryStatus> = _libraryStatus

    private val _selectedTrackId = MutableStateFlow<String?>(null)
    val selectedTrackId: StateFlow<String?> = _selectedTrackId

    private val _availableTracks = MutableStateFlow<List<String>>(emptyList())
    val availableTracks: StateFlow<List<String>> = _availableTracks

    private val _queuedTrackId = MutableStateFlow<String?>(null)
    val queuedTrackId: StateFlow<String?> = _queuedTrackId

    /** 曲一覧画面の「次に再生」ボタンで予約する。今の曲が終わったタイミングでこの曲が再生される。 */
    fun queueTrackNext(trackId: String) {
        _queuedTrackId.value = trackId
    }

    /** シークバー右上のループアイコンで、現在再生中の曲だけをループさせるかどうか。曲が切り替わると自動的にfalseへ戻る。 */
    private val _loopEnabled = MutableStateFlow(false)
    val loopEnabled: StateFlow<Boolean> = _loopEnabled

    /** Settings画面で設定するループ回数。-1は無限ループ、それ以外は1以上の自然数のみを許可する。 */
    private val _loopCount = MutableStateFlow(-1)
    val loopCount: StateFlow<Int> = _loopCount

    /** ループ有効化時点の残り回数。-1(無限)の間は減らさない。0になったらループを終了する。 */
    private var remainingLoops = 0

    /** ループアイコンのタップで呼ばれる。オンにする瞬間、その時点のループ回数設定を残り回数として取り込む。 */
    fun toggleLoop() {
        if (_loopEnabled.value) {
            _loopEnabled.value = false
        } else {
            _loopEnabled.value = true
            remainingLoops = _loopCount.value
        }
    }

    /** Settings画面から呼ばれる。-1、または1以上の自然数以外は無視する(値を書き換えない)。 */
    fun setLoopCount(value: Int) {
        if (value == -1 || value >= 1) {
            _loopCount.value = value
        }
    }

    private val _selectionHistory = MutableStateFlow<List<String>>(emptyList())
    val selectionHistory: StateFlow<List<String>> = _selectionHistory

    private val _waveHistory = MutableStateFlow<List<WavePoint>>(emptyList())
    val waveHistory: StateFlow<List<WavePoint>> = _waveHistory

    private val _autoDjEnabled = MutableStateFlow(false)
    val autoDjEnabled: StateFlow<Boolean> = _autoDjEnabled

    private val _energyMultiplier = MutableStateFlow(1.0f)
    val energyMultiplier: StateFlow<Float> = _energyMultiplier

    private val _bpmMultiplier = MutableStateFlow(1.0f)
    val bpmMultiplier: StateFlow<Float> = _bpmMultiplier

    private val _keyMultiplier = MutableStateFlow(1.0f)
    val keyMultiplier: StateFlow<Float> = _keyMultiplier

    private val _brightnessMultiplier = MutableStateFlow(1.0f)
    val brightnessMultiplier: StateFlow<Float> = _brightnessMultiplier

    private val _appVolume = MutableStateFlow(1.0f)
    val appVolume: StateFlow<Float> = _appVolume

    fun setAppVolume(value: Float) {
        val clamped = value.coerceIn(0f, 1f)
        _appVolume.value = clamped
        controller?.volume = clamped
    }

    fun setEnergyMultiplier(value: Float) { _energyMultiplier.value = value }
    fun setBpmMultiplier(value: Float) { _bpmMultiplier.value = value }
    fun setKeyMultiplier(value: Float) { _keyMultiplier.value = value }
    fun setBrightnessMultiplier(value: Float) { _brightnessMultiplier.value = value }

    private var trackFilesById: Map<String, MusicStorage.TrackFiles> = emptyMap()

    fun pingRust() {
        val input = byteArrayOf(1, 2, 3, 4, 5)
        val output = RustBridge.pingRust(input)
        _pingResult.value = "input=${input.toList()} -> output=${output.toList()}"
    }

    fun initRustLibrary() {
        _libraryStatus.value = LibraryStatus.Loading
        viewModelScope.launch(Dispatchers.IO) {
            val application: android.app.Application = getApplication()
            val availableTracks = MusicStorage.listAvailableTracks(application)
            trackFilesById = availableTracks.associateBy { it.id }

            val entries = availableTracks.map { trackFiles ->
                MmzFileEntry(id = trackFiles.id, mmzBytes = trackFiles.mmzBytesProvider())
            }

            val request = InitLibraryRequest(files = entries)
            val requestBytes = RustCodec.encodeInitLibraryRequest(request)
            val responseBytes = RustBridge.initLibrary(requestBytes)
            val response = RustCodec.decodeInitLibraryResponse(responseBytes)

            _libraryStatus.value = LibraryStatus.Loaded(
                success = response.success,
                trackCount = response.trackCount,
                failedIds = response.failedIds
            )
            _availableTracks.value = trackFilesById.keys.sortedBy { MusicStorage.displayTitle(it) }
        }
    }

    fun startAutoDj() {
        _autoDjEnabled.value = true
        viewModelScope.launch(Dispatchers.IO) {
            playSelectedNext()
        }
    }

    private suspend fun applySelectionResponse(response: SelectNextResponse) {
        // 新しい曲に切り替わるため、前の曲のループ設定は引き継がない
        _loopEnabled.value = false
        _selectedTrackId.value = response.nextTrackId

        val id = response.nextTrackId
        if (id == null) {
            _autoDjEnabled.value = false
            return
        }

        _selectionHistory.value = _selectionHistory.value + id

        _waveHistory.value = (
                _waveHistory.value + WavePoint(
                    energy = response.energy ?: 0f,
                    bpm = response.bpm ?: 0f,
                    keyPitchClass = pitchClassOf(response.keyName),
                    brightness = response.brightness ?: 0f,
                    energyImportance = response.energyImportance,
                    bpmImportance = response.bpmImportance,
                    brightnessImportance = response.brightnessImportance,
                    keyImportance = response.keyImportance
                )
                ).takeLast(WAVE_HISTORY_MAX)

        val trackFiles = trackFilesById[id]
        if (trackFiles == null) {
            _autoDjEnabled.value = false
            return
        }

        withContext(Dispatchers.Main) {
            controller?.setMediaItem(
                MusicStorage.mediaItemFor(trackFiles.audioSource, MusicStorage.displayTitle(id))
            )
            controller?.prepare()
            controller?.play()
        }
    }

    private suspend fun playSelectedNext() {
        val request = SelectNextRequest(
            energyMultiplier = _energyMultiplier.value,
            bpmMultiplier = _bpmMultiplier.value,
            keyMultiplier = _keyMultiplier.value,
            brightnessMultiplier = _brightnessMultiplier.value
        )
        val requestBytes = RustCodec.encodeSelectNextRequest(request)
        val responseBytes = RustBridge.selectNext(requestBytes)
        val response = RustCodec.decodeSelectNextResponse(responseBytes)
        applySelectionResponse(response)
    }

    /** setCurrentTrack(スコアリング無しの手動指定)を呼び、結果を反映する共通処理。 */
    private suspend fun setCurrentTrackAndApply(trackId: String) {
        val request = SetCurrentTrackRequest(trackId = trackId)
        val requestBytes = RustCodec.encodeSetCurrentTrackRequest(request)
        val responseBytes = RustBridge.setCurrentTrack(requestBytes)
        val response = RustCodec.decodeSelectNextResponse(responseBytes)
        applySelectionResponse(response)
    }

    /**
     * 曲一覧画面からユーザーが手動で選んだ曲を、現在の曲として即座に再生開始する。
     * selectNext(スコアリング)とは違い、指定した曲がそのまま採用される。
     * Auto DJは有効化され、この曲が終わったら通常通りselectNextによる自動選曲が続く
     * (ShortWave/BigWaveにもこの曲は反映されるため、以降の選曲にも履歴として影響する)。
     * 即座に割り込んで再生するため、予約済みの「次に再生」があればキャンセルする。
     */
    fun selectTrackManually(trackId: String) {
        _autoDjEnabled.value = true
        _queuedTrackId.value = null
        viewModelScope.launch(Dispatchers.IO) {
            setCurrentTrackAndApply(trackId)
        }
    }

    override fun onCleared() {
        controller?.release()
    }
}