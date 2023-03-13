use std::cmp::Ordering;

pub fn fill_and_check(
    a_iter: &mut impl Iterator<Item = u32>,
    b_iter: &mut impl Iterator<Item = u32>,
    a_chars: &mut Vec<u32>,
    b_chars: &mut Vec<u32>,
) -> Option<Ordering> {
    let mut backup: Option<Ordering> = None;
    let mut bad = false;

    loop {
        let Some(a) = a_iter.next() else { break }; // Break if iterator exhausted
        a_chars.push(a);

        if !ascii_alphanumeric(a) {
            bad = true;
            break; // Break and set `bad` if non-ASCII character found
        }

        let Some(b) = b_iter.next() else { break }; // Break if iterator exhausted
        b_chars.push(b);

        if !ascii_alphanumeric(b) {
            bad = true;
            break; // Break and set `bad` if non-ASCII character found
        }

        if a == b {
            continue; // Continue if we found identical ASCII characters
        }

        let a_folded = if a > 90 { a - 32 } else { a };
        let b_folded = if b > 90 { b - 32 } else { b };

        // This means the characters differ only in case (since they weren't equal before folding)
        if a_folded == b_folded {
            if backup.is_none() {
                // The backup value will be set only once, i.e., at the first case difference. We
                // compare the characters in reverse order here because ASCII has uppercase letters
                // before lowercase, but we need the opposite for Unicode collation.
                backup = Some(b.cmp(&a));
            }

            continue;
        }

        // We found a difference between ASCII characters; return it
        return Some(a_folded.cmp(&b_folded));
    }

    // Finish filling code point Vecs
    a_chars.extend(a_iter);
    b_chars.extend(b_iter);

    if bad {
        return None;
    }

    // If we found no non-ASCII characters, and one string is a prefix of the other, the longer
    // string wins.
    if a_chars.len() != b_chars.len() {
        return Some(a_chars.len().cmp(&b_chars.len()));
    }

    // If we found an ASCII case difference, return it; otherwise this will be None
    backup
}

fn ascii_alphanumeric(c: u32) -> bool {
    (48..=122).contains(&c) && !(58..=64).contains(&c) && !(91..=96).contains(&c)
}
