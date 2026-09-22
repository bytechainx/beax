#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! TDD 行为契约（特性 005 · `beax`）。
//!
//! 入口集合 = 本 crate 全部公开入口（类型 / 方法 / 判定函数 / 守卫 / 解析器 / 源事实常量）。
//! 下表每个入口先在 `/tmp` 变异副本上观测应红、再在本树观测绿；每条变异与红绿结果
//! 由配套的变异脚本逐条实跑（见 PR 描述），红==绿是标准形态
//! （同一用例在变异副本上失败、在原树上通过）。
//!
//! // TDD-PROBE: BeaErrorKind | 变异：把 `RoutedElsewhere` 的分类改映射为 `Invalid` | 红=error_kind_mapping_and_retryable | 绿=error_kind_mapping_and_retryable
//! // TDD-PROBE: BeaError | 变异：`Invalid` 变体的 Display 模板改为 `{0}`（丢中文前缀） | 红=error_display_is_chinese_and_does_not_echo_input | 绿=error_display_is_chinese_and_does_not_echo_input
//! // TDD-PROBE: BeaError::kind | 变异：`Missing` 与 `Invalid` 分类互换 | 红=error_kind_mapping_and_retryable | 绿=error_kind_mapping_and_retryable
//! // TDD-PROBE: BeaError::is_retryable | 变异：改为对全部变体返回 `true` | 红=error_kind_mapping_and_retryable | 绿=error_kind_mapping_and_retryable
//! // TDD-PROBE: BeaResult | 变异：`validate_observation` 丢弃 `validate_period` 的错误 | 红=observation_validation_rules | 绿=observation_validation_rules
//! // TDD-PROBE: Date | 变异：构造时把 `month` 与 `day` 写反 | 红=date_construction_and_calendar_rules | 绿=date_construction_and_calendar_rules
//! // TDD-PROBE: Date::new | 变异：年份下界校验被删除（`Date::new(0,1,1)` 通过） | 红=date_construction_and_calendar_rules | 绿=date_construction_and_calendar_rules
//! // TDD-PROBE: Date::parse | 变异：长度校验放宽为 `< 10`（接受带时间部分者） | 红=date_construction_and_calendar_rules | 绿=date_construction_and_calendar_rules
//! // TDD-PROBE: Date::is_leap_year | 变异：删除 `% 100` 项（1900 被判为闰年） | 红=date_construction_and_calendar_rules | 绿=date_construction_and_calendar_rules
//! // TDD-PROBE: Date::days_in_month | 变异：非法月返回 `31`（`_ => 0` 改为 `31`） | 红=date_construction_and_calendar_rules | 绿=date_construction_and_calendar_rules
//! // TDD-PROBE: validate_date | 变异：日范围校验改为 `day > 31` 才拒绝 | 红=date_construction_and_calendar_rules | 绿=date_construction_and_calendar_rules
//! // TDD-PROBE: Period | 变异：`Quarter` 的季范围上界由 `4` 改为 `5` | 红=period_validation_covers_every_variant | 绿=period_validation_covers_every_variant
//! // TDD-PROBE: validate_period | 变异：`Year(0)` 被接受 | 红=period_validation_covers_every_variant | 绿=period_validation_covers_every_variant
//! // TDD-PROBE: Frequency | 变异：`monthly` 记号映射到 `Quarterly` | 红=frequency_token_and_label_rules | 绿=frequency_token_and_label_rules
//! // TDD-PROBE: Frequency::parse | 变异：未知记号放行（`_` 分支返回 `Ok(Daily)`） | 红=frequency_token_and_label_rules | 绿=frequency_token_and_label_rules
//! // TDD-PROBE: Frequency::as_str | 变异：`Weekly` 记号改为 `week` | 红=frequency_token_and_label_rules | 绿=frequency_token_and_label_rules
//! // TDD-PROBE: BeaUnit | 变异：去掉控制字符校验 | 红=unit_construction_rules | 绿=unit_construction_rules
//! // TDD-PROBE: BeaUnit::new | 变异：允许空串与首尾空白 | 红=unit_construction_rules | 绿=unit_construction_rules
//! // TDD-PROBE: BeaUnit::as_str | 变异：恒返回固定串 | 红=unit_construction_rules | 绿=unit_construction_rules
//! // TDD-PROBE: BeaMissingReason | 变异：`.` 占位被解析成 `Present(0.0)` | 红=parse_keeps_missing_named | 绿=parse_keeps_missing_named
//! // TDD-PROBE: BeaValue | 变异：缺失被折算为 0（`as_f64` 返回 `Some(0.0)`） | 红=value_missing_is_named_never_zero | 绿=value_missing_is_named_never_zero
//! // TDD-PROBE: BeaValue::as_f64 | 变异：缺失时返回 `Some(0.0)` | 红=value_missing_is_named_never_zero | 绿=value_missing_is_named_never_zero
//! // TDD-PROBE: BeaValue::is_missing | 变异：语义反转（`Present` 返回 `true`） | 红=value_missing_is_named_never_zero | 绿=value_missing_is_named_never_zero
//! // TDD-PROBE: BeaObservation | 变异：`vintage` 恒被填为固定日期 | 红=parse_preserves_vintage | 绿=parse_preserves_vintage
//! // TDD-PROBE: validate_observation | 变异：删除有限数校验（NaN 通过） | 红=observation_validation_rules | 绿=observation_validation_rules
//! // TDD-PROBE: parse_bea_observations | 变异：跳过 `ensure_table_frequency` 调用 | 红=parse_rejects_table_frequency_drift | 绿=parse_rejects_table_frequency_drift
//! // TDD-PROBE: is_known_dataset | 变异：改为对任意 Dataset 返回 `true` | 红=source_scope_predicates | 绿=source_scope_predicates
//! // TDD-PROBE: dataset_priority | 变异：`NIPA` 的优先级由 `0` 改为 `1` | 红=source_scope_predicates | 绿=source_scope_predicates
//! // TDD-PROBE: is_core_p0_table | 变异：改为对任意表号返回 `true` | 红=table_whitelist_predicates | 绿=table_whitelist_predicates
//! // TDD-PROBE: is_core_p1_table | 变异：改为对任意表号返回 `true` | 红=table_whitelist_predicates | 绿=table_whitelist_predicates
//! // TDD-PROBE: is_known_nipa_table | 变异：把非白名单表 `T20600` 判为白名单成员 | 红=table_whitelist_predicates | 绿=table_whitelist_predicates
//! // TDD-PROBE: table_priority | 变异：P0 与 P1 的优先级对调 | 红=table_whitelist_predicates | 绿=table_whitelist_predicates
//! // TDD-PROBE: table_frequency | 变异：`T10101` 的清单频率改为年频 | 红=table_frequency_lookup | 绿=table_frequency_lookup
//! // TDD-PROBE: ensure_known_dataset | 变异：改为对未登记 Dataset 返回 `Ok(())` | 红=parse_rejects_out_of_scope_dataset | 绿=parse_rejects_out_of_scope_dataset
//! // TDD-PROBE: ensure_nipa_table | 变异：改为对白名单外表号返回 `Ok(())` | 红=parse_rejects_non_whitelisted_nipa_table | 绿=parse_rejects_non_whitelisted_nipa_table
//! // TDD-PROBE: ensure_table_frequency | 变异：不再拒绝频率漂移 | 红=parse_rejects_table_frequency_drift | 绿=parse_rejects_table_frequency_drift
//! // TDD-PROBE: DATASETS | 变异：从全集中删除 `NIPA` | 红=source_scope_predicates | 绿=source_scope_predicates
//! // TDD-PROBE: CORE_NIPA_P0_TABLES | 变异：把 `T20301` 加入 P0 白名单 | 红=table_whitelist_predicates | 绿=table_whitelist_predicates
//! // TDD-PROBE: CORE_NIPA_P1_TABLES | 变异：从 P1 白名单删除 `T10102` | 红=table_whitelist_predicates | 绿=table_whitelist_predicates
//! // TDD-PROBE: CORE_PCE_ANCHOR_TABLE | 变异：锚点表改为 `T10101` | 红=core_pce_anchor_constant | 绿=core_pce_anchor_constant
//! // TDD-PROBE: CURVE_BUILD_OWNER | 变异：归属方改为 `beax` | 红=curve_products_are_routed_not_local | 绿=curve_products_are_routed_not_local
//! // TDD-PROBE: CORE_PCE_PRIMARY_OWNER | 变异：归属方改为 `beax` | 红=core_pce_primary_is_never_claimed | 绿=core_pce_primary_is_never_claimed
//! // TDD-PROBE: BeaClaim | 变异：把 `CorePcePrimary` 与 `SupplementaryTableValue` 合并为放行 | 红=core_pce_primary_is_never_claimed | 绿=core_pce_primary_is_never_claimed
//! // TDD-PROBE: ensure_claim_local | 变异：对 `YieldCurveConstruction` 返回 `Ok(())` | 红=curve_products_are_routed_not_local | 绿=curve_products_are_routed_not_local
//! // TDD-PROBE: NEAR_SYNONYM_PAIRS | 变异：删除 `(GDP, GDI)` 一对 | 红=near_synonym_pairs_are_enforced | 绿=near_synonym_pairs_are_enforced
//! // TDD-PROBE: is_near_synonym_pair | 变异：改为单向匹配（反向失效） | 红=near_synonym_pairs_are_enforced | 绿=near_synonym_pairs_are_enforced
//! // TDD-PROBE: ensure_not_silent_substitution | 变异：命中后返回 `Ok(())` | 红=near_synonym_pairs_are_enforced | 绿=near_synonym_pairs_are_enforced
//! // TDD-PROBE: is_gdp_or_personal_income_table | 变异：只看 `T10101`（漏判 `T20100`） | 红=anchor_helpers | 绿=anchor_helpers
//! // TDD-PROBE: gdi_candidate_table | 变异：候选表号改为 `T10101` | 红=anchor_helpers | 绿=anchor_helpers
//! // TDD-PROBE: TimePrecision | 变异：`Date` 变体与 `Instant` 对调 | 红=publication_semantics_triple | 绿=publication_semantics_triple
//! // TDD-PROBE: AvailabilityEvidence | 变异：`Inferred` 与 `Official` 对调 | 红=publication_semantics_triple | 绿=publication_semantics_triple
//! // TDD-PROBE: PitEligibility | 变异：`NotEligible` 与 `Formal` 对调 | 红=publication_semantics_triple | 绿=publication_semantics_triple
//! // TDD-PROBE: BeaPublicationSemantics | 变异：构造时把 evidence 填为 `Official` | 红=publication_semantics_triple | 绿=publication_semantics_triple
//! // TDD-PROBE: bea_publication_semantics | 变异：返回 `PitEligibility::Formal` | 红=publication_semantics_triple | 绿=publication_semantics_triple
//! // TDD-PROBE: is_formal_pit_eligible | 变异：改为返回 `true` | 红=formal_pit_is_never_eligible | 绿=formal_pit_is_never_eligible
//! // TDD-PROBE: BeaAccessMode | 变异：覆盖模式改为 `[Live]` | 红=authz_documented_evidence_offline_and_reference | 绿=authz_documented_evidence_offline_and_reference
//! // TDD-PROBE: BeaAuthorization | 变异：证据缺失分支改为返回 `Authorized` | 红=authz_fail_closed_paths | 绿=authz_fail_closed_paths
//! // TDD-PROBE: BeaAuthorizationEvidence | 变异：证据的 `signed_by` 被清空 | 红=authz_documented_evidence_offline_and_reference | 绿=authz_documented_evidence_offline_and_reference
//! // TDD-PROBE: BEA_DECISION_ID | 变异：编号改为被替换的旧值 | 红=authz_documented_evidence_offline_and_reference | 绿=authz_documented_evidence_offline_and_reference
//! // TDD-PROBE: BEA_SIGNED_BY | 变异：签署者改为空串 | 红=authz_documented_evidence_offline_and_reference | 绿=authz_documented_evidence_offline_and_reference
//! // TDD-PROBE: LIVE_OFFICIAL_PIT_DECISION | 变异：裁定值改为 `GO` | 红=authz_live_is_not_covered | 绿=authz_live_is_not_covered
//! // TDD-PROBE: documented_bea_evidence | 变异：`accept_no_pit` 改为 `false` | 红=authz_live_is_not_covered | 绿=authz_live_is_not_covered
//! // TDD-PROBE: authorize_bea | 变异：`Live` 未被覆盖的判定失效（`contains` 检查删除） | 红=authz_live_is_not_covered | 绿=authz_live_is_not_covered
//! // TDD-PROBE: accepts_no_official_pit | 变异：改为返回 `false` | 红=authz_live_is_not_covered | 绿=authz_live_is_not_covered
//! // TDD-PROBE: live_official_pit_decision | 变异：改为返回 `GO` | 红=authz_live_is_not_covered | 绿=authz_live_is_not_covered
//! // TDD-PROBE: validate_authorization_evidence | 变异：跳过 `valid_until` 的日期校验 | 红=authz_evidence_date_validation | 绿=authz_evidence_date_validation
//! // TDD-PROBE: mode_label | 变异：`Offline` 的记号改为 `live` | 红=mode_labels_are_stable | 绿=mode_labels_are_stable
//! // TDD-PROBE: NIPA（常量） | 变异：字面量改为 `NIPA_Z` | 红=dataset_literals_match_the_manifest | 绿=dataset_literals_match_the_manifest
//! // TDD-PROBE: NI_UNDERLYING_DETAIL（常量） | 变异：字面量改为 `NI_UNDERLYING_DETAIL_Z` | 红=dataset_literals_match_the_manifest | 绿=dataset_literals_match_the_manifest
//! // TDD-PROBE: FIXED_ASSETS（常量） | 变异：字面量改为 `FIXED_ASSETS_Z` | 红=dataset_literals_match_the_manifest | 绿=dataset_literals_match_the_manifest
//! // TDD-PROBE: ITA（常量） | 变异：字面量改为 `ITA_Z` | 红=dataset_literals_match_the_manifest | 绿=dataset_literals_match_the_manifest
//! // TDD-PROBE: IIP（常量） | 变异：字面量改为 `IIP_Z` | 红=dataset_literals_match_the_manifest | 绿=dataset_literals_match_the_manifest
//! // TDD-PROBE: INTL_SERV_TRADE（常量） | 变异：字面量改为 `INTL_SERV_TRADE_Z` | 红=dataset_literals_match_the_manifest | 绿=dataset_literals_match_the_manifest
//! // TDD-PROBE: GDP_BY_INDUSTRY（常量） | 变异：字面量改为 `GDP_BY_INDUSTRY_Z` | 红=dataset_literals_match_the_manifest | 绿=dataset_literals_match_the_manifest
//! // TDD-PROBE: UNDERLYING_GDP_BY_INDUSTRY（常量） | 变异：字面量改为 `UNDERLYING_GDP_BY_INDUSTRY_Z` | 红=dataset_literals_match_the_manifest | 绿=dataset_literals_match_the_manifest
//! // TDD-PROBE: INPUT_OUTPUT（常量） | 变异：字面量改为 `INPUT_OUTPUT_Z` | 红=dataset_literals_match_the_manifest | 绿=dataset_literals_match_the_manifest
//! // TDD-PROBE: REGIONAL（常量） | 变异：字面量改为 `REGIONAL_Z` | 红=dataset_literals_match_the_manifest | 绿=dataset_literals_match_the_manifest
//! // TDD-PROBE: MNE（常量） | 变异：字面量改为 `MNE_Z` | 红=dataset_literals_match_the_manifest | 绿=dataset_literals_match_the_manifest
//! // TDD-PROBE: T10101（常量） | 变异：字面量改为 `T10101_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T20100（常量） | 变异：字面量改为 `T20100_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T10102（常量） | 变异：字面量改为 `T10102_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T10103（常量） | 变异：字面量改为 `T10103_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T10104（常量） | 变异：字面量改为 `T10104_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T10105（常量） | 变异：字面量改为 `T10105_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T10106（常量） | 变异：字面量改为 `T10106_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T10107（常量） | 变异：字面量改为 `T10107_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T10201（常量） | 变异：字面量改为 `T10201_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T20301（常量） | 变异：字面量改为 `T20301_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T30100（常量） | 变异：字面量改为 `T30100_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T40100（常量） | 变异：字面量改为 `T40100_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T50100（常量） | 变异：字面量改为 `T50100_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T70100（常量） | 变异：字面量改为 `T70100_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T20300（常量） | 变异：字面量改为 `T20300_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T20600（常量） | 变异：字面量改为 `T20600_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest
//! // TDD-PROBE: T11000（常量） | 变异：字面量改为 `T11000_Z` | 红=table_id_literals_match_the_manifest | 绿=table_id_literals_match_the_manifest

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
    PitEligibility, TimePrecision, CORE_NIPA_P0_TABLES, CORE_NIPA_P1_TABLES, CORE_PCE_ANCHOR_TABLE,
    CORE_PCE_PRIMARY_OWNER, CURVE_BUILD_OWNER, DATASETS, NEAR_SYNONYM_PAIRS, NIPA, T10101, T10102,
    T10103, T11000, T20100, T20300, T20301, T20600,
};

const FIXTURE: &str = include_str!("fixtures/bea_observations_synthetic.json");
const MISSING_FIXTURE: &str = include_str!("fixtures/bea_nipa_tables_synthetic.json");

fn observation() -> BeaObservation {
    BeaObservation {
        dataset_id: NIPA.to_owned(),
        table_id: T10101.to_owned(),
        line_number: 1,
        period: Period::Quarter {
            year: 2026,
            quarter: 2,
        },
        value: BeaValue::Present(1.0),
        unit: BeaUnit::new("Billions of dollars").expect("单位合法"),
        frequency: Frequency::Quarterly,
        vintage: None,
    }
}

fn record(table_id: &str, value: &str, frequency: &str) -> String {
    format!(
        r#"{{"records": [{{"dataset_id": "NIPA", "table_id": "{table_id}", "line_number": 1,
            "date": "2026-06-30", "value": {value}, "unit": "Index", "frequency": "{frequency}"}}]}}"#
    )
}

/// 11 个 Dataset 常量逐字等于清单登记值（源事实防漂）。
#[test]
fn dataset_literals_match_the_manifest() {
    let expected: &[(&str, &str)] = &[
        (beax::NIPA, "NIPA"),
        (beax::NI_UNDERLYING_DETAIL, "NIUnderlyingDetail"),
        (beax::FIXED_ASSETS, "FixedAssets"),
        (beax::ITA, "ITA"),
        (beax::IIP, "IIP"),
        (beax::INTL_SERV_TRADE, "IntlServTrade"),
        (beax::GDP_BY_INDUSTRY, "GDPbyIndustry"),
        (beax::UNDERLYING_GDP_BY_INDUSTRY, "UnderlyingGDPbyIndustry"),
        (beax::INPUT_OUTPUT, "InputOutput"),
        (beax::REGIONAL, "Regional"),
        (beax::MNE, "MNE"),
    ];
    assert_eq!(expected.len(), 11);
    for (actual, manifest) in expected {
        assert_eq!(*actual, *manifest);
    }
    for (dataset, _) in expected {
        assert!(DATASETS.contains(dataset), "全集缺 {dataset}");
    }
    assert_eq!(DATASETS.len(), expected.len());
}

/// 17 个表号常量逐字等于清单登记值（含非白名单登记项）。
#[test]
fn table_id_literals_match_the_manifest() {
    let expected: &[(&str, &str)] = &[
        (T10101, "T10101"),
        (T20100, "T20100"),
        (T10102, "T10102"),
        (T10103, "T10103"),
        (beax::T10104, "T10104"),
        (beax::T10105, "T10105"),
        (beax::T10106, "T10106"),
        (beax::T10107, "T10107"),
        (beax::T10201, "T10201"),
        (T20301, "T20301"),
        (beax::T30100, "T30100"),
        (beax::T40100, "T40100"),
        (beax::T50100, "T50100"),
        (beax::T70100, "T70100"),
        (T20300, "T20300"),
        (T20600, "T20600"),
        (T11000, "T11000"),
    ];
    assert_eq!(expected.len(), 17);
    for (actual, manifest) in expected {
        assert_eq!(*actual, *manifest);
    }
}

/// Dataset 范围与优先级。
#[test]
fn source_scope_predicates() {
    assert_eq!(DATASETS.len(), 11);
    assert!(is_known_dataset(NIPA));
    assert!(!is_known_dataset("BLS"));
    assert_eq!(dataset_priority(NIPA), Some(0));
    assert_eq!(dataset_priority(beax::ITA), Some(1));
    assert_eq!(dataset_priority("BLS"), None);
    for dataset in DATASETS {
        assert!(dataset_priority(dataset).is_some(), "{dataset}");
    }
}

/// NIPA 表白名单与优先级。
#[test]
fn table_whitelist_predicates() {
    assert_eq!(CORE_NIPA_P0_TABLES.len(), 2);
    assert_eq!(CORE_NIPA_P1_TABLES.len(), 12);
    assert!(is_core_p0_table(T10101));
    assert!(!is_core_p0_table(T10102));
    assert!(is_core_p1_table(T10102));
    assert!(!is_core_p1_table(T10101));
    assert!(is_known_nipa_table(T20100));
    assert!(!is_known_nipa_table(T20300));
    assert!(!is_known_nipa_table(T20600));
    assert!(!is_known_nipa_table(T11000));
    assert_eq!(table_priority(T10101), Some(0));
    assert_eq!(table_priority(T20301), Some(1));
    assert_eq!(table_priority(T20600), None);
}

/// 表级频率查询：只有两表有清单声明。
#[test]
fn table_frequency_lookup() {
    assert_eq!(table_frequency(T10101), Some(Frequency::Quarterly));
    assert_eq!(table_frequency(T20100), Some(Frequency::Monthly));
    assert_eq!(table_frequency(T10102), None);
}

/// 核心 PCE 补充口径锚点表。
#[test]
fn core_pce_anchor_constant() {
    assert_eq!(CORE_PCE_ANCHOR_TABLE, T20100);
    assert!(is_core_p0_table(CORE_PCE_ANCHOR_TABLE));
}

/// Date / validate_date：严格 ISO 形态、月日范围、闰年。
#[test]
fn date_construction_and_calendar_rules() {
    assert_eq!(
        Date::parse("2026-06-30").expect("应可解析"),
        Date::new(2026, 6, 30).expect("应可构造")
    );
    for bad in [
        "2026-6-30",
        "2026/06/30",
        "2026-02-30",
        "2026-13-01",
        "10000-01-01",
        "2026-06-30T00:00:00Z",
    ] {
        assert!(Date::parse(bad).is_err(), "应拒绝 {bad}");
    }
    assert!(Date::new(0, 1, 1).is_err());
    assert!(Date::is_leap_year(2024));
    assert!(!Date::is_leap_year(1900));
    assert_eq!(Date::days_in_month(2024, 2), 29);
    assert_eq!(Date::days_in_month(2026, 2), 28);
    assert_eq!(Date::days_in_month(2026, 13), 0);
    assert!(validate_date(&Date {
        year: 2026,
        month: 4,
        day: 31
    })
    .is_err());
}

/// Period / validate_period：五个变体的分量范围。
#[test]
fn period_validation_covers_every_variant() {
    let day = Date::new(2026, 6, 30).expect("日期合法");
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

/// Frequency：七值记号回环，且只接受小写。
#[test]
fn frequency_token_and_label_rules() {
    let all = [
        (Frequency::Daily, "daily"),
        (Frequency::Weekly, "weekly"),
        (Frequency::Monthly, "monthly"),
        (Frequency::Quarterly, "quarterly"),
        (Frequency::Annual, "annual"),
        (Frequency::Event, "event"),
        (Frequency::Irregular, "irregular"),
    ];
    for (frequency, token) in all {
        assert_eq!(frequency.as_str(), token);
        assert_eq!(Frequency::parse(token).expect("应可解析"), frequency);
    }
    assert!(Frequency::parse("Monthly").is_err());
    assert!(Frequency::parse("week").is_err());
}

/// BeaUnit：开放 newtype 的构造规则。
#[test]
fn unit_construction_rules() {
    assert_eq!(
        BeaUnit::new("Millions of dollars")
            .expect("应可构造")
            .as_str(),
        "Millions of dollars"
    );
    assert!(BeaUnit::new("").is_err());
    assert!(BeaUnit::new(" Index").is_err());
    assert!(BeaUnit::new("Index\n").is_err());
    // 内部（非首尾）控制字符也必须被拒绝：它不会被 trim 捕获。
    assert!(BeaUnit::new("Per\u{7}cent").is_err());
    assert!(BeaUnit::new("Per\u{0}cent").is_err());
}

/// BeaValue / BeaMissingReason：缺失具名且绝不折算为 0。
#[test]
fn value_missing_is_named_never_zero() {
    let missing = BeaValue::Missing(BeaMissingReason::NoObservation);
    assert!(missing.is_missing());
    assert_eq!(missing.as_f64(), None);
    assert_ne!(missing, BeaValue::Present(0.0));
    let present = BeaValue::Present(0.0);
    assert!(!present.is_missing());
    assert_eq!(present.as_f64(), Some(0.0));
}

/// BeaObservation / validate_observation：身份、期间、有限数、修订日期。
#[test]
fn observation_validation_rules() {
    let good = observation();
    assert!(validate_observation(&good).is_ok());

    let mut bad = good.clone();
    bad.dataset_id = " NIPA".to_owned();
    assert!(validate_observation(&bad).is_err());

    bad = good.clone();
    bad.table_id = String::new();
    assert!(validate_observation(&bad).is_err());

    bad = good.clone();
    bad.line_number = 0;
    assert!(validate_observation(&bad).is_err());

    bad = good.clone();
    bad.value = BeaValue::Present(f64::NAN);
    assert!(validate_observation(&bad).is_err());

    bad = good.clone();
    bad.period = Period::Year(0);
    assert!(validate_observation(&bad).is_err());

    bad = good.clone();
    bad.vintage = Some(Date {
        year: 2026,
        month: 2,
        day: 30,
    });
    assert!(validate_observation(&bad).is_err());
}

/// parse_bea_observations：合成夹具整体可解析。
#[test]
fn parse_accepts_synthetic_fixture() {
    let observations = parse_bea_observations(FIXTURE).expect("应可解析");
    assert_eq!(observations.len(), 7);
    assert!(observations.iter().all(|o| validate_observation(o).is_ok()));
    assert!(observations.iter().any(|o| o.dataset_id == NIPA));
}

/// 期间投影：月 / 季 / 年按清单频率落到对应 `Period`。
#[test]
fn parse_projects_period_by_frequency() {
    let input = format!(
        r#"{{"records": [{}, {}, {}]}}"#,
        r#"{"dataset_id": "NIPA", "table_id": "T20100", "line_number": 1, "date": "2026-06-30", "value": 1.0, "unit": "Index", "frequency": "monthly"}"#,
        r#"{"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-03-31", "value": 1.0, "unit": "Index", "frequency": "quarterly"}"#,
        r#"{"dataset_id": "Regional", "table_id": "SYN-REGIONAL-1", "line_number": 1, "date": "2026-06-30", "value": 1.0, "unit": "Index", "frequency": "annual"}"#
    );
    let observations = parse_bea_observations(&input).expect("应可解析");
    assert_eq!(
        observations[0].period,
        Period::Month {
            year: 2026,
            month: 6
        }
    );
    assert_eq!(
        observations[1].period,
        Period::Quarter {
            year: 2026,
            quarter: 1
        }
    );
    assert_eq!(observations[2].period, Period::Year(2026));
}

/// 修订标识：给出时保留，未给出时为 `None`（不伪造）。
#[test]
fn parse_preserves_vintage() {
    let observations = parse_bea_observations(FIXTURE).expect("应可解析");
    let vintage = observations
        .iter()
        .find(|o| o.table_id == T20100)
        .expect("夹具含 T20100");
    assert_eq!(
        vintage.vintage,
        Some(Date::new(2026, 7, 31).expect("日期合法"))
    );
    assert!(observations
        .iter()
        .filter(|o| o.table_id != T20100)
        .all(|o| o.vintage.is_none()));
}

/// 未知字段（记录层与顶层）原子失败。
#[test]
fn parse_rejects_unknown_field() {
    let with_extra = r#"{"records": [
        {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly", "endpoint": "x"}]}"#;
    assert!(parse_bea_observations(with_extra).is_err());
    assert!(parse_bea_observations(r#"{"records": [], "token": "x"}"#).is_err());
}

/// 缺必需字段原子失败。
#[test]
fn parse_rejects_missing_field() {
    let no_line = r#"{"records": [
        {"dataset_id": "NIPA", "table_id": "T10101", "date": "2026-06-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly"}]}"#;
    assert!(parse_bea_observations(no_line).is_err());
    assert!(parse_bea_observations(r#"{"items": []}"#).is_err());
}

/// 非法日期形态原子失败。
#[test]
fn parse_rejects_illegal_date() {
    for bad in [
        "2026-6-30",
        "2026/06/30",
        "2026-02-30",
        "2026-06-30T00:00:00Z",
    ] {
        let input = format!(
            r#"{{"records": [{{"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1,
                "date": "{bad}", "value": 1.0, "unit": "Index", "frequency": "quarterly"}}]}}"#
        );
        assert!(parse_bea_observations(&input).is_err(), "{bad}");
    }
}

/// 重复身份被拒绝（选择「拒绝」而非去重）。
#[test]
fn parse_rejects_duplicate_identity() {
    let input = r#"{"records": [
        {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly"},
        {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
         "value": 1.1, "unit": "Index", "frequency": "quarterly"}
    ]}"#;
    assert_eq!(
        parse_bea_observations(input).expect_err("应拒绝").kind(),
        BeaErrorKind::SemanticallyRejected
    );
}

/// `.` 与 null 两种缺失形态都映射为具名缺失。
#[test]
fn parse_keeps_missing_named() {
    let observations = parse_bea_observations(MISSING_FIXTURE).expect("应可解析");
    assert_eq!(observations.len(), 2);
    for record in &observations {
        assert_eq!(
            record.value,
            BeaValue::Missing(BeaMissingReason::NoObservation)
        );
        assert_eq!(record.value.as_f64(), None);
    }
    assert!(parse_bea_observations(&record(T10101, "0.0", "quarterly")).is_ok());
}

/// 清单范围外 Dataset 被拒绝。
#[test]
fn parse_rejects_out_of_scope_dataset() {
    assert!(ensure_known_dataset(NIPA).is_ok());
    assert!(ensure_known_dataset("BLS").is_err());
    let input = r#"{"records": [
        {"dataset_id": "BLS", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly"}]}"#;
    assert!(parse_bea_observations(input).is_err());
}

/// NIPA 白名单外表号被拒绝。
#[test]
fn parse_rejects_non_whitelisted_nipa_table() {
    assert!(ensure_nipa_table(T10101).is_ok());
    assert!(ensure_nipa_table(T20300).is_err());
    assert!(ensure_nipa_table(T20600).is_err());
    assert!(ensure_nipa_table(T11000).is_err());
    assert!(parse_bea_observations(&record(T20600, "1.0", "monthly")).is_err());
}

/// 表级频率漂移被拒绝。
#[test]
fn parse_rejects_table_frequency_drift() {
    let drifted = observation_from(T20100, Frequency::Quarterly);
    assert!(ensure_table_frequency(&drifted).is_err());
    let aligned = observation_from(T20100, Frequency::Monthly);
    assert!(ensure_table_frequency(&aligned).is_ok());
    assert!(parse_bea_observations(&record(T20100, "1.0", "quarterly")).is_err());
    assert!(parse_bea_observations(&record(T20100, "1.0", "monthly")).is_ok());
}

fn observation_from(table_id: &str, frequency: Frequency) -> BeaObservation {
    BeaObservation {
        dataset_id: NIPA.to_owned(),
        table_id: table_id.to_owned(),
        line_number: 1,
        period: Period::Month {
            year: 2026,
            month: 6,
        },
        value: BeaValue::Present(1.0),
        unit: BeaUnit::new("Index").expect("单位合法"),
        frequency,
        vintage: None,
    }
}

/// 值形态白名单：只接受数值、`.`、null。
#[test]
fn parse_rejects_non_numeric_value() {
    for bad in ["\"n/a\"", "true", "[1]", "{}"] {
        assert!(
            parse_bea_observations(&record(T10101, bad, "quarterly")).is_err(),
            "{bad}"
        );
    }
}

/// 曲线产品不在本域。
#[test]
fn curve_products_are_routed_not_local() {
    assert_eq!(CURVE_BUILD_OWNER, "yieldx");
    assert!(ensure_claim_local(BeaClaim::SupplementaryTableValue).is_ok());
    let err = ensure_claim_local(BeaClaim::YieldCurveConstruction).expect_err("应路由");
    assert_eq!(err.kind(), BeaErrorKind::RoutedElsewhere);
    assert!(err.to_string().contains(CURVE_BUILD_OWNER));
}

/// 核心 PCE 主责归 `fredx`，本域 MUST NOT 反向主张。
#[test]
fn core_pce_primary_is_never_claimed() {
    assert_eq!(CORE_PCE_PRIMARY_OWNER, "fredx");
    let err = ensure_claim_local(BeaClaim::CorePcePrimary).expect_err("应拒绝");
    assert_eq!(err.kind(), BeaErrorKind::WriteAuthorityDenied);
    assert!(err.to_string().contains(CORE_PCE_PRIMARY_OWNER));
}

/// 6 对近义非同 ID 双向禁止。
#[test]
fn near_synonym_pairs_are_enforced() {
    assert_eq!(NEAR_SYNONYM_PAIRS.len(), 6);
    for (left, right) in NEAR_SYNONYM_PAIRS {
        assert!(is_near_synonym_pair(left, right));
        assert!(is_near_synonym_pair(right, left));
        assert!(ensure_not_silent_substitution(left, right).is_err());
        assert!(ensure_not_silent_substitution(right, left).is_err());
    }
    assert!(ensure_not_silent_substitution(T10101, T20100).is_ok());
    assert!(ensure_not_silent_substitution(T20100, "PCEPILFE").is_err());
    assert!(ensure_not_silent_substitution("GDP", "GDI").is_err());
}

/// 锚点辅助函数。
#[test]
fn anchor_helpers() {
    assert!(is_gdp_or_personal_income_table(T10101));
    assert!(is_gdp_or_personal_income_table(T20100));
    assert!(!is_gdp_or_personal_income_table(T10102));
    assert_eq!(gdi_candidate_table(), T11000);
}

/// publication 三元组恒为 `(Date, Inferred, NotEligible)`。
#[test]
fn publication_semantics_triple() {
    let semantics: BeaPublicationSemantics = bea_publication_semantics();
    assert_eq!(semantics.time_precision, TimePrecision::Date);
    assert_eq!(semantics.availability, AvailabilityEvidence::Inferred);
    assert_eq!(semantics.eligibility, PitEligibility::NotEligible);
    assert_ne!(semantics.time_precision, TimePrecision::Instant);
    assert_ne!(semantics.eligibility, PitEligibility::Formal);
}

/// 正式 PIT 资格恒为 `false`。
#[test]
fn formal_pit_is_never_eligible() {
    assert!(!is_formal_pit_eligible());
    assert_eq!(
        bea_publication_semantics().eligibility,
        PitEligibility::NotEligible
    );
}

/// 已登记证据覆盖 offline / reference，不覆盖 live。
#[test]
fn authz_documented_evidence_offline_and_reference() {
    assert_eq!(beax::BEA_DECISION_ID, "BEA-PROD-2026-08-17-approve");
    assert_eq!(beax::BEA_SIGNED_BY, "ZoneCNH");
    let evidence = documented_bea_evidence();
    assert_eq!(evidence.decision_id, beax::BEA_DECISION_ID);
    assert_eq!(evidence.signed_by, beax::BEA_SIGNED_BY);
    assert_eq!(
        evidence.authorized_modes,
        vec![BeaAccessMode::Offline, BeaAccessMode::ReferenceOnly]
    );
    assert!(evidence.accept_no_pit);
    assert_eq!(evidence.live_official_pit, "NO-GO");

    let as_of = Date::new(2026, 8, 20).expect("日期合法");
    for mode in [BeaAccessMode::Offline, BeaAccessMode::ReferenceOnly] {
        match authorize_bea(Some(&evidence), mode, as_of) {
            BeaAuthorization::Authorized { scope } => {
                assert!(scope.contains(beax::BEA_DECISION_ID));
                assert!(scope.contains(mode_label(mode)));
            }
            other => panic!("应授权 {mode:?}，实得 {other:?}"),
        }
    }
}

/// live 不在覆盖范围内；源级例外登记值可用。
#[test]
fn authz_live_is_not_covered() {
    assert_eq!(beax::LIVE_OFFICIAL_PIT_DECISION, "NO-GO");
    assert_eq!(live_official_pit_decision(), "NO-GO");
    assert!(accepts_no_official_pit());
    assert!(documented_bea_evidence().accept_no_pit);
    let as_of = Date::new(2026, 8, 20).expect("日期合法");
    match authorize_bea(Some(&documented_bea_evidence()), BeaAccessMode::Live, as_of) {
        BeaAuthorization::Denied { reason } => assert!(reason.contains("live")),
        other => panic!("应拒绝 live，实得 {other:?}"),
    }
}

/// fail-closed：缺失 / 编号不明 / 签署者不明 / 范围不明 / 过期。
#[test]
fn authz_fail_closed_paths() {
    let as_of = Date::new(2026, 8, 20).expect("日期合法");
    assert!(matches!(
        authorize_bea(None, BeaAccessMode::Offline, as_of),
        BeaAuthorization::Denied { .. }
    ));

    let mut evidence = documented_bea_evidence();
    evidence.decision_id = String::new();
    assert!(matches!(
        authorize_bea(Some(&evidence), BeaAccessMode::Offline, as_of),
        BeaAuthorization::Denied { .. }
    ));

    let mut evidence = documented_bea_evidence();
    evidence.signed_by = String::new();
    assert!(matches!(
        authorize_bea(Some(&evidence), BeaAccessMode::Offline, as_of),
        BeaAuthorization::Denied { .. }
    ));

    let mut evidence = documented_bea_evidence();
    evidence.authorized_modes.clear();
    assert!(matches!(
        authorize_bea(Some(&evidence), BeaAccessMode::Offline, as_of),
        BeaAuthorization::Denied { .. }
    ));

    let mut evidence = documented_bea_evidence();
    evidence.valid_until = Date::new(2026, 8, 18).ok();
    match authorize_bea(Some(&evidence), BeaAccessMode::Offline, as_of) {
        BeaAuthorization::Denied { reason } => assert_eq!(reason, "证据已过期"),
        other => panic!("应拒绝过期证据，实得 {other:?}"),
    }
}

/// 证据描述的日期分量校验。
#[test]
fn authz_evidence_date_validation() {
    let valid: BeaAuthorizationEvidence = documented_bea_evidence();
    assert!(validate_authorization_evidence(&valid).is_ok());

    let mut invalid = documented_bea_evidence();
    invalid.signed_at = Some(Date {
        year: 2026,
        month: 13,
        day: 1,
    });
    assert!(validate_authorization_evidence(&invalid).is_err());

    let mut invalid = documented_bea_evidence();
    invalid.valid_until = Some(Date {
        year: 2026,
        month: 2,
        day: 30,
    });
    assert!(validate_authorization_evidence(&invalid).is_err());
}

/// 访问模式记号稳定。
#[test]
fn mode_labels_are_stable() {
    assert_eq!(mode_label(BeaAccessMode::Offline), "offline");
    assert_eq!(mode_label(BeaAccessMode::ReferenceOnly), "reference_only");
    assert_eq!(mode_label(BeaAccessMode::Live), "live");
}

/// 错误分类与重试判定。
#[test]
fn error_kind_mapping_and_retryable() {
    let cases = [
        (BeaError::Invalid("x".into()), BeaErrorKind::Invalid),
        (BeaError::Missing("x".into()), BeaErrorKind::Missing),
        (
            BeaError::AuthorizationDenied("x".into()),
            BeaErrorKind::AuthorizationDenied,
        ),
        (
            BeaError::RoutedElsewhere("x".into()),
            BeaErrorKind::RoutedElsewhere,
        ),
        (
            BeaError::WriteAuthorityDenied("x".into()),
            BeaErrorKind::WriteAuthorityDenied,
        ),
        (
            BeaError::SemanticallyRejected("x".into()),
            BeaErrorKind::SemanticallyRejected,
        ),
        (
            BeaError::NotApplicable("x".into()),
            BeaErrorKind::NotApplicable,
        ),
        (BeaError::Invariant("x".into()), BeaErrorKind::Invariant),
    ];
    for (err, kind) in cases {
        assert_eq!(err.kind(), kind);
        assert_eq!(err.is_retryable(), kind == BeaErrorKind::Invariant);
    }
    let result: BeaResult<()> = ensure_known_dataset(NIPA);
    assert!(result.is_ok());
    let failed: BeaResult<()> = ensure_known_dataset("BLS");
    assert_eq!(
        failed.expect_err("范围外").kind(),
        BeaErrorKind::SemanticallyRejected
    );
}

/// 错误消息为中文，且不回声输入内容。
#[test]
fn error_display_is_chinese_and_does_not_echo_input() {
    let err = parse_bea_observations("{ oops").expect_err("非法 JSON");
    let shown = err.to_string();
    assert!(shown.contains("输入非法"));
    assert!(!shown.contains("oops"));

    let err = BeaError::RoutedElsewhere("曲线构建归 yieldx".into());
    assert!(err.to_string().contains("已路由他处"));
    assert!(!err.to_string().contains("http"));
}
