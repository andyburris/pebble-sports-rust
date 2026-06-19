import { League, Sport } from "./types";

export const ALL_LEAGUES: { [id: number]: League } = {
    10: { id: 10, name: "MLB", slug: "mlb", sport: Sport.Baseball },
    46: { id: 46, name: "NBA", slug: "nba", sport: Sport.Basketball },
    90: { id: 90, name: "NHL", slug: "nhl", sport: Sport.Hockey },
    28: { id: 28, name: "NFL", slug: "nfl", sport: Sport.Football },
    770: { id: 770, name: "MLS", slug: "mls", sport: Sport.Soccer },
    851: { id: 851, name: "WTP", slug: "atp", sport: Sport.Tennis },   
}