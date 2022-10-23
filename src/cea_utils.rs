use once_cell::sync::Lazy;
use tinyvec::{array_vec, ArrayVec};
use unicode_canonical_combining_class::get_canonical_combining_class_u32 as get_ccc;

use crate::cea::generate_cea;
use crate::consts::{INCLUDED_UNASSIGNED, MULT, MULT_CLDR, SING, SING_CLDR};
use crate::tailor::{MULT_AR, SING_AR};
use crate::types::{MultisTable, SinglesTable, Weights};
use crate::{Collator, Locale, Tailoring};

pub fn ccc_sequence_ok(interest_cohort: &[u32]) -> bool {
    let mut max_ccc = 0;

    for elem in interest_cohort {
        let ccc = get_ccc(*elem) as u8;

        if ccc == 0 || ccc <= max_ccc {
            return false;
        }

        max_ccc = ccc;
    }

    true
}

pub fn get_cea(word: &mut Vec<u32>, collator: &mut Collator) -> Vec<ArrayVec<[u16; 4]>> {
    if let Some(hit) = collator.get_cache(word) {
        return hit.clone();
    }

    let orig = word.clone();

    let cea = generate_cea(word, collator);
    collator.put_cache(orig, cea.clone());
    cea
}

pub fn get_implicit_a(cp: u32, shifting: bool) -> ArrayVec<[u16; 4]> {
    let aaaa = if INCLUDED_UNASSIGNED.contains(&cp) {
        64_448 + (cp >> 15)
    } else {
        match cp {
            x if (13_312..=19_903).contains(&x) => 64_384 + (cp >> 15), //     CJK2
            x if (19_968..=40_959).contains(&x) => 64_320 + (cp >> 15), //     CJK1
            x if (63_744..=64_255).contains(&x) => 64_320 + (cp >> 15), //     CJK1
            x if (94_208..=101_119).contains(&x) => 64_256,             //     Tangut
            x if (101_120..=101_631).contains(&x) => 64_258,            //     Khitan
            x if (101_632..=101_775).contains(&x) => 64_256,            //     Tangut
            x if (110_960..=111_359).contains(&x) => 64_257,            //     Nushu
            x if (131_072..=173_791).contains(&x) => 64_384 + (cp >> 15), //   CJK2
            x if (173_824..=191_471).contains(&x) => 64_384 + (cp >> 15), //   CJK2
            x if (196_608..=205_743).contains(&x) => 64_384 + (cp >> 15), //   CJK2
            _ => 64_448 + (cp >> 15),                                   //     unass.
        }
    };

    #[allow(clippy::cast_possible_truncation)]
    if shifting {
        // Add an arbitrary fourth weight if shifting
        ArrayVec::from([aaaa as u16, 32, 2, 65_535])
    } else {
        array_vec!([u16; 4] => aaaa as u16, 32, 2)
    }
}

pub fn get_implicit_b(cp: u32, shifting: bool) -> ArrayVec<[u16; 4]> {
    let mut bbbb = if INCLUDED_UNASSIGNED.contains(&cp) {
        cp & 32_767
    } else {
        match cp {
            x if (13_312..=19_903).contains(&x) => cp & 32_767, //      CJK2
            x if (19_968..=40_959).contains(&x) => cp & 32_767, //      CJK1
            x if (63_744..=64_255).contains(&x) => cp & 32_767, //      CJK1
            x if (94_208..=101_119).contains(&x) => cp - 94_208, //     Tangut
            x if (101_120..=101_631).contains(&x) => cp - 101_120, //   Khitan
            x if (101_632..=101_775).contains(&x) => cp - 94_208, //    Tangut
            x if (110_960..=111_359).contains(&x) => cp - 110_960, //   Nushu
            x if (131_072..=173_791).contains(&x) => cp & 32_767, //    CJK2
            x if (173_824..=191_471).contains(&x) => cp & 32_767, //    CJK2
            x if (196_608..=205_743).contains(&x) => cp & 32_767, //    CJK2
            _ => cp & 32_767,                                   //      unass.
        }
    };

    // BBBB always gets bitwise ORed with this value
    bbbb |= 32_768;

    #[allow(clippy::cast_possible_truncation)]
    if shifting {
        // Add an arbitrary fourth weight if shifting
        ArrayVec::from([bbbb as u16, 0, 0, 65_535])
    } else {
        array_vec!([u16; 4] => bbbb as u16, 0, 0)
    }
}

pub fn get_subset(
    try_two: bool,
    left_val: u32,
    char_vals: &[u32],
    max_right: usize,
) -> ArrayVec<[u32; 3]> {
    if try_two {
        ArrayVec::from([left_val, char_vals[max_right - 1], char_vals[max_right]])
    } else {
        array_vec!([u32; 3] => left_val, char_vals[max_right])
    }
}

pub fn get_table_multis(tailoring: Tailoring) -> &'static Lazy<MultisTable> {
    match tailoring {
        Tailoring::Cldr(Locale::ArabicScript) => &MULT_AR,
        Tailoring::Cldr(Locale::Root) => &MULT_CLDR,
        Tailoring::Ducet => &MULT,
    }
}

pub fn get_table_singles(tailoring: Tailoring) -> &'static Lazy<SinglesTable> {
    match tailoring {
        Tailoring::Cldr(Locale::ArabicScript) => &SING_AR,
        Tailoring::Cldr(Locale::Root) => &SING_CLDR,
        Tailoring::Ducet => &SING,
    }
}

pub fn handle_implicit_weights(left_val: u32, shifting: bool, cea: &mut Vec<ArrayVec<[u16; 4]>>) {
    let first_weights = get_implicit_a(left_val, shifting);
    cea.push(first_weights);

    let second_weights = get_implicit_b(left_val, shifting);
    cea.push(second_weights);
}

pub fn handle_low_weights(
    shifting: bool,
    weights: Weights,
    last_variable: &mut bool,
    cea: &mut Vec<ArrayVec<[u16; 4]>>,
) {
    if shifting {
        let weight_vals = handle_shifted_weights(weights, last_variable);
        cea.push(weight_vals);
    } else {
        let weight_vals = array_vec!(
            [u16; 4] => weights.primary, weights.secondary, weights.tertiary
        );
        cea.push(weight_vals);
    }
}

pub fn handle_shifted_weights(weights: Weights, last_variable: &mut bool) -> ArrayVec<[u16; 4]> {
    if weights.variable {
        *last_variable = true;
        ArrayVec::from([0, 0, 0, weights.primary])
    } else if weights.primary == 0 && (weights.tertiary == 0 || *last_variable) {
        ArrayVec::from([0, 0, 0, 0])
    } else {
        *last_variable = false;
        ArrayVec::from([weights.primary, weights.secondary, weights.tertiary, 65_535])
    }
}

pub fn push_weights(
    row: &Vec<Weights>,
    shifting: bool,
    last_variable: &mut bool,
    cea: &mut Vec<ArrayVec<[u16; 4]>>,
) {
    for weights in row {
        if shifting {
            let weight_vals = handle_shifted_weights(*weights, last_variable);
            cea.push(weight_vals);
        } else {
            let weight_vals = array_vec!(
                [u16; 4] => weights.primary, weights.secondary, weights.tertiary
            );
            cea.push(weight_vals);
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
