use feruca::{Collator, Locale, Tailoring};
use std::cmp::Ordering;

#[test]
fn arabic_script() {
    let persian = "ی";
    let latin = "a";

    let collator = Collator::new(Tailoring::Cldr(Locale::ArabicScript), true);
    let comp = collator.collate(persian, latin);
    assert_eq!(comp, Ordering::Less);
}

#[test]
fn capitalization() {
    let a = "Američane";
    let b = "ameriške";

    let collator = Collator::default();
    let comp = collator.collate(a, b);
    assert_eq!(comp, Ordering::Less);
}
