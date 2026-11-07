use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

// Benchmark the private decoder without exposing it in the library's API.
#[allow(dead_code)]
#[path = "../src/utf8.rs"]
mod utf8;

fn inputs(c: &mut Criterion) {
    let cases: Vec<(&str, Vec<u8>)> = vec![
        (
            "ascii",
            "The quick brown fox jumps over the lazy dog. "
                .repeat(32)
                .into_bytes(),
        ),
        (
            "latin",
            "Hélène Peña Émile Straße déjà vu ".repeat(32).into_bytes(),
        ),
        ("arabic", "سلام دنیا العربية فارسی ".repeat(32).into_bytes()),
        (
            "cjk",
            "日本語中文漢字ひらがなカタカナ".repeat(32).into_bytes(),
        ),
        ("emoji", "😀🦀🌍🚀🎉".repeat(32).into_bytes()),
        ("mixed", "Hello سلام 世界 😀 café ".repeat(32).into_bytes()),
        (
            "invalid",
            b"a\xE2\x82b\xED\xA0\x80\xF4\x90\x80\x80\xFF".repeat(32),
        ),
        ("short", "Émile".as_bytes().to_vec()),
    ];
    for (name, bytes) in &cases {
        let mut g = c.benchmark_group(format!("decode/{name}"));
        g.bench_function("all", |b| {
            b.iter(|| {
                utf8::CodePoints::new(black_box(bytes.as_slice())).fold(0u32, u32::wrapping_add)
            })
        });
        g.bench_function("first-two", |b| {
            b.iter(|| {
                utf8::CodePoints::new(black_box(bytes.as_slice()))
                    .take(2)
                    .sum::<u32>()
            })
        });
        g.finish();
    }
    let pairs: Vec<(&str, Vec<u8>, Vec<u8>)> = vec![
        ("short-ascii", b"cat".to_vec(), b"dog".to_vec()),
        (
            "ascii-tail",
            format!("!{}", "a".repeat(512)).into_bytes(),
            format!("?{}", "a".repeat(512)).into_bytes(),
        ),
        (
            "short-latin",
            "Hélène".as_bytes().to_vec(),
            "Héloïse".as_bytes().to_vec(),
        ),
        (
            "normalization",
            "école".repeat(30).into_bytes(),
            "e\u{301}cole".repeat(30).into_bytes(),
        ),
        (
            "later-levels",
            "École".repeat(30).into_bytes(),
            "école".repeat(30).into_bytes(),
        ),
        (
            "arabic",
            "سلام دنیا".as_bytes().to_vec(),
            "سلام عربی".as_bytes().to_vec(),
        ),
        (
            "cjk",
            "日本語中文".as_bytes().to_vec(),
            "中文日本語".as_bytes().to_vec(),
        ),
        (
            "emoji",
            "😀🦀🌍".as_bytes().to_vec(),
            "🚀🎉🦀".as_bytes().to_vec(),
        ),
        (
            "invalid",
            b"a\xE2\x82b\xED\xA0\x80".repeat(30),
            "a�b���".repeat(30).into_bytes(),
        ),
    ];
    for (name, left, right) in &pairs {
        for shifted in [false, true] {
            let mut collator = feruca::Collator::new(feruca::Tailoring::default(), shifted, false);
            let mut g = c.benchmark_group(format!("collate/{name}/{shifted}"));
            g.bench_function("all", |b| {
                b.iter(|| collator.collate(black_box(left), black_box(right)))
            });
            g.finish();
        }
    }
}

criterion_group!(group, inputs);
criterion_main!(group);
