# Good-Badminton Studio — Desktop App (Tauri 2 + Rust)

Date: 2026-10-01
Status: approved (user: "pakai rust ya, folder baru")

## Goal

Aplikasi desktop Windows untuk menjalankan pipeline Good-Badminton
(`gb_cpp.exe`, parity 100% lulus) dengan UI — tanpa menyentuh logic
pipeline. Cross-platform shell (Win/macOS/Linux); v1 target Windows.

## Architecture

```
Tauri 2 (Rust shell) + web UI (vanilla TS + Vite, no React)
   │  spawn gb_cpp.exe, stream stdout (--progress-json)
   ▼
gb_cpp.exe   ← subprocess, black box. Satu-satunya perubahan di
                Good-Badminton-Cpp: flag --progress-json (opt-in,
                emit {"frame":N,"total":M} per frame via
                progress_callback; default perilaku CLI tak berubah).
```

Folder: `C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Studio`
(sibling Good-Badminton & Good-Badminton-Cpp).

## Features v1 (user memilih semua)

1. **Dasar**: pilih video, pilih template PNG (atau pakai picker sudut),
   opsi audio/en-zh, tombol Run/Batal (kill child), progress bar + ETA,
   buka folder output.
2. **Picker sudut lapangan**: canvas — tampilkan template, klik 4 titik
   berurutan, simpan `court_annotations.txt` format persis Python
   (eval()-compatible; port helper dari Good-Badminton-Cpp/src/system.cpp
   fmt_* / court annotation writer).
3. **Preview hasil**: `<video>` via Tauri asset protocol → putar
   `outputs/<name>/detect_<name>.mp4` setelah sukses.
4. **Riwayat**: `history.json` di app config dir — path video, waktu,
   status (ok/failed/cancelled), durasi proses, jumlah rally (dibaca
   dari metadata.json / jsonl output).

## Pipeline integration

- Config (app data dir `config.json`): `gb_cpp_path`,
  `data_dir` (root Good-Badminton utk weights/templates), default
  nyari di sibling folder otomatis.
- Progress: parsing 1 baris JSON per event dari stdout; stderr → log pane.
- ffmpeg/ffprobe: wajib di PATH (sama seperti CLI); dicek saat start,
  warning kalau tidak ada (video tetap jalan tanpa audio).
- Exit code: 0 ok, 2 model-not-found, 1 error → tampilkan pesan.

## Non-goals v1

Heatmaps, webui lama, multi-bahasa UI (hanya toggle en/zh untuk
pipeline), port pipeline ke macOS/Linux (tahap berikutnya),
real-time pose preview in-app. Port pipeline ke Rust juga non-goal:
target macOS/Linux dicapai dengan **compile C++ per platform**
(ONNX Runtime + OpenCV tersedia di sana) — bukan rewrite. Tahap
berikutnya setelah v1 Windows: build C++ di mac/Linux + installer
per-OS (dmg/AppImage).

## Risks / notes

- WebView2: sudah standar di Win11; Tauri bundle fallback kalau perlu.
- Parity: tidak mungkin terganggu — pipeline tetap exe terpisah;
  perubahan C++ hanya flag opsional (wajib: test CLI lama tanpa flag
  tetap identik output).
- Installer: Tauri NSIS bundler; gb_cpp.exe + weights di-bundle di
  versi installer, path override untuk dev.
