# Plan-T — Design

A bouldering **day-planner for groups with varied skill**. Given a set of areas
and each person's goals for the day, it suggests how to spend the day — which
areas to visit and when — accounting for climb density by grade, travel time,
sun/shade, weather, and each climber's preferences and daily intent.

- **First real use:** a Lake Tahoe bouldering trip, **October 2026** (~4 months
  out as of this writing). Having it ready for the trip is a *nice-to-have, not a
  requirement* — the project is primarily for fun and learning.
- **Trajectory:** start as a local engine + CLI on the laptop; possibly grow a
  web or mobile front-end later.

---

## Implementation status (snapshot)

**Built & tested:**
- World model: `Destination → Region → Area → Sector* → Boulder → Problem`, all
  stored in a generic `Arena<T>` with typed `Id<T>` handles, aggregated in a `Catalog`.
- Value objects: `VGrade` (grade.rs), `ProblemStatus` (problem.rs), `Coordinates`
  + `Aspect` (location.rs); `Feature` set on `Problem`.
- Attributes wired: problem name/grade/status/features/aspect; boulder
  coordinates/aspect; aspect `inherit↓` via `Problem::effective_aspect`.
- CSV importer (import.rs): rows → deduped `Catalog` using `serde` + `csv`, with
  `parse_grade`/`parse_status` and `Result`-based error handling.
- **Sun feature** (sun.rs): `SunPosition::calculate(coords, DateTime<Utc>)`
  (declination + equation of time + hour angle + spherical trig, NOAA-validated),
  and `SunPosition::cover_for(aspect) -> SunCover` (`Sun`/`Glancing`/`Shade`).

**Not yet built (candidates for next sessions):**
- Remaining attributes: stars/quality, steepness, guidebook `number`, style tags.
- Travel: start location, drive + approach legs, totals.
- People & session (Part 2): `ClimberProfile`, `SessionIntent`, `DayContext`, weather.
- The engine (Part 3): `problem_score → area_score → group_area_score → day_plan`.
- Real Tahoe data entry (only a tiny Kingsbury sample exists), and a CLI interface.
- The local-time → UTC boundary conversion (`chrono-tz`).

---

## Tech direction

- **Language: Rust.** Chosen because (a) it's a deliberate learning goal, (b) its
  type system is a domain-modeling asset for exactly the hard part of this
  project — see patterns below, and (c) the "shared core engine" pattern lets the
  same crate back a CLI today and (via WASM / UniFFI bindings) a web or mobile
  front-end later, with no rewrite.
- **Keystone architecture decision:** separate the **engine** (data model +
  scoring/recommendation logic, pure and UI-free) from the **interface** (CLI
  now; web/mobile later). This is what keeps the language choice low-stakes and
  the mobile move cheap.
- **Storage = arena + IDs:** entities live in flat per-type stores (a `Vec`/
  collection each) and reference one another by lightweight **ID newtypes**, not
  pointers. Chosen because relationships cut *across* the guidebook hierarchy
  (proximity, cross-area day plans) — which an ownership tree handles poorly and
  Rust's borrow-checker makes painful to express with pointers. See Decisions Log.
- **Sun position:** *implemented* in `sun.rs`. `SunPosition::calculate` derives
  azimuth + elevation (both `f64`) from `Coordinates` + a UTC `DateTime` via
  Cooper's declination, the equation of time, hour angle, and spherical trig —
  validated against the NOAA solar calculator. Computed in **UTC**; local-time/DST
  conversion is a boundary concern for the caller (`chrono-tz`), keeping the calc
  pure. `cover_for(aspect)` then classifies a vertical face as `Sun`/`Glancing`/
  `Shade` (elevation gate, then cyclic azimuth-vs-aspect within 90°).

---

## Cross-cutting modeling patterns

These rules apply throughout the model:

- **raw vs. derived** — `[raw]` = a fact we hand-enter; `[derived]` = computed by
  the engine and never stored. *Store raw facts; derive summaries.*
- **inherit↓ with override** — `[inherit↓]` an attribute can be set at any level,
  flows downhill, and is overridable below. Tames data-entry burden.
- **taxonomy ≠ geometry** — the guidebook tree is for *identity/browsing*;
  coordinates are for *proximity*. Model both, separately.
- **profile ≠ session** — stable climber facts vs. per-day intent. Only the
  session layer changes day to day.
- **missing data is first-class** — guidebook fields are spotty and weather isn't
  knowable until near the trip. "Unknown" is a real value (Rust `Option`), never a
  fake zero.

---

## Part 1 — The world model

### Hierarchy (taxonomy)

`Destination → Region (guidebook) → Area → Sector* → Boulder → Problem`  (`*` = recursive: 0+ nested levels)

- **Destination** = a climbing destination (Lake Tahoe now; Bishop, etc. later) —
  the top level, holding regions. The model can hold multiple destinations; the
  destination is supplied per-import (file-level), not per-row.
- **Region** = the 5 Tahoe guidebooks (North / East / West / South Shore +
  Outlying).
- Used for *identity and browsing only* — physical closeness is handled by
  coordinates, because areas bleed across guidebook boundaries.
- **Below `Area`, subdivision is recursive — a self-nesting `Sector`.** Depth is
  variable and *differs by destination* (Tahoe vs. Bishop, etc.), so it's modeled
  as data, not fixed levels: a `Sector` holds nested child `Sector`s **and/or**
  boulders, to any depth. `Sector.name` is `Option<String>` — `Some` for a real
  guidebook subarea/sub-subarea (the norm), `None` for a lone-boulder passthrough
  whose label is **derived** from its sole boulder (the rare outlier). `Region` and
  `Area` remain fixed top levels (reliably present, meaningful for display/
  planning). *(see Decisions Log)*

### Boulder — anchors *space*

- `coordinates` **[raw, optional]** — `Option<Coordinates>` (lat/long as `f64`).
  Often absent in guidebooks (sourced from Mountain Project / measurement later).
  Double duty: proximity *and* the sun-math input.
- `aspect` **[raw, optional, inherit↓ to problems]** — `Option<Aspect>`, where
  `Aspect` is **degrees** (`u16`, 0–359) so the sun calc can use it directly;
  compass words (NW) convert *in* and a compass label can derive *out*. Both
  `Coordinates` and `Aspect` are value objects in the `location` module.
- `elevation` **[raw, optional]** — minor; affects sun/temperature slightly.

### Problem — the unit people climb

- `name` **[raw, optional]** — `Option<String>`. `None` for an unnamed open
  project (or unlisted line); display label then derives from the boulder + the
  `number` natural key (same pattern as a passthrough `Sector`). Optionality is
  independent of `status` (established lines can be unnamed too).
- **Grade and climbed-status are two orthogonal fields** (not one):
  - `grade` **[raw]** — `Option<VGrade>` on the V-scale. `None` = grade unknown
    (covers "climbed but ungraded" *and* "not yet recorded"). `VGrade` ranges
    `VB` (easier than V0) through `V0..=V18`. *Range/uncertainty (e.g. "V4/5") is
    deferred — a nice-to-have.* Font scale is a later **derived** mapping, not stored.
  - `status` **[raw]** — `Established` vs. `OpenProject` (the line hasn't been
    sent yet). Kept separate from grade so an open project *with a proposed grade*
    is representable.
- `number` **[raw, optional]** — the *guidebook* number, local to its boulder
  (Boulder B's "2"). A **natural/display** identifier, kept separate from the
  internal `ProblemId` surrogate (storage/joins). Stored, not derived from list
  position. May need a letter suffix (2a/2b) for variations — model later if needed.
- `aspect` **[raw, overrides boulder]** — the sunny-face / shady-face case.
- `steepness` **[raw, optional]** — *double duty:* a sun-geometry input **and** a
  style signal (slab vs. overhang).
- `style_tags` (crimps / slopers / steep / slab / …) **[raw, optional]** — needed
  so style-matching has something to compare against; guidebooks are spotty.
- `quality` / stars **[raw, optional]**
- `features` **[raw, optional set]** — see below.
- `sun_exposure(date, time)` **[derived]** — from aspect + steepness +
  coordinates + solar position. Built instead of trusting "sunny till noon."

### Problem features (optional tags)

A problem may have **any combination (or none)** of:

- `bad_landing` / pad-heavy
- `highball`
- `height_dependent`
- `technical`
- `seepage_prone` / slow-drying *(candidate — see Open Decisions #7)*

These feed into **preferences** (Part 2): some are soft preferences, some are
**vetoes** (e.g. a 5-star highball is worthless to someone who won't climb high
off the deck).

### Travel (was "Approach")

Total time-to-climb has **two legs of different character** and both matter
independently (limited time → "long drive / short walk" vs. "short drive / long
walk" is a real trade-off):

- `start_location` — the day's origin (e.g. the booked BnB) **[raw, per trip/day]**
- **Leg 1 — drive:** `drive_time(start → parking)` — by car.
  - *Open decision:* computed via a routing API vs. hand-entered. [#2]
- **Leg 2 — approach:** on foot, accumulates *down* the hierarchy.
  - `approach_minutes_from_parent` on each container node **[raw]**
  - `total_approach(node)` = sum along the path **[derived]**
  - root of the chain is a `Parking` / trailhead attached to an Area.
- `total_travel = drive_time + total_approach` **[derived]**

---

## Part 2 — People & the day

### ClimberProfile (stable) **[raw]**

- `max_grade`, `flash_grade`
- `style_preferences` — per-style like/dislike weighting.
- `feature_preferences` — per-feature, expressing **both** a soft weight **and**
  an optional **hard veto** (e.g. "never highballs").

### SessionIntent (per climber, **per day** — the interactive knob)

- `desired_range` + `mode` (*cruise* / *project* / *flexible*) — mode reshapes
  what "good grade fit" means (project rewards at/above max; cruise rewards below).
- `weight` — this person's pull in today's group decision.
- `rest_bar` — minimum area quality below which they opt *out* rather than drag
  the group along.

> The three day-types (chase double-digits, prioritize one project, last-day
> egalitarian) are **one weighted mechanism** with `weight` + `rest_bar` tuned
> differently — not separate algorithms.

### DayContext (shared, per day)

- `date` → drives sun position (and season).
- `start_location` (see Travel).
- `weather` **[raw, or derived from a weather API]** — `temperature`,
  `cloud_cover`, `precip`, `recent_precip` (for dryness).
  - **Unknown until ~10 days out** for the October trip → must be manually
    overridable and gracefully absent early.
- `sun_preference` — *signed*: want-sun / want-shade / neutral. Derivable from
  temperature; **cloud cover dampens the weight of sun entirely.**

---

## Part 3 — The engine (all `[derived]`)

1. `problem_score(problem, climber, intent)`
   = grade-fit-given-mode × style-fit × **feature-fit (a veto zeroes it)** × quality
2. `area_score(area, climber, intent, date, time)`
   = aggregate of problems in/under it *(← climb **density** emerges here; never
   stored)* × (`sun_exposure` × `sun_preference` × cloud factor) × dryness
   − `total_travel` cost
3. `group_area_score`
   = **weighted combine** of per-climber `area_score`s, applying `rest_bar`
   opt-outs. *(All group tunability lives here.)*
4. `day_plan`
   = sequence areas across time blocks to chase shade as the sun moves, respect
   travel time, and maximize group scores.
   *Start simple: rank areas per time block. Real sequencing is a later upgrade.*

---

## Open decisions (to resolve in upcoming rounds)

1. ~~Skipped / variable hierarchy levels~~ — **RESOLVED:** subdivision below
   `Area` is a recursive, self-nesting `Sector` (any depth); lone boulders become
   passthrough sectors with `name: None`. See log.
2. **Drive time** — computed via routing API vs. hand-entered.
3. **Horizon obstruction** (tree / neighboring-boulder shade) — a v2 refinement
   of the sun model.
4. **Style & feature metadata entry burden** — how much to enter per problem.
5. **Weather** — which API; how to handle "forecast not available yet."
6. ~~Grade representation~~ — **RESOLVED:** V-scale primary; Font a derived
   mapping later; ranges deferred (nice-to-have). See log.
7. **Dryness** — derive from aspect + steepness, vs. an explicit `seepage_prone`
   tag, vs. both.
8. **`sun_preference`** — manual vs. derived-from-temperature.

---

## Decisions log

- **Language:** Rust.
- **Architecture:** engine (pure data model + scoring) separated from interface
  (CLI now; web/mobile later).
- **Sun:** computed from geometry, never stored; implement the math ourselves and
  validate against a crate + guidebook hints.
- **Taxonomy vs. geometry:** guidebook hierarchy for identity; coordinates for
  proximity.
- **Group aggregation:** one tunable weighted mechanism — per-person `weight` +
  `rest_bar` opt-out — covers all day-types.
- **Travel:** split into drive (car) + approach (foot), both costed independently.
- **Preferences:** support both soft weights and hard vetoes.
- **Sun preference is signed:** sun can be desirable or undesirable; cloud cover
  dampens its relevance.
- **Recursive subdivision below `Area` (`Sector`):** the hierarchy is `Region →
  Area → Sector* → Boulder → Problem`, where `Sector` self-nests to *any* depth
  (subarea, sub-subarea, …) because depth is unknown and **varies by destination**.
  A `Sector` holds child `Sector`s and/or boulders. `Sector.name: Option<String>` —
  `Some` for a real named subdivision (the norm), `None` for a lone-boulder
  passthrough (label derived from its sole boulder). `Region`/`Area` stay fixed.
  Recursion is cheap *because* we chose arena + IDs (a `Sector` holds `SectorId`s,
  not nested values — no borrow-checker pain). Supersedes both the earlier
  "subarea is optional" and "subarea always present (fixed)" decisions.
- **Grade:** V-scale is primary; Font is a derived mapping added later (not
  stored); range/uncertainty deferred as a nice-to-have.
- **Grade vs. climbed-status are two fields:** `grade: Option<VGrade>` (VB through
  V18; `None` = unknown) and `status: Established | OpenProject`. They're
  orthogonal axes, so they don't belong in one enum.
- **Project is a library (`--lib`) crate:** the engine is the core; CLI/other
  interfaces come later as separate front-ends.
- **Storage/identity:** arena + ID pattern (flat per-type stores; reference by an
  ID newtype) chosen over an ownership tree — because taxonomy and geometry are
  cross-cutting relationships over the same boulders. Identity = the ID newtype.
- **Destination is the top domain level; `Catalog` is the storage aggregate (two
  distinct things):** hierarchy is `Destination → Region → Area → Sector* →
  Boulder → Problem`. `Catalog` is the technical container holding every per-type
  store (incl. `DestinationStore`) — *not* a hierarchy node, and able to hold more
  than one destination. Naming the aggregate after a domain level would conflate
  storage with the tree. Destination is supplied per-import (file-level), not per-row.
- **Two kinds of identity (surrogate vs. natural key):** internal `*Id` newtypes
  are system-generated **surrogate** handles (stable, globally unique, for
  storage/joins). Guidebook identifiers — a problem's per-boulder `number`, a
  boulder's `name` — are **natural/display** keys, stored as ordinary attributes.
  Never conflate them (same spirit as taxonomy ≠ geometry).
- **Storage is a generic `Arena<T>` + `Id<T>`:** the six per-type stores collapsed
  into one generic arena and a phantom-typed `Id<T>` (kept type-safe so
  `Id<Boulder>` ≠ `Id<Problem>`); `XStore`/`XId` are type aliases. Refactored under
  green tests.
- **Sun computed in UTC; timezone/DST handled at the boundary.** `calculate` takes
  a `DateTime<Utc>`; converting a user's local (PST/PDT) time to UTC is the
  caller's job via a tz-aware library (`chrono-tz` / `America/Los_Angeles`), so the
  calc stays pure and DST-immune.
- **Representation follows use:** the *face* `Aspect` is `u16` whole degrees
  (coarse, hand-entered); the *sun's* azimuth/elevation are `f64` (computed,
  precise). Same unit, different types, because the use differs.
- **`SunCover` = `Sun`/`Glancing`/`Shade`:** decided by (1) an elevation > 0 gate,
  then (2) the *shortest cyclic* distance between sun azimuth and face aspect
  (`<45°` Sun, `<90°` Glancing, else Shade). Assumes vertical faces (steepness
  deferred). Geometry lives on `SunPosition::cover_for(aspect)`; the per-problem
  version is a caller-side join (the sun calc needs the boulder's coords + the
  problem's effective aspect — the one-way-reference pattern again). Coarse on
  purpose: more granularity would need inputs (incidence intensity, steepness,
  cloud cover) we haven't built.
