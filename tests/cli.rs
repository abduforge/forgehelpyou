use std::process::Command;

#[test]
fn version_flag_prints_package_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_forgehelpyou"))
        .arg("--version")
        .output()
        .expect("run forgehelpyou");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ForgeHelpYou 0.1.0"));
}

#[test]
fn help_lists_primary_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_forgehelpyou"))
        .arg("--help")
        .output()
        .expect("run forgehelpyou");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for item in ["sources", "show", "search", "journal", "doctor"] {
        assert!(stdout.contains(item), "help missing {item}");
    }
}

#[test]
fn unknown_command_returns_nonzero() {
    let output = Command::new(env!("CARGO_BIN_EXE_forgehelpyou"))
        .arg("not-a-command")
        .output()
        .expect("run forgehelpyou");
    assert!(!output.status.success());
}

#[test]
fn empty_search_is_rejected() {
    let output = Command::new(env!("CARGO_BIN_EXE_forgehelpyou"))
        .args(["search", "./missing.log", ""])
        .output()
        .expect("run forgehelpyou");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be empty"));
}
