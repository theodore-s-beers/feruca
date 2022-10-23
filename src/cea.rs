use tinyvec::ArrayVec;
use unicode_canonical_combining_class::get_canonical_combining_class_u32 as get_ccc;

use crate::cea_utils::{
    ccc_sequence_ok, get_subset, get_table_multis, get_table_singles, handle_implicit_weights,
    handle_low_weights, push_weights, remove_pulled,
};
use crate::consts::{LOW, LOW_CLDR, NEED_THREE, NEED_TWO};
use crate::{Collator, Tailoring};

pub fn generate_cea(char_vals: &mut Vec<u32>, collator: &Collator) -> Vec<ArrayVec<[u16; 4]>> {
    let mut cea: Vec<ArrayVec<[u16; 4]>> = Vec::new();

    let cldr = collator.tailoring != Tailoring::Ducet;
    let shifting = collator.shifting;

    let low = if cldr { &LOW_CLDR } else { &LOW };
    let singles = get_table_singles(collator.tailoring);
    let multis = get_table_multis(collator.tailoring);

    let mut input_length = char_vals.len();
    let mut left: usize = 0;
    let mut last_variable = false;

    // We spend essentially the entire function in this loop
    'outer: while left < input_length {
        let left_val = char_vals[left];

        //
        // OUTCOME 0
        //
        // The code point was low, so we could draw from a small map that associates one u32 with
        // one Weights struct. Then push the weights, shifting if necessary. This is the path that
        // catches (most) ASCII characters present in not-completely-ASCII strings.
        //
        if left_val < 183 && left_val != 108 && left_val != 76 {
            let weights = low[&left_val]; // Guaranteed to succeed

            handle_low_weights(shifting, weights, &mut last_variable, &mut cea);

            // Increment and continue outer loop
            left += 1;
            continue;
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
            // OUTCOME 1
            //
            // We only had to check for a single code point, and found it, so we can push the
            // weights and continue. This is a relatively fast path.
            //
            if let Some(row) = singles.get(&left_val) {
                push_weights(row, shifting, &mut last_variable, &mut cea);

                // Increment and continue outer loop
                left += 1;
                continue;
            }

            //
            // OUTCOME 2
            //
            // We checked for a single code point and didn't find it. That means it's unlisted. We
            // then calculate implicit weights, push them, and move on. I used to think there were
            // multiple paths to the "implicit weights" case, but it seems not.
            //
            handle_implicit_weights(left_val, shifting, &mut cea);

            // Increment and continue outer loop
            left += 1;
            continue;
        }

        // Here we consider multi-code-point matches, if possible

        // Don't look past the end of the vec
        let mut right = if left + lookahead > input_length {
            input_length
        } else {
            left + lookahead
        };

        while right > left {
            // If right - left == 1 (which cannot be the case in the first iteration), attempts to
            // find a slice have failed. So look for one code point, in the singles map
            if right - left == 1 {
                // If we found it, we do still need to check for discontiguous matches
                if let Some(row) = singles.get(&left_val) {
                    // Determine how much further right to look
                    let mut max_right = match right {
                        r if r + 2 < input_length => r + 2,
                        r if r + 1 < input_length => r + 1,
                        _ => right, // Skip the loop below; there will be no discontiguous match
                    };

                    let mut try_two = (max_right - right == 2) && cldr;

                    'inner: while max_right > right {
                        // Make sure the sequence of CCC values is kosher
                        let interest_cohort = &char_vals[right..=max_right];

                        if !ccc_sequence_ok(interest_cohort) {
                            // Can also forget about try_two in this case
                            try_two = false;
                            max_right -= 1;
                            continue 'inner;
                        }

                        // Having made it this far, we can test a new subset, adding later char(s)
                        let new_subset = get_subset(try_two, left_val, char_vals, max_right);

                        //
                        // OUTCOME 3
                        //
                        // We found a discontiguous match for one code point. This is a bad path,
                        // since it implies that we: checked for multiple code points; didn't find
                        // them; fell back to check for the initial code point; found it; checked
                        // for discontiguous matches; and found one. Anyway, push the weights...
                        //
                        if let Some(new_row) = multis.get(&new_subset) {
                            push_weights(new_row, shifting, &mut last_variable, &mut cea);

                            // Remove the pulled char(s)
                            remove_pulled(char_vals, max_right, &mut input_length, try_two);

                            // Increment and continue outer loop
                            left += 1;
                            continue 'outer;
                        }

                        // If we tried for two, don't decrement max_right yet
                        // Inner loop will re-run
                        if try_two {
                            try_two = false;
                        } else {
                            // Otherwise decrement max_right; inner loop may re-run, or finish
                            max_right -= 1;
                        }
                    }

                    //
                    // OUTCOME 4
                    //
                    // We checked for multiple code points; failed to find them; fell back to check
                    // for the initial code point; found it; checked for discontiguous matches; and
                    // did not find any. This is a really bad path. Push the weights...
                    //
                    push_weights(row, shifting, &mut last_variable, &mut cea);

                    // Increment and continue outer loop
                    left += 1;
                    continue 'outer;
                }

                // Reaching this point would imply that we looked for multiple code points; failed
                // to find anything; fell back to search for the left code point; and didn't find
                // that, either. So in theory, we would be dealing with an unlisted code point and
                // proceeding to calculate implicit weights. But that's impossible, isn't it? If we
                // started this path by checking for multiples, that means we had one of the code
                // points in NEED_THREE or NEED_TWO -- all of which are listed in the tables. I
                // think this is actually unreachable; and my testing bears that out.

                // no-op
            }

            // At this point, we're trying to find a slice; this comes "before" the section above
            let subset = &char_vals[left..right];

            if let Some(row) = multis.get(subset) {
                // If we found it, we may need to check for a discontiguous match.
                // But that's only if we matched on a set of two code points; and we'll only skip
                // over one to find a possible third.
                let mut try_discont = subset.len() == 2 && (right + 1 < input_length);

                while try_discont {
                    // Need to make sure the sequence of CCCs is kosher
                    let ccc_a = get_ccc(char_vals[right]) as u8;
                    let ccc_b = get_ccc(char_vals[right + 1]) as u8;

                    if ccc_a == 0 || ccc_a >= ccc_b {
                        // Bail -- no discontiguous match
                        try_discont = false;
                        continue;
                    }

                    // Having made it this far, we can test a new subset, adding the later char.
                    // Again, this only happens if we found a match of two code points and want to
                    // add a third; so we can be oddly specific.
                    let new_subset = ArrayVec::from([subset[0], subset[1], char_vals[right + 1]]);

                    //
                    // OUTCOME 5
                    //
                    // We checked for multiple code points; found something; went on to check for a
                    // discontiguous match; and found one. For a complicated case, this is a good
                    // path. Push the weights...
                    //
                    if let Some(new_row) = multis.get(&new_subset) {
                        push_weights(new_row, shifting, &mut last_variable, &mut cea);

                        // Remove the pulled char
                        remove_pulled(char_vals, right + 1, &mut input_length, false);

                        // Increment and continue outer loop
                        left += right - left;
                        continue 'outer;
                    }

                    // The loop will not run again -- no discontiguous match
                    try_discont = false;
                }

                //
                // OUTCOME 6
                //
                // We checked for multiple code points; found something; checked for discontiguous
                // matches; and did not find any. This is an ok path? Push the weights...
                //
                push_weights(row, shifting, &mut last_variable, &mut cea);

                // Increment and continue outer loop
                left += right - left;
                continue 'outer;
            }

            // Shorten slice to try again
            right -= 1;
        }

        // This is another unreachable point. All possible cases for the outer loop have been
        // handled. There's no need to increment.
    }

    // Return!
    cea
}
