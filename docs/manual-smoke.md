# Manual GUI Smoke Checklist

Human-run visual smoke test for Good-Badminton Studio. The automated gates
(`cargo test`, `cargo check`, `bun run build`, CLI parity, NSIS build) are
covered by Task 9 automation — this file covers what a subagent cannot do
without a GUI. Run these steps after `bun run tauri build` succeeds.

## Setup

- OS: Windows 11
- Prereqs: Rust toolchain, Bun, Tauri prerequisites (WebView2), optional ffmpeg
- Test media: `C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton/videos/test4.mp4`
- Test template: `C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton/templates/test4.png`

## Steps

1. Run `bun run tauri dev` from the repo root → the app window opens and the
   Run tab shows the default paths (`gb_cpp` auto-detected) and the
   `ffmpeg_ok` flag matches whether ffmpeg is actually installed.
2. Fill video = `Good-Badminton/videos/test4.mp4`, template =
   `Good-Badminton/templates/test4.png`, out = an empty folder, then press
   Run → the progress bar moves, the ETA counts down, the log fills; when it
   finishes → a "Buka folder output" button appears (clicking it opens the
   folder in Explorer); `detections.jsonl` has ≥ 5871 lines; the History tab
   shows the new entry with a playable preview.
3. Run again, press Cancel midway → the `run-finished` event reports status
   `cancelled` and the `gb_cpp.exe` process is gone (verify in Task Manager).

## Sign-off

- [ ] Step 1 passed (window opens, paths + ffmpeg_ok correct)
- [ ] Step 2 passed (progress, output button, detections.jsonl ≥ 5871 lines, history preview)
- [ ] Step 3 passed (cancel → `cancelled`, process dead)

Tester: ______________  Date: ______________
