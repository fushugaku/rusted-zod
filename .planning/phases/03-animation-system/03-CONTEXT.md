# Phase 3: Animation System - Context

**Created:** 2026-01-25
**Phase:** 03-animation-system

## Vision

All units display proper animations for movement, combat, and death - matching original C engine frame timing exactly. The animation system must feel authentic to the original Z.O.D. game.

## Current State

### Already Implemented (from code review):
- `AnimationConstants.ts` - Full timing constants from original engine:
  - Robot walk/idle/death frame counts and timing
  - Vehicle lid, track, turret rotation timing
  - Cannon placement and fire animation timing
  - All values ported from original C++ source
- `Robot.ts` base class:
  - Animation state machine with `RobotAnimation` enum (STAND, WALK, FIRE, idle anims, death)
  - Frame counts defined in `ROBOT_ANIMATION_FRAMES`
  - `updateAnimation()` method with timing
  - `getSpriteKey()` generates correct atlas frame keys
  - Idle behavior system from `zrobot.cpp` (1/10 chance to animate, 1/3 turn vs action)
- `Vehicle.ts` base class:
  - Turret rotation (independent of hull)
  - Lid animation (open/close with delays)
  - Track frame cycling
  - Damage state visualization
- `Cannon.ts` base class:
  - Placement animation
  - Fire animation
  - Turret rotation in ROTATING_MODE
- Robot subclasses (Grunt, Psycho, etc.) - define fire frame counts
- Atlases generated with correct frame naming

### Needs Implementation:
1. **Animation triggering from game events** - CombatSystem fires `onUnitDestroyed` but Robot doesn't play death animation
2. **Death animation selection by damage type** - 4 standard die types + melt death + missile flip
3. **Attack animation integration** - Fire animation should play when CombatSystem processes attacks
4. **Muzzle flash effects** - Visual feedback during attack
5. **Vehicle attack animations** - Turret recoil/animation during fire
6. **Cannon fire state** - Visual feedback during cannon attacks

## Essential Features (User Requirements)

1. **Robot walk animation in 8 directions** - 4 frames at 300ms intervals
2. **Turret rotation independent of hull** - Already working, verify not broken
3. **Attack animations with muzzle flash** - Fire frames during combat
4. **Death animations by damage type**:
   - die1-4: Standard deaths (random selection, 10/10/10/8 frames)
   - melt: Pyro/laser fire death (17 frames)
   - die5/robot_flip: Missile death with arc trajectory (33 frames)
5. **Idle animations** - Already working, verify not broken

## Boundaries

**In scope:**
- Animation state machine refinements
- Death animation system with damage type selection
- Attack animation integration with CombatSystem
- Muzzle flash/recoil effects
- Effect spawning for deaths

**Out of scope (other phases):**
- Sound effects (Phase 1 - complete)
- Visual effects rendering (effects system exists)
- Grenade throwing mechanics (Phase 4)
- Driver ejection animation (Phase 4)

## Technical Decisions

1. **Death type selection** - Based on `MissileType` from attacker:
   - FLAME, LASER -> melt death
   - ROCKET, CANNON_SHELL, GRENADE -> missile flip (die5)
   - BULLET -> random die1-4

2. **Animation events** - Use Phaser scene events for loose coupling:
   - `unitDeath` -> spawns death effect, triggers animation
   - `unitAttack` -> triggers fire animation

3. **Frame timing** - All values from `AnimationConstants.ts`, no hardcoding

## Risk Areas

- **Animation state conflicts** - Walking while attacking, death while moving
- **Frame key mismatches** - getSpriteKey() must match atlas frame names exactly
- **Timing drift** - Delta time accumulation vs fixed frame intervals

## Reference Files

- `source/erobotdeath.cpp` - Death animation selection (die1-4, melt)
- `source/erobotturrent.cpp` - Missile death flip animation (die5, 33 frames)
- `source/rgrunt.cpp` DoRender() - Fire animation rendering logic
- `source/zrobot.cpp` - Animation state transitions

## Success Criteria

1. Robots animate walking in 8 directions with correct frame timing (300ms intervals)
2. Vehicles animate with turret rotation independent of hull
3. Attack animations play during combat (muzzle flash, recoil)
4. Death animations play on unit destruction (4 robot variants based on damage type)
5. Idle animations continue working (already implemented, verify not broken)
