use crate::cea_match::remove_pulled;
use crate::consts::{DECOMP, FCD};
use unicode_canonical_combining_class::get_canonical_combining_class_u32 as get_ccc;

pub struct VecSource<'a> {
    chars: &'a mut Vec<u32>,
    #[cfg(feature = "pipeline-stats")]
    start: usize,
    pos: usize,
    len: usize,
}

pub trait CodePointSource {
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
    pub const fn new(chars: &'a mut Vec<u32>, pos: usize) -> Self {
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

pub struct Utf8Source<'a> {
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
    pub const fn new(bytes: &'a [u8]) -> Self {
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
