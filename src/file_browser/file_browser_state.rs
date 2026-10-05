use super::{
    DirectoryCountViewport, DirectoryHistory, DirectoryItemCountKey, DirectoryRuntime,
    DirectoryViewMemory, LocalFilter, SelectedPaths, ViewMode, git_status::GitStatusState,
    item_dragging::ItemDragState,
};
use crate::filesystem::{Entry, SortMode};
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    time::Instant,
};

pub(crate) struct FileBrowserState {
    pub(crate) folder_sizes: super::folder_sizes::FolderSizes,
    pub(crate) cwd: PathBuf,
    pub(crate) entries: Vec<Entry>,
    pub(crate) unfiltered_entries: Vec<Entry>,
    pub(crate) local_filter: LocalFilter,
    pub(crate) selected: usize,
    pub(crate) scroll_row: usize,
    pub(crate) view_mode: ViewMode,
    pub(crate) zoom_level: u8,
    pub(crate) sort_mode: SortMode,
    pub(crate) show_hidden: bool,
    /// True when the loaded directory is the trash folder.
    /// Set when the directory snapshot completes.
    pub(crate) in_trash: bool,
    pub(crate) directory_history: DirectoryHistory,
    pub(crate) selected_paths: SelectedPaths,
    /// File where a mouse range starts. Shift+click selects through the clicked file.
    pub(crate) selection_anchor: Option<PathBuf>,
    /// First file marked with Space. The next plain click selects through that file.
    pub(crate) space_range_anchor: Option<PathBuf>,
    pub(crate) directory_item_count_cache: HashMap<DirectoryItemCountKey, Option<usize>>,
    pub(crate) directory_item_count_order: VecDeque<DirectoryItemCountKey>,
    pub(crate) directory_count_viewport: Option<DirectoryCountViewport>,
    pub(crate) directory_item_count_ready_at: Option<Instant>,
    pub(crate) directory_view_memory: HashMap<PathBuf, DirectoryViewMemory>,
    pub(crate) directory_runtime: DirectoryRuntime,
    pub(super) git_status: GitStatusState,
    pub(super) item_drag: ItemDragState,
}

impl FileBrowserState {
    pub(crate) fn new(
        cwd: PathBuf,
        start_in_grid: bool,
        zoom_level: u8,
        show_hidden: bool,
    ) -> Self {
        Self {
            folder_sizes: super::folder_sizes::FolderSizes::default(),
            cwd,
            entries: Vec::new(),
            unfiltered_entries: Vec::new(),
            local_filter: LocalFilter::default(),
            selected: 0,
            scroll_row: 0,
            view_mode: ViewMode::from_start_in_grid(start_in_grid),
            zoom_level,
            sort_mode: SortMode::Name,
            show_hidden,
            in_trash: false,
            directory_history: DirectoryHistory::default(),
            selected_paths: SelectedPaths::default(),
            selection_anchor: None,
            space_range_anchor: None,
            directory_item_count_cache: HashMap::new(),
            directory_item_count_order: VecDeque::new(),
            directory_count_viewport: None,
            directory_item_count_ready_at: None,
            directory_view_memory: HashMap::new(),
            directory_runtime: DirectoryRuntime::new(),
            git_status: GitStatusState::new(),
            item_drag: ItemDragState::default(),
        }
    }

    pub(crate) fn selected_entry(&self) -> Option<&Entry> {
        self.entries.get(self.selected)
    }

    /// Listing copy used as the temporary right-hand file pane.
    ///
    /// The copy starts at the same directory, selection, and scroll position.
    /// Directory watches and in-flight load tokens stay with the original so the
    /// second pane cannot consume the primary browser's jobs.
    pub(crate) fn fork_for_secondary_pane(&self) -> Self {
        const LOAD_TOKEN_OFFSET: u64 = 1 << 48;
        let mut forked = Self::new(
            self.cwd.clone(),
            self.view_mode == ViewMode::Grid,
            self.zoom_level,
            self.show_hidden,
        );
        forked.entries = self.entries.clone();
        forked.unfiltered_entries = self.unfiltered_entries.clone();
        forked.local_filter = self.local_filter.clone();
        forked.selected = self.selected;
        forked.scroll_row = self.scroll_row;
        forked.sort_mode = self.sort_mode;
        forked.in_trash = self.in_trash;
        forked.directory_history = self.directory_history.clone();
        forked.selected_paths = self.selected_paths.clone();
        forked.selection_anchor = self.selection_anchor.clone();
        forked.space_range_anchor = self.space_range_anchor.clone();
        forked.directory_item_count_cache = self.directory_item_count_cache.clone();
        forked.directory_item_count_order = self.directory_item_count_order.clone();
        forked.directory_count_viewport = self.directory_count_viewport;
        forked.directory_view_memory = self.directory_view_memory.clone();
        forked.directory_runtime.fingerprint = self.directory_runtime.fingerprint;
        forked.directory_runtime.load_token = self
            .directory_runtime
            .load_token
            .wrapping_add(LOAD_TOKEN_OFFSET);
        forked.directory_runtime.fingerprint_token = self
            .directory_runtime
            .fingerprint_token
            .wrapping_add(LOAD_TOKEN_OFFSET);
        forked.directory_runtime.use_polling_reload = self.directory_runtime.use_polling_reload;
        forked
    }
}
