#!/usr/bin/env bash
#MISE description="Point Claude Code at the release binary, launched through mise"
#MISE depends=["release"]
set -euo pipefail

# Launch via `mise exec` so the server inherits SPROD_TOKEN from .env.local.
# Claude Code spawns the command bare, with none of mise's environment.
root="$(cygpath -w "$PWD")"
exe="$(cygpath -w "$PWD/target/release/badger-rmcp.exe")"

claude mcp remove badger-mcp -s local 2>/dev/null || true
claude mcp add badger-mcp -- mise -C "$root" exec -- "$exe"
claude mcp get badger-mcp
