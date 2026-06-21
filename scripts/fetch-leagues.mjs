#!/usr/bin/env node
// Step 2: resolve every league slug to its ESPN id + name and generate
// src/ts/api/leagues.ts (the ALL_LEAGUES map, keyed by ESPN id, grouped by
// sport).
//
// Reads scripts/league-slugs.json (run fetch-slugs.mjs first). Fetches each
// league's detail endpoint to read its numeric `id`, then caches the raw
// responses to scripts/league-details.json so re-runs (e.g. while tuning
// ABBREVIATION_MAX_LEN) regenerate instantly without re-hitting the API.
// Requests are rate-limited to stay well under 20/second.
//
// Usage:
//   node scripts/fetch-leagues.mjs            # use cache if present
//   node scripts/fetch-leagues.mjs --refresh  # re-fetch everything from ESPN

import { readFile, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import {
    CORE_BASE,
    SPORTS,
    fetchJson,
    createRateLimiter,
    mapWithConcurrency,
} from "./espn.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const SLUGS_FILE = join(HERE, "league-slugs.json");
const CACHE_FILE = join(HERE, "league-details.json");
const OUT_FILE = join(HERE, "..", "src", "ts", "api", "leagues.ts");

const RPS = 15; // requests/second cap (< the 20/s ESPN budget)
const CONCURRENCY = 8;

// Watch abbreviations longer than this (and that contain a space) get collapsed
// to initials derived from the full name. Tune freely and re-run — with the
// cache in place it regenerates leagues.ts instantly.
const ABBREVIATION_MAX_LEN = 10;

// Full, descriptive name (shown in the phone settings list).
const pickName = (d) => d.name || d.displayName || d.shortName || d.slug;

// Raw short name ESPN gives us, falling back midsizeName -> full name.
const pickRawAbbreviation = (d) => d.abbreviation || d.midsizeName || pickName(d);

// A word we keep whole when collapsing to initials: all-caps acronyms
// (CONMEBOL, UEFA) and pure numbers (10). Otherwise we take the first letter.
function keepWhole(word) {
    if (/^\d+$/.test(word)) return true;
    return word === word.toUpperCase() && word !== word.toLowerCase();
}

// Collapse one segment (no " - " separators) to initials. Intra-word hyphens
// act as word splitters: "Pre-Olympic" -> "PO", "Mitre 10 Cup" -> "M10C".
function collapseSegment(segment) {
    return segment
        .split(/[\s-]+/)
        .filter(Boolean)
        .map((word) => (keepWhole(word) ? word : (word[0] ?? "").toUpperCase()))
        .join("");
}

// Watch-facing short name. Long, multi-word abbreviations collapse to initials,
// but " - " (space-dash-space) acts as a preserved separator with each side
// collapsed independently:
//   "World Cup Qualifying - CONMEBOL" -> "WCQ - CONMEBOL"
//   "CONMEBOL Pre-Olympic Tournament" -> "CONMEBOLPOT"
function shortenAbbreviation(rawAbbreviation, name) {
    const tooLong = rawAbbreviation.length > ABBREVIATION_MAX_LEN;
    const hasSpace = /\s/.test(rawAbbreviation);
    if (!tooLong || !hasSpace) return rawAbbreviation;
    return name
        .split(/\s+-\s+/)
        .map(collapseSegment)
        .join(" - ");
}

// Build a final league entry from a cached raw ESPN response.
function buildEntry(raw) {
    const name = pickName(raw);
    return {
        id: raw.id,
        name,
        abbreviation: shortenAbbreviation(pickRawAbbreviation(raw), name),
        slug: raw.slug,
        sport: raw.sport,
    };
}

async function loadRawDetails(useCache) {
    if (useCache && existsSync(CACHE_FILE)) {
        const raws = JSON.parse(await readFile(CACHE_FILE, "utf8"));
        console.log(`Using cached details for ${raws.length} leagues (--refresh to re-fetch).`);
        return raws;
    }

    const bySport = JSON.parse(await readFile(SLUGS_FILE, "utf8"));
    const sportMeta = new Map(SPORTS.map((s) => [s.slug, s]));
    const jobs = [];
    for (const [sportSlug, { slugs }] of Object.entries(bySport)) {
        const meta = sportMeta.get(sportSlug);
        if (!meta) continue;
        for (const leagueSlug of slugs) {
            jobs.push({ sportSlug, leagueSlug, sport: meta.sport });
        }
    }

    console.log(`Resolving ${jobs.length} leagues at <=${RPS} req/s...`);
    const gate = createRateLimiter(RPS);
    let done = 0;
    const failures = [];

    const fetched = await mapWithConcurrency(jobs, CONCURRENCY, async (job) => {
        await gate();
        const url = `${CORE_BASE}/${job.sportSlug}/leagues/${encodeURIComponent(job.leagueSlug)}`;
        try {
            const d = await fetchJson(url);
            const id = Number(d.id);
            if (!Number.isFinite(id)) throw new Error("missing/invalid id");
            return {
                id,
                name: d.name,
                displayName: d.displayName,
                shortName: d.shortName,
                abbreviation: d.abbreviation,
                midsizeName: d.midsizeName,
                slug: d.slug || job.leagueSlug,
                sport: job.sport,
            };
        } catch (err) {
            failures.push(`${job.sportSlug}/${job.leagueSlug}: ${err.message}`);
            return null;
        } finally {
            if (++done % 50 === 0) console.log(`  ...${done}/${jobs.length}`);
        }
    });

    const raws = fetched.filter(Boolean);
    await writeFile(CACHE_FILE, JSON.stringify(raws, null, 2) + "\n");
    if (failures.length) {
        console.warn(`\n${failures.length} leagues failed to resolve:`);
        for (const f of failures) console.warn(`  ! ${f}`);
    }
    return raws;
}

async function main() {
    const useCache = !process.argv.includes("--refresh");
    const raws = await loadRawDetails(useCache);
    const entries = raws.map(buildEntry);

    // Detect ESPN ids that collide across sports (the map is keyed by id, so a
    // collision would silently drop a league — worth surfacing).
    const seen = new Map();
    const collisions = [];
    for (const l of entries) {
        if (seen.has(l.id)) collisions.push([seen.get(l.id), l]);
        else seen.set(l.id, l);
    }

    await writeOutput(entries);
    const shortened = entries.filter(
        (l) => l.abbreviation !== pickRawAbbreviation(raws.find((r) => r.id === l.id))
    ).length;
    console.log(`\nWrote ${entries.length} leagues -> ${OUT_FILE}`);
    console.log(`Collapsed ${shortened} long abbreviations (ABBREVIATION_MAX_LEN=${ABBREVIATION_MAX_LEN}).`);
    if (collisions.length) {
        console.warn(`\n${collisions.length} ESPN id collision(s) across sports (one entry kept per id):`);
        for (const [a, b] of collisions) {
            console.warn(`  ! id ${a.id}: ${a.sport}/${a.slug} vs ${b.sport}/${b.slug}`);
        }
    }
}

function writeOutput(leagues) {
    // Group by sport (in enum order), de-duped by id, sorted by id within group.
    const groups = new Map(SPORTS.map((s) => [s.sport, []]));
    const usedIds = new Set();
    for (const l of leagues.slice().sort((a, b) => a.id - b.id)) {
        if (usedIds.has(l.id)) continue;
        usedIds.add(l.id);
        groups.get(l.sport)?.push(l);
    }

    const lines = [
        "// AUTO-GENERATED by scripts/fetch-leagues.mjs — do not edit by hand.",
        "// Source: ESPN core API (https://sports.core.api.espn.com). Regenerate with:",
        "//   node scripts/fetch-slugs.mjs && node scripts/fetch-leagues.mjs",
        "",
        'import { League, Sport } from "./types";',
        "",
        "export const ALL_LEAGUES: { [id: number]: League } = {",
    ];

    for (const { sport, enumName } of SPORTS) {
        const entries = groups.get(sport) ?? [];
        if (entries.length === 0) continue;
        lines.push(`    // === ${enumName} ===`);
        for (const l of entries) {
            lines.push(
                `    ${l.id}: { id: ${l.id}, name: ${JSON.stringify(l.name)}, ` +
                `abbreviation: ${JSON.stringify(l.abbreviation)}, ` +
                `slug: ${JSON.stringify(l.slug)}, sport: Sport.${enumName} },`
            );
        }
        lines.push("");
    }

    // drop trailing blank line before the closing brace
    while (lines[lines.length - 1] === "") lines.pop();
    lines.push("}");
    lines.push("");

    return writeFile(OUT_FILE, lines.join("\n"));
}

main().catch((err) => {
    console.error(err);
    process.exit(1);
});
