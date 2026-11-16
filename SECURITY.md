# Security and privacy

ForgeHelpYou 0.1.0 is a local, read-only log viewer and search utility.

## Scope

- It reads only the source selected by the user or the fixed list displayed by `sources` / `doctor`.
- It does not modify, delete, rotate, or upload log files.
- It does not automatically invoke `sudo` or change filesystem permissions.
- It starts `journalctl` directly with argument separation; it does not pass user input through a shell.
- Control characters in file-based log lines are escaped before terminal output.

## Sensitive information

System logs may contain usernames, hostnames, IP addresses, session metadata, URLs, and occasionally credentials or tokens. Do not publish raw logs without reviewing and redacting them first. Avoid attaching logs to public issues unless they have been sanitized.

## Reporting a vulnerability

Please do not publish sensitive vulnerability details in a public issue. Contact the repository maintainer through the private security reporting option on GitHub.

ForgeHelpYou is an early-stage project and has not undergone an independent security audit. Do not treat it as a security boundary or as a replacement for system access controls.
