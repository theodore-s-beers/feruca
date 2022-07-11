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

        a.drain(0..prefix_len);
        b.drain(0..prefix_len);
    }
}

fn find_prefix(a: &[u32], b: &[u32]) -> usize {
    a.iter()
        .zip(b)
        .take_while(|(x, y)| x == y && !NEED_THREE.contains(x) && !NEED_TWO.contains(x))
        .count()
}
