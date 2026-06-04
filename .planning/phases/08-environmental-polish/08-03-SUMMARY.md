---
phase: 08-environmental-polish
plan: 03
subsystem: effects
tags: [track-effects, vehicle, visual-feedback, etrack.cpp]

dependency-graph:
  requires:
    - "08-01: Fog of War foundation"
    - "08-02: Weather effects system"
  provides:
    - "TrackEffect class with 3-stage fade animation"
    - "EffectsSystem.createTrackEffect() method"
    - "Vehicle track marks on terrain"
  affects:
    - "Future: Oil/spark/dirt effects can use similar pattern"

tech-stack:
  added: []
  patterns:
    - "Event-driven effect creation (vehicleTrackDrop event)"
    - "Pooling with MAX limit for performance"
    - "Direction-based sprite positioning from C source"

key-files:
  created:
    - "client/src/effects/TrackEffect.ts"
  modified:
    - "client/src/effects/index.ts"
    - "client/src/effects/EffectsSystem.ts"
    - "client/src/scenes/GameScene.ts"

decisions:
  - id: track-type-enum
    choice: "TrackType enum with TANK and JEEP"
    reason: "Matches etrack.h ET_TANK/ET_JEEP types from C source"
  - id: update-method-name
    choice: "updateTrack() instead of update() to avoid override conflict"
    reason: "Phaser Container.update() signature conflict"
  - id: event-driven-creation
    choice: "Event-based track creation via vehicleTrackDrop"
    reason: "Vehicle.ts already emits events, maintains separation of concerns"

metrics:
  duration: 8 min
  completed: 2026-01-25
---

# Phase 08 Plan 03: Vehicle Track Effects Summary

Track marks appear behind moving vehicles and fade over ~4 seconds, matching the original C engine behavior from etrack.cpp.

## What Was Built

### TrackEffect Class
- Ported from `source/etrack.cpp` with exact timing:
  - 0-3.3s: Full opacity (alpha 0.6)
  - 3.3-3.6s: Medium fade (alpha 0.4)
  - 3.6-3.9s: Fading out (alpha 0.2)
  - 3.9s+: Removed
- Direction-based positioning from `SetTrackCoords()` for all 8 directions
- TrackType enum: TANK (brown) vs JEEP (gray) track colors
- Random jitter (0-2 pixels) matching original randomness

### EffectsSystem Integration
- `createTrackEffect()` method for vehicles to call
- Track effects array with MAX_TRACK_EFFECTS limit (100)
- Automatic cleanup when tracks fade out
- Old tracks removed when limit reached

### GameScene Wiring
- Listens for `vehicleTrackDrop` event from Vehicle.updateTracks()
- Converts VehicleType to TrackType using helper function
- Creates track effects at vehicle position with current direction

## Technical Details

### Track Positioning (from etrack.cpp)
The original C code calculates two track positions (left/right treads) based on direction:
- Direction 0 (East): x1=x2=cx-15, y1=cy-2, y2=cy+10
- Direction 2 (South): y1=y2=cy+15, x1=cx-8, x2=cx+8
- Diagonal directions use offset calculations

### Integration Points
- Vehicle.ts emits `vehicleTrackDrop` event (already implemented)
- Event contains: x, y, direction, vehicleType
- GameScene converts vehicleType to trackType and calls effectsSystem

## Commits

| Hash | Description |
|------|-------------|
| f2d42dd | feat(08-03): add TrackEffect class with fade animation |
| 65d8a00 | feat(08-03): add track effect management to EffectsSystem |
| 07ba6c6 | feat(08-03): wire vehicle track effects to GameScene |

## Verification

- [x] TypeScript compiles without errors
- [x] Build completes successfully
- [x] TrackEffect exports correctly from effects module
- [x] EffectsSystem.createTrackEffect() available
- [x] vehicleTrackDrop event wired in GameScene

## Deviations from Plan

None - plan executed exactly as written.

## Next Phase Readiness

- Track effect pattern can be reused for:
  - Oil drips (TANK_OIL effect)
  - Spark effects (TANK_SPARK effect)
  - Dirt cloud effects (vehicleDirtCloud event already exists)
