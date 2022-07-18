//! This crate provides a basic implementation of the Unicode Collation Algorithm. There is really
//! just one function, `collate`, and a few options that can be passed to it. (The
//! `collate_no_tiebreak` function is a variation whose behavior is a bit more strict.) Despite the
//! bare-bones API, this implementation conforms to the standard and allows for the use of the CLDR
//! root collation order; so it may indeed be useful, even in this early stage of development.

#![warn(clippy::pedantic, clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![deny(missing_docs)]

use bstr::{ByteSlice, B};
use serde::Deserialize;
use std::cmp::Ordering;

mod ascii;
use ascii::{all_ascii, compare_ascii};

mod cea;
use cea::generate_cea;

mod cea_utils;
mod consts;

mod first_weight;
use first_weight::{get_first_primary, safe_first_chars};

mod normalize;
use normalize::make_nfd;

mod prefix;
use prefix::trim_prefix;

mod sort_key;
use sort_key::compare_incremental;

//
// Structs and enums
//

/// This struct specifies the options to be passed to the `collate` function. You can choose between
/// two tables (DUCET and CLDR root), and between two approaches to the handling of variable-weight
/// characters ("non-ignorable" and "shifted"). The default, and a good starting point for Unicode
/// collation, is to use the CLDR table with the "shifted" approach.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct CollationOptions {
    /// The table of weights to be used (currently either DUCET or CLDR)
    pub keys_source: KeysSource,
    /// The approach to handling variable-weight characters ("non-ignorable" or "shifted"). For our
    /// purposes, `shifting` is either true (recommended) or false.
    pub shifting: bool,
}

impl Default for CollationOptions {
    fn default() -> Self {
        Self {
            keys_source: KeysSource::Cldr,
            shifting: true,
        }
    }
}

/// This enum provides for a choice of which table of character weights to use.
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
pub enum KeysSource {
    /// The table associated with the CLDR root collation order (recommended)
    Cldr,
    /// The default table for the Unicode Collation Algorithm
    Ducet,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
struct Weights {
    variable: bool,
    primary: u16,
    secondary: u16,
    tertiary: u16,
}

//
// Public functions
//

/// This is the main public function in the library. It accepts as arguments two string references
/// or byte slices, and a `CollationOptions` struct. It returns an `Ordering` value. This is
/// designed to be used in conjunction with the `sort_by` function in the standard library. Simple
/// usage might look like the following...
///
/// ```
/// use feruca::{collate, CollationOptions};
///
/// let mut names = ["Peng", "Peña", "Ernie", "Émile"];
/// names.sort_by(|a, b| collate(a, b, CollationOptions::default()));
///
/// let expected = ["Émile", "Ernie", "Peña", "Peng"];
/// assert_eq!(names, expected);
/// ```
///
/// Significantly, in the event that two strings are ordered equally per the Unicode Collation
/// Algorithm, this function will use byte-value comparison (i.e., the traditional, naïve way of
/// sorting strings) as a tiebreaker. While this is probably appropriate in most cases, it can be
/// avoided by using the `collate_no_tiebreak` function.
#[must_use]
pub fn collate<T: AsRef<[u8]> + Eq + Ord>(a: &T, b: &T, opt: CollationOptions) -> Ordering {
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
    trim_prefix(&mut a_chars, &mut b_chars, opt);

    // After prefix trimming, one of the Vecs may be empty (but not both!)
    if a_chars.is_empty() || b_chars.is_empty() {
        return a_chars.cmp(&b_chars);
    }

    // One last chance for an early out: if the opening code points of the two Vecs are different,
    // and neither requires checking for a multi-code-point sequence, then we can try comparing
    // their first primary weights. If those are different, and both non-zero, it's decisive.
    if safe_first_chars(&a_chars, &b_chars) {
        let a_first_primary = get_first_primary(a_chars[0], opt);
        let b_first_primary = get_first_primary(b_chars[0], opt);

        if a_first_primary != b_first_primary && a_first_primary != 0 && b_first_primary != 0 {
            return a_first_primary.cmp(&b_first_primary);
        }
    }

    // Otherwise we move forward with full collation element arrays
    let a_cea = generate_cea(&mut a_chars, opt);
    let b_cea = generate_cea(&mut b_chars, opt);

    // Sort keys are processed incrementally, until they yield a result
    let comparison = compare_incremental(&a_cea, &b_cea, opt.shifting);

    if comparison == Ordering::Equal {
        // Tiebreaker
        return a.cmp(b);
    }

    comparison
}

/// This is a variation on the `collate` function, to which it is almost identical. The difference
/// is that, in the event that two strings are ordered equally per the Unicode Collation Algorithm,
/// this function will not attempt to "break the tie" by using byte-value comparison.
#[must_use]
pub fn collate_no_tiebreak<T: AsRef<[u8]> + Eq + Ord>(
    a: &T,
    b: &T,
    opt: CollationOptions,
) -> Ordering {
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

    trim_prefix(&mut a_chars, &mut b_chars, opt);

    if a_chars.is_empty() || b_chars.is_empty() {
        return a_chars.cmp(&b_chars);
    }

    if safe_first_chars(&a_chars, &b_chars) {
        let a_first_primary = get_first_primary(a_chars[0], opt);
        let b_first_primary = get_first_primary(b_chars[0], opt);

        if a_first_primary != b_first_primary && a_first_primary != 0 && b_first_primary != 0 {
            return a_first_primary.cmp(&b_first_primary);
        }
    }

    let a_cea = generate_cea(&mut a_chars, opt);
    let b_cea = generate_cea(&mut b_chars, opt);

    compare_incremental(&a_cea, &b_cea, opt.shifting)
}
