use std::cmp::Ordering;

pub fn all_ascii(a: &[u32], b: &[u32]) -> bool {
    for i in 0..a.len().min(b.len()) {
        if a[i] > 122                      // Too high
            || b[i] > 122
            || a[i] < 48                   // Too low
            || b[i] < 48
            || (58..=64).contains(&a[i])   // Non-alphanumeric
            || (58..=64).contains(&b[i])
            || (91..=96).contains(&a[i])   // Non-alphanumeric
            || (91..=96).contains(&b[i])
        {
            return false;
        }
    }

    true
}

pub fn compare_ascii(a: Vec<u32>, b: Vec<u32>) -> Ordering {
    // Check if the strings have any difference apart from capitalization (probably yes)

    let a_lower: Vec<u32> = a
        .iter()
        .map(|c| if *c > 90 { c - 32 } else { *c })
        .collect();

    let b_lower: Vec<u32> = b
        .iter()
        .map(|c| if *c > 90 { c - 32 } else { *c })
        .collect();

    if a_lower.cmp(&b_lower) != Ordering::Equal {
        return a_lower.cmp(&b_lower);
    }

    // If they were identical apart from capitalization, we need to adjust

    let a_fixed: Vec<u32> = a
        .into_iter()
        .map(|x| if (65..=90).contains(&x) { x * 2 } else { x })
        .collect();

    let b_fixed: Vec<u32> = b
        .into_iter()
        .map(|x| if (65..=90).contains(&x) { x * 2 } else { x })
        .collect();

    a_fixed.cmp(&b_fixed)
}
