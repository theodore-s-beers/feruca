use feruca::{Collator, Tailoring};
use std::cmp::Ordering;

fn conformance(path: &str, collator: &mut Collator) {
    let test_data = std::fs::read_to_string(path).unwrap();

    let mut max_line = String::new();

    'outer: for line in test_data.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let hex_values: Vec<&str> = line.split(' ').collect();
        let mut test_string = String::new();

        for s in hex_values {
            let val = u32::from_str_radix(s, 16).unwrap();

            // Skip lines containing surrogate code points; they would all be replaced with U+FFFD.
            // Conformant implementations are explicitly allowed to do this.
            if val > 55_295 && val < 57_344 {
                continue 'outer;
            }

            test_string.push(char::from_u32(val).unwrap());
        }

        let comparison = collator.collate(&test_string, &max_line);
        if comparison == Ordering::Less {
            panic!();
        }

        max_line = test_string;
    }
}

#[test]
fn ducet_non_ignorable() {
    let path = "test-data/15/CollationTest_NON_IGNORABLE_SHORT.txt";
    let mut collator = Collator::new(Tailoring::Ducet, false, false);
    conformance(path, &mut collator);
}

#[test]
fn ducet_shifted() {
    let path = "test-data/15/CollationTest_SHIFTED_SHORT.txt";
    let mut collator = Collator::new(Tailoring::Ducet, true, false);
    conformance(path, &mut collator);
}

#[test]
fn cldr_non_ignorable() {
    let path = "test-data/15/CollationTest_CLDR_NON_IGNORABLE_SHORT.txt";
    let mut collator = Collator::new(Tailoring::default(), false, false);
    conformance(path, &mut collator);
}

#[test]
fn cldr_shifted() {
    let path = "test-data/15/CollationTest_CLDR_SHIFTED_SHORT.txt";
    let mut collator = Collator::new(Tailoring::default(), true, false);
    conformance(path, &mut collator);
}
