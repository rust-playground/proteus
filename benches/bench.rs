#[macro_use]
extern crate criterion;

use criterion::{Benchmark, Criterion, Throughput};
use proteus::{Parsable, Parser, TransformBuilder};
use serde_json::Value;

fn criterion_benchmark(c: &mut Criterion) {
    let action = Parser::default()
        .parse(r#"const("Dean Karn")"#, "full_name")
        .unwrap();
    let trans = TransformBuilder::default()
        .add_action(action)
        .build()
        .unwrap();

    let input = r#"
    {
        "top": "value"
    }"#;
    let source: Value = serde_json::from_str(input).unwrap();

    c.bench(
        "json",
        Benchmark::new("constant", move |b| {
            b.iter(|| {
                let _res = trans.apply(&source).unwrap();
            })
        })
        .throughput(Throughput::Bytes(input.as_bytes().len() as u64)),
    );

    let action = Parser::default().parse("top", "new").unwrap();
    let trans = TransformBuilder::default()
        .add_action(action)
        .build()
        .unwrap();

    let input = r#"
    {
        "top": "value"
    }"#;
    let source: Value = serde_json::from_str(input).unwrap();

    c.bench(
        "json",
        Benchmark::new("1_top_level", move |b| {
            b.iter(|| {
                let _res = trans.apply(&source).unwrap();
            })
        })
        .throughput(Throughput::Bytes(input.as_bytes().len() as u64)),
    );

    let actions = Parser::default()
        .parse_multi(&[
            Parsable::new("top1", "new1"),
            Parsable::new("top2", "new2"),
            Parsable::new("top3", "new3"),
            Parsable::new("top4", "new4"),
            Parsable::new("top5", "new5"),
            Parsable::new("top6", "new6"),
            Parsable::new("top7", "new7"),
            Parsable::new("top8", "new8"),
            Parsable::new("top9", "new9"),
            Parsable::new("top10", "new10"),
        ])
        .unwrap();

    let trans = TransformBuilder::default()
        .add_actions(actions)
        .build()
        .unwrap();

    let input = r#"
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
    }"#;
    let source: Value = serde_json::from_str(input).unwrap();

    c.bench(
        "json",
        Benchmark::new("10_top_level", move |b| {
            b.iter(|| {
                let _res = trans.apply(&source).unwrap();
            })
        })
        .throughput(Throughput::Bytes(input.as_bytes().len() as u64)),
    );

    let action = Parser::default()
        .parse(
            r#"join(" ", const("Mr."), first_name, meta.middle_name, last_name)"#,
            "full_name",
        )
        .unwrap();
    let trans = TransformBuilder::default()
        .add_action(action)
        .build()
        .unwrap();

    let input = r#"
    {
        "first_name": "Dean",
        "last_name": "Karn",
        "meta": {
            "middle_name":"Peter"
        }
    }"#;
    let source: Value = serde_json::from_str(input).unwrap();

    c.bench(
        "json",
        Benchmark::new("join", move |b| {
            b.iter(|| {
                let _res = trans.apply(&source).unwrap();
            })
        })
        .throughput(Throughput::Bytes(input.as_bytes().len() as u64)),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
