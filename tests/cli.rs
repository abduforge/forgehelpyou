use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_log(contents: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("forgehelpyou-cli-{}-{nonce}.log", std::process::id()));
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn version_flag_prints_package_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_forgehelpyou"))
        .arg("--version")
        .output()
        .expect("run forgehelpyou");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("ForgeHelpYou 0.2.0"));
}

#[test]
fn help_lists_simple_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_forgehelpyou"))
        .arg("--help")
        .output()
        .expect("run forgehelpyou");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for item in ["recent", "errors", "sources", "show", "search"] {
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

#[test]
fn errors_command_filters_the_recent_part_of_a_file() {
    let path = temp_log("service started\nWARNING disk space low\nservice ready\nlogin failed\n");
    let output = Command::new(env!("CARGO_BIN_EXE_forgehelpyou"))
        .args(["errors", path.to_str().unwrap(), "--lines", "3"])
        .output()
        .expect("run forgehelpyou");
    fs::remove_file(path).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("WARNING disk space low"));
    assert!(stdout.contains("login failed"));
    assert!(!stdout.contains("service started"));
}

#[test]
fn log_control_characters_are_escaped_before_printing() {
    let path = temp_log("normal line\nunsafe \u{001b}[31mtext\u{001b}[0m\n");
    let output = Command::new(env!("CARGO_BIN_EXE_forgehelpyou"))
        .args(["show", path.to_str().unwrap()])
        .output()
        .expect("run forgehelpyou");
    fs::remove_file(path).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\\u{001b}"));
    assert!(!stdout.contains('\u{001b}'));
}
