# Sprite/Animation Architecture for Z.O.D. Engine

**Project:** Z.O.D. Engine TypeScript Port (Phaser 3.70)
**Domain:** RTS Unit Rendering with Sprite Animations
**Researched:** 2026-01-25
**Confidence:** HIGH (existing codebase analysis + official Phaser docs + community patterns)

---

## Executive Summary

This document provides architecture recommendations for sprite atlas structure, team color tinting, animation state machines, and performance optimization for the Z.O.D. Engine TypeScript port using Phaser 3.

**Key Findings:**

1. **Sprite Atlas Structure:** Current individual image loading works but should migrate to packed atlases for better memory usage and load times. Recommended: per-team atlases for team-colored sprites, shared atlases for neutral content.

2. **Team Color Implementation:** Pre-baked team colors (current approach) is correct for this project. Runtime tinting would require custom shaders and offers no visual benefit over existing pre-rendered sprites.

3. **Animation State Machine:** The current enum-based state with manual frame tracking works but could be formalized into a reusable `AnimationStateMachine` class for better maintainability.

4. **Performance:** The system-based architecture already used in the codebase is appropriate. Main optimization opportunities: texture atlases, object pooling for effects, and texture swap caching.

---

## 1. Recommended Sprite Atlas Structure

### Current Implementation (Individual Images)

The existing `SpriteLoader.ts` loads ~1600+ individual images:

```typescript
// Current: SpriteLoader.ts lines 178-191
for (let rot = 0; rot < 8; rot++) {
  const rotStr = ROTATIONS[rot];
  for (let frame = 0; frame < 4; frame++) {
    const frameStr = frame.toString().padStart(2, "0");
    const key = `robot_walk_${teamStr}_r${rotStr}_n${frameStr}`;
    this.scene.load.image(key, `assets/units/robots/walk_${teamStr}_r${rotStr}_n${frameStr}.png`);
  }
}
```

**Problems:**
- Thousands of HTTP requests during load
- Each small image gets padded to power-of-2 GPU texture
- WebGL cannot batch sprites from different textures

### Recommended: Category-Based Texture Atlases

Pack sprites into atlases organized by unit type and team:

```
assets/atlases/
  robots/
    robots_red.json + robots_red.png    # All red robot frames (~500 frames)
    robots_blue.json + robots_blue.png
    robots_green.json + robots_green.png
    robots_yellow.json + robots_yellow.png
    robots_shared.json + robots_shared.png  # Team-neutral (deaths, grenades)

  vehicles/
    vehicles_red.json + vehicles_red.png
    vehicles_blue.json + vehicles_blue.png
    vehicles_green.json + vehicles_green.png
    vehicles_yellow.json + vehicles_yellow.png
    vehicles_shared.json + vehicles_shared.png  # Empty vehicles, turrets

  cannons/
    cannons_all.json + cannons_all.png  # All cannon sprites (smaller set)

  effects/
    effects.json + effects.png          # All effect sprites
```

### Atlas Frame Naming Convention

Use Phaser 3 JSONHash format with keys matching current sprite key patterns:

```json
// robots_blue.json
{
  "frames": {
    "robot_walk_blue_r000_n00": { "frame": {"x":0,"y":0,"w":24,"h":24} },
    "robot_walk_blue_r000_n01": { "frame": {"x":24,"y":0,"w":24,"h":24} },
    "robot_walk_blue_r000_n02": { "frame": {"x":48,"y":0,"w":24,"h":24} },
    "robot_walk_blue_r000_n03": { "frame": {"x":72,"y":0,"w":24,"h":24} },
    "robot_stand_blue_r000": { "frame": {"x":96,"y":0,"w":24,"h":24} },
    "robot_grunt_fire_blue_r000_n00": { "frame": {"x":120,"y":0,"w":24,"h":24} }
  }
}
```

### Loading Implementation

```typescript
// Updated SpriteLoader.ts
public loadRobotAtlases(): void {
  const teams = ['red', 'blue', 'green', 'yellow'];

  for (const team of teams) {
    this.scene.load.atlas(
      `robots_${team}`,
      `assets/atlases/robots/robots_${team}.png`,
      `assets/atlases/robots/robots_${team}.json`
    );
  }

  // Shared team-neutral animations
  this.scene.load.atlas(
    'robots_shared',
    'assets/atlases/robots/robots_shared.png',
    'assets/atlases/robots/robots_shared.json'
  );
}
```

### Backward Compatibility

Support both individual images and atlases during migration:

```typescript
// SpriteKeyResolver.ts
public resolveTexture(key: string): { atlas?: string; frame?: string; texture?: string } {
  // Try atlas first
  for (const atlasKey of this.loadedAtlases) {
    const atlas = this.scene.textures.get(atlasKey);
    if (atlas.has(key)) {
      return { atlas: atlasKey, frame: key };
    }
  }

  // Fall back to individual texture
  if (this.scene.textures.exists(key)) {
    return { texture: key };
  }

  return {}; // Not found
}
```

---

## 2. Team Color Implementation Approach

### Current Approach: Pre-Baked Colors (RECOMMENDED - KEEP)

The Z.O.D. Engine uses pre-rendered team-colored sprites:

```
base_red_r000_n00.png     # Red team base sprite
base_blue_r000_n00.png    # Blue team base sprite
base_green_r000_n00.png   # Green team base sprite
base_yellow_r000_n00.png  # Yellow team base sprite
```

**This is correct for Z.O.D. because:**

1. **Original Assets:** The C++ game used pre-baked colors; ported sprites match exactly
2. **Pixel-Perfect Fidelity:** Shading, highlights, and team colors are designed together
3. **Selective Coloring:** Only specific pixels are team-colored (not the whole sprite)
4. **No Shader Complexity:** Simpler rendering pipeline, no custom shaders needed

### Why NOT Runtime Tinting

Phaser 3's built-in tinting (`sprite.setTint()`) would not work correctly:

```typescript
// This would tint ALL pixels, not just team-colored regions
sprite.setTint(0xff0000); // Makes entire sprite red

// This replaces color entirely (loses shading)
sprite.setTintFill(0xff0000); // Flat red with alpha from texture
```

**Problems with runtime tinting for Z.O.D.:**
- Original sprites have palette-based coloring (specific pixels are team color)
- Tinting affects entire sprite, not selective regions
- Would lose the careful shading/highlighting in original art

### Custom Shader Approach (NOT RECOMMENDED for this project)

A custom shader could selectively replace palette colors:

```glsl
// Fragment shader for palette-based team color
precision mediump float;
uniform sampler2D u_texture;
uniform vec3 u_teamColor;
uniform vec3 u_paletteKey;  // Color to replace (e.g., magenta placeholder)
varying vec2 v_texCoord;

void main() {
    vec4 texColor = texture2D(u_texture, v_texCoord);

    // Check if pixel matches palette key color
    if (distance(texColor.rgb, u_paletteKey) < 0.05) {
        // Replace with team color while preserving brightness
        float brightness = (texColor.r + texColor.g + texColor.b) / 3.0;
        gl_FragColor = vec4(u_teamColor * brightness, texColor.a);
    } else {
        gl_FragColor = texColor;
    }
}
```

**Why NOT to implement this:**
- Adds shader complexity with no visual benefit
- Pre-baked sprites already exist and look correct
- Would need to re-author all sprites with placeholder colors
- Performance overhead for real-time color replacement

### Verdict

**Keep pre-baked team colors.** The existing approach is correct for this game.

---

## 3. Animation State Machine Pattern

### Current Implementation Analysis

Robot.ts uses an enum-based state with manual frame management:

```typescript
// Current pattern in Robot.ts
export enum RobotAnimation {
  STAND = "stand",
  WALK = "walk",
  FIRE = "fire",
  DIE1 = "die1",
  // ... 15+ animation states
}

protected currentAnimation: RobotAnimation = RobotAnimation.STAND;
protected animationFrame: number = 0;
protected animationTimer: number = 0;
protected isInIdleAction: boolean = false;  // Extra flag!

protected updateAnimation(delta: number): void {
  const animationSpeed = this.getAnimationSpeedForType();
  this.animationTimer += delta;

  if (this.animationTimer >= animationSpeed) {
    this.animationTimer = 0;
    this.animationFrame++;

    const maxFrames = this.getAnimationFrameCount();
    if (this.animationFrame >= maxFrames) {
      this.onAnimationComplete();
    }
  }
}
```

**Issues:**
- Animation logic duplicated across Robot, Vehicle, Cannon classes
- State transitions are implicit (spread across `setMode()`, `onAnimationComplete()`, etc.)
- Extra flags like `isInIdleAction` complicate state reasoning

### Recommended: Reusable AnimationStateMachine

Extract into a reusable component:

```typescript
// animation/AnimationStateMachine.ts

export interface AnimationStateConfig {
  name: string;
  frameCount: number;
  frameRate: number;  // frames per second
  loop: boolean;
  onEnter?: () => void;
  onUpdate?: (frame: number) => void;
  onComplete?: () => void;
  transitions: Record<string, string>; // event -> nextStateName
}

export class AnimationStateMachine {
  private states = new Map<string, AnimationStateConfig>();
  private currentState: AnimationStateConfig | null = null;
  private currentFrame = 0;
  private frameTimer = 0;

  private sprite: Phaser.GameObjects.Sprite;
  private getSpriteKey: (stateName: string, frame: number) => string;

  constructor(
    sprite: Phaser.GameObjects.Sprite,
    getSpriteKey: (stateName: string, frame: number) => string
  ) {
    this.sprite = sprite;
    this.getSpriteKey = getSpriteKey;
  }

  public addState(config: AnimationStateConfig): this {
    this.states.set(config.name, config);
    return this;
  }

  public setState(stateName: string): void {
    const state = this.states.get(stateName);
    if (!state || state === this.currentState) return;

    this.currentState = state;
    this.currentFrame = 0;
    this.frameTimer = 0;

    state.onEnter?.();
    this.updateSprite();
  }

  public trigger(event: string): void {
    if (!this.currentState) return;

    const nextStateName = this.currentState.transitions[event];
    if (nextStateName) {
      this.setState(nextStateName);
    }
  }

  public update(delta: number): void {
    if (!this.currentState) return;

    const frameTime = 1000 / this.currentState.frameRate;
    this.frameTimer += delta;

    if (this.frameTimer >= frameTime) {
      this.frameTimer -= frameTime;
      this.currentFrame++;

      if (this.currentFrame >= this.currentState.frameCount) {
        if (this.currentState.loop) {
          this.currentFrame = 0;
        } else {
          this.currentFrame = this.currentState.frameCount - 1;
          this.currentState.onComplete?.();
          return;
        }
      }

      this.currentState.onUpdate?.(this.currentFrame);
      this.updateSprite();
    }
  }

  private updateSprite(): void {
    if (!this.currentState) return;
    const key = this.getSpriteKey(this.currentState.name, this.currentFrame);
    if (this.sprite.scene.textures.exists(key)) {
      this.sprite.setTexture(key);
    }
  }

  public getCurrentState(): string | null {
    return this.currentState?.name ?? null;
  }

  public getCurrentFrame(): number {
    return this.currentFrame;
  }
}
```

### Usage in Robot Class

```typescript
// In Robot constructor
this.animFSM = new AnimationStateMachine(
  this.robotSprite!,
  (state, frame) => this.getSpriteKeyForState(state, frame)
);

this.animFSM
  .addState({
    name: 'stand',
    frameCount: 1,
    frameRate: 1,
    loop: true,
    transitions: {
      'move': 'walk',
      'attack': 'fire',
      'die': 'die1',
      'idle': 'cigarette'
    }
  })
  .addState({
    name: 'walk',
    frameCount: ROBOT_ANIMATION_FRAME_COUNTS.WALK,  // 4
    frameRate: 10,
    loop: true,
    transitions: {
      'stop': 'stand',
      'attack': 'fire',
      'die': 'die1'
    }
  })
  .addState({
    name: 'fire',
    frameCount: this.getFireAnimationFrames(),
    frameRate: 15,
    loop: false,
    onComplete: () => this.animFSM.trigger('stop'),
    transitions: {
      'stop': 'stand',
      'die': 'die1'
    }
  })
  .addState({
    name: 'die1',
    frameCount: 10,
    frameRate: 6,  // 160ms per frame = 6.25 FPS
    loop: false,
    onComplete: () => this.onDeathComplete(),
    transitions: {}  // Terminal state
  });

this.animFSM.setState('stand');

// In update():
this.animFSM.update(delta);

// State changes:
this.animFSM.trigger('move');    // stand -> walk
this.animFSM.trigger('attack');  // walk -> fire
```

### Death Animation Selection

The original game has 5 robot death types selected by damage cause:

```typescript
public selectDeathAnimation(cause: DeathCause): string {
  switch (cause) {
    case DeathCause.FIRE:
      return 'die1';  // Burn/melt death (17 frames)
    case DeathCause.EXPLOSION:
      return 'die2';  // Explosion knockback (10 frames)
    case DeathCause.BULLET:
      return Math.random() < 0.5 ? 'die3' : 'die4';  // Random bullet death
    case DeathCause.MISSILE:
      return 'die5';  // Robot flip (33 frames)
    default:
      return 'die1';
  }
}
```

---

## 4. Performance Optimization Strategies

### Current Performance Profile

Based on codebase analysis:

| Metric | Current State | Target |
|--------|---------------|--------|
| HTTP requests at load | 1600+ individual images | 10-20 atlas files |
| Texture memory | ~50MB (padded small images) | ~15MB (packed atlases) |
| Draw calls per frame | High (different textures) | Low (batched from atlases) |
| Effect objects | Created/destroyed per effect | Pooled and reused |

### Optimization 1: Texture Atlas Batching (HIGH IMPACT)

WebGL can batch sprites from the same texture into a single draw call:

```
Before: 100 robots from different images = 100+ draw calls
After:  100 robots from team atlas = 1-4 draw calls
```

Implementation: See Section 1 (Sprite Atlas Structure).

### Optimization 2: Object Pooling for Effects (MEDIUM IMPACT)

Current `EffectsSystem.ts` creates/destroys objects frequently:

```typescript
// Current pattern (EffectsSystem.ts)
const smoke = this.scene.add.circle(x, y, size, color);
// ... later
smoke.destroy();
```

**Recommended Pool Implementation:**

```typescript
// effects/EffectPool.ts
export class EffectPool<T extends Phaser.GameObjects.GameObject> {
  private pool: T[] = [];
  private activeCount = 0;
  private factory: () => T;
  private reset: (obj: T) => void;

  constructor(initialSize: number, factory: () => T, reset: (obj: T) => void) {
    this.factory = factory;
    this.reset = reset;

    for (let i = 0; i < initialSize; i++) {
      const obj = this.factory();
      obj.setActive(false);
      obj.setVisible(false);
      this.pool.push(obj);
    }
  }

  public acquire(): T {
    let obj: T;

    if (this.activeCount < this.pool.length) {
      obj = this.pool[this.activeCount]!;
    } else {
      obj = this.factory();
      this.pool.push(obj);
    }

    this.activeCount++;
    obj.setActive(true);
    obj.setVisible(true);
    this.reset(obj);
    return obj;
  }

  public release(obj: T): void {
    const index = this.pool.indexOf(obj);
    if (index === -1 || index >= this.activeCount) return;

    obj.setActive(false);
    obj.setVisible(false);

    this.activeCount--;
    [this.pool[index], this.pool[this.activeCount]] =
      [this.pool[this.activeCount]!, this.pool[index]!];
  }
}
```

**Usage in EffectsSystem:**

```typescript
private smokePool = new EffectPool<Phaser.GameObjects.Arc>(
  50,
  () => this.scene.add.circle(0, 0, 4, 0x333333),
  (smoke) => {
    smoke.setPosition(0, 0);
    smoke.setAlpha(1);
    smoke.setScale(1);
  }
);
```

### Optimization 3: Texture Swap Caching (LOW-MEDIUM IMPACT)

Current code calls `setTexture()` every frame:

```typescript
// Current (every frame)
protected renderSprite(): void {
  const spriteKey = this.getSpriteKey();
  if (this.scene.textures.exists(spriteKey)) {
    this.robotSprite.setTexture(spriteKey);  // Called even if same
  }
}
```

**Optimized: Cache last key:**

```typescript
private lastSpriteKey = '';

protected renderSprite(): void {
  const spriteKey = this.getSpriteKey();
  if (spriteKey !== this.lastSpriteKey) {
    this.lastSpriteKey = spriteKey;
    if (this.scene.textures.exists(spriteKey)) {
      this.robotSprite.setTexture(spriteKey);
    }
  }
}
```

### Optimization 4: Visibility Culling (LOW IMPACT)

Skip rendering for off-screen units:

```typescript
public updateVisibility(camera: Phaser.Cameras.Scene2D.Camera): void {
  const bounds = camera.worldView;
  const padding = 64;

  const cullBounds = new Phaser.Geom.Rectangle(
    bounds.x - padding,
    bounds.y - padding,
    bounds.width + padding * 2,
    bounds.height + padding * 2
  );

  for (const obj of this.objects.values()) {
    obj.setVisible(cullBounds.contains(obj.x, obj.y));
  }
}
```

**Note:** Phaser handles basic culling automatically; this adds skipping update logic.

### Performance Budget

For 100+ units at 60 FPS:

| System | Budget | Current | Notes |
|--------|--------|---------|-------|
| Animation updates | <2ms | ~1ms | Manual frame tracking is fast |
| State machines | <1ms | N/A | Would add ~0.01ms per unit |
| Effect updates | <2ms | ~3ms | Pooling would reduce |
| Render | <10ms | Varies | Atlases would help |

---

## 5. Vehicle Multi-Sprite Pattern

### Current Jeep Implementation (Good Pattern)

The `Jeep.ts` demonstrates proper multi-layer rendering:

```typescript
// Jeep has 3 sprite layers in Container:
// 1. underSprite - wheels/chassis (behind body, hidden on side views)
// 2. bodySprite - team-colored body with suspension bounce
// 3. jeepTurretSprite - gun turret with independent rotation

const TURRET_X = [0, 6, 12, 20, 25, 20, 15, 5];  // Per-direction offsets
const TURRET_Y = [2, 7, 4, 8, 2, -4, -3, -4];
const TURRET_SHIFT_X = [0, -2, -5, -8, -10, -8, -5, -2];
const TURRET_SHIFT_Y = [0, 0, 0, 0, 0, 5, 6, 5];
```

### Extractable Pattern

```typescript
// objects/MultiLayerUnit.ts
export interface SpriteLayer {
  name: string;
  sprite: Phaser.GameObjects.Sprite;
  offsetX: number[];  // Per-direction offsets [0..7]
  offsetY: number[];
  getKey: (state: UnitState) => string;
  visible?: (state: UnitState) => boolean;
}

export abstract class MultiLayerUnit extends GameObject {
  protected layers: SpriteLayer[] = [];

  protected addLayer(config: Omit<SpriteLayer, 'sprite'>): SpriteLayer {
    const sprite = this.scene.add.sprite(0, 0, '');
    sprite.setOrigin(0, 0);
    this.add(sprite);

    const layer: SpriteLayer = { ...config, sprite };
    this.layers.push(layer);
    return layer;
  }

  protected updateLayers(state: UnitState): void {
    const dir = state.direction;

    for (const layer of this.layers) {
      const isVisible = layer.visible?.(state) ?? true;
      layer.sprite.setVisible(isVisible);

      if (isVisible) {
        const key = layer.getKey(state);
        if (this.scene.textures.exists(key)) {
          layer.sprite.setTexture(key);
        }
        layer.sprite.setPosition(
          layer.offsetX[dir] ?? 0,
          layer.offsetY[dir] ?? 0
        );
      }
    }
  }
}
```

---

## 6. Recommended File Structure

```
client/src/
  animation/
    index.ts                    # Public exports
    AnimationConstants.ts       # (existing) Timing values from C++
    AnimationStateMachine.ts    # Reusable FSM (new)
    SpriteKeyResolver.ts        # Centralized key generation (new)

    manifests/                  # (future) Asset definitions
      RobotManifest.ts
      VehicleManifest.ts
      CannonManifest.ts

  assets/
    SpriteLoader.ts             # (existing) Keep, add atlas support
    AssetLoader.ts              # (future) Manifest-driven loader
    EffectPool.ts               # (new) Object pooling

  objects/
    GameObject.ts               # (existing) Base class
    MultiLayerUnit.ts           # (new) Multi-sprite pattern
    units/
      Robot.ts                  # (existing) Uses AnimationStateMachine
      Vehicle.ts                # (existing) Multi-layer base
      Cannon.ts                 # (existing) Uses AnimationStateMachine
```

---

## 7. Implementation Priority

### Phase 1: Quick Wins (Low Risk)

1. Add texture swap caching to unit classes
2. Create `AnimationStateMachine.ts` as optional component
3. Add `EffectPool.ts` for smoke/particle effects

### Phase 2: State Machine Migration (Medium Risk)

1. Migrate Robot to use AnimationStateMachine
2. Migrate Vehicle to use AnimationStateMachine
3. Migrate Cannon to use AnimationStateMachine
4. Remove duplicated animation logic

### Phase 3: Atlas Migration (Higher Risk)

1. Create texture atlases with TexturePacker
2. Update SpriteLoader to support atlas loading
3. Verify all animations work
4. Remove individual image loading

---

## Sources

### Codebase Analysis

- `/Users/georgiis/Projects/zod-source/client/src/assets/SpriteLoader.ts` (1623 lines)
- `/Users/georgiis/Projects/zod-source/client/src/objects/units/Robot.ts`
- `/Users/georgiis/Projects/zod-source/client/src/objects/units/Vehicle.ts`
- `/Users/georgiis/Projects/zod-source/client/src/objects/units/Cannon.ts`
- `/Users/georgiis/Projects/zod-source/client/src/objects/units/vehicles/Jeep.ts`
- `/Users/georgiis/Projects/zod-source/client/src/animation/AnimationConstants.ts`
- `/Users/georgiis/Projects/zod-source/client/src/effects/EffectsSystem.ts`

### Phaser 3 Official Documentation

- [Phaser Textures Concepts](https://docs.phaser.io/phaser/concepts/textures) - Atlas loading, texture management
- [Phaser Animations Concepts](https://docs.phaser.io/phaser/concepts/animations) - Animation creation, mixing, chaining
- [AnimationState API](https://docs.phaser.io/api-documentation/class/animations-animationstate) - Playback control
- [MultiPipeline (WebGL Batching)](https://docs.phaser.io/api-documentation/class/renderer-webgl-pipelines-multipipeline) - Texture batching

### Tutorials and Community Patterns

- [How to create sprite sheets for Phaser with TexturePacker](https://www.codeandweb.com/texturepacker/tutorials/how-to-create-sprite-sheets-for-phaser) - Atlas creation
- [Working with Texture Atlases in Phaser 3](https://airum82.medium.com/working-with-texture-atlases-in-phaser-3-25c4df9a747a) - Atlas usage patterns
- [State Pattern for Character Movement in Phaser 3](https://blog.ourcade.co/posts/2020/state-pattern-character-movement-phaser-3/) - FSM implementation
- [Phaser FSM Tutorial](https://osmose.ceo/blog/phaser-finite-state-machine/) - State machine patterns
- [How To Use State Machines To Control Behavior And Animations In Phaser](https://gamedevacademy.org/how-to-use-state-machines-to-control-behavior-and-animations-in-phaser/) - Animation FSM

### Performance Resources

- [Phaser Dev Log 242](https://phaser.io/devlogs/242) - Multi-texture support, batching
- [Phaser Dev Log 167](https://phaser.io/devlogs/167) - Pipeline consolidation

---

## Confidence Assessment

| Area | Level | Rationale |
|------|-------|-----------|
| Sprite Atlas Structure | HIGH | Phaser docs + TexturePacker integration is well-documented |
| Team Color Approach | HIGH | Pre-baked is correct; runtime tinting analysis based on Phaser API |
| Animation State Machine | HIGH | Standard pattern with multiple Phaser community examples |
| Performance Optimizations | MEDIUM | Based on community benchmarks; needs profiling on actual codebase |
| Multi-Layer Vehicle Pattern | HIGH | Based on working Jeep.ts implementation |
