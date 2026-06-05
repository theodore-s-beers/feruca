use crate::cea_source::CodePointSource;
use crate::collator::CollationContext;
use crate::consts::INCLUDED_UNASSIGNED;
use crate::tables::CollationTable;
use crate::weights::pack_weights;
use unicode_canonical_combining_class::get_canonical_combining_class_u32 as get_ccc;

pub fn implicit_a(cp: u32) -> u32 {
    let aaaa = if INCLUDED_UNASSIGNED.contains(&cp) {
        0xFBC0 + (cp >> 15)
    } else {
        match cp {
            0x3400..=0x4DBF | 0x20000..=0x2A6DF | 0x2A700..=0x2EE5D | 0x30000..=0x323AF => {
                0xFB80 + (cp >> 15)
            } // CJK2
            0x4E00..=0x9FFF | 0xF900..=0xFAFF => 0xFB40 + (cp >> 15), // CJK1
            0x17000..=0x18AFF | 0x18D00..=0x18D8F => 0xFB00,          // Tangut
            0x18B00..=0x18CFF => 0xFB02,                              // Khitan
            0x1B170..=0x1B2FF => 0xFB01,                              // Nushu
            _ => 0xFBC0 + (cp >> 15),                                 // unass.
        }
    };

    #[allow(clippy::cast_possible_truncation)]
    pack_weights(false, aaaa as u16, 32, 2)
}

pub fn implicit_b(cp: u32) -> u32 {
    let mut bbbb = if INCLUDED_UNASSIGNED.contains(&cp) {
        cp & 0x7FFF
    } else {
        match cp {
            0x17000..=0x18AFF | 0x18D00..=0x18D8F => cp - 0x17000, // Tangut
            0x18B00..=0x18CFF => cp - 0x18B00,                     // Khitan
            0x1B170..=0x1B2FF => cp - 0x1B170,                     // Nushu
            _ => cp & 0x7FFF,                                      // CJK1, CJK2, unass.
        }
    };

    // BBBB always gets bitwise ORed with this value
    bbbb |= 0x8000;

    #[allow(clippy::cast_possible_truncation)]
    pack_weights(false, bbbb as u16, 0, 0)
}

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
    let ccc_a = get_ccc(source.peek(match_len).unwrap()) as u8;
    let ccc_b = get_ccc(next) as u8;

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

    let mut try_two = (try_offset - match_len == 2) && ctx.cldr;

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
        let ccc = get_ccc(source.peek(offset).unwrap()) as u8;

        if ccc == 0 || ccc <= max_ccc {
            return false;
        }

        max_ccc = ccc;
    }

    true
}
