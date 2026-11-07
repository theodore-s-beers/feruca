use feruca::{Collator, Tailoring};
use std::cmp::Ordering;

#[test]
fn bytes_auto() {
    let a = "Hélène";
    let b = "Héloïse";

    let x = a.as_bytes();
    let y = b.as_bytes();

    let mut collator = Collator::default();
    let comp = collator.collate(x, y);
    assert_eq!(comp, Ordering::Less);
}

#[test]
fn invalid_utf8_uses_replacement_character() {
    let a = b"Th\xE9odore".as_slice();
    let b = "Th\u{FFFD}odore".as_bytes();

    // Disable byte-value tiebreaking so equivalent decoded text compares equal.
    let mut collator = Collator::new(Tailoring::default(), true, false);
    let comp = collator.collate(a, b);
    assert_eq!(comp, Ordering::Equal);
}

#[test]
fn malformed_utf8_matches_maximal_subpart_replacement() {
    for (bytes, text) in [
        (b"\xE2\x82".as_slice(), "�"),
        (b"\xE2\x82A".as_slice(), "�A"),
        (b"\xED\xA0\x80".as_slice(), "���"),
        (b"\xF0\x90\x80".as_slice(), "�"),
        (b"\xF4\x90\x80\x80".as_slice(), "����"),
    ] {
        for shifting in [false, true] {
            let mut collator = Collator::new(Tailoring::default(), shifting, false);
            assert_eq!(collator.collate(bytes, text.as_bytes()), Ordering::Equal);
            assert_eq!(collator.collate(text.as_bytes(), bytes), Ordering::Equal);
        }
    }
}
