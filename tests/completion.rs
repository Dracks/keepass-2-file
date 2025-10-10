use std::process::Command;

#[test]
fn test_completion_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "completion", "bash"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("keepass-2-file"));
    assert!(stdout.contains("set__default__kp__db"));
}
