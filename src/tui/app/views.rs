use super::TuiPanel;

impl TuiPanel {
    pub fn next(self) -> Self {
        match self {
            Self::Table => Self::Detail,
            Self::Detail => Self::Log,
            Self::Log => Self::Runtime,
            Self::Runtime => Self::Table,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Table => Self::Runtime,
            Self::Detail => Self::Table,
            Self::Log => Self::Detail,
            Self::Runtime => Self::Log,
        }
    }
}
