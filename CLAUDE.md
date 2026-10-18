# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

An MCP server in Rust that exposes the local [Super Productivity](https://super-productivity.com/) (SP) desktop app's tasks as a tool. It speaks JSON-RPC over stdio through the `rmcp` SDK and reads SP's local REST API at `http://127.0.0.1:3876`.

## Commands

mise owns the workflow; `mise tasks` lists them.

- `mise run check` — tests + clippy. This is exactly what CI runs.
- `mise run test`, `mise run lint` — individually.
- `cargo test <substring>` or `mise run test -- <substring>` — a single test.
- `mise run pipe` — builds the debug binary and drives a real `initialize` +
  `tools/call` through it, printing the rendered task text. This is how the I/O
  layers get verified; there are no mocks.
- `mise run release`, `mise run register` — optimized build, then re-point the
  Claude Code registration at it.

`SPROD_TOKEN` comes from `.env.local` (gitignored) via mise's `[env]`, not from the machine environment. SP must be running for anything that talks to it: `curl --noproxy 127.0.0.1 http://127.0.0.1:3876/health`.

## Architecture

- `src/sp.rs` — the SP client (`SpClient`, async reqwest) and its serde types. Knows nothing about MCP.
- `src/mcp.rs` — `render_tasks`: tasks plus an id→name map become the single block of text the model reads.
- `src/bin/badger-rmcp.rs` — the rmcp server: one tool, the capabilities, the server identity, the stdio transport.

The dependency runs one way: `mcp` uses `sp`, never the reverse. That boundary is load-bearing — it's what let the entire protocol layer be swapped without touching a test. Anything protocol-shaped (request/response types, ids, dispatch, input schemas) belongs to rmcp; don't reimplement it. A hand-written JSON-RPC layer existed until `a3a1861` and is recoverable from git history.

Behaviours that are easy to break:

- **Two error layers.** `Err(McpError)` means the *request* was malformed. A tool that ran and failed returns `Ok(CallToolResult::error(..))` so the model can read why. SP being unreachable is the second kind, never the first.
- **Envelope parsing ignores HTTP status.** SP wraps every response in `{"ok":…, "data"|"error":…}`; `Envelope<T>` is untagged, so parse the body and let the variant decide. `SpError` carries SP's own code and message into the tool's error text.
- **Estimates are milliseconds on the wire, minutes in the output** (`/60_000`). SP sends `0` for "no estimate", which `zero_as_none` plus `#[serde(default)]` turns into `None`; it must never render as `0`.
- **Project names cost one `/projects` call per tool call.** If that call fails, the tool still answers, logging to stderr and falling back to raw ids — a degraded answer beats no answer.
- **Every rendered line carries the task id.** SP's repeating tasks have ids shaped `rpt_<recurrence>_<date>`, which is the only way to tell apart tasks sharing a title.
- **stdout is the protocol channel.** Diagnostics go to stderr; a stray `println!` corrupts the stream.

## Tests

8 tests, all pure: serde round-trips in `sp.rs`, rendering in `mcp.rs`. No network and no mocks, which is why CI needs neither SP nor a token. The HTTP and stdio layers are deliberately *not* unit-tested — their real failure modes are environmental (app closed, bad token, proxy) and a mock asserts none of them. `mise run pipe` against the live app is the check instead.

New behaviour starts as a failing test; the repo owner writes the implementation.

## Operational notes (Windows)

- **Claude Code must launch the server through mise**: `claude mcp add badger-mcp -- mise -C <root> exec -- <exe>`. Registered bare, the server gets no `SPROD_TOKEN`, panics at startup, and the client reports `CONNECTION_CLOSED`.
- **Windows locks a running `.exe`**, so a release build fails with os error 5 while a session runs the server. `mise-tasks/release.sh` renames the old binary aside when it detects a running process.
- **MCP servers load at session start**, so a rebuilt binary needs a Claude relaunch.
- **Inline mise tasks run under `cmd /c`.** Anything with shell syntax must be a file task in `mise-tasks/` with a bash shebang, or set `shell = "bash -c"`.
