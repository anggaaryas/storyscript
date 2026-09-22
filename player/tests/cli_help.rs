use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn run_help_with_timeout(flag: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_storyscript-player"))
        .arg(flag)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("player process should start");
    let deadline = Instant::now() + Duration::from_secs(5);

    loop {
        match child
            .try_wait()
            .expect("player process should be observable")
        {
            Some(_) => {
                return child
                    .wait_with_output()
                    .expect("player output should be readable");
            }
            None if Instant::now() >= deadline => {
                child
                    .kill()
                    .expect("hung player process should be killable");
                let output = child
                    .wait_with_output()
                    .expect("hung player output should be readable");
                panic!(
                    "{flag} did not exit before the TUI timeout: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            None => thread::sleep(Duration::from_millis(10)),
        }
    }
}

#[test]
fn help_flags_exit_successfully_without_starting_the_tui() {
    for flag in ["-h", "--help"] {
        let output = run_help_with_timeout(flag);

        assert!(
            output.status.success(),
            "{flag} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
        assert!(stdout.contains("Usage: storyscript-player"));
        assert!(stdout.contains("[PATH]"));
    }
}
