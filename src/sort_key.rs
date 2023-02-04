use std::cmp::Ordering;

use crate::types::Weights;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
enum CollationLevel {
    Primary,
    Secondary,
    Tertiary,
    Quaternary,
}

pub fn compare_incremental(a_cea: &[Weights], b_cea: &[Weights], shifting: bool) -> Ordering {
    // Primary
    if let Some(o) = compare_at_lvl(a_cea, b_cea, CollationLevel::Primary) {
        return o;
    }

    // Secondary
    if let Some(o) = compare_at_lvl(a_cea, b_cea, CollationLevel::Secondary) {
        return o;
    }

    // Tertiary
    if let Some(o) = compare_at_lvl(a_cea, b_cea, CollationLevel::Tertiary) {
        return o;
    }

    // If not shifting, stop here
    if !shifting {
        return Ordering::Equal;
    }

    // Quaternary
    if let Some(o) = compare_at_lvl(a_cea, b_cea, CollationLevel::Quaternary) {
        return o;
    }

    // If we got to this point, return Equal. The efficiency of processing and comparing sort keys
    // incrementally, for both strings at once, relies on the rarity of needing to continue all the
    // way through tertiary or quaternary weights. (Remember, there are two earlier fast paths for
    // equal strings -- one before normalization, one after.)
    Ordering::Equal
}

fn compare_at_lvl(a_cea: &[Weights], b_cea: &[Weights], lvl: CollationLevel) -> Option<Ordering> {
    // These iterators will try to find nonzero weights at the given level

    let mut a_filter = a_cea
        .iter()
        .map(|row| match lvl {
            CollationLevel::Primary => row.primary,
            CollationLevel::Secondary => row.secondary,
            CollationLevel::Tertiary => row.tertiary,
            CollationLevel::Quaternary => row.quaternary,
        })
        .filter(|x| *x != 0);

    let mut b_filter = b_cea
        .iter()
        .map(|row| match lvl {
            CollationLevel::Primary => row.primary,
            CollationLevel::Secondary => row.secondary,
            CollationLevel::Tertiary => row.tertiary,
            CollationLevel::Quaternary => row.quaternary,
        })
        .filter(|x| *x != 0);

    loop {
        // Advance each iterator; the default value is 0
        let a_weight = a_filter.next().unwrap_or_default();
        let b_weight = b_filter.next().unwrap_or_default();

        // If the weights are non-equal, return the comparison
        if a_weight != b_weight {
            return Some(a_weight.cmp(&b_weight));
        }

        // We know the weights are equal at this point. Just check if `a_weight` is 0. That means
        // both are 0. And that only happens when both iterators are exhausted. This state will be
        // reached eventually, ending the loop.
        if a_weight == 0 {
            return None;
        }

        // Else the loop continues
    }
}
