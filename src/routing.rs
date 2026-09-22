//! beax 侧的跨源守卫：曲线产品路由、核心 PCE 主责、近义非同 ID。
//!
//! 本模块只实现 `cross-source-routing.md` §7 分配给 `beax` 的**自己那一侧**。
//! 跨源整体语义归该契约文档，MUST NOT 在本库内重新裁定任何未决项。

use crate::dataset::{T10101, T11000, T20100, T20300, T20600};
use crate::error::{BeaError, BeaResult};

/// 曲线构建的归属方（`cross-source-routing.md` §2）。
pub const CURVE_BUILD_OWNER: &str = "yieldx";

/// 核心 PCE 采集主责的归属方（`fred-forward`，`cross-source-routing.md` §3/§8 P3）。
pub const CORE_PCE_PRIMARY_OWNER: &str = "fredx";

/// 近义非同 ID 全表中**涉及 `beax` 的 6 对**（清单逐条钉死，MUST NOT 互为别名）。
///
/// 顺序对其判定无影响。
pub const NEAR_SYNONYM_PAIRS: &[(&str, &str)] = &[
    ("GDPNow", "GDP"),
    ("PCEPILFE", "PCEPI"),
    (T20100, "PCEPILFE"),
    (T20100, T20300),
    (T20100, T20600),
    ("GDP", "GDI"),
];

/// 本域可主张的身份类别。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeaClaim {
    /// 表内自身口径的**补充**观测（本域职责）。
    SupplementaryTableValue,
    /// 核心 PCE（`PCEPILFE`）的**主责**采集身份（归 [`CORE_PCE_PRIMARY_OWNER`]）。
    CorePcePrimary,
    /// 收益率曲线（期限结构）构建（归 [`CURVE_BUILD_OWNER`]）。
    YieldCurveConstruction,
}

/// 该对 ID 是否属于清单钉死的「近义非同 ID」。
#[must_use]
pub fn is_near_synonym_pair(left: &str, right: &str) -> bool {
    NEAR_SYNONYM_PAIRS
        .iter()
        .any(|(a, b)| (left == *a && right == *b) || (left == *b && right == *a))
}

/// 禁静默替换守卫：被钉死的近义对 MUST NOT 互换或互为别名。
///
/// # Errors
///
/// 该对属于 [`NEAR_SYNONYM_PAIRS`] 时返回 [`BeaError::SemanticallyRejected`]。
pub fn ensure_not_silent_substitution(left: &str, right: &str) -> BeaResult<()> {
    if is_near_synonym_pair(left, right) {
        return Err(BeaError::SemanticallyRejected(format!(
            "{left} 与 {right} 语义不同，不得互为别名或静默替换"
        )));
    }
    Ok(())
}

/// 身份主张守卫。
///
/// - 本域只可主张**表内自身口径的补充观测**；
/// - 核心 PCE 的采集主责在 [`CORE_PCE_PRIMARY_OWNER`]，本域 MUST NOT 反向主张
///   （`T20100` 表内 PCE 行 ≠ `PCEPILFE`；`PCEPI` 亦非同 ID）；
/// - 曲线构建归 [`CURVE_BUILD_OWNER`]，本域 MUST NOT 自动晋级为曲线观测。
///
/// 本函数是**只读判定**，MUST NOT 改写任何授权或主权登记值。
///
/// # Errors
///
/// 主张 [`BeaClaim::CorePcePrimary`] 返回 [`BeaError::WriteAuthorityDenied`]；
/// 主张 [`BeaClaim::YieldCurveConstruction`] 返回 [`BeaError::RoutedElsewhere`]。
pub fn ensure_claim_local(claim: BeaClaim) -> BeaResult<()> {
    match claim {
        BeaClaim::SupplementaryTableValue => Ok(()),
        BeaClaim::CorePcePrimary => Err(BeaError::WriteAuthorityDenied(format!(
            "核心 PCE 采集主责在 {CORE_PCE_PRIMARY_OWNER}（fred-forward）；本域仅为补充口径"
        ))),
        BeaClaim::YieldCurveConstruction => Err(BeaError::RoutedElsewhere(format!(
            "曲线产品不得自动进入本域，须路由 {CURVE_BUILD_OWNER}"
        ))),
    }
}

/// 该表号是否为本域可承载的 NIPA 白名单表（避免把非白名单表当采集目标）。
///
/// 仅作形态/范围提示；白名单权威在 [`crate::dataset::is_known_nipa_table`]。
#[must_use]
pub fn is_gdp_or_personal_income_table(table_id: &str) -> bool {
    table_id == T10101 || table_id == T20100
}

/// GDI 候选表号；**待 live 核验**，未入默认采集白名单。
#[must_use]
pub fn gdi_candidate_table() -> &'static str {
    T11000
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::BeaErrorKind;

    #[test]
    fn curve_products_are_routed_away() {
        let err = ensure_claim_local(BeaClaim::YieldCurveConstruction).expect_err("应路由");
        assert_eq!(err.kind(), BeaErrorKind::RoutedElsewhere);
        assert!(err.to_string().contains(CURVE_BUILD_OWNER));
    }

    #[test]
    fn core_pce_primary_is_never_claimed_by_this_crate() {
        assert!(ensure_claim_local(BeaClaim::SupplementaryTableValue).is_ok());
        let err = ensure_claim_local(BeaClaim::CorePcePrimary).expect_err("应拒绝");
        assert_eq!(err.kind(), BeaErrorKind::WriteAuthorityDenied);
        assert!(err.to_string().contains(CORE_PCE_PRIMARY_OWNER));
    }

    #[test]
    fn every_documented_pair_is_detected_in_both_directions() {
        assert_eq!(NEAR_SYNONYM_PAIRS.len(), 6);
        for (left, right) in NEAR_SYNONYM_PAIRS {
            assert!(is_near_synonym_pair(left, right), "{left}/{right}");
            assert!(is_near_synonym_pair(right, left), "{right}/{left}");
            assert_eq!(
                ensure_not_silent_substitution(left, right)
                    .expect_err("应拒绝")
                    .kind(),
                BeaErrorKind::SemanticallyRejected
            );
        }
    }

    #[test]
    fn supplementary_pce_line_never_replaces_core_pce_identity() {
        assert!(ensure_not_silent_substitution(T20100, "PCEPILFE").is_err());
        assert!(ensure_not_silent_substitution("PCEPILFE", T20100).is_err());
        assert!(ensure_not_silent_substitution("PCEPILFE", "PCEPI").is_err());
    }

    #[test]
    fn unrelated_pairs_are_allowed() {
        assert!(ensure_not_silent_substitution(T10101, T20100).is_ok());
        assert!(ensure_not_silent_substitution("NIPA", "ITA").is_ok());
    }

    #[test]
    fn anchor_helpers_match_the_manifest() {
        assert!(is_gdp_or_personal_income_table(T10101));
        assert!(is_gdp_or_personal_income_table(T20100));
        assert!(!is_gdp_or_personal_income_table("T10102"));
        assert_eq!(gdi_candidate_table(), "T11000");
    }
}
