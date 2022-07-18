use crate::consts::{NEED_THREE, NEED_TWO, SING, SING_CLDR};
use crate::{CollationOptions, KeysSource};

pub fn trim_prefix(a: &mut Vec<u32>, b: &mut Vec<u32>, opt: CollationOptions) {
    let prefix_len = find_prefix(a, b);

    if prefix_len > 0 {
        // If we're shifting, then we need to look up the final code point in the prefix. If it has
        // a variable weight, or a non-zero primary weight, we can't remove it safely.
        if opt.shifting {
            let sing = if opt.keys_source == KeysSource::Cldr {
                &SING_CLDR
            } else {
                &SING
            };

            if let Some(row) = sing.get(&a[prefix_len - 1]) {
                for weights in row {
                    if weights.variable || weights.primary == 0 {
                        // Before giving up, move back one code point and check again? Then remove
                        // a shorter prefix? In benchmarks, this was a wash. But maybe it could
                        // salvage trimming a shared prefix in some pathological cases.

                        if prefix_len > 1 {
                            if let Some(row) = sing.get(&a[prefix_len - 2]) {
                                for weights in row {
                                    // Seems we can get away with only checking variability here
                                    if weights.variable {
                                        return;
                                    }
                                }
                            }

                            // If that worked, drain the prefix, minus the last code point
                            a.drain(0..prefix_len - 1);
                            b.drain(0..prefix_len - 1);
                        }

                        // Return either way
                        return;
                    }
                }
            }

            // If the code point wasn't found, we're good: it can't be variable or ignorable
        }

        a.drain(0..prefix_len);
        b.drain(0..prefix_len);
    }
}

fn find_prefix(a: &[u32], b: &[u32]) -> usize {
    let mut count = 0;

    for i in 0..a.len().min(b.len()) {
        if a[i] != b[i] || NEED_TWO.contains(&a[i]) || NEED_THREE.contains(&a[i]) {
            break;
        }

        count += 1;
    }

    count
}
