use std::cmp::Ordering;
use tinyvec::ArrayVec;

pub fn compare_incremental(
    a_cea: &[ArrayVec<[u16; 4]>],
    b_cea: &[ArrayVec<[u16; 4]>],
    shifting: bool,
) -> Ordering {
    let a_len = a_cea.len();
    let b_len = b_cea.len();

    // Primary
    if let Some(o) = compare_at_lvl(a_cea, b_cea, a_len, b_len, 0) {
        return o;
    }

    // Secondary
    if let Some(o) = compare_at_lvl(a_cea, b_cea, a_len, b_len, 1) {
        return o;
    }

    // Tertiary
    if let Some(o) = compare_at_lvl(a_cea, b_cea, a_len, b_len, 2) {
        return o;
    }

    // If not shifting, stop here
    if !shifting {
        return Ordering::Equal;
    }

    // Quaternary
    if let Some(o) = compare_at_lvl(a_cea, b_cea, a_len, b_len, 3) {
        return o;
    }

    // If we got to this point, return Equal. The efficiency of processing and comparing sort keys
    // incrementally, for both strings at once, relies on the rarity of needing to continue all the
    // way through tertiary or quaternary weights. (Remember, there are two earlier fast paths for
    // equal strings -- one before normalization, one after.)
    Ordering::Equal
}

fn compare_at_lvl(
    a_cea: &[ArrayVec<[u16; 4]>],
    b_cea: &[ArrayVec<[u16; 4]>],
    a_len: usize,
    b_len: usize,
    lvl: usize,
) -> Option<Ordering> {
    let mut a_cursor = 0;
    let mut b_cursor = 0;

    loop {
        let mut a_weight: u16 = 0;
        let mut b_weight: u16 = 0;

        while a_cursor < a_len {
            if a_cea[a_cursor][lvl] != 0 {
                a_weight = a_cea[a_cursor][lvl];
                a_cursor += 1;
                break;
            }
            a_cursor += 1;
        }

        while b_cursor < b_len {
            if b_cea[b_cursor][lvl] != 0 {
                b_weight = b_cea[b_cursor][lvl];
                b_cursor += 1;
                break;
            }
            b_cursor += 1;
        }

        // This means no further weight at the given level was found in one of the strings
        if a_weight == 0 || b_weight == 0 {
            // If one of them did have another such weight, it wins; return that
            if a_weight != b_weight {
                return Some(a_weight.cmp(&b_weight));
            }
            // Else return None
            return None;
        }

        // If both weights are non-zero, and not equal, return their comparison
        if a_weight != b_weight {
            return Some(a_weight.cmp(&b_weight));
        }
    }
}
