//! This crate provides a basic implementation of the Unicode Collation Algorithm. There is really
//! just one function, `collate`, and a few options that can be passed to it. (The
//! `collate_no_tiebreak` function is a variation whose behavior is a bit more strict.) Despite the
//! bare-bones API, this implementation conforms to the standard and allows for the use of the CLDR
//! root collation order; so it may indeed be useful, even in this early stage of development.

#![warn(clippy::pedantic, clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![deny(missing_docs)]

use serde::Deserialize;
use std::cmp::Ordering;

mod cea;
mod cea_utils;
mod consts;

mod normalize;
use normalize::get_nfd;

mod prefix;
use prefix::trim_prefix;

mod sort_key;
use sort_key::nfd_to_sk;

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
/// and a `CollationOptions` struct. It returns an `Ordering` value. This is designed to be used in
/// conjunction with the `sort_by` function in the standard library. Simple usage might look like
/// the following...
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
pub fn collate(str_a: &str, str_b: &str, opt: CollationOptions) -> Ordering {
    // Early out
    if str_a == str_b {
        return Ordering::Equal;
    }

    let mut a_nfd = get_nfd(str_a);
    let mut b_nfd = get_nfd(str_b);

    // I think it's worth offering an out here, too, in case two strings decompose to the same.
    // If we went forward and generated sort keys, they would be equal, and we would end up at the
    // tiebreaker, anyway.
    if a_nfd == b_nfd {
        // Tiebreaker
        return str_a.cmp(str_b);
    }

    let cldr = opt.keys_source == KeysSource::Cldr;
    trim_prefix(&mut a_nfd, &mut b_nfd, cldr);

    let a_sort_key = nfd_to_sk(&mut a_nfd, opt);
    let b_sort_key = nfd_to_sk(&mut b_nfd, opt);

    let comparison = a_sort_key.cmp(&b_sort_key);

    if comparison == Ordering::Equal {
        // Tiebreaker
        return str_a.cmp(str_b);
    }

    comparison
}

/// This is a variation on the `collate` function, to which it is almost identical. The difference
/// is that, in the event that two strings are ordered equally per the Unicode Collation Algorithm,
/// this function will not attempt to "break the tie" by using byte-value comparison.
#[must_use]
pub fn collate_no_tiebreak(str_a: &str, str_b: &str, opt: CollationOptions) -> Ordering {
    if str_a == str_b {
        return Ordering::Equal;
    }

    let mut a_nfd = get_nfd(str_a);
    let mut b_nfd = get_nfd(str_b);

    if a_nfd == b_nfd {
        return Ordering::Equal;
    }

    let cldr = opt.keys_source == KeysSource::Cldr;
    trim_prefix(&mut a_nfd, &mut b_nfd, cldr);

    let a_sort_key = nfd_to_sk(&mut a_nfd, opt);
    let b_sort_key = nfd_to_sk(&mut b_nfd, opt);

    a_sort_key.cmp(&b_sort_key)
}
