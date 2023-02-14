use bstr::{ByteSlice, B};
use std::cmp::Ordering;

use crate::ascii::try_ascii;
use crate::cea::generate_cea;
use crate::first_weight::try_initial;
use crate::normalize::make_nfd;
use crate::prefix::trim_prefix;
use crate::sort_key::compare_incremental;
use crate::Tailoring;

/// The `Collator` struct is the entry point for this library's API. It defines the options to be
/// used in collation. The method `collate` or `collate_no_tiebreak` will then compare two string
/// references (or byte slices) according to the selected options, and return an `Ordering` value.
///
/// You can choose between two tables of character weights: DUCET and CLDR. With the CLDR table,
/// there is a further choice of locale tailoring. The `Root` locale represents the table in its
/// unmodified form. The `ArabicScript` locale shifts the weights of Arabic-script letters so that
/// they sort before the Latin script. Further locales will be added over time.
///
/// You can also choose between two approaches to the handling of variable-weight characters:
/// "non-ignorable" and "shifted."
///
/// The default for `Collator` is to use the CLDR table with the `Root` locale, and the "shifted"
/// approach. This is a good starting point for collation in many languages.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Collator {
    /// The table of weights to be used: DUCET or CLDR (with a choice of locale for the latter)
    pub tailoring: Tailoring,
    /// The approach to handling variable-weight characters ("non-ignorable" or "shifted"). For our
    /// purposes, `shifting` is either true (recommended) or false.
    pub shifting: bool,
}

impl Default for Collator {
    fn default() -> Self {
        Self::new(Tailoring::default(), true)
    }
}

impl Collator {
    /// Create a new `Collator` with the specified options. Please note that it is also possible to
    /// call `Collator::default()`.
    #[must_use]
    pub const fn new(tailoring: Tailoring, shifting: bool) -> Self {
        Self {
            tailoring,
            shifting,
        }
    }

    /// This is the primary method in the library. It accepts as arguments two string references or
    /// byte slices; compares them using the options chosen; and returns an `Ordering` value. This
    /// is designed to be passed to the `sort_by` (or `sort_unstable_by`) function in the standard
    /// library. Simple usage might look like the following...
    ///
    /// ```
    /// use feruca::{Collator};
    ///
    /// let collator = Collator::default();
    ///
    /// let mut names = ["Peng", "Peña", "Ernie", "Émile"];
    /// names.sort_unstable_by(|a, b| collator.collate(a, b));
    ///
    /// let expected = ["Émile", "Ernie", "Peña", "Peng"];
    /// assert_eq!(names, expected);
    /// ```
    ///
    /// Significantly, in the event that two strings are ordered equally per the Unicode Collation
    /// Algorithm, this method will use byte-value comparison (i.e., the traditional, naïve way of
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

        // If we can get a decisive result from comparing alphanumeric ASCII characters in the two
        // strings, return that
        if let Some(o) = try_ascii(&a_chars, &b_chars) {
            return o;
        }

        // Normalize to NFD
        make_nfd(&mut a_chars);
        make_nfd(&mut b_chars);

        // I think it's worth offering an out here, too, in case two strings decompose to the same.
        // If we went forward and generated sort keys, they would be equal, and we would end up at
        // the tiebreaker, anyway.
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

        // One last chance for an early out: if the opening code points of the two Vecs are
        // different, and neither requires checking for a multi-code-point sequence, then we can try
        // comparing their first primary weights. If those are different, and both non-zero, it's
        // decisive.
        if let Some(o) = try_initial(self, &a_chars, &b_chars) {
            return o;
        }

        // Otherwise we move forward with full collation element arrays
        let a_cea = generate_cea(self, &mut a_chars);
        let b_cea = generate_cea(self, &mut b_chars);

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

        if let Some(o) = try_ascii(&a_chars, &b_chars) {
            return o;
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

        if let Some(o) = try_initial(self, &a_chars, &b_chars) {
            return o;
        }

        let a_cea = generate_cea(self, &mut a_chars);
        let b_cea = generate_cea(self, &mut b_chars);

        compare_incremental(&a_cea, &b_cea, self.shifting)
    }
}
