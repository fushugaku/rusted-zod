# Domain Pitfalls: C++ to TypeScript/Phaser Game Engine Porting

**Project:** Z.O.D. Engine Port
**Domain:** RTS Game Engine Porting (C++ SDL to TypeScript Phaser 3.70)
**Researched:** 2026-01-25
**Constraint:** EXACT mechanical parity with original C engine
**Status:** ~50% complete, core systems working

---

## Critical Pitfalls

Mistakes that cause desyncs, incorrect gameplay behavior, or require rewrites.

---

### Pitfall 1: Integer Division Rounding Differences

**What goes wrong:** JavaScript `Math.floor()` rounds toward negative infinity, while C++ integer division truncates toward zero. This causes different results for negative numbers.

**Why it happens:** Developers assume `Math.floor(x/y)` behaves like C++ `(int)(x/y)`.

**Example:**
```cpp
// C++ - truncates toward zero
int result = -7 / 2;  // result = -3
```
```typescript
// JavaScript - rounds toward negative infinity
const result = Math.floor(-7 / 2);  // result = -4  (WRONG!)
```

**Consequences:**
- Position calculations drift over time at negative coordinates
- Tile coordinate calculations off by one at map edges
- Movement direction calculations inverted at boundaries
- Pathfinding edge detection fails

**Prevention:**
```typescript
// Use Math.trunc() for C++ integer division parity
const cppStyleDivision = (a: number, b: number): number => Math.trunc(a / b);

// Or use bitwise OR for small positive numbers (forces 32-bit int)
const fastIntDiv = (a: number, b: number): number => (a / b) | 0;
```

**Detection:**
- Unit tests with negative coordinate edge cases
- Compare pathfinding results at map boundaries (0,0 corner especially)
- Test attack range calculations when target coords are negative relative to attacker

**Observed in Z.O.D.:** Current `Pathfinding.ts` uses `Math.floor()` for tile conversion. Verify behavior at map edges.

**Confidence:** HIGH (verified from MDN documentation and language specification)

---

### Pitfall 2: Random Number Generator Non-Parity

**What goes wrong:** JavaScript `Math.random()` cannot be seeded, producing different sequences than C++ `rand()`. This breaks deterministic replay and potentially multiplayer sync.

**Why it happens:** JavaScript's built-in RNG is designed for convenience, not reproducibility. C++ `srand(seed)` produces repeatable sequences.

**Original C++ Pattern (from common.h):**
```cpp
inline double frand() { return (rand()%10001) / 10000.0; }
```

**Original uses `rand()` for (non-exhaustive list from source):**
- Idle animation selection: `rand() % 10`, `rand() % 3`, `rand() % 4` (zrobot.cpp:86-94)
- Grenade scatter: `rand() % 48 - 24` pixels (zobject.cpp)
- Damage variance: `(rand() % 10000) / 10000.0 > damage_chance`
- Vehicle track placement: `rand() % 5` (zvehicle.cpp:113)
- Effect particle spread: Various files
- Lid open delay: `0.1 * (rand() % 15)` (zvehicle.cpp:218)

**Consequences:**
- Game replays diverge from original recordings
- Lockstep multiplayer networking may become impossible
- Testing becomes non-reproducible
- "Random" behaviors feel different to players who know original

**Prevention:**
```typescript
// Implement a seeded PRNG matching C's Linear Congruential Generator
class SeededRandom {
  private seed: number;

  constructor(seed: number) {
    this.seed = seed & 0x7fffffff;  // Ensure positive 31-bit
  }

  // Matches glibc LCG implementation
  next(): number {
    this.seed = ((this.seed * 1103515245 + 12345) & 0x7fffffff);
    return this.seed;
  }

  // Match original frand() exactly
  frand(): number {
    return (this.next() % 10001) / 10000.0;
  }

  // Match original rand() % n pattern
  randInt(max: number): number {
    return this.next() % max;
  }
}

// For visual-only randomness (effects), Math.random() is fine
const visualRandom = Math.random();
```

**Detection:**
- Record random sequences in C++ build with known seed
- Compare against TypeScript implementation first 100 values
- Test: `srand(12345)` should produce identical sequences

**Confidence:** HIGH (verified from C++ documentation and source code analysis)

---

### Pitfall 3: Floating-Point Precision Divergence

**What goes wrong:** C++ `float` (32-bit) has different precision than JavaScript `number` (64-bit double). Accumulated differences cause state drift over time.

**Why it happens:** JavaScript only has 64-bit doubles. C++ may use 32-bit floats, and some original calculations explicitly cast to float.

**Original C++ Precision Patterns:**
```cpp
// From zobject.cpp - explicit float cast for distance
dist = sqrt((float)((dx * dx) + (dy * dy)));

// From zsettings.h - mix of int and double
double attack_damage;       // 0.0011046 for Grunt
int attack_radius;          // 120 pixels
double attack_damage_chance;// 0.7

// From ztime.cpp - all double
double ztime;
double game_speed;
```

**Consequences:**
- Position drift over thousands of frames
- Damage calculations slightly off after many attacks
- Movement speeds imperceptibly different
- Lockstep multiplayer desyncs after extended play

**Prevention Options:**

**Option A - Simulate C++ float precision:**
```typescript
// Use Math.fround() to simulate 32-bit float
const asFloat = (n: number): number => Math.fround(n);
const dist = asFloat(Math.sqrt(asFloat(dx * dx) + asFloat(dy * dy)));
```

**Option B - Fixed-point math for critical values:**
```typescript
// Fixed-point with 16-bit fractional part
const FIXED_SHIFT = 16;
const toFixed = (n: number): number => Math.round(n * (1 << FIXED_SHIFT));
const fromFixed = (n: number): number => n / (1 << FIXED_SHIFT);
const fixedMul = (a: number, b: number): number => (a * b) >> FIXED_SHIFT;
```

**Option C - Accept divergence, document tolerances:**
```typescript
// For non-determinism-critical paths
// Document: "positions may drift by up to 0.001 pixels per frame"
```

**Detection:**
- Log position values every 1000 frames, compare C++ vs TS
- Create determinism test: same inputs should yield identical outputs
- Check cumulative damage over extended battles (thousands of attacks)

**Observed in Z.O.D.:** Current `CombatSystem.ts` uses raw JavaScript numbers. Damage values like `0.0011046` may accumulate differently. The combat system documentation notes this is currently using direct number math.

**Confidence:** HIGH (verified from source code and floating-point standards)

---

### Pitfall 4: Game Loop Timing Non-Determinism

**What goes wrong:** Browser `requestAnimationFrame` runs at variable rates (60Hz, 120Hz, 144Hz), causing physics to run at different speeds on different machines.

**Why it happens:** C++ games often use fixed timesteps; JavaScript RAF ties to display refresh rate.

**Original C++ Timing (from ztime.cpp):**
```cpp
void ZTime::UpdateTime()
{
    if(!paused)
        ztime = last_change_front_time +
                ((current_time() - last_change_back_time) * game_speed);
}
```

**Original timing intervals (from source):**
- Robot process interval: `process_time_int = 0.3` (300ms) - zrobot.cpp:50
- Grenade throw interval: `GRENADE_TIME_INT 0.15` (150ms) - zrobot.cpp:5
- Vehicle track drop: `0.2` (200ms) - zvehicle.cpp:136
- Lid animation: `0.2` (200ms) - zvehicle.cpp:248
- Attack intervals: Various per unit type in zsettings.cpp

**Consequences:**
- Units move faster on 120Hz monitors than 60Hz
- Animation timing inconsistent
- Combat DPS varies by frame rate
- Replays run at wrong speed on different hardware

**Prevention - Fixed Timestep with Accumulator:**
```typescript
const FIXED_TIMESTEP = 1000 / 60;  // 16.67ms = 60Hz simulation
let accumulator = 0;
let lastTime = 0;

function gameLoop(timestamp: number) {
  const delta = timestamp - lastTime;
  lastTime = timestamp;

  accumulator += delta;

  // Run fixed timestep updates (may run 0, 1, or multiple times)
  while (accumulator >= FIXED_TIMESTEP) {
    updateGameLogic(FIXED_TIMESTEP);  // Always same delta
    accumulator -= FIXED_TIMESTEP;
  }

  // Interpolate for smooth rendering between logic frames
  const alpha = accumulator / FIXED_TIMESTEP;
  render(alpha);

  requestAnimationFrame(gameLoop);
}
```

**Phaser 3.70 Configuration:**
```typescript
const config: Phaser.Types.Core.GameConfig = {
  fps: {
    target: 60,
    forceSetTimeOut: true,  // More consistent than RAF
    deltaHistory: 10,
    smoothStep: true
  }
};
```

**Detection:**
- Test on 60Hz and 144Hz monitors
- Measure game time vs wall time over 5 minutes
- Compare replay recordings across different hardware
- Time how long it takes a unit to walk a fixed distance

**Observed in Z.O.D.:** Current implementation uses Phaser's default timing. May need custom fixed-step implementation for simulation-critical code.

**Confidence:** HIGH (verified through Phaser docs and game loop patterns)

---

### Pitfall 5: Bitwise Operations 32-Bit Truncation

**What goes wrong:** JavaScript converts numbers to 32-bit signed integers for bitwise operations, causing overflow and unexpected results for large values.

**Why it happens:** JavaScript numbers are 64-bit doubles but bitwise ops truncate to 32-bit signed.

**Consequences:**
- Values > 2^31 wrap incorrectly
- Bit flags corrupted for large flag sets
- Hash calculations produce different results
- Large reference IDs may collide

**Prevention:**
```typescript
// For values that might exceed 31 bits, use explicit handling
const safeAnd = (a: number, b: number): number => {
  if (a > 0x7FFFFFFF || b > 0x7FFFFFFF) {
    return Number(BigInt(a) & BigInt(b));
  }
  return a & b;
};

// For bit shifting, convert to unsigned first
const unsignedRightShift = (n: number, bits: number): number => {
  return (n >>> 0) >> bits;  // >>> 0 converts to unsigned 32-bit
};

// For reference IDs, use smaller numbers or strings
let nextRefId = 1;  // Reset when it gets too large
```

**Detection:**
- Test with reference IDs > 2 billion
- Test bit flag operations with all flags set
- Verify behavior after creating many objects

**Observed in Z.O.D.:** Current `GameObject.ts` uses numeric refIds. Verify they stay within safe range during long play sessions.

**Confidence:** HIGH (verified from JavaScript specification)

---

### Pitfall 6: Sprite Coordinate Origin Mismatch

**What goes wrong:** Units render offset from where they should be, collision detection fails, click targeting is inaccurate.

**Why it happens:**
- SDL renders sprites from **top-left corner** by default
- Phaser 3 renders sprites from **center** by default (origin 0.5, 0.5)
- Original code has complex offset calculations (see vjeep.cpp render offsets)

**Original C++ offset patterns:**
```cpp
// From zvehicle.cpp - robot position in tank
const int robot_shift_x[8] = {3, -1, -3, -7, -10, -7, -4, 0};
const int robot_shift_y[8] = {0, -4, -6, -4,   0,  1,  1, 1};

// From vjeep.cpp - firing position offsets
const int fire_x[8] = {0, 13, 21, 25, 25, 22, 9, -1};
const int fire_y[8] = {2, -3, 2, 12, 19, 26, 26, 17};
```

**Prevention:**
```typescript
// Option A: Set Phaser origin to match SDL (top-left)
sprite.setOrigin(0, 0);

// Option B: Adjust all position calculations
// Add half-width/height to stored positions

// Document which approach is used project-wide!
```

**Detection:**
- Visual comparison screenshots with original
- Selection box not aligning with unit visuals
- Projectiles spawning at wrong positions

**Observed in Z.O.D.:** Recent commit "Fix Jeep rendering with proper sprite positioning from vjeep.cpp" indicates this is actively being addressed.

**Confidence:** HIGH (verified from Phaser docs and source code)

---

## Moderate Pitfalls

Mistakes that cause noticeable differences or technical debt.

---

### Pitfall 7: Animation Frame Timing Mismatch

**What goes wrong:** Animations look subtly wrong - too fast, too slow, or jerky.

**Original C++ (from zrobot.cpp):**
```cpp
process_time_int = 0.3;  // 300ms between animation frames
next_process_time = the_time + process_time_int;

// Frame counts from Init():
// walk[4], cigarette[11], beer[10], full_area_scan[12]
// beat_ground[9], celebrate[4], head_stretch[11]
```

**Prevention:**
```typescript
// Use millisecond timestamps, not frame counts
class AnimationController {
  private nextFrameTime: number = 0;
  private frameIntervalMs: number;

  constructor(intervalMs: number = 300) {  // Match original 0.3
    this.frameIntervalMs = intervalMs;
  }

  update(gameTimeMs: number): boolean {
    if (gameTimeMs < this.nextFrameTime) return false;
    this.nextFrameTime = gameTimeMs + this.frameIntervalMs;
    return true;  // Advance animation frame
  }
}
```

**Detection:**
- Compare idle animation speeds side-by-side with original
- Time grenade throw animation (should be 4 frames at 150ms = 600ms total)

**Confidence:** HIGH (verified from source code)

---

### Pitfall 8: Pathfinding Algorithm Behavioral Differences

**What goes wrong:** Units take different paths than in original, get stuck in different places.

**Original A* characteristics (from zpath_finding_astar.cpp):**
- Uses Manhattan distance heuristic
- Has robot vs vehicle distinction (`is_robot` flag)
- Vehicles check 2x2 tile footprints
- Performance throttled with `SDL_Delay(10)` every 90 iterations
- Specific tie-breaking in equivalent path choices

**Current Implementation:** Uses EasyStar.js library

**Consequences:**
- Units choose different "equally good" routes
- Vehicles may handle diagonal movement differently
- Performance characteristics differ

**Prevention:**
- Accept visual path differences if mechanical parity maintained (same path LENGTH)
- Or port exact A* algorithm for perfect parity
- Test specific scenarios with known expected paths

**Detection:**
- Overlay original paths on ported paths
- Specific test cases for tricky map areas
- Compare path lengths, not exact coordinates

**Confidence:** MEDIUM (EasyStar.js may be acceptable if only visual difference)

---

### Pitfall 9: Memory Management / Object Lifecycle

**What goes wrong:** Memory leaks accumulate, objects reference destroyed entities, game slows down over time.

**Why it happens:**
- C++ uses manual memory management
- JavaScript has GC but retains references
- Phaser has its own destroy lifecycle

**Prevention:**
```typescript
// Object pooling for frequently created/destroyed objects
class ObjectPool<T> {
  private pool: T[] = [];
  private factory: () => T;

  constructor(factory: () => T, initialSize: number = 100) {
    this.factory = factory;
    for (let i = 0; i < initialSize; i++) {
      this.pool.push(factory());
    }
  }

  acquire(): T {
    return this.pool.pop() ?? this.factory();
  }

  release(obj: T): void {
    this.pool.push(obj);
  }
}

// Use for: missiles, path nodes, effect particles
```

**Detection:**
- Profile with Chrome DevTools during heavy combat
- Monitor heap size over 10+ minute play sessions
- Watch for GC pauses during battles

**Confidence:** MEDIUM (general JavaScript pattern)

---

### Pitfall 10: Event Processing Order

**What goes wrong:** Simultaneous attacks resolved in different order, tie-breaking produces different winners.

**Why it happens:** C++ iterates containers in memory order; JavaScript iteration may differ.

**Prevention:**
```typescript
// Use arrays with explicit sorting for deterministic order
const sortedUnits = [...units.values()].sort((a, b) => a.refId - b.refId);
for (const unit of sortedUnits) {
  processUnit(unit);
}

// Or use Map with consistent insertion order (guaranteed in JS)
// But verify insertion order matches original
```

**Detection:**
- Test simultaneous events (two units kill each other same frame)
- Test zone capture with multiple units entering simultaneously

**Confidence:** MEDIUM (depends on whether original order matters)

---

## Minor Pitfalls

Annoyances that are fixable without major refactoring.

---

### Pitfall 11: Null/Undefined vs NULL_TEAM

**What goes wrong:** Original uses `NULL_TEAM` enum value (0); JavaScript may use `null` or `undefined`, causing different truthiness checks.

**Original C++ (from constants.h):**
```cpp
enum team_type { NULL_TEAM, RED_TEAM, BLUE_TEAM, ... };
```

**Prevention:**
```typescript
// Always use explicit enum, never null/undefined for team
enum TeamType {
  NULL_TEAM = 0,  // Matches C++ exactly
  RED_TEAM = 1,
  BLUE_TEAM = 2,
}

// Check explicitly
if (unit.team === TeamType.NULL_TEAM)  // NOT: if (!unit.team)
```

**Observed in Z.O.D.:** Current TypeScript correctly uses `TeamType.NULL` enum. Verified in types/enums.ts.

**Confidence:** HIGH (verified in current implementation)

---

### Pitfall 12: Asset Path Case Sensitivity

**What goes wrong:** Works on Windows, fails on Linux server.

**Original path patterns (from zrobot.cpp):**
```cpp
sprintf(filename_c, "assets/units/robots/stand_%s_r%03d.png",
        team_type_string[i].c_str(), ROTATION[j]);
```

**Prevention:**
- Use lowercase for all asset paths
- Validate asset names in build pipeline
- Test on Linux (CI/CD)

**Confidence:** HIGH (common deployment issue)

---

## RTS-Specific Pitfalls

---

### Pitfall 13: Area Damage Falloff Formula

**What goes wrong:** Explosions deal wrong damage at different distances.

**Original C++ (from zserver.cpp ProcessMissileDamage):**
```cpp
// Linear falloff: 0% damage at radius edge, 100% at center
damage_amount *= (1.0 - (mag / radius));
```

**Current Implementation Check (from GAP-ANALYSIS.md):**
The current implementation may use different falloff. Verify formula matches exactly.

**Prevention:**
```typescript
// CORRECT: Linear falloff matching original
const falloff = 1 - (distance / radius);
const actualDamage = baseDamage * Math.max(0, falloff);
```

**Confidence:** HIGH (verified from source code)

---

### Pitfall 14: Driver Snipe Mechanics

**What goes wrong:** Sniper kills behave differently - wrong units can snipe, wrong vehicles can be sniped.

**Original mechanics (from combat-mechanics.md):**

Units that CAN snipe: Gatling, Grunt, Laser, Psycho, Pyro, Sniper, Jeep

Units that CAN BE sniped: Heavy, Light, Medium, Jeep, Cannons (with driver)

**Logic (from zobject.cpp):**
```cpp
if(can_snipe && attack_object->CanBeSniped() &&
   ((rand() % 10000) / 10000.0 <= snipe_chance))
{
    attack_object->DamageDriverHealth(damage);
}
```

**Result of successful snipe:** Vehicle becomes `NULL_TEAM` (neutral), not destroyed.

**Prevention:** Verify all snipe flags match original exactly. Current CombatSystem.ts appears to implement this but verify against source.

**Confidence:** HIGH (verified from source code)

---

## Testing Strategies for Mechanical Parity

### 1. Determinism Unit Tests
```typescript
describe('Combat Parity', () => {
  it('should match C++ damage values', () => {
    // Grunt damage: 0.0011046 * MAX_UNIT_HEALTH (10000)
    const damage = 0.0011046 * 10000;
    expect(damage).toBeCloseTo(11.046, 3);
  });

  it('should match area damage falloff', () => {
    // At 50% radius, should deal 50% damage
    const radius = 40;
    const distance = 20;
    const falloff = 1 - (distance / radius);
    expect(falloff).toBe(0.5);
  });
});
```

### 2. Seeded Random Sequence Comparison
```typescript
describe('Random Parity', () => {
  it('should match C rand() sequence', () => {
    const rng = new SeededRandom(12345);
    // Values from running C code with srand(12345)
    const expectedFirst10 = [/* capture from C */];
    for (let i = 0; i < 10; i++) {
      expect(rng.next()).toBe(expectedFirst10[i]);
    }
  });
});
```

### 3. Visual Regression Testing
- Screenshot original at specific game frames
- Screenshot TypeScript at identical game state
- Automated pixel diff comparison

### 4. Timing Validation
```typescript
describe('Timing Parity', () => {
  it('robot should walk across map in same time', () => {
    // Setup: Robot at 0,0 moving to 160,0 (10 tiles)
    // Speed: 14 pixels/second (from zsettings.cpp grunt move_speed)
    // Expected: ~11.4 seconds
    const expectedTime = 160 / 14;
    // Simulate and measure
  });
});
```

### 5. Combat Outcome Logging
```typescript
// Log every combat interaction
interface CombatLog {
  frame: number;
  attackerRefId: number;
  targetRefId: number;
  hitRoll: number;
  damage: number;
  targetHealthAfter: number;
}
// Compare logs between C++ and TS runs
```

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Movement System | Integer division (#1), timing (#4) | Use Math.trunc(), fixed timestep |
| Combat System | RNG (#2), floating-point (#3), falloff (#13) | Seeded RNG, verify formulas |
| Pathfinding | Algorithm differences (#8) | Test path lengths, not exact routes |
| Animation | Frame timing (#7), origin (#6) | Match interval values from source |
| Multiplayer (future) | ALL determinism pitfalls | Full lockstep validation required |
| Replays | Accumulated drift | Checksum game state periodically |
| AI Behavior | RNG for decisions | Separate RNG stream, log decisions |

---

## Quick Reference Checklist

Before claiming parity for any system:

- [ ] Integer division uses `Math.trunc()` where original uses int division
- [ ] Random numbers use seeded PRNG for gameplay (Math.random OK for visuals)
- [ ] Timing uses fixed timestep or matches original intervals exactly
- [ ] Sprite origins documented and consistent
- [ ] Animation frame counts and intervals match source
- [ ] Team comparisons use enum values, not null/undefined
- [ ] Combat formulas verified against source (damage, falloff, snipe)
- [ ] Tested with negative coordinates / edge cases
- [ ] Visual comparison against original screenshots

---

## Sources

### Number Precision and Division
- [JavaScript Numbers - W3Schools](https://www.w3schools.com/js/js_numbers.asp)
- [Number - MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number)
- [Math.floor() - MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/floor)
- [Don't Confuse Integer Division with Floor Division](https://marcelkliemannel.com/articles/2021/dont-confuse-integer-division-with-floor-division/)

### Game Loop Timing
- [Fix Your Timestep - Game Programming Patterns](https://gameprogrammingpatterns.com/game-loop.html)
- [JavaScript Game Loops Explained](https://isaacsukin.com/news/2015/01/detailed-explanation-javascript-game-loops-and-timing)
- [Phaser TimeStep Documentation](https://docs.phaser.io/api-documentation/class/core-timestep)
- [Performant Game Loops in JavaScript](https://www.aleksandrhovhannisyan.com/blog/javascript-game-loop/)

### RNG and Determinism
- [rand() and srand() in C++ - GeeksforGeeks](https://www.geeksforgeeks.org/cpp/rand-and-srand-in-ccpp/)
- [Deterministic Lockstep Demo (browser)](https://github.com/pietrobassi/deterministic-lockstep-demo/)
- [Lockstep Networking - SnapNet](https://www.snapnet.dev/blog/netcode-architectures-part-1-lockstep/)
- [Don't use rand() in C++ - Codeforces](https://codeforces.com/blog/entry/61587)

### Bitwise Operations
- [JavaScript Bitwise Operations - W3Schools](https://www.w3schools.com/js/js_bitwise.asp)
- [Bitwise Operators Integer Overflow - JS++ Blog](https://www.onux.com/jspp/blog/bitwise-operators-and-specification-compliant-integer-overflow-optimizations/)

### Original Source Analysis
- Z.O.D. Engine source: `zsettings.cpp`, `ztime.cpp`, `zrobot.cpp`, `zvehicle.cpp`, `zobject.cpp`, `zpath_finding_astar.cpp`, `common.h`, `constants.h`
- Existing documentation: `docs/requirements/combat-mechanics.md`, `docs/requirements/GAP-ANALYSIS.md`
