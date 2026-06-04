# Architecture

**Analysis Date:** 2026-01-24

## Pattern Overview

**Overall:** Modular game engine using Phaser 3 with a multi-system layered architecture

**Key Characteristics:**
- Scene-based game loop (Phaser Scene architecture)
- System-oriented design with specialized managers (Combat, Waypoint, Production, Zone, AI, Effects)
- Container-based rendering with Phaser Groups for object organization
- Network state synchronization from server
- TypeScript strict mode with path aliases for dependency injection

## Layers

**Presentation Layer (UI/Rendering):**
- Purpose: Render game visuals and handle user interface
- Location: `client/src/scenes/`, `client/src/ui/`, `client/src/effects/`, `client/src/animation/`
- Contains: Phaser Scenes (GameScene, HUDScene), UI overlays (Minimap, ProductionWindow, SelectionBox), visual effects
- Depends on: Objects layer, Systems layer
- Used by: Main entry point

**Scene Control Layer:**
- Purpose: Manage scene lifecycle, controllers, and game loop orchestration
- Location: `client/src/scenes/GameScene.ts`, `client/src/scenes/controllers/`
- Contains: GameScene (core loop), CameraController (viewport management), SelectionManager (unit selection), CommandProcessor (order handling)
- Depends on: All systems, Objects layer
- Used by: Phaser engine

**System Layer (Core Game Logic):**
- Purpose: Encapsulate domain logic for specific game mechanics
- Location: `client/src/combat/`, `client/src/waypoint/`, `client/src/production/`, `client/src/zone/`, `client/src/ai/`, `client/src/map/`
- Contains: Combat calculations, pathfinding, unit movement, building production, zone capture, bot behavior, map rendering
- Depends on: Types layer, Configuration layer
- Used by: GameScene controller

**Object Layer (Entity Management):**
- Purpose: Represent and manage game objects with state and behavior
- Location: `client/src/objects/`
- Contains: GameObject (base), Robot/Vehicle/Cannon/Building/Item subclasses, ObjectManager, UnitFactory
- Depends on: Types, Configuration, Systems (for state updates)
- Used by: GameScene, Systems layer

**State & Configuration Layer:**
- Purpose: Define game constants, types, and configuration
- Location: `client/src/types/`, `client/src/config/`
- Contains: Enums (ObjectType, TeamType, WaypointMode), Interfaces (GameObjectState, Waypoint), Constants (DEPTH, TILE_WIDTH, TEAM_COLORS)
- Depends on: None (foundation layer)
- Used by: All other layers

**Asset & Resource Layer:**
- Purpose: Load and manage sprites, animations, and media
- Location: `client/src/assets/`, `client/src/sound/`
- Contains: SpriteLoader, animation definitions, sound system
- Depends on: Phaser, Types
- Used by: Objects layer, GameScene

**Network/IO Layer:**
- Purpose: Handle server communication (stub for WebSocket integration)
- Location: `client/src/game/` (GameStateSystem)
- Contains: Message definitions, state sync logic
- Depends on: Types
- Used by: GameScene

## Data Flow

**Game Initialization:**
1. `client/src/main.ts` creates Phaser Game with scenes
2. BootScene → PreloaderScene → GameScene startup sequence
3. GameScene.create() initializes all systems and loads map data

**Per-Frame Update Loop (GameScene):**
1. Phaser calls scene.update(time, delta)
2. Update cycle:
   - Controllers update (CameraController, SelectionManager)
   - Systems update in order:
     - WaypointSystem: Process movement waypoints
     - CombatSystem: Check attack targets, generate missiles
     - ProductionSystem: Advance building queues
     - ZoneSystem: Check flag captures
     - AISystem: Bot decision making
     - EffectsSystem: Update particles, missiles
     - GameMap: Update animated tiles
   - ObjectManager: Update all objects
   - GameStateSystem: Check victory conditions

**Unit Movement (Waypoint Processing):**
1. CommandProcessor receives right-click command
2. Pathfinding computes path from unit to target
3. WaypointSystem processes waypoints each frame:
   - Check if waypoint reached (distance threshold)
   - Advance to next waypoint or complete
   - Apply terrain speed modifiers
   - Check for engaged enemies (agro system)

**Combat Execution:**
1. WaypointSystem detects unit within attack range
2. CombatSystem processes attack:
   - Roll hit chance
   - Calculate damage (% of target health)
   - Check snipe (driver elimination for vehicles)
   - Generate DamageMissile struct
3. MissileRenderer visualizes projectile path
4. EffectsSystem creates impact effects
5. Target GameObject receives damage, health updated

**Building Production:**
1. Building receives production queue from player
2. ProductionSystem tracks build time based on:
   - Unit type (base time)
   - Building level (faster at higher levels)
   - Building health (slower if damaged)
   - Zone ownership (bonus if owned)
3. When complete, UnitCreatedEvent triggers
4. ObjectManager spawns new unit with rally point waypoint

**Zone Capture:**
1. ZoneSystem.update() checks unit positions against zone bounds
2. Flag collision detected
3. Team ownership changes, ZoneCaptureEvent fired
4. ProductionSystem updates zone ownership percentages
5. GameStateSystem checks victory conditions

**State Update from Server (Stub):**
1. Network layer receives StateUpdateMessage with GameObjectState[]
2. ObjectManager.updateFromState() called for each object
3. GameObject.updateFromState() updates position, health, rotation, mode
4. Visual updates triggered (sprite animation, health bar)

**State Management:**
- Objects maintain BaseObjectState (position, health, mode, selection)
- State dispatched from server (not determined locally)
- Each frame: systems modify internal waypoint/target state, objects reflect in next server update
- Selection state maintained locally (SelectionManager)

## Key Abstractions

**GameObject Hierarchy:**
- Purpose: Represent all physical game entities
- Examples: `client/src/objects/GameObject.ts` (base), `client/src/objects/units/Robot.ts`, `client/src/objects/units/Vehicle.ts`, `client/src/objects/buildings/Building.ts`
- Pattern: Abstract container inheritance, subclasses override createVisuals(), updateVisuals()
- State: Extends Phaser.GameObjects.Container with game state (health, team, mode, selection)

**System Pattern:**
- Purpose: Encapsulate independent game mechanics
- Examples: CombatSystem, WaypointSystem, ProductionSystem, ZoneSystem, AISystem
- Pattern: Singleton-like systems registered in GameScene, public update() method called each frame
- Dependencies injected via setter methods (setSystems, setObjectLookup)

**Controller Pattern:**
- Purpose: Decouple scene from complex subsystems
- Examples: CameraController, SelectionManager, CommandProcessor
- Pattern: Constructed in GameScene, initialize() called, setter methods for dependencies
- Responsibility: Input handling, state aggregation, command dispatch

**Factory Pattern:**
- Purpose: Create game objects from data structures
- Examples: `client/src/objects/UnitFactory.ts` (creates units/vehicles/buildings), SpriteLoader (creates animations)
- Pattern: Stateless or minimal state, single entry point (createFromMapObject, loadSprites)

**Waypoint System Pattern:**
- Purpose: Path-based unit movement and commands
- Examples: `client/src/waypoint/WaypointSystem.ts`, Waypoint interface
- Pattern: Queue of waypoints, each processed sequentially, can be movement/attack/repair/enter modes
- Mode: WaypointMode enum (MOVE, ATTACK, ATTACK_MOVE, PATROL, GUARD, ENTER_VEHICLE, REPAIR, etc.)

**Event Pattern:**
- Purpose: Decouple systems from direct knowledge of each other
- Examples: DeathEvent, ZoneCaptureEvent, UnitCreatedEvent
- Pattern: Callback setters (setOnObjectDeath, setOnZoneCaptured), fired with event data

## Entry Points

**Main Application Entry:**
- Location: `client/src/main.ts`
- Triggers: DOMContentLoaded
- Responsibilities: Create Phaser game instance, initialize scenes, handle tab visibility

**Game Scene Entry:**
- Location: `client/src/scenes/GameScene.ts` - create() method
- Triggers: Scene creation by Phaser
- Responsibilities: Initialize all systems, load map data, setup input handlers, launch HUD overlay

**Per-Frame Update Entry:**
- Location: `client/src/scenes/GameScene.ts` - update(time, delta) method
- Triggers: Phaser game loop (60 FPS by default)
- Responsibilities: Call update on all systems in order, handle input state

**Input Entry Points:**
- Location: `client/src/scenes/GameScene.ts` - setupInput() method
- Handlers: Mouse (left-click selection, right-click commands, drag selection box), Keyboard (camera pan, A/P mode toggle, etc.)

**Server Message Entry (Stub):**
- Location: `client/src/scenes/GameScene.ts` - loadTestMap() (currently loads local test data)
- Intended: Will receive MapDataMessage, StateUpdateMessage, WaypointMessage from server
- Responsibilities: Update map, update object states, apply waypoint commands

## Error Handling

**Strategy:** Try-catch wrapping in critical paths, console logging, graceful fallbacks

**Patterns:**
- Map loading: Try/catch in loadTestMap(), fallback to createTestGrid()
- Object spawning: Try/catch in spawnMapObject(), continue on error with warning
- System initialization: Check for null, log warnings if systems unavailable
- Render failures: Console error but continue (missing sprite handled with placeholder)

**No explicit error throwing:** Most failures result in console warnings and game continuing with reduced functionality

## Cross-Cutting Concerns

**Logging:**
- Entry: console.log/warn/error throughout systems
- Locations: GameScene, ObjectManager, WaypointSystem, CombatSystem, ZoneSystem, AISystem
- Pattern: Descriptive messages with context (refId, team, coordinates)

**Validation:**
- Type safety: TypeScript strict mode enforces interface compliance
- State guards: Null checks before accessing systems (if (!this.combatSystem))
- Data validation: MapBasics array checks, WaypointReachDistance thresholds

**Authentication:**
- Not applicable (single-player/LAN game, no server auth in this client phase)
- PlayerInfo interface supports isLoggedIn flag (for future implementation)

**Performance Considerations:**
- Object pooling: Missiles rendered without creating objects (MissileRenderer)
- Container usage: Objects grouped in Phaser Container at DEPTH.UNITS
- Update limiting: AI thinks every 1s (AI_THINK_INTERVAL), zone check every 200ms
- Pathfinding cached: Computed once per command, reused until waypoint complete

---

*Architecture analysis: 2026-01-24*
