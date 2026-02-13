//! PMS version matching operators.
//!
//! Implements [PMS 8.3.2](https://projects.gentoo.org/pms/latest/pms.html#x1-830008.3.2):
//! given a candidate version, an operator, and a constraint version, determine
//! whether the candidate satisfies the constraint.

use std::cmp::Ordering;

use portage_atom::{Operator, Version};

/// Test whether `candidate` satisfies the version constraint `op constraint`.
///
/// # Operators
///
/// | Operator | Meaning |
/// |----------|---------|
/// | `<`  | candidate is strictly less than constraint |
/// | `<=` | candidate is less than or equal to constraint |
/// | `=`  | candidate is exactly equal to constraint (including revision) |
/// | `>=` | candidate is greater than or equal to constraint |
/// | `>`  | candidate is strictly greater than constraint |
/// | `~`  | candidate has the same base version, ignoring revision |
///
/// When `constraint.glob` is `true` (i.e. the `=cat/pkg-1.2*` form), the `=`
/// operator performs prefix matching via the [`Version`] ordering, which
/// implements PMS 8.3.1 glob semantics.
///
/// See [PMS 8.3.1](https://projects.gentoo.org/pms/9/pms.html#operators).
pub fn version_matches(candidate: &Version, op: &Operator, constraint: &Version) -> bool {
    match op {
        Operator::Less => candidate < constraint,
        Operator::LessOrEqual => candidate <= constraint,
        Operator::Equal => {
            // When the constraint carries a glob suffix (`=cat/pkg-1.2*`),
            // Version::cmp handles prefix matching via glob_cmp.
            // For non-glob `=`, Ord-based equality is equivalent to structural
            // equality on the version components (numbers, letter, suffixes,
            // revision).
            candidate.cmp(constraint) == Ordering::Equal
        }
        Operator::GreaterOrEqual => candidate >= constraint,
        Operator::Greater => candidate > constraint,
        Operator::Approximate => candidate.base() == constraint.base(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    // --- Less ---
    #[test]
    fn less_matches() {
        assert!(version_matches(&v("1.2.3"), &Operator::Less, &v("1.2.4")));
    }

    #[test]
    fn less_equal_does_not_match() {
        assert!(!version_matches(&v("1.2.3"), &Operator::Less, &v("1.2.3")));
    }

    #[test]
    fn less_greater_does_not_match() {
        assert!(!version_matches(&v("1.2.4"), &Operator::Less, &v("1.2.3")));
    }

    // --- LessOrEqual ---
    #[test]
    fn less_or_equal_matches_equal() {
        assert!(version_matches(
            &v("1.2.3"),
            &Operator::LessOrEqual,
            &v("1.2.3")
        ));
    }

    #[test]
    fn less_or_equal_matches_less() {
        assert!(version_matches(
            &v("1.2.2"),
            &Operator::LessOrEqual,
            &v("1.2.3")
        ));
    }

    #[test]
    fn less_or_equal_does_not_match_greater() {
        assert!(!version_matches(
            &v("1.2.4"),
            &Operator::LessOrEqual,
            &v("1.2.3")
        ));
    }

    // --- Equal ---
    #[test]
    fn equal_matches() {
        assert!(version_matches(&v("1.2.3"), &Operator::Equal, &v("1.2.3")));
    }

    #[test]
    fn equal_includes_revision() {
        assert!(!version_matches(
            &v("1.2.3-r1"),
            &Operator::Equal,
            &v("1.2.3")
        ));
    }

    #[test]
    fn equal_revisions_match() {
        assert!(version_matches(
            &v("1.2.3-r1"),
            &Operator::Equal,
            &v("1.2.3-r1")
        ));
    }

    // --- GreaterOrEqual ---
    #[test]
    fn greater_or_equal_matches_equal() {
        assert!(version_matches(
            &v("1.2.3"),
            &Operator::GreaterOrEqual,
            &v("1.2.3")
        ));
    }

    #[test]
    fn greater_or_equal_matches_greater() {
        assert!(version_matches(
            &v("1.2.4"),
            &Operator::GreaterOrEqual,
            &v("1.2.3")
        ));
    }

    #[test]
    fn greater_or_equal_does_not_match_less() {
        assert!(!version_matches(
            &v("1.2.2"),
            &Operator::GreaterOrEqual,
            &v("1.2.3")
        ));
    }

    // --- Greater ---
    #[test]
    fn greater_matches() {
        assert!(version_matches(
            &v("1.2.4"),
            &Operator::Greater,
            &v("1.2.3")
        ));
    }

    #[test]
    fn greater_equal_does_not_match() {
        assert!(!version_matches(
            &v("1.2.3"),
            &Operator::Greater,
            &v("1.2.3")
        ));
    }

    // --- Approximate (~) ---
    #[test]
    fn approximate_ignores_revision() {
        assert!(version_matches(
            &v("1.2.3-r1"),
            &Operator::Approximate,
            &v("1.2.3")
        ));
    }

    #[test]
    fn approximate_matches_same_base() {
        assert!(version_matches(
            &v("1.2.3"),
            &Operator::Approximate,
            &v("1.2.3-r2")
        ));
    }

    #[test]
    fn approximate_different_base() {
        assert!(!version_matches(
            &v("1.2.4"),
            &Operator::Approximate,
            &v("1.2.3")
        ));
    }

    // --- Equal with glob (=*) ---
    //
    // In portage-atom 0.2, `=cat/pkg-1.75*` is represented as
    // `Operator::Equal` with `Version { glob: true, .. }`.
    // The glob semantics are handled by `Version::Ord::cmp` via `glob_cmp`.

    /// Helper: build a glob version (as parsed from `=cat/pkg-VER*`).
    fn vg(s: &str) -> Version {
        let mut ver = Version::parse(s).unwrap();
        ver.glob = true;
        ver
    }

    #[test]
    fn glob_matches_prefix() {
        assert!(version_matches(&v("1.75.0"), &Operator::Equal, &vg("1.75")));
    }

    #[test]
    fn glob_matches_exact() {
        assert!(version_matches(&v("1.75"), &Operator::Equal, &vg("1.75")));
    }

    #[test]
    fn glob_does_not_match_shorter() {
        assert!(!version_matches(&v("1.7"), &Operator::Equal, &vg("1.75")));
    }

    #[test]
    fn glob_does_not_match_different() {
        assert!(!version_matches(
            &v("1.76.0"),
            &Operator::Equal,
            &vg("1.75")
        ));
    }

    #[test]
    fn glob_with_letter() {
        assert!(version_matches(
            &v("1.2.3a"),
            &Operator::Equal,
            &vg("1.2.3a")
        ));
        assert!(!version_matches(
            &v("1.2.3b"),
            &Operator::Equal,
            &vg("1.2.3a")
        ));
    }

    #[test]
    fn glob_without_letter_matches_any_letter() {
        // PMS 8.3.1: "only the given number of version components is used
        // for comparison" — the letter is not a numeric component, so
        // `=1.2.3*` matches `1.2.3a`.
        assert!(version_matches(
            &v("1.2.3a"),
            &Operator::Equal,
            &vg("1.2.3")
        ));
    }

    // --- Suffix edge cases ---
    #[test]
    fn less_with_suffix() {
        assert!(version_matches(
            &v("1.2.3_rc1"),
            &Operator::Less,
            &v("1.2.3")
        ));
    }

    #[test]
    fn greater_with_patchlevel() {
        assert!(version_matches(
            &v("1.2.3_p1"),
            &Operator::Greater,
            &v("1.2.3")
        ));
    }
}
