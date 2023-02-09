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
                // The backup value will be set only once, i.e., at the first case difference. We
                // compare the characters in reverse order here because ASCII has uppercase letters
                // before lowercase, but we need the opposite for Unicode collation.
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

    // Reaching this means we found a case difference, but no other (incl. length); return it
    backup
}

fn ascii_alphanumeric(c: u32) -> bool {
    (48..=122).contains(&c) && !(58..=64).contains(&c) && !(91..=96).contains(&c)
}
