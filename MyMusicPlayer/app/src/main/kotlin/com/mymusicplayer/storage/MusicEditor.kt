package com.mymusicplayer.storage

import android.content.Context
import java.io.File

/** filesDir内の曲(music/ と live_music/)の一覧取得と削除。assetsの曲は対象外。 */
object MusicEditor {

    data class EditableTrack(
        val destination: MusicImporter.Destination,
        val baseName: String,
        val files: List<File>
    ) {
        val key: String get() = "${destination.dirName}/$baseName"
    }

    fun listEditableTracks(context: Context): List<EditableTrack> {
        return MusicImporter.Destination.values().flatMap { destination ->
            val dir = File(context.filesDir, destination.dirName)
            val mp3Files = dir.listFiles { file ->
                file.isFile && file.extension.equals("mp3", ignoreCase = true)
            } ?: emptyArray()

            mp3Files.map { mp3 ->
                val baseName = mp3.nameWithoutExtension
                val files = mutableListOf(mp3)
                if (destination == MusicImporter.Destination.MUSIC) {
                    val mmz = File(dir, "$baseName.mmz")
                    if (mmz.exists()) files.add(mmz)
                }
                EditableTrack(destination, baseName, files)
            }
        }.sortedWith(compareBy({ it.destination.ordinal }, { it.baseName }))
    }

    /** 曲のファイルをすべて削除する。1つでも消せなければfalse。 */
    fun delete(track: EditableTrack): Boolean {
        var allDeleted = true
        for (file in track.files) {
            if (file.exists() && !file.delete()) allDeleted = false
        }
        return allDeleted
    }

    enum class RenameResult {
        SUCCESS, EMPTY, INVALID_NAME, UNCHANGED, ALREADY_EXISTS, FAILED
    }

    private val INVALID_NAME_CHARS = charArrayOf('/', '\\', ':', '*', '?', '"', '<', '>', '|')

    /** 曲のファイル(.mp3、musicなら.mmzも)の名前を、まとめて変える。 */
    fun rename(track: EditableTrack, newNameRaw: String): RenameResult {
        val newName = newNameRaw.trim()

        if (newName.isEmpty()) return RenameResult.EMPTY
        if (newName.any { it in INVALID_NAME_CHARS || it.isISOControl() } ||
            newName.startsWith(".") ||
            newName.toByteArray(Charsets.UTF_8).size > 200
        ) {
            return RenameResult.INVALID_NAME
        }
        if (newName == track.baseName) return RenameResult.UNCHANGED

        val dir = track.files.first().parentFile ?: return RenameResult.FAILED

        val pairs = track.files.map { file ->
            file to File(dir, "$newName.${file.extension}")
        }

        if (pairs.any { (_, target) -> target.exists() }) {
            return RenameResult.ALREADY_EXISTS
        }

        val done = mutableListOf<Pair<File, File>>()
        for ((from, to) in pairs) {
            if (from.renameTo(to)) {
                done.add(from to to)
            } else {
                // 失敗したら、すでに変更した分を元に戻す
                done.forEach { (original, renamed) -> renamed.renameTo(original) }
                return RenameResult.FAILED
            }
        }

        return RenameResult.SUCCESS
    }
}