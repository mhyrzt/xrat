use super::ConfigSort;

impl ConfigSort {
    pub(super) fn next(self) -> Self {
        match self {
            Self::RealDelay => Self::Id,
            Self::Id => Self::Name,
            Self::Name => Self::Protocol,
            Self::Protocol => Self::RealDelay,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::RealDelay => "real-delay",
            Self::Id => "id",
            Self::Name => "name",
            Self::Protocol => "protocol",
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
        }
    }
}
