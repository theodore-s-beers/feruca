use std::cmp::Ordering;

pub fn try_ascii(a: &[u32], b: &[u32]) -> Option<Ordering> {
    let mut backup: Option<Ordering> = None;

    let a_len = a.len();
    let b_len = b.len();

    for i in 0..a_len.min(b_len) {
        if !ascii_alphanumeric(a[i]) || !ascii_alphanumeric(b[i]) {
            return None;
        }

        if a[i] == b[i] {
            continue;
        }

        let a_folded = if a[i] > 90 { a[i] - 32 } else { a[i] };
        let b_folded = if b[i] > 90 { b[i] - 32 } else { b[i] };

        // This means the characters differ only in case (since they weren't equal before folding)
        if a_folded == b_folded {
            if backup.is_none() {
                // Set backup as the comparison of the original characters, in reverse order
                backup = Some(b[i].cmp(&a[i]));
            }

            continue;
        }

        return Some(a_folded.cmp(&b_folded));
    }

    // Slices were equal so far; if one is longer, it wins
    if a_len != b_len {
        return Some(a_len.cmp(&b_len));
    }

    // This would mean we found a case difference, but no other (incl. length); return it
    if backup.is_some() {
        return backup;
    }

    // I believe this is unreachable in practice, but whatever
    Some(Ordering::Equal)
}

fn ascii_alphanumeric(c: u32) -> bool {
    (48..=122).contains(&c) && !(58..=64).contains(&c) && !(91..=96).contains(&c)
}
