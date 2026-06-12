use alloc::vec::Vec;
use pebble::types::Bitmap;
use pebble::app_message::AppMessageDict;
use pebble::layer::{ILayer, MenuLayer, TypedMenuCallbacks, menu_cell_basic_draw};
use pebble::std::ToCString;
use pebble::window::Window;
use taconite::{ScreenFns, ScreenHandle, TaconiteMessageKey, handle_list_message};

use crate::MessageKey;
use crate::model::{League, LeagueIcon};

pub struct LeaguesScreen;

pub struct LeaguesState {
    icons: Option<[Bitmap; 6]>,
    leagues: Vec<Option<League>>,
}

impl Default for LeaguesState {
    fn default() -> Self {
        Self { leagues: Vec::new(), icons: None }
    }
}

pub struct LeaguesLayers {
    menu_layer: MenuLayer<ScreenHandle>,
}

impl ScreenFns for LeaguesScreen {
    type State = LeaguesState;
    type Layers = LeaguesLayers;

    fn create_window(window: &Window, handle: *mut ScreenHandle) -> Self::Layers {
        let root = window.get_root_layer();
        let bounds = root.get_bounds();
        let ml = MenuLayer::new(bounds, handle, TypedMenuCallbacks {
            get_num_sections: Some(|_ctx| 1),
            get_num_rows: Some(|ctx, _section| ctx.state::<LeaguesState>().leagues.len() as u16),
            draw_row: Some(|gctx, cell, index, ctx| {
                let state = ctx.state::<LeaguesState>();
                if let Some(league) = state.leagues.get(index.row_idx()).flatten_ref() {
                    menu_cell_basic_draw(gctx, cell, Some(&league.name), None, state.icons.as_ref().map(|icons| icons[league.icon.clone() as usize].internal));
                }
            }),
            get_cell_height: Some(|_ctx: &ScreenHandle, _index| 64i16),
            ..TypedMenuCallbacks::default()
        });
        ml.set_click_config_onto_window(window);
        root.add_child(&ml);

        LeaguesLayers { menu_layer: ml }
    }

    fn view(_state: &Self::State, layers: &Self::Layers) {
        layers.menu_layer.reload_data();
    }

    fn on_create(handle: &mut ScreenHandle) {
        handle.update(|s: &mut LeaguesState| {
            s.icons.replace([
                Bitmap::new(0),
                Bitmap::new(1),
                Bitmap::new(2),
                Bitmap::new(3),
                Bitmap::new(4),
                Bitmap::new(5),
            ]);
            false
        });
    }

    fn on_messaging_initialized(handle: &mut ScreenHandle) {
        // Send our window ID to the phone so it knows where to route replies.
        taconite::send_message(&[(TaconiteMessageKey::WindowId as u32, handle.window_id as i32)]);
    }

    fn on_message(handle: &mut ScreenHandle, dict: &AppMessageDict) {
        handle.update(|s: &mut LeaguesState| {
            let is_last = handle_list_message(&mut s.leagues, dict, |d| {
                let id = d.find_str(MessageKey::LeagueId as u32).unwrap_or("untitled-league");
                let name = d.find_str(MessageKey::LeagueName as u32).unwrap_or("Untitled League");
                let icon_index = d.find_i32(MessageKey::LeagueIconIndex as u32).unwrap_or(6);
                League { id: id.to_cstring(), name: name.to_cstring(), icon: LeagueIcon::try_from(icon_index).unwrap_or(LeagueIcon::Other) }
            });
            is_last
        });
    }
}