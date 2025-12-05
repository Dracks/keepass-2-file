use std::process::Command;

use test_helpers::TmpFile;

fn create_v1_config() -> TmpFile {
    let current_path = std::env::current_dir().unwrap();
    let current_path_string = current_path.to_str().unwrap();

    let mut file = TmpFile::new_uuid("test_resources/tmp/".into(), "yml".into());
    file.disable_auto_clean();
    file.write(format!(
        "keepass: {current_path_string}/test_resources/db.kdbx
templates: null"
    ));

    file
}

#[test]
fn test_old_config() {
    let config = create_v1_config();
    let output = Command::new("cargo")
        .args(&["run", "--", "--config", config.get().as_str(), "config", "get-kp-db"])
        .output()
        .expect("Not executing the file");

    println!("-- {:?} --", output);
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(false)
}
