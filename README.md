# Star Trader (1974) – Rust TUI Remake

The date is Jan 1, 2070 and interstellar flight has existed for 70 years. There are several star systems that have been colonized. Some are only frontier systems, others are older and more developed. Each of you is the captain of two interstellar trading ships. You will travel from star system to star system, buying and selling merchandise. If you drive a good bargain you can make large profits. As time goes on, each star system will slowly grow, and its needs will change. A star system that now is selling much uranium and raw metals cheaply may not have enough for export in a few years. Your ships can travel about two lightyears in a week and can carry up to 30 tons of cargo. Only class 1 and class II star systems have banks on them. They pay 5% interest and any money you deposit on one planet is available on another—provided there's a local bank.

## Contents

- [Overview](#overview)
- [What is Star Trader?](#what-is-star-trader)
- [Features](#features)
- [How to Play](#how-to-play)
- [Controls](#controls)
- [Keybindings (Customisation)](#keybindings-customisation)
- [Saving & Loading](#saving--loading)
- [Building & Running](#building--running)
- [Logging](#logging)
- [Tech Stack](#tech-stack)

## Overview

This project is a remake of the 1974 BASIC game "Star Trader", by Dave Kaufman, faithfully retaining the core game mechanics and _feel_.

The classic trading loop — travel, haggle, bank, and grow colonies — returns with a modern TUI: structured prompts, event log, input pane, overlays (map/report/help), route scheduling, and optional logging for debugging.

## What is Star Trader?

_Star Trader_ is a 1974 video game and one of the earliest examples of the space-trading genre. Developed by Dave Kaufman in 1973, the game’s BASIC source code was published in the January 1974 issue of the _People’s Computer Company Newsletter_ and later reprinted in the 1977 book _What to Do After You Hit Return_. Designed as a single-player space trading simulation, the game has players travel between star systems on a galactic map, buying and selling six types of goods—uranium, metals, heavy equipment, medicine, software, and gems – as prices and availability fluctuate over time. _Star Trader_ went on to inspire the multiplayer _Trade Wars_ series beginning in 1984, and through that lineage became a foundational antecedent of much of the modern space-trading genre.

## Features

- Core mechanics: trading with multi-round haggling, system stock/price drift, class-based development, travel time with probabilistic delays, banking with interest, and new-star spawning as the frontier develops.
- Travel and routing: schedule courses with ETAs and delays; advance time to the next arrival and update systems as ships dock.
- UI polish: systems/market/ships panels, context-aware prompts, bottom input bar, event log, blinking change highlights, overlays for map, trade report, and help, plus an ultra-compact ships view for narrow terminals.
- Persistence: save/load full state to JSON snapshots (auto-load `savegame.json` on startup; <kbd>v</kbd>/<kbd>o</kbd> for manual save/load).
- Configurable keys: footer/help stay in sync with `keybindings.json` overrides.
- Logging: optional env/CLI-controlled logs for debugging runs.

## How to Play

1. Start the game: `cargo run -p star-trader` from repo root.
2. Navigate panels: <kbd>1</kbd>/<kbd>2</kbd>/<kbd>3</kbd> focuses Systems/Market/Ships; arrow keys move within the active panel.
3. Move ships: <kbd>g</kbd> sets a course to the focused system, or <kbd>d</kbd> picks a destination; <kbd>n</kbd> advances time to the next arrival and applies price/stock changes.
4. Trade: <kbd>b</kbd> buys or <kbd>s</kbd> sells the selected good. Enter quantity, then haggle on price; offers may be accepted, rejected, or countered with limited rounds.
5. Bank: <kbd>k</kbd> deposits and <kbd>l</kbd> withdraws (only at systems with banks). Interest accrues when time advances.
6. Map/report/help: <kbd>m</kbd> opens the map, <kbd>t</kbd> the trade report, <kbd>?</kbd>/<kbd>h</kbd> help; <kbd>Esc</kbd> closes overlays.
7. Save/Load/Reset/Quit: <kbd>v</kbd> save, <kbd>o</kbd> load, <kbd>r</kbd> reset to demo state, <kbd>q</kbd> quit.

## Controls

| Action       | Keys                                                                                                                  |
|--------------|-----------------------------------------------------------------------------------------------------------------------|
| Focus panels | <kbd>1</kbd> (Systems), <kbd>2</kbd> (Market), <kbd>3</kbd> (Ships)                                                   |
| Navigation   | Arrow keys within the active panel                                                                                    |
| Cycle goods  | <kbd>[</kbd> previous, <kbd>]</kbd> next                                                                              |
| Travel       | <kbd>g</kbd> set course, <kbd>d</kbd> pick destination, <kbd>n</kbd> next arrival, <kbd>p</kbd> jump-to system number |
| Trade        | <kbd>b</kbd> buy, <kbd>s</kbd> sell (quantity then price haggle)                                                      |
| Bank         | <kbd>k</kbd> deposit, <kbd>l</kbd> withdraw (where available)                                                         |
| Overlays     | <kbd>m</kbd> map, <kbd>t</kbd> trade report, <kbd>?</kbd> or <kbd>h</kbd> help, <kbd>Esc</kbd> close overlay          |
| Session      | <kbd>v</kbd> save, <kbd>o</kbd> load, <kbd>r</kbd> reset demo, <kbd>q</kbd> quit                                      |

## Keybindings (Customisation)

- Create `keybindings.json` in the repo root to override defaults; missing entries fall back to defaults.
- Allowed values: single ASCII characters or <kbd>up</kbd>, <kbd>down</kbd>, <kbd>left</kbd>, <kbd>right</kbd>, <kbd>esc</kbd>, <kbd>tab</kbd>, <kbd>backtab</kbd>.
- Config reloads on startup and after loading a save (<kbd>o</kbd>).
- Example:

```json
{
  "focus_systems": "1",
  "focus_market": "2",
  "focus_ships": "3",
  "prev_good": "[",
  "next_good": "]",
  "set_course": "g",
  "buy": "b",
  "sell": "s",
  "deposit": "k",
  "withdraw": "l",
  "set_destination": "p",
  "pick_destination": "d",
  "next_arrival": "n",
  "map": "m",
  "report": "t",
  "help": "?",
  "help_alt": "h",
  "save": "v",
  "load": "o",
  "reset": "r",
  "quit": "q"
}
```

## Saving & Loading

- Auto-loads `savegame.json` on startup if present.
- <kbd>v</kbd> saves the current state; <kbd>o</kbd> loads from `savegame.json`.

## Building & Running

- Prerequisites: Rust toolchain (stable) and an ANSI-capable terminal (macOS/Linux/Windows Terminal).
- Build: `cargo build -p star-trader`
- Run: `cargo run -p star-trader`

## Logging

- Env: set `STAR_TRADER_LOG=<level>` or `RUST_LOG=<level>` (e.g., `info`, `debug`); default is `warn`.
- CLI: `--log` enables `info`; `--log-level <level>` sets an explicit level.
- Logs go to stderr and do not affect the TUI.

## Tech Stack

| Crate                                                           | Description                                             |
|-----------------------------------------------------------------|---------------------------------------------------------|
| [ratatui](https://crates.io/crates/ratatui)                     | TUI rendering layer using the crossterm backend feature |
| [crossterm](https://crates.io/crates/crossterm)                 | Terminal input/output driver                            |
| [serde](https://crates.io/crates/serde)                         | Core serialization support (derive)                     |
| [serde_json](https://crates.io/crates/serde_json)               | JSON read/write for save/load                           |
| [rand](https://crates.io/crates/rand)                           | Randomness for delays and price variation (serde1)      |
| [rand_chacha](https://crates.io/crates/rand_chacha)             | ChaCha RNG backing for deterministic seeds (serde1)     |
| [log](https://crates.io/crates/log)                             | Logging facade                                          |
| [env_logger](https://crates.io/crates/env_logger)               | Env-configurable logger implementation                  |
| [anyhow](https://crates.io/crates/anyhow)                       | Contextual error handling for fallible paths            |
| [pretty_assertions](https://crates.io/crates/pretty_assertions) | Friendlier diff output in tests                         |
