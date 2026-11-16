#![forbid(unsafe_code)]

//! A small, read-only helper for viewing and searching local Linux logs.
//!
//! The program uses only the Rust standard library. It does not create logs,
//! change system settings, or send log contents over the network.

use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_LINE_COUNT: usize = 100;
pub const MAX_LINE_COUNT: usize = 5_000;
pub const DEFAULT_MATCH_LIMIT: usize = 100;
pub const MAX_MATCH_LIMIT: usize = 10_000;

#[derive(Clone, Copy, Debug)]
pub struct LogSource {
    pub key: &'static str,
    pub path: &'static str,
    pub description: &'static str,
}

pub const LOG_SOURCES: &[LogSource] = &[
    LogSource { key: "syslog", path: "/var/log/syslog", description: "General system messages (common on Debian-based systems)" },
    LogSource { key: "messages", path: "/var/log/messages", description: "General system messages (used by some distributions)" },
    LogSource { key: "auth", path: "/var/log/auth.log", description: "Authentication events (Debian-based systems)" },
    LogSource { key: "secure", path: "/var/log/secure", description: "Authentication events (some other distributions)" },
    LogSource { key: "kern", path: "/var/log/kern.log", description: "Kernel messages" },
    LogSource { key: "daemon", path: "/var/log/daemon.log", description: "Background service messages" },
    LogSource { key: "dpkg", path: "/var/log/dpkg.log", description: "Package installation activity" },
    LogSource { key: "boot", path: "/var/log/boot.log", description: "Boot messages, when configured" },
    LogSource { key: "xorg", path: "/var/log/Xorg.0.log", description: "X.Org display server log" },
    LogSource { key: "apt-history", path: "/var/log/apt/history.log", description: "APT package transaction history" },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogMatch {
    pub line_number: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub matches: Vec<LogMatch>,
    pub lines_scanned: usize,
    pub results_truncated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceState {
    Readable,
    PermissionDenied,
    NotFound,
    OtherError,
}

pub fn resolve_source(input: &str) -> PathBuf {
    if let Some(source) = LOG_SOURCES.iter().find(|source| source.key == input) {
        PathBuf::from(source.path)
    } else {
        PathBuf::from(input)
    }
}

/// Pick a general-purpose text log that the current user can read.
/// Syslog/messages are preferred over specialized logs such as dpkg.log.
pub fn default_text_source() -> Option<PathBuf> {
    ["syslog", "messages", "kern", "daemon", "auth", "secure"]
        .iter()
        .map(|key| resolve_source(key))
        .find(|path| source_state(path) == SourceState::Readable)
}

fn open_regular_file(path: &Path) -> io::Result<File> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "log source is not a regular file",
        ));
    }
    File::open(path)
}

pub fn source_state(path: &Path) -> SourceState {
    match open_regular_file(path) {
        Ok(_) => SourceState::Readable,
        Err(error) if error.kind() == io::ErrorKind::NotFound => SourceState::NotFound,
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => SourceState::PermissionDenied,
        Err(_) => SourceState::OtherError,
    }
}

pub fn clamp_line_count(value: usize) -> usize {
    value.clamp(1, MAX_LINE_COUNT)
}

pub fn clamp_match_limit(value: usize) -> usize {
    value.clamp(1, MAX_MATCH_LIMIT)
}

fn decode_reverse_line(bytes_reversed: &mut Vec<u8>) -> io::Result<String> {
    bytes_reversed.reverse();
    if bytes_reversed.last() == Some(&b'\r') {
        bytes_reversed.pop();
    }
    String::from_utf8(std::mem::take(bytes_reversed))
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

/// Read the last `count` lines by walking backwards through the file.
/// Memory use is limited to the requested lines and one 8 KiB read buffer.
pub fn tail_file(path: &Path, count: usize) -> io::Result<Vec<String>> {
    const CHUNK_SIZE: u64 = 8 * 1024;

    let mut file = open_regular_file(path)?;
    let file_len = file.seek(SeekFrom::End(0))?;
    if file_len == 0 {
        return Ok(Vec::new());
    }

    let count = clamp_line_count(count);
    let mut position = file_len;
    let mut pending_reversed = Vec::new();
    let mut lines = VecDeque::with_capacity(count);

    // A final newline terminates the last line; it does not add another line.
    file.seek(SeekFrom::End(-1))?;
    let mut last_byte = [0_u8; 1];
    file.read_exact(&mut last_byte)?;
    let mut skip_final_separator = last_byte[0] == b'\n';

    while position > 0 && lines.len() < count {
        let chunk_len = position.min(CHUNK_SIZE) as usize;
        position -= chunk_len as u64;
        file.seek(SeekFrom::Start(position))?;

        let mut chunk = vec![0_u8; chunk_len];
        file.read_exact(&mut chunk)?;

        for byte in chunk.iter().rev() {
            if *byte == b'\n' {
                if skip_final_separator {
                    skip_final_separator = false;
                    continue;
                }
                lines.push_front(decode_reverse_line(&mut pending_reversed)?);
                if lines.len() == count {
                    break;
                }
            } else {
                pending_reversed.push(*byte);
            }
        }
    }

    // The bytes before the first separator are the first line. This also
    // correctly preserves an empty first line in a file containing "\n".
    if lines.len() < count && position == 0 {
        lines.push_front(decode_reverse_line(&mut pending_reversed)?);
    }

    Ok(lines.into_iter().collect())
}

/// Search a text log line by line. Only matching lines are retained in memory.
pub fn search_file(
    path: &Path,
    needle: &str,
    ignore_case: bool,
    limit: usize,
) -> io::Result<SearchResult> {
    let file = open_regular_file(path)?;
    let reader = BufReader::new(file);
    let mut matches = Vec::new();
    let limit = clamp_match_limit(limit);
    let needle_normalized = if ignore_case { needle.to_lowercase() } else { needle.to_owned() };
    let mut lines_scanned = 0usize;
    let mut results_truncated = false;

    for line in reader.lines() {
        let line = line?;
        lines_scanned = lines_scanned.saturating_add(1);
        let haystack = if ignore_case { line.to_lowercase() } else { line.clone() };
        if haystack.contains(&needle_normalized) {
            if matches.len() < limit {
                matches.push(LogMatch { line_number: lines_scanned, text: line });
            } else {
                results_truncated = true;
                break;
            }
        }
    }

    Ok(SearchResult { matches, lines_scanned, results_truncated })
}

/// Keep the last N lines that match an optional substring.
pub fn recent_matching_lines(
    path: &Path,
    needle: Option<&str>,
    ignore_case: bool,
    count: usize,
) -> io::Result<Vec<LogMatch>> {
    let file = open_regular_file(path)?;
    let reader = BufReader::new(file);
    let count = clamp_line_count(count);
    let needle_normalized = needle.map(|s| if ignore_case { s.to_lowercase() } else { s.to_owned() });
    let mut matches = VecDeque::with_capacity(count);

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        let is_match = match needle_normalized.as_deref() {
            None => true,
            Some(needle) if ignore_case => line.to_lowercase().contains(needle),
            Some(needle) => line.contains(needle),
        };
        if is_match {
            if matches.len() == count {
                matches.pop_front();
            }
            matches.push_back(LogMatch { line_number: index + 1, text: line });
        }
    }

    Ok(matches.into_iter().collect())
}

/// A deliberately small keyword filter for the `errors` convenience command.
pub fn looks_like_problem(text: &str) -> bool {
    let lower = text.to_lowercase();
    ["error", "failed", "failure", "warning", "critical", "denied", "segfault", "panic", "fatal"]
        .iter()
        .any(|word| lower.contains(*word))
}

pub fn is_valid_priority(value: &str) -> bool {
    matches!(value.to_ascii_lowercase().as_str(), "emerg" | "alert" | "crit" | "err" | "warning" | "notice" | "info" | "debug")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_file(contents: &str) -> PathBuf {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("forgehelpyou-test-{}-{nonce}.log", std::process::id()));
        std::fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn resolves_known_source_alias() {
        assert_eq!(resolve_source("syslog"), PathBuf::from("/var/log/syslog"));
    }

    #[test]
    fn unknown_source_is_treated_as_path() {
        assert_eq!(resolve_source("./sample.log"), PathBuf::from("./sample.log"));
    }

    #[test]
    fn clamps_line_count() {
        assert_eq!(clamp_line_count(0), 1);
        assert_eq!(clamp_line_count(99_999), MAX_LINE_COUNT);
    }

    #[test]
    fn tail_returns_last_lines_in_order() {
        let path = temporary_file("one\ntwo\nthree\nfour\n");
        let result = tail_file(&path, 2).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(result, vec!["three", "four"]);
    }

    #[test]
    fn tail_handles_missing_final_newline_and_utf8() {
        let path = temporary_file("первая\nвторая");
        let result = tail_file(&path, 1).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(result, vec!["вторая"]);
    }

    #[test]
    fn tail_keeps_blank_lines() {
        let path = temporary_file("one\n\n");
        let result = tail_file(&path, 2).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(result, vec!["one", ""]);
    }

    #[test]
    fn tail_handles_a_file_larger_than_one_read_chunk() {
        let contents = (0..20_000).map(|n| format!("line-{n}\n")).collect::<String>();
        let path = temporary_file(&contents);
        let result = tail_file(&path, 2).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(result, vec!["line-19998", "line-19999"]);
    }

    #[test]
    fn searches_case_insensitively_and_reports_line_numbers() {
        let path = temporary_file("started\nSSH failed\nfinished\n");
        let result = search_file(&path, "ssh", true, 10).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].line_number, 2);
        assert_eq!(result.matches[0].text, "SSH failed");
    }

    #[test]
    fn search_marks_truncated_results() {
        let path = temporary_file("error one\nerror two\nerror three\n");
        let result = search_file(&path, "error", false, 2).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(result.matches.len(), 2);
        assert!(result.results_truncated);
    }

    #[test]
    fn recent_matching_lines_keeps_latest_matches() {
        let path = temporary_file("ok\nfailed one\nok\nfailed two\nfailed three\n");
        let result = recent_matching_lines(&path, Some("failed"), false, 2).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(result.iter().map(|m| m.line_number).collect::<Vec<_>>(), vec![4, 5]);
    }

    #[test]
    fn problem_filter_matches_common_words() {
        assert!(looks_like_problem("service failed to start"));
        assert!(looks_like_problem("WARNING: disk is nearly full"));
        assert!(!looks_like_problem("service started successfully"));
    }

    #[test]
    fn validates_syslog_priorities() {
        assert!(is_valid_priority("warning"));
        assert!(is_valid_priority("ERR"));
        assert!(!is_valid_priority("urgent"));
    }
}
