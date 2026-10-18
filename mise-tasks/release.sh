#!/usr/bin/env bash
#MISE description="optimized build, around the Windows exe lock"
#MISE sources=["src/**/*.rs", "Cargo.toml", "Cargo.lock"]
#MISE outputs=["target/release/badger-rmcp.exe"]
set -euo pipefail

if tasklist //FI "IMAGENAME eq badger-rmcp.exe" | grep -q badger-rmcp; then
  mv -f target/release/badger-rmcp.exe target/release/badger-rmcp.exe.old 2>/dev/null || true
fi
cargo build --release