# Cyber Temple — Rooftop Run

A cyberpunk endless runner built with Bevy. The old rooftop platformer is now a **Temple Run-style** chase: you always sprint forward, the city unrolls under your feet, and the horde closes in the moment you stumble.

-----

## The Run

You are being hunted across neon rooftops. There is no walking back. Speed climbs the farther you go, gaps open up, and a zombie chaser rides your blind spot. Trip twice — or let the horde catch up — and the run is over.

Every attempt is scored. Beat your own high score.

-----

## Controls

The runner never stops. You only steer, jump, and slide.

| Action | Keyboard | Swipe |
| --- | --- | --- |
| Switch lane left / right | `A` `←` / `D` `→` | Swipe left / right |
| Jump over crates | `W` `↑` `Space` | Swipe up |
| Slide under neon bars | `S` `↓` | Swipe down |
| Pause | `Escape` | — |
| Start / run again | `Space` or `Enter` | — |

Mouse drag counts as a swipe, so the same gestures work on a trackpad.

-----

## How It Plays

### Three lanes

The rooftop is three deep tracks. Near lanes render larger and in front; far lanes shrink into the city. Obstacles only hit the lane you are in, so a last-second switch is a real save.

### Jump, slide, or die trying

- **Low magenta crates** — jump.
- **Cyan overhead bars** — slide.
- **Gaps** — jump or you fall into the city.

Miss a read and you **stumble**. The horde lurches closer. Stumble again before you recover, or let the threat meter fill, and you get caught.

### Coins and idols

Gold coins spawn in lines, jump arcs, and slide tunnels. Power-ups appear later in a run:

- **Coin magnet** — nearby coins fly in, even from other lanes.
- **Shield** — eats one mistake.
- **x2 coins** — doubles coin points while it lasts.
- **Nitro boost** — a short speed burst with i-frames.

### The horde

A single chaser stays behind you. Clean running opens a gap. Stumbles slam it shut. When it reaches you, the run ends.

-----

## Scoring

Score is **distance + coin points**. Multiplier pickups only affect coins. High scores are saved to `~/.fuzzy_runner_highscore`.

-----

## Menus

- **Title** — best score, run, settings.
- **Pause** — resume, new run, settings, menu.
- **Settings** — Easy / Normal / Hard. Difficulty changes start speed, how fast you ramp, obstacle density, and how aggressively the horde closes.
- **Game over** — cause of death, stats, new-high-score banner, run again.

-----

## Run it

```bash
cargo run --release
```

Needs a windowed GPU environment (Bevy / WGPU).

-----

## Build

Rust 2021, Bevy 0.13, `bevy-parallax` for the city layers, `rand` for the track.

Menus, HUD, keyboard prompts, spinning coins, roadside temples, and rooftop tiles use free **Kenney** (CC0) packs plus **Orbitron** (OFL) for titles. Credits are in `ATTRIBUTION.md`.

```bash
cargo test
cargo build --release
```
