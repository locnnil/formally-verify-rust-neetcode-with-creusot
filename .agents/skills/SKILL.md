---
name: formal-verification-creusot
description: Guide for using Creusot to formally verify a Rust codebase. Use this when asked to verify code, fix proofs, or work with Creusot verification.
---

# Formal Verification with Creusot

Use the `creusot-mcp` MCP server for all verification tasks. It provides six tools described below.

## Creusot References

- **Guide:** https://guide.creusot.rs/
- **Source & stdlib:** https://github.com/creusot-rs/creusot — search under `creusot-std/src/` for contracts on standard library types (e.g., `Seq`, `Vec`, slice methods like `fill`, `iter`, `get`). When you need to know what a standard library method ensures, search the Creusot repo for its contract.
- **Tests:** Creusot tests are a good source of reference when something isn't documented in the guide.

## Tool Reference

### `start_verification`
Kicks off the proof process in the background. Call this first.

### `set_timeout(timeout)`
Sets the per-prover time limit in `why3find.json`. Modifies the file on disk so that the cache-clearing mechanism detects the change and invalidates stale proof results on the next `start_verification` call. Use this to experiment with different timeout values without manually editing config files.

### `check_verification`
Waits (up to 120s) for the verification to finish and returns the result. The output lists each goal with a pass/fail status, e.g.:
```
Goal Coma.vc_set: ✘ (20/24)
```
This means the `set` function has 20 of 24 subgoals proven and 4 failing.

### `list_subgoals(goal)`
Shows the proof tree for a specific goal, listing every subgoal with its pass/fail status and prover info. Passing subgoals that took more than 60% of the configured time limit are flagged with `⚠ NEAR TIMEOUT`. Use this **before** requesting context to understand which subgoals failed and get their indices.

### `check_goal_context(goal, subgoal_index=0, full=False)`
Returns the Why3 verification context for a **single** failing subgoal. By default uses **compact mode**, which strips boilerplate (type definitions, standard-library axioms, invariant axioms) and keeps only definitions, hypotheses, asserts, constants, and the goal. Pass `full=True` if you need the unfiltered context. Use `list_subgoals` first to find the index of the subgoal you want to inspect.

### `check_goal_file(goal)`
Returns the `.coma` intermediate representation file for a goal. This shows the full translated function body. Rarely needed — prefer `check_goal_context` for debugging failures.

## Proof Methodology

Proving a function correct has three phases:

### 1. Specify — agree on the contract with the user
Write `#[requires]` and `#[ensures]` for the function, then **you MUST present them to the user for review and get explicit approval before moving to phase 2**. Do NOT proceed to proving until the user confirms the specification. Present the proposed contracts and ask the user whether they agree or want changes. The specification is the statement of what "correct" means — if it's wrong, the proof is wasted work. Iterate until the user explicitly approves. Pay attention to edge cases, overflow bounds, and whether the spec is strong enough for callers.

**This is a hard gate:** no amount of proving effort matters if the spec is wrong. Even if the contracts seem obvious, present them — the user may have domain knowledge that changes what "correct" means.

### 2. Prove — make the prover succeed
Use `proof_assert!` liberally to find where reasoning breaks. Add loop invariants, call lemmas, and bridge program/spec terms (see sections below). Use the tool workflow (start → check → list_subgoals → context → fix → re-run) to iterate.

### 3. Optimize — clean up the proof
Once all subgoals pass, remove `proof_assert!` statements that are no longer needed (the prover may have only needed them to guide you, not itself). Extract remaining assertions into lemmas for modularity and reuse. Check for near-timeouts with `list_subgoals` and stabilize fragile proofs. The goal is a clean, fast, maintainable proof.

## Tool Workflow

1. **`start_verification`** → starts the proof.
2. **`check_verification`** → wait for results, see which goals fail.
3. **`list_subgoals(goal)`** → inspect the proof tree of a failing goal to see exactly which subgoals failed.
4. **`check_goal_context(goal, subgoal_index)`** → read the failing subgoal's context one at a time. Understand what the prover knows (axioms/hypotheses) and what it's trying to prove (the goal at the bottom).
5. **Fix the code** → add/adjust contracts, lemmas, or proof hints.
6. **Re-run** → `start_verification` + `check_verification` to confirm the fix.

## Debugging Tips

- **Read the context carefully.** The goal at the bottom of `check_goal_context` output is what the prover must prove. The axioms above it are what it knows. If something you expect is missing from the axioms, the prover doesn't have access to it.
- **The problem may not be in the failing function.** It could be in a dependency — a lemma with a missing `requires`, or a helper with an incomplete `ensures`. Trace the dependency chain and check each contract.
- **`#[trusted]` functions are risky.** Their contracts are assumed correct without proof. If a trusted function's contract is wrong, everything that depends on it may silently have an incorrect proof or fail to verify.
- **Look for similar passing subgoals.** If `split_vc[3]/split_vc[0]` fails but `split_vc[3]/split_vc[1]` passes, comparing their contexts reveals what differs — often a missing lemma or a different case branch.
- **Logic function visibility: `#[logic(open)]` vs `#[logic(opaque)]`.** Use `#[logic(open)]` when callers need to see the function body (the prover can unfold it). Use `#[logic(opaque)]` when callers should only see `requires`/`ensures`, not the implementation. **Do not use bare `#[logic]` with a separate `#[open]` attribute** — the visibility modifier goes inside the `logic(...)` parentheses. If verification fails because the prover can't reason about a logic function's internals, check if it's opaque and whether its ensures are strong enough.
- **Isolating branches with magic.** In complex methods with many branches, the VC can be huge and branches pollute each other's context. To isolate a single branch: create a `#[trusted]` ghost function that `#[ensures(false)]` (a "magic" call), and call it in every branch *except* the one you're trying to prove. This gives the prover a contradiction in the other branches, effectively suppressing them. Once the target branch proves, remove the magic calls and move to the next branch. **This only works in program functions** — in `#[logic]` functions, calling magic in any branch introduces a contradiction into the entire proof context, making everything trivially provable. **Important: magic calls are strictly a development aid. Always remove all magic calls and the magic function itself before considering a proof complete** — a proof that passes with magic is unsound.

## Proof Engineering

### Mutable References: `*x` and `^x` (No `old()`)

Creusot does **not** have an `old()` operator. Instead, mutable references (`&mut T`) use a **two-state model** with prophecy variables:

- **`*x`** — the **current** (initial) value of the reference at function entry.
- **`^x`** — the **final** value of the reference when the borrow expires (i.e., when the function returns).

In `#[requires]`, `*x` refers to the value at call time. In `#[ensures]`, `*x` still refers to the initial value and `^x` refers to the final value. There is no need for `old()` — the initial value is always accessible as `*x`.

**Example — swap:**
```rust
#[ensures(^a == *b)]
#[ensures(^b == *a)]
pub fn swap(a: &mut u64, b: &mut u64) { ... }
```

**Common mistake:** Do not write `old(*x)`, `old(x)`, or any variant — `old` does not exist in Creusot and will cause a compilation error. Use `*x` for the initial value and `^x` for the final value.

**In loop invariants:** `old()` is also not available. To refer to a value from before the loop, capture it with `snapshot!` (or `ghost!` for non-`Copy` types) so it is erased from the compiled code:
```rust
let original_len = snapshot! { v@.len() };
#[invariant(v@.len() == *original_len)]
while ... { ... }
```

### `snapshot!` and `ghost!` — Erased Proof-Only Values

Both `snapshot!` and `ghost!` produce values that are **erased at compilation** — they exist only for verification and generate no runtime code. Use them instead of regular `let` bindings whenever a value is only needed for specifications or proof hints.

**`snapshot!`** creates a `Snapshot<T>` — a **zero-sized, `Copy`** wrapper around a logical value. Use it to capture values for use in invariants and `proof_assert!`. Inside the macro you write pearlite (spec-level) expressions. Dereference with `*`:
```rust
let old_v = snapshot! { v@ };          // capture the logical view of v
let old_len = snapshot! { v@.len() };  // capture a computed spec value
proof_assert!(*old_v == v@);           // use with *
```

**`ghost!`** creates a `Ghost<T>` — a **proof-only heap-allocated** value that supports ownership and mutation. Use it when you need mutable ghost state, or to call `#[check(ghost)]` lemmas. Access with `*` inside `ghost!` blocks:
```rust
ghost! { my_lemma(arg1, arg2) };       // call a ghost lemma
let mut g = ghost!(0u64);              // mutable ghost variable
ghost! { *g = 42 };                    // mutate in ghost context
```

**Quick decision guide:**
| Need | Use |
|---|---|
| Capture a value for loop invariants | `snapshot!` |
| Call a `#[check(ghost)]` lemma | `ghost!` |
| Call a `#[logic(opaque)]` lemma | `snapshot!` |
| Mutable ghost state (e.g., ghost counter) | `ghost!` |
| Bridge proof_assert with a spec expression | `snapshot!` |
| Anything that must be `Copy` | `snapshot!` (always `Copy`) |

### Logic vs Non-Logic Lemmas

Lemmas come in two kinds with very different behavior:

- **Logic lemmas** (`#[logic(opaque)]`): Call with `snapshot!(f(args))`. This introduces the universal axiom (e.g., `forall<x: Int> x > 0 ==> x > -1`) but does **not** instantiate it for specific argument values. The prover must find the instantiation itself via e-matching, which may or may not happen. The arguments in `snapshot!` are irrelevant for instantiation — they just ensure the axiom is in scope.
- **Non-logic lemmas** (`#[check(ghost)]`): Call with `ghost! { f(args) }`. This **does** instantiate the precondition/postcondition pair for the given arguments, giving the prover the concrete fact directly. Use this when you need a specific instance.

If your logic lemma call via `snapshot!` isn't helping, switch to a `#[check(ghost)]` lemma and call it with `ghost! { f(args) }`.

### Bridging Program and Spec Expressions

Program code and spec code can produce different Why3 terms for semantically identical operations:
- `index >> 3` (program) vs `index / 8` (spec) — same value, different AST
- `1u8 << (bit_index as u8)` (program) vs `1u8 << j` (spec) — the cast inserts an extra `of_int(t'int2(...))` wrapper

Add **normalization proof_asserts** to bridge the gap:
```rust
proof_assert!(byte_index == index / 8);
proof_assert!(mask == 1u8 << bit_index);
```
These tell the prover the program-computed value equals the spec-level expression.

### Using proof_assert for Debugging, Lemmas for Modularity

`proof_assert!` is useful for **finding** where a proof breaks — add assertions at intermediate steps to narrow down which reasoning the prover is missing. However, each assertion adds axioms for the solver to consider, which can **increase** solve time (sometimes dramatically — we've seen proof_asserts turn a 13s proof into 37s).

Once you understand the proof structure from your assertions:
1. **Extract** the derived/quantified facts into a lemma with appropriate `#[ensures]`
2. **Remove** those proof_asserts from the function body
3. **Call** the lemma instead
4. **Keep** normalization proof_asserts that bridge program variables to spec terms (see below)

This gives you modularity and stability. The lemma proves its own ensures once, and callers get the conclusion cheaply. However, note that proof speed may not always improve — the caller still needs to combine the lemma's ensures with local context, and if that reasoning is inherently complex, the solve time stays similar.

### Extracting proof_asserts into Lemmas: Constant Symbol Pitfalls

When moving `proof_assert!` statements into a `#[check(ghost)]` lemma, beware of **cross-function constant symbol mismatch**. Each function that mentions `u8::BITS` (or similar constants) gets its own Why3 constant (e.g., `const_BITS'0`, `const_BITS'1`). Even though both represent `8`, SMT quantifier instantiation can fail to connect terms using different symbols.

**Rules for safe extraction:**
- **Lemma ensures should reference `index` directly**, not program variables like `byte_index`. For example, write `index >> u8::BITS.trailing_zeros_logic()` instead of `byte_index`. This way the lemma's ensures produce self-consistent terms.
- **Keep program-variable bridges as proof_asserts in the caller.** Assertions like `proof_assert!(byte_index == index >> u8::BITS.trailing_zeros_logic())` must stay in the function body because they produce terms using the caller's own constants, which match the terms from `get_word_index`'s ensures.
- **Use `Self::MASK_BIT_INDEX`** (or equivalent named constants) in method lemmas rather than `(u8::BITS as usize) - 1usize` to match the terms the program code produces.

Example — good extraction pattern:
```rust
// In the lemma (impl method, #[check(ghost)]):
#[ensures(forall<i: usize> 0usize <= i && i < self.logic_len() && i@ != index@ &&
    (i >> u8::BITS.trailing_zeros_logic()) == (index >> u8::BITS.trailing_zeros_logic()) ==>
    (i & Self::MASK_BIT_INDEX)@ != (index & Self::MASK_BIT_INDEX)@)]
pub(crate) fn set_normalization_lemma(&self, index: usize) { ... }

// In the caller — keep these bridges:
ghost! { self.set_normalization_lemma(index) };
proof_assert!(byte_index == index >> u8::BITS.trailing_zeros_logic());
proof_assert!(bit_index == index & Self::MASK_BIT_INDEX);
```

### Calling Logic Lemmas from Ghost Functions

A `#[check(ghost)]` function cannot directly call a `#[logic(opaque)]` function in its body (error: *"called logic function in program context"*). Use `snapshot!` instead:
```rust
snapshot!(self.bounds_lemma(index));
```
This works in both program and ghost contexts. The pattern `proof_assert!({ self.bounds_lemma(index); true })` also works but is less clean.

### Split `#[ensures]` for Complex Lemmas

A lemma with two separate `#[ensures]` clauses (one per case) proves much faster than a single combined one, because the prover tackles each independently. If a lemma's ensures covers multiple cases (e.g., "different byte" vs "same byte, different bit"), split them.

### Bitvector Arithmetic

- **BV arithmetic wraps.** Properties that seem obvious for mathematical integers (like `x >= N*8 ==> x>>3 >= N`) can be false in bitvector arithmetic if `N*8` overflows. Add explicit overflow guards in `#[requires]`.
- **`#[bitwise_proof]` is required for BV reasoning.** Without it, the prover cannot reason about shifts, masks, and bitwise ops. It triggers the `compute_specified` tactic.

### The Int / usize (BV64) Gap

Creusot's `Seq<T>` uses `Int` indices, but bitvector operations (`>>`, `&`, `<` on `usize`) produce BV64 (`t`) terms. SMT solvers **cannot automatically bridge** between these two worlds:

- `forall j:bv64. P(to_int(j))` does **not** imply `forall i:int. P(i)` — the solver can't synthesize `of_int(i)` to instantiate the BV-quantified formula. E-matching has no trigger to fire.
- Conversely, `forall i:int. P(i)` *can* instantiate for `to_int(j)` if the goal contains `P(to_int(j))` — Int→BV is usually easier than BV→Int.

This matters whenever a spec uses `Seq` indexing (`self.0@[i]` with `i: Int`) but the proof involves BV-computed indices (e.g., `index >> 3`).

**Pattern: Bridging proof_assert for Int→BV Seq access.** When you have an Int-quantified hypothesis like `∀i:Int. 0<=i<len → byte[i]==0` and need to use it with a BV-computed index `widx: usize`, add:
```rust
proof_assert!(
    (forall<i: Int> 0 <= i && i < self.0@.len() ==> self.0@[i] == 0u8) ==>
    forall<j: usize> j < BYTES ==> self.0@[j@] == 0u8
);
```
The `j@` converts BV to Int, giving the prover an explicit Seq access term `self.0@[j@]` that e-matching can connect to the Int-quantified hypothesis.

**Pattern: While loop as BV→Int bridge.** When you've proved `∀j:usize. j < N ==> P(j@)` but need `∀i:Int. 0<=i<N@ ==> P(i)`, use a while loop to enumerate indices:
```rust
let mut idx: usize = 0usize;
#[invariant(idx@ <= N@)]
#[invariant(
    (forall<j: usize> j < N ==> P(j@)) ==>
    forall<k: Int> 0 <= k && k < idx@ ==> P(k)
)]
#[variant(N@ - idx@)]
while idx < N {
    idx = idx + 1usize;
}
```
At each step, the prover instantiates the usize hypothesis with the specific `idx` value (trivial e-matching), extends the Int-quantified invariant, and after the loop `idx@ == N@` gives the full conclusion. Note: `for` loops don't work in `#[check(ghost)]` + `no_std` — use `while` with `#[variant]`.

### VC Context Pollution and Proof Splitting

Each `proof_assert!`, loop invariant, and ghost call adds axioms to the VC context for **all subsequent subgoals**. This can cause previously-passing subgoals to fail — not because they're wrong, but because the extra axioms overwhelm the solver's search.

**Solution: Split complex proofs into separate `#[check(ghost)]` functions.** Each function gets its own isolated VC context. For example, if proving an iff (A ⟺ B):
```rust
#[check(ghost)]
#[ensures(A ==> B)]
fn forward(&self) { /* clean context, only forward-direction hints */ }

#[check(ghost)]
#[ensures(B ==> A)]
fn backward(&self) { /* loop + proof_asserts without polluting forward */ }

#[check(ghost)]
#[ensures(A == B)]
pub(crate) fn lemma(&self) {
    ghost! { self.forward() };
    ghost! { self.backward() };
}
```
This is especially important when one direction needs heavy machinery (loops, many proof_asserts) that would pollute the other direction's context.

### Const Generics

- **Const generics become uninterpreted constants.** The prover doesn't know Rust's type system prevents overflow. You may need explicit `#[requires(...)]` to bound them (e.g., `BYTES@ * u8::BITS@ <= usize::MAX@`).
- **Array length ≠ const generic automatically.** For `[u8; BYTES]`, the prover does NOT automatically know `self.0@.len() == BYTES@`. You may need an explicit `#[requires]` or to invoke the array invariant.

### Opaque Lemmas and E-Matching

A `#[logic(opaque)]` lemma's axioms (its `#[ensures]` clauses) are universally quantified and present in the Why3 theory. However, they only fire via **e-matching** — the prover must find existing terms in the VC that match the axiom's triggers. In practice:

- **Don't assume opaque lemmas fire on their own.** Even though the axiom is technically in scope, if the VC doesn't contain the right trigger terms, the prover will never instantiate it. Always call opaque lemmas with `snapshot!` unless you've verified the proof works without the call.
- **Some opaque lemmas do fire without a call** — particularly those whose `#[requires]` patterns match terms naturally produced by the function being proved (e.g., array accesses, shift operations). These can be marked `#[allow(dead_code)]` and left uncalled. But this is fragile and depends on the specific terms in the VC.
- **Removing an "unused" opaque lemma can break proofs** that silently relied on its axioms via e-matching. Before removing one, verify it doesn't regress other proofs.
- **Opaque lemmas may hurt performance** — in theory, the prover can instantiate them in irrelevant contexts, expanding the search space. This hasn't been empirically confirmed in this codebase, but is worth considering if you observe unexplained slowdowns after adding opaque lemmas.

**Prefer `#[check(ghost)]` lemmas over `#[logic(opaque)]` when possible.** Ghost lemmas are only active where explicitly called via `ghost! { f(args) }`, giving you precise control over when their facts enter the proof context. Only use `#[logic(opaque)]` when you specifically need the axiom available for e-matching across multiple call sites.

### Triggers

SMT solvers instantiate universally quantified formulas via **e-matching**: a quantified axiom only fires when the VC contains a term that matches one of its **triggers**. If no trigger matches, the axiom is dead — even if it's logically relevant. The solver's auto-selected triggers are often suboptimal, leading to either missed instantiations (proof failure) or overly broad matching (proof slowdown). **Always specify triggers explicitly** on quantified `#[ensures]` clauses, `proof_assert!` quantifiers, and logic function postconditions.

#### Syntax

Use `#[trigger(expr)]` inside a quantified `#[ensures]` clause. The trigger expression must mention the bound variables:

```rust
#[logic(opaque)]
#[ensures(forall<i: usize>
    #[trigger(bit_at_byte(byte, i))]
    i < u8::BITS as usize ==> bit_at_byte(byte | mask, i) == (bit_at_byte(byte, i) || bit_at_byte(mask, i))
)]
fn or_bit_decomposition(byte: u8, mask: u8) -> bool { ... }
```

For logic functions, the outer universal quantifier (over the function's parameters) is automatically triggered on calls to the function itself. The `#[trigger]` annotation controls **inner** quantifiers in the `#[ensures]`.

You can also use triggers in `proof_assert!`:
```rust
proof_assert!(forall<j: usize>
    #[trigger(self.0@[j@])]
    j < BYTES ==> self.0@[j@] == snap[j@]
);
```

#### Choosing Good Triggers

- **Pick terms that naturally appear in the VC.** If the goal involves `bit_at_byte(x, k)`, use `#[trigger(bit_at_byte(byte, i))]` — the solver will instantiate when it sees matching `bit_at_byte` terms.
- **Avoid overly broad triggers.** A trigger like `#[trigger(i@)]` would match every `usize`-to-`Int` conversion in the VC, causing explosion. Prefer compound terms like `#[trigger(self.0@[j@])]` that are specific to the context.
- **The trigger must contain ALL bound variables.** A trigger `#[trigger(f(x))]` on `forall<x, y>` is invalid — `y` is not mentioned.

#### The Trigger Marker Trick (Manual Quantifier Instantiation)

When you need a universally quantified lemma to fire only in specific locations, but the natural trigger terms appear everywhere (causing slowdowns), use an **abstract trigger marker**:

1. **Define an opaque logic function that always returns `true`:**
```rust
#[logic(opaque)]
#[ensures(result == true)]
pub fn my_trigger_marker(x: usize, y: usize) -> bool {
    pearlite! { true }
}
```
The `#[ensures(result == true)]` is critical — without it the prover cannot discharge assertions that use the marker, since the body is opaque.

2. **Use the marker as the trigger on your lemma:**
```rust
#[logic(opaque)]
#[ensures(forall<i: usize>
    #[trigger(my_trigger_marker(x, i))]
    i != x ==> some_property(x, i)
)]
fn my_lemma(x: usize) -> bool { ... }
```

3. **Spawn the marker only where you need the lemma to fire:**
```rust
snapshot!(my_lemma(byte_index));  // bring the axiom into scope
proof_assert!(forall<i: usize> my_trigger_marker(byte_index, i));  // spawn trigger terms
```

The `proof_assert!` is trivially true (since the ensures says it's always `true`), but it introduces `my_trigger_marker(byte_index, i)` terms into the VC. These match the lemma's trigger, causing instantiation **only** at that point — not in unrelated subgoals.

This gives you the precision of `ghost!` lemma calls (fire where you want) with the power of universally quantified axioms (the prover gets `forall<i> ...` rather than a single instance).

### Recursion Limit

Heavy use of `#[requires]` attributes can hit Rust's default `recursion_limit = 128`. Add `#![recursion_limit = "256"]` at the crate root if you get recursion limit errors during Creusot translation.
