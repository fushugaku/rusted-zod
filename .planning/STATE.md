# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-25)

**Core value:** Exact mechanical parity with the original C engine
**Current focus:** Milestone v1.0 Complete!

## Current Position

Phase: 8 of 8 (Environmental Polish)
Plan: 3 of 3 in current phase
Status: ✅ MILESTONE COMPLETE
Last activity: 2026-01-25 - Completed all Phase 8 plans (08-01, 08-02, 08-03)

Progress: [########################] 100% (24/24 plans)

## Performance Metrics

**Velocity:**
- Total plans completed: 24
- Average duration: 6.5 min
- Total execution time: 2.8 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-audio-system | 3/3 | 20 min | 6.7 min |
| 02-texture-atlas | 2/2 | 18 min | 9 min |
| 03-animation-system | 3/3 | 26 min | 8.7 min |
| 04-combat-polish | 4/4 | 20 min | 5 min |
| 05-production-polish | 3/3 | 24 min | 8 min |
| 06-hud-enhancement | 3/3 | 18 min | 6 min |
| 07-game-flow-screens | 3/3 | 26 min | 8.7 min |
| 08-environmental-polish | 3/3 | 17 min | 5.7 min |

**Recent Trend:**
- Last 5 plans: 07-03 (6 min), 08-01 (5 min), 08-02 (4 min), 08-03 (8 min)
- Trend: Steady - Milestone complete!

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Roadmap]: 8 phases derived from 21 requirements, following research recommendation order
- [Roadmap]: Audio first (low risk, quick win) before texture atlas migration
- [01-01]: WAV filename as enum value for direct cache lookup (no mapping table needed)
- [01-01]: Legacy SoundType API preserved for backward compatibility
- [01-01]: VEHICLE_FACTORY removed (duplicate of ROBOT_FACTORY per C source)
- [01-02]: Unit voices are non-positional (UI-level) for consistent feedback
- [01-02]: Light explosions for radius < 40px, full explosions otherwise
- [01-02]: Resume audio context on first pointer interaction (browser autoplay policy)
- [01-03]: Announcement enum separate from SoundEvent for semantic clarity
- [01-03]: 5 second timeout fallback for stuck announcement sounds
- [01-03]: 10 second fort attack announcement cooldown to prevent spam
- [02-context]: Pre-baked atlases per team (not runtime tinting) — matches existing assets
- [02-context]: free-tex-packer for atlas generation (open source, CLI, Phaser export)
- [02-context]: Atlases organized by category (robots, vehicles, cannons, buildings, effects)
- [02-01]: Frame names match SpriteLoader.ts patterns exactly (enables drop-in migration)
- [02-01]: Generated atlases gitignored (regenerate with npm run pack-atlases)
- [02-01]: Neutral sprites packed into red team atlas to avoid duplication
- [02-02]: All setTexture calls use (atlasKey, frameKey) two-arg pattern for Phaser 3 API
- [02-02]: Buildings use layered sprite approach (base + team + overlay) from C source
- [02-02]: Building dimensions from C source logical values, not visual asset dimensions
- [03-01]: State priority system: DYING locked, ATTACKING > WALKING > IDLE_ACTION > IDLE
- [03-01]: Delta time accumulation for frame advancement in AnimationStateMachine
- [03-01]: onAttackFired callback uses optional parameters for flexibility
- [03-02]: Attack animation uses 2 frames at 100ms for turret recoil effect
- [03-02]: Cannon returns to STANDING mode after fire animation completes
- [03-02]: All tanks (Light/Medium/Heavy) have lids per original zvehicle.cpp
- [03-03]: Bullet damage selects random die1-4 (10/10/10/8 frames)
- [03-03]: Flame/Laser damage triggers melt death (17 frames)
- [03-03]: Rocket/Grenade/Cannon damage triggers missile flip (33 frames with arc)
- [03-03]: Robot removal deferred until robotDeathComplete event
- [04-01]: Grenade boxes neutral - can be picked up by any team
- [04-01]: Grenade arc height = 30 pixels, parabolic trajectory
- [04-01]: Grenade AOE damage via existing missile explosion system
- [04-02]: Progressive driver damage via applyDriverDamage callback (not instant kill)
- [04-02]: canBeSniped checks both driver presence AND lid state for lidded vehicles
- [04-02]: Driverless vehicles stop immediately via stop() in updateMovement
- [04-04]: Repair effect uses procedural sparks for performance
- [04-04]: Effects keyed by target refId for multi-target support
- [04-04]: Repair building shows effect at center-top where work happens
- [04-03]: GarrisonedUnit stores full combat state (health, grenades, cooldown)
- [04-03]: FortCombatInfo interface avoids circular imports
- [04-03]: Each garrisoned unit fires independently with its own attack speed
- [04-03]: Missiles from garrison originate at fort center position
- [04-03]: GARRISON_FORT for friendly forts, ENTER_FORT for enemy destruction
- [05-01]: Zone bonus shown as green +X% format when > 0%
- [05-01]: Damage penalty shown as red -X% format when > 1%
- [05-01]: Estimated time formatted as M:SS
- [05-01]: Modifiers updated every frame via GameScene update loop
- [05-02]: Rally button toggles mode, left/right click to set position
- [05-02]: Dashed line connects building to rally flag for visual clarity
- [05-02]: Crosshair cursor indicates rally mode is active
- [05-03]: Procedural graphics for workers, cones, barricade (matches RepairEffect approach)
- [05-03]: Callbacks from WaypointSystem to GameScene (maintains separation of concerns)
- [05-03]: activeCraneRepairs Set to track repair state and prevent duplicate callbacks
- [06-01]: Portrait uses composite layered sprites (backdrop, head, eyes, mouth, shoulders, hand)
- [06-01]: 65 animation sequences with frame timing from C source duration_multi = 0.015
- [06-01]: Random idle animations every 0.5-5.5 seconds
- [06-02]: Grenade icon uses procedural graphics (matches RepairEffect approach)
- [06-02]: Health bar three segments: green (current), yellow (damaged), gray (max lost)
- [06-02]: Driver health blue (0x00aaff) for visual separation from vehicle health
- [06-03]: Panel switching based on selection type (single unit/building/multi)
- [06-03]: Two-letter unit abbreviations for queue slots (GR, PS, SN, TO, PY, LA, etc.)
- [06-03]: Zone ownership horizontal bar with color coding (green=owned, yellow=partial, red=contested)
- [07-01]: Green theme for victory, red theme for defeat screens
- [07-01]: Stats panel with label-value layout for game statistics
- [07-01]: Shift+V and Shift+D keyboard shortcuts for testing end screens
- [07-02]: Hardcoded map list (10 maps) for map selection - can be made dynamic with server
- [07-02]: Team selection integrated into MapSelectionScene rather than separate screen
- [07-02]: MapInfo loaded by parsing .map files directly via MapLoader
- [07-03]: Master volume multiplied with specific volume for effective volume calculation
- [07-03]: Game speed presets: 0.5x, 1.0x, 1.5x, 2.0x via Phaser time.timeScale
- [07-03]: Settings auto-load on module initialization from localStorage
- [08-02]: Fog opacity - UNEXPLORED=85% black, EXPLORED=45% black
- [08-02]: Default sight range 8 tiles, buildings 6 tiles
- [08-02]: Visibility updates throttled to 100ms for performance
- [08-03]: TrackType enum with TANK and JEEP matches etrack.h ET_TANK/ET_JEEP
- [08-03]: updateTrack() method to avoid Phaser Container.update() override conflict
- [08-03]: Event-based track creation via vehicleTrackDrop maintains separation of concerns
- [08-01]: animalState property name to avoid Phaser Container.state conflict
- [08-01]: 80% prefer similar direction for natural animal movement paths
- [08-01]: 128px roam distance from home hut (zsettings default)

### Pending Todos

None yet.

### Blockers/Concerns

- Portrait system HIGH complexity noted in research (65 animation sequences, 11 eye positions)

## Session Continuity

Last session: 2026-01-25
Stopped at: ✅ MILESTONE v1.0 COMPLETE - All 24 plans executed
Resume file: None
