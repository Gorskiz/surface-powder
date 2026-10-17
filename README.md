# Surface Powder

A small, native falling-sand sandbox written in Rust and SDL2. Pour powder,
water, fire and other materials into a world of glowing particles, growing
plants, explosions and little stickmen.

Inspired by [DAN-BALL Powder Game](https://dan-ball.jp/en/javagame/dust/).
This is an independent implementation with its own code and visuals; it does
not include the original game's source or assets.

## Features

- 400×300 simulation with 20 materials/tools and a large touch-friendly palette.
- Powder, water, fire, seeds, wood, gunpowder, oil, stone, magma, ice, steam,
  acid, metal, sparks, gas, clones and walls.
- Controllable players and autonomous fighters.
- Adjustable brushes, neon trails, pause and a starter scene.
- Automatic scene persistence, including when the app exits gracefully.
- Native fullscreen rendering at approximately 30 frames per second.
- No browser, game engine, account or network connection required to play.

The original game's wind, wheels, boxes, bubbles, remaining materials, save
format and online sharing are not implemented.

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
| Right mouse drag | Erase |
| Mouse wheel or − / + | Change brush size |
| Arrow keys or A / D / W | Move and jump with player stickmen |
| On-screen arrows / Jump | Touch controls for stickmen |
| Space / Pause | Pause or resume |
| G / Glow | Toggle trails |
| Save | Save the current scene |
| Clear, twice within three seconds | Clear the world |
| Demo | Replace the current scene with the starter scene |
| Escape | Save and exit |

Player and Fighter tools create one stickman per press. The Demo button
replaces the current world immediately; it is not an additional save slot.

## Saves

The scene saves every 30 seconds, when you press Save, and on normal exit,
SIGINT or SIGTERM. It reloads at startup.

Default location: `~/.local/state/surface-powder/world.bin`. Setting
`XDG_STATE_HOME` moves it to `$XDG_STATE_HOME/surface-powder/world.bin`.
Corrupt or incompatible saves are preserved and reported rather than
overwritten. To start fresh, move the save file aside.

## Development

```sh
cargo test --locked
cargo fmt --check
cargo clippy --locked -- -D warnings
cargo run --release --locked -- --benchmark
```

On macOS, use the same `RUSTFLAGS` setting from the build command for commands
that link the executable. Tests cover particle conservation, liquid barriers,
material reactions, conductive sparks, actors, save/load, and input mapping.

For a headless rendering check:

```sh
SDL_VIDEODRIVER=dummy XDG_STATE_HOME=/tmp/surface-powder-preview \
  cargo run --release --locked -- --frames 30 --screenshot preview.ppm
```

On Unix, SIGUSR1 captures `preview.ppm` alongside the saved world.

## Performance

On a Surface Pro 4 (i5-6300U, Debian 13), a starter-scene benchmark measured
about 2 ms per simulation and pixel-render step. The running game measured
approximately 56 MiB proportional memory (PSS). These are sample measurements,
not guarantees for every scene, graphics driver or machine.

## License

[MIT](LICENSE).
