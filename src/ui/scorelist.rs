use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::vec::Vec;
use pebble::GContext;
use pebble::platform::{is_rect, is_round};
use pebble::std::{TimeInfo, ToCString};
use pebble::system::fonts::{FontKey, GFont};
use pebble::types::{GBitmap, GColor, GEdgeInsets, GPoint, GRect, GSize, GTextAlignment, GTextOverflowMode, MenuIndex, time_t};
use pebble::app_message::AppMessageDict;
use pebble::layer::{AsLayer};
use taconite::layer::{ContentIndicatorLayer, Menu, MenuCallbacks};
use taconite::{ScreenCtx, ScreenFns, ScreenMessageCtx, State, StatusBarLayer, TaconiteMessageKey, handle_list_message};

use crate::{MessageKey, MessageType};
use crate::model::{Game, GameState, League, Sport, Team, TeamState};
use crate::ui::components::leagueheader::{HeaderData, HeaderLayer};

pub struct ScoreListScreen;

pub struct ScoreListState {
    pub league: League,
    pub icon: GBitmap,
    pub games: Vec<Game>,
    pub selected_index: MenuIndex,
}

pub struct PartialGame {
    pub id: i32,
    pub league: League,
    pub timestamp: u32,
    pub state: GameState,

    pub home_team: Option<TeamState>,
    pub away_team: Option<TeamState>,
}

#[derive(Default)]
pub struct ScoreMessageState {
    pub games: Vec<Option<PartialGame>>,
}

pub struct ScoreListLayers {
    header_layer: HeaderLayer,
    menu_layer: Menu<ScoreListState>,
    content_indicator_layer: ContentIndicatorLayer,
    status_bar_layer: StatusBarLayer,
}

impl ScreenFns for ScoreListScreen {
    type State = ScoreListState;
    type Layers = ScoreListLayers;
    type TempState = ScoreMessageState;

    fn create_window(ctx: &ScreenCtx<ScoreListState>) -> Self::Layers {
        let root = ctx.root();
        let bounds = root.get_bounds();

        let header_layer = HeaderLayer::new(bounds, ctx.state().map(|s| HeaderData { title: s.focus(|s| &s.league.name), icon: Some(s.focus(|s| &s.icon)), under_status_bar: s.selected_index.row == 0 }));
        
        let content_indicator_layer = ContentIndicatorLayer::new(bounds, State::fixed(false), ctx.state().map(|s| s.selected_index.row < (s.games.len() as u16 - 1)));

        let status_bar_layer = StatusBarLayer::new();
        status_bar_layer.set_colors(GColor::Clear, GColor::White);

        let menu_height_top_inset = if is_rect() { header_layer.get_bounds().size.h } else { 0 };
        let menu_bounds = bounds.inset(GEdgeInsets::top(menu_height_top_inset));
        let ml = Menu::new(menu_bounds, ctx.state(), MenuCallbacks {
            get_num_rows: Some(Box::new(|_ml, ctx, _section_index| ctx.games.len() as u16)),
            draw_row: Some(Box::new(|_ml, gctx, cell, index, state| {
                if let Some(game) = state.games.get(index.row_idx()) {
                    let cell_bounds = cell.get_bounds();
                    let is_selected = cell.is_highlighted();

                    if is_selected || is_rect() {
                        draw_large_row(gctx, cell_bounds, game);
                    } else {
                        draw_small_row(gctx, cell_bounds, game);
                    }
                };
            })),
            get_cell_height: Some(Box::new(|ml, _ctx, index| match is_rect() {
                true => 58,
                false => match ml.is_index_selected(index) {
                    true => 66,
                    false => 26,
                }
            })),
            select_click: Some(Box::new(|_ml, index| {
            })),
            selection_changed: Some(Box::new({
                let state = ctx.state();
                move |_ml, _old_index: MenuIndex, new_index: MenuIndex| {
                    state.update(|s| s.selected_index = new_index);
                }
            })),
            ..MenuCallbacks::default()
        });

        ml.set_highlight_colors(GColor::DukeBlue, GColor::White);
        // ml.set_center_focused(true);
        ml.set_click_config_onto_window(ctx.window());
        root.add_child(&ml);
        root.add_child(&content_indicator_layer);
        root.add_child(&header_layer);
        root.add_child(&status_bar_layer);

        ScoreListLayers { 
            header_layer, 
            menu_layer: ml,
            content_indicator_layer,
            status_bar_layer,
        }
    }

    fn view(state: &Self::State, layers: &Self::Layers) {
        layers.header_layer.render();
        layers.menu_layer.render();
        layers.content_indicator_layer.render();
        layers.status_bar_layer.set_hidden(state.selected_index.row != 0);
    }

    fn on_messaging_initialized(ctx: &ScreenCtx<ScoreListState>) {
        // Send our window ID to the phone so it knows where to route replies.
        taconite::send_message(&[
            (taconite::TaconiteMessageKey::WindowId as u32, ctx.window_id as i32), 
            (taconite::TaconiteMessageKey::WindowType as u32, 1 as i32),
            (taconite::TaconiteMessageKey::SubscriptionEvent as u32, taconite::SubscriptionEvent::Subscribe as i32),
            (MessageKey::LeagueId as u32, ctx.state().with(|s| s.league.id) as i32)
        ]);
    }

    fn on_drop(ctx: &ScreenCtx<Self::State>) {
        taconite::send_message(&[
            (taconite::TaconiteMessageKey::WindowId as u32, ctx.window_id as i32), 
            (taconite::TaconiteMessageKey::WindowType as u32, 1 as i32),
            (taconite::TaconiteMessageKey::SubscriptionEvent as u32, taconite::SubscriptionEvent::Unsubscribe as i32),
        ]);
    }

    fn on_message(ctx: &ScreenMessageCtx<ScoreListState, ScoreMessageState>, dict: &AppMessageDict) {
        let message_type = dict.find_i32(MessageKey::MessageType as u32).unwrap_or(-1);
        // pbl_log!("handling app message, type = %d", message_type);

        let ready_to_commit = ctx.update_temp(|s| {
            if message_type == MessageType::GameInfo as i32 {
                handle_list_message(&mut s.games, dict, |dict| {
                    let game_id = dict.find_i32(MessageKey::GameId as u32).unwrap_or(-1);
                    let league = League { 
                        id: dict.find_i32(MessageKey::LeagueId as u32).unwrap_or(-1), 
                        name: dict.find_str(MessageKey::LeagueName as u32).unwrap_or("Untitled League").to_cstring(), 
                        icon: Sport::try_from(dict.find_i32(MessageKey::SportIconIndex as u32).unwrap_or(Sport::Other as i32)).unwrap_or(Sport::Other),
                    };
                    let timestamp = dict.find_i32(MessageKey::GameTimestamp as u32).unwrap_or(0) as u32;
                    let status = dict.find_i32(MessageKey::GameStatus as u32).unwrap_or(0);
                    let state: GameState = match status {
                        2 => GameState::Active { 
                            time: dict.find_str(MessageKey::GameTime as u32).unwrap_or("").to_cstring(), 
                            details: dict.find_str(MessageKey::GameDetails as u32).unwrap_or("").to_cstring()
                        },
                        1 => {
                            let datestamp = create_datestamp(timestamp);
                            GameState::Final { datestamp }
                        },
                        _ => { 
                            let datestamp = create_datestamp(timestamp);
                            let timestamp = create_timestamp(timestamp);
                            GameState::Scheduled { datestamp, timestamp }
                        },
                    };
                    PartialGame { id: game_id, league, timestamp, state, home_team: None, away_team: None }
                });
            } else if message_type == MessageType::GameTeamState as i32 {
                let game_index = dict.find_i32(TaconiteMessageKey::ItemIndex as u32).unwrap_or(-1);
                
                if let Some(game) = s.games.get_mut(game_index as usize).flatten_mut() {
                    let is_home_team = dict.find_i32(MessageKey::GameTeamIndex as u32).map(|i| i == 0).unwrap_or(false);

                    let team_state = TeamState {
                        team: Team {
                            id: dict.find_i32(MessageKey::TeamId as u32).unwrap_or(-1),
                            name: dict.find_str(MessageKey::TeamName as u32).unwrap_or("Untitled Team").to_cstring(),
                            record: dict.find_str(MessageKey::TeamRecord as u32).unwrap_or("").to_cstring(),
                        },
                        score: dict.find_str(MessageKey::TeamScore as u32).unwrap_or("").to_cstring(),
                        posession: dict.find_i32(MessageKey::TeamPosession as u32).map(|b| b == 1).unwrap_or(false),
                    };

                    if is_home_team {
                        game.home_team.replace(team_state);
                    } else {
                        game.away_team.replace(team_state);
                    }
                }
            }
            let ready_to_commit = s.games.iter().all(|o| o.as_ref().is_some_and(|g| g.home_team.is_some() && g.away_team.is_some()));
            ready_to_commit
        });
        if ready_to_commit {
            ctx.commit(|state, temp| {
                pbl_log!("committing score list, len = %zu", temp.games.len());
                state.games = temp.games.iter_mut().map(|o| {
                    let partial = o.take().unwrap();
                    Game { id: partial.id, league: partial.league, timestamp: partial.timestamp, state: partial.state, home_team: partial.home_team.unwrap(), away_team: partial.away_team.unwrap() }
                }).collect();
            })
        }
    }
}

fn draw_large_row(gctx: &mut GContext, cell_bounds: GRect, game: &Game) {
    let font_bold = GFont::get_system(FontKey::GOTHIC_18_BOLD);
    let font_regular = GFont::get_system(FontKey::GOTHIC_14);
    let horz_padding = if is_round() { 16 } else { 8 };
    let vert_padding = if is_round() { 4 } else { 0 };

    let away_team_name_bounds = draw_and_get_bounds(gctx, &game.away_team.team.name, &font_bold, |_| GPoint { x: horz_padding, y: vert_padding }, GTextAlignment::Left, cell_bounds);
    let home_team_name_bounds = draw_and_get_bounds(gctx, &game.home_team.team.name, &font_bold, |_| GPoint { x: horz_padding, y: vert_padding + away_team_name_bounds.size.h }, GTextAlignment::Left, cell_bounds);

    let _away_team_score_bounds = draw_and_get_bounds(gctx, &game.away_team.score, &font_bold, |text_size| GPoint { x: cell_bounds.size.w - text_size.w - horz_padding, y: vert_padding }, GTextAlignment::Right, cell_bounds);
    let _home_team_score_bounds = draw_and_get_bounds(gctx, &game.home_team.score, &font_bold, |text_size| GPoint { x: cell_bounds.size.w - text_size.w - horz_padding, y: vert_padding + away_team_name_bounds.size.h }, GTextAlignment::Right, cell_bounds);

    match &game.state {
        GameState::Final { datestamp } => {
            draw_and_get_bounds(gctx, &datestamp, &font_regular, |_| GPoint { x: horz_padding, y: home_team_name_bounds.origin.y + home_team_name_bounds.size.h }, GTextAlignment::Left, cell_bounds);
            draw_and_get_bounds(gctx, &c"Final".into(), &font_regular, |text_size| GPoint { x: cell_bounds.size.w - text_size.w - horz_padding, y: home_team_name_bounds.origin.y + home_team_name_bounds.size.h }, GTextAlignment::Right, cell_bounds);
        },
        GameState::Scheduled { datestamp, timestamp } => {
            draw_and_get_bounds(gctx, &datestamp, &font_regular, |_| GPoint { x: horz_padding, y: home_team_name_bounds.origin.y + home_team_name_bounds.size.h }, GTextAlignment::Left, cell_bounds);
            draw_and_get_bounds(gctx, &timestamp, &font_regular, |text_size| GPoint { x: cell_bounds.size.w - text_size.w - horz_padding, y: home_team_name_bounds.origin.y + home_team_name_bounds.size.h }, GTextAlignment::Right, cell_bounds);
        },
        GameState::Active { time, details } => {
            draw_and_get_bounds(gctx, &details, &font_regular, |_| GPoint { x: horz_padding, y: home_team_name_bounds.origin.y + home_team_name_bounds.size.h }, GTextAlignment::Left, cell_bounds);
            draw_and_get_bounds(gctx, &time, &font_regular, |text_size| GPoint { x: cell_bounds.size.w - text_size.w - horz_padding, y: home_team_name_bounds.origin.y + home_team_name_bounds.size.h }, GTextAlignment::Right, cell_bounds);
        },
    }

    // TODO: possession

}

fn draw_small_row(gctx: &mut GContext, cell_bounds: GRect, game: &Game) {
    let font_bold = GFont::get_system(FontKey::GOTHIC_18_BOLD);
    let summary = match game.state {
        GameState::Scheduled { .. } => pbl_format!("{} - {}", game.away_team.team.name, game.home_team.team.name).to_cstring(),
        GameState::Final { .. } | GameState::Active { .. } => pbl_format!("{} {} - {} {}", game.away_team.team.name, game.away_team.score, game.home_team.score, game.home_team.team.name).to_cstring(),
    };
    gctx.draw_text(&summary, &font_bold, cell_bounds, GTextOverflowMode::TrailingEllipsis, GTextAlignment::Center);
}

fn draw_and_get_bounds(gctx: &mut GContext, text: &CString, font: &GFont, origin: impl Fn(GSize) -> GPoint, alignment: GTextAlignment, cell_bounds: GRect) -> GRect {
    let size = gctx.measure_text(text, font, cell_bounds.size, Some((GTextOverflowMode::TrailingEllipsis, alignment)));
    let origin = origin(size);
    let bounds = GRect { origin, size };
    gctx.draw_text(&text, &font, bounds, GTextOverflowMode::TrailingEllipsis, GTextAlignment::Left);
    bounds
}

fn create_datestamp(time: time_t) -> CString {
    let ti = TimeInfo::from_local(time);
    let datestamp = ti.format(c"%x");

    // trim the last three year chars (e.g. "/26")
    let truncated_datestamp = match datestamp.char_indices().rev().nth(2) {
        Some((idx, _)) => datestamp[..idx].to_cstring(),
        None => c"".into(), // Returns empty string if total characters < 3
    };
    truncated_datestamp
}

fn create_timestamp(time: time_t) -> CString {
    let ti = TimeInfo::from_local(time);
    let datestamp = ti.format(c"%X");
    // trim the last three second chars (e.g. ":00")
    let truncated_datestamp = match datestamp.char_indices().rev().nth(2) {
        Some((idx, _)) => datestamp[..idx].to_cstring(),
        None => c"".into(), // Returns empty string if total characters < 3
    };

    truncated_datestamp
}