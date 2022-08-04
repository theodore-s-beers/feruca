use criterion::{criterion_group, criterion_main, Criterion};
use feruca::{Collator, Tailoring};
use std::cmp::Ordering;

fn conformance(path: &str, mut collator: Collator) {
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

        let comparison = collator.collate_no_tiebreak(&test_string, &max_line);
        if comparison == Ordering::Less {
            panic!();
        }

        max_line = test_string;
    }
}

fn ducet_ni(c: &mut Criterion) {
    c.bench_function("DUCET, non-ignorable", |b| {
        b.iter(|| {
            conformance(
                "test-data/CollationTest_NON_IGNORABLE_SHORT.txt",
                Collator::new(Tailoring::Ducet, false),
            )
        })
    });
}

fn ducet_shifted(c: &mut Criterion) {
    c.bench_function("DUCET, shifted", |b| {
        b.iter(|| {
            conformance(
                "test-data/CollationTest_SHIFTED_SHORT.txt",
                Collator::new(Tailoring::Ducet, true),
            )
        })
    });
}

fn cldr_ni(c: &mut Criterion) {
    c.bench_function("CLDR, non-ignorable", |b| {
        b.iter(|| {
            conformance(
                "test-data/CollationTest_CLDR_NON_IGNORABLE_SHORT.txt",
                Collator::new(Tailoring::default(), false),
            )
        })
    });
}

fn cldr_shifted(c: &mut Criterion) {
    c.bench_function("CLDR, shifted", |b| {
        b.iter(|| {
            conformance(
                "test-data/CollationTest_CLDR_SHIFTED_SHORT.txt",
                Collator::default(),
            )
        })
    });
}

criterion_group!(benches, ducet_ni, ducet_shifted, cldr_ni, cldr_shifted);
criterion_main!(benches);
