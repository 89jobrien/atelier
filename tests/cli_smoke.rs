use std::process::Command;

#[test]
fn help_lists_usage_and_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_atelier"))
        .arg("--help")
        .output()
        .expect("atelier should execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Commands:"));
}
