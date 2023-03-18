#![allow(clippy::similar_names)]

use unicode_canonical_combining_class::get_canonical_combining_class_u32 as get_ccc;

use crate::cea_utils::{
    ccc_sequence_ok, fill_weights, get_tables, grow_vec, handle_implicit_weights,
    handle_low_weights, remove_pulled,
};
use crate::consts::{LOW, LOW_CLDR, NEED_THREE, NEED_TWO};
use crate::Tailoring;

pub fn generate_cea(
    cea: &mut Vec<u32>,
    char_vals: &mut Vec<u32>,
    shifting: bool,
    tailoring: Tailoring,
) {
    let mut input_length = char_vals.len();

    let cldr = tailoring != Tailoring::Ducet;
    let low = if cldr { &LOW_CLDR } else { &LOW };
    let (singles, multis) = get_tables(tailoring);

    let mut left: usize = 0;
    let mut cea_idx: usize = 0;
    let mut last_variable = false;

    // We spend essentially the entire function in this loop
    'outer: while left < input_length {
        let left_val = char_vals[left];

        grow_vec(cea, cea_idx);

        //
        // OUTCOME 1
        //
        // The code point was low, so we could draw from a small map that associates one u32 with
        // one set of weights. Then we fill in the weights, shifting if necessary. This is the path
        // that catches (most) ASCII characters present in not-completely-ASCII strings.
        //
        if left_val < 0x00B7 && left_val != 0x006C && left_val != 0x004C {
            let weights = low[&left_val]; // Guaranteed to succeed
            handle_low_weights(cea, weights, &mut cea_idx, shifting, &mut last_variable);
            left += 1;
            continue; // To the next outer loop iteration...
        }

        // At this point, we aren't dealing with a low code point

        // Set lookahead depending on left_val. We need 3 in a few cases; 2 in several dozen cases;
        // and 1 otherwise.
        let lookahead: usize = match left_val {
            x if NEED_THREE.contains(&x) => 3,
            x if NEED_TWO.contains(&x) => 2,
            _ => 1,
        };

        // If lookahead is 1, or if this is the last item in the vec, we'll take an easier path
        let check_multi = lookahead > 1 && (input_length - left > 1);

        if !check_multi {
            //
            // OUTCOME 2
            //
            // We only had to check for a single code point, and found it, so we can fill in the
            // weights and continue. This is a relatively fast path.
            //
            if let Some(row) = singles.get(&left_val) {
                fill_weights(cea, row, &mut cea_idx, shifting, &mut last_variable);
                left += 1;
                continue; // To the next outer loop iteration...
            }

            //
            // OUTCOME 3
            //
            // We checked for a single code point and didn't find it. That means it's unlisted. We
            // then calculate implicit weights, push them, and move on. I used to think there were
            // multiple paths to the "implicit weights" case, but it seems not.
            //
            handle_implicit_weights(cea, left_val, &mut cea_idx);

            left += 1;
            continue; // To the next outer loop iteration...
        }

        // Here we consider multi-code-point matches, if possible

        // Don't look past the end of the vec
        let mut right = input_length.min(left + lookahead);

        while right > left {
            if right - left == 1 {
                // If right - left == 1 (which cannot be the case in the first iteration), attempts
                // to find a multi-code-point match have failed. So we pull the value(s) for the
                // first code point from the singles map. It's guaranteed to be there.
                let row = &singles[&left_val];

                // If we found it, we do still need to check for discontiguous matches
                // Determine how much further right to look
                let mut max_right = match input_length - right {
                    3.. => right + 2,
                    2 => right + 1,
                    _ => right, // Skip the loop below; there will be no discontiguous match
                };

                let mut try_two = (max_right - right == 2) && cldr;

                while max_right > right {
                    // Make sure the sequence of CCC values is kosher
                    let test_range = &char_vals[right..=max_right];

                    if !ccc_sequence_ok(test_range) {
                        try_two = false; // Can forget about try_two in this case
                        max_right -= 1;
                        continue;
                    }

                    // Having made it this far, we can test a new subset, adding later char(s)
                    let new_subset = if try_two {
                        vec![left_val, char_vals[max_right - 1], char_vals[max_right]]
                    } else {
                        vec![left_val, char_vals[max_right]]
                    };

                    //
                    // OUTCOME 6
                    //
                    // We found a discontiguous match after a single code point. This is a bad path,
                    // since it implies that we: checked for a multi-code-point match; didn't find
                    // one; fell back to the initial code point; checked for discontiguous matches;
                    // and found something. Anyway, fill in the weights...
                    //
                    if let Some(new_row) = multis.get(&new_subset) {
                        fill_weights(cea, new_row, &mut cea_idx, shifting, &mut last_variable);

                        // Remove the later char(s) used for the discontiguous match
                        remove_pulled(char_vals, max_right, &mut input_length, try_two);

                        left += 1;
                        continue 'outer;
                    }

                    // If we tried for two, don't decrement max_right yet; inner loop will re-run
                    if try_two {
                        try_two = false;
                    } else {
                        max_right -= 1; // Otherwise decrement; inner loop *may* re-run
                    }
                }

                //
                // OUTCOME 7
                //
                // We checked for a multi-code-point match; failed to find one; fell back to the
                // initial code point; possibly checked for discontiguous matches; and, if so, did
                // not find any. This can be the worst path. Fill in the weights...
                //
                fill_weights(cea, row, &mut cea_idx, shifting, &mut last_variable);
                left += 1;
                continue 'outer;
            }

            // At this point, we're trying to find a slice; this comes "before" the section above
            let subset = &char_vals[left..right];

            if let Some(row) = multis.get(subset) {
                // If we found it, we may need to check for a discontiguous match. But that's only
                // if we matched on a set of two code points; and we'll only skip over one to find a
                // possible third.
                let try_discont = subset.len() == 2 && (right + 1 < input_length);

                if try_discont {
                    // Need to make sure the sequence of CCCs is kosher
                    let ccc_a = get_ccc(char_vals[right]) as u8;
                    let ccc_b = get_ccc(char_vals[right + 1]) as u8;

                    if ccc_a > 0 && ccc_b > ccc_a {
                        // Having made it this far, we can test a new subset, adding the later char.
                        // Again, this only happens if we found a match of two code points and want
                        // to add a third; so we can be oddly specific.
                        let new_subset = vec![subset[0], subset[1], char_vals[right + 1]];

                        //
                        // OUTCOME 4
                        //
                        // We checked for a multi-code-point match; found one; then checked for a
                        // larger discontiguous match; and again found one. For a complicated case,
                        // this is a good path. Fill in the weights...
                        //
                        if let Some(new_row) = multis.get(&new_subset) {
                            fill_weights(cea, new_row, &mut cea_idx, shifting, &mut last_variable);

                            // Remove the later char used for the discontiguous match
                            remove_pulled(char_vals, right + 1, &mut input_length, false);

                            left += right - left;
                            continue 'outer;
                        }
                    }
                }

                //
                // OUTCOME 5
                //
                // We checked for a multi-code-point match; found one; then checked for a larger
                // discontiguous match; and did not find any. An ok path? Fill in the weights...
                //
                fill_weights(cea, row, &mut cea_idx, shifting, &mut last_variable);
                left += right - left; // NB, we increment here by a variable amount
                continue 'outer;
            }

            // Shorten slice to try again
            right -= 1;
        }

        // This point is unreachable. All cases for the outer loop have been handled.
        unreachable!();
    }

    // Set a high value to indicate the end of the weights
    cea[cea_idx] = std::u32::MAX;
}
