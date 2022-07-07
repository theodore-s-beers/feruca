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
// Consts
//

static ALLKEYS: Lazy<HashMap<Vec<u32>, Vec<Weights>>> = Lazy::new(|| {
    let data = include_bytes!("bincode/allkeys_14");
    let decoded: HashMap<Vec<u32>, Vec<Weights>> = bincode::deserialize(&data[..]).unwrap();
    decoded
});

static ALLKEYS_CLDR: Lazy<HashMap<Vec<u32>, Vec<Weights>>> = Lazy::new(|| {
    let data = include_bytes!("bincode/allkeys_cldr_14");
    let decoded: HashMap<Vec<u32>, Vec<Weights>> = bincode::deserialize(&data[..]).unwrap();
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
// Functions
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
pub fn collate(str_a: &str, str_b: &str, options: &CollationOptions) -> Ordering {
    let sort_key_1 = str_to_sort_key(str_a, options);
    let sort_key_2 = str_to_sort_key(str_b, options);

    let comparison = compare_sort_keys(&sort_key_1, &sort_key_2);

    if comparison == Ordering::Equal {
        // Tiebreaker
        return str_a.cmp(str_b);
    }

    comparison
}

fn compare_sort_keys(a: &[u16], b: &[u16]) -> Ordering {
    let min_sort_key_length = a.len().min(b.len());

    for i in 0..min_sort_key_length {
        if a[i] < b[i] {
            return Ordering::Less;
        }

        if a[i] > b[i] {
            return Ordering::Greater;
        }
    }

    Ordering::Equal
}

fn str_to_sort_key(input: &str, options: &CollationOptions) -> Vec<u16> {
    let char_values = get_char_values(input);
    let collation_element_array = get_collation_element_array(char_values, options);
    get_sort_key(&collation_element_array, options.shifting)
}

fn get_char_values(input: &str) -> Vec<u32> {
    UnicodeNormalization::nfd(input).map(|c| c as u32).collect()
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

// This is where the magic happens (or the sausage is made?)
#[allow(clippy::too_many_lines)]
fn get_collation_element_array(
    mut char_values: Vec<u32>,
    options: &CollationOptions,
) -> Vec<Vec<u16>> {
    let keys = match options.keys_source {
        KeysSource::Cldr => &ALLKEYS_CLDR,
        KeysSource::Ducet => &ALLKEYS,
    };

    let cldr = options.keys_source == KeysSource::Cldr;
    let shifting = options.shifting;

    let mut collation_element_array: Vec<Vec<u16>> = Vec::new();

    let mut left: usize = 0;
    let mut last_variable = false;

    'outer: while left < char_values.len() {
        let left_val = char_values[left];

        // Set lookahead depending on left_val. We need 3 in a few cases; 2 in several dozen cases;
        // and 1 otherwise.
        let lookahead: usize = match left_val {
            x if NEED_THREE.contains(&x) => 3,
            x if NEED_TWO.contains(&x) => 2,
            _ => 1,
        };

        // But don't look past the end of the vec
        let mut right = if left + lookahead > char_values.len() {
            char_values.len()
        } else {
            left + lookahead
        };

        while right > left {
            let subset = &char_values[left..right];

            if let Some(value) = keys.get(subset) {
                // This means we've found "the longest initial substring S at [this] point that has
                // a match in the collation element table." Next we check for "non-starters" that
                // follow this substring.
                //
                // The idea is that there could be multiple non-starters in a row, not blocking one
                // another, such that we could skip over one (or more) to make a longer substring
                // that has a match in the table.
                //
                // One example comes from the test string "0438 0306 0334." NFD normalization will
                // reorder that to "0438 0334 0306." This causes a problem, since 0438 and 0306 can
                // go together, but we'll miss it if we don't look past 0334.

                let mut max_right = if (right + 2) < char_values.len() {
                    right + 2
                } else if (right + 1) < char_values.len() {
                    right + 1
                } else {
                    // This should skip the loop below
                    right
                };

                let mut try_two = max_right - right == 2 && cldr;

                'inner: while max_right > right {
                    // We verify that all chars in the range right..=max_right are non-starters
                    // If there are any starters in our range of interest, decrement and continue
                    // The CCCs also have to be increasing, apparently...

                    let interest_cohort = &char_values[right..=max_right];
                    let mut max_ccc = 0;

                    for elem in interest_cohort {
                        let ccc = get_ccc(char::from_u32(*elem).unwrap()) as u8;
                        if ccc == 0 || ccc <= max_ccc {
                            // Decrement and continue
                            // Can also forget about try_two in this case
                            try_two = false;
                            max_right -= 1;
                            continue 'inner;
                        }
                        max_ccc = ccc;
                    }

                    // Having made it this far, we can test a new subset, adding the later char(s)
                    let new_subset = if try_two {
                        [subset, &char_values[max_right - 1..=max_right]].concat()
                    } else {
                        [subset, [char_values[max_right]].as_slice()].concat()
                    };

                    // If the new subset is found in the table...
                    if let Some(new_value) = keys.get(&new_subset) {
                        // Then add these weights instead
                        for weights in new_value {
                            if shifting {
                                // Variable shifting means all weight vectors will have a fourth
                                // value

                                // If all weights were already 0, make the fourth 0
                                if weights.primary == 0
                                    && weights.secondary == 0
                                    && weights.tertiary == 0
                                {
                                    let weight_values = vec![0, 0, 0, 0];
                                    collation_element_array.push(weight_values);

                                // If these weights are marked variable...
                                } else if weights.variable {
                                    let weight_values = vec![0, 0, 0, weights.primary];
                                    collation_element_array.push(weight_values);
                                    last_variable = true;

                                // If these are "ignorable" weights and follow something variable...
                                } else if last_variable
                                    && weights.primary == 0
                                    && weights.tertiary != 0
                                {
                                    let weight_values = vec![0, 0, 0, 0];
                                    collation_element_array.push(weight_values);

                                // Otherwise it can be assumed that we're dealing with something
                                // non-ignorable, or ignorable but not following something variable
                                } else {
                                    let weight_values = vec![
                                        weights.primary,
                                        weights.secondary,
                                        weights.tertiary,
                                        65_535,
                                    ];
                                    collation_element_array.push(weight_values);
                                    last_variable = false;
                                }
                            } else {
                                // If not shifting, we can just push the weights and be done
                                let weight_values =
                                    vec![weights.primary, weights.secondary, weights.tertiary];
                                collation_element_array.push(weight_values);
                            }
                        }

                        // Remove the pulled char(s) (in this order!)
                        char_values.remove(max_right);
                        if try_two {
                            char_values.remove(max_right - 1);
                        }

                        // Increment and continue outer loop
                        left += right - left;
                        continue 'outer;
                    }

                    // If we tried for two, don't decrement max_right yet
                    if try_two {
                        try_two = false;
                    } else {
                        max_right -= 1;
                    }
                }

                // At this point, we're not looking for a discontiguous match. We just need to push
                // the weights from the original subset we found

                for weights in value {
                    if shifting {
                        // Variable shifting means all weight vectors will have a fourth value

                        // If all weights were already 0, make the fourth 0
                        if weights.primary == 0 && weights.secondary == 0 && weights.tertiary == 0 {
                            let weight_values = vec![0, 0, 0, 0];
                            collation_element_array.push(weight_values);

                        // If these weights are marked variable...
                        } else if weights.variable {
                            let weight_values = vec![0, 0, 0, weights.primary];
                            collation_element_array.push(weight_values);
                            last_variable = true;

                        // If these are "ignorable" weights and follow something variable...
                        } else if last_variable && weights.primary == 0 && weights.tertiary != 0 {
                            let weight_values = vec![0, 0, 0, 0];
                            collation_element_array.push(weight_values);

                        // Otherwise it can be assumed that we're dealing with something non-
                        // ignorable, or ignorable but not following something variable
                        } else {
                            let weight_values =
                                vec![weights.primary, weights.secondary, weights.tertiary, 65_535];
                            collation_element_array.push(weight_values);
                            last_variable = false;
                        }
                    } else {
                        // If not shifting, we can just push weights and be done
                        let weight_values =
                            vec![weights.primary, weights.secondary, weights.tertiary];
                        collation_element_array.push(weight_values);
                    }
                }

                // Increment and continue outer loop
                left += right - left;
                continue 'outer;
            }

            // Shorten slice to try again
            right -= 1;
        }

        // By now, we're looking for just one value, and it isn't in the table
        // Time for implicit weights...

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

        // One of the above ranges seems to include some unassigned code points. In order to pass
        // the conformance tests, I'm adding an extra check here. This doesn't feel like a good way
        // of dealing with the problem, but I haven't yet found a better approach that doesn't come
        // with its own downsides.

        if INCLUDED_UNASSIGNED.contains(&left_val) {
            aaaa = 64_448 + (left_val >> 15);
            bbbb = left_val & 32_767;
        }

        // BBBB always gets bitwise ORed with this value
        bbbb |= 32_768;

        #[allow(clippy::cast_possible_truncation)]
        let first_weights = if shifting {
            // Add an arbitrary fourth weight if shifting
            vec![aaaa as u16, 32, 2, 65_535]
        } else {
            vec![aaaa as u16, 32, 2]
        };
        collation_element_array.push(first_weights);

        #[allow(clippy::cast_possible_truncation)]
        let second_weights = if shifting {
            // Add an arbitrary fourth weight if shifting
            vec![bbbb as u16, 0, 0, 65_535]
        } else {
            vec![bbbb as u16, 0, 0]
        };
        collation_element_array.push(second_weights);

        // Finally, increment and let outer loop continue
        left += 1;
    }

    collation_element_array
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

            let sk = str_to_sort_key(&test_string, options);

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
