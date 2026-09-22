#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! AIDD 对抗 / 边界用例（特性 005）。
//!
//! 候选由 AI 生成，逐条人工复核后仅保留「结论=保留」项；丢弃项登记于 PR 描述。
//!
//! // AIDD: 储蓄率表 T20300 被当作 T20100 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §4 近义非同 ID 不得互换 | 结论=保留
//! // AIDD: 非 NIPA 数据集的合成占位表号被白名单误拒 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §3 白名单仅适用 NIPA | 结论=保留
//! // AIDD: line_number 取 0 与取负值 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §2 行号须 ≥ 1 | 结论=保留
//! // AIDD: T20100 被标成 quarterly | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §3 表级频率须与清单一致 | 结论=保留
//! // AIDD: 合法零值 0.0 与缺失混淆 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §2 缺失不得折算为 0 | 结论=保留
//! // AIDD: 闰日 2024-06-30 与 2025-02-29 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §2 日按月份与闰年校验 | 结论=保留
//! // AIDD: GDI 候选 T11000 被当作已入白名单 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §4 候选待 live 核验 | 结论=保留
//! // AIDD: 布尔与数组型观测值 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §7 值形态白名单 | 结论=保留
//! // AIDD: 空 records 数组 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §7 解析器边界 | 结论=保留
//! // AIDD: line_number 取 u32 上界 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §2 行号为无符号整数 | 结论=保留

use beax::{
    ensure_claim_local, ensure_nipa_table, ensure_not_silent_substitution, is_known_nipa_table,
    parse_bea_observations, validate_date, BeaClaim, BeaErrorKind, BeaMissingReason, BeaValue,
    Date, Frequency, T10101, T11000, T20100, T20300, T20600,
};

fn record(table_id: &str, line_number: &str, value: &str, frequency: &str) -> String {
    format!(
        r#"{{"records": [{{"dataset_id": "NIPA", "table_id": "{table_id}",
            "line_number": {line_number}, "date": "2026-06-30", "value": {value},
            "unit": "Index", "frequency": "{frequency}"}}]}}"#
    )
}

/// 边界：储蓄率表 `T20300` / `T20600` 与 `T20100` 语义不同，不得互换，也不在白名单内。
#[test]
fn savings_tables_are_not_interchangeable_with_t20100() {
    for table in [T20300, T20600] {
        assert!(ensure_not_silent_substitution(T20100, table).is_err());
        assert!(!is_known_nipa_table(table));
        assert!(ensure_nipa_table(table).is_err());
        assert!(parse_bea_observations(&record(table, "1", "1.0", "monthly")).is_err());
    }
    assert!(ensure_not_silent_substitution(T20100, T10101).is_ok());
}

/// 边界：NIPA 白名单只对 `dataset_id == NIPA` 生效；非 NIPA 的占位表号不得被误拒。
#[test]
fn table_whitelist_only_applies_to_nipa() {
    let non_nipa = r#"{"records": [
        {"dataset_id": "InputOutput", "table_id": "SYN-IO-1", "line_number": 1,
         "date": "2026-06-30", "value": 1.0, "unit": "Index", "frequency": "annual"}
    ]}"#;
    assert!(parse_bea_observations(non_nipa).is_ok());

    let nipa_placeholder = record("SYN-IO-1", "1", "1.0", "monthly");
    assert!(parse_bea_observations(&nipa_placeholder).is_err());
}

/// 边界：`line_number` 必须 ≥ 1；负值在反序列化阶段即失败。
#[test]
fn line_number_must_be_positive() {
    assert!(parse_bea_observations(&record(T10101, "0", "1.0", "quarterly")).is_err());
    assert!(parse_bea_observations(&record(T10101, "-1", "1.0", "quarterly")).is_err());
    assert!(parse_bea_observations(&record(T10101, "1", "1.0", "quarterly")).is_ok());
    // u32 上界仍是合法行号（本层不对行号上界另设约束）。
    assert!(
        parse_bea_observations(&record(T10101, &u32::MAX.to_string(), "1.0", "quarterly")).is_ok()
    );
}

/// 边界：`T20100` 的清单频率为月频，标成季频必须被拒。
#[test]
fn t20100_frequency_drift_is_rejected() {
    assert!(parse_bea_observations(&record(T20100, "1", "1.0", "quarterly")).is_err());
    assert!(parse_bea_observations(&record(T20100, "1", "1.0", "monthly")).is_ok());
}

/// 边界：合法零值与「缺失」必须可区分；缺失不得折算为 0。
#[test]
fn zero_is_a_reported_value_not_a_missing_marker() {
    let zero = parse_bea_observations(&record(T10101, "1", "0.0", "quarterly")).expect("零值合法");
    assert_eq!(zero[0].value.as_f64(), Some(0.0));
    assert_eq!(zero[0].value, BeaValue::Present(0.0));

    let missing =
        parse_bea_observations(&record(T10101, "1", r#"".""#, "quarterly")).expect("缺失合法");
    assert!(missing[0].value.is_missing());
    assert_ne!(missing[0].value, BeaValue::Present(0.0));
    assert_eq!(
        missing[0].value,
        BeaValue::Missing(BeaMissingReason::NoObservation)
    );
}

/// 边界：闰日只在闰年合法；五位年份被拒。
#[test]
fn calendar_boundaries() {
    assert!(validate_date(&Date::new(2024, 2, 29).expect("闰年")).is_ok());
    assert!(Date::new(2025, 2, 29).is_err());
    assert!(Date::parse("10000-01-01").is_err());
    assert!(Date::parse("9999-12-31").is_ok());
    assert!(Date::parse("2026-06-30 00:00").is_err());
}

/// 边界：GDI 候选 `T11000` MUST NOT 被表述为已入白名单，也不得与 GDP 互换。
#[test]
fn gdi_candidate_is_not_whitelisted() {
    assert!(!is_known_nipa_table(T11000));
    assert!(ensure_nipa_table(T11000).is_err());
    assert!(ensure_not_silent_substitution(T11000, T10101).is_ok());
    assert!(ensure_not_silent_substitution("GDP", "GDI").is_err());
    assert!(ensure_not_silent_substitution("GDPNow", "GDP").is_err());
}

/// 边界：布尔与数组型观测值不在值形态白名单内，必须原子失败。
#[test]
fn non_numeric_value_forms_are_rejected() {
    for bad in ["true", "[1]", "{}"] {
        let err =
            parse_bea_observations(&record(T10101, "1", bad, "quarterly")).expect_err("应拒绝");
        assert_eq!(err.kind(), BeaErrorKind::Invalid, "{bad}");
    }
}

/// 边界：空 `records` 是合法输入（空集合），不是错误。
#[test]
fn empty_records_is_a_valid_input() {
    assert!(parse_bea_observations(r#"{"records": []}"#)
        .expect("空集合合法")
        .is_empty());
}

/// 边界：曲线产品与核心 PCE 主责的主张不得因调用顺序而放行。
#[test]
fn rejection_is_independent_of_call_order() {
    let _ = ensure_claim_local(BeaClaim::SupplementaryTableValue);
    assert!(ensure_claim_local(BeaClaim::YieldCurveConstruction).is_err());
    assert!(ensure_claim_local(BeaClaim::CorePcePrimary).is_err());
    // 频率记号只接受小写。
    assert!(Frequency::parse("Quarterly").is_err());
    assert_eq!(
        Frequency::parse("quarterly").expect("记号"),
        Frequency::Quarterly
    );
}
