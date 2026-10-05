mod actions;
mod directory_counts;
mod job_results;
pub(crate) mod preview;
mod screen_regions;
mod selection;
mod state;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use self::actions::open_with::FallbackOpenOutcome;
#[cfg(test)]
pub(crate) use self::directory_counts::DIRECTORY_ITEM_COUNT_IDLE_DELAY;
#[cfg(test)]
pub(crate) use self::preview::HIGH_FREQUENCY_PREVIEW_REFRESH_DELAY;
pub use self::screen_regions::{
    CopyHit, DuplicateHit, EntryHit, EntryPane, GoToHit, OpenWithHit, PathHit, ScreenRegions,
    SearchHit, SearchScope, ViewMetrics,
};
pub use self::state::App;
pub(crate) use self::state::{
    ClickState, CrossPaneDrag, HeldModifierKeys, NavigationRepeatKey, PendingTerminalTask,
    ScrollLane, ScrollState, WheelProfile, WheelTarget,
};
