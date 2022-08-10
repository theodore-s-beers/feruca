use std::cmp::Ordering;
use tinyvec::ArrayVec;

pub fn compare_incremental(
    a_cea: &[ArrayVec<[u16; 4]>],
    b_cea: &[ArrayVec<[u16; 4]>],
    shifting: bool,
) -> Ordering {
    // Primary
    if let Some(o) = compare_at_lvl(a_cea, b_cea, 0) {
        return o;
    }

    // Secondary
    if let Some(o) = compare_at_lvl(a_cea, b_cea, 1) {
        return o;
    }

    // Tertiary
    if let Some(o) = compare_at_lvl(a_cea, b_cea, 2) {
        return o;
    }

    // If not shifting, stop here
    if !shifting {
        return Ordering::Equal;
    }

    // Quaternary
    if let Some(o) = compare_at_lvl(a_cea, b_cea, 3) {
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
    lvl: usize,
) -> Option<Ordering> {
    // These iterators will try to find nonzero weights at the given level
    let mut a_filter = a_cea.iter().map(|row| row[lvl]).filter(|x| *x != 0);
    let mut b_filter = b_cea.iter().map(|row| row[lvl]).filter(|x| *x != 0);

    loop {
        // Advance each iterator, using 0 as the default value
        let a_weight = a_filter.next().unwrap_or(0);
        let b_weight = b_filter.next().unwrap_or(0);

        // If the weights are non-equal, return the comparison
        if a_weight != b_weight {
            return Some(a_weight.cmp(&b_weight));
        }

        // If both weights are 0, return None. This is the default return. It will be reached
        // eventually, when both iterators are exhausted.
        if a_weight == 0 && b_weight == 0 {
            return None;
        }

        // Else the loop continues
    }
}
