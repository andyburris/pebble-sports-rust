#!/usr/bin/env node
// Step 1: discover every league slug for every supported sport.
//
// Hits the ESPN core API `/leagues` listing once per sport (17 requests total)
// and writes the slugs to scripts/league-slugs.json. That file is the input to
// fetch-leagues.mjs and is human-inspectable so you can see what changed.
//
// Usage: node scripts/fetch-slugs.mjs

import { writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { CORE_BASE, SPORTS, fetchJson } from "./espn.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = join(HERE, "league-slugs.json");

const slugFromRef = (ref) => {
    const m = /\/leagues\/([^?/]+)/.exec(ref);
    return m ? decodeURIComponent(m[1]) : null;
};

async function main() {
    const out = {};
    let total = 0;

    for (const { slug: sportSlug, sport, enumName } of SPORTS) {
        const url = `${CORE_BASE}/${sportSlug}/leagues?limit=1000`;
        let count = 0;
        try {
            const data = await fetchJson(url);
            const slugs = (data.items ?? [])
                .map((it) => slugFromRef(it.$ref))
                .filter(Boolean)
                .sort();
            out[sportSlug] = { sport, enumName, slugs };
            count = slugs.length;
        } catch (err) {
            console.error(`  ! ${sportSlug}: ${err.message}`);
            out[sportSlug] = { sport, enumName, slugs: [] };
        }
        total += count;
        console.log(`  ${sportSlug.padEnd(22)} ${count}`);
    }

    await writeFile(OUT, JSON.stringify(out, null, 2) + "\n");
    console.log(`\nWrote ${total} league slugs across ${SPORTS.length} sports -> ${OUT}`);
}

main().catch((err) => {
    console.error(err);
    process.exit(1);
});
