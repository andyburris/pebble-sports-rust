// Renders a Phosphor "regular" SVG imported as a raw string (`?raw`). We inline raw
// SVGs from @phosphor-icons/core rather than @phosphor-icons/react so the bundle only
// carries the single weight we use, not all six. The source SVGs already use
// fill="currentColor", so `color` (Tailwind text-*) controls the fill.
export function Icon({ svg, size = 24, class: className }: { svg: string, size?: number, class?: string }) {
  const sized = svg.replace("<svg", `<svg width="${size}" height="${size}"`)
  return <span aria-hidden class={className} style={{ display: "inline-flex", flexShrink: 0 }} dangerouslySetInnerHTML={{ __html: sized }} />
}
