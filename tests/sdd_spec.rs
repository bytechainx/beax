#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! SDD 规格对照（特性 005）：把 `docs/标准.md` 的每个 `##` 章节转成可执行断言。
//!
//! 章节与断言函数须与 `docs/标准.md` 的 `##` 章节 1:1（检查器按标题逐字比对）。
//!
//! // SPEC-MAP: S-1 | 1. 源事实与采集范围标准 | assert_source_scope
//! // SPEC-MAP: S-2 | 2. 值与期间标准 | assert_value_and_period
//! // SPEC-MAP: S-3 | 3. 表级白名单与优先级标准 | assert_table_whitelist_and_priority
//! // SPEC-MAP: S-4 | 4. 跨源守卫标准（曲线路由 / 核心 PCE 主责 / 禁静默替换） | assert_cross_source_guards
//! // SPEC-MAP: S-5 | 5. publication 语义标准 | assert_publication_semantics
//! // SPEC-MAP: S-6 | 6. 授权判定标准（fail-closed，live 不被覆盖） | assert_authorization_fail_closed
//! // SPEC-MAP: S-7 | 7. 离线解析标准 | assert_offline_parsing
//! // SPEC-MAP: S-8 | 8. 合成夹具声明 | assert_synthetic_fixtures_declared
//! // SPEC-MAP: S-9 | 9. 依赖与零网络标准 | assert_no_network_and_no_coupling
//! // SPEC-MAP: S-10 | 10. 端点与凭据不入库标准 | assert_endpoints_and_credentials_not_embedded
//! // SPEC-MAP: S-11 | 11. 验收 | assert_acceptance_surface

use beax::{
    authorize_bea, dataset_priority, documented_bea_evidence, ensure_claim_local,
    ensure_known_dataset, ensure_nipa_table, ensure_not_silent_substitution,
    ensure_table_frequency, is_core_p0_table, is_core_p1_table, is_formal_pit_eligible,
    is_known_dataset, is_known_nipa_table, parse_bea_observations, table_frequency, table_priority,
    validate_date, validate_observation, validate_period, BeaAccessMode, BeaAuthorization,
    BeaClaim, BeaErrorKind, BeaMissingReason, BeaObservation, BeaUnit, BeaValue, Date, Frequency,
    Period, CORE_NIPA_P0_TABLES, CORE_NIPA_P1_TABLES, DATASETS, NIPA, T10101, T11000, T20100,
    T20300, T20600,
};

const FIXTURE: &str = include_str!("fixtures/bea_observations_synthetic.json");

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

/// 运行期**递归枚举** `<root>` 下全部 `*.rs` 文件。
///
/// 刻意不用手写文件清单、也不用 `include_str!` 逐个点名：清单会漏掉既有文件，
/// 且新文件永远进不了扫描面（这正是本断言要消灭的盲区）。
fn collect_src_rs(root: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_src_rs(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// S-1：11 个 Dataset、NIPA 为 P0、范围外 Dataset 被拒。
#[test]
fn assert_source_scope() {
    assert_eq!(DATASETS.len(), 11);
    assert_eq!(dataset_priority(NIPA), Some(0));
    for dataset in DATASETS {
        assert!(is_known_dataset(dataset));
        assert!(dataset_priority(dataset).is_some());
    }
    assert_eq!(dataset_priority("ITA"), Some(1));
    assert!(!is_known_dataset("BLS"));
    assert!(ensure_known_dataset(NIPA).is_ok());
    assert!(ensure_known_dataset("BLS").is_err());
}

/// S-2：期间与值形态；缺失具名、非有限数被拒。
#[test]
fn assert_value_and_period() {
    assert!(validate_date(&Date::new(2026, 6, 30).expect("日期合法")).is_ok());
    assert!(validate_period(&Period::Quarter {
        year: 2026,
        quarter: 5
    })
    .is_err());
    assert_eq!(
        Frequency::parse("monthly").expect("记号"),
        Frequency::Monthly
    );
    assert!(Frequency::parse("Monthly").is_err());

    let mut record = observation();
    assert!(validate_observation(&record).is_ok());
    record.value = BeaValue::Missing(BeaMissingReason::NoObservation);
    assert!(validate_observation(&record).is_ok());
    record.value = BeaValue::Present(f64::NAN);
    assert!(validate_observation(&record).is_err());

    record = observation();
    record.line_number = 0;
    assert!(validate_observation(&record).is_err());
}

/// S-3：NIPA 14 表白名单、优先级、表级频率。
#[test]
fn assert_table_whitelist_and_priority() {
    assert_eq!(CORE_NIPA_P0_TABLES.len(), 2);
    assert_eq!(CORE_NIPA_P1_TABLES.len(), 12);
    assert_eq!(CORE_NIPA_P0_TABLES.len() + CORE_NIPA_P1_TABLES.len(), 14);
    assert!(is_core_p0_table(T10101));
    assert!(is_core_p1_table("T10102"));
    assert!(is_known_nipa_table(T20100));
    assert!(!is_known_nipa_table(T20600));
    assert_eq!(table_priority(T10101), Some(0));
    assert_eq!(table_priority("T70100"), Some(1));
    assert_eq!(table_priority(T20300), None);
    assert!(ensure_nipa_table(T20100).is_ok());
    assert!(ensure_nipa_table(T20600).is_err());

    assert_eq!(table_frequency(T10101), Some(Frequency::Quarterly));
    assert_eq!(table_frequency(T20100), Some(Frequency::Monthly));
    assert_eq!(table_frequency("T10102"), None);

    let mut drifted = observation();
    drifted.frequency = Frequency::Monthly;
    assert_eq!(
        ensure_table_frequency(&drifted).expect_err("应拒绝").kind(),
        BeaErrorKind::SemanticallyRejected
    );
    let mut aligned = observation();
    aligned.table_id = T20100.to_owned();
    aligned.frequency = Frequency::Monthly;
    assert!(ensure_table_frequency(&aligned).is_ok());
}

/// S-4：曲线路由、核心 PCE 主责、6 对近义非同 ID。
#[test]
fn assert_cross_source_guards() {
    assert!(ensure_claim_local(BeaClaim::SupplementaryTableValue).is_ok());
    assert_eq!(
        ensure_claim_local(BeaClaim::CorePcePrimary)
            .expect_err("主责归 fredx")
            .kind(),
        BeaErrorKind::WriteAuthorityDenied
    );
    assert_eq!(
        ensure_claim_local(BeaClaim::YieldCurveConstruction)
            .expect_err("曲线归 yieldx")
            .kind(),
        BeaErrorKind::RoutedElsewhere
    );

    assert_eq!(beax::NEAR_SYNONYM_PAIRS.len(), 6);
    for (left, right) in beax::NEAR_SYNONYM_PAIRS {
        assert!(ensure_not_silent_substitution(left, right).is_err());
        assert!(ensure_not_silent_substitution(right, left).is_err());
    }
    assert!(ensure_not_silent_substitution(T10101, T20100).is_ok());
    assert_eq!(beax::gdi_candidate_table(), T11000);
}

/// S-5：publication 三元组恒为 `(Date, Inferred, NotEligible)`；正式 PIT 恒 `false`。
#[test]
fn assert_publication_semantics() {
    let semantics = beax::bea_publication_semantics();
    assert_eq!(semantics.time_precision, beax::TimePrecision::Date);
    assert_eq!(semantics.availability, beax::AvailabilityEvidence::Inferred);
    assert_eq!(semantics.eligibility, beax::PitEligibility::NotEligible);
    assert!(!is_formal_pit_eligible());
}

/// S-6：offline / reference 被覆盖、live 不被覆盖；缺失/不明/过期一律拒绝。
#[test]
fn assert_authorization_fail_closed() {
    let as_of = Date::new(2026, 8, 20).expect("日期合法");
    let evidence = documented_bea_evidence();
    assert_eq!(evidence.live_official_pit, "NO-GO");
    assert!(evidence.accept_no_pit);
    assert!(beax::accepts_no_official_pit());
    assert_eq!(beax::live_official_pit_decision(), "NO-GO");

    for mode in [BeaAccessMode::Offline, BeaAccessMode::ReferenceOnly] {
        assert!(matches!(
            authorize_bea(Some(&evidence), mode, as_of),
            BeaAuthorization::Authorized { .. }
        ));
    }
    assert!(matches!(
        authorize_bea(Some(&evidence), BeaAccessMode::Live, as_of),
        BeaAuthorization::Denied { .. }
    ));
    assert!(matches!(
        authorize_bea(None, BeaAccessMode::Offline, as_of),
        BeaAuthorization::Denied { .. }
    ));

    let mut unknown_scope = documented_bea_evidence();
    unknown_scope.authorized_modes.clear();
    assert!(matches!(
        authorize_bea(Some(&unknown_scope), BeaAccessMode::Offline, as_of),
        BeaAuthorization::Denied { .. }
    ));

    let mut expired = documented_bea_evidence();
    expired.valid_until = Date::new(2026, 8, 16).ok();
    assert!(matches!(
        authorize_bea(Some(&expired), BeaAccessMode::Offline, as_of),
        BeaAuthorization::Denied { .. }
    ));
}

/// S-7：未知字段原子失败、重复身份拒绝、缺失具名、错误不回声输入。
#[test]
fn assert_offline_parsing() {
    let observations = parse_bea_observations(FIXTURE).expect("合成夹具应可解析");
    assert_eq!(observations.len(), 7);
    assert!(observations.iter().all(|o| validate_observation(o).is_ok()));

    let unknown_field = r#"{"records": [
        {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly", "extra": 1}]}"#;
    assert!(parse_bea_observations(unknown_field).is_err());

    let missing_field = r#"{"records": [
        {"dataset_id": "NIPA", "table_id": "T10101", "date": "2026-06-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly"}]}"#;
    assert!(parse_bea_observations(missing_field).is_err());

    let duplicate = r#"{"records": [
        {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
         "value": 1.0, "unit": "Index", "frequency": "quarterly"},
        {"dataset_id": "NIPA", "table_id": "T10101", "line_number": 1, "date": "2026-06-30",
         "value": 1.1, "unit": "Index", "frequency": "quarterly"}]}"#;
    assert_eq!(
        parse_bea_observations(duplicate)
            .expect_err("重复身份")
            .kind(),
        BeaErrorKind::SemanticallyRejected
    );

    let bad = parse_bea_observations("{ oops").expect_err("非法 JSON");
    assert!(!bad.to_string().contains("oops"));
}

/// S-8：夹具自带合成标注，且不被表述为证据。
#[test]
fn assert_synthetic_fixtures_declared() {
    assert!(FIXTURE.contains("\"_synthetic\": true"));
    assert!(FIXTURE.contains("不是真实源数据，不构成任何证据"));
    assert!(FIXTURE.contains("SYN-"), "非 NIPA 表号须为合成占位串");
    let missing_fixture = include_str!("fixtures/bea_nipa_tables_synthetic.json");
    assert!(missing_fixture.contains("\"_synthetic\": true"));
    let observations = parse_bea_observations(missing_fixture).expect("应可解析");
    assert_eq!(observations.len(), 2);
    for record in &observations {
        assert!(record.value.is_missing());
        assert_eq!(record.value.as_f64(), None);
    }
}

/// S-9：本仓 `src/` 下**每一个** `.rs` 文件均不含 URL 端点字面量与凭据读取入口；依赖为精确白名单。
#[test]
fn assert_no_network_and_no_coupling() {
    // ① 依赖集合必须是精确白名单：既挡住禁用依赖，也不接受任何未声明的依赖。
    let manifest = include_str!("../Cargo.toml");
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .expect("Cargo.toml 须含 [dependencies]")
        .split("\n[")
        .next()
        .expect("依赖段存在");
    let mut names: Vec<&str> = deps
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .filter_map(|line| line.split('=').next())
        .map(str::trim)
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["serde", "serde_json", "thiserror"]);
    assert!(!manifest.contains("path = \"../"));

    // ② 运行期递归枚举 `src/` 下全部 `.rs`：以 CARGO_MANIFEST_DIR 为根，**不依赖 cwd**、
    //    不用手写清单，因此新增模块文件（哪怕未接入模块树）也自动进入扫描面。
    let src_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_src_rs(&src_root, &mut files);
    files.sort();
    assert!(
        files.iter().any(|path| path.ends_with("lib.rs")),
        "枚举面须覆盖 src/lib.rs"
    );

    // ③ 覆盖完备性自证：`lib.rs` 里声明的每个 `pub mod <name>;` 都必须出现在枚举面内，
    //    否则「新增了模块文件」可能悄悄逃出扫描。
    let lib_text = std::fs::read_to_string(src_root.join("lib.rs")).expect("读取 src/lib.rs");
    let mut declared: Vec<String> = Vec::new();
    for line in lib_text.lines() {
        if let Some(rest) = line.trim().strip_prefix("pub mod ") {
            if let Some(name) = rest.strip_suffix(';') {
                declared.push(name.to_owned());
            }
        }
    }
    assert!(!declared.is_empty(), "lib.rs 须声明模块");
    for name in &declared {
        let expected = src_root.join(format!("{name}.rs"));
        assert!(
            files.contains(&expected),
            "枚举面漏掉 lib.rs 声明的模块文件：src/{name}.rs"
        );
    }

    // ④ 逐个文件断言：本仓 `src/` 下每一个 `.rs` 文件均不含 http/https 端点字面量，
    //    也不含环境变量 / 凭据读取入口。
    for path in &files {
        let text = std::fs::read_to_string(path).expect("读取 src 下 .rs 文件");
        for needle in ["http://", "https://"] {
            assert!(
                !text.contains(needle),
                "{} 含 URL 端点字面量 `{needle}`",
                path.display()
            );
        }
        for needle in ["env::var", "from_env"] {
            assert!(
                !text.contains(needle),
                "{} 含凭据读取入口 `{needle}`",
                path.display()
            );
        }
    }
}

/// S-10：`src/` 下**每一个** `.rs`、以及 `CONTEXT.md` / `AGENTS.md` 与合成夹具，均不含
/// URL scheme、环境变量读取入口与端点查询串。
///
/// 说明：`docs/标准.md` 本身要在正文里引用 `https://…` 来表达禁令，故文档面**不做整目录扫描**，
/// 只点名这两份不含 URL 的受管文档；`src/` 一律走运行期递归枚举，不用手写清单。
#[test]
fn assert_endpoints_and_credentials_not_embedded() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut surfaces: Vec<std::path::PathBuf> = Vec::new();
    collect_src_rs(&manifest_dir.join("src"), &mut surfaces);
    assert!(
        surfaces.len() >= 2,
        "枚举面须覆盖 src/ 下的全部 .rs 文件（实测 {} 个）",
        surfaces.len()
    );
    surfaces.push(manifest_dir.join("CONTEXT.md"));
    surfaces.push(manifest_dir.join("AGENTS.md"));

    for path in &surfaces {
        let text = std::fs::read_to_string(path).expect("读取受管文件");
        for needle in ["http://", "https://"] {
            assert!(
                !text.contains(needle),
                "{} 含 URL scheme `{needle}`",
                path.display()
            );
        }
        for needle in ["env::var", "from_env"] {
            assert!(
                !text.contains(needle),
                "{} 含环境变量读取入口 `{needle}`",
                path.display()
            );
        }
        // 查询串形态（`?param=`）是端点编址的典型痕迹，本层不得出现。
        assert!(
            !text.contains("?datasetname="),
            "{} 含端点查询串",
            path.display()
        );
    }
    // 夹具只登记数据字段，不登记任何访问参数。
    for forbidden_field in ["\"url\"", "\"endpoint\"", "\"header\"", "\"credential\""] {
        assert!(
            !FIXTURE.contains(forbidden_field),
            "夹具出现访问参数字段 {forbidden_field}"
        );
    }
}

/// S-11：验收命令与门禁面存在。
#[test]
fn assert_acceptance_surface() {
    let contributing = include_str!("../CONTRIBUTING.md");
    for gate in [
        "cargo fmt --all -- --check",
        "cargo clippy --all-targets --all-features -- -D warnings",
        "cargo test --all-features",
        "cargo package --no-verify",
    ] {
        assert!(contributing.contains(gate), "缺门禁命令：{gate}");
    }
    let readme = include_str!("../README.md");
    assert!(readme.contains("production_decision = NO-GO"));
    assert!(readme.contains("## 非目标"));
    assert!(readme.contains("## 门禁"));
}
