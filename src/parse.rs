//! beax 的离线解析器：自有输入形态 → 观测集合。
//!
//! 入口**只接受字符串**：MUST NOT 接受 URL、HTTP 客户端、认证信息或任何网络参数。
//!
//! 输入形态（本库自定，**不是** BEA API 响应的复制）：一个 JSON 对象
//! `{ "_synthetic"?: bool, "_note"?: string, "records": [ … ] }`，每条记录
//! `{ dataset_id, table_id, line_number, date, value, unit, frequency, vintage? }`。
//!
//! 语义：未知字段**原子失败**；重复身份
//! （`dataset_id` + `table_id` + `line_number` + 期间 + `vintage`）**拒绝**且不去重。

use serde::Deserialize;

use crate::dataset::{ensure_known_dataset, ensure_nipa_table, ensure_table_frequency, NIPA};
use crate::error::{BeaError, BeaResult};
use crate::routing::{ensure_claim_local, BeaClaim};
use crate::value::{
    validate_observation, BeaMissingReason, BeaObservation, BeaUnit, BeaValue, Date, Frequency,
    Period,
};

/// 输入顶层信封。`_synthetic` / `_note` 为合成样本标注，属**已知字段**。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnvelope {
    #[serde(default)]
    _synthetic: Option<bool>,
    #[serde(default)]
    _note: Option<String>,
    records: Vec<RawRecord>,
}

/// 单条输入记录。字段名与清单声明的事实对齐（Dataset / 表号 / 行号 / 期间 / 值 / 单位 / 频率）。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRecord {
    dataset_id: String,
    table_id: String,
    line_number: u32,
    date: String,
    value: serde_json::Value,
    unit: String,
    frequency: String,
    #[serde(default)]
    vintage: Option<String>,
}

/// 解析 beax 的离线输入，返回观测集合。
///
/// 逐条执行：身份主张守卫（本域只承载补充观测）→ Dataset 范围 → NIPA 表白名单 →
/// 频率记号 → 期间投影 → 值形态 → 表级频率一致性 → 值对象校验。任一条失败即整体失败（原子）。
///
/// # Examples
///
/// ```
/// use beax::parse_bea_observations;
///
/// let input = r#"{"records": [
///     {"dataset_id": "NIPA", "table_id": "T20100", "line_number": 12, "date": "2026-06-30",
///      "value": 19000.0, "unit": "Billions of dollars", "frequency": "monthly"}
/// ]}"#;
/// let observations = parse_bea_observations(input)?;
/// assert_eq!(observations[0].table_id, "T20100");
/// # Ok::<(), beax::BeaError>(())
/// ```
///
/// # Errors
///
/// - 输入不是合法 JSON 对象 / 含未知字段 / 缺必需字段 → [`BeaError::Invalid`]
/// - `dataset_id` 不在清单范围内、NIPA 表号不在白名单、频率与清单不符 → [`BeaError::SemanticallyRejected`]
/// - 同一身份出现两次 → [`BeaError::SemanticallyRejected`]
pub fn parse_bea_observations(input: &str) -> BeaResult<Vec<BeaObservation>> {
    let envelope: RawEnvelope = serde_json::from_str(input).map_err(json_error)?;
    let mut observations = Vec::with_capacity(envelope.records.len());
    let mut seen: Vec<(String, String, u32, Period, Option<Date>)> =
        Vec::with_capacity(envelope.records.len());
    for raw in &envelope.records {
        let observation = build_observation(raw)?;
        let identity = (
            observation.dataset_id.clone(),
            observation.table_id.clone(),
            observation.line_number,
            observation.period,
            observation.vintage,
        );
        if seen.contains(&identity) {
            return Err(BeaError::SemanticallyRejected(format!(
                "重复身份（dataset_id + table_id + line_number + 期间 + vintage）：{}/{}#{}",
                observation.dataset_id, observation.table_id, observation.line_number
            )));
        }
        seen.push(identity);
        observations.push(observation);
    }
    Ok(observations)
}

/// 把一条原始记录转成观测对象，并逐层执行守卫。
fn build_observation(raw: &RawRecord) -> BeaResult<BeaObservation> {
    ensure_claim_local(BeaClaim::SupplementaryTableValue)?;
    ensure_known_dataset(&raw.dataset_id)?;
    if raw.dataset_id == NIPA {
        ensure_nipa_table(&raw.table_id)?;
    }
    let frequency = Frequency::parse(&raw.frequency)?;
    let date = Date::parse(&raw.date)?;
    let observation = BeaObservation {
        dataset_id: raw.dataset_id.clone(),
        table_id: raw.table_id.clone(),
        line_number: raw.line_number,
        period: project_period(frequency, date)?,
        value: parse_value(&raw.value)?,
        unit: BeaUnit::new(&raw.unit)?,
        frequency,
        vintage: match raw.vintage.as_deref() {
            Some(text) => Some(Date::parse(text)?),
            None => None,
        },
    };
    validate_observation(&observation)?;
    ensure_table_frequency(&observation)?;
    Ok(observation)
}

/// 按清单频率把源日期投影为业务期间。
///
/// 这是**期间投影**（源日期 → 该频率的期间身份），不是派生指标。
fn project_period(frequency: Frequency, date: Date) -> BeaResult<Period> {
    match frequency {
        Frequency::Daily | Frequency::Weekly | Frequency::Irregular => Ok(Period::Day(date)),
        Frequency::Monthly => Ok(Period::Month {
            year: date.year,
            month: date.month,
        }),
        Frequency::Quarterly => Ok(Period::Quarter {
            year: date.year,
            quarter: quarter_of(date.month)?,
        }),
        Frequency::Annual => Ok(Period::Year(date.year)),
        Frequency::Event => Ok(Period::Event { date }),
    }
}

/// 月 → 季；月份越界时返回 [`BeaError::Invalid`]。
fn quarter_of(month: u8) -> BeaResult<u8> {
    if !(1..=12).contains(&month) {
        return Err(BeaError::Invalid("月份须在 1..=12".into()));
    }
    Ok((month - 1) / 3 + 1)
}

/// 解析值形态：数值 → `Present`；`.` 或 `null` → 具名缺失。
fn parse_value(raw: &serde_json::Value) -> BeaResult<BeaValue> {
    match raw {
        serde_json::Value::Number(number) => match number.as_f64() {
            Some(v) => Ok(BeaValue::Present(v)),
            None => Err(BeaError::Invalid("观测值超出 f64 可表示范围".into())),
        },
        serde_json::Value::String(text) if text == "." => {
            Ok(BeaValue::Missing(BeaMissingReason::NoObservation))
        }
        serde_json::Value::Null => Ok(BeaValue::Missing(BeaMissingReason::NoObservation)),
        _ => Err(BeaError::Invalid(
            "观测值只能为数值、`.` 或 null（MUST NOT 把缺失折算为 0）".into(),
        )),
    }
}

/// 把 JSON 解析错误映射为不泄漏输入的 [`BeaError::Invalid`]。
fn json_error(err: serde_json::Error) -> BeaError {
    BeaError::Invalid(format!(
        "离线输入不是合法的 JSON 形态（第 {} 行第 {} 列）",
        err.line(),
        err.column()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const WELL_FORMED: &str = r#"{
        "_synthetic": true,
        "_note": "合成样本",
        "records": [
            {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
             "value": 30000.0, "unit": "Billions of dollars", "frequency": "quarterly"},
            {"dataset_id": "NIPA", "table_id": "T20100", "line_number": 12, "date": "2026-06-30",
             "value": 19000.0, "unit": "Billions of dollars", "frequency": "monthly"}
        ]
    }"#;

    #[test]
    fn parses_well_formed_input() {
        let observations = parse_bea_observations(WELL_FORMED).expect("应可解析");
        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].dataset_id, "NIPA");
        assert_eq!(observations[0].table_id, "T10101");
        assert_eq!(
            observations[0].period,
            Period::Quarter {
                year: 2026,
                quarter: 2
            }
        );
        assert_eq!(
            observations[1].period,
            Period::Month {
                year: 2026,
                month: 6
            }
        );
        assert!(observations[0].vintage.is_none());
    }

    #[test]
    fn non_nipa_dataset_skips_the_table_whitelist() {
        // 清单未登记非 NIPA 数据集的明细表号，故此处用 `SYN-` 前缀的**合成占位串**。
        let input = r#"{"records": [
            {"dataset_id": "Regional", "table_id": "SYN-REGIONAL-1", "line_number": 1,
             "date": "2026-06-30", "value": 1.0, "unit": "Thousands of dollars",
             "frequency": "annual"}
        ]}"#;
        let observations = parse_bea_observations(input).expect("应可解析");
        assert_eq!(observations[0].dataset_id, "Regional");
        assert_eq!(observations[0].period, Period::Year(2026));
    }

    #[test]
    fn missing_value_is_named_and_not_zero() {
        let input = r#"{"records": [
            {"dataset_id": "NIPA", "table_id": "T10102", "line_number": 3, "date": "2026-06-30",
             "value": ".", "unit": "Index", "frequency": "monthly"},
            {"dataset_id": "NIPA", "table_id": "T10103", "line_number": 4, "date": "2026-06-30",
             "value": null, "unit": "Index", "frequency": "monthly"}
        ]}"#;
        let observations = parse_bea_observations(input).expect("应可解析");
        for observation in &observations {
            assert!(observation.value.is_missing());
            assert_eq!(observation.value.as_f64(), None);
            assert_ne!(observation.value.as_f64(), Some(0.0));
        }
    }

    #[test]
    fn vintage_is_preserved_when_present() {
        let input = r#"{"records": [
            {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
             "value": 1.0, "unit": "Index", "frequency": "quarterly", "vintage": "2026-09-30"}
        ]}"#;
        let observations = parse_bea_observations(input).expect("应可解析");
        assert_eq!(
            observations[0].vintage,
            Some(Date::new(2026, 9, 30).expect("日期合法"))
        );
    }

    #[test]
    fn unknown_field_is_rejected_atomically() {
        let input = r#"{"records": [
            {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
             "value": 1.0, "unit": "Index", "frequency": "quarterly", "endpoint": "x"}
        ]}"#;
        assert!(parse_bea_observations(input).is_err());
        assert!(parse_bea_observations(r#"{"records": [], "token": "x"}"#).is_err());
    }

    #[test]
    fn missing_required_field_is_rejected() {
        let input = r#"{"records": [
            {"dataset_id": "NIPA", "table_id": "T10101", "date": "2026-06-30",
             "value": 1.0, "unit": "Index", "frequency": "quarterly"}
        ]}"#;
        assert!(parse_bea_observations(input).is_err());
    }

    #[test]
    fn illegal_date_and_value_forms_are_rejected() {
        for bad_date in [
            "2026-2-3",
            "2026/06/30",
            "2026-02-30",
            "2026-06-30T00:00:00Z",
        ] {
            let input = format!(
                r#"{{"records": [{{"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1,
                    "date": "{bad_date}", "value": 1.0, "unit": "Index",
                    "frequency": "quarterly"}}]}}"#
            );
            assert!(parse_bea_observations(&input).is_err(), "{bad_date}");
        }
        let bad_value = r#"{"records": [
            {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
             "value": "n/a", "unit": "Index", "frequency": "quarterly"}
        ]}"#;
        assert!(parse_bea_observations(bad_value).is_err());
    }

    #[test]
    fn duplicate_identity_is_rejected() {
        let input = r#"{"records": [
            {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
             "value": 1.0, "unit": "Index", "frequency": "quarterly"},
            {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
             "value": 1.1, "unit": "Index", "frequency": "quarterly"}
        ]}"#;
        let err = parse_bea_observations(input).expect_err("应拒绝重复身份");
        assert_eq!(err.kind(), crate::BeaErrorKind::SemanticallyRejected);
    }

    #[test]
    fn out_of_scope_dataset_and_table_are_rejected() {
        let bad_dataset = r#"{"records": [
            {"dataset_id": "BLS", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
             "value": 1.0, "unit": "Index", "frequency": "quarterly"}
        ]}"#;
        assert!(parse_bea_observations(bad_dataset).is_err());

        let bad_table = r#"{"records": [
            {"dataset_id": "NIPA", "table_id": "T20600", "line_number": 1, "date": "2026-06-30",
             "value": 1.0, "unit": "Index", "frequency": "quarterly"}
        ]}"#;
        assert!(parse_bea_observations(bad_table).is_err());
    }

    #[test]
    fn table_frequency_drift_is_rejected() {
        let wrong = r#"{"records": [
            {"dataset_id": "NIPA", "table_id": "T20100", "line_number": 1, "date": "2026-06-30",
             "value": 1.0, "unit": "Index", "frequency": "quarterly"}
        ]}"#;
        assert!(parse_bea_observations(wrong).is_err());
    }

    #[test]
    fn malformed_json_reports_position_without_echoing_input() {
        let err = parse_bea_observations("{ not json").expect_err("应拒绝");
        let shown = err.to_string();
        assert!(!shown.contains("not json"));
        assert!(shown.contains("第"));
    }

    #[test]
    fn empty_records_yield_empty_collection() {
        assert!(parse_bea_observations(r#"{"records": []}"#)
            .expect("应可解析")
            .is_empty());
    }
}
