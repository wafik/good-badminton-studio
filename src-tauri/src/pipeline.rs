use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::{AppHandle, Emitter, Manager};

use crate::config::Config;
use crate::{history, progress, rally};

pub struct RunState {
    pub child: Arc<Mutex<Option<tokio::process::Child>>>,
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

pub async fn spawn_run(
    app: AppHandle,
    state: tauri::State<'_, RunState>,
    cfg: Config,
    params: RunParams,
) -> Result<(), String> {
    let child_arc = state.child.clone();
    {
        let child_opt = child_arc.lock().unwrap();
        if child_opt.is_some() {
            return Err("run already in progress".into());
        }
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
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    *child_arc.lock().unwrap() = Some(child);

    let started = Instant::now();
    let out_dir = params.out_dir.clone();
    let video = params.video.clone();
    let template = params.template.clone();
    let hist_path = state_hist_path(&app);

    tauri::async_runtime::spawn(async move {
        use tokio::io::{AsyncBufReadExt, BufReader};
        let app_out = app.clone();
        let app2 = app.clone();
        let out_task = tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(p) = progress::parse_line(&line) {
                    let _ = app_out.emit("progress", p);
                } else {
                    let _ = app_out.emit("log", serde_json::json!({"stream": "out", "line": line}));
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

        // child sudah dipindah ke state; ambil dari state (lepas lock
        // sebelum await — MutexGuard tidak Send) lalu tunggu exit.
        // take() mengembalikan None hanya bila cancel_run sudah mengambil
        // child lebih dulu (= dibatalkan user).
        let taken = child_arc.lock().unwrap().take();
        let (code, status_label) = match taken {
            Some(mut c) => match c.wait().await {
                Ok(st) => (st.code().unwrap_or(-1), String::new()),
                Err(_) => (-1, "cancelled".to_string()),
            },
            None => (-1, "cancelled".to_string()),
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
    format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    )
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
