---
phase: 03-animation-system
plan: 01
subsystem: animation
tags: [animation, state-machine, robot, typescript]
dependency-graph:
  requires: [02-02]
  provides: [AnimationStateMachine, robot-animation-integration]
  affects: [03-02, 03-03]
tech-stack:
  added: []
  patterns: [state-machine, delta-time-accumulation, callback-pattern]
key-files:
  created:
    - client/src/animation/AnimationStateMachine.ts
  modified:
    - client/src/animation/index.ts
    - client/src/objects/units/Robot.ts
    - client/src/objects/GameObject.ts
    - client/src/objects/units/Cannon.ts
    - client/src/objects/units/Vehicle.ts
decisions:
  - id: anim-state-priority
    choice: State priority system prevents invalid transitions
    rationale: DYING locked, ATTACKING > WALKING > IDLE_ACTION > IDLE
  - id: delta-accumulation
    choice: Delta time accumulation for frame advancement
    rationale: Handles variable frame rates, catches up frames if needed
  - id: onAttackFired-callback
    choice: Optional parameters for target coordinates
    rationale: Robot uses coords for facing, Cannon/Vehicle don't need them
metrics:
  duration: 4 min
  completed: 2026-01-25
---

# Phase 3 Plan 1: Robot Animation State Machine Summary

**One-liner:** AnimationStateMachine class with state priority, delta accumulation, and Robot integration for walk/fire/idle animations

## What Was Built

### AnimationStateMachine class
Central animation state controller with:
- **AnimationState enum**: IDLE, WALKING, ATTACKING, DYING, IDLE_ACTION
- **State priority system**: Higher priority states can interrupt lower ones
- **Locked state**: DYING cannot be transitioned out of
- **Delta time accumulation**: Smooth frame advancement regardless of frame rate
- **Animation config**: frameCount, frameDuration, loop, onComplete callback
- **resetFrame()**: Restart animation for visual feedback (attack kicks)

### Robot Integration
- Initialized with all robot animations from AnimationConstants.ts
- Maps RobotAnimation enum to AnimationState and animation keys
- animationFrame getter reads from state machine
- Walk: 4 frames at 100ms
- Fire: robot-type specific frame counts
- Idle actions: beer, cigarette, scan, stretch with onComplete callbacks

### Attack Animation Callback
- Added onAttackFired() to GameObject base class
- Robot override resets fire animation frame for visual kick effect
- Updated Cannon/Vehicle with override modifier for existing methods

## Technical Details

### State Transition Rules
```
Priority Order (low to high):
  IDLE (0) -> IDLE_ACTION (1) -> WALKING (2) -> ATTACKING (3) -> DYING (100)

- Higher can interrupt lower
- IDLE can always be entered (reset)
- DYING is locked (no transitions out)
```

### Frame Timing
```typescript
// Delta accumulation handles variable framerates
while (this.frameTimer >= config.frameDuration) {
  this.frameTimer -= config.frameDuration;
  this.currentFrame++;
  // Handle loop or completion
}
```

### Animation Registration Example
```typescript
this.animStateMachine.registerAnimation('walk', {
  frameCount: ROBOT_ANIMATION_FRAME_COUNTS.WALK, // 4
  frameDuration: ROBOT_ANIMATION_TIMING.WALK_FRAME_TIME, // 100ms
  loop: true,
});
```

## Commits

| Hash | Type | Description |
|------|------|-------------|
| b74ae62 | feat | Add AnimationStateMachine class |
| bfa821b | feat | Integrate AnimationStateMachine into Robot class |
| 5ba2849 | feat | Wire attack animation to CombatSystem callbacks |

## Files Changed

| File | Change |
|------|--------|
| client/src/animation/AnimationStateMachine.ts | Created - 270 lines |
| client/src/animation/index.ts | Added export |
| client/src/objects/units/Robot.ts | Integrated state machine |
| client/src/objects/GameObject.ts | Added onAttackFired callback |
| client/src/objects/units/Cannon.ts | Added override modifier |
| client/src/objects/units/Vehicle.ts | Added override modifier |

## Verification

- [x] TypeScript compiles without errors
- [x] Build succeeds (`npm run build`)
- [x] AnimationStateMachine exported from animation module
- [x] Robot uses state machine for animation control
- [x] onAttackFired callback chain established

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Override modifier required for existing methods**
- **Found during:** Task 3
- **Issue:** Adding onAttackFired to GameObject caused TS4114 errors for existing Cannon/Vehicle methods
- **Fix:** Added `override` modifier to existing onAttackFired methods
- **Files modified:** Cannon.ts, Vehicle.ts
- **Commit:** 5ba2849

## Next Phase Readiness

Ready for Plan 03-02 (Vehicle/Cannon Animation):
- AnimationStateMachine reusable for other unit types
- Pattern established for animation registration and state transitions
- onAttackFired callback available for all units
