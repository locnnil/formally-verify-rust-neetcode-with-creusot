//! NeetCode 150 — Contains Duplicate (Arrays & Hashing, Easy)
//!
//! Problem: given an integer array `nums`, return `true` if any value appears
//! at least twice, and `false` if every element is distinct.
//!
//! We implement the brute-force O(n^2) comparison of all index pairs. This
//! version is chosen because its contract can be *fully* and *honestly*
//! verified by Creusot: the postcondition states exactly "there exist two
//! distinct positions holding equal values", which is the precise meaning of
//! the problem. The proof is therefore non-trivial — the spec is the problem
//! statement, not a tautology.

use creusot_std::prelude::*;

/// Returns `true` iff `nums` contains a duplicated value.
///
/// The postcondition is the mathematical definition of "contains a duplicate":
/// there exist two positions `i < j` (both in bounds) whose values are equal.
#[ensures(result == (exists<i: Int, j: Int>
    0 <= i && i < j && j < nums@.len() && nums@[i] == nums@[j]))]
pub fn contains_duplicate(nums: &Vec<i32>) -> bool {
    let n = nums.len();
    let mut i = 0;
    // Outer loop invariant: every pair whose smaller index is < i is distinct,
    // i.e. no duplicate hides in the already-scanned prefix of first indices.
    #[invariant(i@ <= n@)]
    #[invariant(forall<a: Int, b: Int>
        0 <= a && a < i@ && a < b && b < n@ ==> nums@[a] != nums@[b])]
    while i < n {
        let mut j = i + 1;
        // Inner loop invariant: nums[i] differs from every element strictly
        // between i and the current j.
        #[invariant(i@ < j@ && j@ <= n@)]
        #[invariant(forall<b: Int> i@ < b && b < j@ ==> nums@[i@] != nums@[b])]
        while j < n {
            if nums[i] == nums[j] {
                // Found an explicit witness pair (i, j): the existential holds.
                return true;
            }
            j += 1;
        }
        i += 1;
    }
    // Every pair was checked and found distinct: the existential is false.
    false
}

#[cfg(test)]
mod tests {
    use super::contains_duplicate;

    #[test]
    fn example1() {
        // nums = [1,2,3,1] -> true (1 appears twice)
        assert!(contains_duplicate(&vec![1, 2, 3, 1]));
    }

    #[test]
    fn example2() {
        // nums = [1,2,3,4] -> false (all distinct)
        assert!(!contains_duplicate(&vec![1, 2, 3, 4]));
    }

    #[test]
    fn example3() {
        // nums = [1,1,1,3,3,4,3,2,4,2] -> true
        assert!(contains_duplicate(&vec![1, 1, 1, 3, 3, 4, 3, 2, 4, 2]));
    }

    #[test]
    fn empty_and_single() {
        assert!(!contains_duplicate(&vec![]));
        assert!(!contains_duplicate(&vec![7]));
    }
}
