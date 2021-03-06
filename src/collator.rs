use bstr::{ByteSlice, B};
use std::cmp::Ordering;

use crate::ascii::{all_ascii, compare_ascii};
use crate::cea::generate_cea;
use crate::first_weight::{get_first_primary, safe_first_chars};
use crate::normalize::make_nfd;
use crate::prefix::trim_prefix;
use crate::sort_key::compare_incremental;
use crate::KeysSource;

/// The `Collator` struct is the entry point for this library's API. It defines the options to be
/// used in collation. The method `collate` or `collate_no_tiebreak` will then compare two string
/// references (or byte slices) according to the selected options, and return an `Ordering` value.
///
/// At present, you can choose between two tables of character weights (DUCET and CLDR root), and
/// between two approaches to the handling of variable-weight characters ("non-ignorable" and
/// "shifted"). The default, and a good starting point for Unicode collation, is to use the CLDR
/// table with the "shifted" approach.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Collator {
    /// The table of weights to be used (currently either DUCET or CLDR)
    pub keys_source: KeysSource,
    /// The approach to handling variable-weight characters ("non-ignorable" or "shifted"). For our
    /// purposes, `shifting` is either true (recommended) or false.
    pub shifting: bool,
}

impl Default for Collator {
    fn default() -> Self {
        Self {
            keys_source: KeysSource::Cldr,
            shifting: true,
        }
    }
}

impl Collator {
    /// Create a new `Collator`. This is equivalent to calling `Collator::default()`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// This is the primary method in the library. It accepts as arguments two string references or
    /// byte slices; compares them using the options chosen; and returns an `Ordering` value. This
    /// is designed to be passed to the `sort_by` function in the standard library. Simple usage
    /// might look like the following...
    ///
    /// ```
    /// use feruca::{Collator};
    ///
    /// let collator = Collator::default();
    ///
    /// let mut names = ["Peng", "Pe??a", "Ernie", "??mile"];
    /// names.sort_by(|a, b| collator.collate(a, b));
    ///
    /// let expected = ["??mile", "Ernie", "Pe??a", "Peng"];
    /// assert_eq!(names, expected);
    /// ```
    ///
    /// Significantly, in the event that two strings are ordered equally per the Unicode Collation
    /// Algorithm, this method will use byte-value comparison (i.e., the traditional, na??ve way of
    /// sorting strings) as a tiebreaker. While this is probably appropriate in most cases, it can
    /// be avoided by using the `collate_no_tiebreak` method.
    pub fn collate<T: AsRef<[u8]> + Eq + Ord + ?Sized>(self, a: &T, b: &T) -> Ordering {
        // Early out; equal is equal
        if a == b {
            return Ordering::Equal;
        }

        // Turn both into Vecs of u32 code points, while validating UTF-8
        let mut a_chars: Vec<u32> = B(a).chars().map(|c| c as u32).collect();
        let mut b_chars: Vec<u32> = B(b).chars().map(|c| c as u32).collect();

        // Check if both are entirely alphanumeric ASCII
        let easy = all_ascii(&a_chars, &b_chars);
        if easy {
            return compare_ascii(a_chars, b_chars);
        }

        // Normalize to NFD
        make_nfd(&mut a_chars);
        make_nfd(&mut b_chars);

        // I think it's worth offering an out here, too, in case two strings decompose to the same.
        // If we went forward and generated sort keys, they would be equal, and we would end up at the
        // tiebreaker, anyway.
        if a_chars == b_chars {
            // Tiebreaker
            return a.cmp(b);
        }

        // Check for a shared prefix that might be safe to trim
        trim_prefix(&mut a_chars, &mut b_chars, self.shifting);

        // After prefix trimming, one of the Vecs may be empty (but not both!)
        if a_chars.is_empty() || b_chars.is_empty() {
            return a_chars.cmp(&b_chars);
        }

        // One last chance for an early out: if the opening code points of the two Vecs are different,
        // and neither requires checking for a multi-code-point sequence, then we can try comparing
        // their first primary weights. If those are different, and both non-zero, it's decisive.
        if safe_first_chars(&a_chars, &b_chars) {
            let a_first_primary = get_first_primary(a_chars[0], self);
            let b_first_primary = get_first_primary(b_chars[0], self);

            if a_first_primary != b_first_primary && a_first_primary != 0 && b_first_primary != 0 {
                return a_first_primary.cmp(&b_first_primary);
            }
        }

        // Otherwise we move forward with full collation element arrays
        let a_cea = generate_cea(&mut a_chars, self);
        let b_cea = generate_cea(&mut b_chars, self);

        // Sort keys are processed incrementally, until they yield a result
        let comparison = compare_incremental(&a_cea, &b_cea, self.shifting);

        if comparison == Ordering::Equal {
            // Tiebreaker
            return a.cmp(b);
        }

        comparison
    }

    /// This is a variation on `collate`, to which it is almost identical. The difference is that,
    /// in the event that two strings are ordered equally per the Unicode Collation Algorithm, this
    /// method will not attempt to "break the tie" by using byte-value comparison.
    pub fn collate_no_tiebreak<T: AsRef<[u8]> + Eq + Ord + ?Sized>(self, a: &T, b: &T) -> Ordering {
        if a == b {
            return Ordering::Equal;
        }

        let mut a_chars: Vec<u32> = B(a).chars().map(|c| c as u32).collect();
        let mut b_chars: Vec<u32> = B(b).chars().map(|c| c as u32).collect();

        let easy = all_ascii(&a_chars, &b_chars);
        if easy {
            return compare_ascii(a_chars, b_chars);
        }

        make_nfd(&mut a_chars);
        make_nfd(&mut b_chars);

        if a_chars == b_chars {
            return Ordering::Equal;
        }

        trim_prefix(&mut a_chars, &mut b_chars, self.shifting);

        if a_chars.is_empty() || b_chars.is_empty() {
            return a_chars.cmp(&b_chars);
        }

        if safe_first_chars(&a_chars, &b_chars) {
            let a_first_primary = get_first_primary(a_chars[0], self);
            let b_first_primary = get_first_primary(b_chars[0], self);

            if a_first_primary != b_first_primary && a_first_primary != 0 && b_first_primary != 0 {
                return a_first_primary.cmp(&b_first_primary);
            }
        }

        let a_cea = generate_cea(&mut a_chars, self);
        let b_cea = generate_cea(&mut b_chars, self);

        compare_incremental(&a_cea, &b_cea, self.shifting)
    }
}
