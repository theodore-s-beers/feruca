use feruca::{Collator, Tailoring};
use std::cmp::Ordering;

#[test]
fn updated_shift_boundaries_preserve_character_order() {
    for shifting in [false, true] {
        let mut collator = Collator::new(Tailoring::default(), shifting, false);
        // The old numeric SHIFT boundaries split these pairs in the beta table
        for (a, b) in [("\u{109F2}b", "\u{109F3}!"), ("\u{1445E}b", "\u{1445F}!")] {
            assert_eq!(collator.collate(a, b), Ordering::Less);
            assert_eq!(collator.collate(b, a), Ordering::Greater);
        }
    }
}

#[test]
fn cldr_and_ducet_retain_different_shifted_symbol_behavior() {
    let mut ducet = Collator::new(Tailoring::Ducet, true, false);
    let mut cldr = Collator::new(Tailoring::default(), true, false);
    // Grave accent is variable in DUCET but not in CLDR root table
    assert_eq!(ducet.collate("`b", "a"), Ordering::Greater);
    assert_eq!(cldr.collate("`b", "a"), Ordering::Less);
}
