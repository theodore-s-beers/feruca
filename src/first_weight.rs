use crate::cea_match::implicit_a;
use crate::collator::CollationContext;
use crate::tables::CollationTable;
use crate::weights::{primary, variability};
use std::cmp::Ordering;

pub fn try_initial(ctx: &CollationContext, a_chars: &[u32], b_chars: &[u32]) -> Option<Ordering> {
    let a_first = a_chars[0];
    let b_first = b_chars[0];

    if !can_compare_initial_primaries(a_first, b_first, ctx.table) {
        return None;
    }

    let a_first_primary = get_first_primary(a_first, ctx);
    if a_first_primary == 0 {
        return None;
    }

    let b_first_primary = get_first_primary(b_first, ctx);
    if b_first_primary == 0 || b_first_primary == a_first_primary {
        return None;
    }

    Some(a_first_primary.cmp(&b_first_primary))
}

fn can_compare_initial_primaries(a: u32, b: u32, table: &CollationTable) -> bool {
    a != b && table.max_len(table.entry(a)) == 1 && table.max_len(table.entry(b)) == 1
}

fn get_first_primary(val: u32, ctx: &CollationContext) -> u16 {
    // Fast path for low code points
    if val < 0xB7 && val != 0x6C && val != 0x4C {
        let weights = ctx.low[val as usize]; // Guaranteed to succeed

        if ctx.shifting && variability(weights) {
            return 0;
        }

        return primary(weights);
    }

    // Or look in the big table
    let entry = ctx.table.entry(val);

    if !CollationTable::is_missing(entry) {
        let row = ctx.table.simple_row(entry);
        if ctx.shifting && variability(row[0]) {
            return 0;
        }

        return primary(row[0]);
    }

    // If all else failed, calculate implicit weights
    let first_weights = implicit_a(val);
    primary(first_weights)
}
