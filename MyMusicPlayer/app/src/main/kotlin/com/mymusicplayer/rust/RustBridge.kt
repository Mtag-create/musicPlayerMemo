package com.mymusicplayer.rust

object RustBridge {
    init {
        System.loadLibrary("algorithm")
    }

    external fun pingRust(input: ByteArray): ByteArray
    external fun initLibrary(input: ByteArray): ByteArray
    external fun selectNext(input: ByteArray): ByteArray
    external fun setCurrentTrack(input: ByteArray): ByteArray
}