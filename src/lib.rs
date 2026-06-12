#![crate_type = "staticlib"]
#![no_std]
#![no_builtins]
#![feature(option_reference_flattening)]

extern crate alloc;
#[macro_use]
extern crate pebble_rust as pebble;
extern crate taconite;

mod ui;
mod model;

use pebble::app;
use pebble::app_message::AppMessage;

use crate::ui::leagues::LeaguesScreen;

pub enum MessageKey {
    // Leagues screen
    LeagueId = 1,
    LeagueName = 2,
    LeagueIconIndex = 3,
}

// ── App entry point ───────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub fn main() -> isize {
    AppMessage::open(200, 200);
    AppMessage::register_inbox(taconite::message_received);

    let app = app::App::new();
    taconite::push_screen::<LeaguesScreen>(false);
    app.run_event_loop();

    pbl_log!("Exiting.");
    0
}
