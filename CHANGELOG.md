# Changelog

All notable changes to Good-Badminton Studio are documented here.

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
