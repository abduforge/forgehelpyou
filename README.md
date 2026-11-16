<div align="center">

# ForgeHelpYou

**Linux logs, made easier.**

[![Download latest release](https://img.shields.io/github/v/release/abduforge/forgehelpyou?display_name=tag&style=for-the-badge&label=DOWNLOAD%20FOR%20LINUX)](https://github.com/abduforge/forgehelpyou/releases/latest)
[![CI](https://github.com/abduforge/forgehelpyou/actions/workflows/ci.yml/badge.svg)](https://github.com/abduforge/forgehelpyou/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Find a log. Read what happened. Search for the error. No complicated setup.**

[Русская документация](README_RU.md) · [Releases](https://github.com/abduforge/forgehelpyou/releases) · [Security](SECURITY.md)

</div>

---

## Get started

1. Click **DOWNLOAD FOR LINUX** above and download the archive for your system.
2. Extract it.
3. Open a terminal in the extracted folder and run:

```bash
./install.sh
```

4. List the log files available on your computer:

```bash
forgehelpyou sources
```

5. View recent system messages:

```bash
forgehelpyou show syslog --lines 80
```

That's it. If your distribution does not have `/var/log/syslog`, use `forgehelpyou sources` to see which files are available.

## What it does

ForgeHelpYou is a small, read-only Linux command-line tool for viewing and searching local system logs.

| You want to… | Run |
|---|---|
| See available log files | `forgehelpyou sources` |
| View the last 100 lines | `forgehelpyou show syslog` |
| View the last 80 lines | `forgehelpyou show syslog --lines 80` |
| Show only lines containing `failed` | `forgehelpyou show auth --contains failed --ignore-case` |
| Search an entire log | `forgehelpyou search auth "authentication failure" --ignore-case` |
| Read the systemd journal | `forgehelpyou journal --since "1 hour ago" --priority warning` |
| Check the local setup | `forgehelpyou doctor` |
| Get help | `forgehelpyou --help` |

Use a source alias (`syslog`, `auth`, `kern`, `dpkg`, etc.) or pass a log-file path directly.

## Supported log sources

ForgeHelpYou checks common paths including:

- `/var/log/syslog` and `/var/log/messages`
- `/var/log/auth.log` and `/var/log/secure`
- `/var/log/kern.log` and `/var/log/daemon.log`
- `/var/log/dpkg.log` and `/var/log/boot.log`
- `/var/log/Xorg.0.log`
- `/var/log/apt/history.log`

It can also read the systemd journal through `journalctl` when that tool is installed and the current account has permission. File-based logs are supported independently, which is useful on systems that do not run systemd.

## Search examples

Find failed login or authentication messages:

```bash
forgehelpyou search auth "failed" --ignore-case
```

Show recent kernel messages from a file:

```bash
forgehelpyou show kern --lines 120
```

Search a log you specify yourself:

```bash
forgehelpyou search ./my-app.log "connection refused" --ignore-case --limit 50
```

Try the included harmless sample log:

```bash
forgehelpyou search examples/sample-syslog.txt "failed" --ignore-case
```

Read warning-level journal entries from the last hour (systemd only):

```bash
forgehelpyou journal --since "1 hour ago" --priority warning
```

## Build from source

Requirements: Linux and Rust 1.70 or newer.

```bash
cargo fmt --all
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
./install.sh
```

The project uses only the Rust standard library; no third-party Rust crates are required.

## Privacy and safety

- ForgeHelpYou reads log files locally. It does not upload, modify, rotate, or delete them.
- It never runs commands through a shell. Journal filters are passed as process arguments.
- It does not automatically use `sudo` or change file permissions.
- Control characters in file-based log output are escaped before they are printed, reducing terminal-control injection risks.
- Logs can contain usernames, IP addresses, hostnames, session details, and sometimes secrets. Review and redact output before sharing it publicly.
- Only inspect systems and log files you own or are authorized to administer.

If a log is not readable, use an account that is authorized to access it. Do not make sensitive log files world-readable just to use this tool.

## Uninstall

```bash
./uninstall.sh
```

This removes the executable from `~/.local/bin`. It does not delete or alter any system logs.

## Project status

ForgeHelpYou 0.1.0 is an early release. It focuses on listing common log sources, viewing recent lines, searching text logs, and optionally reading the systemd journal. More features may be added after feedback and testing.

**Part of the AbduForge ecosystem.**
