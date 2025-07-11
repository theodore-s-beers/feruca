use criterion::{Criterion, criterion_group, criterion_main};
use feruca::{Collator, Tailoring};
use std::cmp::Ordering;

fn conformance(path: &str, collator: &mut Collator) {
    let test_data = std::fs::read_to_string(path).unwrap();

    let mut max_line = String::new();
    let mut test_string = String::new();

    'outer: for line in test_data.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        test_string.clear();

        for s in line.split(' ') {
            let val = u32::from_str_radix(s, 16).unwrap();

            // Skip lines containing surrogate code points; they would all be replaced with U+FFFD.
            // Conformant implementations are explicitly allowed to do this.
            if (0xD800..=0xDFFF).contains(&val) {
                continue 'outer;
            }

            test_string.push(char::from_u32(val).unwrap());
        }

        let comparison = collator.collate(&test_string, &max_line);
        if comparison == Ordering::Less {
            panic!();
        }

        std::mem::swap(&mut max_line, &mut test_string);
    }
}

fn ducet_ni(c: &mut Criterion) {
    c.bench_function("DUCET, non-ignorable", |b| {
        b.iter(|| {
            conformance(
                "test-data/cldr-46_1/CollationTest_NON_IGNORABLE_SHORT.txt",
                &mut Collator::new(Tailoring::Ducet, false, false),
            )
        })
    });
}

fn ducet_shifted(c: &mut Criterion) {
    c.bench_function("DUCET, shifted", |b| {
        b.iter(|| {
            conformance(
                "test-data/cldr-46_1/CollationTest_SHIFTED_SHORT.txt",
                &mut Collator::new(Tailoring::Ducet, true, false),
            )
        })
    });
}

fn cldr_ni(c: &mut Criterion) {
    c.bench_function("CLDR, non-ignorable", |b| {
        b.iter(|| {
            conformance(
                "test-data/cldr-46_1/CollationTest_CLDR_NON_IGNORABLE_SHORT.txt",
                &mut Collator::new(Tailoring::default(), false, false),
            )
        })
    });
}

fn cldr_shifted(c: &mut Criterion) {
    c.bench_function("CLDR, shifted", |b| {
        b.iter(|| {
            conformance(
                "test-data/cldr-46_1/CollationTest_CLDR_SHIFTED_SHORT.txt",
                &mut Collator::new(Tailoring::default(), true, false),
            )
        })
    });
}

criterion_group!(benches, ducet_ni, ducet_shifted, cldr_ni, cldr_shifted);
criterion_main!(benches);
