//! NeetCode 150 — Valid Anagram (Arrays & Hashing, Easy)
//!
//! The executable code uses the requested byte-frequency table: increment every
//! byte seen in `s`, decrement every byte seen in `t`, and accept exactly when
//! all 256 counters return to zero.

use creusot_std::prelude::*;

/// Number of occurrences of byte `x` in the prefix `seq[0..upto)`.
///
/// The recursive shape mirrors the executable prefix loops: the last element of
/// the current prefix either contributes one occurrence of `x` or contributes
/// zero, then the proof recurses on the shorter prefix.
// `open` exposes the definition so loop proofs can unfold one prefix step.
#[logic(open)]
// `upto` decreases by one on each recursive call, proving termination.
#[variant(upto)]
pub fn occ(seq: Seq<u8>, x: u8, upto: Int) -> Int {
    pearlite! {
        if upto <= 0 {
            0
        } else if seq[upto - 1] == x {
            occ(seq, x, upto - 1) + 1
        } else {
            occ(seq, x, upto - 1)
        }
    }
}

/// Returns `true` iff `t` is a byte-for-byte anagram of `s`.
///
/// The length bound is solely an arithmetic-safety precondition: it guarantees
/// the signed `i64` counters cannot overflow while accumulating prefix counts.
// Each vector is short enough that every counter value stays inside `i64`.
#[requires(s@.len() <= i64::MAX@ / 4)]
// Same overflow guard for the decrementing pass over `t`.
#[requires(t@.len() <= i64::MAX@ / 4)]
// The postcondition is the mathematical anagram definition over bytes: equal
// lengths and equal occurrence counts for every possible `u8` value.
#[ensures(result == (s@.len() == t@.len() &&
    forall<x: u8> occ(s@, x, s@.len()) == occ(t@, x, t@.len())))]
pub fn is_anagram(s: &Vec<u8>, t: &Vec<u8>) -> bool {
    let n = s.len();
    if n != t.len() {
        // Unequal lengths cannot satisfy the explicit length conjunct.
        return false;
    }

    let mut count = [0i64; 256];
    let mut i: usize = 0;

    // `i` is the number of bytes from `s` already accumulated.
    #[invariant(i@ <= n@)]
    // Every byte counter equals the logical occurrence count in `s[0..i)`.
    #[invariant(forall<x: u8> count@[x@]@ == occ(s@, x, i@))]
    // Counter values are non-negative and bounded by the processed prefix,
    // making the next `+ 1` safe.
    #[invariant(forall<x: u8> 0 <= count@[x@]@ && count@[x@]@ <= i@)]
    while i < n {
        let b = s[i];
        let idx = b as usize;
        // Bridge the program index `idx` to the specification index `b@`.
        proof_assert!(idx@ == b@);
        // Instantiate the counter bound for the byte being incremented.
        proof_assert!(count@[b@]@ <= i@);
        // The loop guard makes `i` a valid sequence index for unfolding `occ`.
        proof_assert!(i@ < s@.len());
        // Restate the precondition in this VC before the overflow check.
        proof_assert!(s@.len() <= i64::MAX@ / 4);
        // The selected counter is bounded by the processed prefix, so adding
        // one cannot overflow under the function's length precondition.
        proof_assert!(count@[b@]@ + 1 <= i64::MAX@);
        count[idx] += 1;
        i += 1;
        // Unfold `occ` at the new prefix length: the just-read byte contributes
        // exactly one to its own counter and zero to every other byte.
        proof_assert!(s@[i@ - 1] == b);
        proof_assert!(forall<x: u8> count@[x@]@ == occ(s@, x, i@));
    }

    let mut j: usize = 0;
    // `j` is the number of bytes from `t` already subtracted.
    #[invariant(j@ <= n@)]
    // After the first loop, `i == n`, so each counter starts at the full count
    // of `s`; after `j` decrements it additionally subtracts `t[0..j)`.
    #[invariant(forall<x: u8> count@[x@]@ == occ(s@, x, s@.len()) - occ(t@, x, j@))]
    // Counters stay within a signed range bounded by the input lengths, making
    // the next `- 1` safe.
    #[invariant(forall<x: u8> -j@ <= count@[x@]@ && count@[x@]@ <= s@.len())]
    while j < n {
        let b = t[j];
        let idx = b as usize;
        // Bridge the program index `idx` to the specification index `b@`.
        proof_assert!(idx@ == b@);
        // Instantiate the lower counter bound for the byte being decremented.
        proof_assert!(-j@ <= count@[b@]@);
        // The loop guard makes `j` a valid sequence index for unfolding `occ`.
        proof_assert!(j@ < t@.len());
        // Restate the precondition in this VC before the underflow check.
        proof_assert!(t@.len() <= i64::MAX@ / 4);
        // The selected counter is above `i64::MIN`, so subtracting one is safe.
        proof_assert!(i64::MIN@ <= count@[b@]@ - 1);
        count[idx] -= 1;
        j += 1;
        // Unfold `occ` at the new prefix length of `t`: the just-read byte
        // accounts for exactly the one counter that changed.
        proof_assert!(t@[j@ - 1] == b);
        proof_assert!(forall<x: u8> count@[x@]@ == occ(s@, x, s@.len()) - occ(t@, x, j@));
    }

    let mut k: usize = 0;
    // `k` is the next counter position to inspect.
    #[invariant(k@ <= 256)]
    // All counters before `k` have been checked and are zero.
    #[invariant(forall<b: u8> b@ < k@ ==> count@[b@]@ == 0)]
    // The frequency-difference invariant from the two counting loops remains
    // true throughout this read-only scan.
    #[invariant(forall<x: u8> count@[x@]@ == occ(s@, x, s@.len()) - occ(t@, x, t@.len()))]
    while k < 256 {
        if count[k] != 0 {
            let b = k as u8;
            let _ = b;
            // Because `k < 256`, casting to `u8` preserves the mathematical
            // value, letting the non-zero program counter witness the failed
            // occurrence-equality postcondition.
            proof_assert!(b@ == k@);
            proof_assert!(count@[b@]@ != 0);
            proof_assert!(occ(s@, b, s@.len()) != occ(t@, b, t@.len()));
            return false;
        }
        k += 1;
    }

    // Every `u8` value is one of the 256 checked counter positions.
    proof_assert!(forall<x: u8> x@ < 256);
    // Combine that byte-domain fact with the check-loop invariant.
    proof_assert!(forall<x: u8> count@[x@]@ == 0);
    // Substitute the zero counters into the frequency-difference invariant.
    proof_assert!(forall<x: u8> occ(s@, x, s@.len()) == occ(t@, x, t@.len()));
    true
}

#[cfg(test)]
mod tests {
    use super::is_anagram;

    #[test]
    fn standard_examples() {
        assert!(is_anagram(&b"anagram".to_vec(), &b"nagaram".to_vec()));
        assert!(!is_anagram(&b"rat".to_vec(), &b"car".to_vec()));
        assert!(!is_anagram(&b"a".to_vec(), &b"ab".to_vec()));
        assert!(is_anagram(&b"".to_vec(), &b"".to_vec()));
    }

    #[test]
    fn byte_level_cases() {
        assert!(is_anagram(&vec![0, 255, 1, 255], &vec![255, 1, 255, 0]));
        assert!(!is_anagram(&vec![0, 255, 1], &vec![255, 1, 1]));
    }
}
