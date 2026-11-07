/// Forward UTF-8 decoding with maximal-subpart replacement, allocation-free.
pub struct CodePoints<'a> {
    bytes: std::slice::Iter<'a, u8>,
}

impl<'a> CodePoints<'a> {
    #[inline]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes.iter(),
        }
    }
}

impl Iterator for CodePoints<'_> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<u32> {
        // Advance ASCII directly, without entering the multibyte decoder.
        let bytes = self.bytes.as_slice();
        let first = *self.bytes.next()?;
        if first < 0x80 {
            return Some(u32::from(first));
        }
        let (value, len) = decode_non_ascii(bytes, first);
        self.bytes = bytes[len..].iter();
        Some(value)
    }
}

// Classes double as shifts that strip the leading byte's UTF-8 tag bits.
// This technique is used in Hoehrmann's UTF-8 DFA:
// https://bjoern.hoehrmann.de/utf-8/decoder/dfa/
const CLASSES: [u8; 256] = {
    let mut classes = [8; 256];
    let mut byte = 0;
    while byte < 256 {
        classes[byte] = match byte {
            0x00..=0x7F => 0,
            0x80..=0x8F => 1,
            0x90..=0x9F => 9,
            0xA0..=0xBF => 7,
            0xC2..=0xDF => 2,
            0xE0 => 10,
            0xE1..=0xEC | 0xEE..=0xEF => 3,
            0xED => 4,
            0xF0 => 11,
            0xF1..=0xF3 => 6,
            0xF4 => 5,
            _ => 8,
        };
        byte += 1;
    }
    classes
};

// Sixteen columns keep state and class in separate nibbles. A u8 index into
// the padded table is always in bounds, including in safe Rust.
const ACCEPT: u8 = 0;
const REJECT: u8 = 16;
const TRANSITIONS: [u8; 256] = {
    let mut table = [REJECT; 256];
    table[0] = ACCEPT;
    table[2] = 32; // One continuation remaining.
    table[3] = 48; // Two continuations remaining.
    table[6] = 64; // Three continuations remaining.
    table[10] = 80; // E0: exclude overlong encodings.
    table[4] = 96; // ED: exclude surrogates.
    table[11] = 112; // F0: exclude overlong encodings.
    table[5] = 128; // F4: exclude values above U+10FFFF.
    let continuations = [1, 7, 9];
    let mut i = 0;
    while i < continuations.len() {
        let class = continuations[i];
        table[32 + class] = ACCEPT;
        table[48 + class] = 32;
        table[64 + class] = 48;
        i += 1;
    }
    table[80 + 7] = 32; // E0 accepts A0..BF.
    table[96 + 1] = 32; // ED accepts 80..9F.
    table[96 + 9] = 32;
    table[112 + 9] = 48; // F0 accepts 90..BF.
    table[112 + 7] = 48;
    table[128 + 1] = 48; // F4 accepts 80..8F.
    table
};

#[inline]
fn decode_non_ascii(bytes: &[u8], first: u8) -> (u32, usize) {
    // Common two-byte scalars need no state-table walk.
    if let [first @ 0xC2..=0xDF, second @ 0x80..=0xBF, ..] = bytes {
        return ((u32::from(first & 0x1F) << 6) | u32::from(second & 0x3F), 2);
    }
    let class = CLASSES[usize::from(first)];
    let mut state = TRANSITIONS[usize::from(class)];
    if state == REJECT {
        return (0xFFFD, 1);
    }
    let mut value = u32::from(first) & (0xFF >> class);
    for (i, &byte) in bytes[1..].iter().enumerate() {
        let class = CLASSES[usize::from(byte)];
        value = (value << 6) | u32::from(byte & 0x3F);
        state = TRANSITIONS[usize::from(state | class)];
        if state == ACCEPT {
            return (value, i + 2);
        }
        if state == REJECT {
            // The leading byte and accepted continuations form the maximal
            // subpart. Leave the offending byte for the next call.
            return (0xFFFD, i + 1);
        }
    }
    // An incomplete prefix at end of input is one maximal subpart.
    (0xFFFD, bytes.len())
}

#[cfg(test)]
mod tests {
    use super::CodePoints;

    #[test]
    fn every_unicode_scalar() {
        let mut buffer = [0; 4];
        for value in 0..=0x10_FFFF {
            if let Some(c) = char::from_u32(value) {
                let bytes = c.encode_utf8(&mut buffer).as_bytes();
                let mut iter = CodePoints::new(bytes);
                assert_eq!(iter.next(), Some(value));
                assert_eq!(iter.next(), None);
                assert_eq!(iter.next(), None);
            }
        }
    }

    #[test]
    fn maximal_subparts() {
        let cases: &[(&[u8], &[u32])] = &[
            (b"", &[]),
            (b"\xE2\x82", &[0xFFFD]),
            (b"\xE2\x82A", &[0xFFFD, 0x41]),
            (b"\xE2A\x82", &[0xFFFD, 0x41, 0xFFFD]),
            (b"\xED\xA0\x80", &[0xFFFD, 0xFFFD, 0xFFFD]),
            (b"\xC0\xAF", &[0xFFFD, 0xFFFD]),
            (b"\xE0\x80\xAF", &[0xFFFD, 0xFFFD, 0xFFFD]),
            (b"\xF0\x80\x80A", &[0xFFFD, 0xFFFD, 0xFFFD, 0x41]),
            (b"\xF4\x90\x80\x80", &[0xFFFD; 4]),
            (b"\xF5\x80\x80\x80", &[0xFFFD; 4]),
            (b"\xF0\x90\x80", &[0xFFFD]),
            (b"\xF0\x90\x80A", &[0xFFFD, 0x41]),
            (b"\x80\xBF\xFE\xFF", &[0xFFFD; 4]),
            (b"\xE1\x80\xE2\xF0\x91\x92\xF1\xBF", &[0xFFFD; 4]),
        ];
        for &(bytes, expected) in cases {
            assert_eq!(
                CodePoints::new(bytes).collect::<Vec<_>>(),
                expected,
                "{bytes:x?}"
            );
        }
    }

    #[test]
    fn matches_standard_lossy_decoding() {
        // Exhaust all one- and two-byte inputs, including invalid starts and
        // incomplete sequences. The std lib is an independent oracle.
        for first in 0..=255 {
            check(&[first]);
            for second in 0..=255 {
                check(&[first, second]);
            }
        }

        // Exercise longer mixed sequences and errors at different offsets.
        let mut state = 0x1234_5678_u32;
        let mut bytes = [0; 64];
        for len in 0..=bytes.len() {
            for _ in 0..1000 {
                for byte in &mut bytes[..len] {
                    state ^= state << 13;
                    state ^= state >> 17;
                    state ^= state << 5;
                    *byte = state.to_le_bytes()[0];
                }
                check(&bytes[..len]);
            }
        }
    }

    #[test]
    fn fold_after_partial_consumption() {
        for bytes in [
            b"abcdefghijklmnopqrstuvwxyz0123456789".as_slice(),
            "aé日本語😀z".as_bytes(),
            b"a\xE2\x82b\xED\xA0\x80z".as_slice(),
        ] {
            let expected: Vec<_> = String::from_utf8_lossy(bytes)
                .chars()
                .map(u32::from)
                .collect();
            for consumed in 0..=expected.len() {
                let mut iter = CodePoints::new(bytes);
                for &value in &expected[..consumed] {
                    assert_eq!(iter.next(), Some(value));
                }
                let tail = iter.fold(Vec::new(), |mut values, value| {
                    values.push(value);
                    values
                });
                assert_eq!(tail, expected[consumed..]);
            }
        }
    }

    fn check(bytes: &[u8]) {
        assert!(
            CodePoints::new(bytes).eq(String::from_utf8_lossy(bytes).chars().map(u32::from)),
            "{bytes:x?}"
        );
        let folded = CodePoints::new(bytes).fold(Vec::new(), |mut values, value| {
            values.push(value);
            values
        });
        assert!(
            folded
                .into_iter()
                .eq(String::from_utf8_lossy(bytes).chars().map(u32::from)),
            "fold: {bytes:x?}"
        );
    }
}
