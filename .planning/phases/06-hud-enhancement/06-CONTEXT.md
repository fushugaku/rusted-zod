# Phase 6: HUD Enhancement - Context

## Phase Goal
HUD displays unit portraits, detailed stats, and responsive info panels matching the original C engine implementation.

## Source References

### Portrait System (zportrait.h, zportrait.cpp)
- **Base dimensions:** 86x74 pixels per portrait
- **Portrait layers:** backdrop, head (3 directions), eyes (11 positions), mouth (16 positions), shoulders, hand (9 positions)
- **Animation count:** 65 portrait animations defined in `portrait_anim` enum
- **Frame timing:** `duration_multi = 0.015` seconds per frame unit
- **Per-frame state:** look_direction, mouth, eyes, hand position, hand visibility, head offset
- **Random idle animations:** 13 idle variants (blink, wink, surprise, anger, grin, scared, eyes_left/right/up/down, whistle, look_left/right)

### Portrait Graphics Structure (ZPortrait_Unit_Graphics)
Each robot type per team has:
- `shoulders` - 1 sprite
- `head[MAX_LOOK_DIRECTIONS]` - 3 sprites (LOOK_STRAIGHT, LOOK_RIGHT, LOOK_LEFT)
- `eyes[MAX_EYES]` - 11 sprites (indices 0-10)
- `mouth[MAX_MOUTHS]` - 16 sprites (indices 0-15)
- `hand[MAX_HANDS]` - 9 sprites (indices 0-8)

### Asset Path Pattern
```
assets/other/hud/portraits/{robot_type}_{team}/SHEADBI{face_id}_{frame:04d}.png
```
Where face_id maps:
- GRUNT: 2
- PSYCHO: 3
- SNIPER: 4
- TOUGH: 0
- PYRO: 1
- LASER: 1

Frame ordering (per face_id):
- 0: shoulders
- 1-3: head (straight, right, left)
- 4-19: mouth (16 frames)
- 20-30: eyes (11 frames)
- 31-39: hand (9 frames)

### HUD Info Display (zhud.cpp, zhud.h)
- **Unit icon position:** 550, 148 (offset from HUD origin)
- **Unit label position:** 550, 124
- **Vehicle/cannon label position:** 550, 230
- **Health bar position:** 548+14, 213 (74px wide, 8px tall)
- **Grenade display position:** 575, 185 (icon + count)

### Unit States for Portrait Behavior
- **Idle:** Random blink/wink/look animations (0.5-5.5 second intervals)
- **Selected:** Plays reporting animation with voice ("Unit reporting", "Grunts reporting", etc.)
- **Given order:** Plays acknowledgment animation ("Yes sir", "Moving in", "Let's do it", etc.)
- **Under attack:** Plays distress animations ("We're under attack", "Help help", "They're all over us")
- **Target destroyed:** Plays celebration ("Target destroyed", "Good hit", "Nice one")
- **In vehicle:** Different backdrop, uses driver's face

## Existing HUD Implementation (HUDScene.ts)

### Current Structure
- Bottom-right unit panel (200x150)
- Text-only display (name, health text, team, refId)
- Health bar graphic (100px wide)
- No portrait rendering
- No grenade count display
- Limited unit details

### Current Selection Handling
```typescript
// From HUDScene.ts
public updateSelectedUnit(state: GameObjectState | null): void
  - Updates unitNameText
  - Calls drawHealthBar(current, max)
  - Calls getUnitDetails(state) -> basic text info
```

### Event Integration
- Listens for `selectionChanged` event from GameScene
- Primary selection state drives display

## Success Criteria

1. **Portrait Animation System**
   - Load portrait graphics from existing assets (40 sprites per robot/team combo)
   - Composite layered rendering (backdrop, head, eyes, mouth, shoulders, hand)
   - Play 65 animation sequences with correct frame timing
   - Random idle animations every 0.5-5.5 seconds
   - State-based animation triggers (selection, order, combat)

2. **Unit Info Panel**
   - Health bar matching original (green/yellow/red segments)
   - Grenade count with icon (for robots)
   - Driver status for vehicles (driver health vs vehicle health)
   - Ammo/weapon info display

3. **Multi-Selection Display**
   - Show count by unit type when multiple selected
   - Show total/average health for group
   - Group portrait (first selected or dominant type)

4. **Building Info Panel**
   - Production queue visualization
   - Zone ownership indicator
   - Building level display

## Implementation Notes

### Portrait Rendering Approach
Use Phaser sprites with composite layering:
1. Backdrop (terrain-based or vehicle)
2. Head sprite (with x/y offset per frame)
3. Eyes sprite (overlaid on head, position depends on robot type)
4. Mouth sprite (overlaid on head, position depends on robot type)
5. Shoulders sprite (fixed at bottom)
6. Hand sprite (animated position per frame)

### Animation Data Structure
```typescript
interface PortraitFrame {
  duration: number;        // seconds
  lookDirection: LookDirection;  // STRAIGHT, LEFT, RIGHT
  headY: number;          // head offset
  mouth: number;          // mouth sprite index (0-15)
  eyes: number;           // eyes sprite index (0-10)
  handDoRender: boolean;  // show hand
  hand: number;           // hand sprite index (0-8)
  handX: number;          // hand position
  handY: number;
}

interface PortraitAnimation {
  frames: PortraitFrame[];
  totalDuration: number;
}
```

### State-Based Animation Selection
| Unit State | Animation Type | Voice |
|------------|----------------|-------|
| Selected (robot) | Type-specific reporting | "Grunts reporting", "Snipers reporting", etc. |
| Selected (other) | Generic reporting | "Unit reporting 1/2" |
| Move order | Acknowledgment | "Yes sir", "Moving in", "Here we go", etc. |
| Attack order | Attack ack | "Let's get em", "Going in", "Let's do it" |
| Under attack | Distress | "We're under attack", "Help help" |
| Target destroyed | Celebration | "Target destroyed", "Good hit", "Nice one" |

## Dependencies
- Phase 5 complete (production UI patterns established)
- Existing HUDScene.ts structure
- Portrait assets in assets/other/hud/portraits/
- Backdrop assets in assets/other/hud/

## Risks
- HIGH complexity on portrait system (noted in STATE.md as concern)
- 65 animation sequences to implement (consider extracting data from C source)
- Performance with composite sprite rendering (may need render texture caching)
