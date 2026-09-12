mod cli;

fn main() {
    if let Err(error) = cli::run() {
        if std::env::args().any(|argument| argument == "--json") {
            eprintln!(
                "{}",
                serde_json::json!({
                    "status": "error",
                    "code": error.code().to_string(),
                    "message": error.to_string(),
                })
            );
        } else {
            eprintln!("{}: {}", error.code(), error);
        }
        std::process::exit(1);
    }
}
