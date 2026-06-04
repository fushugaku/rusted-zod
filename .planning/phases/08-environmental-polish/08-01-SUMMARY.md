---
phase: 08-environmental-polish
plan: 01
subsystem: animals
tags: [huts, animals, wandering, ahutanimal.cpp, ohut.cpp]

# Dependency graph
requires: []
provides:
  - HutAnimal class with wandering behavior
  - Hut class managing animal spawning lifecycle
  - Animal palette selection by terrain type
affects:
  - 08-02 (fog of war) - no dependency
  - 08-03 (vehicle tracks) - no dependency

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "State machine pattern for animal behavior"
    - "Parent-managed entity lifecycle (Hut owns animals)"

# File tracking
key-files:
  created:
    - client/src/objects/animals/HutAnimal.ts
    - client/src/objects/animals/index.ts
    - client/src/objects/items/Hut.ts
  modified:
    - client/src/objects/items/index.ts
    - client/src/objects/UnitFactory.ts
    - client/src/scenes/GameScene.ts

# Decisions made during execution
decisions:
  - key: "animal-state-name"
    choice: "animalState instead of state"
    reason: "Avoids conflict with Phaser Container.state property"
  - key: "direction-preference"
    choice: "80% prefer similar direction, 20% random"
    reason: "Matches C source IsPrefferedDirection() for natural movement"
  - key: "roam-distance"
    choice: "128 pixels from home hut"
    reason: "From zsettings hut_animal_roam_distance default"

# Metrics
metrics:
  duration: "5 min"
  completed: "2026-01-25"
---

# Phase 08 Plan 01: Animal Spawning from Huts Summary

HutAnimal and Hut classes ported from ahutanimal.cpp and ohut.cpp, enabling palette-appropriate animals to spawn from huts and wander within roam distance.

## What Was Built

### HutAnimal Class (529 lines)
Ported from `source/ahutanimal.cpp`:
- **State machine**: NOTHING (idle), WALKING, LOOKING states
- **Movement**: 15 pixels/sec, sub-pixel accumulation for smooth motion
- **Direction preference**: 80% continue similar direction for natural paths
- **Roam enforcement**: Stays within 128px of home hut
- **Palette selection**: 15 animal types across 5 terrain palettes
- **Placeholder graphics**: Colored circles with size variation by type

### Hut Class (286 lines)
Ported from `source/ohut.cpp`:
- **Animal management**: Creates/destroys HutAnimal instances
- **Periodic checks**: Every 1 second, spawns or sends home animals
- **Max variation**: Random max (1-5) changes every 10 seconds
- **Exit positioning**: Animals spawn at passable tile below hut
- **Palette-aware sprites**: Uses terrain-specific hut graphic when available

### Integration
- UnitFactory creates Hut objects for ItemType.HUT_ITEM
- GameScene update loop processes huts via ObjectManager
- Shift+H debug shortcut spawns test hut at camera center

## Key Code Patterns

### Animal State Machine
```typescript
switch (this.animalState) {
  case AnimalState.NOTHING:
    if (time >= this.nextStateTime) {
      if (Math.random() > 0.2) this.gotoRandomTile();
      else this.setStateLooking();
    }
    break;
  case AnimalState.WALKING:
    // Process movement and animation
    break;
  case AnimalState.LOOKING:
    // Process look animation, then return to NOTHING
    break;
}
```

### Direction Preference (from C source)
```typescript
private isPreferredDirection(curDir: number, newDir: number): boolean {
  if (curDir === -1 || newDir === -1) return false;
  const diff = Math.abs(curDir - newDir);
  // Not preferred if diff is 3, 4, or 5 (opposite-ish directions)
  if (diff >= 3 && diff <= 5) return false;
  return true;
}
```

### Hut Animal Management
```typescript
private manageAnimals(): void {
  const displacement = this.maxAnimals - this.animals.length;
  if (displacement > 0) {
    const spawnCount = Math.floor(Math.random() * (displacement + 1));
    this.createAnimals(spawnCount);
  } else if (displacement < 0) {
    const sendHomeCount = Math.floor(Math.random() * (-displacement + 1));
    this.sendAnimalsHome(sendHomeCount);
  }
}
```

## Commits

| Hash | Type | Description |
|------|------|-------------|
| de381bd | feat | Create HutAnimal class with wandering behavior |
| 968f0e4 | feat | Create Hut class to manage animal spawning |
| f13e1cf | feat | Integrate huts into game loop with test shortcut |

## Verification

- [x] TypeScript compiles without errors
- [x] Client builds successfully
- [x] HutAnimal has NOTHING, WALKING, LOOKING states
- [x] Animals choose palette-appropriate type
- [x] Animals enforce roam distance from home hut
- [x] Hut manages animal count (spawns/sends home)
- [x] Shift+H spawns test hut at camera center
- [x] Animals destroyed when reaching home after goHome()

## Deviations from Plan

None - plan executed exactly as written.

## Next Phase Readiness

Phase 08 Plan 02 (Fog of War) ready to proceed. No dependencies on animal system.

## Notes

- Placeholder graphics used (colored circles) - real sprites can be added later
- Pathfinding passability checks stubbed out (TODO comment in code)
- Animals are purely visual, no gameplay impact (cannot be killed/interacted with)
