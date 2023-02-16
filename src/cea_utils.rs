use once_cell::sync::Lazy;
use unicode_canonical_combining_class::get_canonical_combining_class_u32 as get_ccc;

use crate::consts::{INCLUDED_UNASSIGNED, MULT, MULT_CLDR, SING, SING_CLDR};
use crate::tailor::{MULT_AR, SING_AR};
use crate::types::{MultisTable, SinglesTable};
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
    weights: u32,
    last_variable: &mut bool,
    cea: &mut Vec<u32>,
) {
    if shifting {
        cea.push(shift_weights(weights, last_variable));
    } else {
        cea.push(weights);
    }
}

pub fn implicit_a(cp: u32) -> u32 {
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
    pack_weights(false, aaaa as u16, 32, 2)
}

pub fn implicit_b(cp: u32) -> u32 {
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
    pack_weights(false, bbbb as u16, 0, 0)
}

pub const fn pack_weights(variable: bool, primary: u16, secondary: u16, tertiary: u16) -> u32 {
    let upper = (primary as u32) << 16;

    let v_int = variable as u16;
    let lower = (v_int << 15 | tertiary << 9) | secondary;

    upper | (lower as u32)
}

pub fn push_weights(cea: &mut Vec<u32>, row: &Vec<u32>, shifting: bool, last_variable: &mut bool) {
    if shifting {
        for weights in row {
            cea.push(shift_weights(*weights, last_variable));
        }
    } else {
        for weights in row {
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

pub fn shift_weights(weights: u32, last_variable: &mut bool) -> u32 {
    let (variable, primary, _, tertiary) = unpack_weights(weights);

    if variable {
        *last_variable = true;
        pack_weights(true, primary, 0, 0)
    } else if primary == 0 && (tertiary == 0 || *last_variable) {
        0
    } else {
        *last_variable = false;
        weights
    }
}

pub const fn unpack_weights(packed: u32) -> (bool, u16, u16, u16) {
    let primary = (packed >> 16) as u16;

    let lower = (packed & 0xFFFF) as u16;
    let variable = lower >> 15 == 1;
    let secondary = lower & 0b1_1111_1111;
    let tertiary = (lower >> 9) & 0b11_1111;

    (variable, primary, secondary, tertiary)
}
