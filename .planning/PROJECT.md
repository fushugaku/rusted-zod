# Z.O.D. Engine TypeScript Port

## What This Is

A faithful TypeScript port of the Z.O.D. (Zod) real-time strategy game engine, originally written in C++. The project recreates the classic RTS gameplay using Phaser 3, targeting browser-based play with exact mechanical parity to the original engine. Currently ~50% complete with core systems working.

## Core Value

**Exact mechanical parity with the original C engine.** Every gameplay mechanic must match the source code in `./source/`. When implementing any feature, ALWAYS read the relevant C file first to ensure the TypeScript implementation produces identical behavior.

## Requirements

### Validated

<!-- Shipped and working -->

- Core game loop with Phaser scene architecture — existing
- All unit types implemented (6 robots, 7 vehicles, 4 cannons, 8 buildings) — existing
- Combat system with hit chance, damage calculation, snipe mechanics — existing
- Waypoint/movement system with pathfinding (A*) — existing
- Zone capture and territory control — existing
- Production system with build queues — existing
- AI system with threat assessment, strategy modes, difficulty levels — existing
- Minimap with unit dots and zone colors — existing
- Projectile visuals (missiles, lasers, flames, bullets, grenades) — existing
- Selection system with control groups, attack-move, patrol, guard — existing
- Running/stamina system for robots — existing
- Damage speed modifiers (slow when damaged) — existing
- Vehicle lid open/close animations — existing
- Cannon turret rotation — existing
- Robot idle animations with original timing — existing
- Area damage falloff matching original formula — existing
- ENTER_FORT and ENTER_VEHICLE waypoint modes — existing
- Repair systems (CRANE_REPAIR, UNIT_REPAIR) — existing

### Active

<!-- Current milestone: Single-player complete -->

- [ ] Sprite atlas loading with team color tinting
- [ ] All unit animations (walk, attack, death for all types)
- [ ] Sound asset loading and playback
- [ ] Grenade system (pickup boxes, inventory, throwing, AOE)
- [ ] Driver health system (separate from vehicle health)
- [ ] Production modifiers (zone ownership bonus, damage penalty)
- [ ] Fort firing mechanics with garrisoned units
- [ ] Track effects rendering for vehicles
- [ ] Death animation variants (4 robot death types)
- [ ] Repair visual effects (sparks/wrench)
- [ ] Fog of war on minimap
- [ ] Unit portrait system in HUD
- [ ] Unit info panel with detailed stats
- [ ] Production UI improvements (rally points, progress bars, queue management)
- [ ] Messages/announcements system with computer voice
- [ ] Victory/defeat screens with stats
- [ ] Map selection screen
- [ ] Team selection UI
- [ ] Game settings screen
- [ ] Animal spawning from huts
- [ ] Crane construction visual effect

### Out of Scope

- Multiplayer/networking — deferred to future milestone (architecture supports it)
- Mobile touch controls — web/desktop first
- New units not in original — faithful port only
- Custom maps editor — use original map format
- Replay system — nice to have, not core

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| Sound asset loading and playback | Phase 1 | Pending |
| Messages/announcements system with computer voice | Phase 1 | Pending |
| Sprite atlas loading with team color tinting | Phase 2 | Pending |
| All unit animations (walk, attack, death for all types) | Phase 3 | Pending |
| Death animation variants (4 robot death types) | Phase 3 | Pending |
| Grenade system (pickup boxes, inventory, throwing, AOE) | Phase 4 | Pending |
| Driver health system (separate from vehicle health) | Phase 4 | Pending |
| Fort firing mechanics with garrisoned units | Phase 4 | Pending |
| Repair visual effects (sparks/wrench) | Phase 4 | Pending |
| Production modifiers (zone ownership bonus, damage penalty) | Phase 5 | Pending |
| Production UI improvements (rally points, progress bars, queue management) | Phase 5 | Pending |
| Crane construction visual effect | Phase 5 | Pending |
| Unit portrait system in HUD | Phase 6 | Pending |
| Unit info panel with detailed stats | Phase 6 | Pending |
| Victory/defeat screens with stats | Phase 7 | Pending |
| Map selection screen | Phase 7 | Pending |
| Team selection UI | Phase 7 | Pending |
| Game settings screen | Phase 7 | Pending |
| Animal spawning from huts | Phase 8 | Pending |
| Fog of war on minimap | Phase 8 | Pending |
| Track effects rendering for vehicles | Phase 8 | Pending |

## Context

**Source Material:**
- Original C++ source code in `./source/` (262 files)
- Detailed mechanics documentation in `docs/requirements/`
- Gap analysis tracking parity issues in `docs/requirements/GAP-ANALYSIS.md`

**Current State:**
- ~18,000 lines of TypeScript across 70+ files
- Core gameplay loop functional
- Placeholder graphics used (sprites not yet loaded)
- Sound framework ready, no assets loaded
- Bot AI functional for single-player testing

**Implementation Approach:**
Before implementing ANY feature:
1. Read the relevant C source file(s) in `./source/`
2. Document the exact mechanic in code comments
3. Implement matching TypeScript
4. Verify behavior matches original

## Constraints

- **Tech Stack**: TypeScript + Phaser 3.70 + Vite — already established
- **Browser Target**: ES2020, WebGL/Canvas required
- **Asset Format**: Must use original game assets (sprites, sounds, maps)
- **Mechanical Fidelity**: No "improvements" to original mechanics — match exactly
- **Architecture**: Design for future multiplayer (server-authoritative patterns)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Phaser 3 for rendering | Mature 2D engine, good TypeScript support | Good |
| EasyStar for pathfinding | Simple A* implementation, sufficient for RTS | Good |
| System-based architecture | Separates concerns, testable, matches original structure | Good |
| Always reference C source | Ensures exact parity, prevents drift | Pending |
| Audio first, then atlas | Low risk quick win before complex sprite migration | Pending |
| 8 phases for milestone | Comprehensive depth, follows research recommendations | Pending |

---
*Last updated: 2026-01-25 after roadmap creation*
