#!/usr/bin/env bash
# Stage engine + models ke src-tauri/bundle/{engine,models} (target bundle.resources).
# Pakai: scripts/stage-bundle.sh <cpp-src-dir>   (CI: cpp-engine-src, lokal: ../Good-Badminton-Cpp)
set -euo pipefail
cd "$(dirname "$0")/.."

CPP=${1:?usage: stage-bundle.sh <cpp-src-dir>}
CPP=$(cd "$CPP" && pwd)
OUT=src-tauri/bundle
rm -rf "$OUT/engine" "$OUT/models"
mkdir -p "$OUT/engine" "$OUT/models"
# .gitkeep selamat dari wipe — fresh clone tetap punya direktori resource
touch "$OUT/engine/.gitkeep" "$OUT/models/.gitkeep"

# --- models: lokal dulu (dev), kalau tidak ada tarik dari release cpp v1 ---
REL=${GB_MODELS_URL:-https://github.com/wafik/good-badminton-cpp/releases/download/v1}
for f in yolo11s-ball.onnx yolo11n-pose-dyn.onnx; do
  src=""
  for c in "../Good-Badminton/weights/$f" "$CPP/weights/$f" "weights/$f"; do
    [ -f "$c" ] && { src="$c"; break; }
  done
  if [ -n "$src" ]; then cp "$src" "$OUT/models/"
  else curl -fsSL --retry 3 -o "$OUT/models/$f" "$REL/$f"; fi
done

case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*)
    BIN="$CPP/build/Release"
    cp "$BIN/gb_cpp.exe" "$OUT/engine/"
    # onnxruntime build DirectML (export provider GPU — auto-detect di runtime).
    # Di-download dari nupkg Microsoft.ML.OnnxRuntime.DirectML (lihat release.yml);
    # build CPU biasa TIDAK punya export ini → engine jalan tapi GPU mati.
    DML="$CPP/third_party/onnxruntime-directml-1.19.2/onnxruntime.dll"
    [ -f "$DML" ] || { echo "FATAL: $DML tidak ada — tarik nupkg DirectML dulu (lihat release.yml, Engine deps Windows)"; exit 1; }
    cp "$DML" "$OUT/engine/"
    # paket resmi OpenCV: x64/vc16/bin (glob biar tahan versi)
    for d in "$CPP"/third_party/opencv/build/x64/*/bin; do
      cp "$d"/opencv_world4100.dll "$d"/opencv_videoio_ffmpeg4100_64.dll "$d"/opencv_videoio_msmf4100_64.dll "$OUT/engine/" 2>/dev/null || true
    done
    # fallback: dll sudah nangkring di build Release (mesin dev)
    if [ ! -f "$OUT/engine/opencv_world4100.dll" ]; then
      cp "$BIN"/opencv_world*.dll "$BIN"/opencv_videoio_*.dll "$OUT/engine/" 2>/dev/null || true
    fi
    # CRT agar jalan di Windows bersih (app-local redistributable)
    for d in msvcp140.dll vcruntime140.dll vcruntime140_1.dll msvcp140_1.dll msvcp140_2.dll; do
      cp "/c/Windows/System32/$d" "$OUT/engine/" 2>/dev/null || true
    done
    ;;
  Darwin)
    cp "$CPP/build/gb_cpp" "$OUT/engine/"
    for f in "$CPP"/third_party/onnxruntime-osx-arm64-1.19.2/lib/libonnxruntime*.dylib; do
      [ -f "$f" ] && cp -L "$f" "$OUT/engine/$(basename "$f")"
    done
    # tarik dependensi brew (@rpath) jadi sebelah binary
    dylibbundler -b -x "$OUT/engine/gb_cpp" -d "$OUT/engine" -p @loader_path -of
    install_name_tool -add_rpath @loader_path "$OUT/engine/gb_cpp" 2>/dev/null || true
    ;;
  *)
    cp "$CPP/build/gb_cpp" "$OUT/engine/"
    for f in "$CPP"/third_party/onnxruntime-linux-x64-1.19.2/lib/libonnxruntime.so*; do
      [ -f "$f" ] && cp -L "$f" "$OUT/engine/$(basename "$f")"
    done
    # dependensi non-glibc (opencv, ffmpeg, libstdc++ dst) ikut dibundel
    ldd "$OUT/engine/gb_cpp" | awk '/=> \//{print $3} $1 ~ /^\//{print $1}' | while read -r lib; do
      case "$lib" in
        *libc.so*|*libm.so*|*libpthread*|*libdl.so*|*librt.so*|*ld-linux*|*libresolv*|*libutil.so*|*libnsl*) continue ;;
      esac
      cp -Ln "$lib" "$OUT/engine/" 2>/dev/null || true
    done
    ;;
esac

echo "staged:" && ls -la "$OUT/engine" | head -20 && ls -la "$OUT/models"
