use crate::consts::FCD;
use unicode_canonical_combining_class::get_canonical_combining_class as get_ccc;
use unicode_normalization::UnicodeNormalization;

pub fn get_nfd(input: &str) -> Vec<u32> {
    if fcd(input) {
        input.chars().map(|c| c as u32).collect()
    } else {
        UnicodeNormalization::nfd(input).map(|c| c as u32).collect()
    }
}

fn fcd(input: &str) -> bool {
    let mut c_as_u32: u32;
    let mut curr_lead_cc: u8;
    let mut curr_trail_cc: u8;

    let mut prev_trail_cc: u8 = 0;

    for c in input.chars() {
        c_as_u32 = c as u32;

        if c_as_u32 < 192 {
            prev_trail_cc = 0;
            continue;
        }

        if c_as_u32 == 3_969 || (44_032..=55_215).contains(&c_as_u32) {
            return false;
        }

        if let Some(vals) = FCD.get(&c_as_u32) {
            [curr_lead_cc, curr_trail_cc] = vals.to_be_bytes();
        } else {
            curr_lead_cc = get_ccc(c) as u8;
            curr_trail_cc = get_ccc(c) as u8;
        }

        if curr_lead_cc != 0 && curr_lead_cc < prev_trail_cc {
            return false;
        }

        prev_trail_cc = curr_trail_cc;
    }

    true
}
