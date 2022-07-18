use crate::cea_utils::{get_implicit_a, get_implicit_b, get_shifted_weights};
use crate::consts::{LOW, LOW_CLDR, MULT, MULT_CLDR, NEED_THREE, NEED_TWO, SING, SING_CLDR};
use crate::{CollationOptions, KeysSource};
use tinyvec::{array_vec, ArrayVec};
use unicode_canonical_combining_class::get_canonical_combining_class_u32 as get_ccc;

#[allow(clippy::too_many_lines)]
pub fn generate_cea(char_vals: &mut Vec<u32>, opt: CollationOptions) -> Vec<ArrayVec<[u16; 4]>> {
    let mut cea: Vec<ArrayVec<[u16; 4]>> = Vec::new();

    let cldr = opt.keys_source == KeysSource::Cldr;
    let shifting = opt.shifting;

    let low = if cldr { &LOW_CLDR } else { &LOW };
    let singles = if cldr { &SING_CLDR } else { &SING };
    let multis = if cldr { &MULT_CLDR } else { &MULT };

    let mut left: usize = 0;
    let mut last_variable = false;

    // We spend essentially the entire function in this loop
    'outer: while left < char_vals.len() {
        let left_val = char_vals[left];

        //
        // OUTCOME 0
        //
        // The code point was low, so we could draw from a small map that associates one u32 with
        // one Weights struct. Then push the weights, shifting if necessary. This is the path that
        // catches (most) ASCII characters present in not-completely-ASCII strings.
        //
        if left_val < 183 && left_val != 108 && left_val != 76 {
            let weights = low.get(&left_val).unwrap(); // Guaranteed to succeed

            if shifting {
                let weight_vals = get_shifted_weights(*weights, last_variable);
                cea.push(weight_vals);
                if weights.variable {
                    last_variable = true;
                } else if weights.primary != 0 {
                    last_variable = false;
                }
            } else {
                let weight_vals = array_vec!(
                    [u16; 4] => weights.primary, weights.secondary, weights.tertiary
                );
                cea.push(weight_vals);
            }

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
        let check_multi = lookahead > 1 && (char_vals.len() - left > 1);

        if !check_multi {
            //
            // OUTCOME 1
            //
            // We only had to check for a single code point, and found it, so we can push the
            // weights and continue. This is a relatively fast path.
            //
            if let Some(row) = singles.get(&left_val) {
                for weights in row {
                    if shifting {
                        let weight_vals = get_shifted_weights(*weights, last_variable);
                        cea.push(weight_vals);
                        if weights.variable {
                            last_variable = true;
                        } else if weights.primary != 0 {
                            last_variable = false;
                        }
                    } else {
                        let weight_vals = array_vec!(
                            [u16; 4] => weights.primary, weights.secondary, weights.tertiary
                        );
                        cea.push(weight_vals);
                    }
                }

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

            let first_weights = get_implicit_a(left_val, shifting);
            cea.push(first_weights);

            let second_weights = get_implicit_b(left_val, shifting);
            cea.push(second_weights);

            // Increment and continue outer loop
            left += 1;
            continue;
        }

        // Here we consider multi-code-point matches, if possible

        // Don't look past the end of the vec
        let mut right = if left + lookahead > char_vals.len() {
            char_vals.len()
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
                    let mut max_right = if right + 2 < char_vals.len() {
                        right + 2
                    } else if right + 1 < char_vals.len() {
                        right + 1
                    } else {
                        // This should skip the loop below. There will be no discontiguous match.
                        right
                    };

                    let mut try_two = (max_right - right == 2) && cldr;

                    'inner: while max_right > right {
                        // Make sure the sequence of CCC values is kosher
                        let interest_cohort = &char_vals[right..=max_right];
                        let mut max_ccc = 0;

                        for elem in interest_cohort {
                            let ccc = get_ccc(*elem) as u8;
                            if ccc == 0 || ccc <= max_ccc {
                                // Can also forget about try_two in this case
                                try_two = false;
                                max_right -= 1;
                                continue 'inner;
                            }
                            max_ccc = ccc;
                        }

                        // Having made it this far, we can test a new subset, adding later char(s)
                        let new_subset = if try_two {
                            ArrayVec::from([
                                left_val,
                                char_vals[max_right - 1],
                                char_vals[max_right],
                            ])
                        } else {
                            array_vec!([u32; 3] => left_val, char_vals[max_right])
                        };

                        //
                        // OUTCOME 3
                        //
                        // We found a discontiguous match for one code point. This is a bad path,
                        // since it implies that we: checked for multiple code points; didn't find
                        // them; fell back to check for the initial code point; found it; checked
                        // for discontiguous matches; and found one. Anyway, push the weights...
                        //
                        if let Some(new_row) = multis.get(&new_subset) {
                            for weights in new_row {
                                if shifting {
                                    let weight_vals = get_shifted_weights(*weights, last_variable);
                                    cea.push(weight_vals);
                                    if weights.variable {
                                        last_variable = true;
                                    } else if weights.primary != 0 {
                                        last_variable = false;
                                    }
                                } else {
                                    let weight_vals = array_vec!(
                                        [u16; 4] => weights.primary, weights.secondary, weights.tertiary
                                    );
                                    cea.push(weight_vals);
                                }
                            }

                            // Remove the pulled char(s) (in this order!)
                            char_vals.remove(max_right);
                            if try_two {
                                char_vals.remove(max_right - 1);
                            }

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
                    for weights in row {
                        if shifting {
                            let weight_vals = get_shifted_weights(*weights, last_variable);
                            cea.push(weight_vals);
                            if weights.variable {
                                last_variable = true;
                            } else if weights.primary != 0 {
                                last_variable = false;
                            }
                        } else {
                            let weight_vals = array_vec!(
                                [u16; 4] => weights.primary, weights.secondary, weights.tertiary
                            );
                            cea.push(weight_vals);
                        }
                    }

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
                let mut try_discont = subset.len() == 2 && (right + 1 < char_vals.len());

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
                        for weights in new_row {
                            if shifting {
                                let weight_vals = get_shifted_weights(*weights, last_variable);
                                cea.push(weight_vals);
                                if weights.variable {
                                    last_variable = true;
                                } else if weights.primary != 0 {
                                    last_variable = false;
                                }
                            } else {
                                let weight_vals = array_vec!(
                                    [u16; 4] => weights.primary, weights.secondary, weights.tertiary
                                );
                                cea.push(weight_vals);
                            }
                        }

                        // Remove the pulled char
                        char_vals.remove(right + 1);

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
                for weights in row {
                    if shifting {
                        let weight_vals = get_shifted_weights(*weights, last_variable);
                        cea.push(weight_vals);
                        if weights.variable {
                            last_variable = true;
                        } else if weights.primary != 0 {
                            last_variable = false;
                        }
                    } else {
                        let weight_vals = array_vec!(
                            [u16; 4] => weights.primary, weights.secondary, weights.tertiary
                        );
                        cea.push(weight_vals);
                    }
                }

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
