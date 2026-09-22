use std::process::Command;

#[test]
fn help_flags_exit_successfully_without_compiling_a_file() {
    for flag in ["-h", "--help"] {
        let output = Command::new(env!("CARGO_BIN_EXE_storyscript-parser"))
            .arg(flag)
            .output()
            .expect("parser process should start");

        assert!(
            output.status.success(),
            "{flag} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
        assert!(stdout.contains("Usage: storyscript-parser"));
        assert!(stdout.contains("--json"));
    }
}
