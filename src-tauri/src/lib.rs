// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod annotations;
mod config;
mod history;
mod pipeline;
mod progress;
mod rally;

#[tauri::command]
async fn start_run(
    app: tauri::AppHandle,
    state: tauri::State<'_, pipeline::RunState>,
    params: pipeline::RunParams,
) -> Result<(), String> {
    use tauri::Manager;
    let path = app.path().app_config_dir().unwrap().join("config.json");
    let cfg = config::load(&path)
        .map_err(|e| e.to_string())?
        // first launch: config.json belum ada → fallback ke auto-detect sibling
        .or_else(|| config::detect(&std::env::current_dir().unwrap_or_default()))
        .ok_or_else(|| format!("config tidak ada dan sibling tidak terdeteksi — buat config.json di {}", path.display()))?;
    pipeline::spawn_run(app, state, cfg, params).await
}

#[tauri::command]
async fn cancel_run(state: tauri::State<'_, pipeline::RunState>) -> Result<(), String> {
    if let Some(mut c) = state.child.lock().unwrap().take() {
        let _ = c.start_kill();
    }
    Ok(())
}

#[tauri::command]
fn get_defaults(app: tauri::AppHandle) -> (Option<config::Config>, String, bool) {
    use tauri::Manager;
    let dir = app.path().app_config_dir().unwrap_or_default();
    let cfg = config::load(&dir.join("config.json")).ok().flatten();
    let base = std::env::current_dir().unwrap_or_default();
    let detected = cfg.or_else(|| config::detect(&base));
    (detected, dir.to_string_lossy().into_owned(), which_ffmpeg())
}

#[tauri::command]
fn save_config(app: tauri::AppHandle, cfg: config::Config) -> Result<(), String> {
    use tauri::Manager;
    let dir = app.path().app_config_dir().unwrap();
    config::save(&dir.join("config.json"), &cfg).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_annotations(path: String, corners: [[i32; 2]; 4], mid_height: i32) -> Result<(), String> {
    annotations::write_annotations(std::path::Path::new(&path), &corners, mid_height)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_history(app: tauri::AppHandle) -> Vec<history::Entry> {
    use tauri::Manager;
    let dir = app.path().app_config_dir().unwrap_or_default();
    history::load(&dir.join("history.json")).unwrap_or_default()
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

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(pipeline::RunState { child: Default::default() })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_defaults,
            start_run,
            cancel_run,
            save_config,
            save_annotations,
            list_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
