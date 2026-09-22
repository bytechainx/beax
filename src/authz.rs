//! beax 的授权判定（fail-closed）。
//!
//! 判定是**只读**结论：它不改写 `specs/adapter/bea.owner-approve.json` 的任何登记值，
//! 也不表示本库生产就绪（`production_decision = NO-GO` 与「offline/reference 范围被授权」
//! 共存不矛盾）。
//!
//! 已登记证据的 `scope` 是 **offline_and_reference_only**：offline 与 reference
//! 两个范围被覆盖，**live 不被覆盖**（`live_official_pit = NO-GO`，`accept_no_pit = true`）。
//!
//! 判定输入是调用方给出的证据描述与评估日期（**无墙钟**）；证据缺失 / 签核编号不明 /
//! 签署者不明 / 覆盖范围不明 / 已过期 → 一律 [`BeaAuthorization::Denied`]。

use crate::error::BeaResult;
use crate::value::{validate_date, Date};

/// Owner 签核编号（`specs/adapter/bea.owner-approve.json`）。
pub const BEA_DECISION_ID: &str = "BEA-PROD-2026-08-17-approve";

/// 签署者。
pub const BEA_SIGNED_BY: &str = "ZoneCNH";

/// live 官方 PIT 的裁定值：**NO-GO**。
pub const LIVE_OFFICIAL_PIT_DECISION: &str = "NO-GO";

/// 访问模式。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeaAccessMode {
    /// 离线解析与合成夹具。
    Offline,
    /// 参考口径（reference）：官方直连的对照/补充使用。
    ReferenceOnly,
    /// live 联网访问：本特性不实现，**且未被证据覆盖**。
    Live,
}

/// 授权判定结果。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeaAuthorization {
    /// 证据有效且覆盖本次请求的范围与有效期。
    Authorized {
        /// 被覆盖的源 / 模式 / 用途 / 有效期。
        scope: String,
    },
    /// 证据缺失 / 过期 / 签署者不明 / 覆盖范围不明。
    Denied {
        /// 可读的拒绝理由。
        reason: String,
    },
}

/// 只读的授权证据登记。
///
/// 本结构**只承载证据描述**；它不能被用来「默认放行」——判定入口
/// [`authorize_bea`] 对缺失与不明一律拒绝。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaAuthorizationEvidence {
    /// Owner 签核编号。
    pub decision_id: String,
    /// 签署者。
    pub signed_by: String,
    /// 签核时间（证据登记值）。
    pub signed_at: Option<Date>,
    /// 有效期上界；`None` 表示证据未声明有效期。
    pub valid_until: Option<Date>,
    /// 被覆盖的访问模式集合。
    pub authorized_modes: Vec<BeaAccessMode>,
    /// 是否接受「无官方 PIT」这一源级例外。
    pub accept_no_pit: bool,
    /// live 官方 PIT 的裁定值。
    pub live_official_pit: String,
    /// 范围说明（人类可读）。
    pub scope_note: String,
}

/// 清单已登记的 bea 证据（`BEA-PROD-2026-08-17-approve`）。
///
/// `scope = offline_and_reference_only`：覆盖 [`BeaAccessMode::Offline`] 与
/// [`BeaAccessMode::ReferenceOnly`]，**不覆盖** [`BeaAccessMode::Live`]。
#[must_use]
pub fn documented_bea_evidence() -> BeaAuthorizationEvidence {
    BeaAuthorizationEvidence {
        decision_id: BEA_DECISION_ID.to_owned(),
        signed_by: BEA_SIGNED_BY.to_owned(),
        signed_at: Date::new(2026, 8, 17).ok(),
        valid_until: None,
        authorized_modes: vec![BeaAccessMode::Offline, BeaAccessMode::ReferenceOnly],
        accept_no_pit: true,
        live_official_pit: LIVE_OFFICIAL_PIT_DECISION.to_owned(),
        scope_note: "BEA offline_and_reference_only 范围（live 官方 PIT = NO-GO）".to_owned(),
    }
}

/// fail-closed 授权判定。
///
/// 判据（任一不满足即 `Denied`）：
///
/// 1. 证据存在且非空
/// 2. 签核编号非空（签署者不明）
/// 3. 签署者非空（签署者不明）
/// 4. 覆盖模式集合非空（覆盖范围不明）
/// 5. 证据未过期（`valid_until` 已声明时，`as_of` MUST NOT 晚于它）
/// 6. 请求模式 ∈ 覆盖模式集合（**live 不在其中**）
///
/// `as_of` 由调用方传入，本层不读取系统时间。
#[must_use]
pub fn authorize_bea(
    evidence: Option<&BeaAuthorizationEvidence>,
    requested: BeaAccessMode,
    as_of: Date,
) -> BeaAuthorization {
    let Some(evidence) = evidence else {
        return BeaAuthorization::Denied {
            reason: "缺少 Owner 签核证据".to_owned(),
        };
    };
    if evidence.decision_id.is_empty() {
        return BeaAuthorization::Denied {
            reason: "签核编号不明".to_owned(),
        };
    }
    if evidence.signed_by.is_empty() {
        return BeaAuthorization::Denied {
            reason: "签署者不明".to_owned(),
        };
    }
    if evidence.authorized_modes.is_empty() {
        return BeaAuthorization::Denied {
            reason: "证据未声明任何被覆盖的范围".to_owned(),
        };
    }
    if let Some(valid_until) = evidence.valid_until {
        if as_of > valid_until {
            return BeaAuthorization::Denied {
                reason: "证据已过期".to_owned(),
            };
        }
    }
    if !evidence.authorized_modes.contains(&requested) {
        return BeaAuthorization::Denied {
            reason: format!(
                "请求的访问模式未被证据覆盖：{}（{}）",
                mode_label(requested),
                evidence.scope_note
            ),
        };
    }
    BeaAuthorization::Authorized {
        scope: format!(
            "{} / {} / {}",
            evidence.decision_id,
            evidence.scope_note,
            mode_label(requested)
        ),
    }
}

/// 是否接受「BEA 无官方 vintage 面」这一源级例外（清单登记值）。
#[must_use]
pub fn accepts_no_official_pit() -> bool {
    true
}

/// live 官方 PIT 的裁定值（恒为 [`LIVE_OFFICIAL_PIT_DECISION`]）。
#[must_use]
pub fn live_official_pit_decision() -> &'static str {
    LIVE_OFFICIAL_PIT_DECISION
}

/// 模式的稳定记号。
#[must_use]
pub fn mode_label(mode: BeaAccessMode) -> &'static str {
    match mode {
        BeaAccessMode::Offline => "offline",
        BeaAccessMode::ReferenceOnly => "reference_only",
        BeaAccessMode::Live => "live",
    }
}

/// 校验一份证据描述自身的形态（日期分量合法）。
///
/// # Errors
///
/// `signed_at` / `valid_until` 的日期分量非法时返回 [`crate::BeaError::Invalid`]。
pub fn validate_authorization_evidence(evidence: &BeaAuthorizationEvidence) -> BeaResult<()> {
    if let Some(signed_at) = evidence.signed_at {
        validate_date(&signed_at)?;
    }
    if let Some(valid_until) = evidence.valid_until {
        validate_date(&valid_until)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn as_of() -> Date {
        Date::new(2026, 8, 20).expect("日期合法")
    }

    #[test]
    fn documented_evidence_covers_offline_and_reference_only() {
        let evidence = documented_bea_evidence();
        assert!(validate_authorization_evidence(&evidence).is_ok());
        for mode in [BeaAccessMode::Offline, BeaAccessMode::ReferenceOnly] {
            match authorize_bea(Some(&evidence), mode, as_of()) {
                BeaAuthorization::Authorized { scope } => {
                    assert!(scope.contains(BEA_DECISION_ID));
                    assert!(scope.contains(mode_label(mode)));
                }
                other => panic!("应授权 {mode:?}，实得 {other:?}"),
            }
        }
    }

    #[test]
    fn live_is_not_covered() {
        let evidence = documented_bea_evidence();
        match authorize_bea(Some(&evidence), BeaAccessMode::Live, as_of()) {
            BeaAuthorization::Denied { reason } => {
                assert!(reason.contains("live"));
            }
            other => panic!("应拒绝 live，实得 {other:?}"),
        }
        assert_eq!(live_official_pit_decision(), "NO-GO");
        assert!(accepts_no_official_pit());
        assert_eq!(evidence.live_official_pit, "NO-GO");
    }

    #[test]
    fn missing_evidence_is_denied() {
        match authorize_bea(None, BeaAccessMode::Offline, as_of()) {
            BeaAuthorization::Denied { reason } => assert_eq!(reason, "缺少 Owner 签核证据"),
            other => panic!("应拒绝，实得 {other:?}"),
        }
    }

    #[test]
    fn unknown_scope_is_denied() {
        let mut evidence = documented_bea_evidence();
        evidence.authorized_modes.clear();
        assert!(matches!(
            authorize_bea(Some(&evidence), BeaAccessMode::Offline, as_of()),
            BeaAuthorization::Denied { .. }
        ));

        let mut evidence = documented_bea_evidence();
        evidence.decision_id = String::new();
        assert!(matches!(
            authorize_bea(Some(&evidence), BeaAccessMode::Offline, as_of()),
            BeaAuthorization::Denied { .. }
        ));

        let mut evidence = documented_bea_evidence();
        evidence.signed_by = String::new();
        assert!(matches!(
            authorize_bea(Some(&evidence), BeaAccessMode::Offline, as_of()),
            BeaAuthorization::Denied { .. }
        ));
    }

    #[test]
    fn expired_evidence_is_denied() {
        let mut evidence = documented_bea_evidence();
        evidence.valid_until = Date::new(2026, 8, 16).ok();
        match authorize_bea(Some(&evidence), BeaAccessMode::Offline, as_of()) {
            BeaAuthorization::Denied { reason } => assert_eq!(reason, "证据已过期"),
            other => panic!("应拒绝，实得 {other:?}"),
        }
    }

    #[test]
    fn invalid_evidence_dates_are_rejected() {
        let mut evidence = documented_bea_evidence();
        evidence.valid_until = Some(Date {
            year: 2026,
            month: 2,
            day: 30,
        });
        assert!(validate_authorization_evidence(&evidence).is_err());
    }

    #[test]
    fn mode_labels_are_stable() {
        assert_eq!(mode_label(BeaAccessMode::Offline), "offline");
        assert_eq!(mode_label(BeaAccessMode::ReferenceOnly), "reference_only");
        assert_eq!(mode_label(BeaAccessMode::Live), "live");
    }
}
