#!/usr/bin/env bash
# new_crate.sh <crate_name> -- scaffold a NeetCode problem crate in /project/crates
# Creates Cargo.toml + empty src/lib.rs. Does NOT overwrite an existing crate.
set -euo pipefail
NAME="${1:?usage: new_crate.sh <crate_name>}"
DIR="/project/crates/$NAME"
if [ -e "$DIR" ]; then echo "exists: $DIR"; exit 0; fi
mkdir -p "$DIR/src"
cat > "$DIR/Cargo.toml" <<EOF
[package]
name = "$NAME"
version = "0.1.0"
edition = "2024"

[dependencies]
creusot-std = { workspace = true }

[lints]
workspace = true
EOF
# Valid placeholder so the workspace always parses while the crate is in progress.
printf '//! %s (scaffolding)\n' "$NAME" > "$DIR/src/lib.rs"
echo "created $DIR"
