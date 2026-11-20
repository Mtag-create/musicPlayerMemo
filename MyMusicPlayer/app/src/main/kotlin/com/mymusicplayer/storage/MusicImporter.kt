package com.mymusicplayer.storage

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import java.io.File

/** SAFで選ばれたファイルを、アプリ内ストレージの music/ または live_music/ にコピーする。 */
object MusicImporter {

    enum class Destination(val dirName: String, val allowedExtensions: Set<String>) {
        MUSIC("music", setOf("mp3", "mmz")),
        LIVE("live_music", setOf("mp3"))
    }

    data class Result(
        val copied: List<String>,
        val skipped: List<String>,
        val failed: List<String>
    )

    fun importFiles(context: Context, uris: List<Uri>, destination: Destination): Result {
        val dir = File(context.filesDir, destination.dirName).apply { mkdirs() }

        val copied = mutableListOf<String>()
        val skipped = mutableListOf<String>()
        val failed = mutableListOf<String>()

        for (uri in uris) {
            val rawName = displayName(context, uri)
            if (rawName == null) {
                failed.add(uri.toString())
                continue
            }

            // パス区切りを含む名前でも、dir直下にしか書かないようにする
            val name = File(rawName).name
            val extension = name.substringAfterLast('.', "").lowercase()

            if (extension !in destination.allowedExtensions) {
                skipped.add(name)
                continue
            }

            val target = File(dir, name)
            val temp = File(dir, "$name.tmp")

            try {
                val input = context.contentResolver.openInputStream(uri)
                    ?: throw IllegalStateException("cannot open $uri")
                input.use { src ->
                    temp.outputStream().use { dst -> src.copyTo(dst) }
                }
                if (!temp.renameTo(target)) {
                    throw IllegalStateException("rename failed: $name")
                }
                copied.add(name)
            } catch (e: Exception) {
                temp.delete()
                failed.add(name)
            }
        }

        return Result(copied, skipped, failed)
    }

    private fun displayName(context: Context, uri: Uri): String? {
        return context.contentResolver
            .query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)
            ?.use { cursor ->
                if (cursor.moveToFirst()) cursor.getString(0) else null
            }
    }
}