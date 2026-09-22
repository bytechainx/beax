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

## 相关文档

- 源清单（采集范围权威）：`specs/adapter/bea.md`
- 跨源语义：`specs/005-macro-data-source-crates/contracts/cross-source-routing.md`
- 公共形状契约：`specs/005-macro-data-source-crates/contracts/source-library-contract.md`
- 术语与边界：`CONTEXT.md`；公开面：`docs/API.md`；能力标准：`docs/标准.md`
- 组织 Rust 规范：`~/org-config/rulesets/rust/`（`RULES.md`）
