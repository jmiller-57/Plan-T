# Plan-T

A Rust engine for planning a bouldering day with a group of climbers of different abilities. It scores and ranks climbs against each person's ability, daily goals, and preferences, and computes whether a rock face will be in sun or shade at a given time.

## What it does
- Models a guidebook hierarchy: Destination → Region → Area → nested Sectors → Boulder → Problem
- Imports climb data from CSV (`serde` + `csv`)
- Calculates sun position (azimuth and elevation) from first principles and classifies faces as sun / glancing / shade, **validated against the NOAA solar calculator**
- Scores problems with a multiplicative model: grade fit × feature preferences (with hard vetoes) × quality

## Design highlights
- UI-free library crate, so the same core can back a CLI, web (WASM), or mobile front end
- Generic `Arena<T>` with type-safe `Id<T>` handles (a boulder ID can't index problems)
- "Make illegal states unrepresentable": validated value types such as grades and stars
- Test-driven: 57 unit and integration tests, zero-warning build

See [DESIGN.md](DESIGN.md) for the full design, trade-offs, and roadmap.

## Running
~~~bash
cargo test
~~~

## Status
Catalog, import, sun model, and single-problem scoring are built. Next: area-level scoring, group aggregation, and a CLI.
