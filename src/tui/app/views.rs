use super::TuiView;

impl TuiView {
    pub fn next(self) -> Self {
        match self {
            Self::Configs => Self::Sources,
            Self::Sources => Self::Tests,
            Self::Tests => Self::Diagnostics,
            Self::Diagnostics => Self::Configs,
        }
    }
}
