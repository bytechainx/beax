# beax 上下文

本文件定义 `beax` 与其使用方共享的词汇与边界。它记录领域含义与能力边界，
不记录具体实现细节、部署决定、端点编址或凭据策略。

## 角色与边界

**源事实库**：把 `specs/adapter/bea.md`（source_id = `bea`）声明的 BEA 采集范围
落成类型化常量与守卫的离线库。
_Avoid_: 采集器 / 抓取器（本库不含任何网络能力）

**离线解析**：只吃字符串的解析入口，把自有 JSON 形态转成观测集合。
_Avoid_: API 客户端（本库不接受 URL、认证信息或任何网络参数）

**守卫**：跨源约束在本库内**自己那一侧**的落点（曲线路由、核心 PCE 主责、禁静默替换）。
跨源整体语义归 `specs/features/005-macro-data-source-crates/contracts/cross-source-routing.md`。
_Avoid_: 权威裁定者（本库 MUST NOT 重新裁定任何未决项）

**共享形状**：错误面、值对象与 publication 三元组在各数据源库中**各自实现一遍**，
以文档 `contracts/source-library-contract.md` 冻结形状。
_Avoid_: 公共 core crate（宪章原则 III 禁止第二个共享契约 crate）

## 源事实语义

**Dataset**：BEA 的顶层数据集（`NIPA` / `ITA` / `Regional` …）。`NIPA` 是清单三指标的
native 承载，为 P0；其余 10 个为 keep-P1。
_Avoid_: 表（Dataset 是集合，表是集合内的成员）

**表号**（`table_id`）：BEA 原始表号（如 `T10101`），**保留原样**，MUST NOT 被直接当作指标 ID。
NIPA 表号白名单 = P0 2 表 + P1 12 表 = 14 表。
_Avoid_: 指标 ID（表号是承载容器，指标由「表 + 行」共同定位）

**行号**（`line_number`）：表内行定位；观测身份 = `dataset_id` + `table_id` +
`line_number` + `period` + `vintage`。
_Avoid_: 序列码（本层不引入清单未登记的行/序列编码体系）

**表级频率**：清单只对 `T10101`（季）与 `T20100`（月）声明了频率，本库只对这两表做一致性判定。
_Avoid_: 推定频率（清单未声明的表 MUST NOT 由实现方推定）

**源单位保留**：观测携带源侧单位字面量，本层**不做换算**。清单未逐表声明源单位，
故 `BeaUnit` 是开放 newtype，避免编造单位取值。
_Avoid_: 规范化单位（Canonical 转换归下游 Normalize）

**具名缺失**：缺失以 `BeaMissingReason` 表达，MUST NOT 静默折算为 `0`。
_Avoid_: 零值（0 是一个合法观测，与缺失不是同一件事）

**推断层 publication**：BEA 数据有日期无时刻，且无官方 vintage 面
（源级例外 `PIT-EXCEPT-BEA-001`），故 publication 时刻属推断层，正式 PIT 资格为 `NotEligible`。
_Avoid_: 正式 PIT（live 官方 PIT 恒 `NO-GO`）

## 跨源语义

**曲线产品**：BEA 曲线类产品 MUST NOT 自动进入本域，须路由 `yieldx`。
_Avoid_: 曲线权威（本域不是曲线 owner）

**核心 PCE 主责**（fred-forward）：`PCEPILFE` 的采集主责是 `fredx`；本域 `T20100` 内的
PCE 行只是**补充口径**，MUST NOT 反向主张、MUST NOT 静默替换身份。
_Avoid_: 原生替代转发（原生 vs 转发的最终身份已裁为 `fredx` 侧）

## 授权语义

**授权判定**：一次 fail-closed 的只读结论。它**不改写**
`specs/adapter/bea.owner-approve.json` 的登记值，也不表示本库生产就绪。
_Avoid_: 生产就绪（`production_decision` 恒为 `NO-GO`）

**已登记证据**：`BEA-PROD-2026-08-17-approve`（签署者 `ZoneCNH`），
`scope = offline_and_reference_only`：覆盖 offline 与 reference，**不覆盖 live**；
`accept_no_pit = true`、`live_official_pit = NO-GO`。
_Avoid_: live 授权（清单未授予，本库 MUST NOT 据此放行）

## 非目标（明确排除）

- 联网采集 / live / 官方 PIT
- 端点 URL、认证参数名、限流数字与凭据字段名（属规划描述，不入库）
- 派生指标（增长率、贡献度、平减指数、z-score）
- 单位换算、曲线构建、存储与分发
- 任何能力等级、SLA、新鲜度保证或生产就绪宣称

## `rust-version` 推导

规则：`rust-version` = 依赖图中所有依赖所声明 `rust_version` 的**最大值**
（`cargo metadata --format-version 1` 的 `rust_version` 字段）。

本 crate 的直接依赖与其 `rust_version`：

| 依赖 | 版本 | `rust_version` |
| --- | --- | --- |
| `thiserror` | 2.0.20 | 1.71 |
| `serde` | 1.0.229 | 1.56 |
| `serde_json` | 1.0.151 | 1.71 |

传递依赖：`itoa` 1.0.18（1.68）、`memchr` 2.8.3（1.61）、`proc-macro2` 1.0.107（1.71）、
`quote` 1.0.47（1.71）、`serde_core` 1.0.229（1.56）、`serde_derive` 1.0.229（1.71）、
`syn` 3.0.6（1.71）、`thiserror-impl` 2.0.20（1.71）、`unicode-ident` 1.0.26（1.71）、
`zmij` 1.0.23（1.71）。

最大值 = **1.71**，故 `rust-version = "1.71"`（2026-09-22 由 `cargo metadata` 实读推导）。
所用语言特性（`let … else`、内联格式参数、`const fn` 常量求值）下界均不高于 1.71。

## 已知缺口

| # | 缺口 | 说明 |
| --- | --- | --- |
| G1 | 无联网采集 | 本特性不实现；`production_decision = NO-GO` |
| G2 | 无官方 vintage 面 | live 官方 PIT 恒 `NO-GO`；`is_formal_pit_eligible()` 恒 `false` |
| G3 | 未逐表声明源单位 | 清单未逐表声明源单位，故 `BeaUnit` 为开放 newtype，`unit` 取值由输入给出 |
| G4 | 非 NIPA 数据集无表级明细 | 清单自述「明细为历史材料，未入库」，本库 MUST NOT 编造；夹具用 `SYN-` 前缀占位 |
| G5 | 仅两表有声明频率 | `T10101` / `T20100`；其余表 MUST NOT 推定 |
| G6 | `T11000`（GDI 候选）待 live 核验 | MUST NOT 被表述为已入白名单 |
| G7 | `T20600` 现窗口未入白名单 | 与 `T20100` 语义不同，MUST NOT 互换 |
| G8 | 夹具为合成样本 | `tests/fixtures/` 全部自拟，不是真实源数据，不构成证据 |
| G9 | 未建模 measure / subject 维度 | 清单提及身份含 measure / subject，但未登记其具体字段形态；本层以 `line_number` 定位，避免编造字段名 |
