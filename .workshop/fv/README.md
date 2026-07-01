# Formal Verification Workshop (fv)

The formal verification workshop provides a containerized environment with [Creusot](https://github.com/creusot-rs/creusot) for formally verifying the codebase.

## Setup

Launch the workshop:

```shell
workshop launch fv --verbose
```

## Running Formal Verification

Run formal verification on the `drivers` crate:

```shell
workshop run fv prove
```

## How It Works

- The Workshop container installs the Creusot snap and sets `CREUSOT_RUSTC` for `cargo creusot`.
- The `prove` action cleans the Creusot build cache, then runs `cargo creusot` which:
  1. Compiles the Rust code to `.coma` proof obligations
  2. Runs SMT solvers to discharge the proof obligations

- Solver configuration (timeouts, provers, tactics) is defined in `drivers/why3find.json`.
