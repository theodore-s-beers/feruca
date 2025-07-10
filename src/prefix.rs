use crate::consts::{NEED_THREE, NEED_TWO, VARIABLE};

pub fn find_prefix(a: &[u32], b: &[u32], shifting: bool) -> usize {
    let prefix_len = a
        .iter()
        .zip(b.iter())
        .take_while(|(x, y)| x == y && !NEED_TWO.contains(x) && !NEED_THREE.contains(x))
        .count();

    if prefix_len > 0 {
        // If we're shifting, then we need to look up the final code point in the prefix. If it has
        // a variable weight, or a zero primary weight, we can't remove it safely. I generated a
        // hash set of all such code points.
        if shifting && VARIABLE.contains(&a[prefix_len - 1]) {
            if prefix_len > 1 {
                // If the last code point in the prefix was problematic, we can try shortening by
                // one before giving up.
                if VARIABLE.contains(&a[prefix_len - 2]) {
                    return 0;
                }

                // If that worked, remove the prefix minus one
                return prefix_len - 1;
            }

            return 0;
        }

        return prefix_len;
    }

    0
}
