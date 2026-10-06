use super::super::*;
use super::helpers::{cleanup_app_temp_root, temp_path, wait_for_directory_load};
use crate::places::PlaceRow;
use crossterm::event::KeyEventKind;
use std::fs;

fn press_pin(app: &mut App, key: char) {
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char(key),
        KeyModifiers::SHIFT,
    )))
    .expect("pin key");
}

fn cwd_pin_is_last_place(app: &App) -> bool {
    let cwd = std::fs::canonicalize(&app.file_browser.cwd)
        .unwrap_or_else(|_| app.file_browser.cwd.clone());
    let mut seen = false;
    for row in &app.places.rows {
        match row {
            PlaceRow::Section { .. } => return seen,
            PlaceRow::Item(item) if item.identity_path == cwd => seen = true,
            PlaceRow::Item(_) if seen => return false,
            PlaceRow::Item(_) => {}
        }
    }
    seen
}

fn lists_cwd(app: &App) -> bool {
    let cwd = std::fs::canonicalize(&app.file_browser.cwd)
        .unwrap_or_else(|_| app.file_browser.cwd.clone());
    app.places
        .rows
        .iter()
        .any(|row| row.item().is_some_and(|item| item.identity_path == cwd))
}

#[test]
fn shift_f_pins_and_unpins_the_active_folder_in_places() {
    let root = temp_path("place-tab");
    let folder = root.join("notes");
    fs::create_dir_all(&folder).expect("folder");
    let mut app = App::new_at(folder.clone()).expect("app");
    assert!(!lists_cwd(&app));

    press_pin(&mut app, 'F');
    assert!(cwd_pin_is_last_place(&app));
    assert!(app.status.contains("Pinned"));

    app.places.refresh();
    assert!(cwd_pin_is_last_place(&app));

    let mut repeat = KeyEvent::new(KeyCode::Char('F'), KeyModifiers::SHIFT);
    repeat.kind = KeyEventKind::Repeat;
    app.handle_event(Event::Key(repeat))
        .expect("repeat should keep the pin");
    assert!(cwd_pin_is_last_place(&app));

    press_pin(&mut app, 'F');
    assert!(!lists_cwd(&app));
    assert!(app.status.contains("Removed"));

    cleanup_app_temp_root(app, root);
}

#[test]
fn pinning_another_folder_keeps_the_earlier_pin() {
    let root = temp_path("place-tab-two");
    let first = root.join("first");
    let second = root.join("second");
    fs::create_dir_all(&first).expect("first");
    fs::create_dir_all(&second).expect("second");
    let mut app = App::new_at(first.clone()).expect("app");

    press_pin(&mut app, 'F');
    app.set_dir(second).expect("open second");
    wait_for_directory_load(&mut app);
    press_pin(&mut app, 'F');

    let labels: Vec<&str> = app
        .places
        .rows
        .iter()
        .map(|row| match row {
            PlaceRow::Section { title } => *title,
            PlaceRow::Item(item) => item.title.as_str(),
        })
        .collect();
    let first_at = labels
        .iter()
        .position(|label| *label == "first")
        .expect("first pin");
    let second_at = labels
        .iter()
        .position(|label| *label == "second")
        .expect("second pin");
    assert!(first_at < second_at);
    if let Some(devices_at) = labels.iter().position(|label| *label == "Devices") {
        assert!(second_at < devices_at);
    }

    cleanup_app_temp_root(app, root);
}
