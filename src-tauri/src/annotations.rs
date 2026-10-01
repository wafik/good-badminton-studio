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
