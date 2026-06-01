use super::TuiView;

impl TuiView {
    pub fn label(self) -> &'static str {
        match self {
            Self::Configs => "configs",
            Self::Sources => "sources",
            Self::Tests => "tests",
            Self::Runtime => "runtime",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Self::Configs => "[CONFIGS]",
            Self::Sources => "[SOURCES]",
            Self::Tests => "[TESTS]",
            Self::Runtime => "[RUNTIME]",
        }
    }
}
