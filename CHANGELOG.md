# Changelog

All notable changes to Good-Badminton Studio are documented here.

## v1.5 (2026-10-02)

- **Output codec choice (Run tab)** — pick the final export codec:
  **H.264** (default, fast and compatible with every player) or **H.265**
  (smaller files, slower to encode). The choice is sent to the engine as
  `--output-codec h264|h265` and remembered in `config.json`.

## v1.4.1 (2026-10-02)

- **Fixed macOS crash at analysis start** — the engine freed the ONNX
  `TypeInfo` object before reading the model input shape (use-after-free),
  which segfaulted (exit 139) right after "Initializing YOLO pose model" on
  Apple Silicon; Windows only survived by luck. Fixed in the engine
  (`onnx_util.h`), and CI now loads both ONNX models on every push on
  Windows + macOS so this class of bug can no longer ship.
- **In-app update check** — on startup the app checks the latest GitHub
  release and shows a dismissible banner when a newer version exists, with
  an **Unduh** button that opens the release page in the default browser.
  Offline or on any failure it stays silent (offline-first); a dismissed
  version never re-appears, but a newer one does.

## v1.4 (2026-10-02)

- **Far-side players now detected** — the bundled engine raises the pose
  input size (960 → 1600) and lowers the confidence threshold (0.15 →
  0.10), so players on the far side of the court are tracked instead of
  being dropped (verified: far-side occupancy 0% → 89% on a portrait
  video). Slightly slower on CPU (~1.4× pose time); extra detections are
  filtered by the on-court check. Override with `--pose-conf` /
  `--pose-imgsz`.

## v1.2 (2026-10-01)

- **Self-contained installers** — the C++ engine (`gb_cpp` + runtime
  libraries) and both ONNX models (`yolo11s-ball.onnx`,
  `yolo11n-pose-dyn.onnx`) are now bundled into the installer; install and
  run with no separate setup.
- Engine resource lookup order: `config.json` (user override) → bundled
  resources → sibling folder (development).
- The macOS installer is now Apple Silicon (arm64) native instead of
  universal.
- The engine rebuilds across all three platforms in CI; version bumped to
  1.2.0.
- **GPU auto-detect (Windows)** — the bundled ONNX Runtime build enables
  DirectML: NVIDIA/AMD/Intel GPUs are detected automatically, with
  automatic CPU fallback when the GPU path is unavailable or fails
  (results identical to CPU). Set `GB_FORCE_CPU=1` to force CPU.

## v1.1 (2026-10-01)

- **Optional court template** — the Run tab no longer requires a template PNG.
  Leave the field empty and the pipeline automatically picks a frame from the
  input video that passes court-corner detection (saved as `auto_template.png`
  in the output folder).
- Fixed the Run tab going blank after switching to the Court or History tab
  (view re-render selector).
- Actionable first-run error when `gb_cpp` cannot be located.
- Documentation: usage guide in the README, English release notes.

## v1 (2026-10-01)

- First release.
- Run view: video/template/output pickers, audio & language parameters,
  six display toggles (default `true` = Python CLI parity), live JSON
  progress with ETA, log console, mid-run cancel.
- Court view: 4-corner picker with `mid_height`, exports `annotations.txt`.
- History view: past runs with result-video preview.
- Professional dark "broadcast desk" UI, fully offline fonts.
- Cross-platform installers: Windows (NSIS), macOS (universal), Linux
  (deb/AppImage).
