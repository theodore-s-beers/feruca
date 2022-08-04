use std::cmp::Ordering;

pub fn try_ascii(a: &[u32], b: &[u32]) -> Option<Ordering> {
    let mut backup: Option<Ordering> = None;

    for i in 0..a.len().min(b.len()) {
        if !ascii_an(a[i]) || !ascii_an(b[i]) {
            return None;
        }

        if a[i] == b[i] {
            continue;
        }

        let new_a = if a[i] > 90 { a[i] - 32 } else { a[i] };
        let new_b = if b[i] > 90 { b[i] - 32 } else { b[i] };

        if new_a == new_b {
            if backup == None {
                backup = Some(b[i].cmp(&a[i]));
            }

            continue;
        }

        return Some(new_a.cmp(&new_b));
    }

    if a.len() != b.len() {
        return Some(a.len().cmp(&b.len()));
    }

    if backup.is_some() {
        return backup;
    }

    Some(Ordering::Equal)
}

fn ascii_an(c: u32) -> bool {
    if !(48..=122).contains(&c) || (58..=64).contains(&c) || (91..=96).contains(&c) {
        return false;
    }

    true
}
