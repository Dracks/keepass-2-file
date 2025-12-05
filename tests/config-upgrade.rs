use std::process::Command;

use test_helpers::TmpFile;

fn create_v1_config() -> TmpFile {
    let current_path = std::env::current_dir().unwrap();
    let current_path_string = current_path.to_str().unwrap();

    let file = TmpFile::new_uuid("test_resources/tmp/".into(), "yml".into());
    file.write(format!(
        "keepass: {current_path_string}/test_resources/db.kdbx
templates: null"
    ));

    file
}

#[test]
fn test_load_old_config() {
    let config = create_v1_config();
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--config",
            config.get().as_str(),
            "config",
            "get-kp-db",
        ])
        .output()
        .expect("Not executing the file");

    println!("-- {:?} --", output);
    // assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("db.kdbx"));
}

#[test]
fn test_upgrade_old_config() {
    let config = create_v1_config();
    Command::new("cargo")
        .args(&[
            "run",
            "--",
            "--config",
            config.get().as_str(),
            "config",
            "prune",
        ])
        .output()
        .expect("Not executing the file");

    // assert!(output.stderr.is_empty());
    let config_contents = config.read();
    println!("-- {config_contents} --");
    assert!(config_contents.contains("version: '2'"))
}
