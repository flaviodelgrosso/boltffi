#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "Installing riff CLI..."
cargo install --path "$PROJECT_ROOT/riff_cli" --force

echo ""
echo "Done. Verify with:"
echo "  riff --version"
