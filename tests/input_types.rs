use bstr::B;
use feruca::Collator;
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
fn bytes_literal() {
    let a = b"Theodore";
    let b = b"Th\xE9odore";

    let mut collator = Collator::default();
    let comp = collator.collate(a, b);
    assert_eq!(comp, Ordering::Less);
}
