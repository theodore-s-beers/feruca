use bstr::B;
use feruca::{collate, CollationOptions};
use std::cmp::Ordering;

#[test]
fn bytes_literal() {
    let a = b"Theodore";
    let b = b"Th\xE9odore";

    let comp = collate(a, b, CollationOptions::default());
    assert_eq!(comp, Ordering::Less);
}

#[test]
fn bytes_auto() {
    let a = "Hélène";
    let b = "Héloïse";

    let x = B(a);
    let y = B(b);

    let comp = collate(&x, &y, CollationOptions::default());
    assert_eq!(comp, Ordering::Less);
}
