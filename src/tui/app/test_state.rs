use super::{TestMode, TestScope, TestViewState};

impl Default for TestViewState {
    fn default() -> Self {
        Self {
            scope: TestScope::default(),
            mode: TestMode::default(),
            concurrency: 4,
        }
    }
}

impl TestScope {
    pub fn label(self) -> &'static str {
        match self {
            Self::Focused => "focused",
            Self::Filtered => "filtered",
            Self::AllEnabled => "all enabled",
            Self::Failed => "failed",
            Self::Stale => "stale/untested",
        }
    }
}

impl TestMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Tcp => "tcp",
            Self::RealDelay => "real-delay",
            Self::Both => "tcp + real-delay",
        }
    }
}
