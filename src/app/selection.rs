use super::App;
use crate::chooser::ChooserExit;
use crate::file_browser::{SelectionChange, ViewMode};
use crossterm::event::KeyModifiers;
use std::path::{Path, PathBuf};

impl App {
    pub fn selection_count(&self) -> usize {
        if let Some(overlay) = &self.duplicate_finder.session {
            return overlay.selected_paths.len();
        }
        self.file_browser.selection_count()
    }

    pub(crate) fn selected_paths_sorted(&self) -> Vec<PathBuf> {
        self.file_browser.selected_paths_sorted()
    }

    #[cfg(unix)]
    pub(crate) fn selected_paths_in_selection_order(&self) -> Vec<PathBuf> {
        self.file_browser.selected_paths_in_selection_order()
    }

    pub(crate) fn current_directory_escape_for_paths(&self, paths: &[PathBuf]) -> Option<PathBuf> {
        self.file_browser.current_directory_escape_for_paths(paths)
    }

    pub(crate) fn select_entry_with_mouse(&mut self, index: usize, modifiers: KeyModifiers) {
        if self.file_browser.entries.is_empty() {
            return;
        }
        let index = self.file_browser.clamped_selection_index(index);
        let shift = modifiers.contains(KeyModifiers::SHIFT);
        let control = modifiers.contains(KeyModifiers::CONTROL);
        if shift {
            self.select_range_from_focus(index, control);
            return;
        }
        if control {
            self.toggle_mouse_entry(index);
            return;
        }
        if self.extend_space_range_to(index) {
            return;
        }
        self.file_browser.selected_paths.clear();
        if let Some(path) = self.file_browser.entries.get(index) {
            self.file_browser.selection_anchor = Some(path.path.clone());
        }
        self.select_index(index);
    }

    /// Selects every entry from the focused file through `index`. The keyboard
    /// cursor stays on the focused file.
    fn select_range_from_focus(&mut self, index: usize, keep_existing: bool) {
        let anchor_index = self.file_browser.selected;
        if let Some(entry) = self.file_browser.entries.get(anchor_index) {
            self.file_browser.selection_anchor = Some(entry.path.clone());
        }
        let blocked = if keep_existing {
            self.file_browser.add_selection_range(anchor_index, index)
        } else {
            self.file_browser
                .replace_selection_with_range(anchor_index, index)
        };
        if blocked {
            self.status = "Cannot select nested paths".to_string();
        } else {
            self.status.clear();
        }
    }

    fn toggle_mouse_entry(&mut self, index: usize) {
        let Some(path) = self
            .file_browser
            .entries
            .get(index)
            .map(|entry| entry.path.clone())
        else {
            return;
        };
        match self.file_browser.toggle_selected_path(path.clone()) {
            SelectionChange::NestingConflict => {
                self.status = "Cannot select nested paths".to_string();
            }
            SelectionChange::Inserted | SelectionChange::Removed => {
                self.status.clear();
            }
        }
        self.file_browser.selection_anchor = Some(path);
        self.select_index(index);
    }

    pub(crate) fn toggle_selection(&mut self) {
        let Some(entry) = self.selected_entry() else {
            return;
        };
        let path = entry.path.clone();
        match self.file_browser.toggle_selected_path(path.clone()) {
            SelectionChange::NestingConflict => {
                self.status = "Cannot select nested paths".to_string();
            }
            change @ (SelectionChange::Inserted | SelectionChange::Removed) => {
                self.note_space_range_anchor(&path, matches!(change, SelectionChange::Inserted));
                self.status.clear();
                if self.file_browser.view_mode == ViewMode::List && !self.preview_fullscreen() {
                    self.move_vertical(1);
                }
            }
        }
    }

    fn note_space_range_anchor(&mut self, path: &Path, inserted: bool) {
        if inserted {
            if self.file_browser.space_range_anchor.is_none() {
                self.file_browser.space_range_anchor = Some(path.to_path_buf());
            }
            return;
        }
        if self.file_browser.space_range_anchor.as_deref() == Some(path)
            || self.file_browser.selected_paths.is_empty()
        {
            self.file_browser.space_range_anchor = None;
        }
    }

    fn extend_space_range_to(&mut self, index: usize) -> bool {
        let Some(anchor) = self.file_browser.space_range_anchor.clone() else {
            return false;
        };
        let Some(anchor_index) = self
            .file_browser
            .entries
            .iter()
            .position(|entry| entry.path == anchor)
        else {
            self.file_browser.space_range_anchor = None;
            return false;
        };
        self.file_browser.space_range_anchor = None;
        let blocked = self
            .file_browser
            .replace_selection_with_range(anchor_index, index);
        if blocked {
            self.status = "Cannot select nested paths".to_string();
        } else {
            self.status.clear();
        }
        self.select_index(index);
        true
    }

    pub(crate) fn select_all(&mut self) {
        let blocked = self.file_browser.select_all_visible();
        if blocked {
            self.status = "Cannot select nested paths".to_string();
        } else {
            self.status.clear();
        }
    }

    pub(crate) fn clear_selection(&mut self) {
        if self.file_browser.clear_selection() {
            self.status.clear();
        }
    }

    pub(crate) fn enable_chooser_mode(&mut self) {
        self.chooser.enable();
        self.status = "Chooser mode".to_string();
    }
    pub(crate) fn enable_save_as_mode(&mut self, name: String) {
        self.chooser.enable_save_as(name);
        self.status = "Save as mode".to_string();
    }
    #[allow(dead_code)] // Used by focused chooser contract tests.
    pub(crate) fn enable_portal_chooser_mode(
        &mut self,
        mode: crate::chooser::portal::PortalChooserMode,
    ) -> crate::chooser::portal::ExternalCancellation {
        self.status = mode.status_message().to_string();
        self.chooser.enable_portal(mode)
    }
    #[cfg(any(test, target_os = "linux", target_os = "freebsd"))]
    pub(crate) fn enable_portal_chooser_mode_with_cancellation(
        &mut self,
        mode: crate::chooser::portal::PortalChooserMode,
        cancellation: crate::chooser::portal::ExternalCancellation,
    ) {
        self.status = mode.status_message().to_string();
        self.chooser
            .enable_portal_with_cancellation(mode, cancellation);
    }
    pub(crate) fn save_as_mode(&self) -> bool {
        self.chooser.is_save_as()
    }

    pub(crate) fn take_chooser_exit(&mut self) -> Option<ChooserExit> {
        self.chooser.take_exit()
    }

    pub(crate) fn apply_external_chooser_cancellation(&mut self) -> bool {
        if self.chooser.apply_external_cancellation() {
            self.should_change_directory_on_quit = false;
            self.should_quit = true;
            return true;
        }
        false
    }

    pub(crate) fn chooser_mode(&self) -> bool {
        self.chooser.is_enabled()
    }

    #[cfg(test)]
    pub(crate) fn chooser_exit(&self) -> Option<&ChooserExit> {
        self.chooser.exit()
    }

    pub(crate) fn confirm_chooser(&mut self) {
        if self.chooser.is_save_as() {
            self.open_save_as_prompt();
            return;
        }
        let cwd = self.file_browser.cwd.clone();
        let focused_path = self
            .selected_entry()
            .filter(|entry| {
                !self.chooser.selects_current_directory_when_unmarked() || entry.is_dir()
            })
            .map(|entry| entry.path.clone());
        let selected_paths = self.selected_paths_sorted();
        if self
            .chooser
            .confirm_selection(&cwd, focused_path.as_deref(), selected_paths)
        {
            if self.chooser.exit_is_cancelled() {
                self.should_change_directory_on_quit = false;
            }
            self.should_quit = true;
        } else if let Some(message) = self.chooser.take_rejection() {
            self.status = message.to_string();
        }
    }

    pub(crate) fn confirm_chooser_path(&mut self, path: &Path) {
        if self.chooser.is_save_as() {
            self.open_save_as_prompt();
            return;
        }
        if self.chooser.confirm_path(&self.file_browser.cwd, path) {
            if self.chooser.exit_is_cancelled() {
                self.should_change_directory_on_quit = false;
            }
            self.should_quit = true;
        } else if let Some(message) = self.chooser.take_rejection() {
            self.status = message.to_string();
        }
    }

    pub(crate) fn cancel_chooser(&mut self) {
        if self.chooser.cancel() {
            self.should_change_directory_on_quit = false;
            self.should_quit = true;
        }
    }
}
