//! Independent expectation adapter for the owned engine, including history and resize.
use nebulax_terminal::{CellView as Cell, FeedOutcome, Limits, Size, Terminal, WidthPolicy};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const CORPUS: &[u8] = include_bytes!("../../../tests/fixtures/terminal-replay.json");

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn number(v: &Value, key: &str) -> usize {
    v[key].as_u64().expect("fixture number") as usize
}

fn state(t: &Terminal) -> Value {
    let cells: Vec<Vec<_>> = t
        .history()
        .iter()
        .chain(t.screen())
        .map(|row| {
            row.cells()
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let (text, mut flags) = match c.view() {
                        Cell::Empty => (" ".into(), 0),
                        Cell::Continuation => (" ".into(), 64),
                        Cell::WrapPadding => (" ".into(), 1024),
                        Cell::Lead { cluster, width } => (
                            cluster.chars().collect::<String>(),
                            if width == 2 { 32 } else { 0 },
                        ),
                    };
                    if row.soft_wrapped() && i + 1 == t.size().columns {
                        flags |= 16;
                    }
                    json!({"text": text, "flags": flags})
                })
                .collect()
        })
        .collect();
    let text: Vec<_> = cells
        .iter()
        .map(|r| {
            r.iter()
                .filter(|c| c["flags"].as_u64().unwrap() & (64 | 1024) == 0)
                .map(|c| c["text"].as_str().unwrap())
                .collect::<String>()
                .trim_end_matches(' ')
                .to_owned()
        })
        .collect();
    json!({"columns":t.size().columns,"lines":t.size().lines,"cells":cells,
        "visible_text":text[t.history().len()..],"history_text":text[..t.history().len()],
        "history_lines":t.history().len(),"alternate_screen":t.is_alternate(),"cursor":[t.cursor().row,t.cursor().column],
        "wrap_pending":t.cursor().wrap_pending})
}

fn replay(f: &Value, delivery: (&str, usize)) -> Vec<Value> {
    let mut t = Terminal::new(
        Size {
            columns: number(f, "columns"),
            lines: number(f, "lines"),
        },
        Limits {
            history_rows: number(f, "history_limit"),
            ..Limits::default()
        },
        WidthPolicy::default(),
    )
    .unwrap();
    let mut checkpoints = Vec::new();
    for op in f["operations"].as_array().unwrap() {
        if op["op"] == "resize" {
            let outcome = t
                .resize(Size {
                    columns: number(op, "columns"),
                    lines: number(op, "lines"),
                })
                .unwrap();
            assert_eq!(
                outcome,
                nebulax_terminal::ResizeOutcome::default(),
                "fixture unexpectedly discards retained state"
            );
            assert!(t.invariants_hold());
            checkpoints.push(state(&t));
            continue;
        }
        assert_eq!(op["op"], "feed");
        let bytes = op["text"].as_str().unwrap().as_bytes();
        let mut outcome = FeedOutcome::default();
        let mut feed = |b: &[u8]| {
            let progress = t.feed(b);
            assert_eq!(
                progress.consumed,
                b.len(),
                "fixture feed must not leave an unread suffix"
            );
            assert!(
                !progress.output_blocked && t.pending_output_len() == 0,
                "unexpected fixture output"
            );
            outcome.merge(progress);
            assert!(t.invariants_hold());
        };
        match delivery {
            ("chunks", n) => {
                for chunk in bytes.chunks(n) {
                    feed(chunk);
                }
            }
            ("split", n) => {
                let (a, b) = bytes.split_at(n.min(bytes.len()));
                feed(a);
                feed(b);
            }
            _ => feed(bytes),
        }
        assert!(
            !outcome.unsupported
                && !outcome.parser_limit
                && !outcome.cluster_limit
                && !outcome.style_limit
                && !outcome.orphan_mark
                && !outcome.scrolled_without_history
                && !outcome.history_evicted,
            "in-scope fixture generated unexpected diagnostics: {outcome:?}"
        );
        checkpoints.push(state(&t));
    }
    checkpoints
}

fn expected_errors(state: &Value, expected: &Value) -> Vec<Value> {
    let mut errors = Vec::new();
    for key in ["visible_text", "history_text", "cursor", "wrap_pending"] {
        if state[key] != expected[key] {
            errors.push(json!({"field":key,"expected":expected[key],"actual":state[key]}));
        }
    }
    for c in expected["cells"].as_array().unwrap() {
        let actual = &state["cells"][number(state, "history_lines") + number(c, "line")]
            [number(c, "column")];
        let wanted = json!({"text":c["text"],"flags":c["flags"]});
        if actual != &wanted {
            errors.push(json!({"field":format!("cell[{},{}]",c["line"],c["column"]),"expected":wanted,"actual":actual}));
        }
    }
    errors
}

pub fn run() -> Value {
    let suite: Value = serde_json::from_slice(CORPUS).unwrap();
    let mut results = Vec::new();
    let mut count = 0;
    for f in suite["fixtures"].as_array().unwrap() {
        let id = f["id"].as_str().unwrap();
        let baseline = replay(f, ("whole", 0));
        let end = baseline.last().unwrap();
        let max = f["operations"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|op| op["text"].as_str().map(str::len))
            .max()
            .unwrap();
        let variants = [
            ("repeat", 0),
            ("chunks", 1),
            ("chunks", 2),
            ("chunks", 3),
            ("chunks", 7),
        ]
        .into_iter()
        .chain((1..max).map(|n| ("split", n)));
        let mut equivalence_errors = Vec::new();
        let mut checkpoints = vec![hash(&serde_json::to_vec(&baseline).unwrap())];
        for d in variants {
            let actual = replay(f, d);
            checkpoints.push(hash(&serde_json::to_vec(&actual).unwrap()));
            if actual != baseline {
                equivalence_errors.push(json!({"delivery":d,"actual":actual}));
            }
        }
        let errors = expected_errors(end, &f["expected"]);
        count += checkpoints.len();
        results.push(json!({"id":id,"status":if errors.is_empty() && equivalence_errors.is_empty() {"match"} else {"difference"},
            "expectation_errors":errors,"equivalence_errors":equivalence_errors,"replay_count":checkpoints.len(),
            "checkpoint_sha256":checkpoints,"baseline_checkpoints":baseline,"final_state":end}));
    }
    json!({"schema_version":1,"engine":"nebulax-terminal","engine_version":"0.1.0","unicode_version":"18.0.0",
        "measurement_kind":"headless_correctness_not_performance","corpus_sha256":hash(CORPUS),
        "replay_count":count,"results":results})
}

pub fn acceptable(report: &Value) -> bool {
    report["results"]
        .as_array()
        .unwrap()
        .iter()
        .all(|r| r["status"] == "match")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_existing_expectations_pass_without_deferrals() {
        let r = run();
        assert!(acceptable(&r), "{r:#}");
        let results = r["results"].as_array().unwrap();
        assert_eq!(
            results.iter().filter(|r| r["status"] == "match").count(),
            19
        );
        assert_eq!(
            results.iter().filter(|r| r["status"] == "pending").count(),
            0
        );
    }
    #[test]
    fn wrong_expectations_are_detected() {
        let r = run();
        let s = &r["results"][0]["final_state"];
        let mut e = s.clone();
        e["cells"] = json!([]);
        e["cursor"] = json!([1, 1]);
        assert!(!expected_errors(s, &e).is_empty());
    }
}
