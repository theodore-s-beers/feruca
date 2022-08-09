use crate::cea_utils::{get_implicit_a, get_shifted_weights, get_table_singles};
use crate::consts::{LOW, LOW_CLDR, NEED_THREE, NEED_TWO};
use crate::{Collator, Tailoring};
use std::cmp::Ordering;

pub fn try_initial(a_chars: &[u32], b_chars: &[u32], coll: &Collator) -> Option<Ordering> {
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
    let singles = get_table_singles(coll.tailoring);

    // Fast path for low code points
    if val < 183 && val != 108 && val != 76 {
        let weights = low[&val]; // Guaranteed to succeed

        if shifting {
            let weight_vals = get_shifted_weights(weights, false);
            return weight_vals[0];
        }

        return weights.primary;
    }

    // Or look in the big table
    if let Some(row) = singles.get(&val) {
        if shifting {
            let weight_vals = get_shifted_weights(row[0], false);
            return weight_vals[0];
        }

        return row[0].primary;
    }

    // If all else failed, calculate implicit weights
    let first_weights = get_implicit_a(val, shifting);
    first_weights[0]
}
