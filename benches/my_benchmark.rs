/// The intention here is to demonstrate what seems intuitively true:
/// simple substring search is faster than the regex equivalent. This
/// test isn't meant to be conclusive, merely suggestive.
use criterion::{Criterion, criterion_group, criterion_main};
use regex::Regex;
use std::hint::black_box;

fn basic_contains(haystack: &str, needle: &str) -> bool {
    return haystack.contains(needle);
}

fn regex_contains(haystack: &str, needle: &Regex) -> bool {
    return needle.is_match(haystack);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("basic string contains", |b| {
        b.iter(|| basic_contains(black_box("Hello"), black_box("e")))
    });
    c.bench_function("regex contains", |b| {
        let needle_regex = Regex::new("e").unwrap();
        b.iter(|| regex_contains(black_box("Hello"), black_box(&needle_regex)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
