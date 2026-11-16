# Changelog

## 0.2.0

- Add `recent` to find and display a readable general-purpose log automatically.
- Add `errors` to filter common warning and error keywords from recent log lines.
- Read the end of large files backwards in 8 KiB blocks for faster `show` output.
- Escape control characters in output from `journalctl` as well as text files.
- Simplify the command list and documentation.
- Use `cargo install` instead of custom install and uninstall scripts.
- Add tests for large files, blank lines, UTF-8, control characters, and the errors filter.

## 0.1.0

- List common Linux log files and report whether they are readable.
- Show recent log lines with an optional substring filter.
- Search text log files with case-sensitive or case-insensitive matching.
- Read the systemd journal when `journalctl` is available.
- Add a basic environment diagnostic command.
- Keep file operations read-only and local.
- Escape control characters from file-based log output before printing.
- Use only the Rust standard library.
