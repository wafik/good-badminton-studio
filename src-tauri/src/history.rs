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
