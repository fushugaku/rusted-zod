# Phase 8: Environmental Polish - Context

## Phase Goal

Complete environmental details that add life and polish to the game world:
1. Animals spawning from huts and wandering
2. Fog of war system on minimap
3. Vehicle track effects that fade over time

## Source Analysis

### Animal System (ohut.cpp, ahutanimal.cpp)

**From C source:**
- Animals are managed BY huts, not as independent entities
- Each hut maintains a vector of `AHutAnimal*` pointers
- Huts control animal spawn/despawn based on `max_hut_animals` setting
- Animals are palette-specific (15 animal types across 5 terrain palettes)

**Animal Types (ahutanimal.h):**
```cpp
enum hut_animal_type {
  GREEN_SNAKE, GREEN_LIZARD, DESERT_RABIT, RAPTOR,
  MINI_RAPTOR, PIG_DINO, YELLOW_WORM, ARCTIC_RABIT,
  PENGUIN, WHITE_WOLF, OSTRICH, RAT, TURTLE, RED_WORM,
  GREEN_EYED_FOX
};

// Palette mapping:
DESERT: GREEN_SNAKE, GREEN_LIZARD, DESERT_RABIT
VOLCANIC: RAPTOR, MINI_RAPTOR, PIG_DINO, YELLOW_WORM
ARCTIC: ARCTIC_RABIT, PENGUIN, WHITE_WOLF
JUNGLE: OSTRICH, RAT, TURTLE
CITY: RED_WORM, RAT, GREEN_EYED_FOX
```

**Animal Behavior (ahutanimal.cpp):**
- States: HA_NOTHING, HA_WALKING, HA_LOOKING
- Movement: `move_speed = 15.0` pixels/sec, random tile selection
- Roaming: Constrained by `hut_animal_roam_distance` from home hut
- Animation: Walk (4-8 frames), Look (0-4 frames), Dead (2 images)
- Prefers continuing in similar direction (80% of the time)
- Stays within passable tiles only

**Hut Management (ohut.cpp):**
- `next_hut_animal_time`: Check every 1.0 seconds
- `max_hut_animals`: Random between `hut_animal_min` and `hut_animal_max`
- Creates animals at passable exit tiles
- Sends animals home when over max

### Track Effects (etrack.cpp, etrack.h)

**From C source:**
- Two track types: `ET_TANK` and `ET_JEEP`
- Tracks are per-terrain palette (5 palettes, except CITY)
- Each track has 8 rotation directions
- Fade animation: 3 frames over ~4 seconds
- Timing: `ti = 0` (0-3.3s), `ti = 1` (3.3-3.6s), `ti = 2` (3.6-3.9s), killme at 3.9s

**Track Positioning (etrack.cpp SetTrackCoords):**
- Tracks are placed relative to vehicle center
- Two track marks per vehicle (left and right treads)
- Position offsets depend on direction (8 cases)
- Small random offset (0-1 pixels) added for variation

**Vehicle Integration (zvehicle.cpp):**
- Tracks drop periodically while moving
- `TryDropTracks()` called during movement
- Vehicles store `track_type` (tank or jeep style)

### Fog of War

**Analysis of zmini_map.cpp:**
- Original minimap does NOT implement fog of war
- Shows all units and zones regardless of visibility
- Terrain water tiles can be shown/hidden via `show_terrain` toggle

**For TypeScript implementation:**
- This is a NEW feature not in original C source
- Will track explored/visible tile states
- Minimap shows: unexplored (dark), explored (dim), visible (full)
- Vision provided by player units with sight radius

## Implementation Decisions

### 08-01: Animal System
- **Approach**: Hut class manages animals, not separate system
- **Rendering**: Use placeholder graphics (colored circles) initially, atlas sprites later
- **Performance**: Pool animals per hut, max 3-5 per hut based on settings
- **Simplification**: Focus on DESERT palette first for testing

### 08-02: Fog of War
- **Approach**: Create VisibilitySystem to track tile states
- **States**: UNEXPLORED (never seen), EXPLORED (seen before), VISIBLE (currently visible)
- **Minimap**: Add fog overlay layer with alpha masking
- **Updates**: Recalculate visibility each frame based on unit positions
- **Unit sight**: Use combat stats attack_range * 1.5 as sight radius

### 08-03: Vehicle Tracks
- **Approach**: Add TrackEffect class to EffectsSystem
- **Rendering**: Use procedural graphics (small lines) for tracks
- **Fade**: Alpha tween over 4 seconds matching C source timing
- **Integration**: Vehicles emit tracks while moving based on timer

## File Mapping

| Feature | New Files | Modified Files |
|---------|-----------|----------------|
| Animals | `client/src/objects/animals/HutAnimal.ts` | `client/src/objects/items/Hut.ts`, `client/src/types/enums.ts` |
| Fog of War | `client/src/visibility/VisibilitySystem.ts` | `client/src/ui/Minimap.ts`, `client/src/scenes/GameScene.ts` |
| Tracks | `client/src/effects/TrackEffect.ts` | `client/src/effects/EffectsSystem.ts`, `client/src/objects/units/Vehicle.ts` |

## Performance Considerations

**Animals:**
- Max 50 animals on map (10 huts * 5 each)
- Simple movement, no pathfinding needed
- Culled when off-screen

**Fog of War:**
- Visibility grid matches tile grid (not per-pixel)
- Only update visibility for moving units
- Cache explored state permanently
- Minimap fog uses texture (not per-frame draw)

**Tracks:**
- Max 100 active tracks
- Auto-remove after 4 seconds
- No physics, just visual overlay
- Batch render as single graphics call

## Dependencies

- No external libraries needed
- Uses existing EffectsSystem pattern
- Uses existing Minimap structure
- Uses existing ObjectManager for hut integration

## Asset Requirements

**Animals (can use placeholders):**
- 15 animal types * 8 directions * 4-8 walk frames
- Located: `assets/other/hut_animals/{type}_walk_r{rot}_n{frame}.png`
- Can simplify to colored circles for initial implementation

**Tracks (can use placeholders):**
- 2 types * 5 palettes * 8 directions * 3 fade frames
- Located: `assets/units/vehicles/track_effects/{type}_track_{palette}_r{rot}_n{frame}.png`
- Can use procedural lines for initial implementation

## Success Criteria

1. Huts spawn animals periodically (1-5 per hut)
2. Animals wander randomly within roam distance
3. Animals have walk/look animations (or placeholder visuals)
4. Minimap shows dark overlay for unexplored areas
5. Explored areas show last-known state (dimmed)
6. Visible areas show full brightness
7. Vehicles leave track marks when moving
8. Track marks fade over 4 seconds
9. All effects perform well (60 FPS maintained)
