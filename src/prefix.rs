use crate::consts::{NEED_THREE, NEED_TWO, SING, SING_CLDR};

pub fn trim_prefix(a: &mut Vec<u32>, b: &mut Vec<u32>, cldr: bool) {
    let prefix_len = find_prefix(a, b);

    if prefix_len > 0 {
        let sing = if cldr { &SING_CLDR } else { &SING };

        // Test final code point in prefix; bail if bad
        if let Some(row) = sing.get(&a[prefix_len - 1]) {
            for weights in row {
                if weights.variable || weights.primary == 0 {
                    return;
                }
            }
        }
        // If the code point wasn't in the table, it can't be variable or ignorable

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
