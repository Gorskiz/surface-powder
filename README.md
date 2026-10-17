# Surface Powder

A small, native falling-sand sandbox written in Rust and SDL2. Pour powder,
water, fire and other materials into a world of glowing particles, growing
plants, explosions and little stickmen.

Inspired by [DAN-BALL Powder Game](https://dan-ball.jp/en/javagame/dust/).
This is an independent implementation with its own code and visuals; it does
not include the original game's source or assets.

## Screenshots

The starter scene includes a fan-driven wheel, a tumbling box, a bubble,
water, growing plants and jointed stickmen.

![Starter scene with materials, interactive objects and stickmen](docs/screenshots/starter-scene.png)

Native gameplay on the Surface Pro 4, with touch controls and a saved sandbox scene.

![Surface Pro 4 gameplay showing powder, water, platforms and stickmen](docs/screenshots/surface-gameplay.png)

## Features

- 400×300 simulation with **50 materials and tools**, arranged across three palette pages.
- Momentum-carrying wind and pressure waves with smooth sampling and obstacle deflection.
- Directional fans and pumps; explosions throw particles, bodies and characters outward.
- Anchored spinning wheels, tumbling boxes, material-carrying balls, and floating bubbles.
- Six-joint players and fighters with foot-driven movement, floppy limbs and flying, shrinking body fragments on elimination.
- Brush sizing, neon trails, airflow view, touch controls, and an interactive starter scene.
- Automatic scene persistence, including wind and object state.
- Fixed 60 Hz physics with native rendering capped at approximately 30 frames per second.
- No browser, game engine, account or network connection required to play.

The palette includes powder, water, fire, seeds, gunpowder, fans, ice,
superballs, clones, fireworks, oil, C-4, stone, magma, virus, nitro, ants,
torches, gas, soapy water, thunder, metal, bombs, lasers, acid, vines, salt,
glass, birds, mercury, sparks, fuses, clouds and pumps, plus wood, steam,
saltwater and snow. Tools include Wind, Air, Drag, Bubble, Wheel, Player,
Fighter, Box, Ball, Create, Wall and Erase.

This remains an independent approximation of the original. Air uses 4-pixel
tiles and particles use grid collisions, box-to-box collision uses conservative bounding circles, and bubbles
use a deforming film rather than sealed fluid volumes. Create currently
reuses the clone material behavior. Original saves, online sharing, copy/paste
and the original game's complete UI/interaction rules are not supported.

## Build and run

Requires Rust 1.85 or newer and the SDL2 development library. Linux and macOS
are supported; Linux Wayland has been tested on a Surface Pro 4.

### Debian / Ubuntu

Install [Rust](https://www.rust-lang.org/tools/install), then:

```sh
sudo apt-get install build-essential pkg-config libsdl2-dev
git clone https://github.com/Gorskiz/surface-powder.git
cd surface-powder
cargo run --release --locked
```

### macOS

With Rust and Homebrew installed:

```sh
brew install sdl2
git clone https://github.com/Gorskiz/surface-powder.git
cd surface-powder
RUSTFLAGS="-L native=$(brew --prefix sdl2)/lib" cargo run --release --locked
```

## Controls

| Input | Action |
| --- | --- |
| Touch, pen or left mouse drag | Paint selected material |
| Right mouse drag | Paint secondary material (Erase by default) |
| Set R, then a palette material | Choose secondary material using touch |
| Right-click a palette material | Choose secondary material using a mouse |
| Palette ‹ / ›, Tab, Page Up / Page Down | Browse palette pages |
| F / Flow | Show air velocity and pressure |
| Mouse wheel or − / + | Change brush size |
| Arrow keys or A / D / W | Move and jump with player stickmen |
| On-screen arrows / Jump | Touch controls for stickmen |
| Space / Pause | Pause or resume |
| G / Glow | Toggle trails |
| Save | Save the current scene |
| Clear, twice within three seconds | Clear the world |
| Demo | Replace the current scene with the starter scene |
| Escape | Save and exit |

Player, Fighter, Wheel, Box and Ball create one object per press. Draw a loop
with Bubble and release to create a sized bubble. Drag the rim of a wheel to
spin it, or drag a box/ball to throw it. Wind follows the direction of your
stroke; a held stationary wind brush keeps blowing in that direction. Fans
and pumps take their direction from the stroke used to place them.

Air raises pressure; right-clicking with Air lowers it. On touch, use the
Air + / Air − button beside Flow to switch between pressure and suction. Set R chooses a
material payload for subsequently placed fireworks and balls, as well as the
right mouse brush. Selecting Erase as the secondary resets payloads to powder.
Fireworks launch when ignited; balls release their payload on impact. Gunpowder, C4 and nitro throw existing grains outward; gas mostly ignites and adds pressure. Bomb instead pulls a small cloud inward before the affected grains decay. The
Flow overlay shows velocity arrows, green positive pressure and blue negative
pressure. The default demo includes a fan-driven gear and movable objects.

The Demo button
replaces the current world immediately; it is not an additional save slot.

## Saves

The scene saves every 30 seconds, when you press Save, and on normal exit,
SIGINT or SIGTERM. It reloads at startup.

Default location: `~/.local/state/surface-powder/world.bin`. Setting
`XDG_STATE_HOME` moves it to `$XDG_STATE_HOME/surface-powder/world.bin`.
Version 1 saves load automatically; the first upgraded save creates a
`world.bin.v1.bak` copy. Version 2 saves retain pressure, velocity, bodies and
material payloads. Version 3 adds character joints, death fragments and particle momentum.
Both older versions load automatically and receive a versioned backup before upgrading. Corrupt or incompatible saves are preserved and reported rather than
overwritten. To start fresh, move the save file aside.

## Development

```sh
cargo test --locked
cargo fmt --check
cargo clippy --locked -- -D warnings
cargo run --release --locked -- --benchmark
```

On macOS, use the same `RUSTFLAGS` setting from the build command for commands
that link the executable. Tests cover articulated movement and fragmentation, particle momentum and conservation,
barriers, wind transport, pressure waves, wheel torque,
body collision, bubble popping, material payloads, chemistry, electricity,
legacy-save migration, deterministic save/load continuation and touch input.

For a headless rendering check:

```sh
SDL_VIDEODRIVER=dummy XDG_STATE_HOME=/tmp/surface-powder-preview \
  cargo run --release --locked -- --frames 30 --screenshot preview.ppm
```

On Unix, SIGUSR1 captures `preview.ppm` alongside the saved world.

## Performance

The simulation keeps compact two-byte material cells plus eight bytes of particle momentum
per cell, a coarse air grid, and a maximum
of 128 moving objects. It does not add physics-engine dependencies. Use
`--benchmark` to measure simulation plus pixel rendering on your hardware;
that benchmark excludes SDL presentation and UI drawing. Large active scenes
can reduce frame rate on older hardware.

## License

[MIT](LICENSE).
