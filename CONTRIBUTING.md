# Contributing

Thanks for considering a contribution to ForgeHelpYou.

## Before opening a pull request

- Keep the tool read-only: never edit, delete, rotate, or change permissions on system logs.
- Do not add telemetry or network uploads of log contents.
- Keep command execution shell-free; pass arguments directly to `Command`.
- Treat log contents as untrusted input. Escape terminal control characters before displaying them.
- Add tests for new parsing, filtering, and error-handling behavior.

## Local checks

```bash
cargo fmt --all
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
```

Please describe the user problem, the behavior changed, and the tests you ran.
