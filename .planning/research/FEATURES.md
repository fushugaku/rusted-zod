# Feature Landscape: Single-Player RTS (Z.O.D. Engine Port)

**Domain:** Classic RTS single-player experience (2000s era faithful port)
**Researched:** 2026-01-25
**Confidence:** HIGH (derived from original C++ source code analysis)

---

## Context

This research documents features for completing a **faithful single-player port** of Z.O.D. Engine. The goal is mechanical parity with the original, not modernization. Features are categorized based on what the original game had and what a polished single-player experience requires.

**Key Constraint:** This is a port, not a reimagining. Features not in the original source code are anti-features.

---

## Table Stakes

Features users expect from a playable single-player RTS. Missing = game feels broken or incomplete.

| Feature | Why Expected | Complexity | Current Status | Notes |
|---------|--------------|------------|----------------|-------|
| **Unit animations** | Units look static without movement/attack animations | High | NOT DONE | 8-direction walk, attack, death for all 17 unit types |
| **Sound effects** | Silent combat feels broken | Medium | FRAMEWORK ONLY | ~90 sound types in original (weapons, voice, computer) |
| **Victory/defeat screens** | No clear game ending | Low | NOT DONE | Original has end animations, stats display |
| **Game pause** | Can't step away from game | Low | NOT DONE | P key or menu pause |
| **Basic HUD** | Can't see unit health/selection | Medium | PARTIAL | Health bar exists, missing unit icons in HUD panel |
| **Minimap unit dots** | Can't track battlefield | Low | DONE | Already implemented |
| **Zone ownership colors** | Can't tell who owns what | Low | DONE | Already implemented |
| **Camera controls** | Can't navigate map | Low | DONE | WASD/arrow keys, minimap click |
| **Unit selection** | Can't issue commands | Low | DONE | Box select, click select, control groups |
| **Right-click commands** | Can't move/attack | Low | DONE | Move, attack-move, attack target |

### Animation Breakdown (Table Stakes Detail)

Animation is the largest missing table stakes feature. Original has:

**Robots (6 types x animations):**
- Walk: 8 directions, variable frames per type
- Attack/Fire: weapon-specific animation
- Death: 4 variants (standard, fire, explosion, melt), 8-17 frames each
- Idle: cigarette, beer, scan, stretch animations

**Vehicles (7 types x animations):**
- Movement: 8 directions
- Turret rotation: independent of body
- Lid open/close: 3 frames when robots enter/exit
- Damage states: smoke, fire, oil leak, sparks at different health thresholds

**Cannons (4 types x animations):**
- Turret rotation: 8 directions, 1s rotation cycle
- Fire: weapon-specific (gatling rapid toggle, howitzer/missile single shot)
- Placement: 7 frames when placed on map

**Buildings (8 types):**
- Production: factory activity animations
- Damage: smoke/fire at low health
- Destruction: collapse animation

### Sound Breakdown (Table Stakes Detail)

Original sound engine has ~90 sound types:

**Combat sounds (~20):**
- Weapon fire per type (psycho, rifle, laser, pyro, gatling, jeep, etc.)
- Explosions (5 variants + turret explosion)
- Ricochet, grenade throw

**Voice responses (~45):**
- Unit acknowledgments (yes sir, going in, affirmative, etc.)
- Unit reporting per type (grunts/psychos/snipers/toughs/lasers/pyros)
- Under attack warnings (escalating urgency)
- Target destroyed confirmations
- Victory/defeat reactions

**Computer voice (~15):**
- Manufacturing announcements (robot/vehicle/gun manufactured)
- Territory lost, radar activated, fort under attack
- "You're losing" messages (10 variants)

**Ambient (~5):**
- Factory sounds, radar ping, bat chirp, crow

---

## Polish Features

Features the original game has that complete the experience. Expected for a "finished" port.

| Feature | Value Proposition | Complexity | Current Status | Notes |
|---------|-------------------|------------|----------------|-------|
| **Unit portraits** | Personality, feedback on orders | High | NOT DONE | Animated faces with eye/mouth/hand movements |
| **Computer voice messages** | Status awareness | Medium | NOT DONE | Text overlay + voice for major events |
| **Unit info panel** | Detailed unit stats | Medium | NOT DONE | Shows grenade count, passenger count (APC), driver health |
| **Production progress bar** | Know when units ready | Low | NOT DONE | Visual build time indicator |
| **Production queue display** | Manage build order | Low | NOT DONE | Show queued units, allow reorder/cancel |
| **Rally point system** | Control where units go | Low | NOT DONE | Set/visualize rally points on buildings |
| **Fog of war minimap** | Strategic depth | Medium | NOT DONE | Unexplored areas hidden on minimap |
| **Track marks** | Movement feedback | Low | EVENTS ONLY | Vehicle tracks on terrain (fade over time) |
| **Death animation variants** | Combat variety | Medium | CONSTANTS ONLY | 4 robot death types based on damage source |
| **Repair visual effects** | Feedback on repair action | Low | NOT DONE | Sparks/wrench animation during repair |
| **Damage smoke/fire** | Vehicle health feedback | Medium | EVENTS ONLY | Smoke at 40-70%, fire/oil/sparks below 40% |
| **Crane construction effect** | Building repair feedback | Medium | NOT DONE | Crane enters building, jackhammer animation |
| **Map selection screen** | Choose battlefield | Medium | NOT DONE | List maps, show preview |
| **Team selection UI** | Choose faction | Low | NOT DONE | Color picker for team |
| **Game settings screen** | Customize experience | Medium | NOT DONE | Volume, speed (0.25x-4x), difficulty |
| **Dynamic music** | Atmosphere | Medium | NOT DONE | Calm/attacking/fort danger levels |
| **Grenade system** | Tactical depth | High | NOT DONE | Pickup boxes, inventory, throwing arc, AOE |

### Portrait System Detail (Polish Feature)

The original has a sophisticated portrait system (zportrait.cpp):

- 11 eye positions, 9 hand positions, 16 mouth shapes
- ~65 animation sequences (YES_SIR_ANIM through ENDL3_ANIM)
- Triggered by unit responses, under attack, victory/defeat
- Different graphics per robot type and team color

This is a HIGH complexity polish feature but is very distinctive to the original game.

### Production Modifiers (Polish Feature)

Original has nuanced production speed:
```
build_time = base_time
build_time -= base_time * 0.5 * zone_ownership_percent  // Up to 50% faster
build_time += base_time * 1.25 * (1 - health_percent)   // Up to 125% slower if damaged
```

Currently NOT IMPLEMENTED but affects game balance significantly.

---

## Nice-to-Have Features

Features in original that are low priority for initial release.

| Feature | Value | Complexity | Notes |
|---------|-------|------------|-------|
| **Animal spawning from huts** | Atmosphere | Medium | 3-5 animals roam near huts per planet type |
| **Water tile animations** | Visual polish | Low | Bobbing effect on water tiles |
| **Bridge destruction/repair** | Tactical element | High | Affects pathfinding |
| **Crater creation** | Combat feedback | Medium | Different sizes per weapon |
| **Chat system** | Multiplayer prep | Low | Not needed for single-player |
| **Voting system** | Multiplayer only | Medium | Skip entirely for single-player |

---

## Anti-Features

Features to explicitly NOT build. Either not in original (faithful port constraint) or would detract from the experience.

| Anti-Feature | Why Avoid | What Original Does Instead |
|--------------|-----------|---------------------------|
| **Fog of war on main view** | Not in original Z.O.D. | Only minimap has fog in some modes |
| **Resource harvesting** | Not in Z gameplay | Territory control replaces resources |
| **Base building** | Not in Z gameplay | Capture existing factories |
| **Tech trees** | Not in Z gameplay | Building level determines available units |
| **Unit upgrades** | Not in original | Units are fixed stats |
| **Hero units** | Not in original | All units are expendable |
| **Special abilities (cooldowns)** | Not in original | Grenades are the only "ability" |
| **Modern RTS QoL (grid hotkeys, smart cast)** | Faithful port | Use original control scheme |
| **Tutorials** | Not in original | Player learns through play |
| **Achievements** | Not in original | Just victory/defeat |
| **Replay system** | Not in original | Live game only |
| **Touch controls** | Web/desktop first | Mouse/keyboard only |
| **New units** | Faithful port | Only original 17 unit types |
| **New buildings** | Faithful port | Only original 8 building types |
| **Custom maps editor** | Out of scope | Use original map format |

---

## Feature Dependencies

```
Animation System
  |
  +-- Sprite Atlas Loading (prerequisite)
  |     |
  |     +-- Team Color Tinting
  |
  +-- Animation State Machine
        |
        +-- Walk Animations (robots, vehicles)
        +-- Attack Animations
        +-- Death Animations
        +-- Idle Animations (robots only)

Sound System
  |
  +-- Asset Loading (prerequisite)
  |
  +-- Playback Framework (DONE)
  |
  +-- Sound Categories
        |
        +-- Combat Sounds
        +-- Voice Responses
        +-- Computer Voice
        +-- Ambient

UI Polish
  |
  +-- Unit Portraits (independent)
  |
  +-- Production UI
  |     |
  |     +-- Progress Bar
  |     +-- Queue Display
  |     +-- Rally Points
  |
  +-- Game Flow Screens
        |
        +-- Victory/Defeat (requires: game state detection - DONE)
        +-- Map Selection
        +-- Team Selection
        +-- Settings
```

---

## MVP Recommendation (Playable Single-Player)

For a complete single-player experience, prioritize in this order:

### Phase 1: Core Playability (Must Have)
1. **Sprite atlas loading + team color tinting** - Everything looks like placeholders without this
2. **Unit walk animations** - Movement feels broken without animation
3. **Sound effect loading + playback** - Silent game feels empty
4. **Victory/defeat screens** - Game needs clear ending

### Phase 2: Combat Feel
5. **Unit attack animations** - Combat feels disconnected
6. **Death animations** - Units disappear abruptly
7. **Damage visual states** - Can't see unit health visually
8. **Computer voice for major events** - Status awareness

### Phase 3: Polish
9. **Unit portraits** - Distinctive Z.O.D. feature
10. **Production progress/queue UI** - Better factory management
11. **Rally point system** - Workflow improvement
12. **Game settings screen** - Volume/speed control

### Defer to Post-MVP
- Grenade system (complex, secondary combat mechanic)
- Animal spawning (purely atmospheric)
- Fog of war on minimap (not core to gameplay)
- Track marks (visual only)
- Dynamic music (audio polish)

---

## Complexity Estimates

| Effort Level | Feature Count | Examples |
|--------------|---------------|----------|
| **Low (1-2 days)** | 8 | Victory screen, pause, progress bar, track marks |
| **Medium (3-5 days)** | 10 | Sound loading, production UI, settings screen, damage effects |
| **High (1-2 weeks)** | 4 | Animation system, portrait system, grenade system |

**Total estimate for MVP (Phases 1-2):** ~4-6 weeks
**Total estimate for full polish (Phase 3):** ~2-3 additional weeks

---

## Sources

- **PRIMARY (HIGH confidence):** Original C++ source code in `./source/` (262 files analyzed)
  - `zhud.cpp`, `zhud.h` - HUD structure and buttons
  - `zportrait.cpp`, `zportrait.h` - Portrait animation system
  - `zsound_engine.cpp`, `zsound_engine.h` - Sound system
  - `zcomp_message_engine.cpp` - Computer voice messages
  - `zmusic_engine.cpp` - Dynamic music system
  - `zsettings.cpp` - Game settings and production modifiers
  - `gmm_*.cpp` - Menu screens (main menu, options, map select, team change)

- **SECONDARY (MEDIUM confidence):** Project documentation
  - `./docs/requirements/GAP-ANALYSIS.md` - Implementation gaps
  - `./documentation/mechanics.md` - Mechanics checklist
  - `./TODO.md` - Current implementation status

- **TERTIARY (LOW confidence):** Web research on classic RTS features
  - [Z: Steel Soldiers - Wikipedia](https://en.wikipedia.org/wiki/Z:_Steel_Soldiers)
  - [Z on Steam](https://store.steampowered.com/app/275530/Z/)
