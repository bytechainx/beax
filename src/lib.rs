#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable
    )
)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

//! # beax —— BEA 源事实库（类型化源事实 + 离线解析 + 守卫 + fail-closed 授权判定）
//!
//! 本库把 `specs/adapter/bea.md` 声明的**源事实**落成代码：11 个 Dataset 常量、
//! NIPA 表号白名单与优先级、曲线产品路由、核心 PCE 主责边界、近义非同 ID 禁则、
//! publication 语义三元组，以及一个只吃字符串的离线解析器。
//!
//! ## 能力
//!
//! | 能力 | 状态 |
//! | --- | --- |
//! | Dataset 常量与分组（11 个）与 Dataset 优先级 | 已落（离线） |
//! | NIPA 表号白名单（P0 2 表 + P1 12 表 = 14 表）与表级优先级 | 已落 |
//! | 表级频率一致性守卫（`T10101` 季频 / `T20100` 月频） | 已落 |
//! | 曲线产品路由拒绝（归 `yieldx`） | 已落 |
//! | 核心 PCE 主责边界（`fred-forward`，MUST NOT 反向主张） | 已落（只读判定） |
//! | 近义非同 ID 守卫（涉 BEA 的 6 对） | 已落 |
//! | publication 语义（`Date` + `Inferred` + 正式 PIT `NotEligible`） | 已落 |
//! | 授权判定（fail-closed，仅覆盖 offline / reference，**不含 live**） | 已落 |
//! | 离线 JSON 解析（无网络参数） | 已落 |
//! | 联网采集 / live / 官方 PIT | **未实现**（`live_official_pit = NO-GO`） |
//!
//! ## 责任边界
//!
//! 本库**做**：类型化源事实、离线解析与校验、跨源守卫的「自己那一侧」、只读授权判定。
//! 本库**不做**：联网采集、凭据处理、端点编址、单位换算、派生指标、存储或分发。
//!
//! ## 非目标
//!
//! - 不是联网采集器：无 HTTP 客户端依赖、无端点字面量、不读环境变量或凭据
//! - 不实现派生指标（增长率、贡献度、平减指数归 analytics）
//! - 不做单位换算（归下游 Normalize）；曲线产品须路由 `yieldx`
//! - 不主张核心 PCE 的主责身份（归 `fredx`，fred-forward）
//! - 不新建共享 core crate：本库的公共形状是与兄弟库**各自实现一遍**的同义形状
//!
//! ## 诚实边界
//!
//! `production_decision = NO-GO`；清单 COMPLETE ≠ ship；authorization ≠ Production Ready。
//! 证据的 `scope = offline_and_reference_only`：**live 不被覆盖**，`live_official_pit = NO-GO`。
//!
//! # 最小示例
//!
//! ```
//! use beax::{
//!     authorize_bea, documented_bea_evidence, is_formal_pit_eligible, parse_bea_observations,
//!     BeaAccessMode, BeaAuthorization, BeaClaim, ensure_claim_local,
//! };
//!
//! let input = r#"{"_synthetic": true, "records": [
//!     {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
//!      "value": 30000.0, "unit": "Billions of dollars", "frequency": "quarterly"}
//! ]}"#;
//! let observations = parse_bea_observations(input)?;
//! assert_eq!(observations[0].table_id, "T10101");
//!
//! // 核心 PCE 主责在 fredx；本域只承载补充观测。
//! assert!(ensure_claim_local(BeaClaim::SupplementaryTableValue).is_ok());
//! assert!(ensure_claim_local(BeaClaim::CorePcePrimary).is_err());
//! assert!(ensure_claim_local(BeaClaim::YieldCurveConstruction).is_err());
//!
//! // offline / reference 被覆盖，live 不被覆盖。
//! let as_of = beax::Date::new(2026, 8, 20)?;
//! let evidence = documented_bea_evidence();
//! assert!(matches!(
//!     authorize_bea(Some(&evidence), BeaAccessMode::ReferenceOnly, as_of),
//!     BeaAuthorization::Authorized { .. }
//! ));
//! assert!(matches!(
//!     authorize_bea(Some(&evidence), BeaAccessMode::Live, as_of),
//!     BeaAuthorization::Denied { .. }
//! ));
//! assert!(!is_formal_pit_eligible());
//! # Ok::<(), beax::BeaError>(())
//! ```

pub mod authz;
pub mod dataset;
pub mod error;
pub mod parse;
pub mod pit;
pub mod routing;
pub mod value;

pub use authz::{
    accepts_no_official_pit, authorize_bea, documented_bea_evidence, live_official_pit_decision,
    mode_label, validate_authorization_evidence, BeaAccessMode, BeaAuthorization,
    BeaAuthorizationEvidence, BEA_DECISION_ID, BEA_SIGNED_BY, LIVE_OFFICIAL_PIT_DECISION,
};
pub use dataset::{
    dataset_priority, ensure_known_dataset, ensure_nipa_table, ensure_table_frequency,
    is_core_p0_table, is_core_p1_table, is_known_dataset, is_known_nipa_table, table_frequency,
    table_priority, CORE_NIPA_P0_TABLES, CORE_NIPA_P1_TABLES, CORE_PCE_ANCHOR_TABLE, DATASETS,
    NIPA, T10101, T10102, T10103, T10104, T10105, T10106, T10107, T10201, T11000, T20100, T20300,
    T20301, T20600, T30100, T40100, T50100, T70100,
};
// Dataset 常量：源事实的一等公民，直接暴露在门面上（也可经 `beax::dataset` 访问）。
pub use dataset::{
    FIXED_ASSETS, GDP_BY_INDUSTRY, IIP, INPUT_OUTPUT, INTL_SERV_TRADE, ITA, MNE,
    NI_UNDERLYING_DETAIL, REGIONAL, UNDERLYING_GDP_BY_INDUSTRY,
};
pub use error::{BeaError, BeaErrorKind, BeaResult};
pub use parse::parse_bea_observations;
pub use pit::{
    bea_publication_semantics, is_formal_pit_eligible, AvailabilityEvidence,
    BeaPublicationSemantics, PitEligibility, TimePrecision,
};
pub use routing::{
    ensure_claim_local, ensure_not_silent_substitution, gdi_candidate_table,
    is_gdp_or_personal_income_table, is_near_synonym_pair, BeaClaim, CORE_PCE_PRIMARY_OWNER,
    CURVE_BUILD_OWNER, NEAR_SYNONYM_PAIRS,
};
pub use value::{
    validate_date, validate_observation, validate_period, BeaMissingReason, BeaObservation,
    BeaUnit, BeaValue, Date, Frequency, Period,
};
