use crate::cea_utils::{get_implicit_a, get_shifted_weights};
use crate::consts::{LOW, LOW_CLDR, SING, SING_CLDR};
use crate::consts::{NEED_THREE, NEED_TWO};
use crate::{Collator, KeysSource};

pub fn safe_first_chars(a_chars: &[u32], b_chars: &[u32]) -> bool {
    a_chars[0] != b_chars[0]
        && !NEED_TWO.contains(&a_chars[0])
        && !NEED_TWO.contains(&b_chars[0])
        && !NEED_THREE.contains(&a_chars[0])
        && !NEED_THREE.contains(&b_chars[0])
}

pub fn get_first_primary(val: u32, collator: Collator) -> u16 {
    let cldr = collator.keys_source == KeysSource::Cldr;
    let shifting = collator.shifting;

    let low = if cldr { &LOW_CLDR } else { &LOW };
    let singles = if cldr { &SING_CLDR } else { &SING };

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
