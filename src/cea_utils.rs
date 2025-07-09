use std::sync::LazyLock;
use unicode_canonical_combining_class::get_canonical_combining_class_u32 as get_ccc;

use crate::consts::{INCLUDED_UNASSIGNED, MULT, MULT_CLDR, SING, SING_CLDR};
use crate::tailor::{MULT_AR, MULT_AR_I, SING_AR, SING_AR_I};
use crate::types::{MultisTable, SinglesTable};
use crate::weights::{pack_weights, shift_weights};
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

pub fn fill_weights(
    cea: &mut [u32],
    row: &[u32],
    i: &mut usize,
    shifting: bool,
    last_variable: &mut bool,
) {
    if shifting {
        for weights in row {
            cea[*i] = shift_weights(*weights, last_variable);
            *i += 1;
        }
    } else {
        for weights in row {
            cea[*i] = *weights;
            *i += 1;
        }
    }
}

pub fn get_tables(
    tailoring: Tailoring,
) -> (
    &'static LazyLock<SinglesTable>,
    &'static LazyLock<MultisTable>,
) {
    match tailoring {
        Tailoring::Cldr(Locale::ArabicScript) => (&SING_AR, &MULT_AR),
        Tailoring::Cldr(Locale::ArabicInterleaved) => (&SING_AR_I, &MULT_AR_I),
        Tailoring::Cldr(Locale::Root) => (&SING_CLDR, &MULT_CLDR),
        Tailoring::Ducet => (&SING, &MULT),
    }
}

pub fn grow_vec(cea: &mut Vec<u32>, i: usize) {
    let l = cea.len();

    // U+FDFA has 18 sets of collation weights!
    // We also need one space for the sentinel value, so 19 would do it...
    // But 20 is a nice round number.
    if l - i < 20 {
        cea.resize(l * 2, 0);
    }
}

pub fn handle_implicit_weights(cea: &mut [u32], cp: u32, i: &mut usize) {
    cea[*i] = implicit_a(cp);
    *i += 1;

    cea[*i] = implicit_b(cp);
    *i += 1;
}

pub fn handle_low_weights(
    cea: &mut [u32],
    weights: u32,
    i: &mut usize,
    shifting: bool,
    last_variable: &mut bool,
) {
    if shifting {
        cea[*i] = shift_weights(weights, last_variable);
    } else {
        cea[*i] = weights;
    }

    *i += 1;
}

pub fn implicit_a(cp: u32) -> u32 {
    let aaaa = if INCLUDED_UNASSIGNED.contains(&cp) {
        0xFBC0 + (cp >> 15)
    } else {
        match cp {
            0x3400..=0x4DBF | 0x20000..=0x2A6DF | 0x2A700..=0x2EE5D | 0x30000..=0x323AF => {
                0xFB80 + (cp >> 15)
            } // CJK2
            0x4E00..=0x9FFF | 0xF900..=0xFAFF => 0xFB40 + (cp >> 15), // CJK1
            0x17000..=0x18AFF | 0x18D00..=0x18D8F => 0xFB00,          // Tangut
            0x18B00..=0x18CFF => 0xFB02,                              // Khitan
            0x1B170..=0x1B2FF => 0xFB01,                              // Nushu
            _ => 0xFBC0 + (cp >> 15),                                 // unass.
        }
    };

    #[allow(clippy::cast_possible_truncation)]
    pack_weights(false, aaaa as u16, 32, 2)
}

pub fn implicit_b(cp: u32) -> u32 {
    let mut bbbb = if INCLUDED_UNASSIGNED.contains(&cp) {
        cp & 0x7FFF
    } else {
        match cp {
            0x17000..=0x18AFF | 0x18D00..=0x18D8F => cp - 0x17000, // Tangut
            0x18B00..=0x18CFF => cp - 0x18B00,                     // Khitan
            0x1B170..=0x1B2FF => cp - 0x1B170,                     // Nushu
            _ => cp & 0x7FFF,                                      // CJK1, CJK2, unass.
        }
    };

    // BBBB always gets bitwise ORed with this value
    bbbb |= 0x8000;

    #[allow(clippy::cast_possible_truncation)]
    pack_weights(false, bbbb as u16, 0, 0)
}

pub fn pack_code_points(code_points: &[u32]) -> u64 {
    match code_points.len() {
        2 => (u64::from(code_points[0]) << 21) | u64::from(code_points[1]),
        3 => {
            (u64::from(code_points[0]) << 42)
                | (u64::from(code_points[1]) << 21)
                | u64::from(code_points[2])
        }
        _ => unreachable!(),
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
