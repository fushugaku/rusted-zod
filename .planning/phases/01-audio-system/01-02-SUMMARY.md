---
phase: 01-audio-system
plan: 02
subsystem: audio
tags: [phaser, webaudio, combat-sounds, unit-voices, positional-audio]

# Dependency graph
requires:
  - phase: 01-01
    provides: SoundEvent enum, SoundConfig with volume/rate-limiting, PreloaderScene loading ~120 WAV files
provides:
  - Combat sound integration (weapon fire on attack, explosions on missile impact)
  - Unit voice sounds (yes_sir on selection, move_ack on commands)
  - Positional audio with camera-based culling
  - playWeaponFire() and playUnitVoice() methods in SoundSystem
affects:
  - 01-03 (computer voice announcements)
  - Any phase needing audio feedback for game events

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Combat sound trigger: setOnAttackEffect callback -> playWeaponFire()"
    - "Explosion sound trigger: missileExplosion event -> playRandomExplosion()"
    - "Selection sound: onSelectionChanged callback -> playUnitVoice('yes_sir')"
    - "Command sound: right-click handler -> playUnitVoice('move_ack')"

key-files:
  created: []
  modified:
    - client/src/sound/SoundTypes.ts
    - client/src/sound/SoundSystem.ts
    - client/src/scenes/GameScene.ts

key-decisions:
  - "Unit voices are non-positional (UI-level) for consistent feedback"
  - "Light explosions for radius < 40px, full explosions otherwise"
  - "Resume audio context on first pointer interaction (browser autoplay policy)"

patterns-established:
  - "playWeaponFire(): Positional weapon sounds via playSoundRestricted()"
  - "playUnitVoice(): Non-positional voice lines with voice type parameter"
  - "UNIT_TYPE_REPORTING_SOUNDS: RobotType -> specific reporting voice mapping"

# Metrics
duration: 2min
completed: 2026-01-25
---

# Phase 1 Plan 2: Combat Sounds Integration Summary

**Weapon fire and unit voice sounds wired to combat system and selection handlers, making battles audible with positional audio**

## Performance

- **Duration:** 2 min
- **Started:** 2026-01-25T11:34:30Z
- **Completed:** 2026-01-25T11:36:30Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- Combat system triggers weapon sounds on attack via setOnAttackEffect callback
- Missile explosions play random explosion sounds (light or full based on radius)
- Unit selection plays "Yes sir" acknowledgment, commands play "move_ack"
- Camera position updates each frame for correct positional audio falloff

## Task Commits

Each task was committed atomically:

1. **Task 1: Add weapon and explosion sound methods** - `2154f83` (feat)
2. **Task 2: Wire CombatSystem to trigger sounds** - `889d68c` (feat)
3. **Task 3: Wire selection and commands to play voices** - `dbdd628` (feat)

## Files Created/Modified
- `client/src/sound/SoundTypes.ts` - Added UNIT_TYPE_REPORTING_SOUNDS mapping (RobotType -> voice)
- `client/src/sound/SoundSystem.ts` - Added playWeaponFire() and playUnitVoice() methods
- `client/src/scenes/GameScene.ts` - Wired combat callbacks, selection sounds, command sounds, camera position updates

## Decisions Made
- **Non-positional unit voices:** Unit acknowledgments are UI-level sounds (not positional) for consistent feedback regardless of camera position
- **Light vs full explosions:** Radius < 40px uses light explosion sounds (first 2 variants), larger uses all 5 variants
- **Audio context resume:** Called on both left-click and right-click to ensure browser autoplay policy is satisfied on first interaction

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None - all tasks completed without issues.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Combat sounds working for all unit types (robots, vehicles, cannons)
- Explosion sounds trigger on missile impact
- Unit voices play on selection and commands
- Ready for 01-03 computer voice announcements system
- Rate limiting from 01-01 prevents sound spam on rapid-fire units

---
*Phase: 01-audio-system*
*Completed: 2026-01-25*
