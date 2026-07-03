//! NeetCode 150 — Two Sum (Arrays & Hashing, Easy)
//!
//! Return the two increasing indices whose values add to `target`.  The
//! implementation deliberately uses a brute-force search: it is simple, and the
//! nested-loop invariants can state exactly which candidate pairs have already
//! been ruled out.

use creusot_std::prelude::*;

/// Finds the unique pair of indices `i < j` such that `nums[i] + nums[j] == target`.
///
/// The preconditions below are the problem assumptions plus an overflow guard:
/// a solution exists, and every pairwise machine addition performed by the
/// algorithm stays inside the `i32` range.
#[requires(exists<i: Int, j: Int>
    // A witness pair exists, so the exhaustive search must find a valid answer.
    0 <= i && i < j && j < nums@.len() && nums@[i]@ + nums@[j]@ == target@)]
#[requires(forall<i: Int, j: Int>
    // Rust `i32` addition is checked for overflow by Creusot, so all candidate
    // pair sums are required to be representable as `i32`.
    0 <= i && i < j && j < nums@.len() ==>
        i32::MIN@ <= nums@[i]@ + nums@[j]@ && nums@[i]@ + nums@[j]@ <= i32::MAX@)]
#[ensures(
    // The result is exactly two indices, in bounds and increasing, whose values
    // satisfy the Two Sum equation.
    result@.len() == 2 &&
    0 <= result@[0]@ && result@[0]@ < result@[1]@ &&
    result@[1]@ < nums@.len() &&
    nums@[result@[0]@]@ + nums@[result@[1]@]@ == target@
)]
pub fn two_sum(nums: &Vec<i32>, target: i32) -> Vec<usize> {
    let n = nums.len();
    let mut i = 0usize;
    // All pairs whose first index is already below `i` have been checked and
    // proven not to sum to the target.
    #[invariant(i@ <= n@)]
    #[invariant(forall<a: Int, b: Int>
        0 <= a && a < i@ && a < b && b < n@ ==>
            nums@[a]@ + nums@[b]@ != target@)]
    while i < n {
        let mut j = i + 1usize;
        // For this fixed `i`, every second index below `j` has already failed.
        #[invariant(i@ < j@ && j@ <= n@)]
        #[invariant(forall<b: Int>
            i@ < b && b < j@ ==> nums@[i@]@ + nums@[b]@ != target@)]
        while j < n {
            if nums[i] + nums[j] == target {
                let mut answer = Vec::new();
                answer.push(i);
                answer.push(j);
                // These proof hints connect the two pushes to the postcondition:
                // the vector now contains exactly the discovered witness pair.
                proof_assert!(answer@.len() == 2);
                proof_assert!(answer@[0] == i);
                proof_assert!(answer@[1] == j);
                return answer;
            }
            j += 1;
        }
        i += 1;
    }
    // Unreachable under the existential precondition: the invariants say every
    // in-bounds pair has been ruled out, contradicting the required witness.
    proof_assert!(false);
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::two_sum;

    #[test]
    fn example1() {
        assert_eq!(two_sum(&vec![2, 7, 11, 15], 9), vec![0, 1]);
    }

    #[test]
    fn example2() {
        assert_eq!(two_sum(&vec![3, 2, 4], 6), vec![1, 2]);
    }

    #[test]
    fn example3() {
        assert_eq!(two_sum(&vec![3, 3], 6), vec![0, 1]);
    }

    #[test]
    fn negative_numbers_and_later_pair() {
        assert_eq!(two_sum(&vec![10, -3, 7, 4, 8], 12), vec![3, 4]);
    }
}