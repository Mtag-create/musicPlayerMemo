#!/data/data/com.termux/files/usr/bin/bash
# v "URL" : ライブ音源用。取得 → トリミング → MP3 を MyMusicFolder に作る(解析はしない)
set -e

URL="$1"
if [ -z "$URL" ]; then
  echo '使い方: v "URL"'
  exit 1
fi

BASE="$HOME/storage/downloads/MyMusicFolder"
PROJ="$HOME/mmp/MyMusicProject"

mkdir -p "$BASE"/{Music,Trim,MP3}

# 長いライブ音源の処理中にTermuxが止まらないようにする
termux-wake-lock
trap 'termux-wake-unlock' EXIT

echo "[1/4] 取得 (yt-dlp → Music/)"
yt-dlp --no-playlist -x --audio-format wav --restrict-filenames \
  --postprocessor-args "ffmpeg:-ar 44100 -ac 2" \
  -o "$BASE/Music/%(title)s.%(ext)s" \
  "$URL"

echo "[2/4] トリミング (Music → Trim)"
"$PROJ/MyMusicTrimmer/target/release/my_music_trimmer"

echo "[3/4] MP3エンコード (Trim → MP3)"
"$PROJ/MyMusicCompression/target/release/MyMusicCompression"

echo "[4/4] 索引を更新 (MMPの選択画面に出やすくする)"
for f in /storage/emulated/0/Download/MyMusicFolder/MP3/*; do
  am broadcast -a android.intent.action.MEDIA_SCANNER_SCAN_FILE -d "file://$f" > /dev/null 2>&1 || true
done

echo "完了: $BASE/MP3 (MMPでは「曲を追加」→ live_music を選んで取り込む)"
