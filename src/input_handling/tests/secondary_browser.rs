use super::super::*;
use super::helpers::{cleanup_app_temp_root, temp_path, wait_for_directory_load};
use crossterm::event::KeyEventKind;
use ratatui::layout::Rect;
use std::{fs, thread, time::Duration};

fn press_secondary(app: &mut App) {
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('\\'),
        KeyModifiers::NONE,
    )))
    .expect("secondary browser key");
}

#[test]
fn secondary_browser_key_toggles_preview_pane() {
    let root = temp_path("secondary-browser");
    fs::write(root.join("a.txt"), "a").expect("write a");
    fs::write(root.join("b.txt"), "b").expect("write b");
    let mut app = App::new_at(root.clone()).expect("app");
    let primary_cwd = app.file_browser.cwd.clone();
    let primary_selected = app.file_browser.selected;

    press_secondary(&mut app);
    assert!(app.secondary_browser_open());
    assert_eq!(app.parked_primary.as_ref().unwrap().cwd, primary_cwd);
    assert_eq!(app.file_browser.cwd, primary_cwd);
    assert_eq!(
        app.parked_primary.as_ref().unwrap().selected,
        primary_selected
    );

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('j'),
        KeyModifiers::NONE,
    )))
    .expect("move selection");
    assert_ne!(
        app.file_browser.selected,
        app.parked_primary.as_ref().unwrap().selected
    );
    assert_eq!(
        app.parked_primary.as_ref().unwrap().selected,
        primary_selected
    );

    let mut repeat = KeyEvent::new(KeyCode::Char('\\'), KeyModifiers::NONE);
    repeat.kind = KeyEventKind::Repeat;
    app.handle_event(Event::Key(repeat))
        .expect("repeat should keep the second pane");
    assert!(app.secondary_browser_open());

    let mut release = KeyEvent::new(KeyCode::Char('\\'), KeyModifiers::NONE);
    release.kind = KeyEventKind::Release;
    app.handle_event(Event::Key(release))
        .expect("release should keep the second pane");
    assert!(app.secondary_browser_open());

    press_secondary(&mut app);
    assert!(!app.secondary_browser_open());
    assert_eq!(app.file_browser.cwd, primary_cwd);
    assert_eq!(app.file_browser.selected, primary_selected);

    cleanup_app_temp_root(app, root);
}

#[test]
fn shift_backslash_switches_focus_between_file_panes() {
    let root = temp_path("secondary-browser-focus");
    fs::write(root.join("a.txt"), "a").expect("write a");
    fs::write(root.join("b.txt"), "b").expect("write b");
    fs::write(root.join("c.txt"), "c").expect("write c");
    let mut app = App::new_at(root.clone()).expect("app");

    press_secondary(&mut app);
    assert!(app.secondary_focus_right);
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('j'),
        KeyModifiers::NONE,
    )))
    .expect("move right pane");
    let right_selected = app.file_browser.selected;
    let left_selected = app.parked_primary.as_ref().unwrap().selected;
    assert_ne!(right_selected, left_selected);

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('\\'),
        KeyModifiers::SHIFT,
    )))
    .expect("shift backslash");
    assert!(app.secondary_browser_open());
    assert!(!app.secondary_focus_right);
    assert_eq!(app.file_browser.selected, left_selected);
    assert_eq!(
        app.parked_primary.as_ref().unwrap().selected,
        right_selected
    );

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('G'),
        KeyModifiers::NONE,
    )))
    .expect("move left pane");
    let left_after = app.file_browser.selected;
    assert_ne!(left_after, left_selected);
    assert_eq!(
        app.parked_primary.as_ref().unwrap().selected,
        right_selected
    );

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('|'),
        KeyModifiers::NONE,
    )))
    .expect("pipe focuses the other pane");
    assert!(app.secondary_focus_right);
    assert_eq!(app.file_browser.selected, right_selected);
    assert_eq!(app.parked_primary.as_ref().unwrap().selected, left_after);

    cleanup_app_temp_root(app, root);
}

#[test]
fn ctrl_backslash_swaps_the_two_file_panes() {
    let root = temp_path("secondary-browser-swap");
    let left = root.join("left");
    let right = root.join("right");
    fs::create_dir_all(&left).expect("left dir");
    fs::create_dir_all(&right).expect("right dir");
    fs::write(left.join("from-left.txt"), "l").expect("write left");
    fs::write(right.join("from-right.txt"), "r").expect("write right");
    let mut app = App::new_at(left.clone()).expect("app");

    press_secondary(&mut app);
    app.set_dir(right.clone()).expect("open right dir");
    wait_for_directory_load(&mut app);
    assert!(app.secondary_focus_right);
    let focused_selected = app.file_browser.selected;

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('\\'),
        KeyModifiers::CONTROL,
    )))
    .expect("swap panes");

    assert!(!app.secondary_focus_right);
    assert_eq!(app.file_browser.cwd, right);
    assert_eq!(app.file_browser.selected, focused_selected);
    assert_eq!(app.parked_primary.as_ref().unwrap().cwd, left);
    assert_eq!(app.left_file_browser().cwd, right);
    assert_eq!(app.right_file_browser().unwrap().cwd, left);
    assert_eq!(app.exit_cwd(), right);

    let mut repeat = KeyEvent::new(KeyCode::Char('\\'), KeyModifiers::CONTROL);
    repeat.kind = KeyEventKind::Repeat;
    app.handle_event(Event::Key(repeat))
        .expect("repeat should not swap again");
    assert!(!app.secondary_focus_right);
    assert_eq!(app.left_file_browser().cwd, right);

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('\\'),
        KeyModifiers::CONTROL,
    )))
    .expect("swap back");
    assert!(app.secondary_focus_right);
    assert_eq!(app.file_browser.cwd, right);
    assert_eq!(app.parked_primary.as_ref().unwrap().cwd, left);
    assert_eq!(app.left_file_browser().cwd, left);
    assert_eq!(app.right_file_browser().unwrap().cwd, right);
    assert_eq!(app.exit_cwd(), left);

    cleanup_app_temp_root(app, root);
}

#[test]
fn swap_does_nothing_while_the_second_pane_is_closed() {
    let root = temp_path("secondary-browser-swap-closed");
    fs::write(root.join("a.txt"), "a").expect("write a");
    let mut app = App::new_at(root.clone()).expect("app");
    let cwd = app.file_browser.cwd.clone();

    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('\\'),
        KeyModifiers::CONTROL,
    )))
    .expect("swap while closed");

    assert!(!app.secondary_browser_open());
    assert_eq!(app.file_browser.cwd, cwd);

    cleanup_app_temp_root(app, root);
}

#[test]
fn inactive_pane_drops_moved_file_after_cut_and_paste() {
    let root = temp_path("secondary-browser-refresh");
    let left = root.join("left");
    let right = root.join("right");
    fs::create_dir_all(&left).expect("left dir");
    fs::create_dir_all(&right).expect("right dir");
    fs::write(left.join("note.txt"), "x").expect("write note");
    let mut app = App::new_at(left.clone()).expect("app");

    press_secondary(&mut app);
    app.set_dir(right.clone()).expect("open right dir");
    wait_for_directory_load(&mut app);
    assert_eq!(app.file_browser.cwd, right);

    app.focus_other_file_pane();
    assert_eq!(app.file_browser.cwd, left);
    app.cut();
    app.focus_other_file_pane();
    app.paste().expect("paste");
    wait_for_paste(&mut app);

    assert!(right.join("note.txt").is_file());
    assert!(!left.join("note.txt").exists());
    let parked_names: Vec<_> = app
        .parked_primary
        .as_ref()
        .expect("left pane stays open")
        .entries
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    assert!(
        !parked_names.contains(&"note.txt"),
        "inactive pane still lists {parked_names:?}"
    );
    assert!(
        app.file_browser
            .entries
            .iter()
            .any(|entry| entry.name == "note.txt")
    );

    cleanup_app_temp_root(app, root);
}

#[test]
fn drag_between_file_panes_moves_the_file() {
    let root = temp_path("secondary-browser-drag");
    let left = root.join("left");
    let right = root.join("right");
    fs::create_dir_all(&left).expect("left dir");
    fs::create_dir_all(&right).expect("right dir");
    fs::write(left.join("note.txt"), "x").expect("write note");
    let mut app = App::new_at(left.clone()).expect("app");

    press_secondary(&mut app);
    app.set_dir(right.clone()).expect("open right dir");
    wait_for_directory_load(&mut app);
    let left_index = app
        .parked_primary
        .as_ref()
        .expect("left pane")
        .entries
        .iter()
        .position(|entry| entry.name == "note.txt")
        .expect("note in left pane");
    app.set_screen_regions(ScreenRegions {
        entry_hits: vec![EntryHit {
            rect: Rect::new(0, 1, 20, 1),
            index: left_index,
            pane: EntryPane::Left,
        }],
        left_entries_panel: Some(Rect::new(0, 0, 20, 10)),
        right_entries_panel: Some(Rect::new(21, 0, 20, 10)),
        ..ScreenRegions::default()
    });

    let down = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 1,
        row: 1,
        modifiers: KeyModifiers::NONE,
    };
    let drag = MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: 30,
        row: 2,
        modifiers: KeyModifiers::NONE,
    };
    let up = MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: 30,
        row: 2,
        modifiers: KeyModifiers::NONE,
    };
    app.handle_event(Event::Mouse(down)).expect("drag start");
    app.handle_event(Event::Mouse(drag)).expect("drag move");
    app.handle_event(Event::Mouse(up)).expect("drag drop");
    wait_for_paste(&mut app);

    assert!(right.join("note.txt").is_file());
    assert!(!left.join("note.txt").exists());

    cleanup_app_temp_root(app, root);
}

#[test]
fn secondary_pane_reopens_where_it_was_left() {
    let root = temp_path("secondary-browser-memory");
    let first = root.join("first");
    let second = root.join("second");
    fs::create_dir_all(&first).expect("first dir");
    fs::create_dir_all(&second).expect("second dir");
    fs::write(first.join("a.txt"), "a").expect("write a");
    fs::write(first.join("b.txt"), "b").expect("write b");
    let mut app = App::new_at(root.clone()).expect("app");

    press_secondary(&mut app);
    app.set_dir(first.clone()).expect("open first");
    wait_for_directory_load(&mut app);
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char('j'),
        KeyModifiers::NONE,
    )))
    .expect("select next file");
    let selected_name = app
        .file_browser
        .selected_entry()
        .expect("selection")
        .name
        .clone();
    let saved_cwd = app.file_browser.cwd.clone();

    press_secondary(&mut app);
    assert!(!app.secondary_browser_open());
    app.set_dir(second.clone())
        .expect("leave the left pane elsewhere");
    wait_for_directory_load(&mut app);
    assert_eq!(app.file_browser.cwd, second);

    press_secondary(&mut app);
    wait_for_directory_load(&mut app);
    assert!(app.secondary_focus_right);
    assert_eq!(app.file_browser.cwd, saved_cwd);
    assert_eq!(
        app.file_browser
            .selected_entry()
            .map(|entry| entry.name.as_str()),
        Some(selected_name.as_str())
    );
    assert_eq!(
        app.parked_primary.as_ref().map(|browser| &browser.cwd),
        Some(&second)
    );

    press_secondary(&mut app);
    press_secondary(&mut app);
    wait_for_directory_load(&mut app);
    assert_eq!(app.file_browser.cwd, saved_cwd);

    cleanup_app_temp_root(app, root);
}

fn wait_for_paste(app: &mut App) {
    for _ in 0..500 {
        let _ = app.process_background_jobs();
        if app.file_operations.paste_progress().is_none()
            && app.file_browser.directory_runtime.pending_load.is_none()
        {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for paste and directory reload to complete");
}

#[test]
fn secondary_browser_stays_closed_when_preview_is_hidden() {
    let root = temp_path("secondary-browser-hidden");
    fs::write(root.join("a.txt"), "a").expect("write a");
    let mut app = App::new_at(root.clone()).expect("app");
    app.toggle_preview_pane();

    press_secondary(&mut app);
    assert!(!app.secondary_browser_open());
    assert_eq!(app.status, "Preview pane is not available");

    cleanup_app_temp_root(app, root);
}
