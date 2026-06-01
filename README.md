# Aetherfall

Aetherfall is a new, original Rust sandbox prototype for a falling-sand survival game. It is a clean-room seed project with a fresh name and identity, designed around deterministic cellular simulation rather than copied upstream game code.

## Goals

- Build a Rust-first falling-sand engine core.
- Keep the simulation deterministic and easy to test.
- Provide a foundation for Terraria-like world layering, materials, crafting, and survival systems.
- Avoid rebranding or redistributing third-party assets or source code.

## Current features

- Compact `World` grid with bounds-safe reads and writes.
- Materials: air, stone, sand, water, wood, and fire.
- Gravity for granular materials.
- Liquid falling and lateral spreading.
- Sand-water displacement.
- Fire spread to flammable cells.
- ASCII rendering for fast CLI previews and snapshot tests.

## Run

```bash
cargo run
```

## Test

```bash
cargo test
```

## Legal note

This repository is intended as an original implementation. External projects can be studied for high-level ideas only when their licenses permit it; source code, protected branding, and non-commercial assets should not be copied into this project without explicit permission and attribution.
