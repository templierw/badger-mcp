# badger-mcp

[![check](https://github.com/templierw/badger-mcp/actions/workflows/check.yml/badge.svg)](https://github.com/templierw/badger-mcp/actions/workflows/check.yml)

An MCP server that gives an AI assistant read access to your [Super Productivity](https://super-productivity.com/) tasks. Written in Rust on the official [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk) SDK, talking to SP's local REST API.

## The tool

`list_tasks`, no arguments, one line per task:

```
check dynatrace access - 15 min - project: use cases - id: _HCT5WXfEVYR3XVUuDlE_
misc - project: =SE= - id: rpt_EAWhNKHHUrJ9HjeUeH_Wa_2026-10-18
```

Estimates are in minutes (SP stores milliseconds) and are left out entirely when unset, rather than shown as zero. Project ids are resolved to names, falling back to the raw id if that lookup fails. Each line carries the task id, so a caller can tell apart tasks that share a title — repeating tasks include their occurrence date.

## Requirements

- Super Productivity desktop with the Local REST API enabled (Settings → Misc)
- its access token
- Rust, and [mise](https://mise.jdx.dev/)

## Setup

```sh
echo 'SPROD_TOKEN=<your token>' > .env.local   # gitignored
mise install
mise run register                              # builds release, registers with Claude Code
```

Then restart Claude Code, and the tool appears as `mcp__badger-mcp__list_tasks`.

The registration launches the server through `mise exec`, so the token stays in `.env.local` rather than in your machine environment or your editor's config.

## Development

```sh
mise run check   # tests + clippy, same as CI
mise run pipe    # real handshake + tool call through the debug binary
```

The unit tests cover the serde layer and the output format, and need neither the app nor a token. The HTTP and stdio layers are checked against the running app with `mise run pipe` instead of being mocked.

## Why it exists

A learning project. The JSON-RPC layer was written by hand over stdio first, then replaced with `rmcp` once the protocol was understood — a swap that left the tool's output byte-identical.
