#!/bin/sh
# Hold the standard Linux performance profile only for this game process.
# A selected power-saver profile (including a thermal/battery override) wins.
set -eu
powder_script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
powder_executable="${SURFACE_POWDER_EXECUTABLE:-$powder_script_dir/../target/release/surface-powder}"
if command -v powerprofilesctl >/dev/null 2>&1; then
    powder_profile=$(powerprofilesctl get 2>/dev/null || true)
    if [ "$powder_profile" = balanced ] || [ "$powder_profile" = performance ]; then
        exec powerprofilesctl launch --profile performance \
            --appid org.gorskiz.SurfacePowder --reason 'Native game frame deadlines' \
            -- "$powder_executable" "$@"
    fi
fi
exec "$powder_executable" "$@"
