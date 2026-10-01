use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq)]
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
