# Cosmic Climber — Rust Edition
> A vertical platformer for TV / browser, compiled to WebAssembly via Rust + Macroquad

## Tech Stack
- **Language**: Rust (2021 edition)
- **Rendering**: [Macroquad](https://macroquad.rs/) 0.4 (WebGL backend via miniquad)
- **Build**: `cargo build --target wasm32-unknown-unknown --release`
- **WASM target**: wasm32-unknown-unknown (no SIMD, no threads)
- **Deployment**: Vercel (static hosting, pre-built `dist/`)

## Playing on TV (smart TV / streaming device)

Use your **remote** (not a keyboard). The game expects the same controls most TV apps use:

- **D-pad** (Up, Down, Left, Right) — move, jump, jetpack, and fast fall.
- **OK** (also labeled **Select** or **Enter** on some remotes) — confirm menu choices, **shoot** in-game, and **retry** after game over.
- **Return** (also **Back** on some remotes) — open pause, go back in menus, or exit flows where the platform allows it.

**Tips:** Short taps on Left/Right nudge the astronaut; **hold** to run. **Hold Up** for extra jump height when you land, or for jetpack thrust when you have fuel. **Hold Down** to fall faster.

## TV remote — control reference

| Action        | Remote                    | Behaviour                              |
|---------------|---------------------------|----------------------------------------|
| Move left     | D-pad Left                | Tap = nudge, **hold** = run            |
| Move right    | D-pad Right               | Tap = nudge, **hold** = run            |
| Jump / extra jump | D-pad Up (hold)     | **Hold** at landing = height boost     |
| Jetpack       | D-pad Up (hold)           | **Hold** while jetpack active = thrust |
| Fast fall     | D-pad Down (hold)         | **Hold** = accelerated descent         |
| Shoot laser   | **OK** / Select           | Single shot, no auto-repeat            |
| Pause         | **Return** / Back         | Pause / back                            |
| Start / confirm | **OK** / Select         | Menu: start game, settings, resume     |

## Keyboard (desktop browser only)

For testing on a PC, use the **arrow keys** like a D-pad, **Enter** or **Space** like **OK**, and **Escape** like **Return** / Back:

| Action        | Key              | Behaviour                              |
|---------------|------------------|----------------------------------------|
| Move left     | ◀ Arrow / A      | Tap = nudge, **Hold** = run            |
| Move right    | ▶ Arrow / D      | Tap = nudge, **Hold** = run            |
| Extra jump    | ▲ Arrow / W (hold) | **Hold** at landing = height boost   |
| Jetpack       | ▲ Arrow / W (hold) | **Hold** while jetpack active = thrust |
| Fast fall     | ▼ Arrow / S (hold) | **Hold** = accelerated descent       |
| Shoot laser   | Enter / Space    | Single fire, no auto-repeat            |
| Pause         | Escape           | Toggle pause, no auto-repeat           |

## Levels
| # | World         | New Mechanics                                 |
|---|---------------|-----------------------------------------------|
| 1 | Meadow        | Static platforms only                         |
| 2 | Forest        | Moving platforms                              |
| 3 | Mountain      | Crumbling platforms, wind                     |
| 4 | Ocean         | Bouncy + spring platforms, conveyor           |
| 5 | Arctic        | Ice (slippery), disappearing platforms        |
| 6 | Volcano       | Lava platforms, enemies                       |
| 7 | Sky City      | Cloud platforms, ghost enemies                |
| 8 | Space Station | Turret enemies, fast scroll                   |
| 9 | Deep Space    | Chaser enemies, max disappearing              |
|10 | Cosmic Core   | All types, max speed, all enemies             |

## Platform Types
| Type        | Behaviour                                          |
|-------------|----------------------------------------------------|
| Normal      | Solid and static                                   |
| Moving      | Slides left/right between a fixed range            |
| Bouncy      | Launches player at 1.55× jump force               |
| Crumble     | Breaks ~30 frames after first landing              |
| Disappear   | Turns non-solid for ~60 frames after touch         |
| Spring      | Catapults player upward                            |
| Conveyor    | Pushes player horizontally                         |
| Lava        | Kills on contact — shield does not protect         |
| Cloud       | One-way: only solid when falling downward          |
| Ice         | Very low friction — player slides                  |

## Build Prerequisites
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the WASM target
rustup target add wasm32-unknown-unknown
```

## Build & Run Locally
```bash
# Release build → copies output to dist/
bash build.sh

# Serve locally (WASM requires HTTP, not file://)
npx serve dist
```

Open `http://localhost:3000` in your browser.

> **Note**: Do not open `dist/index.html` directly as a `file://` URL.
> Browsers block WASM loading over the file protocol for security reasons.

## Deploy to Vercel
The project is set up for static deployment — `dist/` is committed to the repo
and Vercel serves it directly with no server-side build step.

```bash
# First deploy
npm install -g vercel
vercel --prod

# Subsequent deploys — just build locally, commit, and push
bash build.sh
git add dist/
git commit -m "update: describe your changes"
git push
# Vercel redeploys automatically in ~10 seconds
```

Vercel settings:
- **Build Command**: *(empty — disabled)*
- **Output Directory**: `dist`

## Update Workflow
```
Edit Rust source
      ↓
bash build.sh          ← compiles + assembles dist/
      ↓
npx serve dist         ← verify locally
      ↓
git add dist/ && git commit && git push
      ↓
Vercel auto-redeploys in ~10 seconds 🚀
```

## Known Issues & Workarounds

### miniquad 0.4.x RefCell panic on mouse events
Miniquad 0.4.x has a bug where `mouse_move` and `focus` events firing
simultaneously cause a `RefCell` double-borrow panic. Since this game is
keyboard/remote only, the fix is to null out all mouse handlers after the
WASM initialises:

```js
setTimeout(function () {
  var canvas = document.getElementById("glcanvas");
  if (canvas) {
    canvas.onmousemove = null;
    canvas.onmousedown = null;
    canvas.onmouseup   = null;
    canvas.onfocus     = null;
  }
}, 800);
```

### Harmless WebGL warnings in console
On load you may see messages like `No glRenderbufferStorageMultisample function in gl.js`.
These are expected — miniquad logs stubs for optional WebGL2 functions that
aren't present in its bundled `gl.js`. They do not affect gameplay.

### Asset filenames are case-sensitive on Vercel (Linux)
Vercel runs on Linux where `astronaut.png` and `Astronaut.png` are different files.
Make sure asset filenames in your Rust code match exactly what's on disk.

## Project Layout
```
cosmic-climber-wasm-rust/
├── src/
│   ├── main.rs     — entry point, window config, texture loading, game loop
│   ├── game.rs     — physics, collision, AI, generation, scoring, rendering
│   └── levels.rs   — 10 level configurations (LevelCfg structs)
├── assets/
│   └── astronaut.png   — sprite sheet (768×64, 12 frames of 64×64)
├── dist/               — pre-built output committed to repo for Vercel
│   ├── index.html
│   ├── cosmic-climber-rs.wasm
│   ├── mq_js_bundle.js
│   └── assets/
│       └── astronaut.png
├── build.sh        — local build script (cargo → dist/)
├── Cargo.toml
├── Cargo.lock
└── README.md
```

## Sprite Sheet Format
`assets/astronaut.png` — 768×64 px, single row, 12 columns, 64×64 per frame:

| Columns | Animation      | Usage                   |
|---------|---------------|-------------------------|
| 0 – 1   | Idle / breathe | Standing still, airborne |
| 8 – 10  | Walk cycle     | Moving left or right     |

Walking frames face right. Left movement uses `flip_x`.

## Rust Migration Notes
This project was originally written in C11 + SDL2 + Emscripten. Key changes
made during the Rust migration:

- **Fixed arrays → `Vec<T>`**: `Platform`, `Enemy`, `Particle`, etc. are now
  dynamic vectors with `.retain()` for culling instead of manual index shifts
- **`static` locals eliminated**: Generation state (`gen_prev_cx`) moved into
  the `Game` struct and reset explicitly in `game.reset()` — fixing a bug where
  platform layout was broken on game restart
- **Input**: Replaced manual `key_held/key_just/key_time` arrays with
  Macroquad's native `is_key_down()` / `is_key_pressed()`
- **JS interop**: Uses `miniquad_add_plugin` instead of raw `extern "C"` for
  `js_load_hi`, `js_save_hi`, `js_set_hud`, `js_set_state`
- **Checkpoint system**: Added `cp_x/cp_y/cp_valid` to `Game` struct —
  respawns player at the last platform they landed on instead of mid-air