#!/usr/bin/env python3
"""Parse Creusot verif/**/proof.json files and report pass/fail per goal.

Usage: proofstat.py [PROJECT_ROOT]
A goal is "proved" iff every leaf in its proof tree was discharged by a prover
(no null / UNPROVEN leaves). Exits 0 if all goals pass, 1 otherwise.
"""
import json
import os
import sys
from pathlib import Path


def walk(node):
    """Yield 'OK' / 'UNPROVEN' for each leaf of a why3find proof tree."""
    if node is None:
        yield "UNPROVEN"
    elif isinstance(node, dict):
        if "prover" in node:
            yield "OK"
        elif "children" in node:
            for child in node["children"]:
                yield from walk(child)
        else:
            # Unknown leaf shape -> treat as unproven to be safe.
            yield "UNPROVEN"


def main():
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()
    verif = root / "verif"
    if not verif.is_dir():
        print(f"NO verif/ dir under {root}")
        return 2
    total_goals = 0
    failed_goals = 0
    crates = {}
    for pj in sorted(verif.rglob("proof.json")):
        data = json.load(open(pj))
        coma = data.get("proofs", {}).get("Coma", {})
        # crate dir name e.g. p001_contains_duplicate_rlib
        rel = pj.relative_to(verif)
        crate = rel.parts[0]
        for goal, tree in coma.items():
            total_goals += 1
            leaves = list(walk(tree))
            ok = leaves and all(l == "OK" for l in leaves)
            crates.setdefault(crate, []).append((goal, ok, leaves.count("UNPROVEN"), len(leaves)))
            if not ok:
                failed_goals += 1
    for crate in sorted(crates):
        goals = crates[crate]
        nbad = sum(1 for _, ok, _, _ in goals if not ok)
        mark = "PASS" if nbad == 0 else f"FAIL({nbad})"
        print(f"[{mark}] {crate}: {len(goals)} goals")
        for goal, ok, nun, ntot in goals:
            if not ok:
                print(f"    UNPROVEN {goal}: {nun}/{ntot} leaves unproven")
    print(f"\nTOTAL: {total_goals} goals, {failed_goals} unproven, "
          f"{len(crates)} crates")
    return 0 if failed_goals == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
