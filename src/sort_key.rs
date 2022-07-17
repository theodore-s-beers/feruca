use std::cmp::Ordering;
use tinyvec::ArrayVec;

// I'll compress this function later; I just got it working
#[allow(clippy::too_many_lines)]
pub fn compare_incremental(
    a_cea: &[ArrayVec<[u16; 4]>],
    b_cea: &[ArrayVec<[u16; 4]>],
    shifting: bool,
) -> Ordering {
    let a_len = a_cea.len();
    let b_len = b_cea.len();

    let mut a_cursor = 0;
    let mut b_cursor = 0;

    loop {
        let mut a_prim: u16 = 0;
        let mut b_prim: u16 = 0;

        while a_cursor < a_len {
            if a_cea[a_cursor][0] != 0 {
                a_prim = a_cea[a_cursor][0];
                a_cursor += 1;
                break;
            }
            a_cursor += 1;
        }

        while b_cursor < b_len {
            if b_cea[b_cursor][0] != 0 {
                b_prim = b_cea[b_cursor][0];
                b_cursor += 1;
                break;
            }
            b_cursor += 1;
        }

        // This means no further primary weight was found in one of the strings
        if a_prim == 0 || b_prim == 0 {
            // If one of them did have another primary weight, it wins; return
            if a_prim != b_prim {
                return a_prim.cmp(&b_prim);
            }
            // Else break the primary weight loop
            break;
        }

        // If both weights are non-zero, and not equal, return their comparison
        if a_prim != b_prim {
            return a_prim.cmp(&b_prim);
        }
    }

    // Reset cursors
    a_cursor = 0;
    b_cursor = 0;

    loop {
        let mut a_sec: u16 = 0;
        let mut b_sec: u16 = 0;

        while a_cursor < a_len {
            if a_cea[a_cursor][1] != 0 {
                a_sec = a_cea[a_cursor][1];
                a_cursor += 1;
                break;
            }
            a_cursor += 1;
        }

        while b_cursor < b_len {
            if b_cea[b_cursor][1] != 0 {
                b_sec = b_cea[b_cursor][1];
                b_cursor += 1;
                break;
            }
            b_cursor += 1;
        }

        // Same logic, but for secondary weights

        if a_sec == 0 || b_sec == 0 {
            if a_sec != b_sec {
                return a_sec.cmp(&b_sec);
            }
            break;
        }

        if a_sec != b_sec {
            return a_sec.cmp(&b_sec);
        }
    }

    // Reset cursors
    a_cursor = 0;
    b_cursor = 0;

    loop {
        let mut a_ter: u16 = 0;
        let mut b_ter: u16 = 0;

        while a_cursor < a_len {
            if a_cea[a_cursor][2] != 0 {
                a_ter = a_cea[a_cursor][2];
                a_cursor += 1;
                break;
            }
            a_cursor += 1;
        }

        while b_cursor < b_len {
            if b_cea[b_cursor][2] != 0 {
                b_ter = b_cea[b_cursor][2];
                b_cursor += 1;
                break;
            }
            b_cursor += 1;
        }

        // Same logic, but for tertiary weights

        if a_ter == 0 || b_ter == 0 {
            if a_ter != b_ter {
                return a_ter.cmp(&b_ter);
            }
            break;
        }

        if a_ter != b_ter {
            return a_ter.cmp(&b_ter);
        }
    }

    // If not shifting, stop here
    if !shifting {
        return Ordering::Equal;
    }

    // Reset cursors
    a_cursor = 0;
    b_cursor = 0;

    loop {
        let mut a_quat: u16 = 0;
        let mut b_quat: u16 = 0;

        while a_cursor < a_len {
            if a_cea[a_cursor][3] != 0 {
                a_quat = a_cea[a_cursor][3];
                a_cursor += 1;
                break;
            }
            a_cursor += 1;
        }

        while b_cursor < b_len {
            if b_cea[b_cursor][3] != 0 {
                b_quat = b_cea[b_cursor][3];
                b_cursor += 1;
                break;
            }
            b_cursor += 1;
        }

        // Same logic, but for quaternary weights

        if a_quat == 0 || b_quat == 0 {
            if a_quat != b_quat {
                return a_quat.cmp(&b_quat);
            }
            break;
        }

        if a_quat != b_quat {
            return a_quat.cmp(&b_quat);
        }
    }

    // If we got to this point, return Equal. The efficiency of processing and comparing sort keys
    // incrementally, for both strings at once, relies on the rarity of needing to continue all the
    // way through tertiary or quaternary weights. (Remember, there are two earlier fast paths for
    // equal strings -- one before normalization, one after.)
    Ordering::Equal
}
