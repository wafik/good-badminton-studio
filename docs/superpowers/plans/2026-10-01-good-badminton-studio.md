# Good-Badminton Studio Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Desktop app (Tauri 2 + Rust + Bun/vanilla-TS) yang menjalankan `gb_cpp.exe` sebagai subprocess dengan UI: setup run, picker sudut lapangan, progress, preview hasil, riwayat.

**Architecture:** Tauri shell (Rust) men-spawn `gb_cpp.exe` dengan flag baru `--progress-json`, membaca stdout line-based (JSON progress / plain log), emit event ke web UI via Tauri event API. Pipeline tidak diubah logikanya — parity 100% tetap tersegel. Folder baru `Good-Badminton-Studio/` sampingan.

**Tech Stack:** Tauri 2, Rust 1.98, Bun 1.4 (package manager + runner — **JANGAN pakai npm/node**), Vite, TypeScript vanilla (tanpa framework), tokio::process.

## Global Constraints

- Package manager: **Bun only** (`bun install`, `bun run`, `bun x`). Jangan pakai npm/npx/node.
- Target: Windows x64 (MSVC). Repo folder: `C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Studio`.
- Pipeline `Good-Badminton-Cpp` hanya boleh **ditambah flag opsional** `--progress-json` (default off ⇒ perilaku CLI identik).
- Anotasi lapangan harus byte-format persis yang dibaca pipeline: 3 baris `corners=[(x, y), ...]`, `roi_corners=[(x, y), (x, y)]`, `mid_height=<int>` (reader: `src/system.cpp:235-273` — line 2 di-skip, roi selalu recompute).
- Model/paths default: sibling `Good-Badminton-Cpp/build/Release/gb_cpp.exe`, data dir `Good-Badminton/` (weights `yolo11n-pose-dyn.onnx` + `yolo11s-ball.onnx`), semuanya di-overridable via config.
- Rally count di riwayat = port aturan `compare_parity.py:rally_count` (START_HITS=3, WINDOW=2.0, QUIET=4.0) di atas jsonl.
- Git: init di folder Studio, commit tiap task.

## File Structure

```
Good-Badminton-Studio/
├── docs/superpowers/plans/2026-10-01-good-badminton-studio.md  (plan ini)
├── package.json, vite.config.ts, tsconfig.json, index.html
├── src/                      # frontend
│   ├── main.ts               # nav + state + event wiring
│   ├── api.ts                # invoke() wrappers + types
│   ├── setup.ts              # view: form run + progress + log
│   ├── corners.ts            # view: canvas picker 4 titik
│   ├── history.ts            # view: riwayat + preview video
│   └── styles.css
└── src-tauri/
    ├── Cargo.toml, tauri.conf.json, build.rs, capabilities/, icons/
    └── src/
        ├── main.rs           # entry (scaffold)
        ├── lib.rs            # Tauri builder + semua #[tauri::command]
        ├── progress.rs       # parse baris stdout -> Progress {frame,total}
        ├── rally.rs          # rally_count(jsonl) port Python rule
        ├── annotations.rs    # tulis court_annotations.txt 3 baris
        ├── history.rs        # history.json load/push
        ├── config.rs         # config.json + auto-detect sibling paths
        └── pipeline.rs       # spawn/cancel gb_cpp + emit events
```

---

### Task 1: Flag `--progress-json` di gb_cpp

**Files:**
- Modify: `Good-Badminton-Cpp/src/system.h:34-51` (SystemOptions)
- Modify: `Good-Badminton-Cpp/src/main.cpp` (arg parse + help)
- Modify: `Good-Badminton-Cpp/src/system.cpp:410-416` (loop)
- Create: `Good-Badminton-Cpp/tests/fixtures/short.mp4` (potongan 5 detik)

**Interfaces:**
- Produces: CLI flag `--progress-json` → stdout per frame: `{"frame":<int>,"total":<int>}\n` (setelah `process_frame`, `fflush(stdout)`). Default TANPA flag = tidak ada baris JSON (regresi wajib dicek).

- [ ] **Step 1: Buat fixture pendek**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Cpp
mkdir -p tests/fixtures
ffmpeg -y -t 5 -i "C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton/videos/test4.mp4" -c copy tests/fixtures/short.mp4
```

- [ ] **Step 2: Tambah field ke SystemOptions** (`system.h`, setelah baris `show_performance_stats`)

```cpp
    bool progress_json = false;      // studio: emit {"frame":N,"total":M}/line to stdout
```

- [ ] **Step 3: Parse flag + help** (`main.cpp`, di blok arg parse dekat `--audio`, dan satu baris di `print_usage`)

```cpp
        } else if (a == "--progress-json") {
            opts.progress_json = true;
        }
```
```cpp
                 "  --progress-json            emit one {\"frame\":N,\"total\":M} JSON line per frame\n"
```

- [ ] **Step 4: Emit di loop** (`system.cpp`, di dalam `while (cap.read(frame))` setelah `process_frame(...)`)

```cpp
        if (opts_.progress_json) {
            std::printf("{\"frame\":%d,\"total\":%d}\n", frame_count, total_frames);
            std::fflush(stdout);
        }
```

- [ ] **Step 5: Build**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Cpp/build
cmake --build . --config Release --target gb_cpp
```
Expected: exit 0.

- [ ] **Step 6: Verifikasi WITH flag — ada baris JSON**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Cpp
./build/Release/gb_cpp.exe tests/fixtures/short.mp4 --template "C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton/templates/test4.png" --out outputs/t1flag --progress-json > /tmp/t1.out 2>/dev/null
grep -c '^{"frame":' /tmp/t1.out
```
Expected: > 0 (sekitar jumlah frame ~150), dan semua baris match `^{"frame":[0-9]+,"total":[0-9]+}$`.

- [ ] **Step 7: Verifikasi WITHOUT flag — regresi bersih**

```bash
./build/Release/gb_cpp.exe tests/fixtures/short.mp4 --template "C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton/templates/test4.png" --out outputs/t1noflag > /tmp/t1b.out 2>/dev/null
grep -c '^{"frame":' /tmp/t1b.out; echo exit=$?
```
Expected: `0` (grep exit 1 = tidak match).

- [ ] **Step 8: Commit**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Cpp
git add src/system.h src/main.cpp src/system.cpp tests/fixtures/short.mp4
git commit -m "feat: add --progress-json opt-in frame progress flag"
```
(Jika folder Cpp bukan git repo: `git init` dulu — kemungkinan sudah ada.)

---

### Task 2: Scaffold Tauri + Bun + plugins

**Files:**
- Create: seluruh scaffold (`package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`, `src/`, `src-tauri/**`) via create-tauri-app
- Modify: `src-tauri/src/lib.rs` (register plugins)
- Modify: `src-tauri/capabilities/default.json` (permissions)
- Create: `.gitignore`

**Interfaces:**
- Produces: app skeleton yang `bun run build` (vite) dan `cargo check` lulus; plugins `dialog` + `opener` aktif.

- [ ] **Step 1: Scaffold via create-tauri-app (vanilla-ts, bun)**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset
bun x create-tauri-app@latest studio-scaffold --template vanilla-ts --manager bun --yes
```
Expected: folder `studio-scaffold/` berisi `package.json`, `src/`, `src-tauri/`.

- [ ] **Step 2: Pindahkan isi ke folder Studio (docs/ tidak boleh tertimpa)**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset
cp -r studio-scaffold/. Good-Badminton-Studio/
rm -rf studio-scaffold
```

- [ ] **Step 3: .gitignore + git init + commit awal**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Studio
printf 'node_modules\ndist\nsrc-tauri/target\nsrc-tauri/gen\n*.log\n' > .gitignore
git init
git add -A
git commit -m "chore: tauri vanilla-ts scaffold"
```

- [ ] **Step 4: Install deps frontend + plugins JS**

```bash
bun install
bun add @tauri-apps/api @tauri-apps/plugin-dialog @tauri-apps/plugin-opener
```

- [ ] **Step 5: Install plugins Rust**

```bash
cd src-tauri
cargo add tauri-plugin-dialog@2 tauri-plugin-opener@2
```

- [ ] **Step 6: Register plugins** — di `src-tauri/src/lib.rs`, rantai `.plugin(...)` setelah `tauri::Builder::default()`:

```rust
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
```

- [ ] **Step 7: Permissions** — tambah ke array `permissions` di `src-tauri/capabilities/default.json`:

```json
"dialog:default",
"opener:default"
```

- [ ] **Step 8: Gate — build lulus**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Studio
bun run build
cd src-tauri && cargo check
```
Expected: dua-duanya exit 0.

- [ ] **Step 9: Commit**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Studio
git add -A && git commit -m "feat: scaffold tauri app with dialog+opener plugins"
```

---

### Task 3: `progress.rs` + `rally.rs` (parser murni, TDD)

**Files:**
- Create: `src-tauri/src/progress.rs`
- Create: `src-tauri/src/rally.rs`
- Modify: `src-tauri/src/lib.rs` (mod declarations)

**Interfaces:**
- Produces:
  - `progress::parse_line(line: &str) -> Option<Progress>`; `struct Progress { pub frame: u32, pub total: u32 }` (derive Serialize)
  - `rally::count_rally(records: &[RallyInput]) -> u32` dan `rally::count_from_jsonl(text: &str) -> u32`; `struct RallyInput { time_sec: Option<f64>, shuttle_seen: bool }`
  - Dipakai Task 5 (pipeline) dan Task 8 (riwayat).

- [ ] **Step 1: Tulis test yang gagal** — `src-tauri/src/progress.rs`:

```rust
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
pub struct Progress {
    pub frame: u32,
    pub total: u32,
}

pub fn parse_line(line: &str) -> Option<Progress> {
    let v: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
    Some(Progress {
        frame: v.get("frame")?.as_u64()? as u32,
        total: v.get("total")?.as_u64()? as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_progress_line() {
        assert_eq!(
            parse_line("{\"frame\":42,\"total\":5871}"),
            Some(Progress { frame: 42, total: 5871 })
        );
    }

    #[test]
    fn ignores_plain_log_lines() {
        assert_eq!(parse_line("Processing complete:"), None);
        assert_eq!(parse_line("{\"schema_version\":\"1.0\"}"), None);
        assert_eq!(parse_line(""), None);
    }
}
```
`src-tauri/src/rally.rs` (port persis `compare_parity.py:23-47`):

```rust
pub struct RallyInput {
    pub time_sec: Option<f64>,
    pub shuttle_seen: bool,
}

/// Offline-equivalent RallyState rule (start 3 hits/2s, quiet end 4s).
pub fn count_rally(recs: &[RallyInput]) -> u32 {
    const START_HITS: usize = 3;
    const WINDOW: f64 = 2.0;
    const QUIET: f64 = 4.0;
    let mut active = false;
    let mut count = 0u32;
    let mut hits: Vec<f64> = Vec::new();
    let mut last_hit: Option<f64> = None;
    for r in recs {
        let Some(t) = r.time_sec else { continue };
        if active {
            if let Some(l) = last_hit {
                if t - l > QUIET {
                    active = false;
                    hits.clear();
                }
            }
        }
        if r.shuttle_seen {
            last_hit = Some(t);
            hits.retain(|h| t - h <= WINDOW);
            hits.push(t);
            if !active && hits.len() >= START_HITS {
                active = true;
                count += 1;
                hits.clear();
            }
        }
    }
    count
}

/// Extract (time_sec, shuttle seen) from detections.jsonl text.
pub fn count_from_jsonl(text: &str) -> u32 {
    let mut recs = Vec::new();
    for line in text.lines() {
        let Ok(v): Result<serde_json::Value, _> = serde_json::from_str(line) else {
            continue;
        };
        let seen = v
            .get("shuttlecock")
            .and_then(|s| s.get("image"))
            .is_some_and(|i| !i.is_null());
        recs.push((
            v.get("time_sec").and_then(|t| t.as_f64()),
            seen,
        ));
    }
    recs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    count_rally(
        &recs
            .into_iter()
            .map(|(t, s)| RallyInput { time_sec: t, shuttle_seen: s })
            .collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seen(t: f64) -> RallyInput {
        RallyInput { time_sec: Some(t), shuttle_seen: true }
    }

    #[test]
    fn three_hits_in_window_starts_one_rally() {
        // 3 hit dalam 2s -> 1 rally; 4s tanpa hit -> end; hit lagi 3x -> rally ke-2
        let recs = vec![
            seen(0.0), seen(0.5), seen(1.0), seen(5.5), seen(6.0), seen(6.5),
        ];
        assert_eq!(count_rally(&recs), 2);
    }

    #[test]
    fn misses_without_time_do_not_count() {
        let recs = vec![
            RallyInput { time_sec: None, shuttle_seen: true },
            seen(0.0), seen(0.5), seen(1.0),
        ];
        assert_eq!(count_rally(&recs), 1);
    }

    #[test]
    fn jsonl_extraction_end_to_end() {
        let jsonl = concat!(
            "{\"frame\":1,\"time_sec\":0.03,\"shuttlecock\":null}\n",
            "{\"frame\":2,\"time_sec\":0.5,\"shuttlecock\":{\"image\":[1,2]}}\n",
            "{\"frame\":3,\"time_sec\":1.0,\"shuttlecock\":{\"image\":[1,2]}}\n",
            "{\"frame\":4,\"time_sec\":1.5,\"shuttlecock\":{\"image\":[1,2]}}\n",
        );
        assert_eq!(count_from_jsonl(jsonl), 1);
    }
}
```

- [ ] **Step 2: Daftarkan modul** di `src-tauri/src/lib.rs` (atas, setelah `mod` statements scaffold — tambahkan bila belum ada):

```rust
mod progress;
mod rally;
```

- [ ] **Step 3: Run test — pastikan compile & lulus**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Studio/src-tauri
cargo test
```
Expected: semua test PASS. Kalau scaffold belum punya `serde_json`/`serde` di Cargo.toml: `cargo add serde serde_json --features serde/derive`.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/progress.rs src-tauri/src/rally.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: progress line parser + offline rally counter"
```

---

### Task 4: `annotations.rs` + `history.rs` + `config.rs` (TDD)

**Files:**
- Create: `src-tauri/src/annotations.rs`
- Create: `src-tauri/src/history.rs`
- Create: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/lib.rs` (mod declarations)

**Interfaces:**
- Produces:
  - `annotations::write_annotations(path: &Path, corners: &[[i32; 2]; 4], mid_height: i32) -> std::io::Result<()>`
  - `history::Entry { id, video, template, status, started_at, elapsed_sec, output_dir, rally_count }`, `history::load(path) -> Vec<Entry>`, `history::push(path, Entry) -> io::Result<()>`
  - `config::Config { gb_cpp_path: String, data_dir: String, ball_model: String, pose_model: String }`, `config::detect(base: &Path) -> Option<Config>`, `config::load(path)`, `config::save(path, &Config)`
  - Dipakai Task 5 (semua) & Task 7/8 (annotations, history).

- [ ] **Step 1: Test gagal** — `annotations.rs`:

```rust
use std::io;
use std::path::Path;

/// Byte-format yang dibaca pipeline (system.cpp:305-307 setelah rewrite;
/// line 2 roi memang di-skip reader -> selalu [(0, 0), (0, 0)]).
pub fn write_annotations(path: &Path, corners: &[[i32; 2]; 4], mid_height: i32) -> io::Result<()> {
    let c: Vec<String> = corners.iter().map(|[x, y]| format!("({}, {})", x, y)).collect();
    let s = format!(
        "corners=[{}]\nroi_corners=[(0, 0), (0, 0)]\nmid_height={}\n",
        c.join(", "),
        mid_height
    );
    std::fs::write(path, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_three_line_python_literal() {
        let dir = std::env::temp_dir().join(format!("gb_ann_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("court_annotations.txt");
        write_annotations(&p, &[[12, 506], [372, 520], [356, 800], [14, 800]], 625).unwrap();
        let got = std::fs::read_to_string(&p).unwrap();
        assert_eq!(
            got,
            "corners=[(12, 506), (372, 520), (356, 800), (14, 800)]\n\
             roi_corners=[(0, 0), (0, 0)]\nmid_height=625\n"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
```
`history.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entry {
    pub id: String,
    pub video: String,
    pub template: String,
    pub status: String, // "ok" | "failed" | "cancelled"
    pub started_at: String,
    pub elapsed_sec: f64,
    pub output_dir: String,
    pub rally_count: Option<u32>,
}

pub fn load(path: &Path) -> io::Result<Vec<Entry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

pub fn push(path: &Path, entry: Entry) -> io::Result<()> {
    let mut items = load(path)?;
    items.insert(0, entry);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(&items)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Entry {
        Entry {
            id: "1".into(),
            video: "v.mp4".into(),
            template: "t.png".into(),
            status: "ok".into(),
            started_at: "2026-10-01T00:00:00Z".into(),
            elapsed_sec: 12.5,
            output_dir: "out".into(),
            rally_count: Some(17),
        }
    }

    #[test]
    fn push_then_load_roundtrip() {
        let p = std::env::temp_dir().join(format!("gb_hist_{}.json", std::process::id()));
        let _ = std::fs::remove_file(&p);
        push(&p, sample()).unwrap();
        push(&p, sample()).unwrap();
        let items = load(&p).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].rally_count, Some(17));
        std::fs::remove_file(&p).ok();
    }
}
```
`config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub gb_cpp_path: String,
    pub data_dir: String,
    pub ball_model: String,
    pub pose_model: String,
}

/// Cari sibling: <root>/Good-Badminton-Cpp/build/Release/gb_cpp.exe + <root>/Good-Badminton/weights.
pub fn detect(base: &Path) -> Option<Config> {
    for anc in base.ancestors().take(6) {
        let exe = anc.join("Good-Badminton-Cpp/build/Release/gb_cpp.exe");
        let data = anc.join("Good-Badminton");
        if exe.is_file() && data.is_dir() {
            return Some(Config {
                gb_cpp_path: exe.to_string_lossy().into_owned(),
                data_dir: data.to_string_lossy().into_owned(),
                ball_model: data.join("weights/yolo11s-ball.onnx").to_string_lossy().into_owned(),
                pose_model: data.join("weights/yolo11n-pose-dyn.onnx").to_string_lossy().into_owned(),
            });
        }
    }
    None
}

pub fn load(path: &Path) -> io::Result<Option<Config>> {
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&std::fs::read_to_string(path)?)?))
}

pub fn save(path: &Path, cfg: &Config) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(cfg)?)?;
    Ok(())
}

#[allow(dead_code)]
pub fn config_path(app_config_dir: PathBuf) -> PathBuf {
    app_config_dir.join("config.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_finds_siblings_from_nested_base() {
        // <tmp>/root/Good-Badminton-Cpp/build/Release/gb_cpp.exe + <tmp>/root/Good-Badminton/
        let root = std::env::temp_dir().join(format!("gb_det_{}", std::process::id()));
        let exe = root.join("Good-Badminton-Cpp/build/Release");
        let data = root.join("Good-Badminton/weights");
        std::fs::create_dir_all(&exe).unwrap();
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(exe.join("gb_cpp.exe"), b"x").unwrap();
        let base = root.join("some/nested/dir");
        std::fs::create_dir_all(&base).unwrap();
        let cfg = detect(&base).expect("detect");
        assert!(cfg.gb_cpp_path.ends_with("gb_cpp.exe"));
        assert!(cfg.pose_model.ends_with("yolo11n-pose-dyn.onnx"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn load_missing_returns_none() {
        let p = std::env::temp_dir().join(format!("gb_cfg_{}.json", std::process::id()));
        assert!(load(&p).unwrap().is_none());
    }
}
```

- [ ] **Step 2: Daftarkan modul** di `lib.rs`:

```rust
mod annotations;
mod config;
mod history;
```

- [ ] **Step 3: Run test**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Studio/src-tauri
cargo test
```
Expected: PASS semua.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: annotations writer, history store, config detection"
```

---

### Task 5: `pipeline.rs` — spawn/cancel + events + commands

**Files:**
- Create: `src-tauri/src/pipeline.rs`
- Modify: `src-tauri/src/lib.rs` (State + commands + wiring)

**Interfaces:**
- Consumes: `progress::parse_line`, `rally::count_from_jsonl`, `annotations::*`, `history::*`, `config::*`
- Produces (dipakai frontend via `invoke`):
  - `get_defaults() -> { config: Option<Config>, config_path: String, ffmpeg_ok: bool }`
  - `start_run(params: RunParams) -> Result<(), String>` — spawn async; events:
    - `event: "progress"` payload `{ frame: u32, total: u32 }`
    - `event: "log"` payload `{ stream: "out"|"err", line: string }`
    - `event: "run-finished"` payload `{ status: "ok"|"failed"|"cancelled", code: i32, elapsed_sec: f64, output_dir: string, rally_count: u32|null }`
  - `cancel_run() -> ()`
  - `save_annotations(path: string, corners: [[i32;2];4], mid_height: i32) -> Result<(), string>`
  - `save_config(cfg: Config) -> Result<(), string>`
- `RunParams { video: String, template: String, annotations: Option<String>, out_dir: String, audio: bool, language: String }`

- [ ] **Step 1: `pipeline.rs`** (state + arg builder + reader — pure fn `args_for` yang testable):

```rust
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Emitter};

use crate::config::Config;
use crate::{history, progress, rally};

pub struct RunState {
    pub child: Mutex<Option<tokio::process::Child>>,
}

pub struct RunParams {
    pub video: String,
    pub template: String,
    pub annotations: Option<String>,
    pub out_dir: String,
    pub audio: bool,
    pub language: String,
}

pub fn args_for(cfg: &Config, p: &RunParams) -> Vec<String> {
    let mut a = vec![
        p.video.clone(),
        "--template".into(),
        p.template.clone(),
        "--out".into(),
        p.out_dir.clone(),
        "--audio".into(),
        if p.audio { "true" } else { "false" }.into(),
        "--language".into(),
        p.language.clone(),
        "--ball-model".into(),
        cfg.ball_model.clone(),
        "--yolo-pose-model".into(),
        cfg.pose_model.clone(),
        "--progress-json".into(),
    ];
    if let Some(ann) = &p.annotations {
        a.push("--annotations".into());
        a.push(ann.clone());
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        Config {
            gb_cpp_path: "exe".into(),
            data_dir: "d".into(),
            ball_model: "b.onnx".into(),
            pose_model: "p.onnx".into(),
        }
    }

    #[test]
    fn args_include_progress_and_models() {
        let p = RunParams {
            video: "v.mp4".into(),
            template: "t.png".into(),
            annotations: Some("a.txt".into()),
            out_dir: "o".into(),
            audio: false,
            language: "en".into(),
        };
        let a = args_for(&cfg(), &p);
        assert!(a.contains(&"--progress-json".to_string()));
        assert!(a.contains(&"--annotations".to_string()));
        assert!(a.windows(2).any(|w| w[0] == "--audio" && w[1] == "false"));
        assert!(a.contains(&"p.onnx".to_string()));
    }

    #[test]
    fn annotations_omitted_when_none() {
        let p = RunParams {
            video: "v.mp4".into(),
            template: "t.png".into(),
            annotations: None,
            out_dir: "o".into(),
            audio: true,
            language: "zh".into(),
        };
        assert!(!args_for(&cfg(), &p).contains(&"--annotations".to_string()));
    }
}
```

Tambahkan fungsi spawn di file yang sama (dipakai command `start_run`):

```rust
pub async fn spawn_run(
    app: AppHandle,
    state: tauri::State<'_, RunState>,
    cfg: Config,
    params: RunParams,
) -> Result<(), String> {
    let mut child_opt = state.child.lock().unwrap();
    if child_opt.is_some() {
        return Err("run already in progress".into());
    }
    let args = args_for(&cfg, &params);
    let mut cmd = tokio::process::Command::new(&cfg.gb_cpp_path);
    cmd.args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null());
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    let mut child = cmd.spawn().map_err(|e| format!("spawn failed: {e}"))?;
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    *child_opt = Some(child);
    drop(child_opt);

    let started = Instant::now();
    let out_dir = params.out_dir.clone();
    let video = params.video.clone();
    let template = params.template.clone();
    let hist_path = state_hist_path(&app);

    tauri::async_runtime::spawn(async move {
        use tokio::io::{AsyncBufReadExt, BufReader};
        let app2 = app.clone();
        let out_task = tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(p) = progress::parse_line(&line) {
                    let _ = app.emit("progress", p);
                } else {
                    let _ = app.emit("log", serde_json::json!({"stream": "out", "line": line}));
                }
            }
        });
        let err_task = tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = app2.emit("log", serde_json::json!({"stream": "err", "line": line}));
            }
        });
        let _ = out_task.await;
        let _ = err_task.await;

        // child sudah dipindah ke state; tunggu exit + ambil status
        let (code, status_label) = {
            let mut guard = state.child.lock().unwrap();
            if let Some(mut c) = guard.take() {
                match c.try_wait() {
                    Ok(Some(st)) => (st.code().unwrap_or(-1), String::new()),
                    _ => {
                        let _ = c.start_kill();
                        (-1, "cancelled".to_string())
                    }
                }
            } else {
                (-1, "cancelled".to_string())
            }
        };
        let elapsed = started.elapsed().as_secs_f64();
        let (status, rally_count) = if !status_label.is_empty() {
            ("cancelled", None)
        } else if code == 0 {
            let jc = std::path::Path::new(&out_dir).join("detections.jsonl");
            let rc = std::fs::read_to_string(&jc)
                .ok()
                .map(|t| rally::count_from_jsonl(&t));
            ("ok", rc)
        } else {
            ("failed", None)
        };
        let _ = history::push(
            &hist_path,
            history::Entry {
                id: chrono_lite_now(),
                video,
                template,
                status: status.into(),
                started_at: String::new(),
                elapsed_sec: elapsed,
                output_dir: out_dir.clone(),
                rally_count,
            },
        );
        let _ = app.emit(
            "run-finished",
            serde_json::json!({
                "status": status, "code": code, "elapsed_sec": elapsed,
                "output_dir": out_dir, "rally_count": rally_count,
            }),
        );
    });
    Ok(())
}

fn state_hist_path(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_config_dir().unwrap_or_default();
    dir.join("history.json")
}

fn chrono_lite_now() -> String {
    // ISO-ish tanpa dependency chrono: unix epoch seconds
    format!("{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
}
```

> Catatan tukar `state` ke dalam `async_runtime::spawn`: karena `tauri::State` tidak 'static, refactor saat implementasi — pendapat yang sudah teruji: simpan `RunState` sebagai `tauri::State` di command, lalu klon `Arc<Mutex<Option<Child>>>` ke dalam task (`RunState { child: Arc<Mutex<Option<Child>>> }`). Signature command tetap `start_run(app, state, ...)`; dalam task pakai Arc clone. Pastikan `cargo test` lulus dan `cargo check` tanpa warning lifetime.

- [ ] **Step 2: Commands di `lib.rs`** (daftarkan di `generate_handler![]`):

```rust
#[tauri::command]
async fn start_run(
    app: tauri::AppHandle,
    state: tauri::State<'_, pipeline::RunState>,
    params: pipeline::RunParams,
) -> Result<(), String> {
    let cfg = config::load(&app.path().app_config_dir().unwrap().join("config.json"))
        .map_err(|e| e.to_string())?
        .ok_or("config belum di-set (get_defaults dulu)")?;
    pipeline::spawn_run(app, state, cfg, params).await
}

#[tauri::command]
async fn cancel_run(state: tauri::State<'_, pipeline::RunState>) {
    if let Some(mut c) = state.child.lock().unwrap().take() {
        let _ = c.start_kill();
    }
}

#[tauri::command]
fn get_defaults(app: tauri::AppHandle) -> (Option<config::Config>, String, bool) {
    let dir = app.path().app_config_dir().unwrap_or_default();
    let cfg = config::load(&dir.join("config.json")).ok().flatten();
    let base = std::env::current_dir().unwrap_or_default();
    let detected = cfg.or_else(|| config::detect(&base));
    (detected, dir.to_string_lossy().into_owned(), which_ffmpeg())
}

#[tauri::command]
fn save_config(app: tauri::AppHandle, cfg: config::Config) -> Result<(), String> {
    let dir = app.path().app_config_dir().unwrap();
    config::save(&dir.join("config.json"), &cfg).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_annotations(path: String, corners: [[i32; 2]; 4], mid_height: i32) -> Result<(), String> {
    annotations::write_annotations(std::path::Path::new(&path), &corners, mid_height)
        .map_err(|e| e.to_string())
}

fn which_ffmpeg() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
```
Register: `.manage(pipeline::RunState { child: Default::default() })` + `tauri::generate_handler![get_defaults, start_run, cancel_run, save_config, save_annotations]`.
`RunParams` butuh `#[derive(serde::Deserialize)]`. `RunState.child: Arc<Mutex<Option<Child>>>` (std::sync::Mutex).

- [ ] **Step 3: `cargo test` + `cargo check`**

```bash
cd src-tauri && cargo test && cargo check
```
Expected: PASS + tanpa error.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: pipeline spawn/cancel with progress events + commands"
```

---

### Task 6: Frontend — view Setup (run + progress + log)

**Files:**
- Modify: `index.html`
- Create: `src/api.ts`, `src/setup.ts`, `src/styles.css`
- Modify: `src/main.ts`

**Interfaces:**
- Consumes: commands Task 5, events `progress`/`log`/`run-finished`.
- Produces: `api.ts` exports `getDefaults()`, `startRun(params)`, `cancelRun()`, `saveAnnotations(...)`, `saveConfig(cfg)`.

- [ ] **Step 1: `src/api.ts`**

```ts
import { invoke } from "@tauri-apps/api/core";

export interface Config {
  gb_cpp_path: string;
  data_dir: string;
  ball_model: string;
  pose_model: string;
}
export interface RunParams {
  video: string;
  template: string;
  annotations: string | null;
  out_dir: string;
  audio: boolean;
  language: "en" | "zh";
}

export const getDefaults = () =>
  invoke<[Config | null, string, boolean]>("get_defaults");
export const startRun = (params: RunParams) => invoke<void>("start_run", { params });
export const cancelRun = () => invoke<void>("cancel_run");
export const saveConfig = (cfg: Config) => invoke<void>("save_config", { cfg });
export const saveAnnotations = (path: string, corners: number[][], mid: number) =>
  invoke<void>("save_annotations", { path, corners, midHeight: mid });
```

- [ ] **Step 2: `index.html`** — nav 3 tab + section per view:

```html
<!doctype html>
<html lang="en">
  <head><meta charset="UTF-8" /><meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Good-Badminton Studio</title></head>
  <body>
    <nav>
      <button data-view="setup" class="active">Run</button>
      <button data-view="corners">Court</button>
      <button data-view="history">History</button>
    </nav>
    <main>
      <section id="view-setup"></section>
      <section id="view-corners" hidden></section>
      <section id="view-history" hidden></section>
    </main>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 3: `src/setup.ts`** — form (video/template/out picker via plugin-dialog, audio checkbox, language select, tombol Run/Cancel, progress bar + ETA, log pane). Inti logika:

```ts
import { open, save } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { openPath } from "@tauri-apps/plugin-opener";
import { startRun, cancelRun, type RunParams } from "./api";

export function renderSetup(root: HTMLElement) {
  root.innerHTML = `
    <div class="form">
      <label>Video <input id="v-video" readonly /><button id="p-video">…</button></label>
      <label>Template <input id="v-template" readonly /><button id="p-template">…</button></label>
      <label>Output dir <input id="v-out" readonly /><button id="p-out">…</button></label>
      <label>Annotations <input id="v-ann" readonly /><button id="p-ann">…</button> (opsional)</label>
      <label>Audio <input type="checkbox" id="v-audio" checked /></label>
      <label>Language <select id="v-lang"><option value="zh">zh</option><option value="en">en</option></select></label>
      <button id="b-run">Run</button><button id="b-cancel" disabled>Cancel</button>
      <button id="b-open" hidden>Buka folder output</button>
    </div>
    <progress id="prog" value="0" max="100"></progress><span id="eta"></span>
    <pre id="log"></pre>`;

  const pick = async (opts: any) => (await open({ multiple: false, ...opts })) as string | null;
  root.querySelector("#p-video")!.addEventListener("click", async () => {
    const f = await pick({ filters: [{ name: "Video", extensions: ["mp4", "avi", "mov"] }] });
    if (f) (root.querySelector("#v-video") as HTMLInputElement).value = f;
  });
  root.querySelector("#p-template")!.addEventListener("click", async () => {
    const f = await pick({ filters: [{ name: "Image", extensions: ["png", "jpg"] }] });
    if (f) (root.querySelector("#v-template") as HTMLInputElement).value = f;
  });
  root.querySelector("#p-out")!.addEventListener("click", async () => {
    const f = await pick({ directory: true });
    if (f) (root.querySelector("#v-out") as HTMLInputElement).value = f;
  });
  root.querySelector("#p-ann")!.addEventListener("click", async () => {
    const f = await pick({ filters: [{ name: "Anotasi", extensions: ["txt"] }] });
    if (f) (root.querySelector("#v-ann") as HTMLInputElement).value = f;
  });

  let t0 = 0;
  listen<{ frame: number; total: number }>("progress", (e) => {
    if (!t0) t0 = Date.now();
    const { frame, total } = e.payload;
    const bar = root.querySelector("#prog") as HTMLProgressElement;
    bar.value = (frame / total) * 100;
    const rate = frame / ((Date.now() - t0) / 1000);
    const eta = rate > 0 ? Math.round((total - frame) / rate) : 0;
    (root.querySelector("#eta") as HTMLElement).textContent = `${frame}/${total} · ETA ${eta}s`;
  });
  listen<{ line: string }>("log", (e) => {
    const log = root.querySelector("#log")!;
    log.textContent += e.payload.line + "\n";
    log.scrollTop = log.scrollHeight;
  });
  listen<{ status: string; output_dir: string }>("run-finished", (e) => {
    (root.querySelector("#b-run") as HTMLButtonElement).disabled = false;
    (root.querySelector("#b-cancel") as HTMLButtonElement).disabled = true;
    const b = root.querySelector("#b-open") as HTMLButtonElement;
    b.hidden = e.payload.status !== "ok";
    b.onclick = () => openPath(e.payload.output_dir);
    (root.querySelector("#eta") as HTMLElement).textContent = `selesai: ${e.payload.status}`;
  });

  root.querySelector("#b-run")!.addEventListener("click", async () => {
    t0 = 0;
    const params: RunParams = {
      video: (root.querySelector("#v-video") as HTMLInputElement).value,
      template: (root.querySelector("#v-template") as HTMLInputElement).value,
      annotations: (root.querySelector("#v-ann") as HTMLInputElement).value || null,
      out_dir: (root.querySelector("#v-out") as HTMLInputElement).value,
      audio: (root.querySelector("#v-audio") as HTMLInputElement).checked,
      language: (root.querySelector("#v-lang") as HTMLSelectElement).value as "en" | "zh",
    };
    if (!params.video || !params.template || !params.out_dir) return alert("lengkapi video/template/output");
    (root.querySelector("#b-run") as HTMLButtonElement).disabled = true;
    (root.querySelector("#b-cancel") as HTMLButtonElement).disabled = false;
    (root.querySelector("#log") as HTMLElement).textContent = "";
    try {
      await startRun(params);
    } catch (e) {
      alert(String(e));
      (root.querySelector("#b-run") as HTMLButtonElement).disabled = false;
    }
  });
  root.querySelector("#b-cancel")!.addEventListener("click", () => cancelRun());
}
```

- [ ] **Step 4: `src/main.ts`** — nav switching + render setup:

```ts
import "./styles.css";
import { renderSetup } from "./setup";
// renderCorners/renderHistory menyusul di Task 7-8 (import bertahap)

const views: Record<string, (el: HTMLElement) => void> = {
  setup: renderSetup,
  // corners, history ditambahkan di task berikutnya
};
function show(name: string) {
  document.querySelectorAll("main section").forEach((s) => (s.hidden = s.id !== `view-${name}`));
  document.querySelectorAll("nav button").forEach((b) =>
    (b as HTMLButtonElement).classList.toggle("active", (b as HTMLButtonElement).dataset.view === name));
  const el = document.getElementById(`view-${name}`);
  if (el && views[name]) views[name](el);
}
document.querySelectorAll("nav button").forEach((b) =>
  (b as HTMLButtonElement).addEventListener("click", () => show((b as HTMLButtonElement).dataset.view!)));
show("setup");
```

- [ ] **Step 5: `styles.css`** minimal (dark, form grid, log monospace) + gate build:

```bash
bun run build
```
Expected: exit 0 (tsc + vite).

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat: setup view with run/cancel/progress/log"
```

---

### Task 7: Frontend — view Court (picker 4 titik)

**Files:**
- Create: `src/corners.ts`
- Modify: `src/main.ts` (register view)

**Interfaces:**
- Consumes: `saveAnnotations`, plugin-dialog `open` (buka PNG), event tidak dipakai.
- Produces: `renderCorners(root: HTMLElement)`; klik canvas simpan 4 titik (koordinat px image), input `mid_height` (default 625), tombol Save → `save_annotations`.

- [ ] **Step 1: `src/corners.ts`**

```ts
import { open, save } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import { saveAnnotations } from "./api";

export function renderCorners(root: HTMLElement) {
  root.innerHTML = `
    <button id="c-open">Buka template…</button>
    <canvas id="c-canvas" width="478" height="850"></canvas>
    <label>mid_height <input id="c-mid" type="number" value="625" /></label>
    <div id="c-pts">Titik: 0/4</div>
    <button id="c-save" disabled>Simpan annotations</button>`;
  const canvas = root.querySelector("#c-canvas") as HTMLCanvasElement;
  const ctx = canvas.getContext("2d")!;
  const pts: [number, number][] = [];
  let img: HTMLImageElement | null = null;
  let imgPath = "";

  const redraw = () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    if (img) ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
    ctx.strokeStyle = "#0f0";
    ctx.lineWidth = 2;
    pts.forEach(([x, y]) => {
      ctx.beginPath();
      ctx.arc(x, y, 5, 0, Math.PI * 2);
      ctx.stroke();
    });
    if (pts.length === 4) {
      ctx.beginPath();
      pts.forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)));
      ctx.closePath();
      ctx.stroke();
    }
    (root.querySelector("#c-pts") as HTMLElement).textContent = `Titik: ${pts.length}/4`;
    (root.querySelector("#c-save") as HTMLButtonElement).disabled = pts.length !== 4;
  };

  root.querySelector("#c-open")!.addEventListener("click", async () => {
    const f = (await open({ multiple: false, filters: [{ name: "PNG", extensions: ["png"] }] })) as string | null;
    if (!f) return;
    imgPath = f;
    img = new Image();
    img.onload = () => {
      canvas.width = img!.width;
      canvas.height = img!.height;
      redraw();
    };
    img.src = convertFileSrc(f);
  });

  canvas.addEventListener("click", (ev) => {
    if (pts.length >= 4) return;
    const r = canvas.getBoundingClientRect();
    const x = Math.round((ev.clientX - r.left) * (canvas.width / r.width));
    const y = Math.round((ev.clientY - r.top) * (canvas.height / r.height));
    pts.push([x, y]);
    redraw();
  });

  root.querySelector("#c-save")!.addEventListener("click", async () => {
    const path = (await save({ defaultPath: "court_annotations.txt", filters: [{ name: "txt", extensions: ["txt"] }] })) as string | null;
    if (!path) return;
    const mid = Number((root.querySelector("#c-mid") as HTMLInputElement).value);
    try {
      await saveAnnotations(path, pts, mid);
      alert("Tersimpan: " + path);
    } catch (e) {
      alert(String(e));
    }
  });
}
```

- [ ] **Step 2: Register di `main.ts`**

```ts
import { renderCorners } from "./corners";
const views: Record<string, (el: HTMLElement) => void> = {
  setup: renderSetup,
  corners: renderCorners,
};
```

- [ ] **Step 3: Gate**

```bash
bun run build
```
Expected: exit 0.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: court corner picker canvas"
```

---

### Task 8: Frontend — view History + preview

**Files:**
- Create: `src/history.ts`
- Modify: `src/main.ts` (register)

**Interfaces:**
- Consumes: `history.json` via command baru `list_history` (Tambahkan di Task 5 lib.rs kalau belum — seluruh point ini: command `list_history() -> Vec<history::Entry>` yang memanggil `history::load`), `convertFileSrc` untuk preview.
- Produces: `renderHistory(root)`; daftar entri + klik → tampilkan metadata + `<video src=convertFileSrc(output_dir + "/detect_<name>.mp4")>`.

- [ ] **Step 1: Command `list_history` di `lib.rs`** (kalau belum ada dari Task 5):

```rust
#[tauri::command]
fn list_history(app: tauri::AppHandle) -> Vec<history::Entry> {
    let dir = app.path().app_config_dir().unwrap_or_default();
    history::load(&dir.join("history.json")).unwrap_or_default()
}
```
+ tambah ke `generate_handler![]`.

- [ ] **Step 2: `src/history.ts`**

```ts
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";

interface Entry {
  id: string;
  video: string;
  template: string;
  status: string;
  elapsed_sec: number;
  output_dir: string;
  rally_count: number | null;
}

export async function renderHistory(root: HTMLElement) {
  const items = await invoke<Entry[]>("list_history");
  root.innerHTML = `<ul id="h-list">${items
    .map(
      (e) => `<li data-out="${e.output_dir}" data-video="${e.video}">
        <b>${e.status}</b> ${e.video.split(/[\\/]/).pop()} · ${e.elapsed_sec.toFixed(0)}s
        · rally ${e.rally_count ?? "-"}
      </li>`,
    )
    .join("")}</ul><div id="h-detail"></div>`;
  root.querySelectorAll("#h-list li").forEach((li) => {
    li.addEventListener("click", async () => {
      const out = (li as HTMLElement).dataset.out!;
      const name = (li as HTMLElement).dataset.video!.split(/[\\/]/).pop()!.replace(/\.[^.]+$/, "");
      const detail = root.querySelector("#h-detail") as HTMLElement;
      if (!out) return;
      const src = convertFileSrc(`${out}/detect_${name}.mp4`);
      detail.innerHTML = `<video controls src="${src}" style="max-height:70vh"></video>`;
    });
  });
}
```

- [ ] **Step 3: Register + gate + commit**

```ts
import { renderHistory } from "./history";
// views: { setup, corners, history: (el) => { void renderHistory(el); } }
```
```bash
bun run build && git add -A && git commit -m "feat: history list with output preview"
```

---

### Task 9: E2E smoke + installer

**Files:**
- Modify: `src-tauri/src/config.rs` only jika auto-detect perlu fallback (kemungkinan tidak)

**Interfaces:**
- Consumes: semuanya.
- Produces: bukti run end-to-end + `bun run tauri build` → installer NSIS.

- [ ] **Step 1: Jalankan dev app**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Studio
bun run tauri dev
```
Expected: window terbuka, tab Run menampilkan path default (gb_cpp terdeteksi), ffmpeg_ok sesuai instalasi.

- [ ] **Step 2: E2E manual di window** — isi video = `Good-Badminton/videos/test4.mp4`, template = `Good-Badminton/templates/test4.png`, out = folder kosong, Run.
Expected: progress bar bergerak, ETA turun, log terisi, setelah selesai tombol "Buka folder output" muncul; `detections.jsonl` ≥5871 baris; tab History menampilkan entri + preview playable.

- [ ] **Step 3: Batal** — Run lagi, tekan Cancel di tengah.
Expected: `run-finished` status `cancelled`, proses `gb_cpp.exe` mati (Task Manager).

- [ ] **Step 4: Regression parity cepat** (fixture 5 detik, CLI polos tanpa flag — memastikan Task 1 tidak mengubah output):

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Cpp
./build/Release/gb_cpp.exe tests/fixtures/short.mp4 --template "C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton/templates/test4.png" --out outputs/t9reg > /tmp/t9.out 2>/dev/null
grep -c '^{"frame":' /tmp/t9.out || true
```
Expected: `0`.

- [ ] **Step 5: Build installer**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Studio
bun run tauri build
```
Expected: exit 0, artefak `.exe` installer di `src-tauri/target/release/bundle/nsis/`.

- [ ] **Step 6: Final commit**

```bash
git add -A && git commit -m "chore: e2e verified, nsis installer builds"
```

---

## Self-Review (plan author)

- **Spec coverage:** dasar (T5+T6), picker sudut (T4+T7), preview (T8), riwayat (T4+T8), `--progress-json` (T1), config/ffmpeg detect (T4/T5), Bun (Global Constraints + T2), installer (T9). ✔
- **Placeholder scan:** signature `RunState.child` = `Arc<Mutex<Option<Child>>>` konsisten; `saveAnnotations(path, corners, mid)` TS ↔ Rust `save_annotations(path, corners, mid_height)` — invoke key `midHeight` → command arg `mid_height` (Tauri case conversion otomatis snake↔camel). `list_history` didefinisikan eksplisit di T8 Step 1. ✔
- **Type consistency:** `progress::Progress` derive Serialize → payload event langsung; `history::Entry` Serialize+Deserialize untuk invoke balik. ✔
