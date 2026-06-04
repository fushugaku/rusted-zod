# Technology Stack: Animation, Audio, and UI Systems

**Project:** Z.O.D. Engine TypeScript Port - Milestone 2
**Researched:** 2026-01-25
**Focus:** Sprite animations, audio management, UI systems for existing Phaser 3.70 RTS game

## Executive Summary

This research covers adding animation, audio, and UI systems to an existing 18,000 LOC Phaser 3.70 TypeScript RTS game. The codebase already has animation infrastructure (`SpriteLoader.ts`, `AnimationConstants.ts`) and a sound system skeleton (`SoundSystem.ts`). Recommendations focus on completing these systems using Phaser 3's native APIs rather than introducing new dependencies.

**Key recommendation:** Use Phaser's built-in AnimationManager, WebAudioSoundManager, and Scene-based UI. Avoid external UI libraries (like rexUI) to maintain consistency with existing codebase patterns.

---

## Recommended Stack Additions

### Sprite Animation System

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Phaser AnimationManager | 3.70 (built-in) | Frame-based sprite animations | Already partially implemented in SpriteLoader.ts; complete the pattern |
| Texture Atlas (JSON Hash) | TexturePacker format | Sprite sheet organization | Reduces draw calls, WebGL texture batching, already using individual frame loading |

**Current State:**
- `SpriteLoader.ts` (1,623 lines) loads individual PNG frames via `scene.load.image()`
- `AnimationConstants.ts` (424 lines) defines timing from original C++ engine
- Animations created via `scene.anims.create()` with `generateFrameNumbers` pattern

**Recommendation:** Continue current approach but migrate to texture atlases for performance.

**Rationale:**
1. Individual frame loading works but causes 100s of HTTP requests during preload
2. Texture atlases reduce this to 1-10 requests total
3. WebGL batches sprites from same atlas, reducing draw calls significantly
4. For RTS with many units (30+ on screen), this is critical for 60fps

**Implementation Pattern:**
```typescript
// Current (individual frames):
this.scene.load.image('robot_walk_blue_r000_n00', 'assets/units/robots/walk_blue_r000_n00.png');
// ... hundreds more

// Recommended (texture atlas):
this.scene.load.atlas('units', 'assets/atlases/units.png', 'assets/atlases/units.json');

// Animation creation stays the same:
this.scene.anims.create({
  key: 'robot_walk_blue_r000',
  frames: this.scene.anims.generateFrameNames('units', {
    prefix: 'walk_blue_r000_n',
    start: 0,
    end: 3,
    zeroPad: 2
  }),
  frameRate: 10,
  repeat: -1
});
```

**Confidence:** HIGH (verified via official Phaser documentation)

### Audio System

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Phaser WebAudioSoundManager | 3.70 (built-in) | All game audio | Better performance than HTML5 Audio, supports spatial audio for RTS |
| Audio Sprites | JSON format | Sound effect organization | Reduces HTTP requests by 60-80%, better for many short effects |

**Current State:**
- `SoundSystem.ts` (415 lines) exists with proper structure
- Uses `scene.sound.add()` and `scene.sound.play()` correctly
- Positional audio calculation already implemented (lines 134-153)
- No audio assets currently loaded

**Recommendation:** Use Web Audio API exclusively with audio sprites for SFX.

**Rationale:**
1. Web Audio API is the standard for games - better performance for rapid successive sounds (gunfire, explosions)
2. Audio sprites combine many short effects into single files, matching the texture atlas pattern
3. Mobile unlock handling already implemented in `SoundSystem.ts` (lines 396-400)
4. Spatial audio support for "sounds near camera" already coded

**Implementation Pattern:**
```typescript
// Preloader - load audio sprite:
this.load.audioSprite('sfx', 'assets/audio/sfx.json', [
  'assets/audio/sfx.ogg',
  'assets/audio/sfx.mp3'  // Fallback for Safari
]);

// SoundSystem - play specific sound:
this.scene.sound.playAudioSprite('sfx', 'rifle_shot', { volume });

// Audio sprite JSON format:
{
  "resources": ["sfx.ogg", "sfx.mp3"],
  "spritemap": {
    "rifle_shot": { "start": 0, "end": 0.5 },
    "explosion_small": { "start": 0.5, "end": 1.2 },
    // ... all effects with start/end times
  }
}
```

**Confidence:** HIGH (verified via official Phaser documentation)

### UI System

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Phaser Scene (parallel UI scene) | 3.70 (built-in) | In-game HUD, unit info, production UI | Already using HUDScene.ts; extend this pattern |
| Phaser NineSlice | 3.70 (built-in) | Scalable UI panels/buttons | WebGL-only but game requires WebGL anyway |
| DOM Overlay (hybrid) | Native HTML | Menus, settings, text-heavy screens | CSS styling for complex forms/text; already configured in game |

**Current State:**
- `HUDScene.ts` runs parallel to `GameScene.ts`
- Uses Phaser's scene system correctly
- Minimap implemented as canvas-drawn UI element
- ProductionWindow uses Phaser graphics primitives

**Recommendation:** Three-tier UI architecture:

1. **HUDScene (Phaser-native):** Unit selection, minimap, resource display, quick actions
2. **DOM Overlay:** Main menu, settings, map selection, victory/defeat screens
3. **NineSlice panels:** Production queue UI, unit info panels, context menus

**Rationale:**
1. HUDScene already exists and works - keep in-game UI here
2. DOM overlay better for text-heavy screens (menus, settings) - easier CSS styling
3. NineSlice provides scalable panels without external dependencies
4. Avoids rexUI dependency - adds 200KB+ and different patterns from existing code

**Implementation Pattern:**
```typescript
// Game config - enable DOM elements:
const config: Phaser.Types.Core.GameConfig = {
  parent: 'game-container',
  dom: { createContainer: true },
  // ...
};

// Menu scene with DOM:
class MenuScene extends Phaser.Scene {
  create() {
    const menu = this.add.dom(400, 300).createFromHTML(`
      <div class="menu-container">
        <button id="play">Play</button>
        <button id="settings">Settings</button>
      </div>
    `);
    menu.addListener('click');
    menu.on('click', this.handleClick, this);
  }
}

// HUDScene with NineSlice panels:
class HUDScene extends Phaser.Scene {
  createUnitInfoPanel() {
    const panel = this.add.nineslice(
      10, 10, 'ui_panel', null,
      200, 150,
      16, 16, 16, 16  // corner sizes
    );
    // Add text/icons as children
  }
}
```

**Confidence:** HIGH (verified via official Phaser documentation)

---

## Alternatives Considered

### Animation Alternatives

| Option | Considered | Why Not |
|--------|------------|---------|
| Spine | Professional skeletal animation | Overkill for sprite-based game; original assets are frame-based |
| Aseprite integration | Native Aseprite file loading | Original assets not in Aseprite format |
| Keep individual frames | Current approach works | Performance degradation with 100s of units; HTTP request overhead |

### Audio Alternatives

| Option | Considered | Why Not |
|--------|------------|---------|
| HTML5 Audio | Simpler, wider support | Poor performance for rapid playback; no spatial audio |
| Howler.js | Popular audio library | Unnecessary - Phaser's WebAudio wrapper is sufficient |
| Web Audio directly | Maximum control | Phaser's abstraction handles browser quirks automatically |

### UI Alternatives

| Option | Considered | Why Not |
|--------|------------|---------|
| rexUI plugin | Feature-rich UI framework | 200KB+ addition; different patterns from existing code; overkill for needs |
| DOM-only | Simpler styling | Poor integration with game objects; can't overlay/blend with canvas |
| Phaser-only | Full canvas control | Text rendering expensive; CSS better for menus |
| React overlay | Modern framework | Massive dependency for simple menus; overkill |

---

## Performance Considerations for RTS

### Many Units (30-100 on screen)

| Concern | Mitigation |
|---------|------------|
| Draw calls | Use texture atlases - all units from same atlas batch together |
| Animation updates | Phaser handles efficiently; 100 sprites with same animation share frame data |
| Sound overlap | Use audio sprites; limit concurrent sounds (already coded in SoundSystem) |
| Memory | Object pooling for projectiles/effects (not units - they persist) |

### Texture Atlas Strategy

For Z.O.D. with ~4 team colors, 8 rotations per unit, and multiple animation states:

```
Recommended atlas organization:
- units_robots.png/json    (~2-4MB) - All robot sprites
- units_vehicles.png/json  (~2-4MB) - All vehicle sprites
- units_cannons.png/json   (~1-2MB) - All cannon sprites
- buildings.png/json       (~2-3MB) - All building sprites
- effects.png/json         (~1-2MB) - Explosions, smoke, fire
- ui.png/json              (~500KB) - UI elements, cursors

Total: ~10-15MB vs current individual files
Benefit: 6 HTTP requests vs hundreds; 6 texture binds vs hundreds
```

### Animation Performance Tips

```typescript
// GOOD: Create global animations once
// In PreloaderScene.create():
this.anims.create({ key: 'robot_walk_blue_r000', ... });

// BAD: Create per-sprite (wastes memory)
// Don't do this in unit constructors

// GOOD: Share animations across all sprites of same type
sprite.play('robot_walk_blue_r000');

// GOOD: Use animation chaining for sequences
sprite.play('robot_fire').chain('robot_walk');
```

---

## Asset Loading Strategy

### Preloader Scene Flow

```typescript
class PreloaderScene extends Phaser.Scene {
  preload() {
    // 1. Load texture atlases (parallel)
    this.load.atlas('units_robots', ...);
    this.load.atlas('units_vehicles', ...);
    this.load.atlas('effects', ...);
    this.load.atlas('ui', ...);

    // 2. Load audio sprites (parallel)
    this.load.audioSprite('sfx', ...);
    this.load.audioSprite('music', ...);

    // 3. Load map data (if needed)
    this.load.binary('map_data', ...);
  }

  create() {
    // 4. Create all global animations
    this.createRobotAnimations();
    this.createVehicleAnimations();
    this.createEffectAnimations();

    // 5. Start game
    this.scene.start('GameScene');
  }
}
```

### Progress Tracking

```typescript
preload() {
  this.load.on('progress', (value: number) => {
    // Update loading bar
    this.progressBar.setScale(value, 1);
  });

  this.load.on('complete', () => {
    // All assets loaded
  });
}
```

---

## Mobile Considerations

### Audio Unlock (Already Implemented)

```typescript
// SoundSystem.ts already handles this:
public resumeContext(): void {
  const soundManager = this.scene.sound as Phaser.Sound.WebAudioSoundManager;
  if (soundManager.context?.state === "suspended") {
    soundManager.context.resume();
  }
}

// Call on first user interaction
document.addEventListener('click', () => this.sound.resumeContext(), { once: true });
```

### Touch Input for UI

```typescript
// DOM elements handle touch natively
// For Phaser UI elements, input already works:
button.setInteractive();
button.on('pointerdown', callback);
```

---

## Installation Requirements

**No new dependencies needed.** All recommended technologies are built into Phaser 3.70.

```bash
# Current package.json is sufficient:
# "phaser": "^3.70.0"

# For texture atlas generation (dev tool, not runtime):
# Use TexturePacker, ShoeBox, or free-tex-packer
```

### Asset Preparation Tools (Optional)

| Tool | Purpose | Cost |
|------|---------|------|
| TexturePacker | Texture atlas generation | Paid (free trial) |
| ShoeBox | Free atlas packer | Free |
| Audiosprite (npm) | Audio sprite generation | Free |

```bash
# Generate audio sprites from individual files:
npm install -g audiosprite
audiosprite -o sfx -f json *.wav
```

---

## Sources

### Official Documentation (HIGH confidence)
- [Phaser Animations Concept Guide](https://docs.phaser.io/phaser/concepts/animations)
- [Phaser Audio Concept Guide](https://docs.phaser.io/phaser/concepts/audio)
- [Phaser Scenes Concept Guide](https://docs.phaser.io/phaser/concepts/scenes)
- [Phaser NineSlice Game Object](https://docs.phaser.io/phaser/concepts/gameobjects/nine-slice)
- [Phaser DOM Element Game Object](https://docs.phaser.io/phaser/concepts/gameobjects/dom-element)

### Community Resources (MEDIUM confidence)
- [How I optimized my Phaser 3 action game - 2025](https://franzeus.medium.com/how-i-optimized-my-phaser-3-action-game-in-2025-5a648753f62b)
- [TexturePacker Phaser Tutorial](https://www.codeandweb.com/texturepacker/tutorials/how-to-create-sprite-sheets-for-phaser)
- [rexUI Plugin Overview](https://rexrainbow.github.io/phaser3-rex-notes/docs/site/ui-overview/)

---

## Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| Animation System | HIGH | Official docs verified; existing implementation pattern in codebase |
| Audio System | HIGH | Official docs verified; SoundSystem.ts already structured correctly |
| UI Architecture | HIGH | Official docs verified; HUDScene pattern already working |
| Texture Atlas Migration | MEDIUM | Pattern is standard but requires asset pipeline changes |
| Performance Claims | MEDIUM | Based on community articles and general WebGL knowledge |

---

## Implementation Order Recommendation

Based on dependencies and risk:

1. **Audio System** (Low risk, quick win)
   - Assets: Create audio sprites from original sound files
   - Code: Complete SoundSystem.ts with audioSprite loading
   - Test: Weapon sounds, explosions, UI clicks

2. **Texture Atlas Migration** (Medium risk, high impact)
   - Tools: Set up TexturePacker or alternative
   - Assets: Generate atlases from existing frames
   - Code: Update SpriteLoader.ts to use atlases
   - Test: Verify all unit types render correctly

3. **Complete Animations** (Low risk, visual polish)
   - Code: Ensure all death, fire, walk animations work
   - Test: All unit types, all teams, all rotations

4. **UI Screens** (Medium risk, user-facing)
   - Code: Menu scene with DOM overlay
   - Code: Victory/defeat screens
   - Code: Settings panel
   - Test: Full flow from menu to game to end

---

*Research completed: 2026-01-25*
