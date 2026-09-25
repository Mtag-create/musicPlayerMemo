package com.mymusicplayer.rust

data class MmzFileEntry(
    val id: String,
    val mmzBytes: ByteArray
)

data class InitLibraryRequest(
    val files: List<MmzFileEntry>
)

data class InitLibraryResponse(
    val success: Boolean,
    val trackCount: Int,
    val failedIds: List<String>
)

data class SelectNextRequest(
    val energyMultiplier: Float,
    val bpmMultiplier: Float,
    val keyMultiplier: Float,
    val brightnessMultiplier: Float
)

data class SetCurrentTrackRequest(
    val trackId: String
)

data class SelectNextResponse(
    val nextTrackId: String?,
    val energy: Float?,
    val bpm: Float?,
    val keyName: String?,
    val keyCorrelation: Float?,
    val brightness: Float?,
    val energyImportance: Float?,
    val bpmImportance: Float?,
    val brightnessImportance: Float?,
    val keyImportance: Float?
)