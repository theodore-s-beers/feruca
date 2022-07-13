use feruca::{collate, CollationOptions};
use std::cmp::Ordering;

#[test]
fn capitalization() {
    let a = "Američane";
    let b = "ameriške";

    let comp = collate(&a, &b, CollationOptions::default());
    assert_eq!(comp, Ordering::Less);
}
