# 变更记录

本文件记录 `beax` 的可见变更。格式对齐 Keep a Changelog，
版本号遵循 `docs/versioning.md`；本仓库是库 crate，`Cargo.lock` 不入库。

## [Unreleased]

### 新增

- 新增 E2E target `tests/e2e_bea.rs`：把本仓**核对器口径内的全部公开条目**逐条真实执行 ——
  权威公开面派生自 `cargo +nightly public-api --simplified`，核对器（带 `llvm-cov`）退出码 **0**，
  权威 **163** / 声明 **163** / `公开 fn 执行 39/39`（分项 `type` 18 / `variant` 46 / `field` 22 /
  `const` 38 / `fn` 39）；单一 `#[test] e2e_bea_all_public_api`（8 个 phase），`[dev-dependencies]`
  仍为空、无网络 / 无凭据 / 无文件副作用（解析阶段读仓内真实合成夹具）。
  **纯测试新增，不改公开 API、不升版本**；口径边界（7 个两级嵌套字段未登记 ⇒ 三层判据不保护、
  derive/auto impl 不计入、`BeaUnit(_)` 字段私有故无 `_0` 条目等）与核对命令见
  `AGENTS.md`「E2E 全公开面覆盖核对」。

## [0.1.1] - 2026-09-23

### 修正

- 修正授权元数据与日期有效区间校验，恢复 fail-closed 契约

## [0.1.0] - 2026-09-22

### 新增

- 源事实常量：11 个 Dataset（`NIPA` 为 P0、其余 10 个 keep-P1）与 Dataset 优先级；
  NIPA 表号白名单（P0 2 表 + P1 12 表 = 14 表）与表级优先级；
  非白名单登记 `T20300` / `T20600` 与 GDI 候选 `T11000`。
- 值对象与校验：`Date`（严格 ISO + 闰年）、`Period`、`Frequency`、`BeaUnit`、`BeaValue`
  （具名缺失）、`BeaObservation`（身份含 `dataset_id` + `table_id` + `line_number`），
  以及 `validate_date` / `validate_period` / `validate_observation`。
- 跨源守卫（本域自己那一侧）：曲线产品路由 `yieldx`、核心 PCE 主责归 `fredx`
  （MUST NOT 反向主张）、6 对近义非同 ID 双向禁静默替换、NIPA 表白名单与表级频率一致性。
- publication 语义三元组：恒为 `(Date, Inferred, NotEligible)`；`is_formal_pit_eligible()` 恒为 `false`。
- fail-closed 授权判定：`BeaAuthorization` / `authorize_bea`；`BEA-PROD-2026-08-17-approve`
  覆盖 offline 与 reference 两个范围，**live 不被覆盖**（`live_official_pit = NO-GO`）。
- 离线解析：`parse_bea_observations`（自有 JSON 形态，未知字段原子失败、重复身份拒绝）。
- 三类测试（`tests/tdd_contracts.rs` / `tests/sdd_spec.rs` / `tests/aidd_boundary.rs`）、
  合成夹具、手写微基准与文档面（README / `docs/API.md` / `docs/标准.md` / `CONTEXT.md`）。

### 边界

- `production_decision = NO-GO`：本版本不含联网采集、live、官方 PIT、单位换算或派生指标；
  端点 URL 与凭据字段名不入库。
