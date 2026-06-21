// Shared helpers for the ESPN league-scraping scripts.
//
// We use ESPN's "core" API as the source of truth rather than scraping the
// markdown docs at https://github.com/pseudo-r/Public-ESPN-API. The core API
// lists every league slug per sport AND exposes each league's numeric ESPN id,
// so a single pipeline gives us slugs + ids + names and stays current.

export const CORE_BASE = "https://sports.core.api.espn.com/v2/sports";

// The 17 sports the documentation covers, mapped to the numeric values of the
// `Sport` enum in src/ts/api/types.ts. `enumName` is only used to emit readable
// section comments in the generated leagues.ts. Keep this in sync with types.ts.
export const SPORTS = [
    { slug: "australian-football", sport: 0,  enumName: "AustralianFootball" },
    { slug: "baseball",            sport: 1,  enumName: "Baseball" },
    { slug: "basketball",          sport: 2,  enumName: "Basketball" },
    { slug: "cricket",             sport: 3,  enumName: "Cricket" },
    { slug: "field-hockey",        sport: 4,  enumName: "FieldHockey" },
    { slug: "football",            sport: 5,  enumName: "Football" },
    { slug: "golf",                sport: 6,  enumName: "Golf" },
    { slug: "hockey",              sport: 7,  enumName: "Hockey" },
    { slug: "lacrosse",            sport: 8,  enumName: "Lacrosse" },
    { slug: "mma",                 sport: 9,  enumName: "MMA" },
    { slug: "racing",              sport: 10, enumName: "Racing" },
    { slug: "rugby",               sport: 11, enumName: "Rugby" },
    { slug: "rugby-league",        sport: 12, enumName: "RugbyLeague" },
    { slug: "soccer",              sport: 13, enumName: "Soccer" },
    { slug: "tennis",              sport: 14, enumName: "Tennis" },
    { slug: "volleyball",          sport: 15, enumName: "Volleyball" },
    { slug: "water-polo",          sport: 16, enumName: "WaterPolo" },
];

export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// Fetch JSON with a few retries for transient network / 5xx errors.
export async function fetchJson(url, { retries = 3 } = {}) {
    let lastErr;
    for (let attempt = 0; attempt <= retries; attempt++) {
        try {
            const res = await fetch(url, { headers: { accept: "application/json" } });
            if (!res.ok) throw new Error(`HTTP ${res.status} for ${url}`);
            return await res.json();
        } catch (err) {
            lastErr = err;
            if (attempt < retries) await sleep(250 * (attempt + 1));
        }
    }
    throw lastErr;
}

// A spacing-based rate limiter: every call to the returned gate resolves at
// least `1000 / rps` ms after the previous one, so request *starts* never
// exceed `rps` per second no matter how much concurrency we run.
export function createRateLimiter(rps) {
    const interval = 1000 / rps;
    let next = 0;
    return async () => {
        const now = Date.now();
        const wait = Math.max(0, next - now);
        next = Math.max(now, next) + interval;
        if (wait > 0) await sleep(wait);
    };
}

// Run `fn` over `items` with a bounded number of concurrent workers.
export async function mapWithConcurrency(items, concurrency, fn) {
    const results = new Array(items.length);
    let cursor = 0;
    const worker = async () => {
        while (true) {
            const i = cursor++;
            if (i >= items.length) return;
            results[i] = await fn(items[i], i);
        }
    };
    await Promise.all(Array.from({ length: Math.min(concurrency, items.length) }, worker));
    return results;
}
