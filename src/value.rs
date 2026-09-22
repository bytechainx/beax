//! beax 的源事实值对象：日期、期间、频率、源侧单位、观测值形态。
//!
//! 本模块只表达源事实，**不做**任何派生计算（增长率、贡献度、平减指数一律归 analytics），
//! 也**不做**单位换算（归下游 Normalize）。
//!
//! 观测身份 = `dataset_id` + `table_id` + `line_number` + `period` + `vintage`。
//! 表号 MUST NOT 被直接当成指标 ID（`source_series_id` 保留原始表号 / 行号）。

use crate::error::{BeaError, BeaResult};

/// 严格 ISO 日期（`YYYY-MM-DD`）。
///
/// 字段公开以便构造与模式匹配，但**直接构造不校验**；构造入口用 [`Date::new`] /
/// [`Date::parse`]，完整性校验用 [`validate_date`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date {
    /// 年（本库只接受 1..=9999）。
    pub year: i16,
    /// 月（1..=12）。
    pub month: u8,
    /// 日（按月份与闰年校验）。
    pub day: u8,
}

impl Date {
    /// 构造并校验一个日期。
    ///
    /// # Errors
    ///
    /// 年份越界、月份不在 1..=12、或日超出该月实际天数时返回 [`BeaError::Invalid`]。
    pub fn new(year: i16, month: u8, day: u8) -> BeaResult<Self> {
        let date = Self { year, month, day };
        validate_date(&date)?;
        Ok(date)
    }

    /// 按严格 ISO 形态 `YYYY-MM-DD` 解析。
    ///
    /// 只接受四位年 + 两位月 + 两位日、以 `-` 分隔且长度恰为 10 的形态。
    /// 拒绝 `2026-2-3`（未补零）、`2026/02/03`（分隔符不符）与带时间部分者。
    ///
    /// # Errors
    ///
    /// 形态不符或取值非法时返回 [`BeaError::Invalid`]。
    pub fn parse(text: &str) -> BeaResult<Self> {
        let bytes = text.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return Err(BeaError::Invalid(
                "日期须为严格的 YYYY-MM-DD（月/日两位补零）".into(),
            ));
        }
        let year = parse_digits(&bytes[0..4])
            .ok_or_else(|| BeaError::Invalid("年份须为四位数字".into()))?;
        let month = parse_digits(&bytes[5..7])
            .ok_or_else(|| BeaError::Invalid("月份须为两位数字".into()))?;
        let day = parse_digits(&bytes[8..10])
            .ok_or_else(|| BeaError::Invalid("日期须为两位数字".into()))?;
        let year = i16::try_from(year).map_err(|_| BeaError::Invalid("年份越界".into()))?;
        let month = u8::try_from(month).map_err(|_| BeaError::Invalid("月份越界".into()))?;
        let day = u8::try_from(day).map_err(|_| BeaError::Invalid("日期越界".into()))?;
        Self::new(year, month, day)
    }

    /// 该年是否为闰年。
    #[must_use]
    pub const fn is_leap_year(year: i16) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    /// 该年该月的天数；月份非法时返回 `0`（不 panic）。
    #[must_use]
    pub const fn days_in_month(year: i16, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }
}

/// 解析定长十进制串；出现非数字字符时返回 `None`。
fn parse_digits(bytes: &[u8]) -> Option<u32> {
    let mut acc: u32 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        acc = acc * 10 + u32::from(b - b'0');
    }
    Some(acc)
}

/// 校验日期：年份 `1..=9999`、月 `1..=12`、日不超过该月实际天数。
///
/// # Errors
///
/// 任一项越界时返回 [`BeaError::Invalid`]。
pub fn validate_date(date: &Date) -> BeaResult<()> {
    if date.year < 1 || date.year > 9999 {
        return Err(BeaError::Invalid("年份须在 1..=9999".into()));
    }
    if date.month < 1 || date.month > 12 {
        return Err(BeaError::Invalid("月份须在 1..=12".into()));
    }
    let limit = Date::days_in_month(date.year, date.month);
    if date.day < 1 || date.day > limit {
        return Err(BeaError::Invalid(format!(
            "日期须在 1..={limit}（{}-{:02}）",
            date.year, date.month
        )));
    }
    Ok(())
}

/// 业务期间。MUST NOT 用裸字符串顶替。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Period {
    /// 日频观测。
    Day(Date),
    /// 月频观测。
    Month {
        /// 年。
        year: i16,
        /// 月（1..=12）。
        month: u8,
    },
    /// 季频观测。
    Quarter {
        /// 年。
        year: i16,
        /// 季（1..=4）。
        quarter: u8,
    },
    /// 年频观测。
    Year(i16),
    /// 事件型观测（以日期定位）。
    Event {
        /// 事件日期。
        date: Date,
    },
}

/// 校验期间：日期分量合法、月 `1..=12`、季 `1..=4`。
///
/// # Errors
///
/// 任一分量越界时返回 [`BeaError::Invalid`]。
pub fn validate_period(period: &Period) -> BeaResult<()> {
    match *period {
        Period::Day(date) | Period::Event { date } => validate_date(&date),
        Period::Month { year, month } => {
            if !(1..=9999).contains(&year) {
                return Err(BeaError::Invalid("年份须在 1..=9999".into()));
            }
            if !(1..=12).contains(&month) {
                return Err(BeaError::Invalid("月份须在 1..=12".into()));
            }
            Ok(())
        }
        Period::Quarter { year, quarter } => {
            if !(1..=9999).contains(&year) {
                return Err(BeaError::Invalid("年份须在 1..=9999".into()));
            }
            if !(1..=4).contains(&quarter) {
                return Err(BeaError::Invalid("季度须在 1..=4".into()));
            }
            Ok(())
        }
        Period::Year(year) => {
            if !(1..=9999).contains(&year) {
                return Err(BeaError::Invalid("年份须在 1..=9999".into()));
            }
            Ok(())
        }
    }
}

/// 源侧频率。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Frequency {
    /// 日频。
    Daily,
    /// 周频。
    Weekly,
    /// 月频。
    Monthly,
    /// 季频。
    Quarterly,
    /// 年频。
    Annual,
    /// 事件型（不定期、由事件触发）。
    Event,
    /// 不规则。
    Irregular,
}

impl Frequency {
    /// 本库离线输入使用的稳定记号。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Quarterly => "quarterly",
            Self::Annual => "annual",
            Self::Event => "event",
            Self::Irregular => "irregular",
        }
    }

    /// 解析本库离线输入使用的频率记号（**只接受小写**）。
    ///
    /// # Errors
    ///
    /// 记号不在白名单内时返回 [`BeaError::Invalid`]。
    pub fn parse(text: &str) -> BeaResult<Self> {
        match text {
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            "quarterly" => Ok(Self::Quarterly),
            "annual" => Ok(Self::Annual),
            "event" => Ok(Self::Event),
            "irregular" => Ok(Self::Irregular),
            _ => Err(BeaError::Invalid(format!("频率记号不在白名单内（{text}）"))),
        }
    }
}

/// 源侧单位。
///
/// **保留源单位**；单位换算 MUST NOT 在源层发生（归下游 Normalize）。
/// 采用开放 newtype 而非封闭枚举：清单未逐表声明源单位，
/// 封闭枚举会迫使实现方**编造**单位取值。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BeaUnit(String);

impl BeaUnit {
    /// 构造源侧单位：非空、无首尾空白、不含控制字符。
    ///
    /// # Errors
    ///
    /// 上述任一条件不满足时返回 [`BeaError::Invalid`]。
    pub fn new(text: &str) -> BeaResult<Self> {
        if text.is_empty() || text.trim() != text {
            return Err(BeaError::Invalid("源单位须为非空且无首尾空白".into()));
        }
        if text.chars().any(char::is_control) {
            return Err(BeaError::Invalid("源单位不得含控制字符".into()));
        }
        Ok(Self(text.to_owned()))
    }

    /// 源单位字面量。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 缺失原因。MUST NOT 把缺失静默折算为 `0`。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BeaMissingReason {
    /// 源未给出该期间的值（含被抑制、未发布两类「无观测」）。
    NoObservation,
}

/// 观测值：要么是数值，要么是**具名**缺失原因。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BeaValue {
    /// 已发布的数值。
    Present(f64),
    /// 缺失，附具名原因。
    Missing(BeaMissingReason),
}

impl BeaValue {
    /// 数值（缺失时为 `None`）。
    #[must_use]
    pub fn as_f64(self) -> Option<f64> {
        match self {
            Self::Present(v) => Some(v),
            Self::Missing(_) => None,
        }
    }

    /// 是否缺失。
    #[must_use]
    pub fn is_missing(self) -> bool {
        matches!(self, Self::Missing(_))
    }
}

/// 一条 BEA 源事实观测。
#[derive(Debug, Clone, PartialEq)]
pub struct BeaObservation {
    /// Dataset 标识（如 `NIPA`）。
    pub dataset_id: String,
    /// 表号（**保留原始表号**，如 `T10101`）；MUST NOT 被直接当作指标 ID。
    pub table_id: String,
    /// 行号（≥ 1）。
    pub line_number: u32,
    /// 业务期间。
    pub period: Period,
    /// 观测值。
    pub value: BeaValue,
    /// 源侧单位（保留原样）。
    pub unit: BeaUnit,
    /// 源侧频率。
    pub frequency: Frequency,
    /// 修订标识；本层无官方 vintage 面时为 `None`，MUST NOT 伪造。
    pub vintage: Option<Date>,
}

/// 校验一条观测：Dataset / 表号非空、行号 ≥ 1、期间合法、数值有限、修订日期合法。
///
/// 表号与清单频率的**跨表一致性**由 [`crate::dataset`] 的守卫负责
/// （见 [`crate::dataset::ensure_table_frequency`]）。
///
/// # Errors
///
/// 任一条件不满足时返回 [`BeaError::Invalid`]。
pub fn validate_observation(observation: &BeaObservation) -> BeaResult<()> {
    if observation.dataset_id.is_empty() || observation.dataset_id.trim() != observation.dataset_id
    {
        return Err(BeaError::Invalid("dataset_id 须为非空且无首尾空白".into()));
    }
    if observation.table_id.is_empty() || observation.table_id.trim() != observation.table_id {
        return Err(BeaError::Invalid("table_id 须为非空且无首尾空白".into()));
    }
    if observation.line_number < 1 {
        return Err(BeaError::Invalid("line_number 须 ≥ 1".into()));
    }
    validate_period(&observation.period)?;
    if let BeaValue::Present(v) = observation.value {
        if !v.is_finite() {
            return Err(BeaError::Invalid(
                "观测值须为有限数（NaN / 无穷不得静默通过）".into(),
            ));
        }
    }
    if let Some(vintage) = observation.vintage {
        validate_date(&vintage)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit() -> BeaUnit {
        BeaUnit::new("Millions of dollars").expect("单位合法")
    }

    fn observation() -> BeaObservation {
        BeaObservation {
            dataset_id: "NIPA".to_owned(),
            table_id: "T10101".to_owned(),
            line_number: 1,
            period: Period::Quarter {
                year: 2026,
                quarter: 2,
            },
            value: BeaValue::Present(1.5),
            unit: unit(),
            frequency: Frequency::Quarterly,
            vintage: None,
        }
    }

    #[test]
    fn date_parse_accepts_strict_iso() {
        assert_eq!(
            Date::parse("2026-08-15").expect("应可解析"),
            Date {
                year: 2026,
                month: 8,
                day: 15
            }
        );
    }

    #[test]
    fn date_parse_rejects_non_strict_forms() {
        for bad in [
            "2026-2-3",
            "2026/02/03",
            "2026-02-03T00:00:00Z",
            "2026-02-30",
            "2026-13-01",
            "0000-01-01",
            "",
        ] {
            assert!(Date::parse(bad).is_err(), "应拒绝：{bad}");
        }
    }

    #[test]
    fn leap_year_and_days_in_month() {
        assert!(Date::is_leap_year(2024));
        assert!(!Date::is_leap_year(1900));
        assert!(Date::is_leap_year(2000));
        assert_eq!(Date::days_in_month(2024, 2), 29);
        assert_eq!(Date::days_in_month(2026, 2), 28);
        assert_eq!(Date::days_in_month(2026, 13), 0);
    }

    #[test]
    fn period_validation_covers_every_variant() {
        let day = Date::new(2026, 8, 15).expect("日期合法");
        assert!(validate_period(&Period::Day(day)).is_ok());
        assert!(validate_period(&Period::Event { date: day }).is_ok());
        assert!(validate_period(&Period::Month {
            year: 2026,
            month: 12
        })
        .is_ok());
        assert!(validate_period(&Period::Month {
            year: 2026,
            month: 13
        })
        .is_err());
        assert!(validate_period(&Period::Quarter {
            year: 2026,
            quarter: 4
        })
        .is_ok());
        assert!(validate_period(&Period::Quarter {
            year: 2026,
            quarter: 5
        })
        .is_err());
        assert!(validate_period(&Period::Year(2026)).is_ok());
        assert!(validate_period(&Period::Year(0)).is_err());
    }

    #[test]
    fn frequency_tokens_round_trip_and_reject_unknown() {
        for freq in [
            Frequency::Daily,
            Frequency::Weekly,
            Frequency::Monthly,
            Frequency::Quarterly,
            Frequency::Annual,
            Frequency::Event,
            Frequency::Irregular,
        ] {
            assert_eq!(Frequency::parse(freq.as_str()).expect("回环"), freq);
        }
        assert!(Frequency::parse("Monthly").is_err());
    }

    #[test]
    fn unit_rejects_blank_and_control_chars() {
        assert_eq!(unit().as_str(), "Millions of dollars");
        assert!(BeaUnit::new("").is_err());
        assert!(BeaUnit::new(" dollars").is_err());
        assert!(BeaUnit::new("dollars\n").is_err());
    }

    #[test]
    fn missing_value_is_named_and_never_zero() {
        let missing = BeaValue::Missing(BeaMissingReason::NoObservation);
        assert!(missing.is_missing());
        assert_eq!(missing.as_f64(), None);
        assert_ne!(missing, BeaValue::Present(0.0));
    }

    #[test]
    fn validate_observation_accepts_well_formed_record() {
        assert!(validate_observation(&observation()).is_ok());
    }

    #[test]
    fn validate_observation_rejects_bad_identity_and_values() {
        let mut bad = observation();
        bad.dataset_id = " NIPA".to_owned();
        assert!(validate_observation(&bad).is_err());

        bad = observation();
        bad.table_id = String::new();
        assert!(validate_observation(&bad).is_err());

        bad = observation();
        bad.line_number = 0;
        assert!(validate_observation(&bad).is_err());

        bad = observation();
        bad.value = BeaValue::Present(f64::NAN);
        assert!(validate_observation(&bad).is_err());

        bad = observation();
        bad.period = Period::Quarter {
            year: 2026,
            quarter: 5,
        };
        assert!(validate_observation(&bad).is_err());

        bad = observation();
        bad.vintage = Some(Date {
            year: 2026,
            month: 2,
            day: 30,
        });
        assert!(validate_observation(&bad).is_err());
    }
}
