use crate::tui::data::TuiData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiView {
    Configs,
    Sources,
    Tests,
    Runtime,
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
    ToggleDeletedFilter,
    SelectFocused,
    EnableFocused,
    DisableFocused,
    RestoreFocused,
    RequestDeleteFocused,
    RequestPurgeFocused,
    Confirm,
    Cancel,
    SwitchView(TuiView),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiConfigCommand {
    Select(i64),
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmState {
    pub kind: ConfirmKind,
    pub title: String,
    pub message: String,
}

#[derive(Debug)]
pub struct TuiApp {
    pub active_view: TuiView,
    pub show_help: bool,
    pub should_quit: bool,
    pub status_message: String,
    pub data: TuiData,
    pub config_list: ConfigListState,
    pub source_list: SourceListState,
    pub test_state: TestViewState,
    pub task_state: crate::tui::task::TuiTaskState,
    pub confirm: Option<ConfirmState>,
}

#[derive(Debug, Default)]
pub struct ConfigListState {
    pub focused: usize,
    pub search_query: String,
    pub editing_search: bool,
    pub sort: ConfigSort,
    pub include_deleted: bool,
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
pub enum TestScope {
    Focused,
    Selected,
    Filtered,
    #[default]
    AllEnabled,
    Failed,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    Id,
    Name,
    Protocol,
}
