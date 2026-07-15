import { DEFAULT_SETTINGS, type PebbleSportsSettings } from "../settings";

export function getInputData(): PebbleSportsSettings {
  const hash = window.location.hash
  if (!hash || hash.length < 2) return DEFAULT_SETTINGS
  return JSON.parse(decodeURIComponent(window.location.hash.substring(1))) as PebbleSportsSettings
}

export function returnToPKJS(updated: PebbleSportsSettings) {
  const url = `pebblejs://close#${encodeURIComponent(JSON.stringify(updated))}`
  window.location.href = url;
}