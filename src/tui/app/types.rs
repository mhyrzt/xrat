use std::time::Instant;

use crate::tui::data::TuiData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiView {
    Configs,
    Sources,
}

impl TuiView {
    /// Cycle to the next tab. New tabs added here automatically join the
    /// `[`/`]` rotation.
    pub fn next(self) -> Self {
        match self {
            TuiView::Configs => TuiView::Sources,
            TuiView::Sources => TuiView::Configs,
        }
    }

    /// Cycle to the previous tab.
    pub fn prev(self) -> Self {
        match self {
            TuiView::Configs => TuiView::Sources,
            TuiView::Sources => TuiView::Configs,
        }
    }
}

/// The four dashboard cards. `Tab`/`Shift+Tab` cycle focus between them; the
/// focused card scrolls (or, for `Table`, moves the row selection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TuiPanel {
    #[default]
    Table,
    Detail,
    Log,
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TuiLogTab {
    #[default]
    XratEvents,
    ProxyEngine,
    Api,
    Stats,
}

impl TuiLogTab {
    /// Tabs in their bar/cycle order.
    pub const ORDER: [Self; 4] = [Self::XratEvents, Self::ProxyEngine, Self::Api, Self::Stats];

    fn position(self) -> usize {
        Self::ORDER.iter().position(|tab| *tab == self).unwrap_or(0)
    }

    pub fn next(self) -> Self {
        let index = self.position();
        Self::ORDER[(index + 1) % Self::ORDER.len()]
    }

    pub fn prev(self) -> Self {
        let index = self.position();
        Self::ORDER[(index + Self::ORDER.len() - 1) % Self::ORDER.len()]
    }
}

/// Vertical scroll offsets for the scrollable cards. Wrapped in `Cell` so the
/// render pass can clamp them to the live content/viewport sizes it computes.
#[derive(Debug, Default)]
pub struct PanelScroll {
    pub detail: std::cell::Cell<u16>,
    pub log: std::cell::Cell<u16>,
    pub runtime: std::cell::Cell<u16>,
}

#[derive(Debug, Default)]
pub struct PanelViewport {
    pub table: std::cell::Cell<u16>,
    pub detail: std::cell::Cell<u16>,
    pub log: std::cell::Cell<u16>,
    pub runtime: std::cell::Cell<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiAction {
    Quit,
    ShowHelp,
    Back,
    MoveDown,
    MoveUp,
    PageDown,
    PageUp,
    MoveTop,
    MoveBottom,
    BeginSearch,
    SearchInput(char),
    SearchBackspace,
    ClearSearch,
    ConfirmSearch,
    CycleSort,
    CycleFilter,
    CycleProtocolFilter,
    ToggleDeletedFilter,
    FocusNextPanel,
    FocusPrevPanel,
    FocusPanel(TuiPanel),
    StartFocused,
    EnableFocused,
    DisableFocused,
    RestoreFocused,
    RequestDeleteFocused,
    RequestPurgeFocused,
    StartTest(TestScope),
    CancelTestBatch,
    RequestBulk(BulkOp),
    ConfirmBulk,
    CancelBulk,
    RuntimeStop,
    RuntimeRestart,
    NextLogTab,
    PrevLogTab,
    RequestClearEvents,
    RefreshFocusedSource,
    RefreshAllSources,
    OpenRenameModal,
    RequestDeleteSource,
    OpenQrFocused,
    CopyFocused,
    OpenQrApiUrl,
    CopyApiUrl,
    RenameInput(char),
    RenameBackspace,
    RenameSubmit,
    Confirm,
    Cancel,
    NextTab,
    PrevTab,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiConfigCommand {
    Enable(i64),
    Disable(i64),
    Restore(i64),
    SoftDelete(i64),
    Purge(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmKind {
    SoftDeleteConfig(i64),
    PurgeConfig(i64),
    DeleteSource(i64),
    ClearEvents,
}

/// Destructive multi-config operations triggered through the chord keymap.
/// Each variant resolves to a set of config ids and a backing repository bulk
/// call. They are gated by an inline (no-modal) y/n confirm in the key bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulkOp {
    DeleteFailed,
    DeleteFiltered,
    DeleteDisabled,
    PurgeFailed,
    PurgeFilteredDeleted,
    PurgeAllDeleted,
    RestoreFilteredDeleted,
    RestoreAllDeleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulkKind {
    SoftDelete,
    Purge,
    Restore,
}

impl BulkOp {
    pub fn kind(self) -> BulkKind {
        match self {
            BulkOp::DeleteFailed | BulkOp::DeleteFiltered | BulkOp::DeleteDisabled => {
                BulkKind::SoftDelete
            }
            BulkOp::PurgeFailed | BulkOp::PurgeFilteredDeleted | BulkOp::PurgeAllDeleted => {
                BulkKind::Purge
            }
            BulkOp::RestoreFilteredDeleted | BulkOp::RestoreAllDeleted => BulkKind::Restore,
        }
    }

    pub fn verb(self) -> &'static str {
        match self.kind() {
            BulkKind::SoftDelete => "soft-delete",
            BulkKind::Purge => "purge",
            BulkKind::Restore => "restore",
        }
    }

    /// Noun describing the targeted set, used in the confirm prompt and status.
    pub fn target(self) -> &'static str {
        match self {
            BulkOp::DeleteFailed | BulkOp::PurgeFailed => "failed",
            BulkOp::DeleteFiltered => "filtered",
            BulkOp::DeleteDisabled => "disabled",
            BulkOp::PurgeFilteredDeleted | BulkOp::RestoreFilteredDeleted => "filtered deleted",
            BulkOp::PurgeAllDeleted | BulkOp::RestoreAllDeleted => "deleted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmState {
    pub kind: ConfirmKind,
    /// Short prompt rendered inline in the key bar (no modal).
    pub prompt: String,
}

#[derive(Debug, Default)]
pub struct RenameModalState {
    pub source_id: i64,
    pub input: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QrKind {
    Config,
    Source,
    Api,
}

impl QrKind {
    pub fn modal_title(self) -> &'static str {
        match self {
            QrKind::Config => "Config QR",
            QrKind::Source => "Subscription QR",
            QrKind::Api => "API QR",
        }
    }
}

#[derive(Debug, Clone)]
pub struct QrModalState {
    pub kind: QrKind,
    pub label: String,
    pub uri: String,
}

impl QrModalState {
    pub fn new(kind: QrKind, label: impl Into<String>, uri: impl Into<String>) -> Self {
        Self {
            kind,
            label: label.into(),
            uri: uri.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChromeMessage {
    pub text: String,
    pub is_error: bool,
    pub expires_at: Instant,
}

#[derive(Debug)]
pub struct TuiApp {
    pub active_view: TuiView,
    pub focused_panel: TuiPanel,
    pub panel_scroll: PanelScroll,
    pub panel_viewport: PanelViewport,
    pub show_help: bool,
    pub should_quit: bool,
    pub data: TuiData,
    pub config_list: ConfigListState,
    pub source_list: SourceListState,
    pub test_state: TestViewState,
    pub task_state: crate::tui::task::TuiTaskState,
    pub confirm: Option<ConfirmState>,
    /// Armed chord leader (`t`/`d`/`D`/`r`) waiting for its second key.
    pub pending_chord: Option<char>,
    /// Bulk operation awaiting inline y/n confirmation in the key bar.
    pub pending_bulk: Option<BulkOp>,
    pub active_log_tab: TuiLogTab,
    pub rename_modal: Option<RenameModalState>,
    pub qr_modal: Option<QrModalState>,
    pub event_log: Vec<String>,
    pub chrome_message: Option<ChromeMessage>,
    pub needs_full_clear: bool,
    pub testing_config_ids: Vec<i64>,
    pub spinner_tick: usize,
    /// Newer release tag discovered by the startup version check, if any.
    pub latest_version: Option<String>,
    /// Proxy engines probed once at startup (availability + version).
    pub engines: Vec<crate::tui::data::EngineInfo>,
    /// Live traffic-stats ring buffer for the stats tab, reset per session.
    pub stats: crate::tui::data::StatsHistory,
}

#[derive(Debug, Default)]
pub struct ConfigListState {
    pub focused: usize,
    pub search_query: String,
    pub editing_search: bool,
    pub sort: ConfigSort,
    pub filter: ConfigFilter,
    pub protocol_filter: Option<String>,
    pub include_deleted: bool,
    pub source_filter: SourceFilter,
}

/// Which configs the Subscriptions tab is scoping the Configs tab to. Mirrors
/// the focused row in the Subscriptions table: the synthetic "All" and
/// "Orphans" rows,
/// or a concrete subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceFilter {
    #[default]
    All,
    Orphans,
    Source(i64),
}

#[derive(Debug, Default)]
pub struct SourceListState {
    pub focused: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestViewState {
    pub scope: TestScope,
    pub mode: TestMode,
    pub concurrency: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum TestScope {
    Focused,
    Filtered,
    #[default]
    AllEnabled,
    Failed,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum TestMode {
    Tcp,
    RealDelay,
    #[default]
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfigSort {
    #[default]
    RealDelay,
    TcpDelay,
    Id,
    Name,
    Protocol,
    Source,
    LastTested,
    ImportedAt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfigFilter {
    #[default]
    None,
    EnabledOnly,
    FailedOnly,
    HasDelay,
}
