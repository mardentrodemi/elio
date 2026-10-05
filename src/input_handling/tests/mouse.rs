use super::super::*;
use super::helpers::{
    OpenInSystemCaptureGuard, read_open_capture, temp_path, wait_for_directory_load,
};
use std::fs;

fn left_click(column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

fn left_release(column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

fn entry_hit(index: usize, row: u16) -> EntryHit {
    EntryHit {
        rect: Rect {
            x: 0,
            y: row,
            width: 20,
            height: 1,
        },
        index,
        pane: EntryPane::Active,
    }
}

#[test]
fn drag_offer_without_click_candidate_uses_entry_hit() {
    let root = temp_path("drag-offer-entry-hit");
    fs::create_dir_all(&root).expect("failed to create temp root");
    let item = root.join("item.txt");
    fs::write(&item, "item").expect("failed to write item");

    let mut app = App::new_at(root.clone()).expect("failed to create app");
    wait_for_directory_load(&mut app);
    let index = app
        .file_browser
        .entries
        .iter()
        .position(|entry| entry.path == item)
        .expect("item should be visible");
    app.set_screen_regions(ScreenRegions {
        entry_hits: vec![entry_hit(index, 3)],
        ..ScreenRegions::default()
    });

    assert_eq!(app.take_drag_export_paths_at(2, 3), vec![item]);

    fs::remove_dir_all(root).ok();
}

#[test]
fn drag_offer_outside_entry_exports_nothing() {
    let root = temp_path("drag-offer-outside-entry");
    fs::create_dir_all(&root).expect("failed to create temp root");
    fs::write(root.join("item.txt"), "item").expect("failed to write item");

    let mut app = App::new_at(root.clone()).expect("failed to create app");
    wait_for_directory_load(&mut app);

    assert!(app.take_drag_export_paths_at(0, 0).is_empty());

    fs::remove_dir_all(root).ok();
}

#[test]
fn double_click_opens_clicked_file_not_multi_selection() {
    let root = temp_path("mouse-double-click-file-selection");
    fs::create_dir_all(&root).expect("failed to create temp root");
    let alpha = root.join("alpha.txt");
    let beta = root.join("beta.txt");
    let gamma = root.join("gamma.txt");
    fs::write(&alpha, "alpha").expect("failed to write alpha");
    fs::write(&beta, "beta").expect("failed to write beta");
    fs::write(&gamma, "gamma").expect("failed to write gamma");
    let capture = root.join("capture.txt");
    let _capture_guard = OpenInSystemCaptureGuard::install(capture.clone());

    let mut app = App::new_at(root.clone()).expect("failed to create app");
    wait_for_directory_load(&mut app);
    app.file_browser.selected_paths.insert(alpha.clone());
    app.file_browser.selected_paths.insert(gamma.clone());
    let beta_index = app
        .file_browser
        .entries
        .iter()
        .position(|entry| entry.path == beta)
        .expect("beta should be visible");
    app.set_screen_regions(ScreenRegions {
        entry_hits: vec![entry_hit(beta_index, 1)],
        ..ScreenRegions::default()
    });

    app.handle_event(left_click(1, 1))
        .expect("first click should focus clicked file");
    assert!(!capture.exists());

    app.handle_event(left_click(1, 1))
        .expect("second click should open clicked file");

    let opened = read_open_capture(&capture);
    let opened: Vec<_> = opened.lines().map(str::to_owned).collect();
    assert_eq!(opened, vec![beta.display().to_string()]);
    assert_eq!(app.status, "Opened beta.txt");

    fs::remove_dir_all(root).ok();
}

#[test]
fn double_click_suppresses_drag_from_held_second_click() {
    let root = temp_path("mouse-double-click-drag-suppression");
    fs::create_dir_all(&root).expect("failed to create temp root");
    let beta = root.join("beta.txt");
    fs::write(&beta, "beta").expect("failed to write beta");
    let capture = root.join("capture.txt");
    let _capture_guard = OpenInSystemCaptureGuard::install(capture);

    let mut app = App::new_at(root.clone()).expect("failed to create app");
    wait_for_directory_load(&mut app);
    let beta_index = app
        .file_browser
        .entries
        .iter()
        .position(|entry| entry.path == beta)
        .expect("beta should be visible");
    app.set_screen_regions(ScreenRegions {
        entry_hits: vec![entry_hit(beta_index, 1)],
        ..ScreenRegions::default()
    });

    app.handle_event(left_click(1, 1)).expect("first click");
    app.handle_event(left_release(1, 1)).expect("first release");
    app.handle_event(left_click(1, 1))
        .expect("second click should open clicked file");

    assert!(app.take_drag_export_paths_at(1, 1).is_empty());

    app.handle_event(left_release(1, 1))
        .expect("second release should clear suppression");
    app.input.last_click = None;
    app.handle_event(left_click(1, 1))
        .expect("next click can start a new drag candidate");

    assert_eq!(app.take_drag_export_paths_at(1, 1), vec![beta]);

    fs::remove_dir_all(root).ok();
}

#[test]
fn double_click_enters_clicked_directory_not_multi_selection() {
    let root = temp_path("mouse-double-click-dir-selection");
    fs::create_dir_all(&root).expect("failed to create temp root");
    let child = root.join("child");
    let selected = root.join("selected.txt");
    fs::create_dir_all(&child).expect("failed to create child dir");
    fs::write(&selected, "selected").expect("failed to write selected file");

    let mut app = App::new_at(root.clone()).expect("failed to create app");
    wait_for_directory_load(&mut app);
    app.file_browser.selected_paths.insert(selected);
    let child_index = app
        .file_browser
        .entries
        .iter()
        .position(|entry| entry.path == child)
        .expect("child should be visible");
    app.set_screen_regions(ScreenRegions {
        entry_hits: vec![entry_hit(child_index, 1)],
        ..ScreenRegions::default()
    });

    app.handle_event(left_click(1, 1))
        .expect("first click should focus clicked directory");
    assert_eq!(app.file_browser.cwd, root);

    app.handle_event(left_click(1, 1))
        .expect("second click should enter clicked directory");
    wait_for_directory_load(&mut app);

    assert_eq!(app.file_browser.cwd, child);

    fs::remove_dir_all(root).ok();
}

#[test]
fn chooser_double_click_confirms_clicked_file() {
    let root = temp_path("chooser-mouse-double-click-file");
    fs::create_dir_all(&root).expect("failed to create temp root");
    let alpha = root.join("alpha.txt");
    let beta = root.join("beta.txt");
    fs::write(&alpha, "alpha").expect("failed to write alpha");
    fs::write(&beta, "beta").expect("failed to write beta");

    let mut app = App::new_at(root.clone()).expect("failed to create app");
    wait_for_directory_load(&mut app);
    app.file_browser.selected_paths.insert(alpha);
    app.enable_chooser_mode();
    let beta_index = app
        .file_browser
        .entries
        .iter()
        .position(|entry| entry.path == beta)
        .expect("beta should be visible");
    app.set_screen_regions(ScreenRegions {
        entry_hits: vec![entry_hit(beta_index, 1)],
        ..ScreenRegions::default()
    });

    app.handle_event(left_click(1, 1))
        .expect("first click should focus clicked file");
    assert_eq!(app.chooser_exit(), None);

    app.handle_event(left_click(1, 1))
        .expect("second click should choose clicked file");

    assert!(app.should_quit);
    assert_eq!(
        app.chooser_exit(),
        Some(&ChooserExit::Confirmed(vec![beta]))
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn chooser_double_click_enters_clicked_directory() {
    let root = temp_path("chooser-mouse-double-click-directory");
    let child = root.join("child");
    fs::create_dir_all(&child).expect("failed to create child directory");

    let mut app = App::new_at(root.clone()).expect("failed to create app");
    wait_for_directory_load(&mut app);
    app.enable_chooser_mode();
    let child_index = app
        .file_browser
        .entries
        .iter()
        .position(|entry| entry.path == child)
        .expect("child should be visible");
    app.set_screen_regions(ScreenRegions {
        entry_hits: vec![entry_hit(child_index, 1)],
        ..ScreenRegions::default()
    });

    app.handle_event(left_click(1, 1))
        .expect("first click should focus clicked directory");
    app.handle_event(left_click(1, 1))
        .expect("second click should enter clicked directory");
    wait_for_directory_load(&mut app);

    assert_eq!(app.file_browser.cwd, child);
    assert!(!app.should_quit);
    assert_eq!(app.chooser_exit(), None);

    fs::remove_dir_all(root).ok();
}

fn click_row(row: u16, modifiers: KeyModifiers) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 1,
        row,
        modifiers,
    })
}

fn selected_file_names(app: &App) -> Vec<String> {
    let mut names: Vec<_> = app
        .file_browser
        .selected_paths
        .iter()
        .filter_map(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn show_three_files(label: &str) -> (PathBuf, App) {
    let root = temp_path(label);
    fs::write(root.join("alpha.txt"), "a").expect("write alpha");
    fs::write(root.join("beta.txt"), "b").expect("write beta");
    fs::write(root.join("gamma.txt"), "c").expect("write gamma");
    let mut app = App::new_at(root.clone()).expect("app");
    wait_for_directory_load(&mut app);
    let hits: Vec<_> = ["alpha.txt", "beta.txt", "gamma.txt"]
        .into_iter()
        .enumerate()
        .map(|(row, name)| {
            let index = app
                .file_browser
                .entries
                .iter()
                .position(|entry| entry.name == name)
                .unwrap_or_else(|| panic!("{name} should be visible"));
            entry_hit(index, row as u16 + 1)
        })
        .collect();
    app.set_screen_regions(ScreenRegions {
        entry_hits: hits,
        ..ScreenRegions::default()
    });
    (root, app)
}

#[test]
fn shift_click_selects_from_the_anchor_through_the_clicked_file() {
    let (root, mut app) = show_three_files("mouse-shift-range");

    app.handle_event(click_row(2, KeyModifiers::NONE))
        .expect("click beta");
    app.handle_event(click_row(3, KeyModifiers::SHIFT))
        .expect("shift-click gamma");
    assert_eq!(
        selected_file_names(&app),
        vec!["beta.txt".to_string(), "gamma.txt".to_string()]
    );

    app.handle_event(click_row(1, KeyModifiers::SHIFT))
        .expect("shift-click alpha");
    assert_eq!(
        selected_file_names(&app),
        vec!["alpha.txt".to_string(), "beta.txt".to_string()]
    );
    assert_eq!(
        app.file_browser
            .selected_entry()
            .map(|entry| entry.name.as_str()),
        Some("beta.txt")
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn ctrl_click_toggles_files_one_by_one() {
    let (root, mut app) = show_three_files("mouse-ctrl-toggle");

    app.handle_event(click_row(1, KeyModifiers::CONTROL))
        .expect("ctrl-click alpha");
    app.handle_event(click_row(3, KeyModifiers::CONTROL))
        .expect("ctrl-click gamma");
    assert_eq!(
        selected_file_names(&app),
        vec!["alpha.txt".to_string(), "gamma.txt".to_string()]
    );

    app.handle_event(click_row(1, KeyModifiers::CONTROL))
        .expect("ctrl-click alpha again");
    assert_eq!(selected_file_names(&app), vec!["gamma.txt".to_string()]);

    app.handle_event(click_row(2, KeyModifiers::NONE))
        .expect("plain click clears the selection");
    assert!(app.file_browser.selected_paths.is_empty());
    assert_eq!(
        app.file_browser
            .selected_entry()
            .map(|entry| entry.name.as_str()),
        Some("beta.txt")
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn click_on_empty_space_focuses_the_inactive_pane() {
    let root = temp_path("inactive-pane-empty-click");
    let right = root.join("right");
    fs::create_dir_all(&right).expect("right dir");
    fs::write(root.join("a.txt"), "a").expect("write a");
    let mut app = App::new_at(root.clone()).expect("app");
    wait_for_directory_load(&mut app);
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('\\'),
        KeyModifiers::NONE,
    )))
    .expect("open second pane");
    app.set_dir(right.clone()).expect("open right dir");
    wait_for_directory_load(&mut app);
    assert!(app.secondary_focus_right);
    assert_eq!(app.file_browser.cwd, right);

    app.set_screen_regions(ScreenRegions {
        left_entries_panel: Some(Rect::new(0, 0, 20, 10)),
        right_entries_panel: Some(Rect::new(21, 0, 20, 10)),
        ..ScreenRegions::default()
    });
    let _ = app.file_browser.selected_paths.insert(right.join("a.txt"));
    let _ = app.file_browser.selected_paths.insert(right.join("b.txt"));
    app.handle_event(Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 3,
        row: 4,
        modifiers: KeyModifiers::NONE,
    }))
    .expect("click empty space in the left pane");

    assert!(!app.secondary_focus_right);
    assert_eq!(app.file_browser.cwd, root);
    assert_eq!(
        app.parked_primary
            .as_ref()
            .map(|browser| browser.cwd.as_path()),
        Some(right.as_path())
    );
    assert!(
        app.parked_primary
            .as_ref()
            .is_some_and(|browser| browser.selected_paths.is_empty())
    );

    let _ = app.file_browser.selected_paths.insert(root.join("a.txt"));
    let _ = app
        .file_browser
        .selected_paths
        .insert(root.join("kept.txt"));
    app.handle_event(Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 3,
        row: 4,
        modifiers: KeyModifiers::NONE,
    }))
    .expect("empty click in the focused pane keeps focus");
    assert!(!app.secondary_focus_right);
    assert!(app.file_browser.selected_paths.is_empty());

    app.handle_event(Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 30,
        row: 4,
        modifiers: KeyModifiers::NONE,
    }))
    .expect("click empty space in the right pane");
    assert!(app.secondary_focus_right);
    assert_eq!(app.file_browser.cwd, right);

    fs::remove_dir_all(root).ok();
}

fn hold_shift(app: &mut App) {
    use crossterm::event::ModifierKeyCode;
    app.handle_event(Event::Key(KeyEvent::new_with_kind(
        KeyCode::Modifier(ModifierKeyCode::LeftShift),
        KeyModifiers::NONE,
        KeyEventKind::Press,
    )))
    .expect("shift press");
}

#[test]
fn held_shift_click_selects_from_the_focused_file() {
    let (root, mut app) = show_three_files("mouse-shift-held-click");
    let beta = app
        .file_browser
        .entries
        .iter()
        .position(|entry| entry.name == "beta.txt")
        .expect("beta");
    app.select_index(beta);

    hold_shift(&mut app);
    app.handle_event(Event::Mouse(MouseEvent {
        kind: MouseEventKind::Moved,
        column: 1,
        row: 3,
        modifiers: KeyModifiers::SHIFT,
    }))
    .expect("moving while shift is held");
    assert!(app.file_browser.selected_paths.is_empty());

    app.handle_event(click_row(3, KeyModifiers::NONE))
        .expect("click gamma while shift is held");
    assert_eq!(
        selected_file_names(&app),
        vec!["beta.txt".to_string(), "gamma.txt".to_string()]
    );
    assert_eq!(
        app.file_browser
            .selected_entry()
            .map(|entry| entry.name.as_str()),
        Some("beta.txt")
    );

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('j'),
        KeyModifiers::NONE,
    )))
    .expect("move down from the focused file");
    assert_eq!(
        app.file_browser
            .selected_entry()
            .map(|entry| entry.name.as_str()),
        Some("gamma.txt")
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn space_then_click_selects_through_the_clicked_file() {
    let (root, mut app) = show_three_files("mouse-space-range");
    let beta = app
        .file_browser
        .entries
        .iter()
        .position(|entry| entry.name == "beta.txt")
        .expect("beta");
    app.select_index(beta);
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char(' '),
        KeyModifiers::NONE,
    )))
    .expect("space selects beta");
    assert_eq!(selected_file_names(&app), vec!["beta.txt".to_string()]);

    app.handle_event(click_row(3, KeyModifiers::NONE))
        .expect("click gamma extends the range");
    app.handle_event(left_release(1, 3))
        .expect("release the range click");
    assert_eq!(
        selected_file_names(&app),
        vec!["beta.txt".to_string(), "gamma.txt".to_string()]
    );
    let exported: Vec<_> = app
        .take_drag_export_paths_at(1, 2)
        .iter()
        .filter_map(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .collect();
    let mut exported = exported;
    exported.sort();
    assert_eq!(exported, selected_file_names(&app));

    app.handle_event(click_row(1, KeyModifiers::NONE))
        .expect("later click clears the selection");
    assert!(app.file_browser.selected_paths.is_empty());
    assert_eq!(
        app.file_browser
            .selected_entry()
            .map(|entry| entry.name.as_str()),
        Some("alpha.txt")
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn the_first_space_selection_stays_the_start_of_the_range() {
    let (root, mut app) = show_three_files("mouse-space-first-anchor");
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char(' '),
        KeyModifiers::NONE,
    )))
    .expect("space selects the first file");
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char(' '),
        KeyModifiers::NONE,
    )))
    .expect("space selects the next file");

    app.handle_event(click_row(3, KeyModifiers::NONE))
        .expect("click gamma");
    assert_eq!(
        selected_file_names(&app),
        vec![
            "alpha.txt".to_string(),
            "beta.txt".to_string(),
            "gamma.txt".to_string()
        ]
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn click_on_empty_space_clears_a_multi_selection() {
    let (root, mut app) = show_three_files("mouse-empty-clears-selection");
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char(' '),
        KeyModifiers::NONE,
    )))
    .expect("space selects the first file");
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char(' '),
        KeyModifiers::NONE,
    )))
    .expect("space selects the next file");
    app.handle_event(click_row(3, KeyModifiers::NONE))
        .expect("click gamma extends the range");
    let focused = app
        .file_browser
        .selected_entry()
        .map(|entry| entry.name.clone());
    app.set_screen_regions(ScreenRegions {
        entry_hits: app.input.screen_regions.entry_hits.clone(),
        entries_panel: Some(Rect::new(0, 0, 20, 10)),
        ..ScreenRegions::default()
    });

    app.handle_event(Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 1,
        row: 8,
        modifiers: KeyModifiers::NONE,
    }))
    .expect("click empty space in the file pane");

    assert!(app.file_browser.selected_paths.is_empty());
    assert_eq!(
        app.file_browser
            .selected_entry()
            .map(|entry| entry.name.clone()),
        focused
    );

    fs::remove_dir_all(root).ok();
}
