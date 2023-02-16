use std::cmp::Ordering;

use crate::cea_utils::unpack_weights;

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

    if let Some(o) = compare_quaternary(a_cea, b_cea) {
        return o;
    }

    // If we got to this point, return Equal. The efficiency of processing and comparing sort keys
    // incrementally, for both strings at once, relies on the rarity of needing to continue all the
    // way through tertiary or quaternary weights. (Remember, there are two earlier fast paths for
    // equal strings -- one before normalization, one after.)
    Ordering::Equal
}

fn compare_primary(a_cea: &[u32], b_cea: &[u32]) -> Option<Ordering> {
    let mut a_filter = a_cea
        .iter()
        .filter(|w| **w != 0)
        .map(|w| unpack_weights(*w))
        .filter(|(_, p, _, _)| *p != 0);

    let mut b_filter = b_cea
        .iter()
        .filter(|w| **w != 0)
        .map(|w| unpack_weights(*w))
        .filter(|(_, p, _, _)| *p != 0);

    loop {
        let (_, a_p, _, _) = a_filter.next().unwrap_or_default();
        let (_, b_p, _, _) = b_filter.next().unwrap_or_default();

        if a_p != b_p {
            return Some(a_p.cmp(&b_p));
        }

        if a_p == 0 {
            return None;
        }
    }
}

fn compare_primary_shifting(a_cea: &[u32], b_cea: &[u32]) -> Option<Ordering> {
    let mut a_filter = a_cea
        .iter()
        .filter(|w| **w != 0)
        .map(|w| unpack_weights(*w))
        .filter(|(v, p, _, _)| !v && *p != 0);

    let mut b_filter = b_cea
        .iter()
        .filter(|w| **w != 0)
        .map(|w| unpack_weights(*w))
        .filter(|(v, p, _, _)| !v && *p != 0);

    loop {
        let (_, a_p, _, _) = a_filter.next().unwrap_or_default();
        let (_, b_p, _, _) = b_filter.next().unwrap_or_default();

        if a_p != b_p {
            return Some(a_p.cmp(&b_p));
        }

        if a_p == 0 {
            return None;
        }
    }
}

fn compare_secondary(a_cea: &[u32], b_cea: &[u32]) -> Option<Ordering> {
    let mut a_filter = a_cea
        .iter()
        .filter(|w| **w != 0)
        .map(|w| unpack_weights(*w))
        .filter(|(_, _, s, _)| *s != 0);

    let mut b_filter = b_cea
        .iter()
        .filter(|w| **w != 0)
        .map(|w| unpack_weights(*w))
        .filter(|(_, _, s, _)| *s != 0);

    loop {
        let (_, _, a_s, _) = a_filter.next().unwrap_or_default();
        let (_, _, b_s, _) = b_filter.next().unwrap_or_default();

        if a_s != b_s {
            return Some(a_s.cmp(&b_s));
        }

        if a_s == 0 {
            return None;
        }
    }
}

fn compare_tertiary(a_cea: &[u32], b_cea: &[u32]) -> Option<Ordering> {
    let mut a_filter = a_cea
        .iter()
        .filter(|w| **w != 0)
        .map(|w| unpack_weights(*w))
        .filter(|(_, _, _, t)| *t != 0);

    let mut b_filter = b_cea
        .iter()
        .filter(|w| **w != 0)
        .map(|w| unpack_weights(*w))
        .filter(|(_, _, _, t)| *t != 0);

    loop {
        let (_, _, _, a_t) = a_filter.next().unwrap_or_default();
        let (_, _, _, b_t) = b_filter.next().unwrap_or_default();

        if a_t != b_t {
            return Some(a_t.cmp(&b_t));
        }

        if a_t == 0 {
            return None;
        }
    }
}

fn compare_quaternary(a_cea: &[u32], b_cea: &[u32]) -> Option<Ordering> {
    let mut a_filter = a_cea
        .iter()
        .filter(|w| **w != 0)
        .map(|w| unpack_weights(*w))
        .filter(|(v, _, s, _)| *v || *s != 0);

    let mut b_filter = b_cea
        .iter()
        .filter(|w| **w != 0)
        .map(|w| unpack_weights(*w))
        .filter(|(v, _, s, _)| *v || *s != 0);

    loop {
        let (_, a_p, _, _) = a_filter.next().unwrap_or_default();
        let (_, b_p, _, _) = b_filter.next().unwrap_or_default();

        if a_p != b_p {
            return Some(a_p.cmp(&b_p));
        }

        if a_p == 0 {
            return None;
        }
    }
}
