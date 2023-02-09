use once_cell::sync::Lazy;
use unicode_canonical_combining_class::get_canonical_combining_class_u32 as get_ccc;

use crate::consts::{INCLUDED_UNASSIGNED, MULT, MULT_CLDR, SING, SING_CLDR};
use crate::tailor::{MULT_AR, SING_AR};
use crate::types::{MultisTable, SinglesTable, Weights};
use crate::{Locale, Tailoring};

pub fn ccc_sequence_ok(test_range: &[u32]) -> bool {
    let mut max_ccc = 0;

    for elem in test_range {
        let ccc = get_ccc(*elem) as u8;

        if ccc == 0 || ccc <= max_ccc {
            return false;
        }

        max_ccc = ccc;
    }

    true
}

pub fn get_implicit_a(cp: u32, shifting: bool) -> Weights {
    let aaaa = if INCLUDED_UNASSIGNED.contains(&cp) {
        64_448 + (cp >> 15)
    } else {
        match cp {
            13_312..=19_903 | 131_072..=173_791 | 173_824..=191_471 | 196_608..=205_743 => {
                64_384 + (cp >> 15)
            } // CJK2
            19_968..=40_959 | 63_744..=64_255 => 64_320 + (cp >> 15), // CJK1
            94_208..=101_119 | 101_632..=101_775 => 64_256,           // Tangut
            101_120..=101_631 => 64_258,                              // Khitan
            110_960..=111_359 => 64_257,                              // Nushu
            _ => 64_448 + (cp >> 15),                                 // unass.
        }
    };

    #[allow(clippy::cast_possible_truncation)]
    if shifting {
        Weights {
            variable: false,
            primary: aaaa as u16,
            secondary: 32,
            tertiary: 2,
            quaternary: Some(65_535), // Arbitrary high fourth weight
        }
    } else {
        Weights {
            variable: false,
            primary: aaaa as u16,
            secondary: 32,
            tertiary: 2,
            quaternary: None,
        }
    }
}

pub fn get_implicit_b(cp: u32, shifting: bool) -> Weights {
    let mut bbbb = if INCLUDED_UNASSIGNED.contains(&cp) {
        cp & 32_767
    } else {
        match cp {
            94_208..=101_119 | 101_632..=101_775 => cp - 94_208, // Tangut
            101_120..=101_631 => cp - 101_120,                   // Khitan
            110_960..=111_359 => cp - 110_960,                   // Nushu
            _ => cp & 32_767,                                    // CJK1, CJK2, unass.
        }
    };

    // BBBB always gets bitwise ORed with this value
    bbbb |= 32_768;

    #[allow(clippy::cast_possible_truncation)]
    if shifting {
        Weights {
            variable: false,
            primary: bbbb as u16,
            secondary: 0,
            tertiary: 0,
            quaternary: Some(65_535), // Arbitrary high fourth weight
        }
    } else {
        Weights {
            variable: false,
            primary: bbbb as u16,
            secondary: 0,
            tertiary: 0,
            quaternary: None,
        }
    }
}

pub fn get_tables(
    tailoring: Tailoring,
) -> (&'static Lazy<SinglesTable>, &'static Lazy<MultisTable>) {
    match tailoring {
        Tailoring::Cldr(Locale::ArabicScript) => (&SING_AR, &MULT_AR),
        Tailoring::Cldr(Locale::Root) => (&SING_CLDR, &MULT_CLDR),
        Tailoring::Ducet => (&SING, &MULT),
    }
}

pub fn handle_low_weights(
    shifting: bool,
    weights: Weights,
    last_variable: &mut bool,
    cea: &mut Vec<Weights>,
) {
    if shifting {
        cea.push(handle_shifted_weights(weights, last_variable));
    } else {
        cea.push(weights);
    }
}

pub fn handle_shifted_weights(weights: Weights, last_variable: &mut bool) -> Weights {
    if weights.variable {
        *last_variable = true;

        Weights {
            variable: true,
            primary: 0,
            secondary: 0,
            tertiary: 0,
            quaternary: Some(weights.primary), // This is apparently always non-zero
        }
    } else if weights.primary == 0 && (weights.tertiary == 0 || *last_variable) {
        Weights {
            variable: false,
            primary: 0,
            secondary: 0,
            tertiary: 0,
            quaternary: None,
        }
    } else {
        *last_variable = false;

        Weights {
            variable: false,
            primary: weights.primary,
            secondary: weights.secondary,
            tertiary: weights.tertiary,
            quaternary: Some(65_535),
        }
    }
}

pub fn push_weights(
    row: &Vec<Weights>,
    shifting: bool,
    last_variable: &mut bool,
    cea: &mut Vec<Weights>,
) {
    for weights in row {
        if shifting {
            cea.push(handle_shifted_weights(*weights, last_variable));
        } else {
            cea.push(*weights);
        }
    }
}

pub fn remove_pulled(char_vals: &mut Vec<u32>, i: usize, input_length: &mut usize, try_two: bool) {
    char_vals.remove(i);
    *input_length -= 1;

    if try_two {
        char_vals.remove(i - 1);
        *input_length -= 1;
    }
}
