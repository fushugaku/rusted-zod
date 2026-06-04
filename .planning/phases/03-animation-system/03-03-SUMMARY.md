---
phase: 03-animation-system
plan: 03
subsystem: animation
tags: [death-animation, missile-type, robot, phaser, combat]

# Dependency graph
requires:
  - phase: 03-01
    provides: AnimationStateMachine with state priority and DYING lock
  - phase: 03-02
    provides: Cannon/vehicle animation patterns
provides:
  - DeathAnimationSystem for spawning death effects
  - Robot death animation selection based on MissileType
  - Missile flip arc trajectory from erobotturrent.cpp
  - Integration with CombatSystem onUnitDestroyed
affects: [04-unit-mechanics, effects-system]

# Tech tracking
tech-stack:
  added: []
  patterns: [death-type-selection, arc-trajectory-physics, deferred-unit-removal]

key-files:
  created:
    - client/src/animation/DeathAnimationSystem.ts
  modified:
    - client/src/animation/index.ts
    - client/src/objects/units/Robot.ts
    - client/src/scenes/GameScene.ts

key-decisions:
  - "Bullet damage selects random die1-4 (10/10/10/8 frames)"
  - "Flame/Laser damage triggers melt death (17 frames)"
  - "Rocket/Grenade/Cannon damage triggers missile flip (33 frames with arc)"
  - "Robot removal deferred until death animation completes via robotDeathComplete event"
  - "Missile flip renders via DeathAnimationSystem (separate from robot sprite)"

patterns-established:
  - "Death type selection: MissileType enum determines DeathType enum"
  - "Deferred removal: onUnitDestroyed triggers animation, robotDeathComplete removes"
  - "Arc trajectory: parabolic flight with scale increase from erobotturrent.cpp"

# Metrics
duration: 12min
completed: 2026-01-25
---

# Phase 03 Plan 03: Death Animation System Summary

**Death animation system with MissileType-based selection (die1-4, melt, missile flip) and arc trajectory from C++ source**

## Performance

- **Duration:** 12 min
- **Started:** 2026-01-25T10:00:00Z
- **Completed:** 2026-01-25T10:12:00Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- DeathAnimationSystem spawns appropriate death effects based on killing blow type
- Robot.playDeathAnimation() locks state machine and plays correct animation
- Missile flip (die5) renders separately with parabolic arc trajectory from erobotturrent.cpp
- CombatSystem integration defers robot removal until animation completes

## Task Commits

Each task was committed atomically:

1. **Task 1: Create DeathAnimationSystem** - `3ec85f7` (feat)
2. **Task 2: Integrate death animations with Robot class** - `d80ceb5` (feat)
3. **Task 3: Wire death animations to CombatSystem** - `be0fb88` (feat)

## Files Created/Modified
- `client/src/animation/DeathAnimationSystem.ts` - Death type enum, frame counts, DeathEffect class with arc trajectory
- `client/src/animation/index.ts` - Export DeathAnimationSystem
- `client/src/objects/units/Robot.ts` - isDying state, playDeathAnimation(), melt animation registration
- `client/src/scenes/GameScene.ts` - DeathAnimationSystem instance, handleUnitDestroyed(), getKillingBlowType()

## Decisions Made
- Death type selection based on MissileType: FLAME/LASER -> melt, ROCKET/GRENADE/CANNON -> missile flip, BULLET -> random die1-4
- Missile flip uses DeathAnimationSystem for rendering (robot sprite hidden) to support arc trajectory
- Robot removal waits for robotDeathComplete event to ensure full animation plays
- Frame timing from AnimationConstants: 160ms standard, 50ms in air / 150ms landing for missile flip

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - TypeScript compiled cleanly, build succeeded.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Death animations fully integrated with CombatSystem
- Phase 3 (Animation System) now complete
- Ready for Phase 4 (Unit Mechanics) - grenade throwing, vehicle driver ejection, etc.

---
*Phase: 03-animation-system*
*Completed: 2026-01-25*
