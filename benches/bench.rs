#[macro_use]
extern crate criterion;

use criterion::{Criterion, Throughput};
use proteus::{actions, TransformBuilder};
use serde_json::Value;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("top_level");
    for (name, input, trans) in [
        (
            "1",
            r#"{"top": "value"}"#,
            TransformBuilder::default()
                .add_actions(actions!(("top", "new")).unwrap())
                .build()
                .unwrap(),
        ),
        (
            "10",
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
            "20",
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
                "top10": "value",
                "top11": "value",
                "top12": "value",
                "top13": "value",
                "top14": "value",
                "top15": "value",
                "top16": "value",
                "top17": "value",
                "top18": "value",
                "top19": "value",
                "top20": "value"
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
                        ("top10", "new10"),
                        ("top11", "new11"),
                        ("top12", "new12"),
                        ("top13", "new13"),
                        ("top14", "new14"),
                        ("top15", "new15"),
                        ("top16", "new16"),
                        ("top17", "new17"),
                        ("top18", "new18"),
                        ("top19", "new19"),
                        ("top20", "new20")
                    )
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
        group.bench_function(*name, |b| {
            b.iter(|| {
                let _res = trans.apply(&source);
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("constant");
    for (name, input, trans) in [
        (
            "string",
            "null",
            TransformBuilder::default()
                .add_actions(actions!((r#"const("Dean Karn")"#, "string")).unwrap())
                .build()
                .unwrap(),
        ),
        (
            "number",
            "null",
            TransformBuilder::default()
                .add_actions(actions!((r#"const(1)"#, "number")).unwrap())
                .build()
                .unwrap(),
        ),
        (
            "array",
            "null",
            TransformBuilder::default()
                .add_actions(actions!((r#"const([1, 2, 3])"#, "array")).unwrap())
                .build()
                .unwrap(),
        ),
        (
            "object",
            "null",
            TransformBuilder::default()
                .add_actions(actions!((r#"const({"key": "value"})"#, "object")).unwrap())
                .build()
                .unwrap(),
        ),
    ]
    .iter()
    {
        let source: Value = serde_json::from_str(input).unwrap();
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_function(*name, |b| {
            b.iter(|| {
                let _res = trans.apply(&source);
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("join");
    for (name, input, trans) in [
        (
            "2",
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
                    actions!((r#"join(" ", first_name, last_name)"#, "full_name")).unwrap(),
                )
                .build()
                .unwrap(),
        ),
        (
            "3",
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
                        r#"join(" ", const("Mr."), first_name, last_name)"#,
                        "full_name"
                    ))
                    .unwrap(),
                )
                .build()
                .unwrap(),
        ),
        (
            "4",
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
        group.bench_function(*name, |b| {
            b.iter(|| {
                let _res = trans.apply(&source);
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("len");
    for (name, input, trans) in [
        (
            "string",
            r#""Dean Karn""#,
            TransformBuilder::default()
                .add_actions(actions!(("len()", "string")).unwrap())
                .build()
                .unwrap(),
        ),
        (
            "array",
            r#"[1, 2, 3]"#,
            TransformBuilder::default()
                .add_actions(actions!(("len()", "array")).unwrap())
                .build()
                .unwrap(),
        ),
        (
            "object",
            r#"{"key": "value"}"#,
            TransformBuilder::default()
                .add_actions(actions!(("len()", "object")).unwrap())
                .build()
                .unwrap(),
        ),
    ]
    .iter()
    {
        let source: Value = serde_json::from_str(input).unwrap();
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_function(*name, |b| {
            b.iter(|| {
                let _res = trans.apply(&source);
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("sum");
    for (name, input, trans) in [
        (
            "constant",
            "null",
            TransformBuilder::default()
                .add_actions(actions!(("sum(const(1))", "sum")).unwrap())
                .build()
                .unwrap(),
        ),
        (
            "two_numbers",
            r#"{"key1": 1, "key2": 1}"#,
            TransformBuilder::default()
                .add_actions(actions!(("sum(key1, key2)", "sum")).unwrap())
                .build()
                .unwrap(),
        ),
        (
            "array",
            "[1, 2, 3]",
            TransformBuilder::default()
                .add_actions(actions!(("sum()", "sum")).unwrap())
                .build()
                .unwrap(),
        ),
    ]
    .iter()
    {
        let source: Value = serde_json::from_str(input).unwrap();
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_function(*name, |b| {
            b.iter(|| {
                let _res = trans.apply(&source);
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("trim");
    for (name, input, trans) in [
        (
            "start_and_end",
            r#"{"key":" value "}"#,
            TransformBuilder::default()
                .add_actions(actions!(("trim(key)", "trim")).unwrap())
                .build()
                .unwrap(),
        ),
        (
            "start",
            r#"{"key":" value "}"#,
            TransformBuilder::default()
                .add_actions(actions!(("trim_start(key)", "trim_start")).unwrap())
                .build()
                .unwrap(),
        ),
        (
            "end",
            r#"{"key":" value "}"#,
            TransformBuilder::default()
                .add_actions(actions!(("trim_end(key)", "trim_end")).unwrap())
                .build()
                .unwrap(),
        ),
    ]
    .iter()
    {
        let source: Value = serde_json::from_str(input).unwrap();
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_function(*name, |b| {
            b.iter(|| {
                let _res = trans.apply(&source);
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("strip");
    for (name, input, trans) in [
        (
            "prefix",
            r#"{"key":"value"}"#,
            TransformBuilder::default()
                .add_actions(actions!((r#"strip_prefix("v", key)"#, "prefix")).unwrap())
                .build()
                .unwrap(),
        ),
        (
            "suffix",
            r#"{"key":"value"}"#,
            TransformBuilder::default()
                .add_actions(actions!((r#"strip_suffix("e", key)"#, "suffix")).unwrap())
                .build()
                .unwrap(),
        ),
    ]
    .iter()
    {
        let source: Value = serde_json::from_str(input).unwrap();
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_function(*name, |b| {
            b.iter(|| {
                let _res = trans.apply(&source);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
