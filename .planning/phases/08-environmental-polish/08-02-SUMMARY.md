---
phase: 08-environmental-polish
plan: 02
subsystem: visibility
tags: [fog-of-war, minimap, visibility, strategic-depth]
requires:
  - 06-01 (Minimap already exists)
provides:
  - Fog of war system for strategic visibility
  - Tile visibility states (UNEXPLORED/EXPLORED/VISIBLE)
affects:
  - Future line-of-sight mechanics
  - Enemy unit visibility logic
tech-stack:
  added: []
  patterns:
    - Per-tile visibility grid
    - Radius-based reveal calculation
    - Graphics layer overlay
key-files:
  created:
    - client/src/visibility/VisibilitySystem.ts
    - client/src/visibility/index.ts
  modified:
    - client/src/ui/Minimap.ts
    - client/src/scenes/GameScene.ts
decisions:
  - fog-opacity: "UNEXPLORED=85% black, EXPLORED=45% black"
  - sight-range: "8 tiles default, 6 tiles for buildings"
  - update-throttle: "100ms visibility updates for performance"
metrics:
  duration: 4 min
  completed: 2026-01-25
---

# Phase 8 Plan 02: Fog of War on Minimap Summary

Fog of war system tracks explored/visible tiles and displays fog overlay on minimap for strategic depth.

## What Was Built

### VisibilitySystem (`client/src/visibility/VisibilitySystem.ts`)

Core visibility tracking system with three tile states:

```typescript
export enum TileVisibility {
  UNEXPLORED = 0,  // Never seen (black fog)
  EXPLORED = 1,    // Seen before but not visible now (dim)
  VISIBLE = 2,     // Currently visible (no fog)
}
```

Key features:
- **Radius-based reveal**: Units reveal tiles in circular radius (~8 tiles)
- **Persistent exploration**: Once explored, tiles stay explored (never return to unexplored)
- **Throttled updates**: 100ms update interval for performance
- **Unit type awareness**: Buildings have reduced sight range (6 tiles)

### Minimap Fog Layer (`client/src/ui/Minimap.ts`)

New `fogLayer` graphics layer added to minimap rendering stack:

```typescript
// Layer order (bottom to top):
background -> terrainLayer -> zoneLayer -> unitLayer -> fogLayer -> cameraViewLayer -> border
```

Fog rendering:
- **UNEXPLORED tiles**: 85% black opacity (dark fog)
- **EXPLORED tiles**: 45% black opacity (dim, shows last-known state)
- **VISIBLE tiles**: No fog overlay

### GameScene Integration

- VisibilitySystem initialized during map loading with map dimensions
- Player team set for visibility calculations
- Unit lookup callback provided for visibility sources
- Visibility updated every frame in update loop
- Wired to Minimap via `setVisibilitySystem()`

## Debug Features

**Shift+R**: Reveals entire map (sets all tiles to VISIBLE)

## Commits

| Hash | Description |
|------|-------------|
| dd2c290 | feat(08-02): add VisibilitySystem for fog of war tracking |
| 280390e | feat(08-02): add fog of war overlay layer to Minimap |
| d7c9ee8 | feat(08-02): integrate VisibilitySystem into GameScene |

## Deviations from Plan

None - plan executed exactly as written.

## Success Criteria Verification

- [x] VisibilitySystem tracks UNEXPLORED, EXPLORED, VISIBLE states
- [x] Fog overlay renders on minimap with correct alpha values
- [x] Player units reveal fog in circular radius (8 tiles)
- [x] Explored tiles stay revealed (don't return to unexplored)
- [x] Non-visible explored tiles show dimmed overlay
- [x] No performance impact (100ms throttled updates)
- [x] Debug reveal command works (Shift+R)

## Next Phase Readiness

Ready for Phase 8 Plan 03 (Vehicle Track Effects) - no blockers.

## Technical Notes

### Performance Optimization

The visibility system uses several optimizations:
1. **Throttled updates**: Only recalculates every 100ms
2. **Set-based visible tracking**: Uses `Set<number>` for O(1) tile lookup
3. **Grid-based storage**: Single array for entire map visibility state
4. **Incremental fog drawing**: Only draws non-visible tiles (skips visible)

### Future Enhancements

Potential improvements for future plans:
- Line-of-sight blocking (terrain/buildings block visibility)
- Variable sight range per unit type
- Enemy unit visibility rules (show in visible areas only)
- Fog of war on main game view (not just minimap)
