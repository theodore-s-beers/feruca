use crate::consts::{INCLUDED_UNASSIGNED, MULT, MULT_CLDR, SING, SING_CLDR};
use crate::tailor::{MULT_AR, SING_AR};
use crate::types::{MultisTable, SinglesTable, Weights};
use crate::{Locale, Tailoring};
use once_cell::sync::Lazy;
use tinyvec::{array_vec, ArrayVec};

pub fn get_implicit_a(code_point: u32, shifting: bool) -> ArrayVec<[u16; 4]> {
    let mut aaaa = match code_point {
        x if (13_312..=19_903).contains(&x) => 64_384 + (code_point >> 15), //     CJK2
        x if (19_968..=40_959).contains(&x) => 64_320 + (code_point >> 15), //     CJK1
        x if (63_744..=64_255).contains(&x) => 64_320 + (code_point >> 15), //     CJK1
        x if (94_208..=101_119).contains(&x) => 64_256,                     //     Tangut
        x if (101_120..=101_631).contains(&x) => 64_258,                    //     Khitan
        x if (101_632..=101_775).contains(&x) => 64_256,                    //     Tangut
        x if (110_960..=111_359).contains(&x) => 64_257,                    //     Nushu
        x if (131_072..=173_791).contains(&x) => 64_384 + (code_point >> 15), //   CJK2
        x if (173_824..=191_471).contains(&x) => 64_384 + (code_point >> 15), //   CJK2
        x if (196_608..=201_551).contains(&x) => 64_384 + (code_point >> 15), //   CJK2
        _ => 64_448 + (code_point >> 15),                                   //     unass.
    };

    if INCLUDED_UNASSIGNED.contains(&code_point) {
        aaaa = 64_448 + (code_point >> 15);
    }

    #[allow(clippy::cast_possible_truncation)]
    if shifting {
        // Add an arbitrary fourth weight if shifting
        ArrayVec::from([aaaa as u16, 32, 2, 65_535])
    } else {
        array_vec!([u16; 4] => aaaa as u16, 32, 2)
    }
}

pub fn get_implicit_b(code_point: u32, shifting: bool) -> ArrayVec<[u16; 4]> {
    let mut bbbb = match code_point {
        x if (13_312..=19_903).contains(&x) => code_point & 32_767, //      CJK2
        x if (19_968..=40_959).contains(&x) => code_point & 32_767, //      CJK1
        x if (63_744..=64_255).contains(&x) => code_point & 32_767, //      CJK1
        x if (94_208..=101_119).contains(&x) => code_point - 94_208, //     Tangut
        x if (101_120..=101_631).contains(&x) => code_point - 101_120, //   Khitan
        x if (101_632..=101_775).contains(&x) => code_point - 94_208, //    Tangut
        x if (110_960..=111_359).contains(&x) => code_point - 110_960, //   Nushu
        x if (131_072..=173_791).contains(&x) => code_point & 32_767, //    CJK2
        x if (173_824..=191_471).contains(&x) => code_point & 32_767, //    CJK2
        x if (196_608..=201_551).contains(&x) => code_point & 32_767, //    CJK2
        _ => code_point & 32_767,                                   //      unass.
    };

    if INCLUDED_UNASSIGNED.contains(&code_point) {
        bbbb = code_point & 32_767;
    }

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

pub fn get_shifted_weights(weights: Weights, last_variable: bool) -> ArrayVec<[u16; 4]> {
    if weights.primary == 0 && weights.secondary == 0 && weights.tertiary == 0 {
        ArrayVec::from([0, 0, 0, 0])
    } else if weights.variable {
        ArrayVec::from([0, 0, 0, weights.primary])
    } else if last_variable && weights.primary == 0 && weights.tertiary != 0 {
        ArrayVec::from([0, 0, 0, 0])
    } else {
        ArrayVec::from([weights.primary, weights.secondary, weights.tertiary, 65_535])
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
