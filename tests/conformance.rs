use feruca::{Collator, Tailoring};
use std::cmp::Ordering;

fn conformance(path: &str, collator: &mut Collator) {
    let test_data = std::fs::read_to_string(path).unwrap();

    let mut max_line = String::new();
    let mut test_string = String::new();
    let mut previous_line_number = 0;
    let mut previous_code_points = "";
    let mut comparisons = 0;
    let mut failures = 0;
    let mut examples = Vec::new();

    'outer: for (index, line) in test_data.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        test_string.clear();

        for s in line.split_whitespace() {
            let val = u32::from_str_radix(s, 16).unwrap();

            // Skip lines with surrogate points; they'd all be replaced with U+FFFD.
            // Conformant implementations are explicitly allowed to do this.
            if (0xD800..=0xDFFF).contains(&val) {
                continue 'outer;
            }

            test_string.push(char::from_u32(val).unwrap());
        }

        if previous_line_number != 0 {
            comparisons += 1;
            if collator.collate(&test_string, &max_line) == Ordering::Less {
                failures += 1;
                if examples.len() < 8 {
                    examples.push(format!(
                        "lines {previous_line_number} -> {}: [{previous_code_points}] > [{line}]",
                        index + 1,
                    ));
                }
            }
        }

        previous_line_number = index + 1;
        previous_code_points = line;
        std::mem::swap(&mut max_line, &mut test_string);
    }

    assert!(comparisons > 0, "{path}: no comparable test data");
    assert_eq!(
        failures,
        0,
        "{path}: {failures} out-of-order pairs in {comparisons} comparisons; first failures:\n{}",
        examples.join("\n"),
    );
}

#[test]
fn ducet_non_ignorable() {
    conformance(
        "test-data/18/CollationTest_NON_IGNORABLE_SHORT.txt",
        &mut Collator::new(Tailoring::Ducet, false, false),
    );
}

#[test]
fn ducet_shifted() {
    conformance(
        "test-data/18/CollationTest_SHIFTED_SHORT.txt",
        &mut Collator::new(Tailoring::Ducet, true, false),
    );
}

#[test]
fn cldr_non_ignorable() {
    conformance(
        "test-data/18/CollationTest_CLDR_NON_IGNORABLE_SHORT.txt",
        &mut Collator::new(Tailoring::default(), false, false),
    );
}

#[test]
fn cldr_shifted() {
    conformance(
        "test-data/18/CollationTest_CLDR_SHIFTED_SHORT.txt",
        &mut Collator::new(Tailoring::default(), true, false),
    );
}

#[cfg(feature = "pipeline-stats")]
#[test]
fn lazy_utf8_primary_path_conforms() {
    let a = "l".repeat(40);
    let b = "m".repeat(40);

    let mut collator = Collator::new(Tailoring::default(), false, false);
    let comparison = collator.collate(&a, &b);

    assert_eq!(comparison, Ordering::Less);
    assert_eq!(collator.stats().lazy_utf8_primary_attempts, 1);
    assert_eq!(collator.stats().lazy_utf8_primary_resolved, 1);
}
