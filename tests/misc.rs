use feruca::Collator;
use std::cmp::Ordering;

#[test]
fn capitalization() {
    let a = "Američane";
    let b = "ameriške";

    let collator = Collator::default();
    let comp = collator.collate(a, b);
    assert_eq!(comp, Ordering::Less);
}
