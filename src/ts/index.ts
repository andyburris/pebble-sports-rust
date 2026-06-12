import { sendItemList } from "./util/messaging"

export enum MessageKey {
    // Leagues screen
    LeagueId = 1,
    LeagueName = 2,
    LeagueIconIndex = 3,

    // Taconite
    taconite_window_id = 1413563215,
    ItemIndex = 1413563216,
    ItemTotal = 1413563217
}

// Signal the watch that the phone is ready; taconite will call
// on_messaging_initialized on all active screens so they send their window IDs.
Pebble.addEventListener('ready', async () => {
    await PebbleTS.sendAppMessage({ taconite_window_id: 0 })
})

// The watch sends its window ID from on_messaging_initialized; reply with the
// example message addressed to that window so taconite routes it to the right screen.
Pebble.addEventListener('appmessage', async (e: any) => {
    const windowId = e.payload.taconite_window_id
    if (windowId !== undefined && windowId !== 0) {
        await handleLeagueMenu(windowId)
    }
})

enum LeagueIcon {
    Baseball = 1,
    Basketball = 2,
    Football = 3,
    Hockey = 4,
    Soccer = 5,
    Other = 6,
}
type League = {
    id: string,
    name: string,
    icon: LeagueIcon,
}
const LEAGUES: League[] = [
    { id: "mlb", name: "MLB", icon: LeagueIcon.Baseball },
    { id: "nba", name: "NBA", icon: LeagueIcon.Basketball },
    { id: "nhl", name: "NHL", icon: LeagueIcon.Hockey },
    { id: "nfl", name: "NFL", icon: LeagueIcon.Football },
]
async function handleLeagueMenu(windowId: number) {
    console.log("sending items")
    await sendItemList(LEAGUES.map(l => ({
        LeagueId: l.id,
        LeagueName: l.name,
        LeagueIconIndex: l.icon
    })), windowId)
}