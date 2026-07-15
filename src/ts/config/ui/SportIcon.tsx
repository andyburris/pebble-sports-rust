import baseball from "@phosphor-icons/core/assets/regular/baseball.svg?raw"
import basketball from "@phosphor-icons/core/assets/regular/basketball.svg?raw"
import boxingGlove from "@phosphor-icons/core/assets/regular/boxing-glove.svg?raw"
import coin from "@phosphor-icons/core/assets/regular/coin.svg?raw"
import flagCheckered from "@phosphor-icons/core/assets/regular/flag-checkered.svg?raw"
import football from "@phosphor-icons/core/assets/regular/football.svg?raw"
import soccerBall from "@phosphor-icons/core/assets/regular/soccer-ball.svg?raw"
import tennisBall from "@phosphor-icons/core/assets/regular/tennis-ball.svg?raw"
import trophy from "@phosphor-icons/core/assets/regular/trophy.svg?raw"
import volleyball from "@phosphor-icons/core/assets/regular/volleyball.svg?raw"
import { Sport } from "../../api/types"
import { Icon } from "./Icon"

// Web-specific Sport -> icon map. Intentionally NOT reusing getIconForSport() from
// api/types.ts, which collapses everything into the watch's 5 PNG slots; here we have
// the full Phosphor set to draw on. Hockey uses Coin as a puck (per request); sports
// without a good Phosphor equivalent fall back to Trophy.
const ICONS: Record<Sport, string> = {
  [Sport.Baseball]: baseball,
  [Sport.Basketball]: basketball,
  [Sport.Football]: football,
  [Sport.AustralianFootball]: football,
  [Sport.Rugby]: football,
  [Sport.RugbyLeague]: football,
  [Sport.Hockey]: coin,
  [Sport.Soccer]: soccerBall,
  [Sport.Tennis]: tennisBall,
  [Sport.Volleyball]: volleyball,
  [Sport.Racing]: flagCheckered,
  [Sport.MMA]: boxingGlove,
  [Sport.Cricket]: trophy,
  [Sport.FieldHockey]: trophy,
  [Sport.Golf]: trophy,
  [Sport.Lacrosse]: trophy,
  [Sport.WaterPolo]: trophy,
}

export function SportIcon({ sport, size, class: className }: { sport: Sport, size?: number, class?: string }) {
  return <Icon svg={ICONS[sport] ?? trophy} size={size} class={className} />
}
