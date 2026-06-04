# Codebase Structure

**Analysis Date:** 2026-01-24

## Directory Layout

```
client/
├── src/                          # Main source code
│   ├── main.ts                  # Application entry point
│   ├── config/                  # Game configuration constants
│   ├── types/                   # TypeScript interfaces and enums
│   ├── scenes/                  # Phaser scenes and scene controllers
│   │   └── controllers/         # Scene-level controllers
│   ├── objects/                 # Game entities (units, buildings, items)
│   │   ├── buildings/           # Building implementations
│   │   ├── items/               # Item implementations
│   │   └── units/               # Unit implementations (robots, vehicles, cannons)
│   ├── systems/                 # Core game systems
│   │   ├── ai/                  # Bot AI system
│   │   ├── combat/              # Combat calculation system
│   │   ├── effects/             # Visual effects system
│   │   ├── game/                # Game state system
│   │   ├── map/                 # Map rendering and pathfinding
│   │   ├── production/          # Building production system
│   │   ├── waypoint/            # Unit movement and orders
│   │   ├── zone/                # Zone territory system
│   │   └── sound/               # Audio system (stub)
│   ├── ui/                      # User interface overlays
│   ├── assets/                  # Asset loading and management
│   ├── animation/               # Animation constants (new)
│   └── game/                    # Game state utilities
├── dist/                         # Build output (Vite)
├── index.html                   # HTML entry point
├── tsconfig.json                # TypeScript configuration
├── vite.config.ts               # Vite build configuration
└── package.json                 # Dependencies and scripts
```

## Directory Purposes

**`client/src/`:**
- Purpose: All TypeScript source code
- Contains: Application logic, game systems, UI, assets
- Key files: main.ts (entry point), types/interfaces.ts (core data structures)

**`client/src/config/`:**
- Purpose: Game configuration constants
- Contains: Game dimensions, tile sizes, depth ordering, camera settings, colors, pathfinding costs
- Key files: `GameConfig.ts` (createGameConfig function, constants)

**`client/src/types/`:**
- Purpose: Type definitions and enums
- Contains: GameObjectState, Waypoint, MapBasics, TileInfo, PlayerInfo, all game enums
- Key files: `interfaces.ts` (40+ interfaces), `enums.ts` (ObjectType, TeamType, WaypointMode, etc.)

**`client/src/scenes/`:**
- Purpose: Phaser scene classes and scene-level controllers
- Contains: GameScene (main gameplay), HUDScene (overlay UI), BootScene, PreloaderScene, JeepTestScene
- Key files: `GameScene.ts` (1115 lines, core game loop)

**`client/src/scenes/controllers/`:**
- Purpose: Modular controllers for scene responsibilities
- Contains: CameraController, SelectionManager, CommandProcessor
- Pattern: Loosely coupled, injected into scene, manage input and state

**`client/src/objects/`:**
- Purpose: Game entity representation
- Contains: Base class GameObject, UnitFactory for spawning
- Key files: `GameObject.ts` (base class), `ObjectManager.ts` (entity registry and spawning)

**`client/src/objects/buildings/`:**
- Purpose: Building unit implementations
- Contains: Building (base), RobotFactory, VehicleFactory, Radar, Repair, Fort
- Inherits from: GameObject

**`client/src/objects/units/`:**
- Purpose: Mobile unit implementations
- Contains: Robot, Vehicle (base), Cannon (deployable unit)
- Subfolders: `robots/` (Grunt, Psycho, Sniper, Tough, Pyro, Laser), `vehicles/` (Jeep, Light, Medium, Heavy, APC, MissileLauncher, Crane), `cannons/` (Gatling, Howitzer, MissileCannon)
- Inherits from: GameObject

**`client/src/objects/items/`:**
- Purpose: Non-unit entities
- Contains: Rock (destructible scenery), Flag (zone markers), Crate (item drops)
- Inherits from: GameObject

**`client/src/ai/`:**
- Purpose: Computer player AI
- Contains: AISystem (single-file, 893 lines)
- Functionality: Bot strategy, threat assessment, unit grouping, production decisions

**`client/src/combat/`:**
- Purpose: Combat mechanics
- Contains: CombatSystem (single-file, 866 lines)
- Functionality: Hit chance, damage calculation, snipe mechanics, missile data structures

**`client/src/effects/`:**
- Purpose: Visual effects and rendering
- Contains: EffectsSystem, MissileRenderer
- Functionality: Explosions, impact animations, projectile visualization

**`client/src/game/`:**
- Purpose: Game state management
- Contains: GameStateSystem (single-file, 477 lines)
- Functionality: Victory/defeat conditions, game pausing, team statistics

**`client/src/map/`:**
- Purpose: Map rendering and pathfinding
- Contains: GameMap (tile rendering), MapLoader (map data loading), Pathfinding (A* pathfinding), TileInfoLoader
- Key files: `GameMap.ts` (448 lines), `Pathfinding.ts` (pathfinding algorithm)

**`client/src/production/`:**
- Purpose: Building production and queues
- Contains: ProductionSystem (single-file, 743 lines)
- Functionality: Unit build times, production queues, build list definitions

**`client/src/waypoint/`:**
- Purpose: Unit movement and order processing
- Contains: WaypointSystem (single-file, 1658 lines, largest system)
- Functionality: Path following, waypoint modes (move, attack, repair, guard), terrain modifiers, stamina

**`client/src/zone/`:**
- Purpose: Zone territory and capture mechanics
- Contains: ZoneSystem (single-file, ~400 lines)
- Functionality: Zone ownership, flag capture detection, zone-based production bonuses

**`client/src/sound/`:**
- Purpose: Audio system
- Contains: SoundSystem, SoundType enum
- Status: Stub implementation (no actual audio loaded yet)

**`client/src/ui/`:**
- Purpose: User interface overlays
- Contains: Minimap (territory visualization), ProductionWindow (build queue UI), SelectionBox (drag selection), WaypointVisualizer (pathfinding preview), CursorManager
- Key files: `Minimap.ts` (465 lines), `ProductionWindow.ts` (510 lines)

**`client/src/assets/`:**
- Purpose: Asset loading management
- Contains: SpriteLoader (single-file, 1623 lines, largest non-system file)
- Functionality: Load all sprites by team/type/rotation, register Phaser animations

**`client/src/animation/`:**
- Purpose: Animation constants and definitions (new directory)
- Contains: AnimationConstants, animation frame data
- Usage: Referenced by SpriteLoader and unit classes

**`client/dist/`:**
- Purpose: Build output from Vite
- Contains: Compiled JavaScript, assets, sourcemaps
- Generated by: `npm run build`

## Key File Locations

**Entry Points:**
- `client/src/main.ts`: Application startup, Phaser initialization
- `client/src/scenes/GameScene.ts`: Main game loop, system orchestration (1115 lines)
- `client/index.html`: HTML container, loads main.ts

**Configuration:**
- `client/src/config/GameConfig.ts`: All game constants (dimensions, colors, depth values)
- `client/tsconfig.json`: TypeScript compiler options (strict mode, path aliases)
- `client/vite.config.ts`: Vite build configuration, module resolution

**Core Logic:**
- `client/src/types/interfaces.ts`: 40+ interfaces defining game state structures
- `client/src/types/enums.ts`: ObjectType, TeamType, WaypointMode, BuildingType, etc.
- `client/src/objects/GameObject.ts`: Base class for all game entities

**Testing:**
- `client/src/scenes/JeepTestScene.ts`: Test scene for vehicle rendering
- Note: No formal unit test framework (Jest, Vitest) configured

## Naming Conventions

**Files:**
- Classes: PascalCase.ts (e.g., `GameScene.ts`, `CombatSystem.ts`, `Robot.ts`)
- Constants/utilities: camelCase or PascalCase.ts (e.g., `GameConfig.ts`, `index.ts` for barrels)
- Barrel exports: `index.ts` (aggregate module exports, found in every subdirectory)

**Directories:**
- Feature/system directories: kebab-case folders (`waypoint/`, `combat/`, `objects/`)
- Subdirectories organized by type: `buildings/`, `units/`, `robots/`, `vehicles/`, `cannons/`, `items/`

**Classes:**
- Base class: PascalCase (e.g., GameObject, Robot, Vehicle)
- System classes: PascalCase + "System" suffix (e.g., CombatSystem, WaypointSystem)
- Specific types: PascalCase (e.g., Grunt, Psycho, Jeep, Light Tank)

**Functions and Methods:**
- Public methods: camelCase (e.g., createVisuals(), updateFromState(), issueCommand())
- Private methods: camelCase with underscore prefix (e.g., _isSelectable, selectionIndicator)
- Constants: UPPER_SNAKE_CASE (e.g., WAYPOINT_REACH_DISTANCE, MAX_QUEUE_ITEMS)

**Variables:**
- Instance properties: camelCase with underscore prefix for private (e.g., _objects, _selectedIds)
- Type/interface parameters: PascalCase (e.g., GameObjectState, Waypoint)
- Enums: PascalCase for enum name, UPPER_SNAKE_CASE for values

## Where to Add New Code

**New Feature (e.g., new unit type):**
- Primary code: `client/src/objects/units/{type}/NewUnit.ts`
- Inherit from: Robot or Vehicle
- Register in: `client/src/objects/UnitFactory.ts` - createFromMapObject() method
- Sprites: Add to `client/src/assets/SpriteLoader.ts` - load spritesheet and animations
- Settings: Add combat stats to `client/src/combat/CombatSystem.ts` - ROBOT_COMBAT_STATS or VEHICLE_COMBAT_STATS
- Types: Add enum value to `client/src/types/enums.ts` - RobotType or VehicleType

**New Building Type:**
- Primary code: `client/src/objects/buildings/NewBuilding.ts`
- Inherit from: Building class
- Register in: `client/src/objects/UnitFactory.ts` - createFromMapObject() method
- Production: Add build list to `client/src/production/ProductionSystem.ts`
- Sprites: Add to `client/src/assets/SpriteLoader.ts`
- Types: Add enum value to `client/src/types/enums.ts` - BuildingType

**New Game System (e.g., diplomacy, espionage):**
- Create: `client/src/{system-name}/{SystemName}System.ts`
- Pattern: Singleton instantiated in GameScene.create()
- Dependencies: Inject via setter methods (setSystems, setObjectLookup, etc.)
- Updates: Add update() call to GameScene's update loop
- Events: Define event interfaces (e.g., DiplomacyEvent), setter callbacks

**New UI Component:**
- Create: `client/src/ui/ComponentName.ts`
- Inherit from: Phaser.GameObjects.Container or Graphics
- Location: Add to scene in GameScene.create() or launch as overlay scene
- Export: Add to `client/src/ui/index.ts` barrel file

**Utility Functions:**
- Shared helpers: Create module in existing system folder or new `client/src/utils/` folder
- Export via barrel: `client/src/{module}/index.ts` re-exports

## Special Directories

**`client/dist/`:**
- Purpose: Build output from Vite
- Generated: Yes (from npm run build)
- Committed: No (in .gitignore)
- Contains: Compiled JS, CSS, HTML, asset references

**`client/node_modules/`:**
- Purpose: Installed dependencies
- Generated: Yes (from npm install)
- Committed: No (in .gitignore)
- Contains: Phaser, Vite, TypeScript, Socket.io-client, EasyStar

**`client/src/animation/` (new):**
- Purpose: Animation frame data and constants
- Generated: No (manually created)
- Committed: Yes
- Contains: AnimationConstants.ts with frame count metadata

---

*Structure analysis: 2026-01-24*
