#!/usr/bin/env bash
# Smoke: binary hasil stage jalan pakai runtime di sebelahnya (--help = exit 0),
# DAN model ONNX benar-benar bisa di-load (segfault/abort di session creation
# harus gagalkan CI — celah yang bikin bug macOS lolos v1.4).
set -euo pipefail
cd "$(dirname "$0")/.."
E=src-tauri/bundle/engine
M=src-tauri/bundle/models
if [ -f "$E/gb_cpp.exe" ]; then BIN="$E/gb_cpp.exe"; else BIN="$E/gb_cpp"; fi
[ -f "$BIN" ] || { echo "engine belum distage"; exit 1; }
out=$("$BIN" --help)
echo "$out" | grep -q -- "-h, --help" || { echo "usage tidak tercetak"; exit 1; }
echo "smoke help ok: $BIN"

# --- model-load smoke -------------------------------------------------------
# System ctor order: fs::exists(video) -> fs checks model -> pose model ->
# ball model -> (setelah itu) process_video buka video. Video dummy kosong
# bikin VideoCapture gagal -> exit 1 (runtime_error), jadi exit 1/4 hanya bisa
# tercapai SETELAH kedua model ter-load. Exit 2 = model load error; sinyal
# (134/139, atau 0xC0000005 di Windows) = crash — semua di luar {1,4} = GAGAL.
[ -f "$M/yolo11s-ball.onnx" ] && [ -f "$M/yolo11n-pose-dyn.onnx" ] \
  || { echo "models belum distage di $M"; exit 1; }
TMP=$(mktemp -d "${TMPDIR:-/tmp}/gb-smoke.XXXXXX")  # template wajib di BSD mktemp (macOS)
trap 'rm -rf "$TMP"' EXIT
: > "$TMP/dummy.mp4"   # cukup ada (fs::exists di ctor); isi kosong
# GB_FORCE_CPU=1: deterministik — DirectML/D3D12 di-runner bisa gagal daftar
# provider; smoke ini menguji model load, bukan GPU.
rc=0
GB_FORCE_CPU=1 "$BIN" "$TMP/dummy.mp4" \
  --ball-model "$M/yolo11s-ball.onnx" \
  --yolo-pose-model "$M/yolo11n-pose-dyn.onnx" \
  --out "$TMP/out" > "$TMP/run.log" 2>&1 || rc=$?
if [ "$rc" -ne 1 ] && [ "$rc" -ne 4 ]; then
  echo "smoke model GAGAL (exit $rc; 139/134 = segfault/abort, 2 = model load error)"
  echo "---- output ----"; cat "$TMP/run.log"; exit 1
fi
# guard: arg-parse/fs-check error di awal juga exit 1 — wajib sampai ke pose model
if ! grep -q "Initializing YOLO pose model" "$TMP/run.log"; then
  echo "smoke model GAGAL: tidak sampai ke inisialisasi pose model (exit $rc)"
  echo "---- output ----"; cat "$TMP/run.log"; exit 1
fi
if grep -Eq "failed to load|model not found" "$TMP/run.log"; then
  echo "smoke model GAGAL: ada model yang gagal load (exit $rc)"
  echo "---- output ----"; cat "$TMP/run.log"; exit 1
fi
echo "smoke model ok: kedua model ter-load (exit $rc)"
echo "smoke ok: $BIN"
