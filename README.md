# Good-Badminton Studio

Desktop GUI for the **Good-Badminton** badminton video analysis pipeline — the C++ build with full parity against the original Python version. Built with **Tauri 2** (Rust shell) + **vanilla TypeScript** (no framework); it drives `gb_cpp.exe` as a subprocess.

![badge](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue) ![badge](https://img.shields.io/badge/Tauri-2-teal) ![badge](https://img.shields.io/badge/bun-1.4-orange)

## Features

- **Run** — pick a video and go: full parameter set (audio, language, 6 display toggles: skeletons, trajectories, player stats, pose ROI), live JSON progress, log console, cancel mid-run.
- **Court** — 4-corner court picker on a canvas, exports `annotations.txt` ready for the C++ CLI.
- **History** — list of past runs with preview of the result video (`detect_<name>.mp4`).
- **Professional dark UI** — "broadcast desk" theme, IBM Plex Sans/Mono, 100% offline (fonts bundled).
- **GPU auto-detect (Windows)** — an NVIDIA/AMD/Intel GPU is picked up automatically via DirectML; if no GPU works, the engine falls back to CPU with identical results. Set `GB_FORCE_CPU=1` to force CPU.

## Getting Started

1. **Run tab — set your inputs**
   - **Video**: browse to your match video (mp4/mov/…).
   - **Court template** *(optional)*: leave it empty — the pipeline automatically picks a frame from your video that passes court-corner detection (saved as `auto_template.png` in the output folder). Browse only if you want to supply your own PNG template.
   - **Output folder**: where results are written (defaults to `outputs/<video-name>`).
   - **Parameters**: audio on/off, language (`zh`/`en`), and the 6 display toggles. All default to `true`, matching the Python CLI.
   - Click **Run analysis** → progress bar + ETA + live log. **Cancel** stops mid-run. When finished, a **Open output folder** button appears.
2. **Court tab — (optional) corner annotations**
   - **Browse template…** to pick a court PNG, then click the **4 corner points** in order (chip `Points: 0/4` → `4/4`).
   - Adjust `mid_height` if needed (default 625), then **Save annotations** → the resulting `annotations.txt` is ready for the Run tab (`--annotations`).
3. **History tab — results & preview**
   - Every run is recorded: status (`ok`/`failed`/`cancelled`), time, duration, rally count.
   - Click a row → **preview the result video** (`detect_<video-name>.mp4`) plus run details in the side panel.
4. **Output** — the output folder contains the result video, `detections.jsonl` (per-frame), and `court_annotations.txt`.

## Requirements

- Download the installer for your OS from the GitHub Releases page — Windows (NSIS `.exe`), macOS (`.dmg`, Apple Silicon), or Linux (`.deb`/AppImage). The installer is self-contained: it already bundles the C++ engine (`gb_cpp` + runtime libraries) and both ONNX models, so no separate setup is needed. Install and run.
- [ffmpeg](https://ffmpeg.org/) on `PATH` (optional, for saving audio).
- [Bun](https://bun.sh) 1.4+ (development/build only).

Developers can still override the bundled engine via `config.json` (`gb_cpp_path`, `ball_model`, `pose_model`); sibling-folder auto-detection still works in development.

## Development

```bash
bun install
bun run tauri dev
```

## Build Installer

```bash
bun run tauri build        # Windows: NSIS; macOS: .app/.dmg; Linux: .deb/.AppImage
```

CI runs the same build on `v*` tags and attaches installers to the GitHub Release (see `.github/workflows/release.yml`).

## Configuration

`config.json` in the app config dir (Windows: `%APPDATA%\com.ulin.good-badminton-studio\`):

```json
{
  "gb_cpp_path": "C:/.../Good-Badminton-Cpp/build/Release/gb_cpp.exe",
  "data_dir": "C:/.../Good-Badminton-Data",
  "ball_model": ".../yolo11s-ball.onnx",
  "pose_model": ".../yolo11n-pose-dyn.onnx",
  "defaults": { "audio": true, "language": "zh", "show_skeletons": true, "...": true }
}
```

The six display toggles default to `true` for parity with the Python CLI. `progress-json` is always sent by the Studio (required for the progress bar) — it is opt-in and does not change default CLI behavior.

Manual smoke test: see [`docs/manual-smoke.md`](docs/manual-smoke.md).

## Credits

- **Upstream / primary reference:** [yo-WASSUP/Good-Badminton](https://github.com/yo-WASSUP/Good-Badminton) — the original Python "AI 羽毛球鹰眼系统" project (computer-vision badminton video analysis), along with its sibling series [Good-Tennis](https://github.com/yo-WASSUP/Good-Tennis) and [Good-Pickleball](https://github.com/yo-WASSUP/Good-Pickleball). Thanks for the ideas, architecture, and models.
- **Local Python pipeline `Good-Badminton`** — the reference implementation rewritten to C++ with 100% CLI parity (coverage/court/shuttle/rally), and the basis for this UI contract.
- **Good-Badminton-Cpp** — the processing engine (OpenCV 4 + ONNX Runtime, YOLO pose & shuttlecock).
- Models: `yolo11n-pose-dyn.onnx`, `yolo11s-ball.onnx` (from the upstream project).

## License

Licensed under the same license as the upstream project ([yo-WASSUP/Good-Badminton](https://github.com/yo-WASSUP/Good-Badminton)) — see [LICENSE](LICENSE).
