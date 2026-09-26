//! Headless acceptance experiment. No PTY, clipboard, GPU or control service.
use alacritty_terminal::Term;
use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::Config;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::vte::ansi;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

pub const CORPUS: &[u8] = include_bytes!("../../../tests/fixtures/terminal-replay.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Suite {
    schema_version: u32,
    fixtures: Vec<Fixture>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    id: String,
    requirement: String,
    columns: usize,
    lines: usize,
    history_limit: usize,
    operations: Vec<Operation>,
    expected: Expected,
    #[serde(default)]
    known_gap: Option<String>,
    #[serde(default)]
    known_gap_state_sha256: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Operation {
    Feed { text: String },
    Resize { columns: usize, lines: usize },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    visible_text: Vec<String>,
    history_text: Vec<String>,
    cursor: [usize; 2],
    wrap_pending: bool,
    #[serde(default)]
    cells: Vec<ExpectedCell>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedCell {
    line: usize,
    column: usize,
    text: String,
    flags: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CellState {
    text: String,
    flags: u16,
    foreground: String,
    background: String,
    underline_color: Option<String>,
    hyperlink_uri: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    columns: usize,
    lines: usize,
    history_lines: usize,
    cursor: [usize; 2],
    wrap_pending: bool,
    mode_bits: u32,
    cells: Vec<Vec<CellState>>,
    events: Vec<String>,
}

#[derive(Serialize)]
pub struct FixtureResult {
    pub id: String,
    requirement: String,
    pub known_gap: Option<String>,
    known_gap_state_sha256: Option<String>,
    pub expectation_errors: Vec<String>,
    pub equivalence_errors: Vec<String>,
    pub replay_count: usize,
    state_sha256: String,
    pub final_state: State,
}

#[derive(Serialize)]
pub struct Report {
    schema_version: u32,
    engine: &'static str,
    engine_version: &'static str,
    corpus_sha256: String,
    measurement_kind: &'static str,
    pub results: Vec<FixtureResult>,
}

impl Report {
    pub fn is_acceptable(&self, allow_known_gaps: bool) -> bool {
        self.results.iter().all(|r| {
            r.equivalence_errors.is_empty()
                && match &r.known_gap {
                    None => r.expectation_errors.is_empty(),
                    // An unexpected pass also requires review: retire the documented gap.
                    Some(_) => {
                        allow_known_gaps
                            && !r.expectation_errors.is_empty()
                            && r.known_gap_state_sha256.as_ref() == Some(&r.state_sha256)
                    }
                }
        })
    }
}

#[derive(Clone, Default)]
struct Events(Rc<RefCell<Vec<String>>>);
impl EventListener for Events {
    fn send_event(&self, event: Event) {
        // Capture synthetic fixture effects without executing them.
        self.0.borrow_mut().push(format!("{event:?}"));
    }
}

struct Size {
    columns: usize,
    lines: usize,
}
impl Dimensions for Size {
    fn total_lines(&self) -> usize {
        self.lines
    }
    fn screen_lines(&self) -> usize {
        self.lines
    }
    fn columns(&self) -> usize {
        self.columns
    }
}

#[derive(Clone, Copy, Debug)]
enum Delivery {
    Whole,
    Chunks(usize),
    Split(usize),
}

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn parse_suite(bytes: &[u8]) -> Result<Suite, String> {
    let suite: Suite = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if suite.schema_version != 1 || suite.fixtures.is_empty() || suite.fixtures.len() > 100 {
        return Err("expected schema 1 and 1..100 fixtures".into());
    }
    let mut ids = BTreeSet::new();
    for f in &suite.fixtures {
        if f.id.is_empty() || !ids.insert(&f.id) || f.requirement.is_empty() {
            return Err("fixture IDs must be nonempty/unique and requirements nonempty".into());
        }
        if f.known_gap.is_some() != f.known_gap_state_sha256.is_some()
            || f.known_gap_state_sha256.as_ref().is_some_and(|s| {
                s.len() != 64
                    || !s
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
        {
            return Err(format!(
                "{}: known gaps require a reviewed state SHA-256",
                f.id
            ));
        }
        let valid_size = |cols, rows| (2..=160).contains(&cols) && (1..=60).contains(&rows);
        if !valid_size(f.columns, f.lines)
            || f.history_limit > 1000
            || f.operations.is_empty()
            || f.operations.len() > 50
            || f.known_gap.as_ref().is_some_and(|s| s.trim().is_empty())
        {
            return Err(format!(
                "{}: invalid size, history, operations or gap reason",
                f.id
            ));
        }
        let (mut columns, mut lines) = (f.columns, f.lines);
        let mut bytes = 0;
        for op in &f.operations {
            match op {
                Operation::Feed { text } => bytes += text.len(),
                Operation::Resize {
                    columns: c,
                    lines: l,
                } => {
                    if !valid_size(*c, *l) {
                        return Err(format!("{}: invalid resize", f.id));
                    }
                    (columns, lines) = (*c, *l);
                }
            }
        }
        if bytes > 4096
            || f.expected.visible_text.len() != lines
            || f.expected.history_text.len() > f.history_limit
            || f.expected.cursor[0] >= lines
            || f.expected.cursor[1] >= columns
            || f.expected
                .cells
                .iter()
                .any(|c| c.line >= lines || c.column >= columns)
        {
            return Err(format!(
                "{}: invalid expected geometry or oversized input",
                f.id
            ));
        }
    }
    Ok(suite)
}

fn snapshot(term: &Term<Events>, events: &Events) -> State {
    let grid = term.grid();
    let cells = (-i32::try_from(grid.history_size()).expect("bounded history")
        ..grid.screen_lines() as i32)
        .map(|line| {
            (0..grid.columns())
                .map(|column| {
                    let c = &grid[Line(line)][Column(column)];
                    let mut text = c.c.to_string();
                    text.extend(c.zerowidth().unwrap_or_default());
                    CellState {
                        text,
                        flags: c.flags.bits(),
                        foreground: format!("{:?}", c.fg),
                        background: format!("{:?}", c.bg),
                        underline_color: c.underline_color().map(|v| format!("{v:?}")),
                        hyperlink_uri: c.hyperlink().map(|v| v.uri().to_owned()),
                    }
                })
                .collect()
        })
        .collect();
    State {
        columns: grid.columns(),
        lines: grid.screen_lines(),
        history_lines: grid.history_size(),
        cursor: [
            grid.cursor.point.line.0 as usize,
            grid.cursor.point.column.0,
        ],
        wrap_pending: grid.cursor.input_needs_wrap,
        mode_bits: term.mode().bits(),
        cells,
        events: events.0.borrow().clone(),
    }
}

fn replay(f: &Fixture, delivery: Delivery) -> Vec<State> {
    let events = Events::default();
    let config = Config {
        scrolling_history: f.history_limit,
        ..Config::default()
    };
    let mut term = Term::new(
        config,
        &Size {
            columns: f.columns,
            lines: f.lines,
        },
        events.clone(),
    );
    let mut parser: ansi::Processor = ansi::Processor::new();
    let mut checkpoints = Vec::new();
    for op in &f.operations {
        match op {
            Operation::Feed { text } => {
                let bytes = text.as_bytes();
                match delivery {
                    Delivery::Whole => parser.advance(&mut term, bytes),
                    Delivery::Chunks(n) => {
                        for chunk in bytes.chunks(n) {
                            parser.advance(&mut term, chunk);
                        }
                    }
                    Delivery::Split(n) => {
                        let (a, b) = bytes.split_at(n.min(bytes.len()));
                        parser.advance(&mut term, a);
                        parser.advance(&mut term, b);
                    }
                }
            }
            Operation::Resize { columns, lines } => term.resize(Size {
                columns: *columns,
                lines: *lines,
            }),
        }
        checkpoints.push(snapshot(&term, &events));
    }
    checkpoints
}

fn row_text(row: &[CellState]) -> String {
    row.iter()
        .filter(|c| {
            c.flags & (Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER).bits() == 0
        })
        .map(|c| c.text.as_str())
        .collect::<String>()
        .trim_end_matches(' ')
        .to_owned()
}

fn check_expected(s: &State, expected: &Expected) -> Vec<String> {
    let mut errors = Vec::new();
    let rows: Vec<_> = s.cells.iter().map(|r| row_text(r)).collect();
    let visible = &rows[s.history_lines..];
    let history = &rows[..s.history_lines];
    if visible != expected.visible_text {
        errors.push(format!(
            "visible text: expected {:?}, got {visible:?}",
            expected.visible_text
        ));
    }
    if history != expected.history_text {
        errors.push(format!(
            "history text: expected {:?}, got {history:?}",
            expected.history_text
        ));
    }
    if s.cursor != expected.cursor {
        errors.push(format!(
            "cursor: expected {:?}, got {:?}",
            expected.cursor, s.cursor
        ));
    }
    if s.wrap_pending != expected.wrap_pending {
        errors.push(format!(
            "wrap_pending: expected {}, got {}",
            expected.wrap_pending, s.wrap_pending
        ));
    }
    for c in &expected.cells {
        let actual = &s.cells[s.history_lines + c.line][c.column];
        if actual.text != c.text || actual.flags != c.flags {
            errors.push(format!(
                "cell [{},{}]: expected text {:?}/flags {}, got {:?}/{}",
                c.line, c.column, c.text, c.flags, actual.text, actual.flags
            ));
        }
    }
    errors
}

pub fn run_suite() -> Result<Report, String> {
    let suite = parse_suite(CORPUS)?;
    let results = suite
        .fixtures
        .iter()
        .map(|f| {
            let baseline = replay(f, Delivery::Whole);
            let final_state = baseline.last().expect("validated operations").clone();
            let expectation_errors = check_expected(&final_state, &f.expected);
            let max_len = f
                .operations
                .iter()
                .filter_map(|op| match op {
                    Operation::Feed { text } => Some(text.len()),
                    _ => None,
                })
                .max()
                .unwrap_or(0);
            let deliveries = [
                Delivery::Whole,
                Delivery::Chunks(1),
                Delivery::Chunks(2),
                Delivery::Chunks(3),
                Delivery::Chunks(7),
            ]
            .into_iter()
            .chain((1..max_len).map(Delivery::Split));
            let mut replay_count = 1;
            let mut equivalence_errors = Vec::new();
            for delivery in deliveries {
                replay_count += 1;
                if replay(f, delivery) != baseline {
                    equivalence_errors
                        .push(format!("operation checkpoints differ with {delivery:?}"));
                }
            }
            let bytes = serde_json::to_vec(&final_state).expect("serializable state");
            FixtureResult {
                id: f.id.clone(),
                requirement: f.requirement.clone(),
                known_gap: f.known_gap.clone(),
                known_gap_state_sha256: f.known_gap_state_sha256.clone(),
                expectation_errors,
                equivalence_errors,
                replay_count,
                state_sha256: hash(&bytes),
                final_state,
            }
        })
        .collect();
    Ok(Report {
        schema_version: 1,
        engine: "alacritty_terminal",
        engine_version: "0.26.0",
        corpus_sha256: hash(CORPUS),
        measurement_kind: "headless_correctness_not_performance",
        results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corpus_meets_expectations_or_reports_documented_gaps() {
        let report = run_suite().unwrap();
        assert!(
            report.is_acceptable(true),
            "{}",
            serde_json::to_string_pretty(&report).unwrap()
        );
    }
    #[test]
    fn changing_an_expected_cursor_is_detected() {
        let mut suite = parse_suite(CORPUS).unwrap();
        let f = &mut suite.fixtures[0];
        let state = replay(f, Delivery::Whole).pop().unwrap();
        f.expected.cursor[1] = (f.expected.cursor[1] + 1) % f.columns;
        assert!(
            check_expected(&state, &f.expected)
                .iter()
                .any(|e| e.starts_with("cursor:"))
        );
    }
    #[test]
    fn invalid_dimensions_unknown_fields_and_duplicate_ids_are_rejected() {
        let original: serde_json::Value = serde_json::from_slice(CORPUS).unwrap();
        for key in ["columns", "history_limit"] {
            let mut data = original.clone();
            data["fixtures"][0][key] = 1_000_000.into();
            assert!(parse_suite(&serde_json::to_vec(&data).unwrap()).is_err());
        }
        let mut data = original.clone();
        data["fixtures"][0]["typo"] = true.into();
        assert!(parse_suite(&serde_json::to_vec(&data).unwrap()).is_err());
        let mut data = original;
        data["fixtures"][1]["id"] = data["fixtures"][0]["id"].clone();
        assert!(parse_suite(&serde_json::to_vec(&data).unwrap()).is_err());
    }
    #[test]
    fn gap_tolerance_never_hides_chunking_failure_or_unexpected_pass() {
        let mut report = run_suite().unwrap();
        report.results[0].known_gap = Some("test-only gap".into());
        report.results[0].expectation_errors.clear();
        assert!(!report.is_acceptable(true));
        report.results[0]
            .expectation_errors
            .push("unmet requirement".into());
        report.results[0]
            .equivalence_errors
            .push("chunk mismatch".into());
        assert!(!report.is_acceptable(true));
        assert!(!report.is_acceptable(false));
    }

    #[test]
    fn changed_known_gap_state_requires_new_review() {
        let mut report = run_suite().unwrap();
        assert!(report.is_acceptable(true));
        let gap = report
            .results
            .iter_mut()
            .find(|r| r.known_gap.is_some())
            .unwrap();
        gap.state_sha256 = "0".repeat(64);
        assert!(!report.is_acceptable(true));
    }
}
