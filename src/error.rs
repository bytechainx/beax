//! beax 的错误分类与错误类型。
//!
//! 错误分类按「调用方应如何反应」划分，调用方 MUST 匹配 [`BeaErrorKind`]，
//! MUST NOT 用错误消息的字符串匹配替代。
//!
//! 错误消息一律为简体中文，且 MUST NOT 回显原始响应正文、凭据或整行配置源码。

/// 错误分类：按「调用方应如何反应」划分。
///
/// 禁止用字符串匹配替代对本枚举的匹配。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BeaErrorKind {
    /// 输入形态或取值非法（调用方应修数据，重试无意义）。
    Invalid,
    /// 缺少必需项。
    Missing,
    /// 授权判定未通过（fail-closed 落点）。
    AuthorizationDenied,
    /// 该产品不属于本域，已被路由规则拒绝。
    RoutedElsewhere,
    /// 越权写入（权威写入归他域）。
    WriteAuthorityDenied,
    /// 结构可解析但语义不被接受（如近义非同 ID）。
    SemanticallyRejected,
    /// 尚未实现的规划能力。
    NotApplicable,
    /// 不变量被破坏（库内 bug 的信号）。
    Invariant,
}

/// beax 错误。保留可区分的分类与来源链。
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum BeaError {
    /// 输入非法。
    #[error("输入非法：{0}")]
    Invalid(String),
    /// 缺少必需项。
    #[error("缺少必需项：{0}")]
    Missing(String),
    /// 授权判定未通过。
    #[error("授权判定未通过：{0}")]
    AuthorizationDenied(String),
    /// 产品不属于本域。
    #[error("不属于本域，已路由他处：{0}")]
    RoutedElsewhere(String),
    /// 越权写入。
    #[error("越权写入被拒绝：{0}")]
    WriteAuthorityDenied(String),
    /// 语义拒绝。
    #[error("语义不被接受：{0}")]
    SemanticallyRejected(String),
    /// 规划能力未实现。
    #[error("规划能力尚未实现：{0}")]
    NotApplicable(String),
    /// 不变量被破坏。
    #[error("不变量被破坏：{0}")]
    Invariant(String),
}

impl BeaError {
    /// 分类，供调用方按「如何反应」分流。
    #[must_use]
    pub fn kind(&self) -> BeaErrorKind {
        match self {
            Self::Invalid(_) => BeaErrorKind::Invalid,
            Self::Missing(_) => BeaErrorKind::Missing,
            Self::AuthorizationDenied(_) => BeaErrorKind::AuthorizationDenied,
            Self::RoutedElsewhere(_) => BeaErrorKind::RoutedElsewhere,
            Self::WriteAuthorityDenied(_) => BeaErrorKind::WriteAuthorityDenied,
            Self::SemanticallyRejected(_) => BeaErrorKind::SemanticallyRejected,
            Self::NotApplicable(_) => BeaErrorKind::NotApplicable,
            Self::Invariant(_) => BeaErrorKind::Invariant,
        }
    }

    /// 是否值得重试。本层无网络，除 `Invariant` 外一律 `false`。
    ///
    /// `Invariant` 返回 `true` 只表示「这是库内 bug 的信号，值得复查后重跑」，
    /// 不代表任何自动重试策略。
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(self.kind(), BeaErrorKind::Invariant)
    }
}

/// 本 crate 统一结果别名。
pub type BeaResult<T> = Result<T, BeaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_maps_every_variant() {
        let cases = [
            (BeaError::Invalid("x".into()), BeaErrorKind::Invalid),
            (BeaError::Missing("x".into()), BeaErrorKind::Missing),
            (
                BeaError::AuthorizationDenied("x".into()),
                BeaErrorKind::AuthorizationDenied,
            ),
            (
                BeaError::RoutedElsewhere("x".into()),
                BeaErrorKind::RoutedElsewhere,
            ),
            (
                BeaError::WriteAuthorityDenied("x".into()),
                BeaErrorKind::WriteAuthorityDenied,
            ),
            (
                BeaError::SemanticallyRejected("x".into()),
                BeaErrorKind::SemanticallyRejected,
            ),
            (
                BeaError::NotApplicable("x".into()),
                BeaErrorKind::NotApplicable,
            ),
            (BeaError::Invariant("x".into()), BeaErrorKind::Invariant),
        ];
        for (err, expected) in cases {
            assert_eq!(err.kind(), expected);
        }
    }

    #[test]
    fn only_invariant_is_retryable() {
        assert!(BeaError::Invariant("x".into()).is_retryable());
        assert!(!BeaError::Invalid("x".into()).is_retryable());
        assert!(!BeaError::Missing("x".into()).is_retryable());
        assert!(!BeaError::AuthorizationDenied("x".into()).is_retryable());
        assert!(!BeaError::RoutedElsewhere("x".into()).is_retryable());
        assert!(!BeaError::WriteAuthorityDenied("x".into()).is_retryable());
        assert!(!BeaError::SemanticallyRejected("x".into()).is_retryable());
        assert!(!BeaError::NotApplicable("x".into()).is_retryable());
    }

    #[test]
    fn display_is_non_empty_and_chinese() {
        let err = BeaError::RoutedElsewhere("曲线构建归 yieldx".into());
        let shown = err.to_string();
        assert!(!shown.is_empty());
        assert!(shown.contains("曲线构建归 yieldx"));
        assert!(shown.contains("已路由他处"));
    }

    #[test]
    fn display_does_not_leak_credentials_or_endpoints() {
        // 本层不接触凭据与端点；此处钉死错误消息只由调用方给出的短标签构成。
        let err = BeaError::Invalid("dataset_id 为空".into());
        let shown = err.to_string();
        assert!(!shown.contains("password"));
        assert!(!shown.contains("API_KEY"));
        assert!(!shown.contains("http"));
    }
}
