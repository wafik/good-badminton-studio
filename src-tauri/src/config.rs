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

/// Resource bundle tertanam installer (tauri.conf `bundle.resources`):
/// `<resource_dir>/engine/gb_cpp(.exe)` + `<resource_dir>/models/*.onnx`.
/// Menang atas sibling detect — app terpasang harus self-contained.
pub fn detect_resource(resource: &Path) -> Option<Config> {
    let exe = if cfg!(windows) {
        resource.join("engine/gb_cpp.exe")
    } else {
        resource.join("engine/gb_cpp")
    };
    let ball = resource.join("models/yolo11s-ball.onnx");
    let pose = resource.join("models/yolo11n-pose-dyn.onnx");
    if exe.is_file() && ball.is_file() && pose.is_file() {
        Some(Config {
            gb_cpp_path: exe.to_string_lossy().into_owned(),
            data_dir: resource.join("models").to_string_lossy().into_owned(),
            ball_model: ball.to_string_lossy().into_owned(),
            pose_model: pose.to_string_lossy().into_owned(),
            defaults: RunDefaults::default(),
        })
    } else {
        None
    }
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
    fn detect_resource_requires_engine_and_models() {
        let root = std::env::temp_dir().join(format!("gb_res_{}", std::process::id()));
        std::fs::create_dir_all(&root).ok();
        assert!(detect_resource(&root).is_none());
        std::fs::create_dir_all(root.join("engine")).unwrap();
        std::fs::create_dir_all(root.join("models")).unwrap();
        let exe = if cfg!(windows) {
            root.join("engine/gb_cpp.exe")
        } else {
            root.join("engine/gb_cpp")
        };
        std::fs::write(&exe, b"x").unwrap();
        std::fs::write(root.join("models/yolo11s-ball.onnx"), b"b").unwrap();
        // pose belum ada → belum lengkap, harus None
        assert!(detect_resource(&root).is_none());
        std::fs::write(root.join("models/yolo11n-pose-dyn.onnx"), b"p").unwrap();
        let cfg = detect_resource(&root).expect("resource");
        assert!(cfg.gb_cpp_path.contains("gb_cpp"));
        assert!(cfg.ball_model.ends_with("yolo11s-ball.onnx"));
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
