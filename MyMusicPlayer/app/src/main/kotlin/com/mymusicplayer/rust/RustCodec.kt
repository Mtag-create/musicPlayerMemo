package com.mymusicplayer.rust

import org.msgpack.core.MessagePack

/**
 * Kotlin↔Rust間のJNI通信用MessagePackエンコード/デコード。
 *
 * Rust側(protocol.rs)はマップ形式(with_struct_map())でシリアライズしているため、
 * ここでもマップのキーとしてRust側の構造体フィールド名(スネークケース)を
 * そのまま文字列で使う。フィールド名が一致しないとRust側でデコードに失敗する。
 */
object RustCodec {

    fun encodeInitLibraryRequest(request: InitLibraryRequest): ByteArray {
        val packer = MessagePack.newDefaultBufferPacker()

        packer.packMapHeader(1)
        packer.packString("files")
        packer.packArrayHeader(request.files.size)
        for (file in request.files) {
            packer.packMapHeader(2)
            packer.packString("id")
            packer.packString(file.id)
            packer.packString("mmz_bytes")
            packer.packBinaryHeader(file.mmzBytes.size)
            packer.writePayload(file.mmzBytes)
        }

        packer.close()
        return packer.toByteArray()
    }

    fun decodeInitLibraryResponse(bytes: ByteArray): InitLibraryResponse {
        val unpacker = MessagePack.newDefaultUnpacker(bytes)

        val fieldCount = unpacker.unpackMapHeader()
        var success = false
        var trackCount = 0
        val failedIds = mutableListOf<String>()

        repeat(fieldCount) {
            when (unpacker.unpackString()) {
                "success" -> success = unpacker.unpackBoolean()
                "track_count" -> trackCount = unpacker.unpackInt()
                "failed_ids" -> {
                    val n = unpacker.unpackArrayHeader()
                    repeat(n) { failedIds.add(unpacker.unpackString()) }
                }
                else -> unpacker.skipValue()
            }
        }

        unpacker.close()
        return InitLibraryResponse(success, trackCount, failedIds)
    }

    fun encodeSelectNextRequest(request: SelectNextRequest): ByteArray {
        val packer = MessagePack.newDefaultBufferPacker()

        packer.packMapHeader(4)
        packer.packString("energy_multiplier")
        packer.packFloat(request.energyMultiplier)
        packer.packString("bpm_multiplier")
        packer.packFloat(request.bpmMultiplier)
        packer.packString("key_multiplier")
        packer.packFloat(request.keyMultiplier)
        packer.packString("brightness_multiplier")
        packer.packFloat(request.brightnessMultiplier)

        packer.close()
        return packer.toByteArray()
    }

    fun encodeSetCurrentTrackRequest(request: SetCurrentTrackRequest): ByteArray {
        val packer = MessagePack.newDefaultBufferPacker()

        packer.packMapHeader(1)
        packer.packString("track_id")
        packer.packString(request.trackId)

        packer.close()
        return packer.toByteArray()
    }

    fun decodeSelectNextResponse(bytes: ByteArray): SelectNextResponse {
        val unpacker = MessagePack.newDefaultUnpacker(bytes)

        val fieldCount = unpacker.unpackMapHeader()
        var nextTrackId: String? = null
        var energy: Float? = null
        var bpm: Float? = null
        var keyName: String? = null
        var keyCorrelation: Float? = null
        var brightness: Float? = null
        var energyImportance: Float? = null
        var bpmImportance: Float? = null
        var brightnessImportance: Float? = null
        var keyImportance: Float? = null

        repeat(fieldCount) {
            when (unpacker.unpackString()) {
                "next_track_id" -> {
                    nextTrackId = if (unpacker.tryUnpackNil()) null else unpacker.unpackString()
                }
                "energy" -> {
                    energy = if (unpacker.tryUnpackNil()) null else unpacker.unpackFloat()
                }
                "bpm" -> {
                    bpm = if (unpacker.tryUnpackNil()) null else unpacker.unpackFloat()
                }
                "key_name" -> {
                    keyName = if (unpacker.tryUnpackNil()) null else unpacker.unpackString()
                }
                "key_correlation" -> {
                    keyCorrelation = if (unpacker.tryUnpackNil()) null else unpacker.unpackFloat()
                }
                "brightness" -> {
                    brightness = if (unpacker.tryUnpackNil()) null else unpacker.unpackFloat()
                }
                "energy_importance" -> {
                    energyImportance = if (unpacker.tryUnpackNil()) null else unpacker.unpackFloat()
                }
                "bpm_importance" -> {
                    bpmImportance = if (unpacker.tryUnpackNil()) null else unpacker.unpackFloat()
                }
                "brightness_importance" -> {
                    brightnessImportance = if (unpacker.tryUnpackNil()) null else unpacker.unpackFloat()
                }
                "key_importance" -> {
                    keyImportance = if (unpacker.tryUnpackNil()) null else unpacker.unpackFloat()
                }
                else -> unpacker.skipValue()
            }
        }

        unpacker.close()
        return SelectNextResponse(
            nextTrackId, energy, bpm, keyName, keyCorrelation, brightness,
            energyImportance, bpmImportance, brightnessImportance, keyImportance
        )
    }
}