#![allow(clippy::similar_names)]

use unicode_canonical_combining_class::get_canonical_combining_class_u32 as get_ccc;

use crate::cea_utils::{
    ccc_sequence_ok, fill_weights, grow_vec, handle_implicit_weights, handle_low_weights,
    remove_pulled,
};
use crate::collator::CollationContext;
use crate::tables::CollationTable;

pub fn generate_cea(
    cea: &mut Vec<u32>,
    chars: &mut Vec<u32>,
    ctx: &CollationContext,
    mut left: usize,
) {
    let mut input_len = chars.len();
    let table = ctx.table;

    let mut cea_idx: usize = 0;
    let mut last_variable = false;

    // We spend essentially the entire function in this loop.
    'outer: while left < input_len {
        let left_val = chars[left];

        grow_vec(cea, cea_idx);

        // Fast path for most low code points, including most ASCII characters that remain after
        // the initial ASCII check in `Collator::collate`.
        if left_val < 0xB7 && left_val != 0x6C && left_val != 0x4C {
            let weights = ctx.low[left_val as usize];
            handle_low_weights(cea, weights, &mut cea_idx, ctx.shifting, &mut last_variable);
            left += 1;
            continue;
        }

        // Above the low table, the collation table gives us either a simple row, a missing entry
        // requiring implicit weights, or a contraction start with lookahead metadata.
        let entry = table.entry(left_val);
        let lookahead = table.max_len(entry);

        if lookahead == 1 {
            if CollationTable::is_missing(entry) {
                // Unlisted code points receive implicit weights.
                handle_implicit_weights(cea, left_val, &mut cea_idx);
            } else {
                // Simple one-code-point match.
                fill_weights(
                    cea,
                    table.simple_row(entry),
                    &mut cea_idx,
                    ctx.shifting,
                    &mut last_variable,
                );
            }

            left += 1;
            continue;
        }

        // This is a contraction-start entry, but there's no following code point to match. Fall
        // back to its simple row.
        if input_len - left == 1 {
            fill_weights(
                cea,
                table.simple_row(entry),
                &mut cea_idx,
                ctx.shifting,
                &mut last_variable,
            );

            left += 1;
            continue;
        }

        // Try the longest contiguous contraction first, without looking past the end of the input.
        let mut right = input_len.min(left + lookahead);

        while right > left {
            // Contiguous contraction attempts failed. Fall back to the first code point, but first
            // check whether later combining marks can form a discontiguous contraction.
            if right - left == 1 {
                let row = table.simple_row(entry);

                // A discontiguous contraction was found after a single-code-point fallback. Remove
                // the later code point(s) so they aren't processed again.
                if let Some((pull_index, pulled_two, new_row)) =
                    try_pulled_contraction(ctx, entry, chars, right, input_len)
                {
                    fill_weights(cea, new_row, &mut cea_idx, ctx.shifting, &mut last_variable);
                    remove_pulled(chars, pull_index, &mut input_len, pulled_two);

                    left += 1;
                    continue 'outer;
                }

                // No contraction matched; emit the simple row.
                fill_weights(cea, row, &mut cea_idx, ctx.shifting, &mut last_variable);
                left += 1;
                continue 'outer;
            }

            // Try the current contiguous subset. The table supports contractions of length 2 or 3.
            let subset_len = right - left;
            let row = match subset_len {
                2 => table.get2(entry, chars[left + 1]),
                3 => table.get3(entry, chars[left + 1], chars[left + 2]),
                _ => unreachable!(),
            };

            if let Some(row) = row {
                // A two-code-point contraction can sometimes be extended by pulling in a later
                // combining mark, provided the canonical combining classes allow it.
                if let Some(new_row) =
                    try_discontiguous_contraction(table, entry, chars, left, right)
                {
                    fill_weights(cea, new_row, &mut cea_idx, ctx.shifting, &mut last_variable);
                    remove_pulled(chars, right + 1, &mut input_len, false);

                    left += right - left;
                    continue 'outer;
                }

                // Contiguous contraction matched, with no larger discontiguous match.
                fill_weights(cea, row, &mut cea_idx, ctx.shifting, &mut last_variable);
                left += right - left;
                continue 'outer;
            }

            // Shorten the subset and try again.
            right -= 1;
        }

        // All outer-loop cases should have been handled above.
        unreachable!();
    }

    // Sentinel marks the end of the generated collation elements.
    cea[cea_idx] = u32::MAX;
}

fn try_discontiguous_contraction<'a>(
    table: &'a CollationTable,
    entry: u64,
    chars: &[u32],
    left: usize,
    right: usize,
) -> Option<&'a [u32]> {
    if !CollationTable::is_contraction(entry) || right - left != 2 {
        return None;
    }

    let next = *chars.get(right + 1)?;
    let ccc_a = get_ccc(chars[right]) as u8;
    let ccc_b = get_ccc(next) as u8;

    if ccc_a > 0 && ccc_b > ccc_a {
        table.get3(entry, chars[left + 1], next)
    } else {
        None
    }
}

fn try_pulled_contraction<'a>(
    ctx: &'a CollationContext,
    entry: u64,
    chars: &[u32],
    right: usize,
    input_len: usize,
) -> Option<(usize, bool, &'a [u32])> {
    let mut try_right = match input_len - right {
        3.. => right + 2,
        2 => right + 1,
        _ => right,
    };

    let mut try_two = (try_right - right == 2) && ctx.cldr;

    while try_right > right {
        let test_range = &chars[right..=try_right];
        if !ccc_sequence_ok(test_range) {
            try_two = false;
            try_right -= 1;
            continue;
        }

        let new_row = if try_two {
            ctx.table
                .get3(entry, chars[try_right - 1], chars[try_right])
        } else {
            ctx.table.get2(entry, chars[try_right])
        };

        if let Some(new_row) = new_row {
            return Some((try_right, try_two, new_row));
        }

        if try_two {
            try_two = false;
        } else {
            try_right -= 1;
        }
    }

    None
}
