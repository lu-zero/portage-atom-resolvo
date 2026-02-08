//! PMS version matching operators.
//!
//! Implements [PMS 8.3.2](https://projects.gentoo.org/pms/latest/pms.html#x1-830008.3.2):
//! given a candidate version, an operator, and a constraint version, determine
//! whether the candidate satisfies the constraint.

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
/// | `=*` | candidate's version string starts with constraint as a prefix |
///
/// See [PMS 8.3.2](https://projects.gentoo.org/pms/latest/pms.html#x1-830008.3.2).
pub fn version_matches(candidate: &Version, op: &Operator, constraint: &Version) -> bool {
    match op {
        Operator::Less => candidate < constraint,
        Operator::LessOrEqual => candidate <= constraint,
        Operator::Equal => candidate == constraint,
        Operator::GreaterOrEqual => candidate >= constraint,
        Operator::Greater => candidate > constraint,
        Operator::Approximate => candidate.base() == constraint.base(),
        Operator::EqualGlob => glob_matches(candidate, constraint),
    }
}

/// `=*` glob matching: the candidate's numeric components must start with the
/// constraint's components as a prefix, and the candidate's letter (if the
/// constraint specifies one) must match.
///
/// For example, `=dev-lang/rust-1.75*` matches `1.75.0`, `1.75.1`, `1.75`,
/// but not `1.7` or `1.8`.
fn glob_matches(candidate: &Version, constraint: &Version) -> bool {
    // The constraint's numeric components must be a prefix of the candidate's.
    if candidate.numbers.len() < constraint.numbers.len() {
        return false;
    }
    for (c, p) in candidate.numbers.iter().zip(constraint.numbers.iter()) {
        if c != p {
            return false;
        }
    }

    // If constraint specifies a letter, candidate must match it.
    if let Some(cl) = constraint.letter {
        match candidate.letter {
            Some(l) => l == cl,
            None => {
                // Candidate has no letter but constraint does — only matches
                // if candidate has more numeric components (the letter is part
                // of the prefix, not a separate component to match).
                false
            }
        }
    } else {
        true
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

    // --- EqualGlob (=*) ---
    #[test]
    fn glob_matches_prefix() {
        assert!(version_matches(
            &v("1.75.0"),
            &Operator::EqualGlob,
            &v("1.75")
        ));
    }

    #[test]
    fn glob_matches_exact() {
        assert!(version_matches(
            &v("1.75"),
            &Operator::EqualGlob,
            &v("1.75")
        ));
    }

    #[test]
    fn glob_does_not_match_shorter() {
        assert!(!version_matches(
            &v("1.7"),
            &Operator::EqualGlob,
            &v("1.75")
        ));
    }

    #[test]
    fn glob_does_not_match_different() {
        assert!(!version_matches(
            &v("1.76.0"),
            &Operator::EqualGlob,
            &v("1.75")
        ));
    }

    #[test]
    fn glob_with_letter() {
        assert!(version_matches(
            &v("1.2.3a"),
            &Operator::EqualGlob,
            &v("1.2.3a")
        ));
        assert!(!version_matches(
            &v("1.2.3b"),
            &Operator::EqualGlob,
            &v("1.2.3a")
        ));
    }

    #[test]
    fn glob_without_letter_matches_any_letter() {
        assert!(version_matches(
            &v("1.2.3a"),
            &Operator::EqualGlob,
            &v("1.2.3")
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
