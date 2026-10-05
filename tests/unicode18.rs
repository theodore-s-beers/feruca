use feruca::{Collator, Tailoring};
use std::cmp::Ordering;

#[test]
fn tibetan_discontiguous_three_character_contractions() {
    for shifting in [false, true] {
        let mut collator = Collator::new(Tailoring::Ducet, shifting, false);
        for base in ['\u{0FB2}', '\u{0FB3}'] {
            for vowel in ['\u{0F72}', '\u{0F74}', '\u{0F80}'] {
                let interrupted = format!("{base}\u{0334}\u{0F71}{vowel}");
                let contiguous = format!("{base}\u{0F71}{vowel}\u{0334}");
                assert_eq!(collator.collate(&interrupted, &contiguous), Ordering::Equal);
                assert_eq!(collator.collate(&contiguous, &interrupted), Ordering::Equal);
            }
        }
    }
}

#[test]
fn shared_prefix_must_not_split_a_kannada_contraction() {
    for shifting in [false, true] {
        let mut collator = Collator::new(Tailoring::Ducet, shifting, false);
        // Include long prefixes to exercise lazy UTF-8 path as well
        for prefix in ["".to_owned(), "a".repeat(80)] {
            let a = format!("{prefix}\u{0CC8}\u{0CC6}\u{0CC2}\u{0CD6}b");
            let b = format!("{prefix}\u{0CC8}\u{0CC6}\u{0CC2}\u{0CD5}!");
            assert_eq!(collator.collate(&a, &b), Ordering::Less);
            assert_eq!(collator.collate(&b, &a), Ordering::Greater);
        }
    }
}

#[test]
fn new_combining_marks_are_canonically_reordered() {
    for shifting in [false, true] {
        let mut collator = Collator::new(Tailoring::Ducet, shifting, false);
        for mark in ['\u{1ACF}', '\u{1E6E3}'] {
            let a = format!("x{mark}\u{0334}");
            let b = format!("x\u{0334}{mark}");
            assert_eq!(collator.collate(&a, &b), Ordering::Equal);
            assert_eq!(collator.collate(&b, &a), Ordering::Equal);
        }
    }
}

#[test]
fn implicit_weights_use_unicode18_for_both_tables() {
    let mut ducet = Collator::new(Tailoring::Ducet, false, false);
    let mut cldr = Collator::new(Tailoring::default(), false, false);
    // Unicode 18 separates Tangut characters from their components
    assert_eq!(ducet.collate("\u{18AFF}", "\u{18D00}"), Ordering::Greater);
    assert_eq!(cldr.collate("\u{18AFF}", "\u{18D00}"), Ordering::Greater);
    // New Han chars sort before unassigned chars; holes remain unassigned
    assert_eq!(ducet.collate("\u{33479}", "\u{0378}"), Ordering::Less);
    assert_eq!(ducet.collate("\u{2B81E}", "\u{2B820}"), Ordering::Less);
    assert_eq!(ducet.collate("\u{2A6E0}", "\u{2B81F}"), Ordering::Less);
    // Jurchen and Seal precede Han; unassigned Jurchen holes do not
    assert_eq!(ducet.collate("\u{18E00}", "\u{3D000}"), Ordering::Less);
    assert_eq!(ducet.collate("\u{3FC3F}", "\u{4E00}"), Ordering::Less);
    assert_eq!(ducet.collate("\u{19192}", "\u{4E00}"), Ordering::Greater);
}
