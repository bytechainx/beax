# beax

`beax` 是 BEA 源的**离线源事实库**：把 `specs/adapter/bea.md` 声明的采集范围落成代码 ——
11 个 Dataset 常量、NIPA 表号白名单与优先级、曲线产品路由、核心 PCE 主责边界、
近义非同 ID 禁则、publication 语义与一个只吃字符串的离线解析器。

- **不是联网采集器**：零 HTTP 客户端依赖、零端点字面量、零凭据读取
- **零内部耦合**：不依赖任何 `bytechainx/*` crate，公共形状与兄弟库各自实现一遍
- **零派生指标**：不做单位换算、不算增长率 / 贡献度 / 平减指数
- 统一错误面：`BeaErrorKind` 语义分类 + `BeaError` / `BeaResult<T>`
- fail-closed 授权判定：证据缺失 / 不明 / 过期 / 未覆盖一律拒绝；**live 不在覆盖范围内**

## 安装（引入方式）

本 crate **不发布到 crates.io**，以 git 依赖引入：

```toml
[dependencies]
beax = { git = "https://github.com/bytechainx/beax" }
```

本地开发可直接用路径依赖：

```toml
[dependencies]
beax = { path = "../beax" }
```

## 用法示例

```rust
use beax::{parse_bea_observations, ensure_claim_local, BeaClaim, BeaErrorKind};

let input = r#"{"_synthetic": true, "records": [
    {"dataset_id": "NIPA", "table_id": "T20100", "line_number": 12, "date": "2026-06-30",
     "value": 19000.0, "unit": "Billions of dollars", "frequency": "monthly"}
]}"#;
let observations = parse_bea_observations(input)?;
assert_eq!(observations[0].table_id, "T20100");

// 守卫：核心 PCE 主责在 fredx，本域只承载补充观测。
assert!(ensure_claim_local(BeaClaim::SupplementaryTableValue).is_ok());
assert_eq!(
    ensure_claim_local(BeaClaim::CorePcePrimary).unwrap_err().kind(),
    BeaErrorKind::WriteAuthorityDenied
);
# Ok::<(), beax::BeaError>(())
```

### NIPA 表号白名单与守卫

```rust
use beax::{
    ensure_nipa_table, ensure_not_silent_substitution, is_known_nipa_table,
    CORE_NIPA_P0_TABLES, CORE_NIPA_P1_TABLES,
};

assert_eq!(CORE_NIPA_P0_TABLES.len() + CORE_NIPA_P1_TABLES.len(), 14);
assert!(is_known_nipa_table("T10101"));
assert!(ensure_nipa_table("T20600").is_err());
// 近义非同 ID 不得互换（表内 PCE 行 ≠ 核心 PCE 价格指数）。
assert!(ensure_not_silent_substitution("T20100", "PCEPILFE").is_err());
assert!(ensure_not_silent_substitution("GDP", "GDI").is_err());
```

## 边界（摘要）

| 边界 | 处理 |
| --- | --- |
| NIPA 表号 | 白名单 14 表（P0 2 + P1 12）；白名单外一律拒绝 |
| 曲线产品 | BEA 曲线类 MUST NOT 自动进本域，须路由 `yieldx` |
| 核心 PCE（`PCEPILFE`） | 采集主责在 `fredx`（fred-forward）；本域表内 PCE 行只是补充口径，MUST NOT 反向主张 |
| 近义非同 ID（6 对） | 双向禁静默替换（`T20100`≠`PCEPILFE`、`GDP`≠`GDI` …） |
| `T11000`（GDI 候选） | 待 live 核验，MUST NOT 被表述为已入白名单 |
| live 官方 PIT | **NO-GO**；证据只覆盖 offline 与 reference 两个范围 |

## 非目标

- 不做联网采集：无 HTTP 客户端、无端点字面量、不读环境变量或凭据
- 不做派生指标（增长率 / 贡献度 / 平减指数归 analytics）
- 不做单位换算（归下游 Normalize）、不构建曲线（归 `yieldx`）
- 不做 live / 官方 PIT（BEA 无官方 vintage 面）
- 不宣称任何能力等级、SLA、新鲜度保证或生产就绪

## 门禁

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo package --no-verify
```

三大类测试随仓提供：`tests/tdd_contracts.rs`（TDD 行为契约）、`tests/sdd_spec.rs`
（与 `docs/标准.md` 章节 1:1）、`tests/aidd_boundary.rs`（对抗 / 边界用例）。
`tests/fixtures/` 下**全部为合成样本**，不是真实源数据，不构成任何证据。

**诚实边界**：`production_decision = NO-GO`；清单 COMPLETE ≠ ship；
authorization ≠ Production Ready。offline / reference 范围被授权不代表 live 被授权。

## 许可

MIT OR Apache-2.0
