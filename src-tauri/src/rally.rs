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
