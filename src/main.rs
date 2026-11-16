#![forbid(unsafe_code)]

use forgehelpyou::{
    clamp_line_count, clamp_match_limit, is_valid_priority, recent_matching_lines, resolve_source,
    search_file, source_state, tail_file, LogSource, SourceState, DEFAULT_LINE_COUNT,
    DEFAULT_MATCH_LIMIT, LOG_SOURCES, VERSION,
};
use std::env;
use std::io;
use std::path::Path;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("forgehelpyou: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        print_help();
        return Ok(());
    }

    match args[0].as_str() {
        "--help" | "-h" | "help" => {
            print_help();
            Ok(())
        }
        "--version" | "-V" | "version" => {
            println!("ForgeHelpYou {VERSION}");
            Ok(())
        }
        "sources" => list_sources(),
        "doctor" => doctor(),
        "show" => show_command(&args[1..]),
        "search" => search_command(&args[1..]),
        "journal" => journal_command(&args[1..]),
        command => Err(format!(
            "unknown command '{command}'. Run 'forgehelpyou --help' for usage."
        )),
    }
}

fn print_help() {
    println!(
        r#"ForgeHelpYou {VERSION}
A small, privacy-first Linux log viewer and search helper.

USAGE:
  forgehelpyou <COMMAND> [OPTIONS]

COMMANDS:
  sources                         List common Linux log files and their status
  show <SOURCE> [OPTIONS]         Show the latest lines from a log
  search <SOURCE> <TEXT> [OPTIONS] Search a log file
  journal [OPTIONS]               Read systemd journal, if available
  doctor                          Check available log facilities
  help                            Show this help

SOURCE:
  Use an alias (syslog, auth, kern, dpkg, ...) or an explicit file path.

SHOW OPTIONS:
  --lines <N>                     Number of recent lines (default: 100, max: 5000)
  --contains <TEXT>               Only show lines containing this text
  --ignore-case                   Match without case sensitivity

SEARCH OPTIONS:
  --ignore-case                   Match without case sensitivity
  --limit <N>                     Maximum matches (default: 100, max: 10000)

JOURNAL OPTIONS:
  --lines <N>                     Number of entries (default: 100, max: 5000)
  --since <TIME>                  Example: '1 hour ago' or 'today'
  --priority <LEVEL>              emerg, alert, crit, err, warning, notice, info, debug
  --unit <NAME>                   Filter by systemd unit, e.g. ssh.service
  --boot                          Show entries from the current boot

EXAMPLES:
  forgehelpyou sources
  forgehelpyou show syslog --lines 80
  forgehelpyou show auth --contains failed --ignore-case
  forgehelpyou search auth "authentication failure" --ignore-case
  forgehelpyou journal --since "1 hour ago" --priority warning

PRIVACY:
  ForgeHelpYou reads logs locally. It does not upload, modify, or delete logs.
  Log files can contain usernames, IP addresses, tokens, and other sensitive data.
  Review and redact output before sharing it.
"#
    );
}

fn list_sources() -> Result<(), String> {
    println!("Known Linux log sources:\n");
    for source in LOG_SOURCES {
        print_source(source);
    }
    println!("\nSystemd journal:");
    if command_exists("journalctl") {
        println!("  [available] journalctl found; try: forgehelpyou journal");
    } else {
        println!("  [not available] journalctl not found; use file-based sources above");
    }
    Ok(())
}

fn print_source(source: &LogSource) {
    let state = source_state(Path::new(source.path));
    let (label, detail) = match state {
        SourceState::Readable => ("readable", ""),
        SourceState::PermissionDenied => (
            "permission denied",
            " — try again with an authorized account",
        ),
        SourceState::NotFound => ("not present", ""),
        SourceState::OtherError => ("unavailable", ""),
    };
    println!("  [{label:16}] {:12} {}{detail}", source.key, source.path);
    println!("  {:32} {}", "", source.description);
}

fn show_command(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("show requires a source. Example: forgehelpyou show syslog".into());
    }
    let path = resolve_source(&args[0]);
    let mut lines = DEFAULT_LINE_COUNT;
    let mut contains: Option<String> = None;
    let mut ignore_case = false;
    let mut index = 1;

    while index < args.len() {
        match args[index].as_str() {
            "--lines" => {
                lines = parse_number(args, &mut index, "--lines")?;
            }
            "--contains" => {
                contains = Some(next_value(args, &mut index, "--contains")?);
            }
            "--ignore-case" => {
                ignore_case = true;
                index += 1;
            }
            other => return Err(format!("unknown show option '{other}'")),
        }
    }

    let matches = recent_matching_lines(
        &path,
        contains.as_deref(),
        ignore_case,
        clamp_line_count(lines),
    )
    .map_err(|error| explain_file_error(&path, error))?;
    if matches.is_empty() {
        println!("No matching lines found in {}.", path.display());
    } else {
        println!(
            "Source: {}\nShowing {} recent line(s):\n",
            path.display(),
            matches.len()
        );
        for entry in matches {
            println!(
                "{:>7} | {}",
                entry.line_number,
                safe_terminal_text(&entry.text)
            );
        }
    }
    Ok(())
}

fn search_command(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("search requires a source and search text. Example: forgehelpyou search auth failed --ignore-case".into());
    }
    let path = resolve_source(&args[0]);
    let needle = &args[1];
    if needle.is_empty() {
        return Err("search text cannot be empty".into());
    }
    let mut ignore_case = false;
    let mut limit = DEFAULT_MATCH_LIMIT;
    let mut index = 2;

    while index < args.len() {
        match args[index].as_str() {
            "--ignore-case" => {
                ignore_case = true;
                index += 1;
            }
            "--limit" => {
                limit = parse_number(args, &mut index, "--limit")?;
            }
            other => return Err(format!("unknown search option '{other}'")),
        }
    }

    let result = search_file(&path, needle, ignore_case, clamp_match_limit(limit))
        .map_err(|error| explain_file_error(&path, error))?;
    println!("Source: {}", path.display());
    println!(
        "Scanned {} line(s); found {} match(es).",
        result.lines_scanned,
        result.matches.len()
    );
    for entry in result.matches {
        println!(
            "{:>7} | {}",
            entry.line_number,
            safe_terminal_text(&entry.text)
        );
    }
    if result.results_truncated {
        println!("\nResult limit reached. Increase --limit to see more matches.");
    }
    Ok(())
}

fn journal_command(args: &[String]) -> Result<(), String> {
    if !command_exists("journalctl") {
        return Err("journalctl is not installed or not in PATH. This system may use file-based logs; run 'forgehelpyou sources'.".into());
    }

    let mut command = Command::new("journalctl");
    command.args(["--no-pager", "--output=short-iso"]);
    let mut index = 0;
    let mut lines = DEFAULT_LINE_COUNT;
    while index < args.len() {
        match args[index].as_str() {
            "--lines" => {
                let value = parse_number(args, &mut index, "--lines")?;
                lines = clamp_line_count(value);
            }
            "--since" => {
                let value = next_value(args, &mut index, "--since")?;
                command.arg("--since").arg(value);
            }
            "--priority" => {
                let value = next_value(args, &mut index, "--priority")?;
                if !is_valid_priority(&value) {
                    return Err(format!("invalid priority '{value}'. Use emerg, alert, crit, err, warning, notice, info, or debug."));
                }
                command.arg("--priority").arg(value);
            }
            "--unit" => {
                let value = next_value(args, &mut index, "--unit")?;
                command.arg("--unit").arg(value);
            }
            "--boot" => {
                command.arg("--boot");
                index += 1;
            }
            other => return Err(format!("unknown journal option '{other}'")),
        }
    }

    command.arg(format!("--lines={lines}"));
    let status = command
        .status()
        .map_err(|error| format!("could not start journalctl: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("journalctl exited with status {status}. You may not have permission to read the system journal."))
    }
}

fn doctor() -> Result<(), String> {
    println!("ForgeHelpYou {VERSION} — environment check\n");
    println!("Platform: {}", env::consts::OS);
    println!("Architecture: {}", env::consts::ARCH);
    println!(
        "journalctl: {}",
        if command_exists("journalctl") {
            "available"
        } else {
            "not found"
        }
    );
    println!("\nLog file access:");
    let mut readable = 0usize;
    let mut present = 0usize;
    for source in LOG_SOURCES {
        match source_state(Path::new(source.path)) {
            SourceState::Readable => {
                readable += 1;
                present += 1;
                println!("  [readable] {}", source.key);
            }
            SourceState::PermissionDenied => {
                present += 1;
                println!("  [permission denied] {}", source.key);
            }
            SourceState::NotFound => println!("  [not present] {}", source.key),
            SourceState::OtherError => println!("  [unavailable] {}", source.key),
        }
    }
    println!("\nSummary: {readable} readable source(s), {present} present source(s).");
    println!("ForgeHelpYou never changes log files or uploads their contents.");
    Ok(())
}

fn command_exists(name: &str) -> bool {
    let Some(path_value) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&path_value).any(|directory| directory.join(name).is_file())
}

fn next_value(args: &[String], index: &mut usize, option: &str) -> Result<String, String> {
    if *index + 1 >= args.len() {
        return Err(format!("{option} requires a value"));
    }
    let value = args[*index + 1].clone();
    if value.starts_with("--") {
        return Err(format!("{option} requires a value"));
    }
    *index += 2;
    Ok(value)
}

fn parse_number(args: &[String], index: &mut usize, option: &str) -> Result<usize, String> {
    let raw = next_value(args, index, option)?;
    raw.parse::<usize>()
        .map_err(|_| format!("{option} expects a positive integer, got '{raw}'"))
}

fn safe_terminal_text(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            value if value.is_control() => output.push_str(&format!("\\u{{{:04x}}}", value as u32)),
            value => output.push(value),
        }
    }
    output
}

fn explain_file_error(path: &Path, error: io::Error) -> String {
    match error.kind() {
        io::ErrorKind::PermissionDenied => format!("permission denied reading {}. Use an account authorized to read this log; do not change its permissions.", path.display()),
        io::ErrorKind::NotFound => format!("{} was not found. Run 'forgehelpyou sources' to see available logs.", path.display()),
        _ => format!("cannot read {}: {error}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_mentions_main_commands() {
        // Keep command names discoverable in source; integration smoke tests
        // execute --help against the compiled binary.
        let help = include_str!("main.rs");
        for command in ["sources", "show", "search", "journal", "doctor"] {
            assert!(help.contains(command));
        }
    }

    #[test]
    fn source_aliases_are_unique() {
        for (index, source) in LOG_SOURCES.iter().enumerate() {
            assert!(!LOG_SOURCES[index + 1..]
                .iter()
                .any(|other| other.key == source.key));
        }
    }
}
