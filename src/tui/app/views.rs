use super::TuiView;

impl TuiView {
    pub fn next(self) -> Self {
        match self {
            Self::Configs => Self::Sources,
            Self::Sources => Self::Configs,
        }
    }
}
