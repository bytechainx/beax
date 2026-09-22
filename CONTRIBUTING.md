# CONTRIBUTING.md — 贡献指南（beax）

本文件面向贡献者，汇总本地门禁与提交约定。
AI Agent 的工作约定另见 [`AGENTS.md`](./AGENTS.md)；术语与边界见 [`CONTEXT.md`](./CONTEXT.md)。

## 开发流程

- 本仓库是**独立的单 crate 仓库**：不依赖 `xhyper.rs` 主工程及其内部 crate，
  也不依赖任何 `bytechainx/*` crate（MUST NOT 出现 `path = "../…"`）。
- substantial 变更走 feature branch → PR → review → merge，**禁止直接 push `main`**。
- 合并方式固定为 **create a merge commit**（`gh pr merge` 须显式传 `--subject` 与 `--body`）。
- 提交信息遵循 Conventional Commits（`feat:` / `fix:` / `docs:` / `ci:` / `chore:` / `refactor:`），
  描述用简体中文。

## 本地门禁（四件套）

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features        # 含单元测试、集成测试与 doctest
cargo package --no-verify        # 只校验打包元数据；本 crate 不发布 crates.io
```

## 复用口径（不发布 crates.io）

- 本 crate **不发布到 crates.io**，仅以 GitHub 源码 / git 依赖形式复用。
- 文档与元数据中不得出现「可独立发布」「可直接 `cargo publish`」等表述，
  也不得放置 crates.io / docs.rs 徽章与外链。
- `Cargo.toml` 的 `documentation` 指向 `https://github.com/bytechainx/beax#readme`。
- 消费方引入方式以 [`README.md`](./README.md) 的「安装」小节为准。

## 开发约定

- 注释、文档、错误消息使用**简体中文**；标识符保持英文。
- **零网络**：MUST NOT 引入 HTTP 客户端（`reqwest` / `hyper` / `ureq` / `curl` / `isahc` …）、
  异步运行时（`tokio` / `async-std`）、`chrono` / `time`、`rand`；MUST NOT 写端点字面量；
  MUST NOT 读环境变量或凭据。
- **端点与凭据不入库**：端点 URL、认证参数名、限流数字与凭据字段名 MUST NOT 出现在源码、
  文档或夹具中——它们属规划描述，不是访问合同。
- **零派生与零换算**：值对象里 MUST NOT 出现增长率 / 贡献度 / 平减指数 / z-score；
  单位换算归下游，本层只保留源单位。
- **零重复裁定**：跨源未决项（GDI 候选 `T11000` 的核验、非 NIPA 数据集明细表号、
  measure / subject 维度）MUST NOT 由实现者单方面裁定或编造。
- **身份边界**：核心 PCE 采集主责在 `fredx`（fred-forward），本域 MUST NOT 反向主张，
  也 MUST NOT 用表内 PCE 行静默替换 `PCEPILFE`。
- 错误模型：`BeaErrorKind` 语义分类 + `#[non_exhaustive]` 的 `BeaError` + `BeaResult<T>`；
  调用方按分类分流，MUST NOT 用消息字符串匹配。
- 错误消息 MUST NOT 回显原始响应正文、凭据或整行配置源码。
- 非测试路径 MUST NOT 出现 `unwrap()` / `expect()` / `panic!` / `println!` / `dbg!`
  （`[lints.clippy]` 已 `deny` 相应项）；所有 `pub` 项 MUST 有中文 `///` 文档。
- 生产段行数 MUST ≤ 500（`MR-STRUCT-007` 的 WARN 阈值以下），MUST NOT 触及 800 行 ERROR 阈值；
  单文件模块超过约 100 行宜拆为 `foo.rs` + `foo/` 子模块（禁 `mod.rs`）。
- 单元测试与源码**同文件**（组织 P0）；集成测试放 `tests/` 且只访问公共 API。
- `tests/fixtures/` 下全部为**合成样本**，MUST 带 `"_synthetic": true` 标注，
  MUST NOT 被表述为「实测」「核验 PASS」或任何证据等级。

## 提交前自检清单

- [ ] `cargo fmt --all -- --check` 通过
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` 通过
- [ ] `cargo test --all-features` 通过（含 doctest）
- [ ] `cargo package --no-verify` 通过
- [ ] `grep -c '^## ' docs/标准.md` 与 `grep -c '// SPEC-MAP:' tests/sdd_spec.rs` 相等
- [ ] `grep -rn 'https\?://' src` 无输出；`grep -rnE 'env::var|from_env' src` 无输出
- [ ] 新增 `pub` 项都有中文 `///` 文档
