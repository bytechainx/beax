# 变更记录

本文件记录 `beax` 的可见变更。格式对齐 Keep a Changelog，
版本号遵循 `docs/versioning.md`；本仓库是库 crate，`Cargo.lock` 不入库。

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
