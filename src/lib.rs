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

use alloc::vec::Vec;
use pebble::{app, types::GBitmap};
use pebble::app_message::AppMessage;

use crate::ui::leagues::{LeaguesScreen, LeaguesState};


pub enum MessageType {
    GameInfo = 1,
    GameTeamState = 2,
}
pub enum MessageKey {
    // Comms
    MessageType = 0,

    // Sport
    SportIconIndex = 1,
    
    // League
    LeagueId = 2,
    LeagueName = 3,

    // Game
    GameId = 4,
    GameTimestamp = 5,
    GameStatus = 6,
    GameDetails = 7,
    GameTime = 8,
    GameTeamIndex = 9,

    // TeamState
    TeamScore = 10,
    TeamPosession = 11,

    // Team
    TeamId = 12,
    TeamName = 13,
    TeamRecord = 14,
}

// ── App entry point ───────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub fn main() -> isize {
    AppMessage::open(200, 200);
    AppMessage::register_inbox(taconite::message_received);

    let app = app::App::new();
    taconite::push_screen::<LeaguesScreen>(LeaguesState { leagues: Vec::new(), icons: create_sport_icons() }, true);
    app.run_event_loop();

    pbl_log!("Exiting.");
    0
}

// ── Tiny mem* intrinsics ──────────────────────────────────────────────────────
// Compiler-emitted bulk moves (large struct copies, Vec growth, CString clones)
// lower to these C symbols. We disable compiler_builtins' mem feature (see
// .cargo/config.toml) and provide minimal byte-loop versions instead — the
// optimized builtins are ~1.5 KB/0.7 KB each, which we can't afford on aplite.
// The crate is `#![no_builtins]`, so these loops won't be re-lowered back into
// memcpy/memset calls (no recursion).

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        let mut i = 0;
        while i < n {
            *dest.add(i) = *src.add(i);
            i += 1;
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        if (dest as usize) < (src as usize) {
            let mut i = 0;
            while i < n {
                *dest.add(i) = *src.add(i);
                i += 1;
            }
        } else {
            // Overlapping & dest after src: copy backwards.
            let mut i = n;
            while i > 0 {
                i -= 1;
                *dest.add(i) = *src.add(i);
            }
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    unsafe {
        let byte = c as u8;
        let mut i = 0;
        while i < n {
            *dest.add(i) = byte;
            i += 1;
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    unsafe {
        let mut i = 0;
        while i < n {
            let av = *a.add(i);
            let bv = *b.add(i);
            if av != bv {
                return av as i32 - bv as i32;
            }
            i += 1;
        }
    }
    0
}

// On thumb the compiler emits the ARM EABI names (__aeabi_memcpy /
// __aeabi_memmove4 / …), not the plain C ones. compiler_builtins provides *weak*
// versions of these that drag in its large mem impls (~2.3 KB), so define strong
// aliases here to override them and let --gc-sections drop the big ones. The
// alignment-suffixed variants (4/8) share the unaligned implementation.
macro_rules! aeabi_copy_alias {
    ($($name:ident => $imp:ident),* $(,)?) => {$(
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(dest: *mut u8, src: *const u8, n: usize) {
            unsafe { $imp(dest, src, n); }
        }
    )*};
}
aeabi_copy_alias! {
    __aeabi_memcpy => memcpy, __aeabi_memcpy4 => memcpy, __aeabi_memcpy8 => memcpy,
    __aeabi_memmove => memmove, __aeabi_memmove4 => memmove, __aeabi_memmove8 => memmove,
}

// __aeabi_memset takes (dest, n, c) — note the byte and length are swapped
// relative to C memset(dest, c, n). __aeabi_memclr zero-fills.
macro_rules! aeabi_set_alias {
    ($($name:ident),* $(,)?) => {$(
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(dest: *mut u8, n: usize, c: i32) {
            unsafe { memset(dest, c, n); }
        }
    )*};
}
aeabi_set_alias! { __aeabi_memset, __aeabi_memset4, __aeabi_memset8 }

macro_rules! aeabi_clr_alias {
    ($($name:ident),* $(,)?) => {$(
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(dest: *mut u8, n: usize) {
            unsafe { memset(dest, 0, n); }
        }
    )*};
}
aeabi_clr_alias! { __aeabi_memclr, __aeabi_memclr4, __aeabi_memclr8 }

// ── ARM EHABI personality stubs ───────────────────────────────────────────────
// The unwinder (`_Unwind_*` / `__gnu_unwind_*`, ~2.9 KB from libgcc) is reachable
// only through these personality routines, which `.ARM.exidx` tables reference.
// We build with panic=abort + force-unwind-tables=no, so unwinding never happens
// and any residual table entry never actually runs. Providing strong empty stubs
// keeps libgcc's versions (and everything they call) out of the link.

#[unsafe(no_mangle)]
pub extern "C" fn __aeabi_unwind_cpp_pr0() {}
#[unsafe(no_mangle)]
pub extern "C" fn __aeabi_unwind_cpp_pr1() {}
#[unsafe(no_mangle)]
pub extern "C" fn __aeabi_unwind_cpp_pr2() {}

fn create_sport_icons() -> [GBitmap; 6] {
    [
        GBitmap::new(0),
        GBitmap::new(1),
        GBitmap::new(2),
        GBitmap::new(3),
        GBitmap::new(4),
        GBitmap::new(5),
    ]
}