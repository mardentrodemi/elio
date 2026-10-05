use super::super::App;
use crate::file_browser::{DirectoryHistoryMode, DirectoryLoadCompletion, PendingDirectoryLoad};
use std::path::{Path, PathBuf};

impl App {
    pub(crate) fn secondary_browser_open(&self) -> bool {
        self.parked_primary.is_some()
    }

    /// Directory elio should `cd` into on quit. The original left pane wins
    /// while the preview slot is a second browser.
    pub(crate) fn exit_cwd(&self) -> PathBuf {
        if self.secondary_browser_open() && self.secondary_focus_right {
            self.parked_primary
                .as_ref()
                .map(|browser| browser.cwd.clone())
                .unwrap_or_else(|| self.file_browser.cwd.clone())
        } else {
            self.file_browser.cwd.clone()
        }
    }

    pub(crate) fn show_secondary_browser(&mut self) {
        if self.parked_primary.is_some() {
            return;
        }
        if crate::config::layout()
            .panes
            .is_some_and(|panes| panes.preview == 0)
        {
            self.status = "Preview pane disabled in config".to_string();
            return;
        }
        if self.preview_fullscreen() || !self.preview_visible() {
            self.status = "Preview pane is not available".to_string();
            return;
        }

        let restoring = self.remembered_secondary.is_some();
        let secondary = self
            .remembered_secondary
            .take()
            .unwrap_or_else(|| self.file_browser.fork_for_secondary_pane());
        let primary = std::mem::replace(&mut self.file_browser, secondary);
        self.parked_primary = Some(primary);
        self.secondary_focus_right = true;
        self.reset_directory_watch();
        self.queue_terminal_image_geometry_clear();
        if restoring {
            self.reload_remembered_secondary();
        }
        let key = crate::config::key_bindings().secondary_browser.to_string();
        self.status =
            format!("Second file pane — press {key} to restore preview, Shift+\\ switches focus");
    }

    pub(crate) fn toggle_secondary_browser(&mut self) {
        if self.secondary_browser_open() {
            self.hide_secondary_browser();
        } else {
            self.show_secondary_browser();
        }
    }

    pub(crate) fn hide_secondary_browser(&mut self) {
        let Some(other) = self.parked_primary.take() else {
            return;
        };
        let secondary = if self.secondary_focus_right {
            std::mem::replace(&mut self.file_browser, other)
        } else {
            other
        };
        self.store_secondary_browser(secondary);
        self.secondary_focus_right = false;
        self.reset_directory_watch();
        self.queue_terminal_image_geometry_clear();
        self.refresh_preview();
        self.status = "Preview restored".to_string();
    }

    pub(crate) fn focus_other_file_pane(&mut self) {
        if !self.secondary_browser_open() {
            return;
        }
        self.swap_with_parked_primary();
        self.secondary_focus_right = !self.secondary_focus_right;
        self.reset_directory_watch();
        self.status = if self.secondary_focus_right {
            "Focus: right file pane".to_string()
        } else {
            "Focus: left file pane".to_string()
        };
    }

    /// Exchanges the active browser with the parked primary so a background
    /// result can be applied to the pane it belongs to.
    pub(crate) fn swap_with_parked_primary(&mut self) {
        if let Some(parked) = self.parked_primary.as_mut() {
            std::mem::swap(&mut self.file_browser, parked);
        }
    }

    pub(crate) fn browser_for_visual_pane(
        &self,
        pane: crate::app::EntryPane,
    ) -> Option<&crate::file_browser::FileBrowserState> {
        match pane {
            crate::app::EntryPane::Active => Some(&self.file_browser),
            crate::app::EntryPane::Left => Some(self.left_file_browser()),
            crate::app::EntryPane::Right => self.right_file_browser(),
        }
    }

    pub(crate) fn left_file_browser(&self) -> &crate::file_browser::FileBrowserState {
        if self.secondary_browser_open() && self.secondary_focus_right {
            self.parked_primary.as_ref().unwrap_or(&self.file_browser)
        } else {
            &self.file_browser
        }
    }

    pub(crate) fn right_file_browser(&self) -> Option<&crate::file_browser::FileBrowserState> {
        if !self.secondary_browser_open() {
            return None;
        }
        if self.secondary_focus_right {
            Some(&self.file_browser)
        } else {
            self.parked_primary.as_ref()
        }
    }

    pub(crate) fn visual_pane_is_focused(&self, pane: crate::app::EntryPane) -> bool {
        match pane {
            crate::app::EntryPane::Active => true,
            crate::app::EntryPane::Left => !self.secondary_focus_right,
            crate::app::EntryPane::Right => self.secondary_focus_right,
        }
    }

    /// Re-read the unfocused pane from disk so a cut, paste, or external change
    /// shows up without giving that pane keyboard focus.
    pub(crate) fn reload_inactive_file_pane(&mut self) {
        let Some(browser) = self.parked_primary.as_ref() else {
            return;
        };
        let cwd = browser.cwd.clone();
        let show_hidden = browser.show_hidden || browser.in_trash;
        let sort_mode = browser.sort_mode;
        let selected_path = browser.selected_entry().map(|entry| entry.path.clone());
        let fingerprint = browser.directory_runtime.fingerprint;
        let Ok(snapshot) = crate::filesystem::load_directory_snapshot(
            &cwd,
            show_hidden,
            sort_mode,
            crate::config::ui().folders_first,
        ) else {
            return;
        };
        if snapshot.fingerprint == fingerprint {
            return;
        }
        let Some(browser) = self.parked_primary.as_mut() else {
            return;
        };
        browser.unfiltered_entries = snapshot.entries;
        browser.directory_runtime.fingerprint = snapshot.fingerprint;
        browser.directory_runtime.last_auto_reload_at = std::time::Instant::now();
        browser.directory_count_viewport = None;
        browser.directory_item_count_ready_at = None;
        browser.apply_local_filter();
        browser.selected = selected_path
            .and_then(|path| browser.entries.iter().position(|entry| entry.path == path))
            .unwrap_or(0);
        browser.clamp_selection();
        let stale: Vec<PathBuf> = browser
            .selected_paths
            .iter()
            .filter(|path| {
                !browser
                    .unfiltered_entries
                    .iter()
                    .any(|entry| entry.path == **path)
            })
            .cloned()
            .collect();
        for path in stale {
            browser.selected_paths.remove(&path);
        }
    }

    pub(crate) fn reload_focused_after_drop_into_other(
        &mut self,
        dest_dir: &Path,
        navigating: bool,
    ) {
        if navigating {
            self.reload_inactive_file_pane();
            return;
        }
        let other_is_dest = self
            .parked_primary
            .as_ref()
            .is_some_and(|browser| browser.cwd == dest_dir);
        if other_is_dest {
            let _ = self.queue_directory_reload(false);
        } else {
            self.reload_inactive_file_pane();
        }
    }

    fn reload_remembered_secondary(&mut self) {
        self.file_browser.remember_current_directory_view();
        let reselect = self
            .file_browser
            .selected_entry()
            .map(|entry| entry.path.clone());
        let name = self
            .file_browser
            .selected_entry()
            .map(|entry| entry.name.clone());
        let cwd = self.file_browser.cwd.clone();
        let _ = self.queue_directory_load(PendingDirectoryLoad {
            token: 0,
            target_cwd: cwd.clone(),
            previous_cwd: cwd,
            previous_selected_path: reselect.clone(),
            previous_selection_name: name,
            reselect_path: reselect,
            history_mode: DirectoryHistoryMode::None,
            refresh_search: false,
            completion: DirectoryLoadCompletion::Keep,
        });
    }

    fn store_secondary_browser(&mut self, mut browser: crate::file_browser::FileBrowserState) {
        browser.directory_runtime.watch = None;
        browser.directory_runtime.pending_reload_at = None;
        browser.directory_runtime.pending_load = None;
        browser.directory_runtime.pending_fingerprint_scan = None;
        self.remembered_secondary = Some(browser);
    }
}
