#!/usr/bin/env bash
#MISE description="Handshake + tools/call through the debug binary"
#MISE depends=["build"]
set -euo pipefail

exe=target/debug/badger-rmcp.exe
init='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"mise-pipe","version":"0"}}}'
note='{"jsonrpc":"2.0","method":"notifications/initialized"}'
call='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_tasks","arguments":{}}}'

printf '%s\n' "$init" "$note" "$call" \
  | "$exe" \
  | jq -r 'select(.id==3) | .result.content[0].text'