use crate::tui::data::TuiData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiView {
    Configs,
    Sources,
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

/// Vertical scroll offsets for the scrollable cards. Wrapped in `Cell` so the
/// render pass can clamp them to the live content/viewport sizes it computes.
#[derive(Debug, Default)]
pub struct PanelScroll {
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
    StartFocused,
    EnableFocused,
    DisableFocused,
    RestoreFocused,
    RequestDeleteFocused,
    RequestPurgeFocused,
    StartTestBatch,
    StartTestAllEnabled,
    StartTestFiltered,
    CancelTestBatch,
    RuntimeStop,
    RuntimeRestart,
    RefreshFocusedSource,
    RefreshAllSources,
    OpenImportModal,
    OpenRenameModal,
    RequestDeleteSource,
    OpenQrFocused,
    CopyFocused,
    OpenQrApiUrl,
    CopyApiUrl,
    ImportInput(char),
    ImportBackspace,
    ImportSubmit,
    RenameInput(char),
    RenameBackspace,
    RenameSubmit,
    Confirm,
    Cancel,
    SwitchView(TuiView),
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmState {
    pub kind: ConfirmKind,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct ImportModalState {
    pub input: String,
    pub error: Option<String>,
}

#[derive(Debug, Default)]
pub struct RenameModalState {
    pub source_id: i64,
    pub input: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QrModalState {
    pub title: String,
    pub uri: String,
}

impl QrModalState {
    pub fn new(title: impl Into<String>, uri: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            uri: uri.into(),
        }
    }
}

#[derive(Debug)]
pub struct TuiApp {
    pub active_view: TuiView,
    pub focused_panel: TuiPanel,
    pub panel_scroll: PanelScroll,
    pub show_help: bool,
    pub should_quit: bool,
    pub status_message: String,
    pub data: TuiData,
    pub config_list: ConfigListState,
    pub source_list: SourceListState,
    pub test_state: TestViewState,
    pub task_state: crate::tui::task::TuiTaskState,
    pub confirm: Option<ConfirmState>,
    pub import_modal: Option<ImportModalState>,
    pub rename_modal: Option<RenameModalState>,
    pub qr_modal: Option<QrModalState>,
    pub event_log: Vec<String>,
    pub needs_full_clear: bool,
    pub testing_config_ids: Vec<i64>,
    pub spinner_tick: usize,
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

/// Which configs the Sources tab is scoping the Configs tab to. Mirrors the
/// focused row in the Sources table: the synthetic "All" and "Orphans" rows,
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
