use bstr::B;
use feruca::{Collator, Tailoring};
use std::cmp::Ordering;

#[test]
fn bytes_auto() {
    let a = "Hélène";
    let b = "Héloïse";

    let x = B(a);
    let y = B(b);

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
