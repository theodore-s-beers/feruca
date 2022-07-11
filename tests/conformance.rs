use feruca::{collate_no_tiebreak, CollationOptions, KeysSource};
use std::cmp::Ordering;

fn conformance(path: &str, options: CollationOptions) {
    let test_data = std::fs::read_to_string(path).unwrap();

    let mut max_line = String::new();

    for line in test_data.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let hex_values: Vec<&str> = line.split(' ').collect();
        let mut test_string = String::new();

        for s in hex_values {
            let val = u32::from_str_radix(s, 16).unwrap();
            // We have to use an unsafe function for the conformance tests because they
            // deliberately introduce invalid character values.
            let c = unsafe { std::char::from_u32_unchecked(val) };
            test_string.push(c);
        }

        let comparison = collate_no_tiebreak(&test_string, &max_line, options);
        if comparison == Ordering::Less {
            panic!();
        }

        max_line = test_string;
    }
}

#[test]
fn ducet_non_ignorable() {
    let path = "test-data/CollationTest_NON_IGNORABLE_SHORT.txt";

    let options = CollationOptions {
        keys_source: KeysSource::Ducet,
        shifting: false,
    };

    conformance(path, options);
}

#[test]
fn ducet_shifted() {
    let path = "test-data/CollationTest_SHIFTED_SHORT.txt";

    let options = CollationOptions {
        keys_source: KeysSource::Ducet,
        shifting: true,
    };

    conformance(path, options);
}

#[test]
fn cldr_non_ignorable() {
    let path = "test-data/CollationTest_CLDR_NON_IGNORABLE_SHORT.txt";

    let options = CollationOptions {
        keys_source: KeysSource::Cldr,
        shifting: false,
    };

    conformance(path, options);
}

#[test]
fn cldr_shifted() {
    let path = "test-data/CollationTest_CLDR_SHIFTED_SHORT.txt";

    let options = CollationOptions {
        keys_source: KeysSource::Cldr,
        shifting: true,
    };

    conformance(path, options);
}
