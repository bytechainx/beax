#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! beax 热路径：离线解析 + 跨源守卫 + 授权判定。
//!
//! 本基准是**自计时的手写基准**（`harness = false`，自带 `fn main`），
//! 不是可复现的性能证据：它不构成任何 SLA 或能力宣称。

use std::hint::black_box;
use std::time::Instant;

use beax::{
    authorize_bea, documented_bea_evidence, ensure_claim_local, ensure_not_silent_substitution,
    parse_bea_observations, BeaAccessMode, BeaAuthorization, BeaClaim, Date, T10101,
};

/// 迭代次数（固定值，不读环境变量）。
const ITERS: u32 = 20_000;

/// 热路径输入：两条记录的合成样本（非真实源数据）。
const INPUT: &str = r#"{"records": [
    {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
     "value": 30000.0, "unit": "Billions of dollars", "frequency": "quarterly"},
    {"dataset_id": "NIPA", "table_id": "T20100", "line_number": 12, "date": "2026-06-30",
     "value": 19000.0, "unit": "Billions of dollars", "frequency": "monthly"}
]}"#;

fn main() {
    let as_of = Date::new(2026, 8, 20).expect("日期合法");
    let evidence = documented_bea_evidence();
    let start = Instant::now();
    let mut acc = 0u64;
    for _ in 0..ITERS {
        let observations = parse_bea_observations(INPUT).expect("解析");
        acc = acc.wrapping_add(observations.len() as u64);
        if matches!(
            authorize_bea(Some(&evidence), BeaAccessMode::ReferenceOnly, as_of),
            BeaAuthorization::Authorized { .. }
        ) {
            acc = acc.wrapping_add(1);
        }
        if ensure_claim_local(BeaClaim::CorePcePrimary).is_err() {
            acc = acc.wrapping_add(1);
        }
        if ensure_not_silent_substitution(T10101, "T20100").is_ok() {
            acc = acc.wrapping_add(1);
        }
        black_box(&observations);
    }
    let elapsed = start.elapsed();
    println!(
        "bench_beax_hot_path: iters={ITERS} total={elapsed:?} per_iter={:?} acc={}",
        elapsed / ITERS,
        black_box(acc)
    );
}
