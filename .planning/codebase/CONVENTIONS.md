# Coding Conventions

**Analysis Date:** 2026-01-24

## Naming Patterns

**Files:**
- PascalCase for classes and main exports: `GameObject.ts`, `CombatSystem.ts`, `SelectionManager.ts`
- camelCase for utility/helper functions: `pathfinding.ts`, `soundSystem.ts`
- Index barrel files: `index.ts` in each module directory for grouped exports
- Configuration files: `GameConfig.ts` for constants, `.ts` extension for all source files

**Functions:**
- camelCase for all methods: `getHealth()`, `setPosition()`, `updateHealthBar()`, `processActiveAttacks()`
- Public methods: explicit access modifiers with `public` keyword
- Private methods: prefixed with `private`, named descriptively: `drawPlaceholder()`, `getMissileType()`, `processMissileExplosion()`
- Callback methods: named with "on" prefix: `onMissileFired`, `onDamageDealt`, `onUnitDestroyed`, `onObjectDeath`
- Getter methods: use `get` prefix: `getHealth()`, `getTeam()`, `getMaxHealth()`, `getPosition()`

**Variables:**
- camelCase for local and instance variables: `refId`, `objectType`, `maxHealth`, `selectionIndicator`
- Constants: UPPER_SNAKE_CASE in config files: `TILE_WIDTH`, `GAME_HEIGHT`, `DEFAULT_SERVER_URL`, `ATTACK_ALERT_COOLDOWN`
- Private instance variables: single leading underscore: `_isSelected`, `_isSelectable`, `_delta`
- Map/collection naming: descriptive plurals: `objects`, `missiles`, `zones`, `controlGroups`, `activeAttacks`

**Types:**
- PascalCase for interfaces: `GameObjectState`, `SelectionInfo`, `CombatStats`, `DamageMissile`, `ZoneInfo`
- Enums: PascalCase for enum name, UPPER_SNAKE_CASE for values: `enum RobotAnimation { STAND, WALK, FIRE }`
- Type unions: descriptive names matching intent: `GameObjectState` (union type for all object states)
- Generic parameters: single letter or descriptive: `T`, `K`, `V` for maps/generics

## Code Style

**Formatting:**
- No ESLint or Prettier configured - style follows TypeScript conventions
- Consistent indentation: 2 spaces (Vite default)
- Line length: ~100-120 characters typical, no hard limit enforced
- Semicolons: Required at end of statements
- Quotes: Double quotes for strings: `"value"`, backticks for template literals

**Linting:**
- TypeScript strict mode enabled in `tsconfig.json`
- All strict checks active:
  - `strict: true` - enables all strict type checking
  - `noImplicitAny: true` - no untyped implicit any
  - `strictNullChecks: true` - null/undefined type safety
  - `noUnusedLocals: true` - removes dead variables
  - `noUnusedParameters: true` - removes unused params
  - `noImplicitReturns: true` - all code paths return
  - `exactOptionalPropertyTypes: true` - strict optional property handling

**Comments:**
- JSDoc comments on public methods: `/** Comment */` format
- Block comments for major sections: `// =======...=======`
- Inline comments for non-obvious logic
- No auto-comment requirement, but used extensively for complex systems
- Section headers divide methods: `// VISUALS`, `// SELECTION`, `// HEALTH BAR`

**Spacing:**
- Blank line between method sections
- Double newline separates major sections
- No trailing whitespace
- Proper spacing around operators: `const health = maxHealth - damage`

## Import Organization

**Order:**
1. External libraries: `import Phaser from "phaser"`
2. Types from constants: `import { DEPTH, TEAM_COLORS } from "@/config/GameConfig"`
3. Enums/types: `import { ObjectType, TeamType } from "@/types"`
4. Type imports: `import type { GameObjectState, Point } from "@/types"`
5. Classes/modules: `import { GameObject } from "../GameObject"`
6. Local exports: `import { Robot, Vehicle } from "@/objects"`
7. Default exports last: `import CombatSystem from "@/combat"`

**Path Aliases:**
- `@/` points to `src/` directory (configured in `tsconfig.json`)
- Always use alias imports: `import { GameObject } from "@/objects"`
- Never use relative paths: avoid `../../../objects/GameObject`
- Import from barrel files where available: `import { Minimap, SelectionBox } from "@/ui"`

**Barrel Files:**
- Each module has `index.ts` exporting public API
- `src/ui/index.ts` exports: `SelectionBox`, `ProductionWindow`, `Minimap`, `WaypointVisualizer`, `CursorManager`
- `src/scenes/index.ts` exports scene classes
- Import from module directory, not individual files
- Type exports use `export type` syntax

## Error Handling

**Patterns:**
- Try-catch for map loading and initialization: `spawnFromMapData()` wraps spawning in try-catch
- Console logging for errors: `console.error("Failed to...", error)`
- Console warnings for non-fatal issues: `console.warn("No objects array...")`
- Validation checks before operations: null checks on objects before accessing properties
- Graceful fallbacks: catch block logs error and continues: `catch (err) => { console.error(...); this.createTestGrid(); }`
- Guard clauses for null checks: early return if lookups fail

**Examples from code:**
```typescript
// ObjectManager.spawnFromMapData() - wraps spawning in try-catch
for (const mapObj of mapData.objects) {
  try {
    const obj = this.spawnMapObject(mapObj);
  } catch (error) {
    console.error(`Failed to spawn object ${mapObj.refId}:`, error);
    failedCount++;
  }
}

// CombatSystem.processActiveAttacks() - guard clauses
if (!attacker || !target) {
  attacksToRemove.push(attack.attackerRefId);
  return;
}

// GameScene.loadTestMap() - fallback on error
this.loadTestMap().catch((err) => {
  console.error("Failed to load test map:", err);
  this.createTestGrid();
});
```

## Logging

**Framework:** `console` methods (no logging library)

**Patterns:**
- `console.log()` for general info: `"🎮 Zod Engine - Initializing..."`, `"Combat: Unit X started attacking Y"`
- `console.error()` for errors: `"Failed to spawn object"`, `"Failed to load test map"`
- `console.warn()` for warnings: `"No objects array in map data"`, `"Object X already exists"`
- No log levels implemented - just console methods
- Emoji used in initialization messages: 🎮 emoji appears in startup logs

**When to Log:**
- System initialization: GameScene creation, scene launch
- Data loading: map data spawning, object counts
- Combat events: unit attack start, damage dealt, unit destroyed
- Warnings on non-fatal issues
- Errors on failure paths

## Module Design

**Exports:**
- Public classes exported from module file
- Interfaces exported separately: `export interface GameObjectState`
- Enums exported: `export enum ObjectType`
- Constants exported: `export const TILE_WIDTH = 16`
- Private properties marked with `private` keyword
- Public methods have explicit `public` keyword (optional but used)

**Class Structure:**
- Constructor initializes all properties
- Property declarations at top with type annotations
- Public methods before private methods
- Getter/setter pairs grouped together
- Helper/utility methods at bottom

**Example from GameObject:**
```typescript
export class GameObject extends Phaser.GameObjects.Container {
  // Properties declared with types
  protected refId: number;
  protected objectType: ObjectType;
  protected team: TeamType;
  protected health: number;

  constructor(scene: Phaser.Scene, state: GameObjectState) {
    super(scene, state.x, state.y);
    // Initialize all properties
  }

  // Public methods
  public override update(_time: number, _delta: number): void { }
  public updateFromState(state: GameObjectState): void { }

  // Private helper methods
  protected createVisuals(): void { }
  protected drawPlaceholder(): void { }

  // Getters/setters
  public getHealth(): number { return this.health; }
  public setHealth(health: number): void { }
}
```

## Function Design

**Size:**
- Functions typically 20-100 lines depending on complexity
- Large systems broken into modules: `WaypointSystem.ts` (1658 lines), `CombatSystem.ts` (866 lines)
- Largest files: `WaypointSystem.ts`, `SpriteLoader.ts`, `GameScene.ts`
- Method separation: each responsibility in its own method

**Parameters:**
- Methods accept typed parameters: `public setOnUnitLookup(getUnitInfo: (refId: number) => UnitInfo | null)`
- Callback pattern used extensively for event handling
- Config/options passed as single object when multiple related params: `(options: { x, y, team })`
- No long parameter lists - use objects or split methods if needed

**Return Values:**
- Explicit return types on all public methods: `: void`, `: number`, `: boolean`
- Null returns for not-found cases: `return null` when object doesn't exist
- Union types for flexible returns: `{ refId: number; damage: number; newHealth: number; isDriverHit: boolean; missile?: DamageMissile }`
- Void for side-effect methods: `public setHealth(health: number): void`

**Decorators/Overrides:**
- `override` keyword used for Phaser lifecycle: `public override update()`, `public override getBounds()`
- Keyword required due to TypeScript strict settings

---

*Convention analysis: 2026-01-24*
