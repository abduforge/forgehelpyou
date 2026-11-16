# Changelog

## 0.1.0 — Initial release

- List common Linux log sources and report whether they are readable.
- Show recent log lines with an optional substring filter.
- Search text log files with case-sensitive or case-insensitive matching.
- Read the systemd journal when `journalctl` is available.
- Add an environment diagnostic command.
- Keep file operations read-only and local.
- Escape control characters from file-based log output before printing.
- Use only the Rust standard library.
