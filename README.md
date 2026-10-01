# Good-Badminton Studio

Desktop GUI untuk pipeline analisis video bulu tangkis **Good-Badminton** — build dengan C++ (paritas penuh terhadap versi Python). Dibangun dengan **Tauri 2** (Rust shell) + **vanilla TypeScript** (tanpa framework), meng-gb_cpp.exe sebagai subprocess.

![badge](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue) ![badge](https://img.shields.io/badge/Tauri-2-teal) ![badge](https://img.shields.io/badge/bun-1.4-orange)

## Fitur

- **Run** — pilih video & template, parameter lengkap (audio, bahasa, 6 toggle display: skeletons, trajectories, player stats, pose ROI), progress JSON live, log console, cancel di tengah jalan.
- **Court** — picker 4 sudut lapangan di atas template (canvas), simpan `annotations.txt` siap pakai C++.
- **History** — daftar run sebelumnya + preview video hasil (`detect_<nama>.mp4`).
- **Professional dark UI** — tema "broadcast desk", IBM Plex Sans/Mono, 100% offline (font dibundel).

## Prasyarat

- Binary **Good-Badminton-Cpp** (`gb_cpp`) — deteksi otomatis dari folder sibling `Good-Badminton-Cpp`, atau arahkan manual lewat `config.json` (path tampil di pesan error first-run).
- [ffmpeg](https://ffmpeg.org/) di `PATH` (opsional, untuk simpan audio).
- [Bun](https://bun.sh) 1.4+ (hanya untuk development/build).

## Development

```bash
bun install
bun run tauri dev
```

## Build installer

```bash
bun run tauri build        # Windows: NSIS; macOS: .app/.dmg; Linux: .deb/.AppImage
```

CI menjalankan build yang sama di tag `v*` dan melampirkan installer ke GitHub Release (lihat `.github/workflows/release.yml`).

## Konfigurasi

`config.json` di app config dir (Windows: `%APPDATA%\com.ulin.good-badminton-studio\`):

```json
{
  "gb_cpp_path": "C:/.../Good-Badminton-Cpp/build/Release/gb_cpp.exe",
  "data_dir": "C:/.../Good-Badminton-Data",
  "ball_model": ".../yolo11s-ball.onnx",
  "pose_model": ".../yolo11n-pose-dyn.onnx",
  "defaults": { "audio": true, "language": "zh", "show_skeletons": true, "...": true }
}
```

Enam toggle display **default `true`** = paritas dengan CLI Python. `progress-json` selalu dikirim dari Studio (wajib untuk progress bar) — flag itu opt-in, tidak mengubah perilaku default CLI.

Smoke test manual: lihat [`docs/manual-smoke.md`](docs/manual-smoke.md).

## Credits

- **Upstream / referensi utama:** [yo-WASSUP/Good-Badminton](https://github.com/yo-WASSUP/Good-Badminton) — proyek asli Python "AI 羽毛球鹰眼系统" (analisis video bulu tangkis berbasis computer vision), beserta seri setenya [Good-Tennis](https://github.com/yo-WASSUP/Good-Tennis) dan [Good-Pickleball](https://github.com/yo-WASSUP/Good-Pickleball). Terima kasih atas ide, arsitektur, dan modelnya.
- **Pipeline Python lokal `Good-Badminton`** — implementasi acuan (reference) yang di-rewrite ke C++ dengan paritas CLI 100% (coverage/court/shuttle/rally), dan menjadi dasar kontrak UI ini.
- **Good-Badminton-Cpp** — mesin pemrosesnya (OpenCV 4 + ONNX Runtime, YOLO pose & shuttlecock).
- Model: `yolo11n-pose-dyn.onnx`, `yolo11s-ball.onnx` (dari proyek upstream).

## Lisensi

Ikuti lisensi proyek upstream ([yo-WASSUP/Good-Badminton](https://github.com/yo-WASSUP/Good-Badminton)).
