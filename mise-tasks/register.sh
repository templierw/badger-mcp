#!/usr/bin/env bash
#MISE description="Point Claude Code at the freshly built release binary"
#MISE depends=["release"]
set -euo pipefail

exe="$(cygpath -w "$PWD/target/release/badger-rmcp.exe")"
claude mcp remove badger-mcp -s local 2>/dev/null || true
claude mcp add badger-mcp -- "$exe"
claude mcp get badger-mcp