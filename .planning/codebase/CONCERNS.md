# Codebase Concerns

**Analysis Date:** 2026-01-24

## Tech Debt

**Large/Complex Files Lacking Refactoring:**
- Issue: Multiple files exceed 800+ lines of code, particularly `WaypointSystem.ts` (1658 lines), `SpriteLoader.ts` (1623 lines), and `GameScene.ts` (1115 lines). These are difficult to test, understand, and maintain.
- Files: `client/src/waypoint/WaypointSystem.ts`, `client/src/assets/SpriteLoader.ts`, `client/src/scenes/GameScene.ts`
- Impact: Higher bug potential, difficult to implement features, harder to fix issues in isolation, poor code reusability
- Fix approach: Break into smaller, single-responsibility modules. Separate concerns like animation management, sprite key generation, and waypoint logic into dedicated classes.

**Type Safety Issues with 'any' Casting:**
- Issue: Multiple locations use `as any` or `as unknown` to bypass TypeScript type checking, particularly when interfacing with Phaser objects and inter-scene communication
- Files: `client/src/scenes/GameScene.ts:1044` (HUDScene cast as any), `client/src/scenes/JeepTestScene.ts:174-217` (multiple jeep property casts), `client/src/scenes/controllers/CommandProcessor.ts:144-148` (function parameters typed as any), `client/src/waypoint/WaypointSystem.ts:1116`, `client/src/waypoint/WaypointSystem.ts:1606`
- Impact: Loss of type safety, potential runtime errors when properties don't exist, makes refactoring dangerous
- Fix approach: Create proper TypeScript interfaces for Phaser extensions and inter-scene data. Use proper type definitions instead of `any` parameters. Create wrapper types for complex Phaser objects.

**Missing Cleanup on Async Operations:**
- Issue: `setTimeout` calls in `EffectsSystem.ts` are not tracked or canceled. If the scene is destroyed while timeouts are pending, callbacks will execute on a destroyed scene.
- Files: `client/src/effects/EffectsSystem.ts:495-537` (multiple setTimeout calls in death effects without tracking)
- Impact: Potential memory leaks, console errors when callbacks execute on destroyed scenes, delayed effects triggering incorrectly
- Fix approach: Store setTimeout IDs and clear them when scene shuts down. Use Phaser's built-in timing system (`this.time.addEvent`) instead.

**Unhandled Promise Rejections:**
- Issue: Async pathfinding and map loading use `.catch()` but don't properly propagate errors in some cases
- Files: `client/src/scenes/controllers/CommandProcessor.ts:184-230` (catch block silently fails pathfinding), `client/src/scenes/GameScene.ts:379-398` (map loading error handling)
- Impact: Silent failures that could leave units in bad state, user confusion about why commands don't work
- Fix approach: Implement proper error handling with user feedback. Log errors to console for debugging. Validate all async operations complete before allowing game interactions.

**Console.log Statements Left in Production Code:**
- Issue: 102 console.log statements throughout codebase used for debugging, not structured logging
- Files: Throughout `client/src/` (search for `console.log`)
- Impact: Performance overhead in production, console spam for end users, difficult to enable/disable logging
- Fix approach: Implement structured logging system with log levels (debug, info, warn, error). Create logger module and replace all console calls.

## Known Issues

**Null Dereference Risks in Null-Initialized Systems:**
- Issue: GameScene initializes many systems as null (`private objectManager: ObjectManager | null = null`), but throughout the codebase they're accessed without null checks (e.g., `this.objectManager.getObject()` without checking if null first)
- Files: `client/src/scenes/GameScene.ts:39-57`, then used throughout in methods without guards
- Trigger: If a method is called before `setupSystems()` completes or during scene shutdown
- Workaround: Ensure setupSystems() completes synchronously before any interaction. But this is not guaranteed.

**Missing null/undefined Checks in Object Access:**
- Issue: Functions like `getObject()` can return undefined, but callers assume existence
- Files: `client/src/objects/ObjectManager.ts:327`, `client/src/scenes/controllers/CommandProcessor.ts:100`, callers don't all check for undefined
- Trigger: Querying for deleted objects, race conditions during cleanup
- Impact: Potential crashes when objects are destroyed mid-operation

**Undefined isInitialized Patterns:**
- Issue: Controllers and systems lack consistent initialization guards. Many check `if (!this.waypointSystem)` but others don't
- Files: Inconsistent checks in `client/src/scenes/controllers/CommandProcessor.ts` vs other systems
- Impact: Silent failures in some cases, exceptions in others, hard to debug

## Security Considerations

**No Input Validation on Map Loading:**
- Risk: Map loader reads binary data without validating structure. Malformed map files could cause memory issues or incorrect reads
- Files: `client/src/map/MapLoader.ts:91-97`, `client/src/map/TileInfoLoader.ts:33-39`
- Current mitigation: Basic error handling on fetch failure only
- Recommendations: Add bounds checking on array access, validate binary data structure before use, add size limits on loaded assets

**No Bounds Checking on Pathfinding:**
- Risk: Pathfinding allows movement to any coordinate without validating against map bounds
- Files: `client/src/map/Pathfinding.ts:98`, `client/src/scenes/controllers/CommandProcessor.ts:199-202`
- Current mitigation: Units can theoretically move off-map
- Recommendations: Add map boundary validation. Clamp coordinates to valid ranges. Validate destination points before pathfinding.

**Loose Type Checking in Combat System:**
- Risk: Combat targets are retrieved by ID but type is not verified before damage application
- Files: `client/src/combat/CombatSystem.ts`, `client/src/waypoint/WaypointSystem.ts:1116` (grenade amount retrieval without type check)
- Impact: Wrong object types could be damaged, stat calculations could fail
- Recommendations: Use type guards or discriminated unions instead of duck typing

## Performance Bottlenecks

**WaypointSystem Update Loop Complexity:**
- Problem: `WaypointSystem.update()` processes every unit every frame with complex pathfinding and collision checks (1658 lines of logic)
- Files: `client/src/waypoint/WaypointSystem.ts`
- Cause: Monolithic system doing too many things - movement, collision, engagement, repairs, all in one update loop
- Improvement path: Split into smaller systems. Use spatial hashing for collision detection. Defer non-critical checks to lower-frequency update cycles.

**Sprite Loading Creates Excessive Animations:**
- Problem: `SpriteLoader.ts` pre-creates animations for all rotations of all units at startup. With 6 robot types, 7 vehicle types, 8 rotations each = potentially 1000+ animations in memory
- Files: `client/src/assets/SpriteLoader.ts` (1623 lines of animation creation)
- Cause: No lazy-loading of animations. All created upfront during scene preload
- Improvement path: Create animations on-demand as units are spawned. Use sprite atlases more efficiently. Cache only active unit rotations.

**ObjectManager Linear Search on Spatial Queries:**
- Problem: `getObjectAt()` iterates all objects to find object at position - O(n) operation
- Files: `client/src/objects/ObjectManager.ts:327+`
- Cause: No spatial index structure
- Improvement path: Implement quadtree or grid-based spatial partitioning. Cache results frame-to-frame.

**Minimap Updates Every Frame:**
- Problem: Minimap redraws full terrain, zones, and units every update cycle
- Files: `client/src/ui/Minimap.ts:465+`
- Cause: Direct pixel manipulation in graphics layer without caching or dirty rectangles
- Improvement path: Only redraw dirty regions. Cache terrain layer. Update unit positions incrementally.

**SpriteLoader Sprite Key Lookup String Manipulation:**
- Problem: Sprite keys are generated by string concatenation in critical paths (every frame for rendering)
- Files: `client/src/assets/SpriteLoader.ts:95-150`
- Cause: No caching of sprite key lookups
- Improvement path: Pre-compute and cache sprite keys. Use enum-based lookups instead of string building.

## Fragile Areas

**WaypointSystem Unit Interaction Logic:**
- Files: `client/src/waypoint/WaypointSystem.ts` (especially repair, enter, attack logic)
- Why fragile: Massive state machine with many interlocking flags (isMoving, isRunning, isGuarding, isPatrolling, etc). Changing one behavior affects others unpredictably
- Safe modification: Add unit tests for each waypoint mode before changing. Document state transitions. Consider refactoring to explicit state pattern.
- Test coverage: No tests exist for this critical system

**GameObject Inheritance Hierarchy:**
- Files: `client/src/objects/GameObject.ts` extended by `Robot.ts`, `Vehicle.ts`, `Cannon.ts`, `Building.ts`
- Why fragile: Inheritance used for polymorphism, but different object types have different capabilities. Base class tries to handle all cases with checks for `getObjectType()`. Changes to base class affect all subclasses.
- Safe modification: Add comprehensive type guards before modifying base behavior. Test all subclasses when changing base class.
- Test coverage: No unit tests

**CommandProcessor Command Routing:**
- Files: `client/src/scenes/controllers/CommandProcessor.ts:143-179`
- Why fragile: Complex nested conditionals determine which command type to issue based on target type and selected unit type. New unit/building types require modifying routing logic.
- Safe modification: Add new types to routing only after implementation is complete. Test each routing path independently.
- Test coverage: None

**GameScene Initialization Order:**
- Files: `client/src/scenes/GameScene.ts:70-99`
- Why fragile: Multiple systems must initialize in specific order (objectManager before pathfinding, etc), but order is implicit in code sequence
- Safe modification: Add initialization guards. Make order explicit with dependency injection or ordered initialization list.
- Test coverage: No scene tests

**Phaser Inter-Scene Communication:**
- Files: `client/src/scenes/GameScene.ts:1044-1050` (casting HUDScene as any)
- Why fragile: GameScene and HUDScene communicate via scene registry and optional chaining on untyped objects. Adding/removing HUDScene methods breaks without TypeScript error.
- Safe modification: Create proper scene data interfaces. Use scene events instead of direct method calls.
- Test coverage: No integration tests between scenes

## Test Coverage Gaps

**No Unit Tests for Core Systems:**
- What's not tested: `WaypointSystem` movement logic, `CombatSystem` damage calculations, `ProductionSystem` queue logic, `AISystem` decision making
- Files: All major system files under `client/src/`
- Risk: Bugs go undetected until gameplay testing. Regressions introduced during refactoring.
- Priority: High - these are the most complex and error-prone systems

**No Integration Tests:**
- What's not tested: Multi-system interactions (combat + waypoints, production + selection, etc)
- Files: No test files exist for scene interactions
- Risk: Systems work in isolation but fail when integrated. Race conditions between systems.
- Priority: High

**No E2E Tests:**
- What's not tested: Full game flows (build unit → move → attack → die)
- Files: None
- Risk: User-facing bugs go to production
- Priority: Medium

**No UI Tests:**
- What's not tested: Selection system, minimap accuracy, production window logic
- Files: `client/src/scenes/controllers/SelectionManager.ts`, `client/src/ui/`, `client/src/scenes/HUDScene.ts`
- Risk: UI bugs affect player experience
- Priority: Medium

## Scaling Limits

**Object Count Limit:**
- Current capacity: Tested with pre-spawned units from map load
- Limit: Unknown. No profiling done. Linear searches and unoptimized physics checks suggest <1000 units would start to impact performance
- Scaling path: Implement spatial partitioning (quadtree), lazy physics checks, object pooling

**Memory Usage:**
- Current capacity: Unknown. SpriteLoader pre-creates all animations. No asset caching strategy.
- Limit: Large maps with many unit types could exceed typical browser memory (~100-500MB)
- Scaling path: Implement sprite streaming. Load animations on-demand. Use compressed asset formats.

**Map Size Limit:**
- Current capacity: Code supports arbitrary map sizes loaded from binary format
- Limit: Tile rendering is unoptimized (full redraw each frame in some systems)
- Scaling path: Implement tile culling. Use chunked rendering. Cache visible tile region.

## Missing Critical Features

**No Dedicated Logging System:**
- Problem: 102 console.log statements scattered throughout. No way to enable/disable logging per component or log level
- Blocks: Debugging production issues, performance profiling
- Alternative: Implement logger factory with levels and targets

**No Error Boundary / Crash Recovery:**
- Problem: Single exception in game loop crashes entire game. No recovery mechanism.
- Blocks: Graceful degradation, error reporting to users
- Alternative: Wrap scene updates in try-catch, show error UI, allow reload

**No Asset Management / Memory Cleanup:**
- Problem: Assets loaded once, never unloaded. No pooling of frequently created objects (projectiles, effects)
- Blocks: Long play sessions, multiple map loads, efficient resource use
- Alternative: Implement asset manager with unload tracking. Object pools for effects/projectiles.

**No Input Validation / Bounds Checking:**
- Problem: Movement commands, map coordinates, object IDs accepted without validation
- Blocks: Robust error handling, preventing invalid state
- Alternative: Validate all user input and external data at boundaries

**No Configuration System:**
- Problem: Speeds, distances, timers are hard-coded constants throughout codebase
- Blocks: Tuning gameplay without code changes, supporting multiple difficulty levels
- Alternative: Create configuration file or system for all gameplay constants

---

*Concerns audit: 2026-01-24*
