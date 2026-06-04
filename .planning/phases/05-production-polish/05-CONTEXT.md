# Phase 5: Production System Polish - Context

## Phase Goal

Production system shows accurate modifiers and improved UI with rally points and queue management.

## Requirements (from ROADMAP.md)

1. **Production modifiers (zone ownership bonus, damage penalty)**
   - Zone ownership reduces build time (up to 50% at 100% ownership)
   - Building damage increases build time (up to 125% at 0% health)
   - Formula from `source/zbuilding.cpp BuildTimeModified()`

2. **Production UI improvements (rally points, progress bars, queue management)**
   - Rally point visualization and setting
   - Progress bars with time remaining
   - Queue reordering and cancellation

3. **Crane construction visual effect**
   - Construction workers, cones, sign from `source/ecraneconco.cpp`
   - Animated workers (jackhammer, paper-pointing)
   - Travel animation to/from building site

## Current State Analysis

### Production System (`client/src/production/ProductionSystem.ts`)

**Already implemented:**
- Building registration and production state tracking
- Build queue (max 5 items) with add/cancel functionality
- Zone ownership modifier calculation (lines 553-561):
  ```typescript
  // Zone ownership reduces build time (up to 50% at 100% ownership)
  let modifiedTime = baseBuildTime - (baseBuildTime * 0.5 * zoneOwnership);
  // Building damage increases build time (up to 125% at 0% health)
  const healthPercent = health / maxHealth;
  modifiedTime += modifiedTime * (1.25 * (1.0 - healthPercent));
  ```
- Rally point storage and setting (line 680-686)
- Progress percentage calculation (line 600-611)
- Time remaining calculation (line 617-630)

**Not yet implemented:**
- Modifier display in UI (no visual feedback of zone/damage effects)
- Rally point visualization (line drawn on map)
- Dynamic build time updates when health changes during production

### Zone System (`client/src/zone/ZoneSystem.ts`)

**Already implemented:**
- Zone capture and ownership tracking
- Team ownership percentage calculation
- Production time modifier calculation (line 353-358)

**Not yet implemented:**
- Connection between zone ownership changes and production UI updates

### Production Window (`client/src/ui/ProductionWindow.ts`)

**Already implemented:**
- Basic window with unit selection buttons
- Progress bar display
- Queue display with cancel buttons
- Building health percentage display

**Missing:**
- Modifier display (zone bonus, damage penalty)
- Rally point button/UI
- Build time estimate with modifiers
- Visual indication of current modifiers

### Effects System (`client/src/effects/EffectsSystem.ts`)

**Already implemented:**
- Explosion, debris, smoke effects
- Damage smoke effects
- Muzzle flash effects
- Effect lifecycle management

**Not yet implemented:**
- Crane construction effect (ECraneConco equivalent)

## C Source Reference

### Build Time Modifiers (zbuilding.cpp:661-667)
```cpp
double ZBuilding::BuildTimeModified(double base_build_time)
{
    base_build_time -= base_build_time * 0.5 * zone_ownage;
    base_build_time += base_build_time * (1.25 * (1.0 - (1.0 * health / max_health)));
    return base_build_time;
}
```
Already ported correctly.

### Crane Construction Effect (ecraneconco.cpp)

Complex multi-element effect:
- `conco` - Construction barricade (8 frames)
- `cone0`, `cone1` - Traffic cones
- `sign` - Warning sign (8 flip frames)
- `robot_jackhammer` - Worker with jackhammer (2 frames)
- `robot_paper` - Worker with clipboard (2 frames)
- `robot_point` - Worker pointing (3 frames)
- `robot_travel` - Workers traveling (left/right/updown)

Animation timing:
- Travel time: 0.8 seconds
- Jackhammer animation: 0.045s per frame
- Paper/point animation: 0.15s per frame

### Production GUI (gwproduction.cpp)

Shows:
- Building name (Fort/Robot Factory/Vehicle Factory)
- State label (Select/Building/Paused/Place)
- Build time remaining formatted as `M:SS`
- Health percentage
- Progress bar
- Queue buttons
- OK/Cancel/Place buttons

## Dependencies

### From Previous Phases
- Phase 2: Texture atlas system for crane effect sprites
- Phase 3: Animation state machine patterns

### External Systems
- ZoneSystem: Provides ownership percentages
- ProductionSystem: Provides build state and modifiers
- EffectsSystem: Will host crane construction effect

## Plan Structure

### Plan 01: Production Modifier Display
- Add modifier info to ProductionSystem
- Update ProductionWindow to show zone bonus and damage penalty
- Dynamic time estimate updates

### Plan 02: Rally Points and Queue Management
- Rally point visualization (line/flag on map)
- Rally point setting via right-click on map
- Queue reordering via drag or arrow buttons

### Plan 03: Crane Construction Effect
- CraneConstructionEffect class in effects system
- Sprite loading for construction workers/props
- Integration with production completion events

## Success Criteria

1. Production speed shows zone ownership bonus (faster in owned zones)
2. Production speed shows damage penalty (slower when building damaged)
3. Rally points can be set and units move there after production
4. Build queue shows progress bars and allows reordering/cancellation
5. Crane building shows construction animation effect
