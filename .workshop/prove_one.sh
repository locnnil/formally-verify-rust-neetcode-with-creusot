#!/usr/bin/env bash
# prove_one.sh <crate_name>
# Verify a SINGLE crate with Creusot in an ISOLATED temp project so that parallel
# proving agents never collide on /project's shared target/ and verif/ dirs.
#
# It copies the crate into a fresh standalone Creusot project under
# /tmp/prove/<crate>, builds with its own target dir, runs `cargo creusot`,
# and reports per-goal pass/fail via proofstat.py. The original crate in
# /project/crates is the source of truth and is NOT modified.
set -uo pipefail
NAME="${1:?usage: prove_one.sh <crate_name>}"
SRC="/project/crates/$NAME"
[ -d "$SRC" ] || { echo "no such crate: $SRC"; exit 2; }

WORK="/tmp/prove/$NAME"
mkdir -p "$WORK/src"
# Standalone (non-workspace) Cargo.toml with the exact-pinned creusot-std.
cat > "$WORK/Cargo.toml" <<EOF
[package]
name = "$NAME"
version = "0.1.0"
edition = "2024"

[dependencies]
creusot-std = "=0.12.0-dev"

[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(creusot)'] }
EOF
# Refresh sources (remove stale, copy current) but keep the build/proof caches.
rm -f "$WORK/src/"*.rs
cp "$SRC/src/"*.rs "$WORK/src/" 2>/dev/null || true
cp /project/why3find.json "$WORK/why3find.json"
# Fresh translation each run, but keep cargo build cache + why3find proof cache.
rm -rf "$WORK/target/creusot"

export CREUSOT_RUSTC=/snap/creusot/current/bin/creusot-rustc
export CARGO_TARGET_DIR="$WORK/target"
cd "$WORK"
echo "=== cargo creusot ($NAME) in $WORK ==="
timeout "${PROVE_TIMEOUT:-560}" cargo creusot -- --target x86_64-unknown-linux-gnu 2>&1 | tail -25
echo "=== proof status ==="
python3 /project/.workshop/proofstat.py "$WORK"
