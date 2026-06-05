use crate::weights::{primary, secondary, tertiary, variability};
use std::cmp::Ordering;

pub fn compare_incremental(a_cea: &[u32], b_cea: &[u32], shifting: bool) -> Ordering {
    if shifting {
        if let Some(o) = compare_primary_shifting(a_cea, b_cea) {
            return o;
        }
    } else if let Some(o) = compare_primary(a_cea, b_cea) {
        return o;
    }

    if let Some(o) = compare_secondary(a_cea, b_cea) {
        return o;
    }

    if let Some(o) = compare_tertiary(a_cea, b_cea) {
        return o;
    }

    // If not shifting, stop here
    if !shifting {
        return Ordering::Equal;
    }

    // i.e., compare "quaternary" weights
    if let Some(o) = compare_primary(a_cea, b_cea) {
        return o;
    }

    // If we got to this point, return Equal. The efficiency of processing and comparing sort keys
    // incrementally, for both strings at once, relies on the rarity of needing to continue all the
    // way through tertiary or quaternary weights. (Remember, there are two earlier fast paths for
    // equal strings -- one before normalization, one after.)
    Ordering::Equal
}

fn compare_primary(a_cea: &[u32], b_cea: &[u32]) -> Option<Ordering> {
    let a_weights = a_cea
        .iter()
        .take_while(|x| **x < u32::MAX)
        .map(|w| primary(*w))
        .filter(|p| *p != 0);

    let b_weights = b_cea
        .iter()
        .take_while(|x| **x < u32::MAX)
        .map(|w| primary(*w))
        .filter(|p| *p != 0);

    compare_nonzero_weights(a_weights, b_weights)
}

fn compare_primary_shifting(a_cea: &[u32], b_cea: &[u32]) -> Option<Ordering> {
    let a_weights = a_cea
        .iter()
        .take_while(|x| **x < u32::MAX)
        .filter(|w| !variability(**w))
        .map(|w| primary(*w))
        .filter(|p| *p != 0);

    let b_weights = b_cea
        .iter()
        .take_while(|x| **x < u32::MAX)
        .filter(|w| !variability(**w))
        .map(|w| primary(*w))
        .filter(|p| *p != 0);

    compare_nonzero_weights(a_weights, b_weights)
}

fn compare_secondary(a_cea: &[u32], b_cea: &[u32]) -> Option<Ordering> {
    let a_weights = a_cea
        .iter()
        .take_while(|x| **x < u32::MAX)
        .map(|w| secondary(*w))
        .filter(|s| *s != 0);

    let b_weights = b_cea
        .iter()
        .take_while(|x| **x < u32::MAX)
        .map(|w| secondary(*w))
        .filter(|s| *s != 0);

    compare_nonzero_weights(a_weights, b_weights)
}

fn compare_tertiary(a_cea: &[u32], b_cea: &[u32]) -> Option<Ordering> {
    let a_weights = a_cea
        .iter()
        .take_while(|x| **x < u32::MAX)
        .map(|w| tertiary(*w))
        .filter(|t| *t != 0);

    let b_weights = b_cea
        .iter()
        .take_while(|x| **x < u32::MAX)
        .map(|w| tertiary(*w))
        .filter(|t| *t != 0);

    compare_nonzero_weights(a_weights, b_weights)
}

fn compare_nonzero_weights(
    mut a_weights: impl Iterator<Item = u16>,
    mut b_weights: impl Iterator<Item = u16>,
) -> Option<Ordering> {
    loop {
        let a_weight = a_weights.next().unwrap_or_default();
        let b_weight = b_weights.next().unwrap_or_default();

        if a_weight != b_weight {
            return Some(a_weight.cmp(&b_weight));
        }

        if a_weight == 0 {
            return None;
        }
    }
}
