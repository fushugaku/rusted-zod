---
phase: 04-combat-polish
plan: 02
subsystem: combat
tags: [snipe, driver-health, vehicle, typescript]

# Dependency graph
requires:
  - phase: 03-animation-system
    provides: Death animations and state machine for combat feedback
provides:
  - Driver health tracking separate from vehicle health
  - Snipe mechanic for killing drivers
  - Driverless vehicle behavior (immobile, cannot attack)
  - Vehicle state export for network sync
affects: [04-03, 05-ui, network-sync]

# Tech tracking
tech-stack:
  added: []
  patterns: [callback-based damage application, state export for network sync]

key-files:
  created: []
  modified:
    - client/src/objects/units/Vehicle.ts
    - client/src/combat/CombatSystem.ts

key-decisions:
  - "Progressive driver damage via applyDriverDamage callback (not instant kill)"
  - "canBeSniped checks both driver presence AND lid state for lidded vehicles"
  - "Driverless vehicles stop immediately via stop() in updateMovement"

patterns-established:
  - "Driver health separate from vehicle health for sniper value"
  - "Callback pattern for cross-system damage application"

# Metrics
duration: 5min
completed: 2026-01-25
---

# Phase 04 Plan 02: Driver Health System Summary

**Driver health tracking with snipe damage to driver, driverless vehicle behavior, and state export for network sync**

## Performance

- **Duration:** 5 min
- **Started:** 2026-01-25T17:04:47Z
- **Completed:** 2026-01-25T17:10:02Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- Driver health tracked separately from vehicle health
- Snipe mechanic damages driver health via callback
- Driverless vehicles cannot move or attack
- State export includes driver health for network sync
- Visual indicator for driverless state (red X)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add driver health getter methods** - `c88dd27` (feat)
2. **Task 2: Update CombatSystem snipe damage** - `2422b19` (feat)
3. **Task 3: Add driverless vehicle behavior** - `8e6ec32` (feat)

## Files Created/Modified

- `client/src/objects/units/Vehicle.ts` - Added driver health getters, driverless behavior, canEnterAsDriver/enterAsDriver, getState()
- `client/src/combat/CombatSystem.ts` - Added canBeSniped/driverHealth to UnitInfo, applyDriverDamage callback, updated snipe logic

## Decisions Made

1. **Progressive driver damage via callback** - Plan called for applyDriverDamage callback that returns boolean for driver kill. This supports both instant kills (high damage) and progressive damage over multiple hits.

2. **canBeSniped respects lid state** - Changed snipe check from `hasDriver` to `canBeSniped` which internally checks both driver presence AND lid state for lidded vehicles (Light/Medium/Heavy tanks).

3. **Driverless vehicles stop immediately** - When driver is killed, `updateMovement()` now calls `stop()` immediately rather than letting movement complete. This matches original zvehicle.cpp behavior.

## Deviations from Plan

None - plan executed exactly as written.

Most of the driver health infrastructure was already in place from previous work:
- Vehicle.ts already had driverHealth/driverMaxHealth properties
- Vehicle.ts already had setInitialDriver(), damageDriver(), killDriver() methods
- Vehicle.ts already had canBeSniped() method
- CombatSystem.ts already had onDriverSniped callback

This plan added the missing pieces:
- Getter methods for driver health
- applyDriverDamage callback for progressive damage
- canBeSniped in UnitInfo for proper lid state checking
- Driverless vehicle movement/attack restrictions
- canEnterAsDriver/enterAsDriver for future ENTER_VEHICLE waypoint
- getState() for network synchronization

## Issues Encountered

1. **Linter conflicts** - ESLint auto-fixed some property references inconsistently (adding underscore prefix then removing it). Resolved by re-applying edits and verifying compilation.

2. **Missing ObjectType import** - getState() needed ObjectType enum which wasn't imported. Added to import statement.

3. **Incorrect property names** - Used `selectable` and `selected` instead of `isSelectable()` and `isSelected()` methods. Fixed by calling the methods.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Driver health system complete and ready for UI display
- Snipe mechanic functional for combat scenarios
- Network sync includes driver state for multiplayer
- ENTER_VEHICLE waypoint can use canEnterAsDriver/enterAsDriver when implemented

---
*Phase: 04-combat-polish*
*Completed: 2026-01-25*
