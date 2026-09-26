use std::process::Command;

#[test]
fn strict_and_explicit_gap_modes_preserve_the_same_evidence() {
    let strict = Command::new(env!("CARGO_BIN_EXE_nebulax-replay"))
        .output()
        .unwrap();
    assert_eq!(strict.status.code(), Some(1));
    let tolerated = Command::new(env!("CARGO_BIN_EXE_nebulax-replay"))
        .arg("--allow-known-gaps")
        .output()
        .unwrap();
    assert!(tolerated.status.success());
    assert_eq!(strict.stdout, tolerated.stdout);
    let report: serde_json::Value = serde_json::from_slice(&strict.stdout).unwrap();
    assert!(
        report["results"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| !f["expectation_errors"].as_array().unwrap().is_empty())
    );
}

#[test]
fn unknown_arguments_fail_without_a_success_report() {
    let result = Command::new(env!("CARGO_BIN_EXE_nebulax-replay"))
        .arg("--unknown")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
}
