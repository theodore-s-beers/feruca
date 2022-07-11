use crate::consts::INCLUDED_UNASSIGNED;
use crate::Weights;
use tinyvec::{array_vec, ArrayVec};

pub fn get_implicit_a(code_point: u32, shifting: bool) -> ArrayVec<[u16; 4]> {
    #[allow(clippy::manual_range_contains)]
    let mut aaaa = match code_point {
        x if x >= 13_312 && x <= 19_903 => 64_384 + (code_point >> 15), //     CJK2
        x if x >= 19_968 && x <= 40_959 => 64_320 + (code_point >> 15), //     CJK1
        x if x >= 63_744 && x <= 64_255 => 64_320 + (code_point >> 15), //     CJK1
        x if x >= 94_208 && x <= 101_119 => 64_256,                     //     Tangut
        x if x >= 101_120 && x <= 101_631 => 64_258,                    //     Khitan
        x if x >= 101_632 && x <= 101_775 => 64_256,                    //     Tangut
        x if x >= 110_960 && x <= 111_359 => 64_257,                    //     Nushu
        x if x >= 131_072 && x <= 173_791 => 64_384 + (code_point >> 15), //   CJK2
        x if x >= 173_824 && x <= 191_471 => 64_384 + (code_point >> 15), //   CJK2
        x if x >= 196_608 && x <= 201_551 => 64_384 + (code_point >> 15), //   CJK2
        _ => 64_448 + (code_point >> 15),                               //     unass.
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
    #[allow(clippy::manual_range_contains)]
    let mut bbbb = match code_point {
        x if x >= 13_312 && x <= 19_903 => code_point & 32_767, //      CJK2
        x if x >= 19_968 && x <= 40_959 => code_point & 32_767, //      CJK1
        x if x >= 63_744 && x <= 64_255 => code_point & 32_767, //      CJK1
        x if x >= 94_208 && x <= 101_119 => code_point - 94_208, //     Tangut
        x if x >= 101_120 && x <= 101_631 => code_point - 101_120, //   Khitan
        x if x >= 101_632 && x <= 101_775 => code_point - 94_208, //    Tangut
        x if x >= 110_960 && x <= 111_359 => code_point - 110_960, //   Nushu
        x if x >= 131_072 && x <= 173_791 => code_point & 32_767, //    CJK2
        x if x >= 173_824 && x <= 191_471 => code_point & 32_767, //    CJK2
        x if x >= 196_608 && x <= 201_551 => code_point & 32_767, //    CJK2
        _ => code_point & 32_767,                               //      unass.
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

pub(crate) fn get_shifted_weights(weights: Weights, last_variable: bool) -> ArrayVec<[u16; 4]> {
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
