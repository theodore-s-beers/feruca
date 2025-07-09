use feruca::{Collator, Locale, Tailoring};
use std::cmp::Ordering;

#[test]
fn arabic_interleaved() {
    let mut names = vec!["Bob", "Alice", "أحمد"];
    let expected = vec!["Alice", "أحمد", "Bob"];

    let mut collator = Collator::new(Tailoring::Cldr(Locale::ArabicInterleaved), true, true);
    names.sort_unstable_by(|a, b| collator.collate(a, b));

    assert_eq!(names, expected);
}

#[test]
fn arabic_script() {
    let persian = "ی";
    let latin = "a";

    let mut collator = Collator::new(Tailoring::Cldr(Locale::ArabicScript), true, true);
    let comp = collator.collate(persian, latin);
    assert_eq!(comp, Ordering::Less);
}

#[test]
fn capitalization() {
    let a = "Američane";
    let b = "ameriške";

    let mut collator = Collator::default();
    let comp = collator.collate(a, b);
    assert_eq!(comp, Ordering::Less);
}

#[test]
fn fdfa() {
    // This will panic if the CEA length is not doubled early enough.
    // U+FDFA has 18 sets of collation weights, more than any other code point.
    let a = "llllllllllllllllllllllllllllllllllllllllllllllﷺ";
    let b = "ā";

    let mut collator = Collator::default();
    let comp = collator.collate(a, b);
    assert_eq!(comp, Ordering::Greater);
}
