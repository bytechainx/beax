# beax Agent 指南

> 本文件为 AI Agent 在本仓库工作时的入口指南。

## 项目定位

BEA 源事实库：Dataset 与 NIPA 表号常量、跨源守卫、离线解析与 fail-closed 授权判定。
**不是联网采集器**；`production_decision = NO-GO`；live 官方 PIT 恒 `NO-GO`。

## 技术栈

- Rust edition 2021，`rust-version = "1.71"`（由依赖 `rust_version` 最大值推导，见 `CONTEXT.md`）
- 依赖仅 `thiserror` 2、`serde` 1（`derive`）、`serde_json` 1
- **零内部耦合**：不依赖任何 `bytechainx/*` crate；MUST NOT 出现 `path = "../…"`
- crate 级 lint：`unwrap_used` / `expect_used` / `panic` / `unreachable` / `todo` /
  `unimplemented` 全部 `deny`（测试与基准经头部 `#![allow(...)]` 豁免）

## 代码结构

```text
src/
├── lib.rs      # crate 文档 + 模块声明 + 门面 pub use + doctest
├── error.rs    # BeaErrorKind / BeaError / BeaResult
├── value.rs    # Date / Period / Frequency / BeaUnit / BeaValue / BeaObservation + validate*
├── dataset.rs  # 11 个 Dataset 常量、NIPA 14 表白名单、优先级、频率表、范围守卫
├── routing.rs  # 曲线路由、核心 PCE 主责、近义非同 ID（本域自己那一侧）
├── pit.rs      # publication 三元组（恒为 Date / Inferred / NotEligible）
├── authz.rs    # fail-closed 授权判定与只读证据登记（offline / reference 覆盖，live 不覆盖）
└── parse.rs    # 离线 JSON 解析（只吃字符串）
```

依赖方向单向：`error` ← `value` ← {`dataset`, `authz`, `pit`, `parse`}；
`routing` 依赖 `error` + `dataset`。

## 开发约定

- 注释与文档使用简体中文；标识符保持英文；所有 `pub` 项有 `///` 文档（`missing_docs` 已 `deny`）
- 禁止裸 `unwrap()` / `expect()` / `panic!` / `println!`（库代码；lint 已 deny）
- **硬禁止**：HTTP 客户端、异步运行时、`chrono`/`time`、`rand`、端点字面量、环境变量/凭据读取、
  代理配置、`bytechainx/*` 依赖
- **硬禁止**：端点 URL、认证参数名、限流数字与凭据字段名入库（属规划描述）
- **硬禁止**：值对象里做单位换算或派生指标；曲线构建（归 `yieldx`）；
  反向主张核心 PCE 主责（归 `fredx`）
- 清单范围外的 Dataset / 白名单外的 NIPA 表号 MUST NOT 被接受；
  未决项（GDI 候选、非 NIPA 明细表号、measure / subject）MUST NOT 由本库裁定或编造
- 夹具必须标注 `"_synthetic": true`，MUST NOT 被表述为证据
- 生产段 ≤ 500 行（WARN 阈值以下）；单文件超过约 100 行宜拆 `foo.rs` + `foo/`；禁 `mod.rs`

## 门禁四件套

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo package --no-verify
```

## E2E 全公开面覆盖核对（手工，**不进 CI**）

除四件套外，本仓有一条**公开面覆盖**核对：`tests/e2e_bea.rs` 必须把本仓**核对器口径内的全部公开条目**
逐条真实执行一遍。核对器在**元仓库根目录**执行（`--root` 指向本仓的隔离 worktree 父目录）：

```bash
node scripts/verify-e2e-coverage.mjs beax \
  --root /home/workspace/bytechainx/.worktrees/beax \
  --target-dir /home/workspace/bytechainx/.cargo/wt/beax
```

- 退出码：**0** 全过 / **1** 有发现（含未覆盖）/ **2** 工具自身或环境错误（缺工具时**一律 2**，不降级成「通过」）
- 三层判据，缺一不可：① 权威公开面由 `cargo +nightly public-api --simplified` 派生（**不采信测试自述**）；
  ② 测试内 `E2E_MANIFEST` 与权威公开面**双向 diff**（少一条 = missing、多一条 = ghost，都判红）；
  ③ `-C instrument-coverage` + `cargo-llvm-cov` **按函数**取执行次数，每条公开 `fn` 的 count 必须 > 0
- **实测基线（2026-09-23，带 `llvm-cov`）**：退出码 **0**；权威公开条目 **163** / 清单声明 **163** /
  `公开 fn 执行 39/39`（分项 `type` 18 / `variant` 46 / `field` 22 / `const` 38 / `fn` 39）；
  文件 sha256 `5146ea778701bbf54af542f574e19748f3c76812213da790ba11266740a75c95`（基线 `origin/main = b14d1fd`）。
  `tests/e2e_bea.rs` 为**单一** `#[test] e2e_bea_all_public_api`（8 个 phase 子函数），
  `[dev-dependencies]` 为空、无环境变量 / 无网络 / 无文件副作用（解析阶段读仓内真实合成夹具）；
  清单**逐字节等于** `renderManifest(authoritative)`（工具生成、非手抄）
- **本仓是离线仓 ⇒ 测试不标注忽略、默认参与 CI**（与 `clickhousex` / `taosx` / `kafkax` 等价需要外部服务的仓相反）
- **为何本仓需要它**：`beax` 是**公开模块**（`pub mod`）形态的仓 —— `cargo public-api` 对这类仓的
  定义行与固有 impl 方法**一律带模块段**（`pub fn beax::value::Date::new(…)`、`impl beax::value::Date`），
  只有再导出项才是 crate 根形态。核对器在 2026-09-23 才修好这一形态（`F4`，此前对模块段
  「漏方法 + 造幽灵」且**双向 diff 恒绿**）⇒ **跑之前先确认取到的是修好的核对器版本**
- **口径边界（**不得**当成「已覆盖全部公开接口」）**：
  - **7 个「两级嵌套」公开字段未登记 ⇒ 三层判据不保护**（**不是**「未覆盖」—— 它们在行为上确实被测到，
    构造 / 穷尽解构 / `validate_period` / `Debug` 真读）：`BeaAuthorization::Authorized::scope`、
    `BeaAuthorization::Denied::reason`、`Period::Event::date`、`Period::Month::{year,month}`、
    `Period::Quarter::{quarter,year}`。删字段或改名时核对器**不报**，只能靠**编译失败**兜底。
    该下界**全局存在**（`configx` 2 / `taosx` 2 / `clickhousex` 0 / `kafkax` 0 / `fredx` 7）
  - **元组结构体**：本仓 `pub struct BeaUnit(_)` 的字段是**私有**的 ⇒ **没有** `BeaUnit::0` 条目
    （与 `fredx` 的公开字段 `FredUnit::0` 不同）；但元组结构体**本身**（`type BeaUnit`）属真实公开面，
    **必须**登记（这正是 `F4` 的 `\(` 修法覆盖的形态）
  - **derive / auto impl 不计入**（`clone` / `eq` / `fmt` / `serialize` …）⇒ 口径实为「公开**条目**（子集）」，
    故正确表述是「已覆盖**核对器口径内的**全部公开条目（163 条）」
  - **拒绝理由串是措辞锁**：`authorize_bea` 的拒绝分支断言**精确理由串**（锁在 `b14d1fd`）。
    改文案即红 —— 这是**有意**（措辞即契约），代价是**无关重构也会变红**
- 另需外部工具 `cargo +nightly public-api` / `cargo-llvm-cov` / `rustfilt`；口径与判定细节见元仓库
  `scripts/AGENTS.md` §2.1.1

## 相关文档

- 源清单（采集范围权威）：`specs/adapter/bea.md`
- 跨源语义：`specs/005-macro-data-source-crates/contracts/cross-source-routing.md`
- 公共形状契约：`specs/005-macro-data-source-crates/contracts/source-library-contract.md`
- 术语与边界：`CONTEXT.md`；公开面：`docs/API.md`；能力标准：`docs/标准.md`
- 组织 Rust 规范：`~/org-config/rulesets/rust/`（`RULES.md`）
