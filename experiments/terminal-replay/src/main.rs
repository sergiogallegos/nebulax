use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let allow_known = match args.as_slice() {
        [] => false,
        [arg] if arg == "--allow-known-gaps" => true,
        [arg] if arg == "--help" => {
            println!(
                "nebulax-replay [--allow-known-gaps]\nRuns the embedded synthetic corpus; emits JSON.\nExit 1: requirement gap or unexpected result; exit 2: invalid invocation/data.\nKnown gaps remain failures in the report even when explicitly tolerated."
            );
            return ExitCode::SUCCESS;
        }
        _ => {
            eprintln!("usage: nebulax-replay [--allow-known-gaps]");
            return ExitCode::from(2);
        }
    };
    match nebulax_replay::run_suite() {
        Ok(report) => {
            let success = report.is_acceptable(allow_known);
            let mut out = io::stdout().lock();
            if serde_json::to_writer_pretty(&mut out, &report).is_err() || writeln!(out).is_err() {
                eprintln!("could not write replay report");
                return ExitCode::from(2);
            }
            ExitCode::from(u8::from(!success))
        }
        Err(error) => {
            eprintln!("invalid replay corpus: {error}");
            ExitCode::from(2)
        }
    }
}
