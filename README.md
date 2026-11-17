# ForgeHelpYou

A small Rust command-line tool for reading Linux logs. I made it because I wanted a quick way to check common log files without remembering every path and option.

It is a small learning project, not a logging framework and not a replacement for `journalctl`, `log`, or `tracing`. It reads logs that already exist.

## Install

For a prebuilt Linux x86_64 binary, download the archive from [Releases](https://github.com/abduforge/forgehelpyou/releases), then run:

```bash
tar -xzf forgehelpyou-*-x86_64-unknown-linux-gnu.tar.gz
install -Dm755 forgehelpyou/forgehelpyou ~/.local/bin/forgehelpyou
```

Or build and install from the source directory with Cargo:

```bash
cargo install --path .
```

Remove a Cargo installation with `cargo uninstall forgehelpyou`. Remove a manually installed binary with `rm ~/.local/bin/forgehelpyou`.

## Commands

```bash
forgehelpyou recent                         # recent messages from an available log
forgehelpyou errors                         # common warning/error lines
forgehelpyou sources                        # known log files and access status
forgehelpyou show auth --lines 80           # last 80 lines from auth.log
forgehelpyou search syslog "failed" --ignore-case --context 2
forgehelpyou journal --since "1 hour ago"  # systemd only
```

`recent` checks common text logs first. If none can be read, it tries `journalctl` when available. `errors` checks the last 500 lines by default; pass a source path or alias and `--lines N` to change the input. Its keyword list is intentionally simple, so it can miss unusual messages or match harmless ones.

`search --context N` prints up to N lines before and after each match. Overlapping context is merged, so the same line is not printed twice. The context value is limited to 20 lines on each side.

Use `forgehelpyou --help` for all options. A source can be an alias such as `syslog`, `auth`, or `kern`, or a path to a log file.

## How it reads files

- `show` reads backwards in small blocks, so asking for the last few lines does not require reading the whole file into memory.
- `search` reads one line at a time and keeps only a limited number of matches plus requested context.
- Invalid UTF-8 in a log is replaced with `�` instead of stopping the read.
- Terminal control characters in file output are escaped before printing.
- No third-party Rust crates are used.

## Notes

The available files depend on your Linux distribution and logging setup. Some files may require permissions your account does not have. ForgeHelpYou does not run `sudo`, change permissions, edit or delete logs, or send data over the network.

Logs may contain usernames, IP addresses, hostnames, or secrets. Check the output before sharing it. Only inspect systems you own or are allowed to administer.

## Build and test

Requires Rust 1.70 or newer.

```bash
cargo fmt --all -- --check
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
```

## License

MIT. See [LICENSE](LICENSE).
