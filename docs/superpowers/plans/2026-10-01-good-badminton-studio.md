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
- Model/paths default: sibling `Good-Badminton-Cpp/build/Release/gb_cpp.exe`, data dir `Good-Badminton/` (weights `yolo11n-pose-dyn.onnx` + `yolo11s-ball.onnx`), semuanya di-overridable via config. Deteksi wajib menoleransi lintas-OS: `gb_cpp.exe` (Windows) maupun `gb_cpp` (macOS/Linux), path `build/Release/` maupun `build/`.
- Rally count di riwayat = port aturan `compare_parity.py:rally_count` (START_HITS=3, WINDOW=2.0, QUIET=4.0) di atas jsonl.
- Toggle display pipeline (`--skeletons`, `--player-trajectories`, `--court-trajectory`, `--shuttlecock-trajectory`, `--player-stats`, `--pose-roi`) wajib default **true** (identik Python main.py — parity tidak boleh berubah). UI mengekspos semuanya; `config.json` menyimpan preferensi user (`defaults`), di-prefill ke form.
- Git: init di folder Studio, commit tiap task.
- UI (Tasks 6-8) wajib terlihat **profesional & calm** (user: "desain benar2 terlihat profesional"). Design tokens terpusat di `styles.css`: tema gelap teal ala broadcast-desk (`--bg` ink, `--accent` teal, kartu `--surface` + border halus), font **IBM Plex Sans/Mono** via `@fontsource` (bundled offline — jangan pakai font generic), hierarki tombol primary/ghost/danger, form per kartu berkelompok, pane console monospace. Semua ID elemen yang dirujuk logic (semua selector `#…` di brief) **tidak boleh berubah**. Selektor khusus Task 7 (court) & Task 8 (history) ditulis di `styles.css` Task 6 dengan komentar penanda — jangan ubah markup brief di task tersebut tanpa juga menyertakan class-nya.

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
    /// Parameter run default untuk prefill UI (lama: field boleh absen → Default).
    #[serde(default)]
    pub defaults: RunDefaults,
}

/// Default identik main.py: audio on, enam toggle display true, language zh.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunDefaults {
    pub audio: bool,
    pub language: String,
    pub show_skeletons: bool,
    pub show_player_trajectories: bool,
    pub show_court_trajectory: bool,
    pub show_shuttlecock_trajectory: bool,
    pub show_player_stats: bool,
    pub show_pose_roi: bool,
}

impl Default for RunDefaults {
    fn default() -> Self {
        RunDefaults {
            audio: true,
            language: "zh".into(),
            show_skeletons: true,
            show_player_trajectories: true,
            show_court_trajectory: true,
            show_shuttlecock_trajectory: true,
            show_player_stats: true,
            show_pose_roi: true,
        }
    }
}

/// Cari sibling: binary pipeline (gb_cpp.exe di Windows, gb_cpp di
/// macOS/Linux; build/Release utk VS, build/ utk Make/Ninja) + <root>/Good-Badminton/weights.
pub fn detect(base: &Path) -> Option<Config> {
    const EXES: &[&str] = &[
        "Good-Badminton-Cpp/build/Release/gb_cpp.exe",
        "Good-Badminton-Cpp/build/Release/gb_cpp",
        "Good-Badminton-Cpp/build/gb_cpp",
        "Good-Badminton-Cpp/build/gb_cpp.exe",
    ];
    for anc in base.ancestors().take(6) {
        let data = anc.join("Good-Badminton");
        if !data.is_dir() {
            continue;
        }
        let Some(exe) = EXES.iter().map(|e| anc.join(e)).find(|p| p.is_file()) else {
            continue;
        };
        return Some(Config {
            gb_cpp_path: exe.to_string_lossy().into_owned(),
            data_dir: data.to_string_lossy().into_owned(),
            ball_model: data.join("weights/yolo11s-ball.onnx").to_string_lossy().into_owned(),
            pose_model: data.join("weights/yolo11n-pose-dyn.onnx").to_string_lossy().into_owned(),
            defaults: RunDefaults::default(),
        });
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

    #[test]
    fn defaults_roundtrip() {
        let p = std::env::temp_dir().join(format!("gb_cfg_rt_{}.json", std::process::id()));
        let cfg = Config {
            gb_cpp_path: "e".into(),
            data_dir: "d".into(),
            ball_model: "b".into(),
            pose_model: "p".into(),
            defaults: RunDefaults::default(),
        };
        save(&p, &cfg).unwrap();
        let back = load(&p).unwrap().unwrap();
        assert!(back.defaults.show_skeletons && back.defaults.audio);
        assert_eq!(back.defaults.language, "zh");
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn legacy_config_without_defaults_field_loads() {
        let p = std::env::temp_dir().join(format!("gb_cfg_legacy_{}.json", std::process::id()));
        std::fs::write(
            &p,
            "{\"gb_cpp_path\":\"e\",\"data_dir\":\"d\",\"ball_model\":\"b\",\"pose_model\":\"p\"}",
        )
        .unwrap();
        let back = load(&p).unwrap().unwrap();
        assert!(back.defaults.show_player_stats);
        std::fs::remove_file(&p).ok();
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

#[derive(serde::Deserialize, Clone, Debug)]
pub struct RunParams {
    pub video: String,
    pub template: String,
    pub annotations: Option<String>,
    pub out_dir: String,
    pub audio: bool,
    pub language: String,
    pub show_skeletons: bool,
    pub show_player_trajectories: bool,
    pub show_court_trajectory: bool,
    pub show_shuttlecock_trajectory: bool,
    pub show_player_stats: bool,
    pub show_pose_roi: bool,
}

pub fn args_for(cfg: &Config, p: &RunParams) -> Vec<String> {
    let b = |v: bool| if v { "true" } else { "false" }.to_string();
    let mut a = vec![
        p.video.clone(),
        "--template".into(),
        p.template.clone(),
        "--out".into(),
        p.out_dir.clone(),
        "--audio".into(),
        b(p.audio),
        "--language".into(),
        p.language.clone(),
        "--ball-model".into(),
        cfg.ball_model.clone(),
        "--yolo-pose-model".into(),
        cfg.pose_model.clone(),
        "--progress-json".into(),
        // display toggles — selalu eksplisit; default C++ = true = Python parity
        "--skeletons".into(),
        b(p.show_skeletons),
        "--player-trajectories".into(),
        b(p.show_player_trajectories),
        "--court-trajectory".into(),
        b(p.show_court_trajectory),
        "--shuttlecock-trajectory".into(),
        b(p.show_shuttlecock_trajectory),
        "--player-stats".into(),
        b(p.show_player_stats),
        "--pose-roi".into(),
        b(p.show_pose_roi),
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
            defaults: crate::config::RunDefaults::default(),
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
            show_skeletons: false,
            show_player_trajectories: true,
            show_court_trajectory: true,
            show_shuttlecock_trajectory: true,
            show_player_stats: true,
            show_pose_roi: false,
        };
        let a = args_for(&cfg(), &p);
        assert!(a.contains(&"--progress-json".to_string()));
        assert!(a.contains(&"--annotations".to_string()));
        assert!(a.windows(2).any(|w| w[0] == "--audio" && w[1] == "false"));
        assert!(a.windows(2).any(|w| w[0] == "--skeletons" && w[1] == "false"));
        assert!(a.windows(2).any(|w| w[0] == "--pose-roi" && w[1] == "false"));
        assert!(a.windows(2).any(|w| w[0] == "--court-trajectory" && w[1] == "true"));
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
            show_skeletons: true,
            show_player_trajectories: true,
            show_court_trajectory: true,
            show_shuttlecock_trajectory: true,
            show_player_stats: true,
            show_pose_roi: true,
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
                started_at: chrono_lite_now(),
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
    // unix epoch detik (bukan ISO 8601 — konsumen UI format sendiri)
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
    let path = app.path().app_config_dir().unwrap().join("config.json");
    let cfg = config::load(&path)
        .map_err(|e| e.to_string())?
        // first launch: config.json belum ada → fallback ke auto-detect sibling
        .or_else(|| config::detect(&std::env::current_dir().unwrap_or_default()))
        .ok_or("config tidak ada dan sibling Good-Badminton-Cpp tidak terdeteksi")?;
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
- Modify: `index.html`, `package.json` (via `bun add` fontsource), `src-tauri/capabilities/default.json`
- Create: `src/api.ts`, `src/setup.ts`, `src/styles.css`
- Modify: `src/main.ts`

**Interfaces:**
- Consumes: commands Task 5, events `progress`/`log`/`run-finished`.
- Produces: `api.ts` exports `getDefaults()`, `startRun(params)`, `cancelRun()`, `saveAnnotations(...)`, `saveConfig(cfg)`.

- [ ] **Step 1: `src/api.ts`**

```ts
import { invoke } from "@tauri-apps/api/core";

export interface RunDefaults {
  audio: boolean;
  language: string;
  show_skeletons: boolean;
  show_player_trajectories: boolean;
  show_court_trajectory: boolean;
  show_shuttlecock_trajectory: boolean;
  show_player_stats: boolean;
  show_pose_roi: boolean;
}
export interface Config {
  gb_cpp_path: string;
  data_dir: string;
  ball_model: string;
  pose_model: string;
  defaults: RunDefaults;
}
export interface RunParams {
  video: string;
  template: string;
  annotations: string | null;
  out_dir: string;
  audio: boolean;
  language: "en" | "zh";
  show_skeletons: boolean;
  show_player_trajectories: boolean;
  show_court_trajectory: boolean;
  show_shuttlecock_trajectory: boolean;
  show_player_stats: boolean;
  show_pose_roi: boolean;
}

export const getDefaults = () =>
  invoke<[Config | null, string, boolean]>("get_defaults");
export const startRun = (params: RunParams) => invoke<void>("start_run", { params });
export const cancelRun = () => invoke<void>("cancel_run");
export const saveConfig = (cfg: Config) => invoke<void>("save_config", { cfg });
export const saveAnnotations = (path: string, corners: number[][], mid: number) =>
  invoke<void>("save_annotations", { path, corners, midHeight: mid });
```

- [ ] **Step 2: `index.html`** — topbar (brand + 3 tab) + section per view:

```html
<!doctype html>
<html lang="en">
  <head><meta charset="UTF-8" /><meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Good-Badminton Studio</title></head>
  <body>
    <nav class="topbar">
      <div class="brand">
        <svg class="brand-mark" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" aria-hidden="true">
          <rect x="3.5" y="4.5" width="17" height="15" rx="1.5"/>
          <path d="M3.5 12h17M8.5 4.5v15M15.5 4.5v15"/>
        </svg>
        <span class="brand-name">GOOD-BADMINTON <em>STUDIO</em></span>
      </div>
      <div class="tabs">
        <button data-view="setup" class="tab active">Run</button>
        <button data-view="corners" class="tab">Court</button>
        <button data-view="history" class="tab">History</button>
      </div>
    </nav>
    <main>
      <section id="view-setup" class="view"></section>
      <section id="view-corners" class="view" hidden></section>
      <section id="view-history" class="view" hidden></section>
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
import { startRun, cancelRun, getDefaults, saveConfig, type RunParams } from "./api";

let t0 = 0;
let listenersBound = false;

export function renderSetup(root: HTMLElement) {
  root.innerHTML = `
    <div class="pane-grid">
      <div class="stack">
        <section class="card">
          <header class="card-head"><h2>Source</h2></header>
          <div class="field">
            <span class="field-label">Video</span>
            <div class="picker">
              <input id="v-video" readonly placeholder="Pilih video pertandingan…" />
              <button id="p-video" class="btn btn-ghost" type="button">Browse</button>
            </div>
          </div>
          <div class="field">
            <span class="field-label">Template lapangan</span>
            <div class="picker">
              <input id="v-template" readonly placeholder="PNG template sudut lapangan…" />
              <button id="p-template" class="btn btn-ghost" type="button">Browse</button>
            </div>
          </div>
        </section>
        <section class="card">
          <header class="card-head"><h2>Output</h2></header>
          <div class="field">
            <span class="field-label">Folder output</span>
            <div class="picker">
              <input id="v-out" readonly placeholder="Folder tujuan hasil analisis…" />
              <button id="p-out" class="btn btn-ghost" type="button">Browse</button>
            </div>
          </div>
          <div class="field">
            <span class="field-label">Anotasi lapangan <em class="opt">opsional</em></span>
            <div class="picker">
              <input id="v-ann" readonly placeholder="court_annotations.txt — kosongkan untuk auto-detect" />
              <button id="p-ann" class="btn btn-ghost" type="button">Browse</button>
            </div>
          </div>
        </section>
        <section class="card">
          <header class="card-head"><h2>Parameters</h2></header>
          <div class="row">
            <label class="switch-row">
              <input type="checkbox" id="v-audio" class="switch" checked />
              <span>Simpan audio</span>
            </label>
            <div class="field field-half">
              <span class="field-label">Bahasa</span>
              <select id="v-lang"><option value="zh">zh</option><option value="en">en</option></select>
            </div>
          </div>
          <fieldset class="params">
            <legend>Display overlay</legend>
            <div class="tog-grid">
              <label class="tog"><input type="checkbox" id="p-skel" checked /><span>Skeleton</span></label>
              <label class="tog"><input type="checkbox" id="p-trail" checked /><span>Player trail</span></label>
              <label class="tog"><input type="checkbox" id="p-court" checked /><span>Court trail</span></label>
              <label class="tog"><input type="checkbox" id="p-shuttle" checked /><span>Shuttle trail</span></label>
              <label class="tog"><input type="checkbox" id="p-stats" checked /><span>Stats panel</span></label>
              <label class="tog"><input type="checkbox" id="p-roi" checked /><span>Pose ROI</span></label>
            </div>
          </fieldset>
          <div class="actions">
            <button id="b-run" class="btn btn-primary" type="button">Run analysis</button>
            <button id="b-cancel" class="btn btn-danger" type="button" disabled>Cancel</button>
            <button id="b-open" class="btn btn-ghost" type="button" hidden>Buka folder output</button>
          </div>
        </section>
      </div>
      <div class="stack">
        <section class="card progress-card">
          <header class="card-head"><h2>Progress</h2><span id="run-status" class="chip" hidden></span></header>
          <progress id="prog" value="0" max="100"></progress>
          <span id="eta" class="mono">—</span>
        </section>
        <section class="card console">
          <header class="card-head"><h2>Console</h2></header>
          <pre id="log" aria-live="polite"></pre>
        </section>
      </div>
    </div>`;

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

  // Prefill parameter dari config.json (preferensi user terakhir)
  void getDefaults().then(([cfg]) => {
    const d = cfg?.defaults;
    if (!d) return;
    (root.querySelector("#v-audio") as HTMLInputElement).checked = d.audio;
    (root.querySelector("#v-lang") as HTMLSelectElement).value = d.language;
    const pairs: [string, boolean][] = [
      ["#p-skel", d.show_skeletons],
      ["#p-trail", d.show_player_trajectories],
      ["#p-court", d.show_court_trajectory],
      ["#p-shuttle", d.show_shuttlecock_trajectory],
      ["#p-stats", d.show_player_stats],
      ["#p-roi", d.show_pose_roi],
    ];
    for (const [id, v] of pairs) (root.querySelector(id) as HTMLInputElement).checked = v;
  });

  // listen() global — cukup sekali; handler selalu query DOM via root saat event tiba
  if (!listenersBound) {
    listenersBound = true;
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
      // status final di chip durable — #eta hanya untuk frame/ETA
      const chip = root.querySelector("#run-status") as HTMLElement;
      chip.textContent = e.payload.status;
      chip.className = `chip s-${e.payload.status}`;
      chip.hidden = false;
    });
  }

  root.querySelector("#b-run")!.addEventListener("click", async () => {
    t0 = 0;
    const chk = (id: string) => (root.querySelector(id) as HTMLInputElement).checked;
    const params: RunParams = {
      video: (root.querySelector("#v-video") as HTMLInputElement).value,
      template: (root.querySelector("#v-template") as HTMLInputElement).value,
      annotations: (root.querySelector("#v-ann") as HTMLInputElement).value || null,
      out_dir: (root.querySelector("#v-out") as HTMLInputElement).value,
      audio: (root.querySelector("#v-audio") as HTMLInputElement).checked,
      language: (root.querySelector("#v-lang") as HTMLSelectElement).value as "en" | "zh",
      show_skeletons: chk("#p-skel"),
      show_player_trajectories: chk("#p-trail"),
      show_court_trajectory: chk("#p-court"),
      show_shuttlecock_trajectory: chk("#p-shuttle"),
      show_player_stats: chk("#p-stats"),
      show_pose_roi: chk("#p-roi"),
    };
    if (!params.video || !params.template || !params.out_dir) return alert("lengkapi video/template/output");
    (root.querySelector("#b-run") as HTMLButtonElement).disabled = true;
    (root.querySelector("#b-cancel") as HTMLButtonElement).disabled = false;
    (root.querySelector("#log") as HTMLElement).textContent = "";
    (root.querySelector("#run-status") as HTMLElement).hidden = true;
    try {
      // persist preferensi parameter (gagal simpan tidak menghalangi run)
      const [cfg] = await getDefaults();
      if (cfg) {
        cfg.defaults = {
          audio: params.audio,
          language: params.language,
          show_skeletons: params.show_skeletons,
          show_player_trajectories: params.show_player_trajectories,
          show_court_trajectory: params.show_court_trajectory,
          show_shuttlecock_trajectory: params.show_shuttlecock_trajectory,
          show_player_stats: params.show_player_stats,
          show_pose_roi: params.show_pose_roi,
        };
        await saveConfig(cfg).catch(() => {});
      }
      await startRun(params);
    } catch (e) {
      alert(String(e));
      (root.querySelector("#b-run") as HTMLButtonElement).disabled = false;
    }
  });
  root.querySelector("#b-cancel")!.addEventListener("click", () => cancelRun());
}
```

- [ ] **Step 4: Font bundle + `src/main.ts`** — nav switching + render setup:

```bash
bun add @fontsource/ibm-plex-sans @fontsource/ibm-plex-mono
```

```ts
import "./styles.css";
import "@fontsource/ibm-plex-sans/400.css";
import "@fontsource/ibm-plex-sans/500.css";
import "@fontsource/ibm-plex-sans/600.css";
import "@fontsource/ibm-plex-mono/400.css";
import "@fontsource/ibm-plex-mono/500.css";
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

- [ ] **Step 5: `src/styles.css`** — design system penuh (tema broadcast-desk) + gate build:

```css
/* Good-Badminton Studio — broadcast-desk theme (calm, informative, profesional) */

:root {
  --bg: #0a1116;
  --surface: #101a21;
  --surface-2: #15222b;
  --line: #1d2f3b;
  --line-soft: #16242e;
  --ink: #dce8ef;
  --muted: #8098a9;
  --faint: #4e6577;
  --accent: #43dcc9;
  --accent-ink: #06231f;
  --accent-soft: rgba(67, 220, 201, 0.1);
  --accent-line: rgba(67, 220, 201, 0.38);
  --ok: #74d68f;
  --warn: #e0b552;
  --err: #e57b74;
  --err-soft: rgba(229, 123, 116, 0.12);
  --radius: 10px;
  --radius-sm: 7px;
  --font-sans: "IBM Plex Sans", ui-sans-serif, sans-serif;
  --font-mono: "IBM Plex Mono", ui-monospace, monospace;
}

* { box-sizing: border-box; }

body {
  margin: 0;
  color: var(--ink);
  font: 14px/1.5 var(--font-sans);
  background:
    radial-gradient(1100px 480px at 72% -8%, rgba(67, 220, 201, 0.055), transparent 60%),
    repeating-linear-gradient(90deg, transparent 0 119px, rgba(255, 255, 255, 0.014) 119px 120px),
    var(--bg);
  min-height: 100vh;
}

::selection { background: var(--accent-line); color: #fff; }

:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }

/* --- topbar --- */
.topbar {
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  height: 52px;
  padding: 0 20px;
  background: rgba(10, 17, 22, 0.86);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid var(--line);
}
.brand { display: flex; align-items: center; gap: 10px; }
.brand-mark { width: 22px; height: 22px; color: var(--accent); }
.brand-name {
  font: 600 12.5px/1 var(--font-mono);
  letter-spacing: 0.14em;
  color: var(--ink);
}
.brand-name em { font-style: normal; color: var(--accent); }

.tabs { display: flex; gap: 4px; }
.tab {
  position: relative;
  border: 0;
  background: none;
  color: var(--muted);
  font: 500 12px/1 var(--font-sans);
  letter-spacing: 0.1em;
  text-transform: uppercase;
  padding: 10px 14px;
  cursor: pointer;
  transition: color 0.15s ease;
}
.tab:hover { color: var(--ink); }
.tab.active { color: var(--ink); }
.tab.active::after {
  content: "";
  position: absolute;
  left: 14px;
  right: 14px;
  bottom: -1px;
  height: 2px;
  background: var(--accent);
  border-radius: 2px 2px 0 0;
}

/* --- layout --- */
main { padding: 20px 22px 36px; max-width: 1280px; margin: 0 auto; }

.pane-grid {
  display: grid;
  grid-template-columns: minmax(360px, 1fr) 1.25fr;
  gap: 18px;
  align-items: start;
}
.stack { display: flex; flex-direction: column; gap: 18px; min-width: 0; }

@media (max-width: 960px) {
  .pane-grid { grid-template-columns: 1fr; }
}

/* --- cards + stagger --- */
.card {
  background: var(--surface);
  border: 1px solid var(--line-soft);
  border-radius: var(--radius);
  padding: 16px 18px 18px;
}
.card-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin-bottom: 14px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--line-soft);
}
.card-head h2 {
  margin: 0;
  font: 600 11.5px/1 var(--font-mono);
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--muted);
}

.view:not([hidden]) > .pane-grid > .stack > .card,
.view:not([hidden]) > .hist-grid {
  animation: rise 0.38s ease both;
}
.view:not([hidden]) > .pane-grid > .stack > .card:nth-child(2) { animation-delay: 0.06s; }
.view:not([hidden]) > .pane-grid > .stack:nth-child(2) > .card:nth-child(1) { animation-delay: 0.12s; }
.view:not([hidden]) > .pane-grid > .stack:nth-child(2) > .card:nth-child(2) { animation-delay: 0.18s; }
@keyframes rise {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: none; }
}

/* --- fields --- */
.field { margin-bottom: 12px; }
.field:last-child { margin-bottom: 0; }
.field-label {
  display: block;
  margin-bottom: 6px;
  font: 500 11.5px/1 var(--font-sans);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
}
.field-label .opt {
  font-style: normal;
  font-weight: 400;
  letter-spacing: 0.04em;
  text-transform: none;
  color: var(--faint);
  margin-left: 6px;
}
.field-half { flex: 1; min-width: 120px; margin-bottom: 0; }

.picker { display: flex; gap: 8px; }

input, select {
  width: 100%;
  min-width: 0;
  padding: 8px 11px;
  color: var(--ink);
  background: var(--surface-2);
  border: 1px solid var(--line);
  border-radius: var(--radius-sm);
  font: 13.5px/1.4 var(--font-sans);
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}
input::placeholder { color: var(--faint); }
input:focus, select:focus {
  outline: none;
  border-color: var(--accent-line);
  box-shadow: 0 0 0 3px var(--accent-soft);
}
.picker input { flex: 1; }
select {
  appearance: none;
  background-image: linear-gradient(45deg, transparent 50%, var(--muted) 50%),
    linear-gradient(135deg, var(--muted) 50%, transparent 50%);
  background-position: calc(100% - 17px) 55%, calc(100% - 12px) 55%;
  background-size: 5px 5px;
  background-repeat: no-repeat;
  padding-right: 30px;
}
.num { width: 96px; font-family: var(--font-mono); }

/* --- buttons --- */
.btn {
  padding: 8px 15px;
  border-radius: var(--radius-sm);
  border: 1px solid transparent;
  font: 500 13px/1.2 var(--font-sans);
  cursor: pointer;
  white-space: nowrap;
  transition: filter 0.15s ease, border-color 0.15s ease, background 0.15s ease;
}
.btn:disabled { opacity: 0.42; cursor: not-allowed; }
.btn-primary {
  background: var(--accent);
  color: var(--accent-ink);
  font-weight: 600;
}
.btn-primary:hover:not(:disabled) { filter: brightness(1.08); }
.btn-ghost {
  background: transparent;
  border-color: var(--line);
  color: var(--ink);
}
.btn-ghost:hover:not(:disabled) { border-color: var(--accent-line); }
.btn-danger {
  background: transparent;
  border-color: rgba(229, 123, 116, 0.45);
  color: var(--err);
}
.btn-danger:hover:not(:disabled) { background: var(--err-soft); }

.actions { display: flex; gap: 10px; margin-top: 16px; }

/* --- parameters --- */
.row { display: flex; align-items: flex-end; gap: 18px; margin-bottom: 14px; }

.switch-row {
  display: flex;
  align-items: center;
  gap: 9px;
  font-size: 13.5px;
  color: var(--ink);
  cursor: pointer;
}
.switch {
  appearance: none;
  width: 36px;
  height: 19px;
  flex: none;
  border-radius: 99px;
  background: var(--line);
  border: none;
  position: relative;
  cursor: pointer;
  transition: background 0.18s ease;
  padding: 0;
}
.switch::before {
  content: "";
  position: absolute;
  top: 2.5px;
  left: 3px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--muted);
  transition: transform 0.18s ease, background 0.18s ease;
}
.switch:checked { background: var(--accent); }
.switch:checked::before { transform: translateX(16px); background: var(--accent-ink); }

.params { border: 0; margin: 0; padding: 0; }
.params legend {
  padding: 0;
  margin-bottom: 8px;
  font: 500 11.5px/1 var(--font-sans);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
}
.tog-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}
.tog {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 9px 11px;
  background: var(--surface-2);
  border: 1px solid var(--line-soft);
  border-radius: var(--radius-sm);
  font-size: 13px;
  cursor: pointer;
  transition: border-color 0.15s ease, background 0.15s ease;
}
.tog:hover { border-color: var(--line); }
.tog:has(input:checked) {
  border-color: var(--accent-line);
  background: var(--accent-soft);
}
.tog input {
  width: auto;
  accent-color: var(--accent);
  padding: 0;
}

/* --- progress + console --- */
.progress-card progress {
  width: 100%;
  height: 10px;
  appearance: none;
  border: 1px solid var(--line-soft);
  border-radius: 6px;
  background: var(--surface-2);
  overflow: hidden;
  display: block;
}
progress::-webkit-progress-bar { background: var(--surface-2); }
progress::-webkit-progress-value {
  background: linear-gradient(90deg, rgba(67, 220, 201, 0.55), var(--accent));
  border-radius: 5px;
}
progress::-moz-progress-bar {
  background: linear-gradient(90deg, rgba(67, 220, 201, 0.55), var(--accent));
  border-radius: 5px;
}
.mono { font-family: var(--font-mono); }
#eta {
  display: block;
  margin-top: 9px;
  text-align: right;
  font-size: 12.5px;
  color: var(--muted);
  font-variant-numeric: tabular-nums;
}

.console { padding-bottom: 0; }
.console .card-head { margin-bottom: 0; border-bottom: 0; padding-bottom: 0; }
#log {
  margin: 12px -18px -18px;
  padding: 13px 16px;
  height: min(38vh, 420px);
  overflow: auto;
  background: #080e12;
  border-top: 1px solid var(--line-soft);
  border-radius: 0 0 var(--radius) var(--radius);
  color: #9fbccd;
  font: 12.5px/1.6 var(--font-mono);
  white-space: pre-wrap;
  word-break: break-all;
}
#log::-webkit-scrollbar { width: 10px; }
#log::-webkit-scrollbar-thumb { background: var(--line); border-radius: 5px; border: 2px solid #080e12; }

/* --- Task 7 hooks: court picker --- */
.court-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 14px;
}
.court-bar .field-inline {
  display: flex;
  align-items: center;
  gap: 8px;
  font: 500 11.5px/1 var(--font-sans);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
}
.chip {
  padding: 5px 11px;
  background: var(--surface-2);
  border: 1px solid var(--line);
  border-radius: 99px;
  font: 500 12px/1.3 var(--font-mono);
  color: var(--accent);
  font-variant-numeric: tabular-nums;
}
.canvas-wrap {
  display: flex;
  justify-content: center;
  padding: 18px;
  overflow: auto;
}
#c-canvas {
  max-width: 100%;
  height: auto;
  border: 1px solid var(--line);
  border-radius: 6px;
  cursor: crosshair;
  box-shadow: 0 18px 44px rgba(0, 0, 0, 0.45);
}

/* --- Task 8 hooks: history --- */
.hist-grid {
  display: grid;
  grid-template-columns: minmax(300px, 0.9fr) 1.4fr;
  gap: 18px;
  align-items: start;
}
@media (max-width: 960px) {
  .hist-grid { grid-template-columns: 1fr; }
}
.hist-list {
  list-style: none;
  margin: 0;
  padding: 0;
  background: var(--surface);
  border: 1px solid var(--line-soft);
  border-radius: var(--radius);
  overflow: hidden;
}
.hist-item {
  display: grid;
  grid-template-columns: auto 1fr;
  grid-template-areas: "status name" "status meta";
  gap: 2px 12px;
  padding: 12px 14px;
  border-bottom: 1px solid var(--line-soft);
  cursor: pointer;
  transition: background 0.12s ease;
}
.hist-item:last-child { border-bottom: 0; }
.hist-item:hover { background: var(--surface-2); }
.hist-status {
  grid-area: status;
  align-self: start;
  padding: 3px 8px;
  border-radius: 99px;
  font: 600 10.5px/1.4 var(--font-mono);
  letter-spacing: 0.06em;
  text-transform: uppercase;
}
.s-ok { background: rgba(116, 214, 143, 0.14); color: var(--ok); }
.s-failed { background: var(--err-soft); color: var(--err); }
.s-cancelled { background: rgba(128, 152, 169, 0.14); color: var(--muted); }
.hist-name { grid-area: name; font-size: 13.5px; font-weight: 500; }
.hist-meta {
  grid-area: meta;
  font-size: 12px;
  color: var(--muted);
  font-variant-numeric: tabular-nums;
}
.hist-empty {
  padding: 26px 14px;
  text-align: center;
  color: var(--faint);
  font-size: 13px;
  cursor: default;
}
.hist-detail { min-height: 200px; }
.hist-detail:empty::after {
  content: "Pilih run untuk melihat preview";
  display: block;
  padding: 64px 0;
  text-align: center;
  color: var(--faint);
  font-size: 13px;
}
.hist-video {
  display: block;
  width: 100%;
  max-height: 70vh;
  border-radius: 6px;
  background: #000;
}

@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after { animation: none !important; transition: none !important; }
}
```

```bash
bun run build
```
Expected: exit 0 (tsc + vite).

- [ ] **Step 6: Capability untuk "Buka folder output"** — di `src-tauri/capabilities/default.json` tambahkan `"opener:allow-open-path"` ke array `permissions` (Task 2 hanya memasang `opener:default` yang TIDAK mencakup `open_path` — tanpa ini ACL menolak `openPath()` saat tombol diklik).

- [ ] **Step 7: Commit**

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
    <div class="court-bar">
      <button id="c-open" class="btn btn-ghost" type="button">Buka template…</button>
      <span id="c-pts" class="chip">Titik: 0/4</span>
      <label class="field-inline">mid_height <input id="c-mid" class="num" type="number" value="625" /></label>
      <button id="c-save" class="btn btn-primary" type="button" disabled>Simpan annotations</button>
    </div>
    <div class="canvas-wrap card"><canvas id="c-canvas" width="478" height="850"></canvas></div>`;
  const canvas = root.querySelector("#c-canvas") as HTMLCanvasElement;
  const ctx = canvas.getContext("2d")!;
  const pts: [number, number][] = [];
  let img: HTMLImageElement | null = null;
  let imgPath = "";

  const redraw = () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    if (img) ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
    // connector antar titik (aksen teal; dashed sampai 4 titik lengkap)
    if (pts.length >= 2) {
      ctx.strokeStyle = "rgba(67, 220, 201, 0.9)";
      ctx.lineWidth = 2;
      ctx.setLineDash(pts.length === 4 ? [] : [6, 4]);
      ctx.beginPath();
      pts.forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)));
      if (pts.length === 4) ctx.closePath();
      ctx.stroke();
      ctx.setLineDash([]);
    }
    // marker bernomor (urutan klik)
    ctx.font = "600 11px 'IBM Plex Mono', monospace";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    pts.forEach(([x, y], i) => {
      ctx.beginPath();
      ctx.arc(x, y, 9, 0, Math.PI * 2);
      ctx.fillStyle = "#43dcc9";
      ctx.fill();
      ctx.strokeStyle = "#06231f";
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.fillStyle = "#06231f";
      ctx.fillText(String(i + 1), x, y + 0.5);
    });
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
  root.innerHTML = `
    <div class="hist-grid">
      <ul id="h-list" class="hist-list">${
        items.length
          ? items
              .map(
                (e) => `<li class="hist-item" data-out="${e.output_dir}" data-video="${e.video}">
          <span class="hist-status s-${e.status}">${e.status}</span>
          <span class="hist-name">${e.video.split(/[\\/]/).pop()}</span>
          <span class="hist-meta mono">${e.elapsed_sec.toFixed(0)}s · rally ${e.rally_count ?? "—"}</span>
        </li>`,
              )
              .join("")
          : `<li class="hist-empty">Belum ada run — jalankan analisis di tab Run</li>`
      }</ul>
      <div id="h-detail" class="hist-detail card"></div>
    </div>`;
  root.querySelectorAll("#h-list li").forEach((li) => {
    li.addEventListener("click", async () => {
      const out = (li as HTMLElement).dataset.out!;
      const name = (li as HTMLElement).dataset.video!.split(/[\\/]/).pop()!.replace(/\.[^.]+$/, "");
      const detail = root.querySelector("#h-detail") as HTMLElement;
      if (!out) return;
      const src = convertFileSrc(`${out}/detect_${name}.mp4`);
      detail.innerHTML = `<video class="hist-video" controls src="${src}"></video>`;
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

### Task 10: Flag toggle display pipeline (repo Cpp)

**Files:**
- Modify: `Good-Badminton-Cpp/src/system.h:34-52` (SystemOptions + ganti komentar ponytail)
- Modify: `Good-Badminton-Cpp/src/main.cpp` (6 arg + help)
- Modify: `Good-Badminton-Cpp/src/system.cpp` (ctor init-list + body)

**Interfaces:**
- Produces: 6 flag CLI baru — `--skeletons|--player-trajectories|--court-trajectory|--shuttlecock-trajectory|--player-stats|--pose-roi`, semua `true|false`, **default true** (identik Python main.py; parity default tak boleh berubah). Dipakai Task 5 `args_for`.

- [ ] **Step 1: SystemOptions** — ganti blok komentar ponytail (`system.h:47-51`) menjadi:

```cpp
    bool show_skeletons = true;              // main.py --skeletons default true
    bool show_player_trajectories = true;    // main.py --player-trajectories default true
    bool show_court_trajectory = true;       // main.py --court-trajectory default true
    bool show_shuttlecock_trajectory = true; // main.py --shuttlecock-trajectory default true
    bool show_player_stats = true;           // main.py --player-stats default true
    bool show_pose_roi = true;               // main.py --pose-roi default true

    // ponytail: --save-images / --visualize-positions tetap pin ke default
    // Python (false, tanpa flag CLI); heatmaps tetap Python-side per spec.
```

- [ ] **Step 2: Parse + help** — di blok arg `main.cpp` (pola `--audio`), enam cabang + enam baris help:

```cpp
        } else if (a == "--skeletons") {
            if (!parse_bool(need("--skeletons"), opts.show_skeletons)) bad_value = true;
        } else if (a == "--player-trajectories") {
            if (!parse_bool(need("--player-trajectories"), opts.show_player_trajectories)) bad_value = true;
        } else if (a == "--court-trajectory") {
            if (!parse_bool(need("--court-trajectory"), opts.show_court_trajectory)) bad_value = true;
        } else if (a == "--shuttlecock-trajectory") {
            if (!parse_bool(need("--shuttlecock-trajectory"), opts.show_shuttlecock_trajectory)) bad_value = true;
        } else if (a == "--player-stats") {
            if (!parse_bool(need("--player-stats"), opts.show_player_stats)) bad_value = true;
        } else if (a == "--pose-roi") {
            if (!parse_bool(need("--pose-roi"), opts.show_pose_roi)) bad_value = true;
        }
```
```
                 "  --skeletons true|false     draw pose skeleton (default: true)\n"
                 "  --player-trajectories true|false  player dots+trails (default: true)\n"
                 "  --court-trajectory true|false     court trajectory overlay (default: true)\n"
                 "  --shuttlecock-trajectory true|false  shuttle trail (default: true)\n"
                 "  --player-stats true|false  stats panel (default: true)\n"
                 "  --pose-roi true|false       pose ROI overlay (default: true)\n"
```
(Petakan tiap flag ke field `opts.show_*` yang sesuai; `bad_value` adalah variabel bool yang sudah ada di loop arg.)

- [ ] **Step 3: Wiring ctor** (`system.cpp`) — init-list (sekitar baris 128-131) tambah tiga entry:

```cpp
      show_pose_roi_(opts_.show_pose_roi),
      show_court_trajectory_(opts_.show_court_trajectory),
      show_player_stats_(opts_.show_player_stats),
```
Di body ctor, ganti `sp.show_trajectory = true;` (baris ~159) menjadi:
```cpp
    sp.show_trajectory = opts_.show_shuttlecock_trajectory;
```
dan setel public field renderer (setelah `pose_analyzer_` dibuat atau di mana pun ctor body sudah mengakses anggota — sebelum loop video):
```cpp
    skeleton_renderer_.show_skeletons = opts_.show_skeletons;
    skeleton_renderer_.show_player_trajectories = opts_.show_player_trajectories;
```
(Field ada di `include/gb/viz.h:71-72`, public, default true.)

- [ ] **Step 4: Build**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Cpp/build
cmake --build . --config Release --target gb_cpp
```
Expected: exit 0.

- [ ] **Step 5: Verifikasi — satu check runnable** (pre-seed annotations karena auto-detect gagal headless):

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Cpp
mkdir -p outputs/t10a outputs/t10b
cp outputs/test4/court_annotations.txt outputs/t10a/ && cp outputs/test4/court_annotations.txt outputs/t10b/
./build/Release/gb_cpp.exe tests/fixtures/short.mp4 --template "C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton/templates/test4.png" --out outputs/t10a > /dev/null 2>&1; echo A=$?
./build/Release/gb_cpp.exe tests/fixtures/short.mp4 --template "C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton/templates/test4.png" --out outputs/t10b --skeletons false --pose-roi false --player-stats false --court-trajectory false --shuttlecock-trajectory false --player-trajectories false > /dev/null 2>&1; echo B=$?
ffmpeg -y -loglevel error -i outputs/t10a/detect_short.mp4 -vf "select=eq(n\,75)" -vframes 1 /tmp/t10a.png
ffmpeg -y -loglevel error -i outputs/t10b/detect_short.mp4 -vf "select=eq(n\,75)" -vframes 1 /tmp/t10b.png
md5sum /tmp/t10a.png /tmp/t10b.png
```
Expected: `A=0 B=0`, dua md5 **berbeda** (overlay berbeda → pixel beda). Regresi: jalankan ulang A tanpa flag apa pun → exit 0 dan video identik-semantik (tidak ada perubahan perilaku default).

- [ ] **Step 6: Commit**

```bash
cd C:/Users/Ulin/Documents/kerjaan/riset/Good-Badminton-Cpp
git add src/system.h src/main.cpp src/system.cpp
git commit -m "feat: expose python display toggles as CLI flags (defaults unchanged)"
```

---

## Self-Review (plan author)

- **Spec coverage:** dasar (T5+T6), picker sudut (T4+T7), preview (T8), riwayat (T4+T8), `--progress-json` (T1), config/ffmpeg detect (T4/T5), Bun (Global Constraints + T2), installer (T9). ✔
- **Placeholder scan:** signature `RunState.child` = `Arc<Mutex<Option<Child>>>` konsisten; `saveAnnotations(path, corners, mid)` TS ↔ Rust `save_annotations(path, corners, mid_height)` — invoke key `midHeight` → command arg `mid_height` (Tauri case conversion otomatis snake↔camel). `list_history` didefinisikan eksplisit di T8 Step 1. ✔
- **Type consistency:** `progress::Progress` derive Serialize → payload event langsung; `history::Entry` Serialize+Deserialize untuk invoke balik. ✔
