import caretDown from "@phosphor-icons/core/assets/regular/caret-down.svg?raw";
import caretUp from "@phosphor-icons/core/assets/regular/caret-up.svg?raw";
import magnifyingGlass from "@phosphor-icons/core/assets/regular/magnifying-glass.svg?raw";
import plus from "@phosphor-icons/core/assets/regular/plus.svg?raw";
import x from "@phosphor-icons/core/assets/regular/x.svg?raw";
import { autocompleteLite, useCombobox } from "@szhsin/react-autocomplete";
import { useMemo, useState } from "preact/hooks";
import { League } from "../api/types";
import { type PebbleSportsSettings } from "../settings";
import { getInputData, returnToPKJS } from "./comms";
import { searchLeagues } from "./leagueSearch";
import { Icon } from "./ui/Icon";
import { Logo } from "./ui/Logo";
import { SportIcon } from "./ui/SportIcon";

const MAX_RESULTS = 8

export function App() {
  const [settings, setSettings] = useState<PebbleSportsSettings>(getInputData())

  const setLeagues = (leagues: League[]) => setSettings({ ...settings, leagues })
  const addLeague = (league: League) => setLeagues([...settings.leagues, league])
  const removeLeague = (id: number) => setLeagues(settings.leagues.filter(l => l.id !== id))
  const moveLeague = (index: number, dir: -1 | 1) => {
    const target = index + dir
    if (target < 0 || target >= settings.leagues.length) return
    const next = [...settings.leagues]
    ;[next[index], next[target]] = [next[target], next[index]]
    setLeagues(next)
  }

  return <div class="text-lg bg-white">
    <div class="flex flex-col min-h-screen max-w-2xl mx-auto">
      <div class="flex flex-col px-4 py-4 gap-4">
        <div class="flex flex-col gap-2 pt-4">
          <Logo />
          <h1 class="text-4xl font-bold">Sports</h1>
        </div>
        <LeaguesList leagues={settings.leagues} onRemove={removeLeague} onMove={moveLeague} />
        <AddLeague activeLeagues={settings.leagues} onAdd={addLeague} />
        <OtherSettings settings={settings.options} setSettings={(o) => {
          setSettings({ ...settings, options: o })
        }} />
      </div>

      <div class="sticky bottom-0 mt-auto p-3">
        <button class="w-full p-4 bg-blue-600 border border-blue-700 text-white rounded-full hover:bg-blue-700 focus:bg-blue-800 cursor-pointer" onClick={() => {
          returnToPKJS(settings)
        }}>
          Save changes
        </button>
      </div>
    </div>
  </div>
}

function Panel({ label, children }: { label: string, children: preact.ComponentChildren }) {
  return <div class="flex flex-col border-slate-200 border rounded-2xl">
    <div class="px-4 py-3 border-b border-slate-200">
      <p class="text-sm text-slate-500">{label}</p>
    </div>
    {children}
  </div>
}

function IconButton({ children, label, onClick, disabled }: { children: preact.ComponentChildren, label: string, onClick?: () => void, disabled?: boolean }) {
  return <button type="button" aria-label={label} disabled={disabled} onClick={onClick} class="h-10 min-w-10 px-3 flex items-center justify-center rounded-xl hover:bg-slate-100 focus:bg-slate-200 disabled:opacity-30 disabled:pointer-events-none cursor-pointer text-slate-500">
    {children}
  </button>
}

function LeaguesList({ leagues, onRemove, onMove }: { leagues: League[], onRemove: (id: number) => void, onMove: (index: number, dir: -1 | 1) => void }) {
  return <Panel label="Your leagues">
    <ul>
      {leagues.map((league, index) => (
        <li key={league.id} class="flex gap-4 justify-between px-4 py-3 items-center">
          <LeagueItemBase league={league} />
          <div class="flex items-center">
            <IconButton label={`Remove ${league.abbreviation}`} onClick={() => onRemove(league.id)}><Icon svg={x} /></IconButton>
            <IconButton label={`Move ${league.abbreviation} up`} disabled={index === 0} onClick={() => onMove(index, -1)}><Icon svg={caretUp} /></IconButton>
            <IconButton label={`Move ${league.abbreviation} down`} disabled={index === leagues.length - 1} onClick={() => onMove(index, 1)}><Icon svg={caretDown} /></IconButton>
          </div>
        </li>
      ))}
    </ul>
  </Panel>
}

function AddLeague({ activeLeagues, onAdd }: { activeLeagues: League[], onAdd: (league: League) => void }) {
  const [value, setValue] = useState("")

  const items = useMemo(() => {
    const activeIds = new Set(activeLeagues.map(l => l.id))
    return searchLeagues(value)
      .filter(l => !activeIds.has(l.id))
      .slice(0, MAX_RESULTS)
  }, [value, activeLeagues])

  const { getInputProps, getListProps, getItemProps, open, focusIndex } = useCombobox({
    items,
    value,
    onChange: (v) => setValue(v ?? ""),
    feature: autocompleteLite<League>(),
    getItemValue: (league) => league?.name ?? "",
    selected: undefined,
    onSelectChange: (league) => {
      if (league) {
        onAdd(league)
        setValue("")
      }
    },
  })

  const showList = open && items.length > 0

  return <Panel label="Add a league">
    <div class="px-4 py-3 flex items-center gap-3 text-slate-500">
      <Icon svg={magnifyingGlass} class="shrink-0" />
      <input {...getInputProps()} placeholder="Search..." class="w-full outline-none text-black placeholder:text-slate-400" />
    </div>
    {/* casts: szhsin's getters carry a React-typed `ref` that doesn't line up with
        Preact's element ref type under compat — runtime is fine, types only. */}
    <div {...(getListProps() as any)} class={showList ? "border-t border-slate-200" : "hidden"}>
      {items.map((league, index) => (
        <div {...(getItemProps({ item: league, index }) as any)} key={league.id} class={`flex gap-4 justify-between px-4 py-3 items-center cursor-pointer ${index === focusIndex ? "bg-slate-100" : ""}`}>
          <LeagueItemBase league={league} />
          <span class="h-10 min-w-10 px-3 flex items-center justify-center text-slate-500" aria-hidden><Icon svg={plus} /></span>
        </div>
      ))}
    </div>
  </Panel>
}

function LeagueItemBase({ league }: { league: League }) {
  return <div class="flex items-center gap-3 min-w-0">
    <SportIcon sport={league.sport} size={24} class="text-slate-500" />
    <div class="min-w-0">
      <p class="font-medium">{league.abbreviation}</p>
      <p class="text-sm text-slate-500 truncate">{league.name}</p>
    </div>
  </div>
}

function OtherSettings({ settings, setSettings }: { settings: PebbleSportsSettings["options"], setSettings: (s: PebbleSportsSettings["options"]) => void }) {
  return <Panel label="Other settings">
    <div class="flex flex-col">
      <SettingItem label="Add games to timeline" value={settings.timeline} labels={{ "never": "Never", "favorites": "Favorites only" }} onChange={(v) => setSettings({ ...settings, timeline: v })} />
      <SettingItem label="Show team records" value={settings.records} labels={{ "never": "Never", "final-only": "Final only", "always": "Always" }} onChange={(v) => setSettings({ ...settings, records: v })} />
    </div>
  </Panel>
}

function SettingItem<T extends string>({ label, value, labels, onChange }: { label: string, value: T, labels: Record<T, string>, onChange: (v: T) => void }) {
  return <div class="flex gap-4 justify-between px-4 py-3">
    <p>{label}</p>
    <select value={value} onChange={(e) => onChange(e.currentTarget.value as T)} class="text-right">
      {Object.entries(labels).map(([k, v]) => <option value={k}>{v as string}</option>)}
    </select>
  </div>
}
