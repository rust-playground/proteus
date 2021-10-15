#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion, Throughput};
use proteus::{actions, TransformBuilder};
use serde_json::Value;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("json");

    for (name, input, trans) in [
        (
            "constant",
            r#"{"top": "value"}"#,
            TransformBuilder::default()
                .add_actions(actions!((r#"const("Dean Karn")"#, "full_name")).unwrap())
                .build()
                .unwrap(),
        ),
        (
            "1_top_level",
            r#"{"top": "value"}"#,
            TransformBuilder::default()
                .add_actions(actions!(("top", "new")).unwrap())
                .build()
                .unwrap(),
        ),
        (
            "10_top_level",
            r#"
            {
                "top1": "value",
                "top2": "value",
                "top3": "value",
                "top4": "value",
                "top5": "value",
                "top6": "value",
                "top7": "value",
                "top8": "value",
                "top9": "value",
                "top10": "value"
            }"#,
            TransformBuilder::default()
                .add_actions(
                    actions!(
                        ("top1", "new1"),
                        ("top2", "new2"),
                        ("top3", "new3"),
                        ("top4", "new4"),
                        ("top5", "new5"),
                        ("top6", "new6"),
                        ("top7", "new7"),
                        ("top8", "new8"),
                        ("top9", "new9"),
                        ("top10", "new10")
                    )
                    .unwrap(),
                )
                .build()
                .unwrap(),
        ),
        (
            "join",
            r#"
            {
                "first_name": "Dean",
                "last_name": "Karn",
                "meta": {
                    "middle_name":"Peter"
                }
            }"#,
            TransformBuilder::default()
                .add_actions(
                    actions!((
                        r#"join(" ", const("Mr."), first_name, meta.middle_name, last_name)"#,
                        "full_name"
                    ))
                    .unwrap(),
                )
                .build()
                .unwrap(),
        ),
    ]
    .iter()
    {
        let source: Value = serde_json::from_str(input).unwrap();
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new(*name, ""), &source, |b, source| {
            b.iter(|| trans.apply(source));
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
