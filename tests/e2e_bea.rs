#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! E2E（beax）：在**离线合成夹具**上端到端执行**全部**公开接口。
//!
//! beax 是**离线**源事实库：无外部服务、无凭据、不读环境变量，故本测试**不标注为忽略**，
//! 默认参与 CI。对齐对象是 `cargo +nightly public-api --simplified` 导出的完整公开面
//! （`fn` / `type` / `field` / `const` / `variant` 五类），逐条登记在 [`E2E_MANIFEST`]，
//! 运行期由 `cover` 登记表核对「声明 = 实际执行」（缺一即失败）。
//!
//! **独立核对**：`scripts/verify-e2e-coverage.mjs` 会重新派生公开面与清单双向 diff，
//! 并用 `-C instrument-coverage` + `llvm-cov report --show-functions` 断言每条公开
//! `fn` 执行次数 > 0；本文件内的登记表只是**声明**，不是唯一证据。
//!
//! 解析阶段的输入是仓内**真实夹具**（`include_str!` 读 `tests/fixtures/*.json`），
//! 而非内联字符串——这是「真实文件」的 E2E 证据。
//!
//! ```text
//! cd .worktrees/beax/beax
//! CARGO_TARGET_DIR=/home/workspace/bytechainx/.cargo/wt/beax cargo test --test e2e_bea
//! ```

use std::collections::BTreeSet;

use beax::{
    accepts_no_official_pit, authorize_bea, bea_publication_semantics, dataset_priority,
    documented_bea_evidence, ensure_claim_local, ensure_known_dataset, ensure_nipa_table,
    ensure_not_silent_substitution, ensure_table_frequency, gdi_candidate_table, is_core_p0_table,
    is_core_p1_table, is_formal_pit_eligible, is_gdp_or_personal_income_table, is_known_dataset,
    is_known_nipa_table, is_near_synonym_pair, live_official_pit_decision, mode_label,
    parse_bea_observations, table_frequency, table_priority, validate_authorization_evidence,
    validate_date, validate_observation, validate_period, AvailabilityEvidence, BeaAccessMode,
    BeaAuthorization, BeaAuthorizationEvidence, BeaClaim, BeaError, BeaErrorKind, BeaMissingReason,
    BeaObservation, BeaPublicationSemantics, BeaResult, BeaUnit, BeaValue, Date, Frequency, Period,
    PitEligibility, TimePrecision, BEA_DECISION_ID, BEA_SIGNED_BY, CORE_NIPA_P0_TABLES,
    CORE_NIPA_P1_TABLES, CORE_PCE_ANCHOR_TABLE, CORE_PCE_PRIMARY_OWNER, CURVE_BUILD_OWNER,
    DATASETS, FIXED_ASSETS, GDP_BY_INDUSTRY, IIP, INPUT_OUTPUT, INTL_SERV_TRADE, ITA,
    LIVE_OFFICIAL_PIT_DECISION, MNE, NEAR_SYNONYM_PAIRS, NIPA, NI_UNDERLYING_DETAIL, REGIONAL,
    T10101, T10102, T10103, T10104, T10105, T10106, T10107, T10201, T11000, T20100, T20300, T20301,
    T20600, T30100, T40100, T50100, T70100, UNDERLYING_GDP_BY_INDUSTRY,
};

/// 公开面清单：`(条目类别, 入口 id)`，由 `cargo +nightly public-api --simplified` 派生并冻结。
///
/// 类别取值域：`fn` / `type` / `field` / `const` / `variant`。
/// 该清单是运行时登记的**唯一事实源**——`cover::hit` 拒绝清单外的 id，收尾断言拒绝
/// 「声明了却没执行」的条目。清单本身的时效性由外部核对器与公开面 diff 保证。
const E2E_MANIFEST: &[(&str, &str)] = &[
    ("type", "BeaAccessMode"),
    ("variant", "BeaAccessMode::Live"),
    ("variant", "BeaAccessMode::Offline"),
    ("variant", "BeaAccessMode::ReferenceOnly"),
    ("type", "BeaAuthorization"),
    ("variant", "BeaAuthorization::Authorized"),
    ("variant", "BeaAuthorization::Denied"),
    ("type", "BeaAuthorizationEvidence"),
    ("field", "BeaAuthorizationEvidence::accept_no_pit"),
    ("field", "BeaAuthorizationEvidence::authorized_modes"),
    ("field", "BeaAuthorizationEvidence::decision_id"),
    ("field", "BeaAuthorizationEvidence::live_official_pit"),
    ("field", "BeaAuthorizationEvidence::scope_note"),
    ("field", "BeaAuthorizationEvidence::signed_at"),
    ("field", "BeaAuthorizationEvidence::signed_by"),
    ("field", "BeaAuthorizationEvidence::valid_until"),
    ("const", "BEA_DECISION_ID"),
    ("const", "BEA_SIGNED_BY"),
    ("const", "LIVE_OFFICIAL_PIT_DECISION"),
    ("fn", "accepts_no_official_pit"),
    ("fn", "authorize_bea"),
    ("fn", "documented_bea_evidence"),
    ("fn", "live_official_pit_decision"),
    ("fn", "mode_label"),
    ("fn", "validate_authorization_evidence"),
    ("const", "CORE_NIPA_P0_TABLES"),
    ("const", "CORE_NIPA_P1_TABLES"),
    ("const", "CORE_PCE_ANCHOR_TABLE"),
    ("const", "DATASETS"),
    ("const", "FIXED_ASSETS"),
    ("const", "GDP_BY_INDUSTRY"),
    ("const", "IIP"),
    ("const", "INPUT_OUTPUT"),
    ("const", "INTL_SERV_TRADE"),
    ("const", "ITA"),
    ("const", "MNE"),
    ("const", "NIPA"),
    ("const", "NI_UNDERLYING_DETAIL"),
    ("const", "REGIONAL"),
    ("const", "T10101"),
    ("const", "T10102"),
    ("const", "T10103"),
    ("const", "T10104"),
    ("const", "T10105"),
    ("const", "T10106"),
    ("const", "T10107"),
    ("const", "T10201"),
    ("const", "T11000"),
    ("const", "T20100"),
    ("const", "T20300"),
    ("const", "T20301"),
    ("const", "T20600"),
    ("const", "T30100"),
    ("const", "T40100"),
    ("const", "T50100"),
    ("const", "T70100"),
    ("const", "UNDERLYING_GDP_BY_INDUSTRY"),
    ("fn", "dataset_priority"),
    ("fn", "ensure_known_dataset"),
    ("fn", "ensure_nipa_table"),
    ("fn", "ensure_table_frequency"),
    ("fn", "is_core_p0_table"),
    ("fn", "is_core_p1_table"),
    ("fn", "is_known_dataset"),
    ("fn", "is_known_nipa_table"),
    ("fn", "table_frequency"),
    ("fn", "table_priority"),
    ("type", "BeaError"),
    ("variant", "BeaError::AuthorizationDenied"),
    ("variant", "BeaError::Invalid"),
    ("variant", "BeaError::Invariant"),
    ("variant", "BeaError::Missing"),
    ("variant", "BeaError::NotApplicable"),
    ("variant", "BeaError::RoutedElsewhere"),
    ("variant", "BeaError::SemanticallyRejected"),
    ("variant", "BeaError::WriteAuthorityDenied"),
    ("fn", "BeaError::is_retryable"),
    ("fn", "BeaError::kind"),
    ("type", "BeaErrorKind"),
    ("variant", "BeaErrorKind::AuthorizationDenied"),
    ("variant", "BeaErrorKind::Invalid"),
    ("variant", "BeaErrorKind::Invariant"),
    ("variant", "BeaErrorKind::Missing"),
    ("variant", "BeaErrorKind::NotApplicable"),
    ("variant", "BeaErrorKind::RoutedElsewhere"),
    ("variant", "BeaErrorKind::SemanticallyRejected"),
    ("variant", "BeaErrorKind::WriteAuthorityDenied"),
    ("type", "BeaResult"),
    ("fn", "parse_bea_observations"),
    ("type", "AvailabilityEvidence"),
    ("variant", "AvailabilityEvidence::Calendar"),
    ("variant", "AvailabilityEvidence::Inferred"),
    ("variant", "AvailabilityEvidence::Official"),
    ("type", "PitEligibility"),
    ("variant", "PitEligibility::Formal"),
    ("variant", "PitEligibility::NotEligible"),
    ("type", "TimePrecision"),
    ("variant", "TimePrecision::Date"),
    ("variant", "TimePrecision::Instant"),
    ("type", "BeaPublicationSemantics"),
    ("field", "BeaPublicationSemantics::availability"),
    ("field", "BeaPublicationSemantics::eligibility"),
    ("field", "BeaPublicationSemantics::time_precision"),
    ("fn", "bea_publication_semantics"),
    ("fn", "is_formal_pit_eligible"),
    ("type", "BeaClaim"),
    ("variant", "BeaClaim::CorePcePrimary"),
    ("variant", "BeaClaim::SupplementaryTableValue"),
    ("variant", "BeaClaim::YieldCurveConstruction"),
    ("const", "CORE_PCE_PRIMARY_OWNER"),
    ("const", "CURVE_BUILD_OWNER"),
    ("const", "NEAR_SYNONYM_PAIRS"),
    ("fn", "ensure_claim_local"),
    ("fn", "ensure_not_silent_substitution"),
    ("fn", "gdi_candidate_table"),
    ("fn", "is_gdp_or_personal_income_table"),
    ("fn", "is_near_synonym_pair"),
    ("type", "BeaMissingReason"),
    ("variant", "BeaMissingReason::NoObservation"),
    ("type", "BeaValue"),
    ("variant", "BeaValue::Missing"),
    ("variant", "BeaValue::Present"),
    ("fn", "BeaValue::as_f64"),
    ("fn", "BeaValue::is_missing"),
    ("type", "Frequency"),
    ("variant", "Frequency::Annual"),
    ("variant", "Frequency::Daily"),
    ("variant", "Frequency::Event"),
    ("variant", "Frequency::Irregular"),
    ("variant", "Frequency::Monthly"),
    ("variant", "Frequency::Quarterly"),
    ("variant", "Frequency::Weekly"),
    ("fn", "Frequency::as_str"),
    ("fn", "Frequency::parse"),
    ("type", "Period"),
    ("variant", "Period::Day"),
    ("variant", "Period::Event"),
    ("variant", "Period::Month"),
    ("variant", "Period::Quarter"),
    ("variant", "Period::Year"),
    ("type", "BeaObservation"),
    ("field", "BeaObservation::dataset_id"),
    ("field", "BeaObservation::frequency"),
    ("field", "BeaObservation::line_number"),
    ("field", "BeaObservation::period"),
    ("field", "BeaObservation::table_id"),
    ("field", "BeaObservation::unit"),
    ("field", "BeaObservation::value"),
    ("field", "BeaObservation::vintage"),
    ("type", "BeaUnit"),
    ("fn", "BeaUnit::as_str"),
    ("fn", "BeaUnit::new"),
    ("type", "Date"),
    ("field", "Date::day"),
    ("field", "Date::month"),
    ("field", "Date::year"),
    ("fn", "Date::days_in_month"),
    ("fn", "Date::is_leap_year"),
    ("fn", "Date::new"),
    ("fn", "Date::parse"),
    ("fn", "validate_date"),
    ("fn", "validate_observation"),
    ("fn", "validate_period"),
];

/// 覆盖登记表：只登记**真实发生**的调用/读取，不登记「计划要调用」。
mod cover {
    use std::collections::BTreeSet;
    use std::sync::{Mutex, OnceLock};

    static EXECUTED: OnceLock<Mutex<BTreeSet<(&'static str, &'static str)>>> = OnceLock::new();

    fn log() -> &'static Mutex<BTreeSet<(&'static str, &'static str)>> {
        EXECUTED.get_or_init(|| Mutex::new(BTreeSet::new()))
    }

    /// 登记一次真实执行。清单外的 `(类别, id)` 立即 panic，防止调用点与清单漂移。
    pub fn hit(kind: &'static str, id: &'static str) {
        assert!(
            super::E2E_MANIFEST
                .iter()
                .any(|(declared_kind, declared_id)| *declared_kind == kind && *declared_id == id),
            "登记了清单外的公开条目：{kind} {id}"
        );
        log().lock().expect("覆盖登记表锁中毒").insert((kind, id));
    }

    pub fn executed() -> BTreeSet<(&'static str, &'static str)> {
        log().lock().expect("覆盖登记表锁中毒").clone()
    }
}

/// 覆盖登记的简写入口（保持调用点可读）。
fn hit(kind: &'static str, id: &'static str) {
    cover::hit(kind, id);
}

/// 清单自身良构：类别取值域合法、`(类别, id)` 不重复。
fn assert_manifest_wellformed() {
    let mut seen: BTreeSet<(&str, &str)> = BTreeSet::new();
    for (kind, id) in E2E_MANIFEST {
        assert!(
            matches!(*kind, "fn" | "type" | "field" | "const" | "variant"),
            "未知条目类别 {kind}（id={id}）"
        );
        assert!(seen.insert((kind, id)), "清单重复条目：{kind} {id}");
    }
    assert!(!E2E_MANIFEST.is_empty(), "清单不得为空");
}

/// 收尾断言：声明集合与执行集合必须**双向相等**。
fn assert_coverage_complete() {
    let declared: BTreeSet<(&str, &str)> = E2E_MANIFEST.iter().copied().collect();
    let executed = cover::executed();

    let missing: Vec<&(&str, &str)> = declared.difference(&executed).collect();
    let ghost: Vec<&(&str, &str)> = executed.difference(&declared).collect();

    assert!(
        missing.is_empty(),
        "以下 {} 条公开条目被声明却未执行：{missing:?}",
        missing.len()
    );
    assert!(
        ghost.is_empty(),
        "以下 {} 条执行未登记在清单：{ghost:?}",
        ghost.len()
    );
    eprintln!(
        "E2E 覆盖：{}/{} 条公开条目全部执行（beax）",
        executed.len(),
        declared.len()
    );
}

/// 构造一条形态合法的合成观测（供各守卫的入参使用）。
fn sample_observation(table_id: &str, frequency: Frequency) -> BeaObservation {
    BeaObservation {
        dataset_id: NIPA.to_owned(),
        table_id: table_id.to_owned(),
        line_number: 1,
        period: Period::Year(2026),
        value: BeaValue::Present(1.0),
        unit: BeaUnit::new("Index").expect("合成单位合法"),
        frequency,
        vintage: None,
    }
}

/// 阶段 1：38 个公开常量逐条取值断言（3 签核 + 11 Dataset + 17 表号 + 4 集合/锚点 + 3 归属）。
fn phase_constants() {
    // —— 授权 / 裁定三常量 ——
    hit("const", "BEA_DECISION_ID");
    assert_eq!(BEA_DECISION_ID, "BEA-PROD-2026-08-17-approve");
    hit("const", "BEA_SIGNED_BY");
    assert_eq!(BEA_SIGNED_BY, "ZoneCNH");
    hit("const", "LIVE_OFFICIAL_PIT_DECISION");
    assert_eq!(LIVE_OFFICIAL_PIT_DECISION, "NO-GO");

    // —— 11 个 Dataset 常量（取值 + 唯一性）——
    let dataset_values: [(&str, &str, &str); 11] = [
        ("NIPA", NIPA, "NIPA"),
        (
            "NI_UNDERLYING_DETAIL",
            NI_UNDERLYING_DETAIL,
            "NIUnderlyingDetail",
        ),
        ("FIXED_ASSETS", FIXED_ASSETS, "FixedAssets"),
        ("ITA", ITA, "ITA"),
        ("IIP", IIP, "IIP"),
        ("INTL_SERV_TRADE", INTL_SERV_TRADE, "IntlServTrade"),
        ("GDP_BY_INDUSTRY", GDP_BY_INDUSTRY, "GDPbyIndustry"),
        (
            "UNDERLYING_GDP_BY_INDUSTRY",
            UNDERLYING_GDP_BY_INDUSTRY,
            "UnderlyingGDPbyIndustry",
        ),
        ("INPUT_OUTPUT", INPUT_OUTPUT, "InputOutput"),
        ("REGIONAL", REGIONAL, "Regional"),
        ("MNE", MNE, "MNE"),
    ];
    let mut seen_datasets: BTreeSet<&str> = BTreeSet::new();
    for (id, value, expected) in dataset_values {
        hit("const", id);
        assert_eq!(value, expected, "{id} 取值不符");
        assert!(
            seen_datasets.insert(value),
            "{id} 与其它 Dataset 常量重复：{value}"
        );
    }

    // —— 17 个表号常量（常量名即取值）——
    let table_values: [(&str, &str); 17] = [
        ("T10101", T10101),
        ("T10102", T10102),
        ("T10103", T10103),
        ("T10104", T10104),
        ("T10105", T10105),
        ("T10106", T10106),
        ("T10107", T10107),
        ("T10201", T10201),
        ("T11000", T11000),
        ("T20100", T20100),
        ("T20300", T20300),
        ("T20301", T20301),
        ("T20600", T20600),
        ("T30100", T30100),
        ("T40100", T40100),
        ("T50100", T50100),
        ("T70100", T70100),
    ];
    let mut seen_tables: BTreeSet<&str> = BTreeSet::new();
    for (id, value) in table_values {
        hit("const", id);
        assert_eq!(value, id, "表号常量 {id} 取值应等于其名");
        assert!(
            seen_tables.insert(value),
            "{id} 与其它表号常量重复：{value}"
        );
    }

    // —— Dataset 全集：11 条、无重复、覆盖全部常量取值 ——
    hit("const", "DATASETS");
    assert_eq!(DATASETS.len(), 11, "Dataset 全集应为 11");
    let dataset_set: BTreeSet<&str> = DATASETS.iter().copied().collect();
    assert_eq!(dataset_set.len(), 11, "Dataset 全集出现重复");
    for (_, value, _) in dataset_values {
        assert!(dataset_set.contains(value), "DATASETS 缺少 {value}");
    }

    // —— NIPA 表白名单：P0 2 表 + P1 12 表 ——
    hit("const", "CORE_NIPA_P0_TABLES");
    assert_eq!(CORE_NIPA_P0_TABLES.len(), 2, "P0 应为 2 表");
    assert!(CORE_NIPA_P0_TABLES.contains(&T10101));
    assert!(CORE_NIPA_P0_TABLES.contains(&T20100));
    hit("const", "CORE_NIPA_P1_TABLES");
    assert_eq!(CORE_NIPA_P1_TABLES.len(), 12, "P1 应为 12 表");
    for table in CORE_NIPA_P1_TABLES {
        assert!(
            !CORE_NIPA_P0_TABLES.contains(table),
            "{table} 不得同时属 P0"
        );
    }
    hit("const", "CORE_PCE_ANCHOR_TABLE");
    assert_eq!(CORE_PCE_ANCHOR_TABLE, T20100);
    assert!(CORE_NIPA_P0_TABLES.contains(&CORE_PCE_ANCHOR_TABLE));

    // —— 归属方常量（跨源路由）——
    hit("const", "CURVE_BUILD_OWNER");
    assert_eq!(CURVE_BUILD_OWNER, "yieldx");
    hit("const", "CORE_PCE_PRIMARY_OWNER");
    assert_eq!(CORE_PCE_PRIMARY_OWNER, "fredx");

    // —— 近义非同 ID 表：6 对 ——
    hit("const", "NEAR_SYNONYM_PAIRS");
    assert_eq!(NEAR_SYNONYM_PAIRS.len(), 6, "涉 beax 的近义对应为 6 对");
}

/// 阶段 2：值对象（日期 / 期间 / 频率 / 单位 / 缺失 / 观测 + 穷尽解构 + 枚举变体逐个构造）。
fn phase_value_types() {
    // —— Date：类型 + 3 字段 + 4 方法 ——
    hit("type", "Date");
    hit("fn", "Date::new");
    let date = Date::new(2026, 8, 20).expect("合法日期必须可构造");
    hit("fn", "Date::parse");
    assert_eq!(Date::parse("2026-08-20").expect("严格 ISO 应可解析"), date);
    for bad in [
        "2026-8-20",
        "2026/08/20",
        "2026-02-30",
        "2026-13-01",
        "0000-01-01",
        "2026-08-20T00:00:00Z",
        "",
    ] {
        assert!(Date::parse(bad).is_err(), "必须拒绝非严格形态：{bad}");
    }
    hit("fn", "Date::is_leap_year");
    assert!(Date::is_leap_year(2024));
    assert!(!Date::is_leap_year(1900));
    assert!(Date::is_leap_year(2000));
    hit("fn", "Date::days_in_month");
    assert_eq!(Date::days_in_month(2024, 2), 29);
    assert_eq!(Date::days_in_month(2026, 2), 28);
    assert_eq!(Date::days_in_month(2026, 4), 30);
    assert_eq!(Date::days_in_month(2026, 13), 0, "非法月份不 panic，返回 0");
    hit("fn", "validate_date");
    validate_date(&date).expect("合法日期必须通过校验");
    assert!(validate_date(&Date {
        year: 0,
        month: 1,
        day: 1
    })
    .is_err());
    let Date { year, month, day } = date;
    hit("field", "Date::year");
    assert_eq!(year, 2026);
    hit("field", "Date::month");
    assert_eq!(month, 8);
    hit("field", "Date::day");
    assert_eq!(day, 20);

    // —— Period：类型 + 5 变体 + validate_period ——
    hit("type", "Period");
    let day_period = Period::Day(date);
    hit("variant", "Period::Day");
    let month_period = Period::Month {
        year: 2026,
        month: 6,
    };
    hit("variant", "Period::Month");
    let quarter_period = Period::Quarter {
        year: 2026,
        quarter: 2,
    };
    hit("variant", "Period::Quarter");
    let year_period = Period::Year(2026);
    hit("variant", "Period::Year");
    let event_period = Period::Event { date };
    hit("variant", "Period::Event");
    hit("fn", "validate_period");
    for ok in [
        day_period,
        month_period,
        quarter_period,
        year_period,
        event_period,
    ] {
        validate_period(&ok).expect("合法期间必须通过校验");
    }
    assert!(validate_period(&Period::Month {
        year: 2026,
        month: 13
    })
    .is_err());
    assert!(validate_period(&Period::Quarter {
        year: 2026,
        quarter: 5
    })
    .is_err());
    assert!(validate_period(&Period::Year(0)).is_err());
    assert!(validate_period(&Period::Day(Date {
        year: 2026,
        month: 2,
        day: 30
    }))
    .is_err());

    // —— Frequency：类型 + 7 变体 + as_str / parse 回环 ——
    hit("type", "Frequency");
    let frequencies: [(&str, Frequency, &str); 7] = [
        ("Frequency::Daily", Frequency::Daily, "daily"),
        ("Frequency::Weekly", Frequency::Weekly, "weekly"),
        ("Frequency::Monthly", Frequency::Monthly, "monthly"),
        ("Frequency::Quarterly", Frequency::Quarterly, "quarterly"),
        ("Frequency::Annual", Frequency::Annual, "annual"),
        ("Frequency::Event", Frequency::Event, "event"),
        ("Frequency::Irregular", Frequency::Irregular, "irregular"),
    ];
    for (id, frequency, token) in frequencies {
        hit("variant", id);
        hit("fn", "Frequency::as_str");
        assert_eq!(frequency.as_str(), token, "{id} 记号不符");
        hit("fn", "Frequency::parse");
        assert_eq!(
            Frequency::parse(token).expect("记号回环必须成功"),
            frequency
        );
    }
    assert!(Frequency::parse("Monthly").is_err(), "只接受小写记号");

    // —— BeaUnit：开放 newtype ——
    hit("type", "BeaUnit");
    hit("fn", "BeaUnit::new");
    let unit = BeaUnit::new("Billions of dollars").expect("合法单位必须可构造");
    hit("fn", "BeaUnit::as_str");
    assert_eq!(unit.as_str(), "Billions of dollars");
    assert!(BeaUnit::new("").is_err());
    assert!(BeaUnit::new(" dollars").is_err());
    assert!(BeaUnit::new("dollars\n").is_err());

    // —— 缺失原因 / 观测值 ——
    hit("type", "BeaMissingReason");
    hit("variant", "BeaMissingReason::NoObservation");
    let missing = BeaValue::Missing(BeaMissingReason::NoObservation);
    hit("type", "BeaValue");
    hit("variant", "BeaValue::Missing");
    let present = BeaValue::Present(2.5);
    hit("variant", "BeaValue::Present");
    hit("fn", "BeaValue::as_f64");
    assert_eq!(present.as_f64(), Some(2.5));
    assert_eq!(missing.as_f64(), None);
    assert_ne!(missing.as_f64(), Some(0.0), "缺失不得静默折算为 0");
    hit("fn", "BeaValue::is_missing");
    assert!(missing.is_missing());
    assert!(!present.is_missing());

    // —— BeaObservation：8 字段穷尽解构 + validate_observation ——
    let observation = sample_observation(T10101, Frequency::Quarterly);
    hit("type", "BeaObservation");
    hit("fn", "validate_observation");
    validate_observation(&observation).expect("合法观测必须通过校验");

    let mut bad = observation.clone();
    bad.dataset_id = " NIPA".to_owned();
    assert!(validate_observation(&bad).is_err(), "首尾空白 dataset_id");
    let mut bad = observation.clone();
    bad.table_id = String::new();
    assert!(validate_observation(&bad).is_err(), "空 table_id");
    let mut bad = observation.clone();
    bad.line_number = 0;
    assert!(validate_observation(&bad).is_err(), "行号 < 1");
    let mut bad = observation.clone();
    bad.value = BeaValue::Present(f64::NAN);
    assert!(validate_observation(&bad).is_err(), "NaN 不得静默通过");
    let mut bad = observation.clone();
    bad.period = Period::Quarter {
        year: 2026,
        quarter: 5,
    };
    assert!(validate_observation(&bad).is_err(), "非法季度");
    let mut bad = observation.clone();
    bad.vintage = Some(Date {
        year: 2026,
        month: 2,
        day: 30,
    });
    assert!(validate_observation(&bad).is_err(), "非法修订日期");

    let BeaObservation {
        dataset_id,
        table_id,
        line_number,
        period,
        value,
        unit,
        frequency,
        vintage,
    } = observation;
    hit("field", "BeaObservation::dataset_id");
    assert_eq!(dataset_id, "NIPA");
    hit("field", "BeaObservation::table_id");
    assert_eq!(table_id, "T10101");
    hit("field", "BeaObservation::line_number");
    assert_eq!(line_number, 1);
    hit("field", "BeaObservation::period");
    assert_eq!(period, Period::Year(2026));
    hit("field", "BeaObservation::value");
    assert_eq!(value.as_f64(), Some(1.0));
    hit("field", "BeaObservation::unit");
    assert_eq!(unit.as_str(), "Index");
    hit("field", "BeaObservation::frequency");
    assert_eq!(frequency, Frequency::Quarterly);
    hit("field", "BeaObservation::vintage");
    assert!(vintage.is_none(), "未给修订标识时不得伪造");
}

/// 阶段 3：错误面（8 变体逐个构造 + 分类 + 可重试 + 结果别名）。
fn phase_error_plane() {
    hit("type", "BeaError");
    let cases: [(&str, BeaError, &str, BeaErrorKind, bool); 8] = [
        (
            "BeaError::Invalid",
            BeaError::Invalid("e2e".into()),
            "BeaErrorKind::Invalid",
            BeaErrorKind::Invalid,
            false,
        ),
        (
            "BeaError::Missing",
            BeaError::Missing("e2e".into()),
            "BeaErrorKind::Missing",
            BeaErrorKind::Missing,
            false,
        ),
        (
            "BeaError::AuthorizationDenied",
            BeaError::AuthorizationDenied("e2e".into()),
            "BeaErrorKind::AuthorizationDenied",
            BeaErrorKind::AuthorizationDenied,
            false,
        ),
        (
            "BeaError::RoutedElsewhere",
            BeaError::RoutedElsewhere("e2e".into()),
            "BeaErrorKind::RoutedElsewhere",
            BeaErrorKind::RoutedElsewhere,
            false,
        ),
        (
            "BeaError::WriteAuthorityDenied",
            BeaError::WriteAuthorityDenied("e2e".into()),
            "BeaErrorKind::WriteAuthorityDenied",
            BeaErrorKind::WriteAuthorityDenied,
            false,
        ),
        (
            "BeaError::SemanticallyRejected",
            BeaError::SemanticallyRejected("e2e".into()),
            "BeaErrorKind::SemanticallyRejected",
            BeaErrorKind::SemanticallyRejected,
            false,
        ),
        (
            "BeaError::NotApplicable",
            BeaError::NotApplicable("e2e".into()),
            "BeaErrorKind::NotApplicable",
            BeaErrorKind::NotApplicable,
            false,
        ),
        (
            "BeaError::Invariant",
            BeaError::Invariant("e2e".into()),
            "BeaErrorKind::Invariant",
            BeaErrorKind::Invariant,
            true,
        ),
    ];
    hit("type", "BeaErrorKind");
    for (error_id, error, kind_id, expected_kind, retryable) in cases {
        hit("variant", error_id);
        hit("variant", kind_id);
        hit("fn", "BeaError::kind");
        hit("fn", "BeaError::is_retryable");
        assert_eq!(error.kind(), expected_kind, "{error_id} 分类不符");
        assert_eq!(error.is_retryable(), retryable, "{error_id} 可重试分类不符");
        assert!(!error.to_string().is_empty(), "{error_id} Display 不得为空");
    }
    // 仅 Invariant 值得复查重跑；其余一律不可重试。
    assert!(BeaError::Invariant("x".into()).is_retryable());
    assert!(!BeaError::Invalid("x".into()).is_retryable());

    // —— BeaResult 别名：成功 / 失败两路 ——
    fn as_result(value: u8) -> BeaResult<u8> {
        Ok(value)
    }
    hit("type", "BeaResult");
    assert_eq!(as_result(7).expect("Ok 分支"), 7);
    let failed: BeaResult<u8> = Err(BeaError::Invalid("e2e".into()));
    assert!(failed.is_err());
}

/// 阶段 4：Dataset 与 NIPA 表白名单守卫（全部 10 个只读判定/守卫函数）。
fn phase_dataset_plane() {
    hit("fn", "is_known_dataset");
    for dataset in DATASETS.iter().copied() {
        assert!(is_known_dataset(dataset), "{dataset} 应为已登记 Dataset");
    }
    assert!(!is_known_dataset("BLS"), "范围外 Dataset 不得被判为已登记");

    hit("fn", "dataset_priority");
    for dataset in DATASETS.iter().copied() {
        assert!(dataset_priority(dataset).is_some(), "{dataset} 应有优先级");
    }
    assert_eq!(dataset_priority(NIPA), Some(0), "NIPA 为 P0");
    assert_eq!(dataset_priority(ITA), Some(1), "其余为 P1");
    assert_eq!(dataset_priority("NotADataset"), None);

    hit("fn", "is_core_p0_table");
    for table in CORE_NIPA_P0_TABLES.iter().copied() {
        assert!(is_core_p0_table(table), "{table} 应为 P0 表");
    }
    assert!(!is_core_p0_table(T10102));

    hit("fn", "is_core_p1_table");
    for table in CORE_NIPA_P1_TABLES.iter().copied() {
        assert!(is_core_p1_table(table), "{table} 应为 P1 表");
    }
    assert!(!is_core_p1_table(T10101));

    hit("fn", "is_known_nipa_table");
    for table in CORE_NIPA_P0_TABLES
        .iter()
        .chain(CORE_NIPA_P1_TABLES.iter())
        .copied()
    {
        assert!(is_known_nipa_table(table), "{table} 应在 14 表白名单内");
    }
    for outside in [T20300, T20600, T11000, "T99999"] {
        assert!(!is_known_nipa_table(outside), "{outside} 不在白名单");
    }

    hit("fn", "table_priority");
    assert_eq!(table_priority(T10101), Some(0));
    assert_eq!(table_priority(T10102), Some(1));
    assert_eq!(table_priority(T11000), None);

    hit("fn", "table_frequency");
    assert_eq!(table_frequency(T10101), Some(Frequency::Quarterly));
    assert_eq!(table_frequency(T20100), Some(Frequency::Monthly));
    assert_eq!(table_frequency(T10102), None, "未声明频率的表不得推定");

    hit("fn", "ensure_known_dataset");
    ensure_known_dataset(NIPA).expect("NIPA 已登记");
    assert_eq!(
        ensure_known_dataset("BLS")
            .expect_err("范围外 Dataset 必须被拒绝")
            .kind(),
        BeaErrorKind::SemanticallyRejected
    );

    hit("fn", "ensure_nipa_table");
    ensure_nipa_table(T10101).expect("T10101 在白名单内");
    assert_eq!(
        ensure_nipa_table(T20600)
            .expect_err("白名单外表号必须被拒绝")
            .kind(),
        BeaErrorKind::SemanticallyRejected
    );

    hit("fn", "ensure_table_frequency");
    ensure_table_frequency(&sample_observation(T20100, Frequency::Monthly))
        .expect("T20100 月频与清单一致");
    assert!(
        ensure_table_frequency(&sample_observation(T20100, Frequency::Quarterly)).is_err(),
        "T20100 频率漂移必须被拒绝"
    );
    ensure_table_frequency(&sample_observation(T10102, Frequency::Annual))
        .expect("未声明频率的表不做一致性判定");
}

/// 阶段 5：跨源路由守卫（曲线路由 / 核心 PCE 主责 / 近义非同 ID / GDI 候选）。
fn phase_routing_plane() {
    hit("type", "BeaClaim");
    hit("variant", "BeaClaim::SupplementaryTableValue");
    hit("fn", "ensure_claim_local");
    ensure_claim_local(BeaClaim::SupplementaryTableValue).expect("表内补充观测是本域职责");

    hit("variant", "BeaClaim::CorePcePrimary");
    let pce = ensure_claim_local(BeaClaim::CorePcePrimary).expect_err("不得反向主张核心 PCE 主责");
    assert_eq!(pce.kind(), BeaErrorKind::WriteAuthorityDenied);
    assert!(
        pce.to_string().contains(CORE_PCE_PRIMARY_OWNER),
        "拒绝理由须点名主责方：{pce}"
    );

    hit("variant", "BeaClaim::YieldCurveConstruction");
    let curve =
        ensure_claim_local(BeaClaim::YieldCurveConstruction).expect_err("曲线产品须路由他处");
    assert_eq!(curve.kind(), BeaErrorKind::RoutedElsewhere);
    assert!(
        curve.to_string().contains(CURVE_BUILD_OWNER),
        "拒绝理由须点名归属方：{curve}"
    );

    hit("fn", "is_near_synonym_pair");
    hit("fn", "ensure_not_silent_substitution");
    for (left, right) in NEAR_SYNONYM_PAIRS.iter().copied() {
        assert!(is_near_synonym_pair(left, right), "{left}/{right} 应被识别");
        assert!(
            is_near_synonym_pair(right, left),
            "{right}/{left} 双向都应被识别"
        );
        assert_eq!(
            ensure_not_silent_substitution(left, right)
                .expect_err("近义非同 ID 必须被拒绝")
                .kind(),
            BeaErrorKind::SemanticallyRejected
        );
    }
    ensure_not_silent_substitution(T10101, T20100).expect("非同义对不得被误拒");

    hit("fn", "gdi_candidate_table");
    assert_eq!(gdi_candidate_table(), T11000, "GDI 候选表为 T11000");

    hit("fn", "is_gdp_or_personal_income_table");
    assert!(is_gdp_or_personal_income_table(T10101));
    assert!(is_gdp_or_personal_income_table(T20100));
    assert!(!is_gdp_or_personal_income_table(T10102));
}

/// 阶段 6：publication 语义三元组（时间精度 + 可得性证据层 + 正式 PIT 资格）。
fn phase_publication_plane() {
    hit("type", "TimePrecision");
    hit("variant", "TimePrecision::Date");
    hit("variant", "TimePrecision::Instant");
    let precisions = [TimePrecision::Date, TimePrecision::Instant];
    assert_eq!(precisions.len(), 2);

    hit("type", "AvailabilityEvidence");
    hit("variant", "AvailabilityEvidence::Official");
    hit("variant", "AvailabilityEvidence::Calendar");
    hit("variant", "AvailabilityEvidence::Inferred");
    let availability = [
        AvailabilityEvidence::Official,
        AvailabilityEvidence::Calendar,
        AvailabilityEvidence::Inferred,
    ];
    assert_eq!(availability.len(), 3);

    hit("type", "PitEligibility");
    hit("variant", "PitEligibility::Formal");
    hit("variant", "PitEligibility::NotEligible");
    let formal = PitEligibility::Formal;
    let not_eligible = PitEligibility::NotEligible;
    assert_ne!(formal, not_eligible, "正式 PIT 与非资格必须是两个不同取值");

    hit("fn", "bea_publication_semantics");
    hit("type", "BeaPublicationSemantics");
    let BeaPublicationSemantics {
        time_precision,
        availability,
        eligibility,
    } = bea_publication_semantics();
    hit("field", "BeaPublicationSemantics::time_precision");
    assert_eq!(time_precision, TimePrecision::Date, "源只有日期");
    hit("field", "BeaPublicationSemantics::availability");
    assert_eq!(availability, AvailabilityEvidence::Inferred, "离线推断层");
    hit("field", "BeaPublicationSemantics::eligibility");
    assert_eq!(eligibility, PitEligibility::NotEligible, "不进正式 PIT");
    assert_ne!(time_precision, TimePrecision::Instant, "不得静默升格为时刻");

    hit("fn", "is_formal_pit_eligible");
    assert!(!is_formal_pit_eligible(), "本层恒为不可进正式 PIT");
}

/// 阶段 7：fail-closed 授权判定（离线 / reference 被覆盖，live 不被覆盖）。
fn phase_authorization_plane() {
    hit("type", "BeaAccessMode");
    let modes = [
        BeaAccessMode::Offline,
        BeaAccessMode::ReferenceOnly,
        BeaAccessMode::Live,
    ];
    hit("variant", "BeaAccessMode::Offline");
    hit("variant", "BeaAccessMode::ReferenceOnly");
    hit("variant", "BeaAccessMode::Live");
    assert_eq!(modes.len(), 3);

    hit("fn", "mode_label");
    assert_eq!(mode_label(BeaAccessMode::Offline), "offline");
    assert_eq!(mode_label(BeaAccessMode::ReferenceOnly), "reference_only");
    assert_eq!(mode_label(BeaAccessMode::Live), "live");

    hit("fn", "documented_bea_evidence");
    let evidence = documented_bea_evidence();
    hit("type", "BeaAuthorizationEvidence");
    hit("fn", "validate_authorization_evidence");
    validate_authorization_evidence(&evidence).expect("已登记证据必须形态合法");

    hit("fn", "accepts_no_official_pit");
    assert!(
        accepts_no_official_pit(),
        "BEA 无官方 vintage 面属已登记例外"
    );
    hit("fn", "live_official_pit_decision");
    assert_eq!(live_official_pit_decision(), LIVE_OFFICIAL_PIT_DECISION);

    let BeaAuthorizationEvidence {
        decision_id,
        signed_by,
        signed_at,
        valid_until,
        authorized_modes,
        accept_no_pit,
        live_official_pit,
        scope_note,
    } = evidence.clone();
    hit("field", "BeaAuthorizationEvidence::decision_id");
    assert_eq!(decision_id, BEA_DECISION_ID);
    hit("field", "BeaAuthorizationEvidence::signed_by");
    assert_eq!(signed_by, BEA_SIGNED_BY);
    hit("field", "BeaAuthorizationEvidence::signed_at");
    assert_eq!(signed_at, Date::new(2026, 8, 17).ok());
    hit("field", "BeaAuthorizationEvidence::valid_until");
    assert!(valid_until.is_none(), "已登记证据未声明有效期上界");
    hit("field", "BeaAuthorizationEvidence::authorized_modes");
    assert_eq!(
        authorized_modes,
        vec![BeaAccessMode::Offline, BeaAccessMode::ReferenceOnly]
    );
    hit("field", "BeaAuthorizationEvidence::accept_no_pit");
    assert!(accept_no_pit);
    hit("field", "BeaAuthorizationEvidence::live_official_pit");
    assert_eq!(live_official_pit, LIVE_OFFICIAL_PIT_DECISION);
    hit("field", "BeaAuthorizationEvidence::scope_note");
    assert!(scope_note.contains("offline_and_reference_only"));

    let as_of = Date::new(2026, 8, 20).expect("评估日期合法");
    hit("fn", "authorize_bea");
    hit("type", "BeaAuthorization");
    hit("variant", "BeaAuthorization::Authorized");
    for mode in [BeaAccessMode::Offline, BeaAccessMode::ReferenceOnly] {
        match authorize_bea(Some(&evidence), mode, as_of) {
            BeaAuthorization::Authorized { scope } => {
                assert!(scope.contains(BEA_DECISION_ID), "scope 须含签核编号");
                assert!(scope.contains(mode_label(mode)), "scope 须含模式记号");
            }
            other => panic!("应授权 {mode:?}，实得 {other:?}"),
        }
    }

    // live 不在覆盖集合内（live_official_pit = NO-GO）。
    hit("variant", "BeaAuthorization::Denied");
    match authorize_bea(Some(&evidence), BeaAccessMode::Live, as_of) {
        BeaAuthorization::Denied { reason } => {
            assert!(reason.contains("live"), "拒绝理由须点名 live：{reason}");
        }
        other => panic!("应拒绝 live，实得 {other:?}"),
    }

    // —— 其余 fail-closed 拒绝路径 ——
    match authorize_bea(None, BeaAccessMode::Offline, as_of) {
        BeaAuthorization::Denied { reason } => assert_eq!(reason, "缺少 Owner 签核证据"),
        other => panic!("证据缺失应拒绝，实得 {other:?}"),
    }
    let mut empty_decision = documented_bea_evidence();
    empty_decision.decision_id = String::new();
    match authorize_bea(Some(&empty_decision), BeaAccessMode::Offline, as_of) {
        BeaAuthorization::Denied { reason } => assert_eq!(reason, "签核编号不明"),
        other => panic!("签核编号不明应拒绝，实得 {other:?}"),
    }
    let mut unknown_signer = documented_bea_evidence();
    unknown_signer.signed_by = String::new();
    match authorize_bea(Some(&unknown_signer), BeaAccessMode::Offline, as_of) {
        BeaAuthorization::Denied { reason } => assert_eq!(reason, "签署者不明"),
        other => panic!("签署者不明应拒绝，实得 {other:?}"),
    }
    let mut unknown_scope = documented_bea_evidence();
    unknown_scope.authorized_modes.clear();
    match authorize_bea(Some(&unknown_scope), BeaAccessMode::Offline, as_of) {
        BeaAuthorization::Denied { reason } => assert_eq!(reason, "证据未声明任何被覆盖的范围"),
        other => panic!("覆盖范围不明应拒绝，实得 {other:?}"),
    }
    // 真过期：有效区间自洽（签署日 ≤ 上界），但评估日晚于上界。
    let mut expired = documented_bea_evidence();
    expired.signed_at = Date::new(2026, 8, 1).ok();
    expired.valid_until = Date::new(2026, 8, 10).ok();
    match authorize_bea(Some(&expired), BeaAccessMode::Offline, as_of) {
        BeaAuthorization::Denied { reason } => assert_eq!(reason, "证据已过期"),
        other => panic!("过期证据应拒绝，实得 {other:?}"),
    }
    // 上界当日（含）不算过期：`as_of == valid_until` 必须放行。
    let mut inclusive = documented_bea_evidence();
    inclusive.signed_at = Date::new(2026, 8, 1).ok();
    inclusive.valid_until = Date::new(2026, 8, 20).ok();
    match authorize_bea(Some(&inclusive), BeaAccessMode::Offline, as_of) {
        BeaAuthorization::Authorized { .. } => {}
        other => panic!("上界当日不得判过期，实得 {other:?}"),
    }

    // 加固路径：纯空白范围说明、早于签署日的评估日、非法评估日一律拒绝。
    let mut blank_scope = documented_bea_evidence();
    blank_scope.scope_note = "   ".to_owned();
    match authorize_bea(Some(&blank_scope), BeaAccessMode::Offline, as_of) {
        BeaAuthorization::Denied { reason } => assert_eq!(reason, "证据范围说明不明"),
        other => panic!("纯空白范围说明应拒绝，实得 {other:?}"),
    }
    match authorize_bea(
        Some(&evidence),
        BeaAccessMode::Offline,
        Date::new(2026, 8, 1).expect("合法"),
    ) {
        BeaAuthorization::Denied { reason } => assert_eq!(reason, "证据尚未签署生效"),
        other => panic!("早于签署日应拒绝，实得 {other:?}"),
    }
    match authorize_bea(
        Some(&evidence),
        BeaAccessMode::Offline,
        Date {
            year: 2026,
            month: 99,
            day: 99,
        },
    ) {
        BeaAuthorization::Denied { reason } => assert_eq!(reason, "评估日期或证据有效区间非法"),
        other => panic!("非法评估日应拒绝，实得 {other:?}"),
    }

    // 证据自身形态非法（日期分量越界 / 有效区间倒置）必须被拒绝。
    let mut malformed = documented_bea_evidence();
    malformed.valid_until = Some(Date {
        year: 2026,
        month: 2,
        day: 30,
    });
    assert!(validate_authorization_evidence(&malformed).is_err());
    let mut inverted = documented_bea_evidence();
    inverted.signed_at = Date::new(2026, 8, 17).ok();
    inverted.valid_until = Date::new(2026, 8, 16).ok();
    assert!(
        validate_authorization_evidence(&inverted).is_err(),
        "签署日晚于有效期上界必须被拒绝"
    );
}

/// 阶段 8：离线解析器（读仓内**真实夹具**，并覆盖全部拒绝路径）。
fn phase_parse_plane() {
    const OBSERVATIONS_FIXTURE: &str = include_str!("fixtures/bea_observations_synthetic.json");
    const MISSING_FIXTURE: &str = include_str!("fixtures/bea_nipa_tables_synthetic.json");

    hit("fn", "parse_bea_observations");
    let observations =
        parse_bea_observations(OBSERVATIONS_FIXTURE).expect("真实合成夹具必须可解析");
    assert_eq!(observations.len(), 7, "夹具共 7 条记录");
    assert_eq!(observations[0].dataset_id, "NIPA");
    assert_eq!(observations[0].table_id, "T10101");
    assert_eq!(
        observations[0].period,
        Period::Quarter {
            year: 2026,
            quarter: 2
        },
        "季频按源日期投影为季度"
    );
    assert_eq!(
        observations[1].period,
        Period::Quarter {
            year: 2026,
            quarter: 1
        }
    );
    assert_eq!(
        observations[2].period,
        Period::Month {
            year: 2026,
            month: 6
        },
        "月频投影为月"
    );
    assert_eq!(
        observations[2].vintage,
        Date::new(2026, 7, 31).ok(),
        "修订标识须保留"
    );
    assert_eq!(observations[2].value.as_f64(), Some(19_000.0));
    assert_eq!(observations[5].period, Period::Year(2026), "年频投影为年");
    assert_eq!(observations[5].dataset_id, "Regional");
    for observation in &observations {
        validate_observation(observation).expect("夹具观测必须形态合法");
    }

    let missing = parse_bea_observations(MISSING_FIXTURE).expect("缺失夹具必须可解析");
    assert_eq!(missing.len(), 2);
    for observation in &missing {
        assert!(
            observation.value.is_missing(),
            "`.` 与 null 都须映射为具名缺失"
        );
        assert_eq!(observation.value.as_f64(), None);
        assert_ne!(observation.value.as_f64(), Some(0.0), "缺失不得折算为 0");
    }

    // —— 拒绝路径（原子失败）——
    assert!(
        parse_bea_observations(r#"{"records": [], "token": "x"}"#).is_err(),
        "顶层未知字段必须原子失败"
    );
    let unknown_record_field = r#"{"records": [
        {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly", "endpoint": "x"}
    ]}"#;
    assert!(
        parse_bea_observations(unknown_record_field).is_err(),
        "记录级未知字段必须原子失败"
    );
    let missing_required = r#"{"records": [
        {"dataset_id": "NIPA", "table_id": "T10101", "date": "2026-06-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly"}
    ]}"#;
    assert!(parse_bea_observations(missing_required).is_err(), "缺行号");
    let out_of_scope_dataset = r#"{"records": [
        {"dataset_id": "BLS", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly"}
    ]}"#;
    assert!(parse_bea_observations(out_of_scope_dataset).is_err());
    let out_of_whitelist_table = r#"{"records": [
        {"dataset_id": "NIPA", "table_id": "T20600", "line_number": 1, "date": "2026-06-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly"}
    ]}"#;
    assert!(parse_bea_observations(out_of_whitelist_table).is_err());
    let frequency_drift = r#"{"records": [
        {"dataset_id": "NIPA", "table_id": "T20100", "line_number": 1, "date": "2026-06-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly"}
    ]}"#;
    assert!(parse_bea_observations(frequency_drift).is_err());
    let illegal_value = r#"{"records": [
        {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
         "value": "n/a", "unit": "Index", "frequency": "quarterly"}
    ]}"#;
    assert!(parse_bea_observations(illegal_value).is_err());
    let illegal_date = r#"{"records": [
        {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-02-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly"}
    ]}"#;
    assert!(parse_bea_observations(illegal_date).is_err());
    let duplicate_identity = r#"{"records": [
        {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly"},
        {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
         "value": 1.1, "unit": "Index", "frequency": "quarterly"}
    ]}"#;
    assert_eq!(
        parse_bea_observations(duplicate_identity)
            .expect_err("重复身份必须被拒绝")
            .kind(),
        BeaErrorKind::SemanticallyRejected
    );

    // 畸形 JSON：不泄漏输入正文。
    let malformed = parse_bea_observations("{ not json").expect_err("畸形 JSON 必须被拒绝");
    assert_eq!(malformed.kind(), BeaErrorKind::Invalid);
    assert!(
        !malformed.to_string().contains("not json"),
        "错误消息不得回显输入：{malformed}"
    );
    assert!(malformed.to_string().contains("第"), "须给出定位");

    assert!(parse_bea_observations(r#"{"records": []}"#)
        .expect("空记录应可解析")
        .is_empty());
}

/// 单一驱动用例：保证阶段顺序与覆盖断言在同一个进程内完成。
#[test]
fn e2e_bea_all_public_api() {
    assert_manifest_wellformed();
    phase_constants();
    phase_value_types();
    phase_error_plane();
    phase_dataset_plane();
    phase_routing_plane();
    phase_publication_plane();
    phase_authorization_plane();
    phase_parse_plane();
    assert_coverage_complete();
}
