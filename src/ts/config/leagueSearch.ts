import { ALL_LEAGUES } from "../api/leagues"
import { League, Sport } from "../api/types"

// Lowercase and strip everything but [a-z0-9] — removes punctuation AND spaces so the
// haystack is one condensed string. Query tokens are normalized the same way, so
// "water polo" (-> water, polo) and "waterpolo" both match "WaterPolo", and "mens"
// matches "Men's".
const normalize = (s: string) => s.toLowerCase().replace(/[^a-z0-9]/g, "")

// Extensible synonym system: leagues matched by `when` gain the extra `add` keywords in
// their search haystack. Add more rules here as needed.
type KeywordRule = { when: (l: League) => boolean; add: string[] }
const KEYWORD_RULES: KeywordRule[] = [
  { when: (l) => /ncaa/i.test(l.name), add: ["college"] },
  // future examples, ready to enable:
  // { when: (l) => l.sport === Sport.Basketball, add: ["bball"] },
  // { when: (l) => l.sport === Sport.Soccer,      add: ["football"] },
]

const keywordsFor = (l: League) =>
  KEYWORD_RULES.filter((r) => r.when(l)).flatMap((r) => r.add)

// Built once at module load. haystack = normalized concat of every searchable source:
// name, abbreviation, slug, sport name (Sport[num] reverse lookup), and keywords.
const SEARCH_INDEX = Object.values(ALL_LEAGUES).map((league) => ({
  league,
  haystack: [league.name, league.abbreviation, league.slug, Sport[league.sport], ...keywordsFor(league)]
    .map(normalize)
    .join(""),
}))

// A league matches iff EVERY query token is a substring of its haystack (token-wise AND),
// so interrupting words ("NCAA Men's Basketball") don't break a "NCAA Basketball" query.
export function searchLeagues(query: string): League[] {
  const tokens = query.split(/\s+/).map(normalize).filter(Boolean)
  if (!tokens.length) return []
  return SEARCH_INDEX
    .filter(({ haystack }) => tokens.every((t) => haystack.includes(t)))
    .map(({ league }) => league)
}
