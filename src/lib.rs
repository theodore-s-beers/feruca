//! This crate provides a basic implementation of the Unicode Collation Algorithm. There is really
//! just one function, `collate`, and a few options that can be passed to it. But the implementation
//! conforms to the standard and allows for the use of the CLDR root collation order; so it may
//! indeed be useful, even in this early stage of development.

#![warn(clippy::pedantic, clippy::cargo)]
#![deny(missing_docs)]

use std::cmp::Ordering;
use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::Deserialize;
use unicode_canonical_combining_class::get_canonical_combining_class as get_ccc;
use unicode_normalization::UnicodeNormalization;

//
// Structs etc.
//

/// This struct specifies the options to be passed to the `collate` function. You can choose between
/// two tables (DUCET and CLDR root), and between two approaches to the handling of variable-weight
/// characters ("non-ignorable" and "shifted"). The default, and a good starting point for Unicode
/// collation, is to use the CLDR table with the "shifted" approach.
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
#[derive(PartialEq, Eq)]
pub enum KeysSource {
    /// The table associated with the CLDR root collation order (recommended)
    Cldr,
    /// The default table for the Unicode Collation Algorithm
    Ducet,
}

#[derive(Deserialize)]
struct Weights {
    variable: bool,
    primary: u16,
    secondary: u16,
    tertiary: u16,
}

//
// Static/const
//

static SINGLES: &[u8; 662_610] = include_bytes!("bincode/singles");
static MULTIS: &[u8; 35_328] = include_bytes!("bincode/multis");
static SINGLES_CLDR: &[u8; 662_466] = include_bytes!("bincode/singles_cldr");
static MULTIS_CLDR: &[u8; 35_724] = include_bytes!("bincode/multis_cldr");

static S_KEYS: Lazy<HashMap<u32, Vec<Weights>>> = Lazy::new(|| {
    let decoded: HashMap<u32, Vec<Weights>> = bincode::deserialize(SINGLES).unwrap();
    decoded
});

static M_KEYS: Lazy<HashMap<Vec<u32>, Vec<Weights>>> = Lazy::new(|| {
    let decoded: HashMap<Vec<u32>, Vec<Weights>> = bincode::deserialize(MULTIS).unwrap();
    decoded
});

static S_KEYS_CLDR: Lazy<HashMap<u32, Vec<Weights>>> = Lazy::new(|| {
    let decoded: HashMap<u32, Vec<Weights>> = bincode::deserialize(SINGLES_CLDR).unwrap();
    decoded
});

static M_KEYS_CLDR: Lazy<HashMap<Vec<u32>, Vec<Weights>>> = Lazy::new(|| {
    let decoded: HashMap<Vec<u32>, Vec<Weights>> = bincode::deserialize(MULTIS_CLDR).unwrap();
    decoded
});

const NEED_THREE: [u32; 4] = [3_270, 3_545, 4_018, 4_019];

const NEED_TWO: [u32; 59] = [
    76, 108, 1_048, 1_080, 1_575, 1_608, 1_610, 2_503, 2_887, 2_962, 3_014, 3_015, 3_142, 3_263,
    3_274, 3_398, 3_399, 3_548, 3_648, 3_649, 3_650, 3_651, 3_652, 3_661, 3_776, 3_777, 3_778,
    3_779, 3_780, 3_789, 3_953, 4_133, 6_581, 6_582, 6_583, 6_586, 6_917, 6_919, 6_921, 6_923,
    6_925, 6_929, 6_970, 6_972, 6_974, 6_975, 6_978, 43_701, 43_702, 43_705, 43_707, 43_708,
    69_937, 69_938, 70_471, 70_841, 71_096, 71_097, 71_989,
];

const INCLUDED_UNASSIGNED: [u32; 4] = [177_977, 178_206, 183_970, 191_457];

//
// Functions, public
//

/// This is, so far, the only public function in the library. It accepts as arguments two string
/// references and a `CollationOptions` struct. It returns an `Ordering` value. This is designed to
/// be used in conjunction with the `sort_by` function in the standard library. Simple usage might
/// look like the following...
///
/// ```
/// use feruca::{collate, CollationOptions};
///
/// let mut names = ["Peng", "Peña", "Ernie", "Émile"];
/// names.sort_by(|a, b| collate(a, b, &CollationOptions::default()));
///
/// let expected = ["Émile", "Ernie", "Peña", "Peng"];
/// assert_eq!(names, expected);
/// ```
///
/// Significantly, in the event that two strings are ordered equally per the Unicode Collation
/// Algorithm, this function will use byte-value comparison (i.e., the traditional, naïve way of
/// sorting strings) as a tiebreaker. A `collate_no_tiebreak` function may be added in the future,
/// if there is demand for it.
#[must_use]
pub fn collate(str_a: &str, str_b: &str, opt: &CollationOptions) -> Ordering {
    // Early out
    if str_a == str_b {
        return Ordering::Equal;
    }

    let a_nfd = get_nfd(str_a);
    let b_nfd = get_nfd(str_b);

    // I think it's worth offering an out here, too, in case two strings decompose to the same
    if a_nfd == b_nfd {
        // Tiebreaker
        return str_a.cmp(str_b);
    }

    let a_sort_key = nfd_to_sk(a_nfd, opt);
    let b_sort_key = nfd_to_sk(b_nfd, opt);

    let comparison = compare_sort_keys(&a_sort_key, &b_sort_key);

    if comparison == Ordering::Equal {
        // Tiebreaker
        return str_a.cmp(str_b);
    }

    comparison
}

//
// Functions, private
//

fn compare_sort_keys(a: &[u16], b: &[u16]) -> Ordering {
    let min_length = a.len().min(b.len());

    for i in 0..min_length {
        if a[i] < b[i] {
            return Ordering::Less;
        }

        if a[i] > b[i] {
            return Ordering::Greater;
        }
    }

    Ordering::Equal
}

fn get_nfd(input: &str) -> Vec<u32> {
    UnicodeNormalization::nfd(input).map(|c| c as u32).collect()
}

fn nfd_to_sk(nfd: Vec<u32>, opt: &CollationOptions) -> Vec<u16> {
    let cea = get_collation_element_array(nfd, opt);
    get_sort_key(&cea, opt.shifting)
}

fn get_sort_key(collation_element_array: &[Vec<u16>], shifting: bool) -> Vec<u16> {
    let max_level = if shifting { 4 } else { 3 };
    let mut sort_key: Vec<u16> = Vec::new();

    for i in 0..max_level {
        if i > 0 {
            sort_key.push(0);
        }

        for elem in collation_element_array.iter() {
            if elem[i] != 0 {
                sort_key.push(elem[i]);
            }
        }
    }

    sort_key
}

#[allow(clippy::too_many_lines)]
fn get_collation_element_array(mut char_vals: Vec<u32>, opt: &CollationOptions) -> Vec<Vec<u16>> {
    let mut cea: Vec<Vec<u16>> = Vec::new();

    let cldr = opt.keys_source == KeysSource::Cldr;
    let shifting = opt.shifting;

    let singles = if cldr { &S_KEYS_CLDR } else { &S_KEYS };
    let multis = if cldr { &M_KEYS_CLDR } else { &M_KEYS };

    let mut left: usize = 0;
    let mut last_variable = false;

    'outer: while left < char_vals.len() {
        let left_val = char_vals[left];

        // Set lookahead depending on left_val. We need 3 in a few cases; 2 in several dozen cases;
        // and 1 otherwise.
        let lookahead: usize = match left_val {
            x if NEED_THREE.contains(&x) => 3,
            x if NEED_TWO.contains(&x) => 2,
            _ => 1,
        };

        let check_multi = lookahead > 1 && char_vals.len() - left > 1;

        // If lookahead is 1, or if this is the last item in the vec, take an easy path
        if !check_multi {
            if let Some(row) = singles.get(&left_val) {
                // If we found our row, push weights to the collation element array
                for weights in row {
                    if shifting {
                        let weight_vals = get_shifted_weights(weights, last_variable);
                        cea.push(weight_vals);
                        if weights.variable {
                            last_variable = true;
                        } else if weights.primary != 0 {
                            last_variable = false;
                        }
                    } else {
                        let weight_vals =
                            vec![weights.primary, weights.secondary, weights.tertiary];
                        cea.push(weight_vals);
                    }
                }

                // Increment and continue outer loop
                left += 1;
                continue 'outer;
            }
        }

        // Next consider multi-code-point matches, if applicable
        // If we just tried to find a single, and didn't find it, we skip all the way down to the
        // implicit weights section

        // Don't look past the end of the vec
        let mut right = if left + lookahead > char_vals.len() {
            char_vals.len()
        } else {
            left + lookahead
        };

        'middle: while check_multi && right > left {
            // If right - left == 1 (which cannot be the case in the first iteration), attempts to
            // find a slice have failed. So look for one code point, in the singles map
            if right - left == 1 {
                // If we found it, we do still need to check for discontiguous matches
                if let Some(value) = singles.get(&left_val) {
                    // Determine how much further right to look
                    let mut max_right = if right + 2 < char_vals.len() {
                        right + 2
                    } else if right + 1 < char_vals.len() {
                        right + 1
                    } else {
                        // This should skip the loop below. There will be no discontiguous match.
                        right
                    };

                    let mut try_two = max_right - right == 2 && cldr;

                    'inner: while max_right > right {
                        // Make sure the sequence of CCC values is kosher
                        let interest_cohort = &char_vals[right..=max_right];
                        let mut max_ccc = 0;

                        for elem in interest_cohort {
                            let ccc = get_ccc(char::from_u32(*elem).unwrap()) as u8;
                            if ccc == 0 || ccc <= max_ccc {
                                // Can also forget about try_two in this case
                                try_two = false;
                                max_right -= 1;
                                continue 'inner;
                            }
                            max_ccc = ccc;
                        }

                        // Having made it this far, we can test a new subset, adding later char(s)
                        let new_subset = if try_two {
                            [[left_val].as_slice(), &char_vals[max_right - 1..=max_right]].concat()
                        } else {
                            vec![left_val, char_vals[max_right]]
                        };

                        // If the new subset is found in the table...
                        if let Some(new_value) = multis.get(&new_subset) {
                            // Then add these weights instead
                            for weights in new_value {
                                if shifting {
                                    let weight_vals = get_shifted_weights(weights, last_variable);
                                    cea.push(weight_vals);
                                    if weights.variable {
                                        last_variable = true;
                                    } else if weights.primary != 0 {
                                        last_variable = false;
                                    }
                                } else {
                                    let weight_vals =
                                        vec![weights.primary, weights.secondary, weights.tertiary];
                                    cea.push(weight_vals);
                                }
                            }

                            // Remove the pulled char(s) (in this order!)
                            char_vals.remove(max_right);
                            if try_two {
                                char_vals.remove(max_right - 1);
                            }

                            // Increment and continue outer loop
                            left += right - left;
                            continue 'outer;
                        }

                        // If we tried for two, don't decrement max_right yet
                        // Inner loop will re-run
                        if try_two {
                            try_two = false;
                        } else {
                            // Otherwise decrement max_right; inner loop may re-run, or finish
                            max_right -= 1;
                        }
                    }

                    // At this point, we're not looking for a discontiguous match. We just need to
                    // push the weights we found above.

                    for weights in value {
                        if shifting {
                            let weight_vals = get_shifted_weights(weights, last_variable);
                            cea.push(weight_vals);
                            if weights.variable {
                                last_variable = true;
                            } else if weights.primary != 0 {
                                last_variable = false;
                            }
                        } else {
                            let weight_vals =
                                vec![weights.primary, weights.secondary, weights.tertiary];
                            cea.push(weight_vals);
                        }
                    }

                    // Increment and continue outer loop
                    left += right - left;
                    continue 'outer;
                }

                // We failed to find the last code point, after trying multiples and failing there,
                // too. This is the most wasteful path. Now we skip down to calculate implicit
                // weights for this code point. Decrementing `right` and continuing the middle loop
                // should accomplish that.
                right -= 1;
                continue 'middle;
            }

            // If we got here, we're trying to find a slice
            let subset = &char_vals[left..right];

            if let Some(value) = multis.get(subset) {
                // If we found it, we need to check for discontiguous matches
                // Determine how much further right to look
                let mut max_right = if (right + 2) < char_vals.len() {
                    right + 2
                } else if (right + 1) < char_vals.len() {
                    right + 1
                } else {
                    // This should skip the loop below. There will be no discontiguous match.
                    right
                };

                let mut try_two = max_right - right == 2 && cldr;

                'inner: while max_right > right {
                    // Need to make sure the sequence of CCCs is kosher
                    let interest_cohort = &char_vals[right..=max_right];
                    let mut max_ccc = 0;

                    for elem in interest_cohort {
                        let ccc = get_ccc(char::from_u32(*elem).unwrap()) as u8;
                        if ccc == 0 || ccc <= max_ccc {
                            // Can also forget about try_two in this case
                            try_two = false;
                            max_right -= 1;
                            continue 'inner;
                        }
                        max_ccc = ccc;
                    }

                    // Having made it this far, we can test a new subset, adding the later char(s)
                    let new_subset = if try_two {
                        [subset, &char_vals[max_right - 1..=max_right]].concat()
                    } else {
                        [subset, [char_vals[max_right]].as_slice()].concat()
                    };

                    // If the new subset is found in the table...
                    if let Some(new_value) = multis.get(&new_subset) {
                        // Then add these weights instead
                        for weights in new_value {
                            if shifting {
                                let weight_vals = get_shifted_weights(weights, last_variable);
                                cea.push(weight_vals);
                                if weights.variable {
                                    last_variable = true;
                                } else if weights.primary != 0 {
                                    last_variable = false;
                                }
                            } else {
                                let weight_vals =
                                    vec![weights.primary, weights.secondary, weights.tertiary];
                                cea.push(weight_vals);
                            }
                        }

                        // Remove the pulled char(s) (in this order!)
                        char_vals.remove(max_right);
                        if try_two {
                            char_vals.remove(max_right - 1);
                        }

                        // Increment and continue outer loop
                        left += right - left;
                        continue 'outer;
                    }

                    // If we tried for two, don't decrement max_right yet; inner loop will re-run
                    if try_two {
                        try_two = false;
                    } else {
                        // Otherwise decrement max_right; inner loop may re-run, or finish
                        max_right -= 1;
                    }
                }

                // At this point, we're not looking for a discontiguous match. We just need to push
                // the weights from the original subset we found.

                for weights in value {
                    if shifting {
                        let weight_vals = get_shifted_weights(weights, last_variable);
                        cea.push(weight_vals);
                        if weights.variable {
                            last_variable = true;
                        } else if weights.primary != 0 {
                            last_variable = false;
                        }
                    } else {
                        let weight_vals =
                            vec![weights.primary, weights.secondary, weights.tertiary];
                        cea.push(weight_vals);
                    }
                }

                // Increment and continue outer loop
                left += right - left;
                continue 'outer;
            }

            // Shorten slice to try again
            right -= 1;
        }

        // By now, we're looking for just one value, and it isn't in the table.
        // Time for implicit weights...

        let first_weights = get_implicit_a(left_val, shifting);
        cea.push(first_weights);

        let second_weights = get_implicit_b(left_val, shifting);
        cea.push(second_weights);

        // Finally, increment and let outer loop continue
        left += 1;
    }

    cea
}

fn get_shifted_weights(weights: &Weights, last_variable: bool) -> Vec<u16> {
    if weights.primary == 0 && weights.secondary == 0 && weights.tertiary == 0 {
        vec![0, 0, 0, 0]
    } else if weights.variable {
        vec![0, 0, 0, weights.primary]
    } else if last_variable && weights.primary == 0 && weights.tertiary != 0 {
        vec![0, 0, 0, 0]
    } else {
        vec![weights.primary, weights.secondary, weights.tertiary, 65_535]
    }
}

fn get_implicit_a(left_val: u32, shifting: bool) -> Vec<u16> {
    #[allow(clippy::manual_range_contains)]
    let mut aaaa = match left_val {
        x if x >= 13_312 && x <= 19_903 => 64_384 + (left_val >> 15), //     CJK2
        x if x >= 19_968 && x <= 40_959 => 64_320 + (left_val >> 15), //     CJK1
        x if x >= 63_744 && x <= 64_255 => 64_320 + (left_val >> 15), //     CJK1
        x if x >= 94_208 && x <= 101_119 => 64_256,                   //     Tangut
        x if x >= 101_120 && x <= 101_631 => 64_258,                  //     Khitan
        x if x >= 101_632 && x <= 101_775 => 64_256,                  //     Tangut
        x if x >= 110_960 && x <= 111_359 => 64_257,                  //     Nushu
        x if x >= 131_072 && x <= 173_791 => 64_384 + (left_val >> 15), //   CJK2
        x if x >= 173_824 && x <= 191_471 => 64_384 + (left_val >> 15), //   CJK2
        x if x >= 196_608 && x <= 201_551 => 64_384 + (left_val >> 15), //   CJK2
        _ => 64_448 + (left_val >> 15),                               //     unass.
    };

    if INCLUDED_UNASSIGNED.contains(&left_val) {
        aaaa = 64_448 + (left_val >> 15);
    }

    #[allow(clippy::cast_possible_truncation)]
    if shifting {
        // Add an arbitrary fourth weight if shifting
        vec![aaaa as u16, 32, 2, 65_535]
    } else {
        vec![aaaa as u16, 32, 2]
    }
}

fn get_implicit_b(left_val: u32, shifting: bool) -> Vec<u16> {
    #[allow(clippy::manual_range_contains)]
    let mut bbbb = match left_val {
        x if x >= 13_312 && x <= 19_903 => left_val & 32_767, //      CJK2
        x if x >= 19_968 && x <= 40_959 => left_val & 32_767, //      CJK1
        x if x >= 63_744 && x <= 64_255 => left_val & 32_767, //      CJK1
        x if x >= 94_208 && x <= 101_119 => left_val - 94_208, //     Tangut
        x if x >= 101_120 && x <= 101_631 => left_val - 101_120, //   Khitan
        x if x >= 101_632 && x <= 101_775 => left_val - 94_208, //    Tangut
        x if x >= 110_960 && x <= 111_359 => left_val - 110_960, //   Nushu
        x if x >= 131_072 && x <= 173_791 => left_val & 32_767, //    CJK2
        x if x >= 173_824 && x <= 191_471 => left_val & 32_767, //    CJK2
        x if x >= 196_608 && x <= 201_551 => left_val & 32_767, //    CJK2
        _ => left_val & 32_767,                               //      unass.
    };

    if INCLUDED_UNASSIGNED.contains(&left_val) {
        bbbb = left_val & 32_767;
    }

    // BBBB always gets bitwise ORed with this value
    bbbb |= 32_768;

    #[allow(clippy::cast_possible_truncation)]
    if shifting {
        // Add an arbitrary fourth weight if shifting
        vec![bbbb as u16, 0, 0, 65_535]
    } else {
        vec![bbbb as u16, 0, 0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conformance(path: &str, options: &CollationOptions) {
        let test_data = std::fs::read_to_string(path).unwrap();

        let mut max_sk: Vec<u16> = Vec::new();

        for line in test_data.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let hex_values: Vec<&str> = line.split(' ').collect();
            let mut test_string = String::new();

            for s in hex_values {
                let val = u32::from_str_radix(s, 16).unwrap();
                // We have to use an unsafe function for the conformance tests because they
                // deliberately introduce invalid character values.
                let c = unsafe { std::char::from_u32_unchecked(val) };
                test_string.push(c);
            }

            let nfd = get_nfd(&test_string);
            let sk = nfd_to_sk(nfd, options);

            let comparison = compare_sort_keys(&sk, &max_sk);
            if comparison == Ordering::Less {
                panic!();
            }

            max_sk = sk;
        }
    }

    #[test]
    fn ducet_non_ignorable() {
        let path = "test-data/CollationTest_NON_IGNORABLE_SHORT.txt";

        let options = CollationOptions {
            keys_source: KeysSource::Ducet,
            shifting: false,
        };

        conformance(path, &options);
    }

    #[test]
    fn ducet_shifted() {
        let path = "test-data/CollationTest_SHIFTED_SHORT.txt";

        let options = CollationOptions {
            keys_source: KeysSource::Ducet,
            shifting: true,
        };

        conformance(path, &options);
    }

    #[test]
    fn cldr_non_ignorable() {
        let path = "test-data/CollationTest_CLDR_NON_IGNORABLE_SHORT.txt";

        let options = CollationOptions {
            keys_source: KeysSource::Cldr,
            shifting: false,
        };

        conformance(path, &options);
    }

    #[test]
    fn cldr_shifted() {
        let path = "test-data/CollationTest_CLDR_SHIFTED_SHORT.txt";

        let options = CollationOptions {
            keys_source: KeysSource::Cldr,
            shifting: true,
        };

        conformance(path, &options);
    }
}
