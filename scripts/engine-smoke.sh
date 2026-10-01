#!/usr/bin/env bash
# Smoke: binary hasil stage jalan pakai runtime di sebelahnya (--help = exit 0).
set -euo pipefail
cd "$(dirname "$0")/.."
E=src-tauri/bundle/engine
if [ -f "$E/gb_cpp.exe" ]; then BIN="$E/gb_cpp.exe"; else BIN="$E/gb_cpp"; fi
[ -f "$BIN" ] || { echo "engine belum distage"; exit 1; }
out=$("$BIN" --help)
echo "$out" | grep -q -- "-h, --help" || { echo "usage tidak tercetak"; exit 1; }
echo "smoke ok: $BIN"
