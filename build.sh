#@IgnoreInspection BashAddShebang

target="thumbv7m-none-eabi"

export RUSTFLAGS="-C relocation-model=pie -C codegen-units=1 -C link-arg=--gc-sections -C link-arg=--build-id=sha1 -C link-arg=--emit-relocs -C debuginfo=2 -Z unstable-options -C panic=immediate-abort -C force-unwind-tables=no"

# Build the project through Cargo
cargo --version
cargo build --target $target --release || exit 1

# Extract the self-contained staticlib into a *clean* dir (use GNU ar from the
# Pebble SDK; macOS BSD ar can't handle GNU-format archives). We extract
# libappmessage.a — the final crate-type=staticlib output, which bundles exactly
# the LTO'd objects we need (core/alloc/compiler_builtins/our crate) — rather than
# scraping target/.../deps/*.o. The deps dir accumulates stale .rcgu.o across
# builds and holds the un-LTO'd multi-CGU dependency objects; globbing it links
# duplicate/old symbols (silently resolved by --allow-multiple-definition).
PEBBLE_AR="$HOME/Library/Application Support/Pebble SDK/SDKs/current/toolchain/arm-none-eabi/bin/arm-none-eabi-ar"
LINK_OBJS="target/$target/release/link-objs"
rm -rf "$LINK_OBJS"
mkdir -p "$LINK_OBJS"
( cd "$LINK_OBJS" && "$PEBBLE_AR" x "../libappmessage.a" )

# Compile TypeScript before waf bundles it
bunx pkts build
mkdir -p src/js
cp src/ts-build/index.js src/js/pebble-js-app.js
rm -rf src/ts-build

# Build through waf
pebble build
