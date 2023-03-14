use bstr::{ByteSlice, B};
use std::cmp::Ordering;

use crate::ascii::fill_and_check;
use crate::cea::generate_cea;
use crate::first_weight::try_initial;
use crate::normalize::make_nfd;
use crate::prefix::trim_prefix;
use crate::sort_key::compare_incremental;
use crate::Tailoring;

/// The `Collator` struct is the entry point for this library's API. It defines the options to be
/// used in collation. The method `collate` will then compare two string references (or byte slices)
/// according to the selected options, and return an `Ordering` value.
///
/// You can choose between two tables of character weights: DUCET and CLDR. With the CLDR table,
/// there is a further choice of locale tailoring. The `Root` locale represents the table in its
/// unmodified form. The `ArabicScript` locale shifts the weights of Arabic-script letters so that
/// they sort before the Latin script. Further locales will be added over time.
///
/// You can also choose between two approaches to the handling of variable-weight characters:
/// "non-ignorable" and "shifted." Finally, you can select whether to use byte-value comparison as a
/// tiebreaker when two strings produce identical Unicode Collation Algorithm sort keys.
///
/// The default for `Collator` is to use the CLDR table with the `Root` locale; to use the "shifted"
/// approach for variable-weight characters; and to break ties with byte-value comparison. This
/// should be a good starting point for collation in many languages.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Collator {
    /// The table of weights to be used: DUCET or CLDR (with a choice of locale for the latter)
    pub tailoring: Tailoring,
    /// The approach to handling variable-weight characters: "non-ignorable" (i.e., `false`) or
    /// "shifted" (i.e., `true`)
    pub shifting: bool,
    /// Whether to use byte-value comparison as a tiebreaker when two strings produce identical
    /// Unicode Collation Algorithm sort keys
    pub tiebreak: bool,
    a_cea: Vec<u32>,
    b_cea: Vec<u32>,
}

impl Default for Collator {
    fn default() -> Self {
        Self::new(Tailoring::default(), true, true)
    }
}

impl Collator {
    /// Create a new `Collator` with the specified options. NB: it is also possible to call
    /// `Collator::default()`.
    #[must_use]
    pub fn new(tailoring: Tailoring, shifting: bool, tiebreak: bool) -> Self {
        Self {
            tailoring,
            shifting,
            tiebreak,
            a_cea: vec![0; 32],
            b_cea: vec![0; 32],
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
    /// let mut collator = Collator::default();
    ///
    /// let mut names = ["Peng", "Peña", "Ernie", "Émile"];
    /// names.sort_unstable_by(|a, b| collator.collate(a, b));
    ///
    /// let expected = ["Émile", "Ernie", "Peña", "Peng"];
    /// assert_eq!(names, expected);
    /// ```
    pub fn collate<T: AsRef<[u8]> + Eq + Ord + ?Sized>(&mut self, a: &T, b: &T) -> Ordering {
        // Early out; equal is equal
        if a == b {
            return Ordering::Equal;
        }

        // Validate UTF-8 and make an iterator for u32 code points
        let mut a_iter = B(a).chars().map(|c| c as u32);
        let mut b_iter = B(b).chars().map(|c| c as u32);

        // Set up Vecs for code points
        let mut a_chars: Vec<u32> = Vec::new();
        let mut b_chars: Vec<u32> = Vec::new();

        // While iterating through input strings and filling code point Vecs, try to get a result by
        // comparing ASCII characters. This can avoid a lot of computation.
        if let Some(o) = fill_and_check(&mut a_iter, &mut b_iter, &mut a_chars, &mut b_chars) {
            return o;
        }

        // Normalize to NFD
        make_nfd(&mut a_chars);
        make_nfd(&mut b_chars);

        // I think it's worth offering an out here, too, in case two strings decompose to the same.
        // If we went forward and generated sort keys, they would be equal, anyway.
        if a_chars == b_chars {
            if self.tiebreak {
                return a.cmp(b);
            }

            return Ordering::Equal;
        }

        // Check for a shared prefix that might be safe to trim
        let shifting = self.shifting;
        trim_prefix(&mut a_chars, &mut b_chars, shifting);

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
        let tailoring = self.tailoring;
        generate_cea(&mut self.a_cea, &mut a_chars, shifting, tailoring);
        generate_cea(&mut self.b_cea, &mut b_chars, shifting, tailoring);

        // Sort keys are processed incrementally, until they yield a result
        let comparison = compare_incremental(&self.a_cea, &self.b_cea, shifting);

        if comparison == Ordering::Equal && self.tiebreak {
            return a.cmp(b);
        }

        comparison
    }
}
