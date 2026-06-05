use crate::collator::CollationContext;
use crate::consts::VARIABLE;
use unicode_canonical_combining_class::get_canonical_combining_class_u32 as get_ccc;

pub fn find_byte_prefix(a: &[u8], b: &[u8], ctx: &CollationContext) -> usize {
    let mut prefix_len = a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count();

    while prefix_len > 0
        && a.get(prefix_len)
            .is_some_and(|byte| is_utf8_continuation(*byte))
    {
        prefix_len -= 1;
    }

    if prefix_len == 0 {
        return 0;
    }

    let Some(previous) = previous_char(a, prefix_len) else {
        return 0;
    };

    if ctx.table.max_len(ctx.table.entry(previous)) != 1 {
        return 0;
    }

    if ctx.shifting && VARIABLE.contains(previous) {
        return 0;
    }

    let Ok(a_next) = next_char(a, prefix_len) else {
        return 0;
    };
    let Ok(b_next) = next_char(b, prefix_len) else {
        return 0;
    };

    if a_next.is_some_and(|c| get_ccc(c) as u8 != 0)
        || b_next.is_some_and(|c| get_ccc(c) as u8 != 0)
    {
        return 0;
    }

    prefix_len
}

fn previous_char(bytes: &[u8], end: usize) -> Option<u32> {
    let mut start = end - 1;
    while start > 0 && is_utf8_continuation(bytes[start]) {
        start -= 1;
    }

    std::str::from_utf8(&bytes[start..end])
        .ok()?
        .chars()
        .next()
        .map(|c| c as u32)
}

fn next_char(bytes: &[u8], start: usize) -> Result<Option<u32>, ()> {
    let Some(&first) = bytes.get(start) else {
        return Ok(None);
    };

    let len = utf8_char_width(first).ok_or(())?;
    std::str::from_utf8(bytes.get(start..start + len).ok_or(())?)
        .map_err(|_| ())?
        .chars()
        .next()
        .map(|c| Some(c as u32))
        .ok_or(())
}

const fn is_utf8_continuation(byte: u8) -> bool {
    (byte & 0b1100_0000) == 0b1000_0000
}

const fn utf8_char_width(first: u8) -> Option<usize> {
    match first {
        0x00..=0x7F => Some(1),
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
}

pub fn find_prefix(a: &[u32], b: &[u32], ctx: &CollationContext) -> usize {
    let prefix_len = a
        .iter()
        .zip(b.iter())
        .take_while(|(x, y)| x == y && ctx.table.max_len(ctx.table.entry(**x)) == 1)
        .count();

    if prefix_len > 0 {
        // If we're shifting, then we need to look up the final code point in the prefix. If it has
        // a variable weight, or a zero primary weight, we can't remove it safely.
        if ctx.shifting && VARIABLE.contains(a[prefix_len - 1]) {
            if prefix_len > 1 {
                // If the last code point in the prefix was problematic, we can try shortening by
                // one before giving up.
                if VARIABLE.contains(a[prefix_len - 2]) {
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
