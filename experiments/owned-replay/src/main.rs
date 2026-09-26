fn main() {
    if std::env::args_os().len() != 1 {
        eprintln!("usage: nebulax-owned-replay (no arguments)");
        std::process::exit(2);
    }
    let report = nebulax_owned_replay::run();
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    if !nebulax_owned_replay::acceptable(&report) {
        std::process::exit(1);
    }
}
