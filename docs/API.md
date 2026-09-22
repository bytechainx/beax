# beax 公开 API

本文对应 `beax 0.1.1` 的离线公开消费面。全部入口**不触网**、不读凭据、不做单位换算。

## 值对象与校验

| 类型 / 入口 | 语义 |
| --- | --- |
| `Date` | 严格 ISO 日期 `YYYY-MM-DD`（`year` / `month` / `day` 公开字段） |
| `Date::new` / `Date::parse` | 构造并校验 / 按严格 ISO 解析（拒绝未补零、非 `-` 分隔、带时间部分者） |
| `Date::is_leap_year` / `Date::days_in_month` | 格里高利闰年判定 / 该年该月天数（月份非法返回 `0`，不 panic） |
| `Period` | 业务期间：`Day` / `Month` / `Quarter` / `Year` / `Event` |
| `Frequency` | 源侧频率七值；`Frequency::parse` / `Frequency::as_str` 为稳定记号 |
| `BeaUnit` | 源侧单位（开放 newtype）；`BeaUnit::new` / `BeaUnit::as_str` |
| `BeaMissingReason` | 具名缺失原因（`NoObservation`） |
| `BeaValue` | `Present(f64)` / `Missing(BeaMissingReason)`；`as_f64` / `is_missing` |
| `BeaObservation` | 一条观测：`dataset_id` + `table_id` + `line_number` + `period` + `value` + `unit` + `frequency` + `vintage` |
| `validate_date` / `validate_period` / `validate_observation` | 值对象完整性校验，返回 `BeaResult<()>` |

## 源事实常量

| 入口 | 语义 |
| --- | --- |
| 11 个 Dataset 常量 | `NIPA`（P0）与其余 10 个 P1 Dataset |
| 17 个表号常量 | NIPA 白名单 14 表 + `T20300`/`T20600`（非白名单）+ `T11000`（GDI 候选） |
| `DATASETS` | Dataset 全集（11 个） |
| `CORE_NIPA_P0_TABLES` / `CORE_NIPA_P1_TABLES` | 表级白名单 P0（2）/ P1（12） |
| `CORE_PCE_ANCHOR_TABLE` | 核心 PCE 补充口径锚点表（`T20100`） |
| `is_known_dataset` / `dataset_priority` | Dataset 范围与优先级（NIPA=0，其余=1） |
| `is_core_p0_table` / `is_core_p1_table` / `is_known_nipa_table` / `table_priority` | 表级范围与优先级 |
| `table_frequency` | 清单声明的表级频率（未声明返回 `None`） |

## 校验与守卫

| 入口 | 语义 |
| --- | --- |
| `ensure_known_dataset` | 拒绝清单范围外的 Dataset |
| `ensure_nipa_table` | 拒绝 NIPA 白名单外的表号 |
| `ensure_table_frequency` | 表级频率一致性 |
| `BeaClaim` / `ensure_claim_local` | 身份主张守卫：补充观测放行；核心 PCE 主责与曲线构建一律拒绝 |
| `CORE_PCE_PRIMARY_OWNER` / `CURVE_BUILD_OWNER` | 核心 PCE 主责归 `fredx` / 曲线构建归 `yieldx` |
| `NEAR_SYNONYM_PAIRS` / `is_near_synonym_pair` | 涉 BEA 的 6 对近义非同 ID |
| `ensure_not_silent_substitution` | 双向禁静默替换 |
| `is_gdp_or_personal_income_table` / `gdi_candidate_table` | 锚点表判定 / GDI 候选表号 |

## publication 语义

| 入口 | 语义 |
| --- | --- |
| `TimePrecision` / `AvailabilityEvidence` / `PitEligibility` | 三元组枚举 |
| `BeaPublicationSemantics` / `bea_publication_semantics` | 恒为 `(Date, Inferred, NotEligible)` |
| `is_formal_pit_eligible` | 恒为 `false` |

## 授权判定

| 入口 | 语义 |
| --- | --- |
| `BeaAccessMode` | `Offline` / `ReferenceOnly` / `Live` |
| `BeaAuthorization` | `Authorized { scope }` / `Denied { reason }` |
| `BeaAuthorizationEvidence` | 只读证据登记（编号 / 签署者 / 日期 / 覆盖模式 / `accept_no_pit` / `live_official_pit` / 范围说明） |
| `documented_bea_evidence` | 已登记证据 `BEA-PROD-2026-08-17-approve`（覆盖 offline 与 reference，**不含 live**） |
| `authorize_bea` | fail-closed 判定（缺失 / 不明 / 过期 / 未覆盖一律拒绝） |
| `validate_authorization_evidence` | 证据日期分量校验 |
| `mode_label` | 访问模式的稳定记号 |
| `BEA_DECISION_ID` / `BEA_SIGNED_BY` / `LIVE_OFFICIAL_PIT_DECISION` | 签核编号 / 签署者 / live 官方 PIT 裁定（`NO-GO`） |
| `accepts_no_official_pit` / `live_official_pit_decision` | 源级例外登记值 |

## 离线解析

| 入口 | 语义 |
| --- | --- |
| `parse_bea_observations` | 自有 JSON 形态 → `Vec<BeaObservation>`；未知字段原子失败、重复身份拒绝 |

## 错误面

| 入口 | 语义 |
| --- | --- |
| `BeaErrorKind` | 八类语义分类（`Invalid` / `Missing` / `AuthorizationDenied` / `RoutedElsewhere` / `WriteAuthorityDenied` / `SemanticallyRejected` / `NotApplicable` / `Invariant`） |
| `BeaError` / `BeaError::kind` / `BeaError::is_retryable` | 错误类型与分类/重试判定（仅 `Invariant` 具重试标记） |
| `BeaResult<T>` | 统一结果别名 |
