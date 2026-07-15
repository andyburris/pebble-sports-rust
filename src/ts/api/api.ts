import { PebbleGame, PebbleGameStatus } from "..";
import { BaseballCompetition, BaseScoreboard, Competition, Competitor, Event, FootballCompetition, League, Sport } from "./types";

export async function getGamesForLeague(league: League): Promise<PebbleGame[]> {
    console.log("getting endpoint")
    const endpoint = getEndpointForLeague(league)
    console.log("endpoint =", endpoint)
    const response: BaseScoreboard = await fetch(endpoint).then(r => r.json()).catch(e => console.error("error getting games from espn API, e =", JSON.stringify(e)))
    console.log("got response")
    try {
        const parsed = response.events.map(e => responseEventToPebbleGame(league, e))
        console.log("parsed, len =", parsed.length)
        return parsed
    } catch (e) {
        console.error("error while parsing =", e)
        throw e
    }
}

function responseEventToPebbleGame(league: League, event: Event): PebbleGame {
    const competition = event.competitions[0]
    const date = new Date(competition.date)
    const rawStatus = competition.status.type.name
    const status: PebbleGameStatus = 
        (rawStatus === "STATUS_SCHEDULED") ? "scheduled"
        : (competition.status.type.completed || competition.status.type.state === "post" || rawStatus === "STATUS_FINAL") ? "final"
        : (competition.status.type.state === "in" || rawStatus === "STATUS_IN_PROGRESS" || rawStatus === "STATUS_DELAYED") ? { 
            time: competition.status.type.shortDetail.replace("- ", ""),
            details: parseGameDetails(league.sport, competition)
        }
        : "scheduled"

    const homeCompetitor = competition.competitors[0] // ESPN lists home team first
    const awayCompetitor = competition.competitors[1]

    function getRecord(competitor: Competitor) {
        return competitor.records?.[0]?.summary ?? (competitor as any).record?.summary ?? ""
    }

    const posessionTeamId = getPossessionTeamId(league.sport, competition)

    // TODO: strip names of special characters that pebble can't handle (like accents, emojis, etc). replace with closest ascii equivalent if possible.
    return {
        id: parseInt(competition.id),
        timestamp: date.getTime(),
        status: status,
        homeTeam: {
            team: { id: parseInt(homeCompetitor.id), name: homeCompetitor.team?.abbreviation ?? homeCompetitor.athlete?.shortName ?? "Home", record: getRecord(homeCompetitor) },
            score: homeCompetitor.score,
            posession: homeCompetitor.id === posessionTeamId,
        },
        awayTeam: {
            team: { id: parseInt(awayCompetitor.id), name: awayCompetitor.team?.abbreviation ?? awayCompetitor.athlete?.shortName ?? "Away", record: getRecord(awayCompetitor) },
            score: awayCompetitor.score,
            posession: awayCompetitor.id === posessionTeamId,
        },

    }
}

function getEndpointForLeague(league: League): string {
    return `https://site.api.espn.com/apis/site/v2/sports/${getSlugForSport(league.sport)}/${league.slug}/scoreboard`
}

function getSlugForSport(sport: Sport): string {
    switch(sport) {
        case Sport.AustralianFootball: return "australian-football"
        case Sport.Baseball: return "baseball"
        case Sport.Basketball: return "basketball"
        case Sport.Cricket: return "cricket"
        case Sport.FieldHockey: return "field-hockey"
        case Sport.Football: return "football"
        case Sport.Golf: return "golf"
        case Sport.Hockey: return "hockey"
        case Sport.Lacrosse: return "lacrosse"
        case Sport.MMA: return "mma"
        case Sport.Racing: return "racing"
        case Sport.Rugby: return "rugby"
        case Sport.RugbyLeague: return "rugby-league"
        case Sport.Soccer: return "soccer"
        case Sport.Tennis: return "tennis"
        case Sport.Volleyball: return "volleyball"
        case Sport.WaterPolo: return "water-polo"
    }
}

function parseGameDetails(sport: Sport, game: Competition): string {
    if(sport === Sport.Baseball) {
        const situation: BaseballCompetition["situation"] = (game as any).situation
        if(!situation) return ""
        return situation.balls + "-" + situation.strikes + ", " + situation.outs + " outs";
    } else if (sport === Sport.Football) {
        const situation: FootballCompetition["situation"] = (game as any).situation
        if(!situation) return ""
        return situation.downDistanceText ?? "";
    } else {
        return ""
    }

}

function getPossessionTeamId(sport: Sport, game: Competition): string | undefined {
    if(sport === Sport.Baseball) {
        const situation: BaseballCompetition["situation"] | undefined = (game as any).situation
        if(!situation) return undefined
        return situation.batter?.athlete.team.id ?? situation.dueUp?.[0]?.athlete.team.id
    } else if (sport === Sport.Football) {
        const situation: FootballCompetition["situation"] | undefined = (game as any).situation
        if(!situation) return undefined
        return situation.posession;
    } else {
        return undefined
    }
}
