import { getGamesForLeague } from "./api/api"
import { ALL_LEAGUES } from "./api/leagues"
import { getIconForSport, League } from "./api/types"
import { sendAdvancedList, sendItemList } from "./util/messaging"
import { subscribe, unsubscribe } from "./util/subscription"

export type AppMessageData = Partial<Record<keyof typeof MessageKey, string | number>>
export enum MessageKey {
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

    // Taconite
    taconite_WindowId = 1413563215,
    taconite_WindowType = 1413563216,
    taconite_ItemIndex = 1413563217,
    taconite_ItemTotal = 1413563218,
    taconite_SubscriptionEvent = 1413563219,
}
export enum SubscriptionEvent { Subscribe = 0, Unsubscribe = 1 }

export enum MessageType {
    GameInfo = 1,
    GameTeamState = 2,
}


// Signal the watch that the phone is ready; taconite will call
// on_messaging_initialized on all active screens so they send their window IDs.
Pebble.addEventListener('ready', async () => {
    await PebbleTS.sendAppMessage({ taconite_WindowId: 0 })
})

// The watch sends its window ID from on_messaging_initialized; reply with the
// example message addressed to that window so taconite routes it to the right screen.
Pebble.addEventListener('appmessage', async (e: { payload: AppMessageData }) => {
    if (typeof e.payload.taconite_WindowId === "number" && typeof e.payload.taconite_WindowType === "number") {
        await handleWindowMessage(e.payload.taconite_WindowId, e.payload.taconite_WindowType, e.payload)
    } else {
        console.error(`got an appmessage an invalid windowId or an invalid windowType, payload = ${JSON.stringify(e.payload)}`)
    }
})


async function handleWindowMessage(windowId: number, windowType: number, payload: AppMessageData) {
    console.log("handling message for windowId =", windowId, "windowType =", windowType)
    if(windowType === 0) { return await handleLeagueMenu(windowId) }
    else if(windowType === 1) { return await handleScoreList(windowId, payload); }
}


const ACTIVE_LEAGUES: League[] = [
    ALL_LEAGUES[606],
    ALL_LEAGUES[10],
    ALL_LEAGUES[46],
    ALL_LEAGUES[90],
    ALL_LEAGUES[28],
]
async function handleLeagueMenu(windowId: number) {
    console.log("sending items")
    await sendItemList(ACTIVE_LEAGUES.map(l => ({
        LeagueId: l.id,
        LeagueName: l.abbreviation,
        SportIconIndex: getIconForSport(l.sport),
    })), windowId)
}

async function handleScoreList(windowId: number, payload: AppMessageData) {
    if(payload.taconite_SubscriptionEvent === SubscriptionEvent.Subscribe && payload.LeagueId !== undefined) {
        const leagueId = payload.LeagueId! as number
        const league = ALL_LEAGUES[leagueId]
        if(!league) {
            console.error(`couldn't find league with id = ${leagueId}`)
            return
        }
        subscribe(windowId, 15000, (_initial: boolean) => {
            getGamesForLeague(league)
                .then(games => {
                    // console.log("sending score items =", games.map(g => JSON.stringify(g)))
                    console.log("sending score items with timestamps =", games.map(g => g.timestamp / 10e2))

                    return sendAdvancedList(games, windowId, async (game, send) => {
                        await send({
                            MessageType: MessageType.GameInfo,
                            GameId: game.id,
                            GameTimestamp: game.timestamp / 10e2, // pebble handles timestamps in seconds, js in millis

                            ...(
                                game.status === "scheduled" ? { GameStatus: 0 }
                                : game.status === "final" ? { GameStatus: 1 }
                                : { GameStatus: 2, GameDetails: game.status.details, GameTime: game.status.time }
                            )
                        })
                        await send({
                            MessageType: MessageType.GameTeamState,
                            GameTeamIndex: 0,
                            TeamId: game.homeTeam.team.id,
                            TeamName: game.homeTeam.team.name,
                            TeamRecord: game.homeTeam.team.record,
                            TeamScore: game.homeTeam.score,
                            TeamPosession: game.homeTeam.posession ? 1 : 0
                        })
                        await send({
                            MessageType: MessageType.GameTeamState,
                            GameTeamIndex: 1,
                            TeamId: game.awayTeam.team.id,
                            TeamName: game.awayTeam.team.name,
                            TeamRecord: game.awayTeam.team.record,
                            TeamScore: game.awayTeam.score,
                            TeamPosession: game.awayTeam.posession ? 1 : 0
                        })
                    })
                })
        })
    } else if (payload.taconite_SubscriptionEvent === SubscriptionEvent.Unsubscribe) {
        unsubscribe(windowId)
    } else {
        console.error(`all score list messages from watch must be subscribe with a league id or unsubscribe requests, payload = ${JSON.stringify(payload)}`);
    }
}

export type PebbleGame = {
    id: number,
    timestamp: number,
    status: PebbleGameStatus,

    homeTeam: PebbleTeamState,
    awayTeam: PebbleTeamState,
}
export type PebbleGameStatus = 
    | "scheduled"
    | "final"
    | { time: string, details: string }
export type PebbleTeamState = {
    team: PebbleTeam,
    score: string,
    posession: boolean,
}
export type PebbleTeam = {
    id: number,
    name: string,
    record: string,
}