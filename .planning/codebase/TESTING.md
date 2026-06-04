# Testing Patterns

**Analysis Date:** 2026-01-24

## Test Framework

**Status:** No testing framework configured

- No Jest, Vitest, or other test runner installed
- No test files present in codebase (no `.test.ts` or `.spec.ts` files found)
- `package.json` has no testing dependencies
- TypeScript only supports type checking via `tsc --noEmit`

**Run Commands:**
```bash
npm run typecheck  # Type checking only (tsc --noEmit)
npm run build      # Compile and build production bundle
npm run dev        # Start development server
```

**Current State:**
- Type safety enforced through strict TypeScript compiler settings
- No unit tests, integration tests, or E2E tests
- Code review and manual testing only

## Test File Organization

**Not applicable** - no tests currently exist

**Recommended Structure (if tests were added):**
```
client/src/
├── combat/
│   ├── CombatSystem.ts
│   └── CombatSystem.test.ts        # Co-located with implementation
├── objects/
│   ├── GameObject.ts
│   └── GameObject.test.ts
└── utils/
    ├── pathfinding.ts
    └── pathfinding.test.ts
```

**Proposed Naming:**
- Test files use `.test.ts` suffix (not `.spec.ts`)
- One test file per source file
- Co-located in same directory as implementation

## Type Safety (Current Testing Strategy)

**Primary Quality Control:**
- TypeScript strict mode enforces type safety
- All files must pass: `npm run typecheck` without errors

**Strict Compiler Settings:**
From `tsconfig.json`:
```json
{
  "strict": true,                          // All strict checks enabled
  "noImplicitAny": true,                   // No untyped any
  "strictNullChecks": true,                // Null/undefined safety
  "strictFunctionTypes": true,             // Function signature checking
  "strictPropertyInitialization": true,   // All properties initialized
  "noImplicitReturns": true,               // All code paths must return
  "noUnusedLocals": true,                  // No dead variables
  "noUnusedParameters": true,              // No unused params
  "exactOptionalPropertyTypes": true,      // Strict optional properties
  "noImplicitOverride": true,              // Override keyword required
  "noPropertyAccessFromIndexSignature": true  // No loose index access
}
```

**What This Catches:**
- Null pointer exceptions: `strictNullChecks` catches undefined access
- Type mismatches: passing wrong types to functions
- Dead code: unused variables and parameters
- Missing return statements: code paths with no return
- Initialization bugs: properties not initialized in constructor

## Mocking Strategy

**Current Approach:**
- Mock/stub implementations used for Phaser framework integration
- Scene objects created as standalone instances for testing without full Phaser

**Patterns Observed:**

**1. Callback-based dependency injection** (`CombatSystem.ts`):
```typescript
// External systems inject their own methods
private getUnitInfo?: (refId: number) => UnitInfo | null;
private applyDamageToUnit?: (refId: number, damage: number, attackerRefId: number) => void;

public setUnitLookup(
  getUnitInfo: (refId: number) => UnitInfo | null,
  applyDamage: (refId: number, damage: number, attackerRefId: number) => void
): void {
  this.getUnitInfo = getUnitInfo;
  this.applyDamageToUnit = applyDamage;
}
```

**2. Event callback handlers** (`CombatSystem.ts`):
```typescript
private onMissileFired?: (missile: DamageMissile) => void;
private onDamageDealt?: (targetRefId: number, damage: number) => void;
private onUnitDestroyed?: (targetRefId: number, attackerRefId: number) => void;

public setOnMissileFired(callback: (missile: DamageMissile) => void): void {
  this.onMissileFired = callback;
}

// Usage: this.onMissileFired?.(missile);
```

**3. Scene reference injection** (`ObjectManager.ts`):
```typescript
constructor(scene: Phaser.Scene) {
  this.scene = scene;
  this.objectContainer = scene.add.container(0, 0);
}
```

## What to Mock (If Tests Were Added)

**Should Mock:**
- Phaser scene and game objects (graphics, containers, sprites)
- Network socket connections (`socket.io-client`)
- Canvas/rendering operations
- Timer/animation frame callbacks
- External data sources (map files, asset loaders)

**Should NOT Mock:**
- Business logic (combat calculations, damage processing)
- State management (zone ownership, object tracking)
- Pathfinding and waypoint logic
- Game rules and physics

**Example Strategy:**
```typescript
// Mock Phaser Scene
const mockScene = {
  add: {
    graphics: () => ({ clear: jest.fn() }),
    container: () => ({ add: jest.fn(), setDepth: jest.fn() }),
  },
  events: {
    emit: jest.fn(),
  },
} as unknown as Phaser.Scene;

// Real business logic: CombatSystem
const combat = new CombatSystem(mockScene);

// Inject mock dependencies
combat.setUnitLookup(
  (refId) => mockUnits.get(refId),
  (refId, damage) => applyDamageToUnit(refId, damage)
);
```

## Manual Testing Approach (Current)

**Test Scenes:**
- `JeepTestScene.ts` - Dedicated test scene for Jeep vehicle rendering
- Test scenes allow manual verification of functionality
- Scenes can be swapped in via scene launcher

**Game Testing:**
- Play through gameplay manually
- Verify combat calculations work correctly
- Check waypoint following and pathfinding
- Confirm UI interactions respond properly

**Debugging:**
- Game instance exposed globally: `(window as any).game` in `main.ts`
- Console logging for combat events, unit spawning, zone captures
- Development mode with Vite hot reload

## Coverage

**Requirements:** No coverage targets enforced

**What's NOT Tested (No Test Suite):**
- Combat damage calculations
- Pathfinding and waypoint following
- Zone capture logic
- AI decision making
- Unit spawning and object management
- Selection and command processing
- Animation frame updates

**Type Coverage:**
All source files have complete type annotations due to strict TypeScript config.

## Test-Friendly Code Patterns

**Existing Patterns That Support Testing:**

1. **Dependency Injection via Constructor:**
```typescript
constructor(scene: Phaser.Scene) {
  this.scene = scene;
}
```

2. **Callback Registration Pattern:**
```typescript
public setUnitLookup(getUnitInfo, applyDamage): void {
  this.getUnitInfo = getUnitInfo;
  this.applyDamageToUnit = applyDamage;
}
```

3. **Pure Functions for Calculations:**
`CombatSystem.getCombatStats()` is static and deterministic:
```typescript
public static getCombatStats(objectType: ObjectType, objectId: number): CombatStats | null {
  switch (objectType) {
    case ObjectType.OBJECT_ROBOT:
      return ROBOT_COMBAT_STATS[objectId as RobotType] ?? null;
    // ...
  }
}
```

4. **Event Emission:**
```typescript
this.scene.events.emit("missileExplosion", missile);
```

## Recommendations for Future Testing

**Phase 1 - Unit Tests:**
- Test combat system calculations: damage, hit chance, snipe logic
- Test pathfinding algorithms
- Test zone capture logic
- Test zone ownership calculations

**Phase 2 - Integration Tests:**
- Test unit spawning with ObjectManager
- Test waypoint system with path calculation
- Test production system building units
- Test zone system with flag capture

**Phase 3 - E2E Tests:**
- Use Phaser test utilities or Cypress
- Test full gameplay sequences
- Test multiplayer scenarios (if networked)
- Test UI interactions

**Testing Library Recommendations:**
- `jest` or `vitest` for unit/integration tests
- `Phaser Testing Plugin` or `phaser-testing` for Phaser-specific testing
- Keep minimal mocking - test real business logic

---

*Testing analysis: 2026-01-24*
