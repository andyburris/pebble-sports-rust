#!/bin/bash
# Usage: ./sym_sizes.sh [elf_file]
# Lists all symbols sorted by size descending, in decimal

ELF="${1:-build/aplite/pebble-app.elf}"

# If the requested ELF doesn't exist, fall back to the first platform that built.
if [ ! -f "$ELF" ]; then
    fallback="$(ls build/*/pebble-app.elf 2>/dev/null | head -1)"
    if [ -z "$fallback" ]; then
        echo "error: no built ELF found (looked for '$ELF' and build/*/pebble-app.elf)." >&2
        echo "Run ./build.sh first, or pass a path: ./sym_sizes.sh build/basalt/pebble-app.elf" >&2
        exit 1
    fi
    echo "note: '$ELF' not found; using '$fallback'" >&2
    ELF="$fallback"
fi

TOOLCHAIN="$HOME/Library/Application Support/Pebble SDK/SDKs/current/toolchain/arm-none-eabi/bin"
NM="$TOOLCHAIN/arm-none-eabi-nm"
SIZE="$TOOLCHAIN/arm-none-eabi-size"
RESOURCES="$(dirname "$ELF")/app_resources.pbpack"
RAM_LIMIT=24576

# Print RAM summary from section sizes (matches Pebble build output)
"$SIZE" "$ELF" 2>/dev/null | python3 -c "
import sys, os
line = sys.stdin.readlines()[-1]
parts = line.split()
text, data, bss = int(parts[0]), int(parts[1]), int(parts[2])
ram = text + data + bss
limit = $RAM_LIMIT
res = os.path.getsize('$RESOURCES') if os.path.exists('$RESOURCES') else None
print(f'RAM:       {ram:6d} / {limit} bytes  ({ram/limit*100:.1f}% used, {limit-ram} remaining)')
print(f'  .text    {text:6d}  .data  {data:6d}  .bss  {bss:6d}')
if res is not None:
    print(f'Resources: {res:6d} bytes (flash, not counted against RAM limit)')
print()
"

"$NM" -S --size-sort "$ELF" 2>/dev/null \
  | python3 -c "
import sys
rows = []
for line in sys.stdin:
    parts = line.split()
    if len(parts) < 4:
        continue
    typ = parts[2]
    if typ not in ('T','t','W','w'):
        continue
    size = int(parts[1], 16)
    if size == 0:
        continue
    rows.append((size, parts[3]))
rows.sort(reverse=True)
for size, name in rows:
    print(f'{size:6d}  {name}')
"
