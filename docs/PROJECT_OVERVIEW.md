# ForgeHelpYou 0.1.0 — project overview

## Goal

Make common Linux log-reading tasks approachable through a small CLI with clear commands and helpful errors.

## Scope of 0.1.0

- Discover a fixed list of common text-log locations.
- Show the most recent lines from a selected source.
- Search a selected text log by substring.
- Optionally read the systemd journal with `journalctl`.
- Report basic local log availability and permissions.

## Non-goals

- No log collection agent or background daemon.
- No remote log upload, telemetry, or network connection.
- No automatic log deletion, rotation, editing, permission changes, or remediation.
- No privilege escalation or automatic `sudo`.
- No promise to parse every distribution-specific log format.

## Design principles

1. Read-only by default.
2. Local processing only.
3. No shell interpolation when starting external commands.
4. Treat log text as untrusted input.
5. Keep the command set small and explain errors in plain language.
6. Prefer the standard library unless an additional dependency has a clear benefit.

## Future ideas

- Structured parsers for selected common log formats.
- Time-range filters for file-based logs.
- Export of redacted reports with explicit user action.
- Optional live-follow mode with clear stop controls.
- More distribution-specific source discovery.
