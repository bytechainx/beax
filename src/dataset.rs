//! beax 的源事实常量表：Dataset 全集、NIPA 表号白名单、优先级与频率。
//!
//! 本模块是 `specs/adapter/bea.md` 的**范围对齐落点**。它只登记清单明确列出的
//! Dataset 名与表号；清单未列出的表号 MUST NOT 在此新增。
//!
//! 边界：BEA 无官方 vintage 面（源级例外），live 官方 PIT **NO-GO**；
//! 核心 PCE 的采集主责在 `fredx`（fred-forward），本库只是补充口径。

use crate::error::{BeaError, BeaResult};
use crate::value::{BeaObservation, Frequency};

// ---------------------------------------------------------------------------
// 一、Dataset 全集（11 个）
// ---------------------------------------------------------------------------

/// NIPA（国民经济核算）：**P0**，承载清单三指标的 native Dataset。
pub const NIPA: &str = "NIPA";
/// NIUnderlyingDetail（NIPA 下钻明细）：P1。
pub const NI_UNDERLYING_DETAIL: &str = "NIUnderlyingDetail";
/// FixedAssets（固定资产）：P1。
pub const FIXED_ASSETS: &str = "FixedAssets";
/// ITA（国际交易）：P1。
pub const ITA: &str = "ITA";
/// IIP（国际投资头寸）：P1。
pub const IIP: &str = "IIP";
/// IntlServTrade（国际服务贸易）：P1。
pub const INTL_SERV_TRADE: &str = "IntlServTrade";
/// GDPbyIndustry（分行业 GDP）：P1。
pub const GDP_BY_INDUSTRY: &str = "GDPbyIndustry";
/// UnderlyingGDPbyIndustry（分行业 GDP 下钻）：P1。
pub const UNDERLYING_GDP_BY_INDUSTRY: &str = "UnderlyingGDPbyIndustry";
/// InputOutput（投入产出）：P1。
pub const INPUT_OUTPUT: &str = "InputOutput";
/// Regional（区域）：P1。
pub const REGIONAL: &str = "Regional";
/// MNE（跨国公司）：P1。
pub const MNE: &str = "MNE";

// ---------------------------------------------------------------------------
// 二、NIPA 表号白名单（P0 2 表 + P1 12 表 = 14 表）
// ---------------------------------------------------------------------------

/// `T10101`：GDP（支出法）P0 表。
pub const T10101: &str = "T10101";
/// `T20100`：个人收入与支出 P0 表；核心 PCE 的**补充口径**锚点表。
pub const T20100: &str = "T20100";
/// `T10102`：P1 表。
pub const T10102: &str = "T10102";
/// `T10103`：P1 表。
pub const T10103: &str = "T10103";
/// `T10104`：P1 表。
pub const T10104: &str = "T10104";
/// `T10105`：P1 表。
pub const T10105: &str = "T10105";
/// `T10106`：P1 表。
pub const T10106: &str = "T10106";
/// `T10107`：P1 表。
pub const T10107: &str = "T10107";
/// `T10201`：P1 表。
pub const T10201: &str = "T10201";
/// `T20301`：P1 表。
pub const T20301: &str = "T20301";
/// `T30100`：P1 表。
pub const T30100: &str = "T30100";
/// `T40100`：P1 表。
pub const T40100: &str = "T40100";
/// `T50100`：P1 表。
pub const T50100: &str = "T50100";
/// `T70100`：P1 表。
pub const T70100: &str = "T70100";

/// `T20300`：储蓄率相关表；**不在白名单**，与 `T20100` 语义不同。
pub const T20300: &str = "T20300";
/// `T20600`：储蓄率相关表；现窗口未入白名单，与 `T20100` 语义不同。
pub const T20600: &str = "T20600";
/// `T11000`：GDI 的**候选**表（待 live 核验入白名单）；与 GDP 语义不同。
pub const T11000: &str = "T11000";

// ---------------------------------------------------------------------------
// 三、集合与优先级
// ---------------------------------------------------------------------------

/// Dataset 全集（11 个）。
pub const DATASETS: &[&str] = &[
    NIPA,
    NI_UNDERLYING_DETAIL,
    FIXED_ASSETS,
    ITA,
    IIP,
    INTL_SERV_TRADE,
    GDP_BY_INDUSTRY,
    UNDERLYING_GDP_BY_INDUSTRY,
    INPUT_OUTPUT,
    REGIONAL,
    MNE,
];

/// NIPA 表级 **P0** 白名单（2 表）。
pub const CORE_NIPA_P0_TABLES: &[&str] = &[T10101, T20100];

/// NIPA 表级 **P1** 白名单（12 表）。
pub const CORE_NIPA_P1_TABLES: &[&str] = &[
    T10102, T10103, T10104, T10105, T10106, T10107, T10201, T20301, T30100, T40100, T50100, T70100,
];

/// 核心 PCE 的**补充口径**锚点表（主责采集在 `fredx`）。
pub const CORE_PCE_ANCHOR_TABLE: &str = T20100;

/// 该 Dataset 名是否在清单登记范围内。
#[must_use]
pub fn is_known_dataset(dataset_id: &str) -> bool {
    DATASETS.contains(&dataset_id)
}

/// 该表号是否为 NIPA 表级 P0 白名单成员。
#[must_use]
pub fn is_core_p0_table(table_id: &str) -> bool {
    CORE_NIPA_P0_TABLES.contains(&table_id)
}

/// 该表号是否为 NIPA 表级 P1 白名单成员。
#[must_use]
pub fn is_core_p1_table(table_id: &str) -> bool {
    CORE_NIPA_P1_TABLES.contains(&table_id)
}

/// 该表号是否为 NIPA 白名单表（P0 ∪ P1，共 14 表）。
#[must_use]
pub fn is_known_nipa_table(table_id: &str) -> bool {
    is_core_p0_table(table_id) || is_core_p1_table(table_id)
}

/// Dataset 优先级：`NIPA` = 0，其余 10 个 = 1；未登记返回 `None`。
#[must_use]
pub fn dataset_priority(dataset_id: &str) -> Option<u8> {
    match dataset_id {
        NIPA => Some(0),
        NI_UNDERLYING_DETAIL
        | FIXED_ASSETS
        | ITA
        | IIP
        | INTL_SERV_TRADE
        | GDP_BY_INDUSTRY
        | UNDERLYING_GDP_BY_INDUSTRY
        | INPUT_OUTPUT
        | REGIONAL
        | MNE => Some(1),
        _ => None,
    }
}

/// 表级优先级：P0 表 = 0，P1 表 = 1；未登记返回 `None`。
#[must_use]
pub fn table_priority(table_id: &str) -> Option<u8> {
    if is_core_p0_table(table_id) {
        Some(0)
    } else if is_core_p1_table(table_id) {
        Some(1)
    } else {
        None
    }
}

/// 清单声明的表级频率；未声明的表返回 `None`。
///
/// 清单只对 `T10101`（GDP，季频）与 `T20100`（个人收入与支出，月频）声明了频率，
/// 其余表 MUST NOT 由实现方推定。
#[must_use]
pub fn table_frequency(table_id: &str) -> Option<Frequency> {
    match table_id {
        T10101 => Some(Frequency::Quarterly),
        T20100 => Some(Frequency::Monthly),
        _ => None,
    }
}

/// 拒绝清单登记范围外的 Dataset（原子失败，MUST NOT 静默接受）。
///
/// # Errors
///
/// 该 Dataset 未登记时返回 [`BeaError::SemanticallyRejected`]。
pub fn ensure_known_dataset(dataset_id: &str) -> BeaResult<()> {
    if is_known_dataset(dataset_id) {
        return Ok(());
    }
    Err(BeaError::SemanticallyRejected(format!(
        "dataset_id 不在清单登记范围内：{dataset_id}"
    )))
}

/// 拒绝 NIPA 白名单外的表号（仅当 Dataset 为 `NIPA` 时适用）。
///
/// # Errors
///
/// 表号不在 14 表白名单内时返回 [`BeaError::SemanticallyRejected`]。
pub fn ensure_nipa_table(table_id: &str) -> BeaResult<()> {
    if is_known_nipa_table(table_id) {
        return Ok(());
    }
    Err(BeaError::SemanticallyRejected(format!(
        "table_id 不在 NIPA 白名单内：{table_id}"
    )))
}

/// 表级频率一致性守卫：清单声明过频率的表，其观测频率 MUST 等于清单值。
///
/// # Errors
///
/// 频率与清单声明不符时返回 [`BeaError::SemanticallyRejected`]。
pub fn ensure_table_frequency(observation: &BeaObservation) -> BeaResult<()> {
    if let Some(expected) = table_frequency(&observation.table_id) {
        if observation.frequency != expected {
            return Err(BeaError::SemanticallyRejected(format!(
                "{} 的清单频率为 {}，输入为 {}",
                observation.table_id,
                expected.as_str(),
                observation.frequency.as_str()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{BeaUnit, BeaValue, Date, Period};

    #[test]
    fn dataset_set_is_eleven_and_nipa_is_the_only_p0() {
        assert_eq!(DATASETS.len(), 11);
        for dataset in DATASETS {
            assert!(is_known_dataset(dataset));
            assert!(dataset_priority(dataset).is_some());
        }
        let mut sorted = DATASETS.to_vec();
        sorted.sort_unstable();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(before, sorted.len(), "Dataset 出现重复");
        assert_eq!(dataset_priority(NIPA), Some(0));
        assert_eq!(dataset_priority(ITA), Some(1));
        assert_eq!(dataset_priority("NotADataset"), None);
    }

    #[test]
    fn nipa_tables_are_two_plus_twelve() {
        assert_eq!(CORE_NIPA_P0_TABLES.len(), 2);
        assert_eq!(CORE_NIPA_P1_TABLES.len(), 12);
        assert_eq!(CORE_NIPA_P0_TABLES.len() + CORE_NIPA_P1_TABLES.len(), 14);
        for table in CORE_NIPA_P0_TABLES {
            assert!(is_core_p0_table(table));
            assert!(is_known_nipa_table(table));
            assert_eq!(table_priority(table), Some(0));
        }
        for table in CORE_NIPA_P1_TABLES {
            assert!(is_core_p1_table(table));
            assert!(is_known_nipa_table(table));
            assert_eq!(table_priority(table), Some(1));
        }
        for outside in [T20300, T20600, T11000, "T99999"] {
            assert!(!is_known_nipa_table(outside), "{outside}");
            assert_eq!(table_priority(outside), None);
        }
    }

    #[test]
    fn only_two_tables_have_declared_frequency() {
        assert_eq!(table_frequency(T10101), Some(Frequency::Quarterly));
        assert_eq!(table_frequency(T20100), Some(Frequency::Monthly));
        assert_eq!(table_frequency(T10102), None);
    }

    #[test]
    fn core_pce_anchor_is_t20100() {
        assert_eq!(CORE_PCE_ANCHOR_TABLE, T20100);
        assert!(is_core_p0_table(CORE_PCE_ANCHOR_TABLE));
    }

    #[test]
    fn guards_reject_out_of_scope_inputs() {
        assert!(ensure_known_dataset(NIPA).is_ok());
        assert!(ensure_known_dataset("BLS").is_err());
        assert!(ensure_nipa_table(T10101).is_ok());
        assert!(ensure_nipa_table(T20600).is_err());
    }

    #[test]
    fn frequency_guard_rejects_manifest_drift() {
        let mut observation = BeaObservation {
            dataset_id: NIPA.to_owned(),
            table_id: T20100.to_owned(),
            line_number: 1,
            period: Period::Month {
                year: 2026,
                month: 6,
            },
            value: BeaValue::Present(1.0),
            unit: BeaUnit::new("Millions of dollars").expect("单位合法"),
            frequency: Frequency::Quarterly,
            vintage: None,
        };
        assert!(ensure_table_frequency(&observation).is_err());
        observation.frequency = Frequency::Monthly;
        assert!(ensure_table_frequency(&observation).is_ok());

        let undeclared = BeaObservation {
            dataset_id: NIPA.to_owned(),
            table_id: T10102.to_owned(),
            line_number: 1,
            period: Period::Year(2026),
            value: BeaValue::Present(1.0),
            unit: BeaUnit::new("Index").expect("单位合法"),
            frequency: Frequency::Annual,
            vintage: Some(Date::new(2026, 3, 31).expect("日期合法")),
        };
        assert!(ensure_table_frequency(&undeclared).is_ok());
    }
}
