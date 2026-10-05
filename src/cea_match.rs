use crate::ccc::get_ccc;
use crate::cea_source::CodePointSource;
use crate::collator::CollationContext;
use crate::tables::CollationTable;

pub fn remove_pulled(char_vals: &mut Vec<u32>, i: usize, input_length: &mut usize, try_two: bool) {
    char_vals.remove(i);
    *input_length -= 1;

    if try_two {
        char_vals.remove(i - 1);
        *input_length -= 1;
    }
}

pub fn try_discontiguous_contraction<'a>(
    table: &'a CollationTable,
    entry: u64,
    source: &mut impl CodePointSource,
    match_len: usize,
) -> Option<&'a [u32]> {
    if !CollationTable::is_contraction(entry) || match_len != 2 {
        return None;
    }

    let next = source.peek(match_len + 1)?;
    let ccc_a = get_ccc(source.peek(match_len).unwrap());
    let ccc_b = get_ccc(next);

    if ccc_a > 0 && ccc_b > ccc_a {
        table.get3(entry, source.peek(1).unwrap(), next)
    } else {
        None
    }
}

pub fn try_pulled_contraction<'a>(
    ctx: &'a CollationContext,
    entry: u64,
    source: &mut impl CodePointSource,
    match_len: usize,
) -> Option<(usize, bool, &'a [u32])> {
    let mut try_offset = match source.remaining() - match_len {
        3.. => match_len + 2,
        2 => match_len + 1,
        _ => match_len,
    };

    let mut try_two = try_offset - match_len == 2;

    while try_offset > match_len {
        if !ccc_sequence_ok(source, match_len, try_offset) {
            try_two = false;
            try_offset -= 1;
            continue;
        }

        let new_row = if try_two {
            ctx.table.get3(
                entry,
                source.peek(try_offset - 1).unwrap(),
                source.peek(try_offset).unwrap(),
            )
        } else {
            ctx.table.get2(entry, source.peek(try_offset).unwrap())
        };

        if let Some(new_row) = new_row {
            return Some((try_offset, try_two, new_row));
        }

        if try_two {
            try_two = false;
        } else {
            try_offset -= 1;
        }
    }

    None
}

fn ccc_sequence_ok(
    source: &mut impl CodePointSource,
    start_offset: usize,
    end_offset: usize,
) -> bool {
    let mut max_ccc = 0;

    for offset in start_offset..=end_offset {
        let ccc = get_ccc(source.peek(offset).unwrap());

        if ccc == 0 || ccc <= max_ccc {
            return false;
        }

        max_ccc = ccc;
    }

    true
}
