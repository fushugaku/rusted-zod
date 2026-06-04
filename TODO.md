# Z Game JS Port - Comprehensive TODO Plan

**Project Status:** ~50% complete
**Total LOC:** ~18,000 TypeScript across 70+ files
**Last Updated:** 2026-01-24

---

## Phase 1: CRITICAL - Core Gameplay Foundation

### 1.1 Asset & Sprite System
- [ ] Implement sprite atlas loading system
- [ ] Create texture key mapping for all unit types
- [ ] Replace placeholder graphics with actual sprites
- [ ] Add fallback system when sprites unavailable
- [ ] Implement team color tinting for sprites

### 1.2 Animation System
- [ ] Implement robot walk animations (8 directions)
- [ ] Implement robot attack/fire animations
- [ ] Implement robot death animations
- [ ] Implement vehicle movement animations
- [ ] Implement vehicle turret rotation
- [ ] Implement cannon firing animations
- [ ] Implement building state animations (damaged, destroyed)
- [ ] Add idle animations for all units

### 1.3 Projectile Visuals
- [x] Implement missile sprite rendering (MissileRenderer.ts)
- [x] Add missile trail effects
- [x] Implement bullet tracer effects
- [x] Add laser beam rendering
- [x] Implement flamethrower effects
- [x] Add grenade arc trajectory visualization

### 1.4 Sound System Basics
- [x] Initialize Phaser sound manager (SoundSystem.ts)
- [x] Implement weapon fire sounds (framework ready)
- [x] Add explosion sounds (framework ready)
- [x] Implement unit acknowledgment sounds (framework ready)
- [x] Add building production sounds (framework ready)
- [x] Implement zone capture sound (framework ready)
- [ ] Add ambient/background sounds
- [ ] Load actual sound asset files

### 1.5 Enter Vehicle Mechanic
- [x] Implement robot entering vehicle logic (WaypointSystem.ts)
- [ ] Add vehicle lid open/close animations
- [x] Implement driver assignment (callback system ready)
- [x] Handle passenger system for APC
- [x] Implement robot exiting vehicle (callback system ready)
- [x] Add driver ejection on vehicle destruction

---

## Phase 2: HIGH - Advanced Combat & Unit Mechanics

### 2.1 Sniper Mechanics
- [x] Implement sniper driver sniping (kill driver, keep vehicle)
- [x] Add sniper targeting priority logic
- [x] Implement driver-less vehicle behavior
- [x] Add visual indicator for driver-less vehicles

### 2.2 Grenade System
- [ ] Implement grenade pickup boxes
- [ ] Add grenade inventory to robots
- [ ] Implement grenade throwing arc
- [ ] Add grenade explosion with AOE damage
- [ ] Implement PICKUP_GRENADES waypoint mode

### 2.3 Combat Visual Feedback
- [x] Add hit flash effects on units
- [x] Implement damage smoke/fire states for vehicles
- [x] Add crater creation on explosions
- [ ] Implement track marks for vehicles
- [x] Add muzzle flash effects
- [x] Add bullet tracer effects

### 2.4 Unit Speed & Movement
- [x] Implement actual unit speeds (WaypointSystem.ts - ROBOT_SPEEDS, VEHICLE_SPEEDS)
- [ ] Add running/stamina system for robots
- [x] Implement terrain speed modifiers (roads faster, water slower)
- [x] Add formation movement for groups

### 2.5 Repair Systems
- [x] Implement CRANE_REPAIR waypoint logic
- [x] Implement UNIT_REPAIR waypoint logic
- [x] Add repair building functionality
- [ ] Implement repair visual effects (sparks/wrench animation)

### 2.6 Fort Mechanics
- [x] Implement ENTER_FORT waypoint logic
- [x] Add fort entry points
- [x] Implement robots garrisoning in forts
- [ ] Add fort firing mechanics with garrisoned units

---

## Phase 3: MEDIUM - AI & Multiplayer Infrastructure

### 3.1 Bot AI - Basic
- [x] Create AISystem.ts framework
- [x] Implement threat assessment algorithm
- [x] Add target selection logic (priority: threats > production > territory)
- [x] Implement attack order generation
- [x] Add retreat/defense behavior
- [x] Implement build order AI

### 3.2 Bot AI - Advanced
- [x] Add strategic modes (aggressive, defensive, balanced, expansion, all-out)
- [ ] Implement unit grouping and squad tactics
- [x] Add territory control evaluation
- [x] Implement resource/production optimization
- [x] Add difficulty levels for AI (Easy, Normal, Hard)

### 3.3 Server Implementation
- [ ] Create Node.js server entry point
- [ ] Implement Socket.IO server setup
- [ ] Create game room/lobby system
- [ ] Add player connection management
- [ ] Implement game state authority (server-authoritative)

### 3.4 Network Protocol
- [ ] Define command packet format
- [ ] Implement client-to-server command sending
- [ ] Add server game state broadcasting
- [ ] Implement tick-based synchronization
- [ ] Add network lag compensation
- [ ] Implement reconnection handling

### 3.5 Multiplayer Game Flow
- [ ] Implement player ready system
- [ ] Add game start synchronization
- [ ] Implement pause/resume for all players
- [ ] Add player disconnect handling
- [x] Implement victory/defeat detection (GameStateSystem.ts)

---

## Phase 4: POLISH - UI/UX & Quality of Life

### 4.1 Minimap
- [x] Implement minimap rendering (Minimap.ts)
- [x] Add unit dots on minimap
- [x] Show zone ownership colors
- [x] Implement minimap click-to-move camera
- [ ] Add fog of war on minimap

### 4.2 HUD Improvements
- [x] Implement dynamic cursor types (attack, grab, repair, enter) - CursorManager.ts
- [ ] Add unit portrait system
- [x] Implement waypoint path visualization (WaypointVisualizer.ts)
- [ ] Add unit info panel with detailed stats
- [ ] Implement building info/status display
- [ ] Add resource/territory control bar

### 4.3 Production UI
- [ ] Add rally point setting UI
- [ ] Implement cannon placement UI
- [ ] Show building level in UI
- [ ] Add production progress bar
- [ ] Implement queue management (reorder, cancel)

### 4.4 Messages & Announcements
- [ ] Implement news/message system
- [ ] Add computer voice announcements
- [x] Show zone capture notifications
- [x] Add unit under attack alerts
- [ ] Implement chat system for multiplayer

### 4.5 Game Flow Screens
- [ ] Implement victory screen with stats
- [ ] Add defeat screen
- [ ] Create map selection screen
- [ ] Implement team selection UI
- [ ] Add game options/settings screen

---

## Phase 5: OPTIONAL - Advanced Features & Polish

### 5.1 Visual Effects
- [ ] Add water tile animations
- [ ] Implement building smoke/exhaust effects
- [ ] Add environmental particles (dust, debris)
- [ ] Implement weather effects
- [ ] Add day/night cycle (if supported)

### 5.2 Map & Terrain
- [ ] Implement destructible objects
- [ ] Add hut/animal decorations
- [ ] Implement bridge crossing mechanics
- [ ] Add pathfinding optimization (region-based)
- [ ] Implement threaded pathfinding for performance

### 5.3 Audio Polish
- [ ] Add background music system
- [ ] Implement voice acting integration
- [ ] Add environmental sounds
- [ ] Implement 3D positional audio
- [ ] Add audio settings (volume, mute)

### 5.4 Statistics & Replay
- [ ] Implement game statistics tracking
- [ ] Add end-game stats screen
- [ ] Create replay recording system
- [ ] Implement replay playback
- [ ] Add replay export/sharing

### 5.5 Quality of Life
- [x] Implement control groups (Ctrl+1-9, recall with 1-9)
- [x] Add double-click to select all of type (on screen)
- [x] Implement attack-move command (A + click)
- [x] Add stop command (S key)
- [x] Add patrol command (P + click)
- [x] Implement guard command (G key)

---

## Known TODO Comments in Code

| File | Line | Description |
|------|------|-------------|
| GameScene.ts | 497 | Set initial waypoint to rally point |
| GameScene.ts | 564 | Play capture sound |
| GameScene.ts | 565 | Update minimap |
| GameScene.ts | 610 | Create missile visual effect |
| PreloaderScene.ts | 173 | Full sound loading |
| ObjectManager.ts | 405 | Trigger death effect and remove object |
| WaypointSystem.ts | 411 | Implement actual enter logic |
| WaypointSystem.ts | 533 | Get actual speed from unit stats |

---

## Implementation Status Summary

| Category | Defined | Working | % Complete |
|----------|---------|---------|------------|
| Robot Types | 6 | 6 | 100% |
| Vehicle Types | 7 | 7 | 100% |
| Cannon Types | 4 | 4 | 100% |
| Building Types | 8 | 8 | 100% |
| Waypoint Modes | 10 | 6 | 60% |
| Combat Stats | 17 | 17 | 100% |
| Effect Types | 44 | 20 | 45% |
| Sound System | ✓ | Framework | 50% |
| AI System | ✓ | Framework | 70% |
| Minimap | ✓ | Working | 80% |
| Game State | ✓ | Working | 90% |
| Missile Renderer | ✓ | Working | 80% |
| Combat Visuals | ✓ | Working | 80% |
| Waypoint Visualizer | ✓ | Working | 100% |
| Sniper Mechanics | ✓ | Working | 100% |
| Networking | - | 0 | 0% |
| Sprite Loading | - | 0 | 0% |
| Animations | 28+ | 0 | 0% |

---

## Priority Notes

**Minimum Viable Single-Player:**
- Complete Phase 1 (assets, animation, sound)
- Complete Phase 2.1-2.3 (combat feel)
- Complete Phase 3.1-3.2 (bot AI)

**Minimum Viable Multiplayer:**
- All of single-player above
- Complete Phase 3.3-3.5 (networking)
- Complete Phase 4.1-4.2 (essential UI)

**Production Ready:**
- All phases complete
- Performance optimization
- Cross-browser testing
- Mobile support (optional)
