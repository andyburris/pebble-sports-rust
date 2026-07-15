import { ALL_LEAGUES } from "./api/leagues"
import { League } from "./api/types"

export type PebbleSportsSettings = {
    leagues: League[],
    options: {
        timeline: "never" | "favorites",
        records: "always" | "never" | "final-only",
    }
}

export function getCurrentSettings(): PebbleSportsSettings {
    const foundSettings = localStorage.getItem("settings")
    if (!foundSettings) {
        saveSettings(DEFAULT_SETTINGS)
        return DEFAULT_SETTINGS
    }
    return JSON.parse(foundSettings)
}

export function saveSettings(settings: PebbleSportsSettings) {
    localStorage.setItem("settings", JSON.stringify(settings))
}

const DEFAULT_LEAGUES: League[] = [
    ALL_LEAGUES[606],
    ALL_LEAGUES[10],
    ALL_LEAGUES[46],
    ALL_LEAGUES[90],
    ALL_LEAGUES[28],
]

export const DEFAULT_SETTINGS: PebbleSportsSettings = {
    leagues: DEFAULT_LEAGUES,
    options: {
        timeline: "never",
        records: "always",
    }
}