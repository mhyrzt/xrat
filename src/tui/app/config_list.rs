use super::{ConfigFilter, ConfigSort};

impl ConfigSort {
    pub(super) fn next(self) -> Self {
        match self {
            Self::RealDelay => Self::TcpDelay,
            Self::TcpDelay => Self::Id,
            Self::Id => Self::Name,
            Self::Name => Self::Protocol,
            Self::Protocol => Self::Source,
            Self::Source => Self::RealDelay,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::RealDelay => "real-delay",
            Self::TcpDelay => "tcp-delay",
            Self::Id => "id",
            Self::Name => "name",
            Self::Protocol => "protocol",
            Self::Source => "source",
        }
    }

    pub(super) fn compare(
        self,
        left: &crate::tui::data::TuiConfigRow,
        right: &crate::tui::data::TuiConfigRow,
    ) -> std::cmp::Ordering {
        match self {
            Self::RealDelay => (left.real_delay_ms.unwrap_or(i64::MAX), left.id)
                .cmp(&(right.real_delay_ms.unwrap_or(i64::MAX), right.id)),
            Self::TcpDelay => (left.tcp_ms.unwrap_or(i64::MAX), left.id)
                .cmp(&(right.tcp_ms.unwrap_or(i64::MAX), right.id)),
            Self::Id => left.id.cmp(&right.id),
            Self::Name => left
                .display_name()
                .to_lowercase()
                .cmp(&right.display_name().to_lowercase())
                .then_with(|| left.id.cmp(&right.id)),
            Self::Protocol => left
                .protocol
                .cmp(&right.protocol)
                .then_with(|| left.id.cmp(&right.id)),
            Self::Source => (left.source_id.unwrap_or(i64::MAX), left.id)
                .cmp(&(right.source_id.unwrap_or(i64::MAX), right.id)),
        }
    }
}

impl ConfigFilter {
    pub(super) fn next(self) -> Self {
        match self {
            Self::None => Self::EnabledOnly,
            Self::EnabledOnly => Self::FailedOnly,
            Self::FailedOnly => Self::HasDelay,
            Self::HasDelay => Self::None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::EnabledOnly => "enabled-only",
            Self::FailedOnly => "failed-only",
            Self::HasDelay => "has-delay",
        }
    }

    pub(super) fn matches(self, config: &crate::tui::data::TuiConfigRow) -> bool {
        match self {
            Self::None => true,
            Self::EnabledOnly => config.is_enabled && !config.is_deleted,
            Self::FailedOnly => config.failure_reason.is_some(),
            Self::HasDelay => config.real_delay_ms.is_some(),
        }
    }
}
