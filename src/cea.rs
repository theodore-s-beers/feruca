use crate::cea_match::{
    implicit_a, implicit_b, try_discontiguous_contraction, try_pulled_contraction,
};
use crate::cea_source::{CodePointSource, Utf8Source, VecSource};
use crate::collator::CollationContext;
#[cfg(feature = "pipeline-stats")]
use crate::collator::PipelineStats;
use crate::tables::CollationTable;
use crate::weights::{primary, shift_weights, variability};
use std::cmp::Ordering;

const PENDING_CE_CAPACITY: usize = 20;

pub enum LazyPrimaryResult {
    Decided(Ordering),
    ReusablePrefix,
    NeedsFullFallback,
}

pub fn compare_primary_streaming(
    a_cea: &mut Vec<u32>,
    b_cea: &mut Vec<u32>,
    a_chars: &mut Vec<u32>,
    b_chars: &mut Vec<u32>,
    ctx: &CollationContext,
    left: usize,
    #[cfg(feature = "pipeline-stats")] stats: &mut PipelineStats,
) -> Option<Ordering> {
    a_cea.clear();
    b_cea.clear();

    let mut a_cursor = CeaCursor::new(VecSource::new(a_chars, left), ctx);
    let mut b_cursor = CeaCursor::new(VecSource::new(b_chars, left), ctx);

    loop {
        let a_p = next_primary(&mut a_cursor, a_cea, ctx.shifting);
        let b_p = next_primary(&mut b_cursor, b_cea, ctx.shifting);

        if a_p != b_p {
            #[cfg(feature = "pipeline-stats")]
            {
                stats.codepoints_consumed_primary +=
                    u64::try_from(a_cursor.consumed() + b_cursor.consumed()).unwrap_or(u64::MAX);
            }

            return Some(a_p.cmp(&b_p));
        }

        if a_p == 0 {
            a_cea.push(u32::MAX);
            b_cea.push(u32::MAX);

            #[cfg(feature = "pipeline-stats")]
            {
                stats.codepoints_consumed_primary +=
                    u64::try_from(a_cursor.consumed() + b_cursor.consumed()).unwrap_or(u64::MAX);
            }

            return None;
        }
    }
}

pub fn compare_primary_streaming_utf8(
    a_cea: &mut Vec<u32>,
    b_cea: &mut Vec<u32>,
    a_bytes: &[u8],
    b_bytes: &[u8],
    ctx: &CollationContext,
) -> LazyPrimaryResult {
    a_cea.clear();
    b_cea.clear();

    let mut a_cursor = CeaCursor::new(Utf8Source::new(a_bytes), ctx);
    let mut b_cursor = CeaCursor::new(Utf8Source::new(b_bytes), ctx);

    loop {
        let a_p = next_primary(&mut a_cursor, a_cea, ctx.shifting);
        let b_p = next_primary(&mut b_cursor, b_cea, ctx.shifting);

        if a_cursor.is_blocked() || b_cursor.is_blocked() {
            return if !ctx.shifting && a_cursor.can_resume() && b_cursor.can_resume() {
                LazyPrimaryResult::ReusablePrefix
            } else {
                LazyPrimaryResult::NeedsFullFallback
            };
        }

        if a_p != b_p {
            return LazyPrimaryResult::Decided(a_p.cmp(&b_p));
        }

        if a_p == 0 {
            a_cea.push(u32::MAX);
            b_cea.push(u32::MAX);
            return LazyPrimaryResult::ReusablePrefix;
        }
    }
}

fn next_primary(
    cursor: &mut CeaCursor<'_, impl CodePointSource>,
    buffer: &mut Vec<u32>,
    shifting: bool,
) -> u16 {
    while let Some(weights) = cursor.next_ce() {
        buffer.push(weights);

        if shifting && variability(weights) {
            continue;
        }

        let primary = primary(weights);
        if primary != 0 {
            return primary;
        }
    }

    0
}

struct CeaCursor<'a, S> {
    source: S,
    ctx: &'a CollationContext,
    pending: [u32; PENDING_CE_CAPACITY],
    pending_start: usize,
    pending_len: usize,
    last_variable: bool,
}

impl<'a, S: CodePointSource> CeaCursor<'a, S> {
    const fn new(source: S, ctx: &'a CollationContext) -> Self {
        Self {
            source,
            ctx,
            pending: [0; PENDING_CE_CAPACITY],
            pending_start: 0,
            pending_len: 0,
            last_variable: false,
        }
    }

    #[cfg(feature = "pipeline-stats")]
    fn consumed(&self) -> usize {
        self.source.consumed()
    }

    fn is_blocked(&self) -> bool {
        self.source.is_blocked()
    }

    fn can_resume(&self) -> bool {
        self.source.can_resume()
    }

    fn next_ce(&mut self) -> Option<u32> {
        loop {
            if self.pending_start < self.pending_len {
                let weights = self.pending[self.pending_start];
                self.pending_start += 1;
                return Some(weights);
            }

            if self.source.is_empty() || self.source.is_blocked() {
                return None;
            }

            self.pending_len = 0;
            self.pending_start = 0;
            self.queue_next_match();

            if self.pending_len == 0 && self.source.is_blocked() {
                return None;
            }
        }
    }

    fn queue_next_match(&mut self) {
        let Some(left_val) = self.source.peek(0) else {
            return;
        };

        // Fast path for most low code points, including most ASCII characters that remain after
        // the initial ASCII check in `Collator::collate`.
        if left_val < 0xB7 && left_val != 0x6C && left_val != 0x4C {
            self.queue_weight(self.ctx.low[left_val as usize]);
            self.source.consume(1);
            return;
        }

        // Above the low table, the collation table gives us either a simple row, a missing entry
        // requiring implicit weights, or a contraction start with lookahead metadata.
        let table = self.ctx.table;
        let entry = table.entry(left_val);
        let lookahead = table.max_len(entry);

        if lookahead == 1 {
            if CollationTable::is_missing(entry) {
                // Unlisted code points receive implicit weights.
                self.queue_raw_weight(implicit_a(left_val));
                self.queue_raw_weight(implicit_b(left_val));
            } else {
                // Simple one-code-point match.
                self.queue_row(table.simple_row(entry));
            }

            self.source.consume(1);
            return;
        }

        // This is a contraction-start entry, but there's no following code point to match. Fall
        // back to its simple row.
        if self.source.remaining() == 1 {
            self.queue_row(table.simple_row(entry));
            self.source.consume(1);
            return;
        }

        // Try the longest contiguous contraction first, without looking past the end of the input.
        let mut match_len = self.source.remaining().min(lookahead);

        while match_len > 0 {
            // Contiguous contraction attempts failed. Fall back to the first code point, but first
            // check whether later combining marks can form a discontiguous contraction.
            if match_len == 1 {
                let row = table.simple_row(entry);

                // A discontiguous contraction was found after a single-code-point fallback. Remove
                // the later code point(s) so they aren't processed again.
                if let Some((pull_index, pulled_two, new_row)) =
                    try_pulled_contraction(self.ctx, entry, &mut self.source, match_len)
                {
                    self.queue_row(new_row);
                    self.source.remove_pulled_lookahead(pull_index, pulled_two);

                    self.source.consume(1);
                    return;
                }

                // No contraction matched; emit the simple row.
                self.queue_row(row);
                self.source.consume(1);
                return;
            }

            // Try the current contiguous subset. The table supports contractions of length 2 or 3.
            let row = match match_len {
                2 => table.get2(entry, self.source.peek(1).unwrap()),
                3 => table.get3(
                    entry,
                    self.source.peek(1).unwrap(),
                    self.source.peek(2).unwrap(),
                ),
                _ => unreachable!(),
            };

            if let Some(row) = row {
                // A two-code-point contraction can sometimes be extended by pulling in a later
                // combining mark, provided the canonical combining classes allow it.
                if let Some(new_row) =
                    try_discontiguous_contraction(table, entry, &mut self.source, match_len)
                {
                    self.queue_row(new_row);
                    self.source.remove_pulled_lookahead(match_len + 1, false);

                    self.source.consume(match_len);
                    return;
                }

                // Contiguous contraction matched, with no larger discontiguous match.
                self.queue_row(row);
                self.source.consume(match_len);
                return;
            }

            // Shorten the subset and try again.
            match_len -= 1;
        }

        // All outer-loop cases should have been handled above.
        unreachable!();
    }

    fn queue_row(&mut self, row: &[u32]) {
        assert!(row.len() <= PENDING_CE_CAPACITY);

        for &weights in row {
            self.queue_weight(weights);
        }
    }

    const fn queue_weight(&mut self, weights: u32) {
        let weights = if self.ctx.shifting {
            shift_weights(weights, &mut self.last_variable)
        } else {
            weights
        };

        self.queue_raw_weight(weights);
    }

    const fn queue_raw_weight(&mut self, weights: u32) {
        self.pending[self.pending_len] = weights;
        self.pending_len += 1;
    }
}
