---
phase: 04-combat-polish
plan: 01
subsystem: combat
tags: [grenade, pickup, waypoint, robot, combat, projectile]

# Dependency graph
requires:
  - phase: 03-01
    provides: AnimationStateMachine for throw animation
  - phase: 03-03
    provides: MissileType.GRENADE for death selection
provides:
  - GrenadeBox pickupable map item
  - Robot grenade inventory methods (addGrenades, canThrowGrenades)
  - PICKUP_GRENADES waypoint handling
  - createGrenadeMissile() for grenade throwing
  - Grenade arc trajectory calculation
affects: [unit-mechanics, combat-effects]

# Tech tracking
tech-stack:
  added: []
  patterns: [grenade-arc-trajectory, pickup-item-waypoint]

key-files:
  created:
    - client/src/objects/items/GrenadeBox.ts (existed before plan execution)
  modified:
    - client/src/objects/units/Robot.ts
    - client/src/objects/units/robots/Grunt.ts
    - client/src/combat/CombatSystem.ts

key-decisions:
  - "Grenade boxes are neutral - can be picked up by any team"
  - "Robots can only pick up grenades if grenadeAmount < maxGrenades"
  - "Grenade arc height = 30 pixels, parabolic trajectory"
  - "Grenade random offset = 24 pixels for target inaccuracy"
  - "Grenade AOE damage with distance falloff via existing missile explosion system"

patterns-established:
  - "Pickup waypoint: move to item, trigger callback, destroy item"
  - "Arc trajectory: 4 * maxHeight * progress * (1 - progress)"

# Metrics
duration: 3min
completed: 2026-01-25
---

# Phase 04 Plan 01: Grenade System Summary

GrenadeBox pickups, robot inventory, throwing with arc trajectory, AOE damage

## What Was Built

1. **GrenadeBox Map Item** (already existed)
   - 16x16 pixel pickupable item
   - Default 5 grenades per box (from zsettings)
   - Fallback graphics (olive green box with yellow center)
   - canPickup() checks if amount > 0
   - pickup() returns amount and destroys box

2. **Robot Grenade Methods**
   - `addGrenades(amount)` - adds to inventory with max cap
   - `canPickupGrenades()` - true if grenadeAmount < maxGrenades
   - `canThrowGrenades()` - true if grenadeAmount > 0
   - `throwGrenade()` - decrements count, plays throw animation

3. **PICKUP_GRENADES Waypoint**
   - Already implemented in WaypointSystem
   - Robot moves to grenade box
   - Triggers onUnitPickupGrenades callback
   - Pickup animation based on facing direction

4. **Combat System Grenade Support**
   - `createGrenadeMissile()` - public method for robot throwing
   - `getGrenadeMissilePosition()` - returns x, y, height for arc rendering
   - Exported `GRENADE_SETTINGS` with arcHeight and randomOffset
   - Grenade explosions use existing missileExplosion event

## Key Files

| File | Changes |
|------|---------|
| `Robot.ts` | Added addGrenades(), canThrowGrenades() methods |
| `Grunt.ts` | Added override modifier to canThrowGrenades() |
| `CombatSystem.ts` | Added createGrenadeMissile(), getGrenadeMissilePosition(), exported GRENADE_SETTINGS |

## Deviations from Plan

None - plan executed as written. GrenadeBox.ts already existed with full implementation before plan execution.

## Verification

- [x] TypeScript compiles without errors
- [x] GrenadeBox class exported from items module
- [x] ItemType.GRENADES_ITEM exists in enums
- [x] WaypointMode.PICKUP_GRENADES exists in enums
- [x] Robot.addGrenades() and canPickupGrenades() implemented
- [x] CombatSystem.createGrenadeMissile() creates grenade projectile

## Next Phase Readiness

Phase 04 Plan 02 (Sniper Driver Kill) can proceed. All grenade system dependencies are in place.
