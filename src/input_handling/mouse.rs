use super::*;

const HELP_WHEEL_LINES: isize = 2;
const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(450);

impl App {
    pub(crate) fn remember_drag_candidate(&mut self, path: PathBuf) {
        self.file_browser.remember_drag_candidate(path);
    }

    #[cfg(unix)]
    pub(crate) fn clear_drag_candidate(&mut self) {
        self.file_browser.clear_drag_candidate();
    }

    pub(crate) fn clear_drag_state(&mut self) {
        self.file_browser.clear_drag_state();
    }

    pub(crate) fn suppress_drag_until_button_up(&mut self) {
        self.file_browser.suppress_drag_until_button_up();
        self.input.cross_pane_drag = None;
    }

    #[cfg(any(unix, test))]
    pub(crate) fn take_drag_export_paths_at(&mut self, column: u16, row: u16) -> Vec<PathBuf> {
        let fallback_candidate = self.entry_path_at(column, row);
        self.file_browser.take_drag_export_paths(fallback_candidate)
    }

    #[cfg(any(unix, test))]
    fn entry_path_at(&self, column: u16, row: u16) -> Option<PathBuf> {
        let hit = self.entry_hit_at(column, row)?;
        self.path_in_pane(hit.pane, hit.index).map(|(path, _)| path)
    }

    fn entry_hit_at(&self, column: u16, row: u16) -> Option<EntryHit> {
        self.input
            .screen_regions
            .entry_hits
            .iter()
            .find(|hit| hit.rect.contains((column, row).into()))
            .cloned()
    }

    fn path_in_pane(&self, pane: EntryPane, index: usize) -> Option<(PathBuf, bool)> {
        let entry = self.browser_for_visual_pane(pane)?.entries.get(index)?;
        Some((entry.path.clone(), entry.is_dir()))
    }

    pub(crate) fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        if self.chooser.save_as().is_some_and(|save| save.is_open()) {
            return Ok(());
        }
        if self.file_operations.trash_is_open() {
            return self.handle_trash_mouse(mouse);
        }

        if self.file_operations.restore_is_open() {
            return self.handle_restore_mouse(mouse);
        }

        if self.file_operations.archive_password_is_open() {
            return self.handle_archive_password_mouse(mouse);
        }

        if self.file_operations.archive_create_is_open() {
            return self.handle_archive_create_mouse(mouse);
        }

        if self.file_operations.create_is_open() {
            return self.handle_create_mouse(mouse);
        }

        if self.file_operations.rename_is_open() {
            return self.handle_rename_mouse(mouse);
        }

        if self.file_operations.bulk_rename_is_open() {
            return self.handle_bulk_rename_mouse(mouse);
        }

        if self.file_operations.editor_rename_confirm_is_open() {
            return self.handle_editor_rename_confirm_mouse(mouse);
        }

        if self.overlays.goto.is_some() {
            return self.handle_goto_mouse(mouse);
        }

        if self.file_operations.copy_is_open() {
            return self.handle_copy_mouse(mouse);
        }

        if self.overlays.open_with.is_some() {
            return self.handle_open_with_mouse(mouse);
        }

        if self.overlays.help {
            return self.handle_help_mouse(mouse);
        }

        if self.duplicate_finder.session.is_some() {
            return self.handle_duplicate_mouse(mouse);
        }

        if self.fuzzy_finder.search.is_some() {
            return self.handle_search_mouse(mouse);
        }

        if self.preview_fullscreen() {
            match mouse.kind {
                MouseEventKind::ScrollDown => self.handle_wheel_event(mouse, 1),
                MouseEventKind::ScrollUp => self.handle_wheel_event(mouse, -1),
                MouseEventKind::ScrollLeft => self.handle_horizontal_wheel_event(mouse, -1),
                MouseEventKind::ScrollRight => self.handle_horizontal_wheel_event(mouse, 1),
                MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                    self.input.hover_panel = self.panel_target_at(mouse.column, mouse.row);
                    self.update_wheel_target_from_position(mouse.column, mouse.row);
                }
                MouseEventKind::Down(_) | MouseEventKind::Up(_) => self.clear_drag_state(),
            }
            return Ok(());
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.input.cross_pane_drag = None;
                self.clear_drag_state();
                self.update_wheel_target_from_position(mouse.column, mouse.row);
                if let Some(rect) = self.input.screen_regions.back_button
                    && rect.contains((mouse.column, mouse.row).into())
                {
                    return self.go_back();
                }
                if let Some(rect) = self.input.screen_regions.forward_button
                    && rect.contains((mouse.column, mouse.row).into())
                {
                    return self.go_forward();
                }
                if let Some(rect) = self.input.screen_regions.parent_button
                    && rect.contains((mouse.column, mouse.row).into())
                {
                    return self.go_parent();
                }
                if let Some(rect) = self.input.screen_regions.hidden_button
                    && rect.contains((mouse.column, mouse.row).into())
                {
                    self.toggle_hidden_files()?;
                    return Ok(());
                }
                if let Some(rect) = self.input.screen_regions.view_button
                    && rect.contains((mouse.column, mouse.row).into())
                {
                    self.toggle_view_mode();
                    return Ok(());
                }

                if let Some(target) = self
                    .input
                    .screen_regions
                    .sidebar_hits
                    .iter()
                    .find(|hit| hit.rect.contains((mouse.column, mouse.row).into()))
                    .cloned()
                {
                    return self.set_dir(target.path);
                }

                if let Some(hit) = self.entry_hit_at(mouse.column, mouse.row) {
                    let Some((path, is_dir)) = self.path_in_pane(hit.pane, hit.index) else {
                        return Ok(());
                    };
                    let mut modifiers = self.effective_mouse_modifiers(&mouse);
                    let mut shift = modifiers.contains(KeyModifiers::SHIFT);
                    let control = modifiers.contains(KeyModifiers::CONTROL);
                    if self.secondary_browser_open() && !self.visual_pane_is_focused(hit.pane) {
                        self.focus_other_file_pane();
                        modifiers.remove(KeyModifiers::SHIFT);
                        shift = false;
                    }
                    let extend_space_range =
                        !shift && !control && self.file_browser.space_range_anchor.is_some();
                    if shift || control || extend_space_range {
                        self.suppress_drag_until_button_up();
                    } else {
                        let paths = self
                            .browser_for_visual_pane(hit.pane)
                            .map(|browser| browser.paths_for_drag(&path))
                            .unwrap_or_default();
                        self.input.cross_pane_drag = Some(CrossPaneDrag {
                            paths,
                            source: hit.pane,
                            armed: false,
                        });
                        self.remember_drag_candidate(path.clone());
                    }
                    self.select_entry_with_mouse(hit.index, modifiers);
                    if !shift && !control && self.is_double_click(&path) {
                        if self.chooser_mode() && !self.save_as_mode() && !is_dir {
                            self.confirm_chooser_path(&path);
                        } else {
                            self.open_entry_at_index(hit.index)?;
                        }
                        self.suppress_drag_until_button_up();
                    }
                    self.input.last_click = Some(ClickState {
                        path,
                        at: Instant::now(),
                    });
                } else if self.pointer_is_in_file_pane(mouse.column, mouse.row) {
                    self.clear_selection();
                    if self.pointer_is_in_unfocused_file_pane(mouse.column, mouse.row) {
                        self.focus_other_file_pane();
                    }
                }
            }
            MouseEventKind::Up(_) => {
                self.finish_cross_pane_drag(mouse.column, mouse.row)?;
                self.clear_drag_state();
            }
            MouseEventKind::ScrollDown => {
                self.handle_wheel_event(mouse, 1);
            }
            MouseEventKind::ScrollUp => {
                self.handle_wheel_event(mouse, -1);
            }
            MouseEventKind::ScrollLeft => {
                self.handle_horizontal_wheel_event(mouse, -1);
            }
            MouseEventKind::ScrollRight => {
                self.handle_horizontal_wheel_event(mouse, 1);
            }
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                // Track hover panel from Moved events separately. These events come from
                // ?1003h (any-event tracking) and always carry the true cursor position,
                // making hover_panel a reliable routing source when scroll event coordinates
                // are inaccurate (observed in some Alacritty/Ghostty configurations).
                self.input.hover_panel = self.panel_target_at(mouse.column, mouse.row);
                self.update_wheel_target_from_position(mouse.column, mouse.row);
                if matches!(mouse.kind, MouseEventKind::Drag(_))
                    && let Some(drag) = self.input.cross_pane_drag.as_mut()
                {
                    drag.armed = true;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_help_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.clear_wheel_scroll();
                self.overlays.help = false;
            }
            MouseEventKind::ScrollDown => {
                self.scroll_help_by(HELP_WHEEL_LINES);
            }
            MouseEventKind::ScrollUp => {
                self.scroll_help_by(-HELP_WHEEL_LINES);
            }
            _ => {}
        }
        Ok(())
    }

    fn panel_target_at(&self, column: u16, row: u16) -> Option<WheelTarget> {
        if self
            .input
            .screen_regions
            .preview_panel
            .is_some_and(|rect| rect.contains((column, row).into()))
        {
            Some(WheelTarget::Preview)
        } else if self
            .input
            .screen_regions
            .entries_panel
            .is_some_and(|rect| rect.contains((column, row).into()))
        {
            Some(WheelTarget::Entries)
        } else {
            None
        }
    }

    pub(crate) fn update_wheel_target_from_position(&mut self, column: u16, row: u16) {
        if let Some(target) = self.panel_target_at(column, row) {
            self.input.last_wheel_target = Some(target);
        }
    }

    pub(crate) fn resolve_wheel_target(&mut self, column: u16, row: u16) -> Option<WheelTarget> {
        if let Some(target) = self.panel_target_at(column, row) {
            self.input.last_wheel_target = Some(target);
            return Some(target);
        }

        if let Some(preview) = self.input.screen_regions.preview_panel
            && column >= preview.x
        {
            self.input.last_wheel_target = Some(WheelTarget::Preview);
            return self.input.last_wheel_target;
        }

        if let Some(entries) = self.input.screen_regions.entries_panel
            && column >= entries.x
            && column < entries.x.saturating_add(entries.width)
        {
            self.input.last_wheel_target = Some(WheelTarget::Entries);
            return self.input.last_wheel_target;
        }

        self.input.last_wheel_target
    }

    fn pointer_is_in_file_pane(&self, column: u16, row: u16) -> bool {
        let regions = &self.input.screen_regions;
        let point = (column, row).into();
        regions
            .entries_panel
            .is_some_and(|rect| rect.contains(point))
            || regions
                .left_entries_panel
                .is_some_and(|rect| rect.contains(point))
            || regions
                .right_entries_panel
                .is_some_and(|rect| rect.contains(point))
    }

    fn pointer_is_in_unfocused_file_pane(&self, column: u16, row: u16) -> bool {
        if !self.secondary_browser_open() {
            return false;
        }
        let regions = &self.input.screen_regions;
        let pane = if self.secondary_focus_right {
            regions.left_entries_panel
        } else {
            regions.right_entries_panel
        };
        pane.is_some_and(|rect| rect.contains((column, row).into()))
    }

    fn effective_mouse_modifiers(&self, mouse: &MouseEvent) -> KeyModifiers {
        let mut modifiers = mouse.modifiers;
        if self.input.held_modifiers.shift() {
            modifiers |= KeyModifiers::SHIFT;
        }
        if self.input.held_modifiers.control() {
            modifiers |= KeyModifiers::CONTROL;
        }
        modifiers
    }

    fn finish_cross_pane_drag(&mut self, column: u16, row: u16) -> Result<()> {
        let Some(drag) = self.input.cross_pane_drag.take() else {
            return Ok(());
        };
        if !drag.armed || drag.paths.is_empty() {
            return Ok(());
        }
        let Some(dest) = self.cross_pane_drop_directory(column, row, drag.source) else {
            return Ok(());
        };
        self.drop_cut_into_directory(dest, drag.paths)
    }

    fn cross_pane_drop_directory(
        &self,
        column: u16,
        row: u16,
        source: EntryPane,
    ) -> Option<PathBuf> {
        if !self.secondary_browser_open() {
            return None;
        }
        let regions = &self.input.screen_regions;
        let dest = if regions
            .left_entries_panel
            .is_some_and(|rect| rect.contains((column, row).into()))
        {
            EntryPane::Left
        } else if regions
            .right_entries_panel
            .is_some_and(|rect| rect.contains((column, row).into()))
        {
            EntryPane::Right
        } else {
            return None;
        };
        if dest == source {
            return None;
        }
        if let Some(hit) = self.entry_hit_at(column, row)
            && hit.pane == dest
            && let Some((path, true)) = self.path_in_pane(hit.pane, hit.index)
        {
            return Some(path);
        }
        self.browser_for_visual_pane(dest)
            .map(|browser| browser.cwd.clone())
    }

    fn drop_cut_into_directory(&mut self, dest: PathBuf, paths: Vec<PathBuf>) -> Result<()> {
        #[cfg(any(unix, test))]
        {
            let preparation = self.file_operations.prepare_drop(
                &dest,
                paths,
                crate::file_operations::ClipOp::Cut,
            );
            if let Some(request) = preparation.request {
                self.job_scheduler.submit_paste(request);
            }
            self.status = preparation.status;
        }
        #[cfg(not(any(unix, test)))]
        {
            let _ = (dest, paths);
        }
        Ok(())
    }

    pub(crate) fn is_double_click(&self, path: &Path) -> bool {
        self.input
            .last_click
            .as_ref()
            .is_some_and(|click| click.path == path && click.at.elapsed() <= DOUBLE_CLICK_WINDOW)
    }
}
