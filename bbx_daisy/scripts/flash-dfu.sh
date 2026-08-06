#!/usr/bin/env bash
#
# Flash a Daisy patch over USB DFU using dfu-util.
#
# This doubles as the cargo `runner` configured in .cargo/config.toml, so
# `cargo run --example <name> --release` (from the bbx_daisy/ directory) builds,
# converts the ELF to a raw binary, and flashes it. It can also be run directly:
#
#     ./scripts/flash-dfu.sh target/thumbv7em-none-eabihf/release/examples/01_blink
#
# The Daisy must be in DFU mode first:
#     hold BOOT, tap RESET, release BOOT   (enumerates as USB 0483:df11)
#
# Environment overrides:
#     OBJCOPY     objcopy binary to use      (default: arm-none-eabi-objcopy)
#     FLASH_ADDR  flash destination address  (default: 0x08000000, internal flash)

set -euo pipefail

ELF="${1:?usage: flash-dfu.sh <path-to-elf>}"
OBJCOPY="${OBJCOPY:-arm-none-eabi-objcopy}"
FLASH_ADDR="${FLASH_ADDR:-0x08000000}"

if ! command -v "$OBJCOPY" >/dev/null 2>&1; then
    echo "error: '$OBJCOPY' not found." >&2
    echo "       Install the ARM GNU toolchain, or set OBJCOPY=llvm-objcopy." >&2
    exit 1
fi

if ! command -v dfu-util >/dev/null 2>&1; then
    echo "error: 'dfu-util' not found. Install it (macOS: brew install dfu-util)." >&2
    exit 1
fi

BIN="${ELF}.bin"
"$OBJCOPY" -O binary "$ELF" "$BIN"

echo "Flashing $(basename "$BIN") to ${FLASH_ADDR} via DFU..."
echo "(If this hangs or errors, put the Daisy in DFU mode: hold BOOT, tap RESET, release BOOT.)"
dfu-util -a 0 -s "${FLASH_ADDR}:leave" -D "$BIN" -d ,0483:df11
