use std::cmp::Ordering;

pub enum AsciiResult {
    Continue {
        a_needs_nfd: bool,
        b_needs_nfd: bool,
    },
    Done(Ordering),
}

pub fn fill_and_check(
    a_iter: &mut impl Iterator<Item = u32>,
    b_iter: &mut impl Iterator<Item = u32>,
    a_chars: &mut Vec<u32>,
    b_chars: &mut Vec<u32>,
) -> AsciiResult {
    let mut backup: Option<Ordering> = None;
    let mut ascii_failed = false;
    let mut a_needs_nfd = false;
    let mut b_needs_nfd = false;

    #[allow(clippy::while_let_loop)]
    loop {
        let Some(a) = a_iter.next() else { break }; // Break if iterator exhausted
        a_chars.push(a);
        a_needs_nfd |= a >= 0xC0;

        if !ascii_alphanumeric(a) {
            ascii_failed = true;
            break; // Break and set `ascii_failed` if non-ASCII character found
        }

        let Some(b) = b_iter.next() else { break }; // Break if iterator exhausted
        b_chars.push(b);
        b_needs_nfd |= b >= 0xC0;

        if !ascii_alphanumeric(b) {
            ascii_failed = true;
            break; // Break and set `ascii_failed` if non-ASCII character found
        }

        if a == b {
            continue; // Continue if we found identical ASCII characters
        }

        let a_folded = if a > 0x5A { a - 0x20 } else { a };
        let b_folded = if b > 0x5A { b - 0x20 } else { b };

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
        return AsciiResult::Done(a_folded.cmp(&b_folded));
    }

    // Finish filling code point Vecs
    let a_tail_start = a_chars.len();
    a_chars.extend(a_iter);
    a_needs_nfd |= a_chars[a_tail_start..].iter().any(|c| *c >= 0xC0);

    let b_tail_start = b_chars.len();
    b_chars.extend(b_iter);
    b_needs_nfd |= b_chars[b_tail_start..].iter().any(|c| *c >= 0xC0);

    if ascii_failed {
        return AsciiResult::Continue {
            a_needs_nfd,
            b_needs_nfd,
        };
    }

    // If we found no non-ASCII characters, and one string is a prefix of the other, the longer
    // string wins.
    if a_chars.len() != b_chars.len() {
        return AsciiResult::Done(a_chars.len().cmp(&b_chars.len()));
    }

    // If we found an ASCII case difference, return it; otherwise this will be None
    backup.map_or(
        AsciiResult::Continue {
            a_needs_nfd,
            b_needs_nfd,
        },
        AsciiResult::Done,
    )
}

fn ascii_alphanumeric(c: u32) -> bool {
    (0x30..=0x7A).contains(&c)
        && !(0x3A..=0x40).contains(&c) // Punctuation and symbols
        && !(0x5B..=0x60).contains(&c) // More symbols
}
