use alloc::boxed::Box;
use alloc::vec::Vec;
use pebble::platform::is_rect;
use pebble::types::{GBitmap, GColor, MenuIndex};
use pebble::app_message::AppMessageDict;
use pebble::layer::{AsLayer};
use pebble::std::ToCString;
use taconite::layer::{Menu, MenuCallbacks};
use taconite::{ScreenCtx, ScreenFns, ScreenMessageCtx, TaconiteMessageKey, handle_list_message};

use crate::MessageKey;
use crate::model::{League, Sport};
use crate::ui::scorelist::{ScoreListScreen, ScoreListState};

pub struct LeaguesScreen;

pub struct LeaguesState {
    pub icons: [GBitmap; 6],
    pub leagues: Vec<League>,
}

pub struct LeaguesLayers {
    menu_layer: Menu<LeaguesState>,
}

impl ScreenFns for LeaguesScreen {
    type State = LeaguesState;
    type TempState = Vec<Option<League>>;
    type Layers = LeaguesLayers;

    fn create_window(ctx: &ScreenCtx<LeaguesState>) -> Self::Layers {
        let root = ctx.root();
        let bounds = root.get_bounds();
        let ml = Menu::new(bounds, &ctx.state(), MenuCallbacks {
            get_num_rows: Some(Box::new(|_ml, ctx, _section_index| ctx.leagues.len() as u16)),
            draw_row: Some(Box::new(|_ml, gctx, cell, index, state| {
                if let Some(league) = state.leagues.get(index.row_idx()) {
                    // Sport is 1-based (Baseball = 1 … Other = 6); icons is a 0-based
                    // [GBitmap; 6]. Subtract one and use .get() so an unknown sport
                    // yields no icon instead of an out-of-bounds panic.
                    let icon = state.icons.get(league.icon as usize);
                    cell.draw_basic(gctx, Some(&league.name), None, icon);
                }
            })),
            get_cell_height: Some(Box::new(|_ml, _ctx, _index| if is_rect() { 48i16} else { 64i16 })),
            select_click: Some(Box::new({
                let state = ctx.state();
                move |_ml, index| {
                    let Some(selected_league) = state.with(|s| s.leagues.get(index.row_idx()).cloned()) else { return; };
                    let Some(selected_icon) = state.with(|s| s.icons.get(selected_league.icon as usize).cloned()) else { return; };
                    taconite::push_screen::<ScoreListScreen>(ScoreListState { league: selected_league, icon: selected_icon, games: Vec::new(), selected_index: MenuIndex { section: 0, row: 0 } }, true);
                }
            })),
            ..MenuCallbacks::default()
        });

        ml.set_highlight_colors(GColor::DukeBlue, GColor::White);        
        ml.set_click_config_onto_window(ctx.window());
        root.add_child(&ml);

        LeaguesLayers { menu_layer: ml }
    }

    fn view(_state: &Self::State, layers: &Self::Layers) {
        layers.menu_layer.render();
    }

    fn on_messaging_initialized(ctx: &ScreenCtx<LeaguesState>) {
        // Send our window ID to the phone so it knows where to route replies.
        taconite::send_message(&[
            (TaconiteMessageKey::WindowId as u32, ctx.window_id as i32), 
            (TaconiteMessageKey::WindowType as u32, 0 as i32)
        ]);
    }

    fn on_message(ctx: &ScreenMessageCtx<LeaguesState, Vec<Option<League>>>, dict: &AppMessageDict) {
        let is_last = ctx.update_temp(|s| {
            let is_last = handle_list_message(s, dict, |d| {
                let id = d.find_i32(MessageKey::LeagueId as u32).unwrap_or(-1);
                let name = d.find_str(MessageKey::LeagueName as u32).unwrap_or("Untitled League");
                let icon_index = d.find_i32(MessageKey::SportIconIndex as u32).unwrap_or(0);
                League { id: id, name: name.to_cstring(), icon: Sport::try_from(icon_index).unwrap_or(Sport::Other) }
            });
            is_last
        });
        if is_last { ctx.commit(|state, temp| {
            state.leagues = temp.iter_mut().map(|o| o.take().unwrap()).collect();
        }); }
    }
}