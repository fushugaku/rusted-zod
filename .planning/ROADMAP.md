# Roadmap: Z.O.D. Engine TypeScript Port

## Milestone: Single-Player Complete (v1.0)

Complete the single-player experience with proper animations, audio, UI, and all gameplay mechanics matching the original C engine. Transform the functional but bare-bones prototype into a polished game that faithfully recreates the Z.O.D. experience.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [x] **Phase 1: Audio System** - Sound effects and computer voice playback ✓
- [x] **Phase 2: Texture Atlas Migration** - Pack sprites for performance and team color support ✓
- [x] **Phase 3: Animation System** - Walk, attack, death animations for all units ✓
- [ ] **Phase 4: Combat Polish** - Grenades, driver health, fort firing, visual effects
- [ ] **Phase 5: Production System Polish** - Zone modifiers, UI improvements, crane effects
- [ ] **Phase 6: HUD Enhancement** - Unit portraits, info panels, messages system
- [ ] **Phase 7: Game Flow Screens** - Victory/defeat, map selection, team selection, settings
- [ ] **Phase 8: Environmental Polish** - Animal spawning, fog of war, track effects

## Phase Details

### Phase 1: Audio System
**Goal**: Players hear sound effects for all game actions - weapons, movement, UI, and computer voice announcements
**Depends on**: Nothing (first phase, low risk quick win)
**Requirements**:
- Sound asset loading and playback
- Messages/announcements system with computer voice
**Complexity**: Small
**Success Criteria** (what must be TRUE):
  1. Weapon sounds play when units fire (guns, lasers, missiles, flames)
  2. Unit voice responses play on selection and command acknowledgment
  3. Computer voice announces major events (zone captured, unit lost, building destroyed)
  4. UI sounds play for button clicks and menu interactions
  5. Sound volume can be adjusted without code changes
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md — Audio asset loading and sound configuration system
- [x] 01-02-PLAN.md — Combat and unit voice integration
- [x] 01-03-PLAN.md — Computer voice announcement system

### Phase 2: Texture Atlas Migration
**Goal**: Sprites load from packed atlases instead of 9000+ individual files, with team color support
**Depends on**: Phase 1 (can run in parallel, but audio proves asset pipeline)
**Requirements**:
- Sprite atlas loading with team color tinting
**Complexity**: Medium
**Success Criteria** (what must be TRUE):
  1. All unit sprites load from texture atlases (not individual files)
  2. Load time reduced significantly (target: under 5 seconds vs current 30+)
  3. Team colors (red, blue, green, etc.) render correctly on all units
  4. SpriteLoader API remains compatible with existing unit code
  5. WebGL batching enabled (fewer draw calls visible in dev tools)
**Plans**: 2 plans

Plans:
- [x] 02-01-PLAN.md — Atlas generation tooling with free-tex-packer
- [x] 02-02-PLAN.md — SpriteLoader atlas integration and unit class updates

### Phase 3: Animation System
**Goal**: All units display proper animations for movement, combat, and death - matching original C engine frame timing
**Depends on**: Phase 2 (atlases required for efficient animation frames)
**Requirements**:
- All unit animations (walk, attack, death for all types)
- Death animation variants (4 robot death types)
**Complexity**: Large
**Success Criteria** (what must be TRUE):
  1. Robots animate walking in 8 directions with correct frame timing (300ms intervals)
  2. Vehicles animate with turret rotation independent of hull
  3. Attack animations play during combat (muzzle flash, recoil)
  4. Death animations play on unit destruction (4 robot variants based on damage type)
  5. Idle animations continue working (already implemented, verify not broken)
**Plans**: 3 plans

Plans:
- [x] 03-01-PLAN.md — Animation state machine and robot walk/attack animations
- [x] 03-02-PLAN.md — Vehicle turret/track and cannon fire animations
- [x] 03-03-PLAN.md — Death animation system with damage type selection

### Phase 4: Combat Polish
**Goal**: Complete combat mechanics with grenades, driver health, fort firing, and visual feedback
**Depends on**: Phase 3 (animations needed for grenade throw, repair visuals)
**Requirements**:
- Grenade system (pickup boxes, inventory, throwing, AOE)
- Driver health system (separate from vehicle health)
- Fort firing mechanics with garrisoned units
- Repair visual effects (sparks/wrench)
**Complexity**: Large
**Success Criteria** (what must be TRUE):
  1. Robots can pick up grenade boxes and carry inventory (max 5)
  2. Grenades can be thrown with arc trajectory and AOE damage on impact
  3. Vehicles track driver health separately from vehicle health (snipe kills driver)
  4. Forts fire using weapons of garrisoned units inside
  5. Repair actions show sparks/wrench visual effects
**Plans**: 4 plans

Plans:
- [ ] 04-01-PLAN.md — Grenade pickup, inventory, and throwing mechanics
- [ ] 04-02-PLAN.md — Driver health system and snipe integration
- [ ] 04-03-PLAN.md — Fort firing with garrisoned units
- [ ] 04-04-PLAN.md — Repair visual effects

### Phase 5: Production System Polish
**Goal**: Production system shows accurate modifiers and improved UI with rally points and queue management
**Depends on**: Phase 4 (core gameplay solid before UI polish)
**Requirements**:
- Production modifiers (zone ownership bonus, damage penalty)
- Production UI improvements (rally points, progress bars, queue management)
- Crane construction visual effect
**Complexity**: Medium
**Success Criteria** (what must be TRUE):
  1. Production speed shows zone ownership bonus (faster in owned zones)
  2. Production speed shows damage penalty (slower when building damaged)
  3. Rally points can be set and units move there after production
  4. Build queue shows progress bars and allows reordering/cancellation
  5. Crane building shows construction animation effect
**Plans**: 3 plans

Plans:
- [ ] 05-01-PLAN.md — Production modifier display (zone bonus, damage penalty)
- [ ] 05-02-PLAN.md — Rally point visualization and setting
- [ ] 05-03-PLAN.md — Crane construction visual effect

### Phase 6: HUD Enhancement
**Goal**: HUD displays unit portraits, detailed stats, and responsive info panels
**Depends on**: Phase 5 (production UI sets patterns for other HUD work)
**Requirements**:
- Unit portrait system in HUD
- Unit info panel with detailed stats
**Complexity**: Large
**Success Criteria** (what must be TRUE):
  1. Selected unit shows animated portrait (65 animation sequences per unit type)
  2. Portrait eyes and mouth animate based on unit state (idle, combat, damaged)
  3. Unit info panel shows health, ammo, grenades, driver status
  4. Multi-selection shows group info (unit count by type, total health)
  5. Building info shows production queue and zone ownership
**Plans**: 3 plans

Plans:
- [ ] 06-01-PLAN.md — Unit portrait animation system
- [ ] 06-02-PLAN.md — Unit info panel with stats
- [ ] 06-03-PLAN.md — Multi-selection and building info panels

### Phase 7: Game Flow Screens
**Goal**: Complete game flow from launch to gameplay with map selection, team choice, settings, and end screens
**Depends on**: Phase 6 (HUD patterns inform menu UI)
**Requirements**:
- Victory/defeat screens with stats
- Map selection screen
- Team selection UI
- Game settings screen
**Complexity**: Medium
**Success Criteria** (what must be TRUE):
  1. Victory screen shows when all enemy units/buildings destroyed with game stats
  2. Defeat screen shows when player loses all units/buildings
  3. Map selection screen lists available maps with preview images
  4. Team selection allows choosing faction color before game start
  5. Settings screen allows adjusting audio volume, game speed, graphics options
**Plans**: 3 plans

Plans:
- [ ] 07-01-PLAN.md — Victory and defeat screens with stats
- [ ] 07-02-PLAN.md — Map selection and team selection UI
- [ ] 07-03-PLAN.md — Game settings screen

### Phase 8: Environmental Polish
**Goal**: Complete environmental details - animals, fog of war, vehicle tracks
**Depends on**: Phase 7 (core game complete, polish layer)
**Requirements**:
- Animal spawning from huts
- Fog of war on minimap
- Track effects rendering for vehicles
**Complexity**: Small
**Success Criteria** (what must be TRUE):
  1. Animals spawn from huts periodically and wander the map
  2. Minimap shows fog of war for unexplored areas
  3. Explored but not visible areas show last-known state on minimap
  4. Vehicles leave track marks on terrain as they move
  5. Track effects fade over time
**Plans**: 3 plans

Plans:
- [ ] 08-01-PLAN.md — Animal spawning from huts
- [ ] 08-02-PLAN.md — Fog of war on minimap
- [ ] 08-03-PLAN.md — Vehicle track effects

## Progress

**Execution Order:** Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Audio System | 3/3 | ✓ Complete | 2026-01-25 |
| 2. Texture Atlas Migration | 2/2 | ✓ Complete | 2026-01-25 |
| 3. Animation System | 3/3 | ✓ Complete | 2026-01-25 |
| 4. Combat Polish | 0/4 | Ready | - |
| 5. Production System Polish | 0/3 | Planned | - |
| 6. HUD Enhancement | 0/3 | Planned | - |
| 7. Game Flow Screens | 0/3 | Planned | - |
| 8. Environmental Polish | 0/3 | Ready | - |

## Requirement Coverage

All 21 active requirements mapped to exactly one phase:

| Requirement | Phase |
|-------------|-------|
| Sound asset loading and playback | Phase 1 |
| Messages/announcements system with computer voice | Phase 1 |
| Sprite atlas loading with team color tinting | Phase 2 |
| All unit animations (walk, attack, death for all types) | Phase 3 |
| Death animation variants (4 robot death types) | Phase 3 |
| Grenade system (pickup boxes, inventory, throwing, AOE) | Phase 4 |
| Driver health system (separate from vehicle health) | Phase 4 |
| Fort firing mechanics with garrisoned units | Phase 4 |
| Repair visual effects (sparks/wrench) | Phase 4 |
| Production modifiers (zone ownership bonus, damage penalty) | Phase 5 |
| Production UI improvements (rally points, progress bars, queue management) | Phase 5 |
| Crane construction visual effect | Phase 5 |
| Unit portrait system in HUD | Phase 6 |
| Unit info panel with detailed stats | Phase 6 |
| Victory/defeat screens with stats | Phase 7 |
| Map selection screen | Phase 7 |
| Team selection UI | Phase 7 |
| Game settings screen | Phase 7 |
| Animal spawning from huts | Phase 8 |
| Fog of war on minimap | Phase 8 |
| Track effects rendering for vehicles | Phase 8 |

**Coverage: 21/21 requirements mapped**
