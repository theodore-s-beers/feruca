#![allow(clippy::similar_names)]

use crate::cea_utils::{implicit_a, implicit_b, remove_pulled};
use crate::collator::CollationContext;
#[cfg(feature = "pipeline-stats")]
use crate::collator::PipelineStats;
use crate::consts::{DECOMP, FCD};
use crate::tables::CollationTable;
use crate::weights::{primary, shift_weights, variability};
use std::cmp::Ordering;
use unicode_canonical_combining_class::get_canonical_combining_class_u32 as get_ccc;

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

struct VecSource<'a> {
    chars: &'a mut Vec<u32>,
    #[cfg(feature = "pipeline-stats")]
    start: usize,
    pos: usize,
    len: usize,
}

trait CodePointSource {
    fn is_empty(&mut self) -> bool;
    fn remaining(&mut self) -> usize;
    fn is_blocked(&self) -> bool;
    fn can_resume(&self) -> bool;
    #[cfg(feature = "pipeline-stats")]
    fn consumed(&self) -> usize;
    fn peek(&mut self, offset: usize) -> Option<u32>;
    fn consume(&mut self, count: usize);
    fn remove_pulled_lookahead(&mut self, offset: usize, pulled_two: bool);
}

impl<'a> VecSource<'a> {
    const fn new(chars: &'a mut Vec<u32>, pos: usize) -> Self {
        Self {
            #[cfg(feature = "pipeline-stats")]
            start: pos,
            pos,
            len: chars.len(),
            chars,
        }
    }
}

impl CodePointSource for VecSource<'_> {
    fn is_empty(&mut self) -> bool {
        self.pos >= self.len
    }

    fn remaining(&mut self) -> usize {
        self.len - self.pos
    }

    fn is_blocked(&self) -> bool {
        false
    }

    fn can_resume(&self) -> bool {
        true
    }

    #[cfg(feature = "pipeline-stats")]
    fn consumed(&self) -> usize {
        self.pos - self.start
    }

    fn peek(&mut self, offset: usize) -> Option<u32> {
        let index = self.pos + offset;
        (index < self.len).then(|| self.chars[index])
    }

    fn consume(&mut self, count: usize) {
        self.pos += count;
    }

    fn remove_pulled_lookahead(&mut self, offset: usize, pulled_two: bool) {
        let index = self.pos + offset;
        remove_pulled(self.chars, index, &mut self.len, pulled_two);
    }
}

struct Utf8Source<'a> {
    bytes: &'a [u8],
    byte_pos: usize,
    lookahead: [u32; 4],
    lookahead_bytes: [usize; 4],
    lookahead_len: usize,
    blocked: bool,
    can_resume: bool,
    prev_trail_cc: u8,
}

impl<'a> Utf8Source<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            byte_pos: 0,
            lookahead: [0; 4],
            lookahead_bytes: [0; 4],
            lookahead_len: 0,
            blocked: false,
            can_resume: true,
            prev_trail_cc: 0,
        }
    }

    fn fill(&mut self, offset: usize) {
        while self.lookahead_len <= offset && self.decoded_byte_end() < self.bytes.len() {
            if self.blocked {
                return;
            }

            let start = self.decoded_byte_end();
            let Some((code_point, len)) = decode_utf8(&self.bytes[start..]) else {
                self.blocked = true;
                self.can_resume = true;
                return;
            };

            if !self.accepts_normalization_boundary(code_point) {
                self.blocked = true;
                return;
            }

            self.lookahead[self.lookahead_len] = code_point;
            self.lookahead_bytes[self.lookahead_len] = len;
            self.lookahead_len += 1;
        }
    }

    fn decoded_byte_end(&self) -> usize {
        self.byte_pos
            + self.lookahead_bytes[..self.lookahead_len]
                .iter()
                .sum::<usize>()
    }

    fn accepts_normalization_boundary(&mut self, code_point: u32) -> bool {
        if code_point < 0xC0 {
            self.prev_trail_cc = 0;
            return true;
        }

        if code_point == 0x0F81 || (0xAC00..=0xD7A3).contains(&code_point) {
            self.can_resume = false;
            return false;
        }

        if DECOMP.get(code_point).is_some() {
            self.can_resume = false;
            return false;
        }

        let (lead_cc, trail_cc) = FCD.get(code_point).map_or_else(
            || {
                let cc = get_ccc(code_point) as u8;
                (cc, cc)
            },
            |vals| vals.to_be_bytes().into(),
        );

        if lead_cc != 0 && lead_cc < self.prev_trail_cc {
            self.can_resume = false;
            return false;
        }

        self.prev_trail_cc = trail_cc;
        true
    }
}

impl CodePointSource for Utf8Source<'_> {
    fn is_empty(&mut self) -> bool {
        self.fill(0);
        self.lookahead_len == 0 && self.byte_pos >= self.bytes.len()
    }

    fn remaining(&mut self) -> usize {
        self.fill(3);

        if self.blocked || self.decoded_byte_end() == self.bytes.len() {
            self.lookahead_len
        } else {
            self.lookahead_len.max(4)
        }
    }

    fn is_blocked(&self) -> bool {
        self.blocked
    }

    fn can_resume(&self) -> bool {
        self.can_resume
    }

    #[cfg(feature = "pipeline-stats")]
    fn consumed(&self) -> usize {
        self.byte_pos
    }

    fn peek(&mut self, offset: usize) -> Option<u32> {
        self.fill(offset);
        (offset < self.lookahead_len).then(|| self.lookahead[offset])
    }

    fn consume(&mut self, count: usize) {
        self.byte_pos += self.lookahead_bytes[..count].iter().sum::<usize>();
        self.lookahead.copy_within(count..self.lookahead_len, 0);
        self.lookahead_bytes
            .copy_within(count..self.lookahead_len, 0);
        self.lookahead_len -= count;
    }

    fn remove_pulled_lookahead(&mut self, _offset: usize, _pulled_two: bool) {
        self.blocked = true;
        self.can_resume = false;
    }
}

fn decode_utf8(bytes: &[u8]) -> Option<(u32, usize)> {
    let first = *bytes.first()?;

    if first < 0x80 {
        return Some((u32::from(first), 1));
    }

    let (mut code_point, len) = if first & 0xE0 == 0xC0 {
        (u32::from(first & 0x1F), 2)
    } else if first & 0xF0 == 0xE0 {
        (u32::from(first & 0x0F), 3)
    } else if first & 0xF8 == 0xF0 {
        (u32::from(first & 0x07), 4)
    } else {
        return None;
    };

    if bytes.len() < len {
        return None;
    }

    for &byte in &bytes[1..len] {
        if byte & 0xC0 != 0x80 {
            return None;
        }

        code_point = (code_point << 6) | u32::from(byte & 0x3F);
    }

    if !valid_utf8_scalar(code_point, len) {
        return None;
    }

    Some((code_point, len))
}

const fn valid_utf8_scalar(code_point: u32, len: usize) -> bool {
    match len {
        2 => code_point >= 0x80 && code_point <= 0x7FF,
        3 => {
            code_point >= 0x800
                && code_point <= 0xFFFF
                && (code_point < 0xD800 || code_point > 0xDFFF)
        }
        4 => code_point >= 0x1_0000 && code_point <= 0x10_FFFF,
        _ => false,
    }
}

fn try_discontiguous_contraction<'a>(
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

fn try_pulled_contraction<'a>(
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
