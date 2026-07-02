#!/usr/bin/env bash
# verify_one.sh <crate_name>
# Robustly verify ONE finished crate in its ISOLATED /tmp project: run the
# Creusot proof (prove_one.sh) AND the crate's cargo tests there, so results are
# immune to other agents' in-progress (empty) crates breaking workspace parsing.
set -uo pipefail
NAME="${1:?usage: verify_one.sh <crate_name>}"
echo "######## verify $NAME ########"
bash /project/.workshop/prove_one.sh "$NAME" 2>&1 | grep -E "^\[(PASS|FAIL)|TOTAL|UNPROVEN|error\[|error:|panicked" | head -8
echo "--- cargo test (isolated) ---"
( cd "/tmp/prove/$NAME" && cargo test 2>&1 | grep -E "test result|error\[|error:" | head -3 )
