#![forbid(unsafe_code)]

use forgehelpyou::{
    clamp_context_lines, clamp_line_count, clamp_match_limit, default_text_source,
    is_valid_priority, looks_like_problem, resolve_source, search_file_with_context, source_state,
    tail_file, ContextLine, LogSource, SourceState, DEFAULT_LINE_COUNT, DEFAULT_MATCH_LIMIT,
    LOG_SOURCES, VERSION,
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
        "recent" => recent_command(&args[1..]),
        "errors" => errors_command(&args[1..]),
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
A small command-line tool for reading Linux logs.

USAGE:
  forgehelpyou <COMMAND> [OPTIONS]

COMMANDS:
  recent [OPTIONS]               Show recent messages from an available log
  errors [SOURCE] [OPTIONS]      Find common warning and error messages
  sources                        List known log files and their status
  show <SOURCE> [OPTIONS]        Show the last lines from a file
  search <SOURCE> <TEXT>         Search a file
  journal [OPTIONS]              Read the systemd journal, if available
  help                           Show this help

SOURCE:
  Use an alias (syslog, auth, kern, dpkg, ...) or a file path.

OPTIONS:
  --lines <N>                    Number of lines to read (default: 100, max: 5000)
  --contains <TEXT>              Keep only lines containing this text (show)
  --ignore-case                  Ignore letter case (show/search)
  --limit <N>                    Maximum search results (default: 100)
  --context <N>                  Show N lines before and after each match (max: 20)

JOURNAL OPTIONS:
  --since <TIME>                 Example: '1 hour ago' or 'today'
  --priority <LEVEL>             emerg, alert, crit, err, warning, notice, info, debug
  --unit <NAME>                  Filter by systemd unit, e.g. ssh.service
  --boot                         Show entries from the current boot

EXAMPLES:
  forgehelpyou recent
  forgehelpyou errors
  forgehelpyou sources
  forgehelpyou show auth --lines 80
  forgehelpyou search syslog "connection refused" --ignore-case --context 2
  forgehelpyou journal --since "1 hour ago" --priority warning

ForgeHelpYou reads local logs. It does not create, change, or upload them.
It is a small log-reading tool, not an application logger or a replacement for tracing.
"#
    );
}

fn list_sources() -> Result<(), String> {
    println!("Known Linux log files:\n");
    for source in LOG_SOURCES {
        print_source(source);
    }
    println!("\nSystemd journal:");
    if command_exists("journalctl") {
        println!("  [available] journalctl found; try: forgehelpyou journal");
    } else {
        println!("  [not available] journalctl not found; file-based logs can still be used");
    }
    Ok(())
}

fn print_source(source: &LogSource) {
    let state = source_state(Path::new(source.path));
    let (label, detail) = match state {
        SourceState::Readable => ("readable", ""),
        SourceState::PermissionDenied => (
            "permission denied",
            " — use an account allowed to read this file",
        ),
        SourceState::NotFound => ("not present", ""),
        SourceState::OtherError => ("unavailable", ""),
    };
    println!("  [{label:16}] {:12} {}{detail}", source.key, source.path);
}

fn recent_command(args: &[String]) -> Result<(), String> {
    let lines = parse_lines_option(args, DEFAULT_LINE_COUNT)?;
    if let Some(path) = default_text_source() {
        let entries = tail_file(&path, lines).map_err(|error| explain_file_error(&path, error))?;
        println!("Source: {}\n", path.display());
        if entries.is_empty() {
            println!("The log is empty.");
        } else {
            for line in entries {
                println!("{}", safe_terminal_text(&line));
            }
        }
        Ok(())
    } else if command_exists("journalctl") {
        journal_command(&["--lines".to_owned(), lines.to_string()])
    } else {
        Err("no readable text logs found and journalctl is not available. Run 'forgehelpyou sources' to inspect log files.".into())
    }
}

fn errors_command(args: &[String]) -> Result<(), String> {
    let mut source_arg: Option<&str> = None;
    let mut lines = 500usize;
    let mut index = 0;

    if let Some(first) = args.first() {
        if !first.starts_with("--") {
            source_arg = Some(first);
            index = 1;
        }
    }
    while index < args.len() {
        match args[index].as_str() {
            "--lines" => {
                lines = parse_number(args, &mut index, "--lines")?;
            }
            other => return Err(format!("unknown errors option '{other}'")),
        }
    }
    let lines = clamp_line_count(lines);

    let path = match source_arg {
        Some(source) => resolve_source(source),
        None => match default_text_source() {
            Some(path) => path,
            None if command_exists("journalctl") => {
                return journal_command(&[
                    "--priority".to_owned(), "warning".to_owned(),
                    "--lines".to_owned(), lines.to_string(),
                ]);
            }
            None => return Err("no readable text logs found and journalctl is not available. Run 'forgehelpyou sources' to inspect log files.".into()),
        },
    };

    let entries = tail_file(&path, lines).map_err(|error| explain_file_error(&path, error))?;
    let mut found = 0usize;
    println!(
        "Source: {} (checked the last {lines} lines)\n",
        path.display()
    );
    for line in entries {
        if looks_like_problem(&line) {
            println!("{}", safe_terminal_text(&line));
            found += 1;
        }
    }
    if found == 0 {
        println!("No common warning/error keywords found in that part of the log.");
    } else {
        println!("\nFound {found} possible warning/error line(s).");
    }
    Ok(())
}

fn parse_lines_option(args: &[String], default: usize) -> Result<usize, String> {
    let mut lines = default;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--lines" => {
                lines = parse_number(args, &mut index, "--lines")?;
            }
            other => return Err(format!("unknown option '{other}'")),
        }
    }
    Ok(clamp_line_count(lines))
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

    if let Some(needle) = contains {
        let entries = forgehelpyou::recent_matching_lines(
            &path,
            Some(&needle),
            ignore_case,
            clamp_line_count(lines),
        )
        .map_err(|error| explain_file_error(&path, error))?;
        if entries.is_empty() {
            println!("No matching lines found in {}.", path.display());
        } else {
            println!("Source: {}\n", path.display());
            for entry in entries {
                println!(
                    "{:>7} | {}",
                    entry.line_number,
                    safe_terminal_text(&entry.text)
                );
            }
        }
    } else {
        let entries = tail_file(&path, clamp_line_count(lines))
            .map_err(|error| explain_file_error(&path, error))?;
        if entries.is_empty() {
            println!("No lines found in {}.", path.display());
        } else {
            println!("Source: {}\n", path.display());
            for line in entries {
                println!("{}", safe_terminal_text(&line));
            }
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
    let mut context = 0usize;
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
            "--context" => {
                context = parse_number(args, &mut index, "--context")?;
            }
            other => return Err(format!("unknown search option '{other}'")),
        }
    }

    let result = search_file_with_context(
        &path,
        needle,
        ignore_case,
        clamp_match_limit(limit),
        clamp_context_lines(context),
    )
    .map_err(|error| explain_file_error(&path, error))?;
    println!("Source: {}", path.display());
    println!(
        "Scanned up to {} line(s); found {} match(es).",
        result.lines_scanned,
        result.matches.len()
    );
    for line in result.lines {
        print_context_line(&line);
    }
    if result.results_truncated {
        println!("\nResult limit reached. Increase --limit to see more matches.");
    }
    Ok(())
}

fn print_context_line(line: &ContextLine) {
    let marker = if line.is_match { '>' } else { ' ' };
    println!(
        "{marker} {:>7} | {}",
        line.line_number,
        safe_terminal_text(&line.text)
    );
}

fn journal_command(args: &[String]) -> Result<(), String> {
    if !command_exists("journalctl") {
        return Err("journalctl is not available. This system may use file-based logs; run 'forgehelpyou sources'.".into());
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

    let output = command
        .arg(format!("--lines={lines}"))
        .output()
        .map_err(|error| format!("could not start journalctl: {error}"))?;
    if output.status.success() {
        print_external_lines(&output.stdout);
        Ok(())
    } else {
        let detail = String::from_utf8_lossy(&output.stderr);
        let detail = safe_terminal_text(detail.trim());
        if detail.is_empty() {
            Err(format!("journalctl exited with status {}. You may not have permission to read the journal.", output.status))
        } else {
            Err(format!("journalctl: {detail}"))
        }
    }
}

fn print_external_lines(bytes: &[u8]) {
    let output = String::from_utf8_lossy(bytes);
    for line in output.lines() {
        println!("{}", safe_terminal_text(line));
    }
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
        io::ErrorKind::PermissionDenied => format!("permission denied reading {}. Use an account allowed to read this log; do not change its permissions.", path.display()),
        io::ErrorKind::NotFound => format!("{} was not found. Run 'forgehelpyou sources' to see available logs.", path.display()),
        _ => format!("cannot read {}: {error}", path.display()),
    }
}
