use std::cmp::Ordering;

use crate::cea_utils::{get_tables, implicit_a};
use crate::consts::{LOW, LOW_CLDR, NEED_THREE, NEED_TWO};
use crate::weights::{primary, variability};
use crate::{Collator, Tailoring};

pub fn try_initial(coll: &Collator, a_chars: &[u32], b_chars: &[u32]) -> Option<Ordering> {
    let a_first = a_chars[0];
    let b_first = b_chars[0];

    if !safe_chars(a_first, b_first) {
        return None;
    }

    let a_first_primary = get_first_primary(a_first, coll);
    if a_first_primary == 0 {
        return None;
    }

    let b_first_primary = get_first_primary(b_first, coll);
    if b_first_primary == 0 || b_first_primary == a_first_primary {
        return None;
    }

    Some(a_first_primary.cmp(&b_first_primary))
}

fn safe_chars(a: u32, b: u32) -> bool {
    a != b
        && !NEED_TWO.contains(&a)
        && !NEED_TWO.contains(&b)
        && !NEED_THREE.contains(&a)
        && !NEED_THREE.contains(&b)
}

fn get_first_primary(val: u32, coll: &Collator) -> u16 {
    let cldr = coll.tailoring != Tailoring::Ducet;
    let shifting = coll.shifting;

    let low = if cldr { &LOW_CLDR } else { &LOW };

    // Fast path for low code points
    if val < 0xB7 && val != 0x6C && val != 0x4C {
        let weights = low[val as usize]; // Guaranteed to succeed

        if shifting && variability(weights) {
            return 0;
        }

        return primary(weights);
    }

    // Or look in the big table
    let (singles, _) = get_tables(coll.tailoring);

    if let Some(row) = singles.get(&val) {
        if shifting && variability(row[0]) {
            return 0;
        }

        return primary(row[0]);
    }

    // If all else failed, calculate implicit weights
    let first_weights = implicit_a(val);
    primary(first_weights)
}
