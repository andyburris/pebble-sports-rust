use alloc::boxed::Box;
use alloc::vec::Vec;
use pebble::std::ToCString;
use pebble::types::Bitmap;
use pebble::app_message::AppMessageDict;
use pebble::layer::{ILayer, menu_cell_basic_draw};
use taconite::layer::{Menu, MenuCallbacks};
use taconite::{ScreenCtx, ScreenFns, ScreenMessageCtx, TaconiteMessageKey, handle_list_message};

use crate::{MessageKey, MessageType};
use crate::model::{Game, GameState, League, Sport, Team, TeamState};
use crate::ui::components::leagueheader::{HeaderData, HeaderLayer};

pub struct ScoreListScreen;

pub struct ScoreListState {
    pub league: League,
    pub icon: Bitmap,
    pub games: Vec<Game>
}

pub struct PartialGame {
    pub id: i32,
    pub league: League,
    pub timestamp: i32,
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
}

impl ScreenFns for ScoreListScreen {
    type State = ScoreListState;
    type Layers = ScoreListLayers;
    type TempState = ScoreMessageState;

    fn create_window(ctx: &ScreenCtx<ScoreListState>) -> Self::Layers {
        let root = ctx.root();
        let bounds = root.get_bounds();

        let ml = Menu::new(bounds, ctx, MenuCallbacks {
            get_num_rows: Some(Box::new(|ctx, _section_index| ctx.games.len() as u16)),
            draw_row: Some(Box::new(|gctx, cell, index, state| {
                if let Some(game) = state.games.get(index.row_idx()) {
                    menu_cell_basic_draw(gctx, cell, Some(&game.home_team.team.name), None, None);
                } else {
                    menu_cell_basic_draw(gctx, cell, Some(c"no game for some reason"), None, None);
                }
            })),
            get_cell_height: Some(Box::new(|_ctx, _index| 64i16)),
            select_click: Some(Box::new(|state, index| {
            })),
            ..MenuCallbacks::default()
        });


        ml.set_click_config_onto_window(ctx.window());
        root.add_child(&ml);

        let header_layer = HeaderLayer::new(bounds, ctx, |s, draw| {
            let d = HeaderData { title: s.league.name.clone(), icon: Some(s.icon.internal), under_status_bar: false };
            draw(&d);
        });
        root.add_child(&header_layer);


        ScoreListLayers { 
            header_layer, 
            menu_layer: ml 
        }
    }

    fn view(state: &Self::State, layers: &Self::Layers) {
        layers.menu_layer.render();
        layers.header_layer.render();
    }

    fn on_messaging_initialized(ctx: &ScreenCtx<ScoreListState>) {
        // Send our window ID to the phone so it knows where to route replies.
        taconite::send_message(&[
            (taconite::TaconiteMessageKey::WindowId as u32, ctx.window_id as i32), 
            (taconite::TaconiteMessageKey::WindowType as u32, 1 as i32),
            (taconite::TaconiteMessageKey::SubscriptionEvent as u32, taconite::SubscriptionEvent::Subscribe as i32),
            (MessageKey::LeagueId as u32, 10 as i32)
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
        pbl_log!("handling app message, type = %d", message_type);

        let ready_to_commit = ctx.update_temp(|s| {
            if message_type == MessageType::GameInfo as i32 {
                handle_list_message(&mut s.games, dict, |dict| {
                    let game_id = dict.find_i32(MessageKey::GameId as u32).unwrap_or(-1);
                    let league = League { 
                        id: dict.find_i32(MessageKey::LeagueId as u32).unwrap_or(-1), 
                        name: dict.find_str(MessageKey::LeagueName as u32).unwrap_or("Untitled League").to_cstring(), 
                        icon: Sport::try_from(dict.find_i32(MessageKey::SportIconIndex as u32).unwrap_or(Sport::Other as i32)).unwrap_or(Sport::Other),
                    };
                    let timestamp: i32 = dict.find_i32(MessageKey::GameTimestamp as u32).unwrap_or(-1);
                    let status = dict.find_i32(MessageKey::GameStatus as u32).unwrap_or(0);
                    let state: GameState = match status {
                        2 => GameState::Active { 
                            time: dict.find_str(MessageKey::GameTime as u32).unwrap_or("").to_cstring(), 
                            details: dict.find_str(MessageKey::GameDetails as u32).unwrap_or("").to_cstring()
                        },
                        1 => GameState::Final,
                        _ => GameState::Scheduled,
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