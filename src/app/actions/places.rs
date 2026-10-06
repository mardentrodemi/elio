use super::super::App;
use crate::places::PlaceTabChange;

impl App {
    pub(crate) fn toggle_place_tab(&mut self) {
        let cwd = self.file_browser.cwd.clone();
        self.status = match self.places.toggle_session_tab(&cwd) {
            PlaceTabChange::Added { title } => format!("Pinned {title} in Places"),
            PlaceTabChange::Removed { title } => format!("Removed {title} from Places"),
            PlaceTabChange::AlreadyListed { title } => format!("{title} is already in Places"),
        };
    }
}
