# Project Research Summary

**Project:** Z.O.D. Engine TypeScript Port - Milestone 2
**Domain:** Classic RTS Single-Player Port (C++ SDL to TypeScript Phaser 3.70)
**Researched:** 2026-01-25
**Confidence:** HIGH

---

## Executive Summary

This research covers completing the Z.O.D. Engine port's visual and audio polish: sprite animations, sound effects, and UI systems. The existing 18,000 LOC codebase already has partial infrastructure (SpriteLoader.ts, SoundSystem.ts, HUDScene.ts) that needs completion rather than replacement. All recommendations use Phaser 3.70's built-in APIs with zero new dependencies.

The recommended approach is straightforward: complete the existing systems using established Phaser patterns. Migrate from individual sprite files to texture atlases for performance (currently 1600+ HTTP requests at load). Implement audio sprites for the ~90 sound types. Use three-tier UI architecture (HUDScene for game UI, DOM overlay for menus, NineSlice for panels).

Key risk is **C++ to JavaScript behavioral differences** causing mechanical parity breaks. Critical areas: integer division rounding (use `Math.trunc()`), RNG determinism (implement seeded LCG), floating-point precision drift, and sprite coordinate origin differences. These are well-understood problems with documented solutions.

---

## Key Recommendations

### Technology Stack (from STACK.md)

**Zero new dependencies.** Complete existing Phaser 3.70 systems:

- **Texture Atlases (JSON Hash format):** Replace 1600+ individual images with 6-10 packed atlases. Reduces HTTP requests, enables WebGL batching.
- **Phaser AnimationManager:** Already partially implemented in SpriteLoader.ts. Continue pattern with atlas-based frame names.
- **Phaser WebAudioSoundManager + Audio Sprites:** SoundSystem.ts already structured correctly. Add audioSprite loading for ~90 sound effects.
- **Three-tier UI:** HUDScene (game HUD), DOM overlay (menus), NineSlice (panels). Avoids 200KB+ rexUI dependency.

### Feature Prioritization (from FEATURES.md)

**Must Have (Table Stakes):**
- Unit walk/attack/death animations (8-direction, 17 unit types)
- Sound effects (weapons, voice responses, computer voice)
- Victory/defeat screens with stats
- Game pause functionality

**Should Have (Polish):**
- Unit portraits (distinctive Z.O.D. feature, HIGH complexity)
- Production progress bars and queue display
- Computer voice messages for major events
- Damage visual states (smoke/fire at health thresholds)

**Defer to Post-MVP:**
- Grenade system (complex, secondary combat mechanic)
- Dynamic music (audio polish)
- Animal spawning (purely atmospheric)
- Fog of war on minimap

### Architecture Approach (from ARCHITECTURE.md)

Keep pre-baked team colors (correct for this game). Extract reusable `AnimationStateMachine` class to replace duplicated animation logic across Robot/Vehicle/Cannon classes. Implement object pooling for effects (smoke, particles) to reduce GC pressure.

**Major components:**
1. **SpriteLoader.ts** - Add texture atlas support alongside existing individual loading
2. **AnimationStateMachine.ts** - New reusable FSM component with state transitions and callbacks
3. **EffectPool.ts** - Object pooling for frequently created/destroyed visual effects

---

## Critical Pitfalls (from PITFALLS.md)

1. **Integer Division Rounding** - JavaScript `Math.floor()` rounds toward -infinity; C++ truncates toward zero. Use `Math.trunc()` for all division from C++ source. Critical for negative coordinates.

2. **RNG Non-Determinism** - JavaScript `Math.random()` cannot be seeded. Implement seeded LCG matching glibc's constants for gameplay randomness. OK to use `Math.random()` for visual-only effects.

3. **Floating-Point Precision Drift** - C++ uses 32-bit floats in many calculations. Use `Math.fround()` for critical values or accept documented tolerances. Important for combat damage accumulation.

4. **Game Loop Timing** - Browser RAF runs at variable rates (60/120/144Hz). Implement fixed timestep with accumulator for deterministic simulation. Original uses 300ms animation intervals.

5. **Sprite Coordinate Origin Mismatch** - SDL renders from top-left; Phaser defaults to center. Recent Jeep fix shows this is actively being addressed. Document chosen approach project-wide.

---

## Implementation Order

### Phase 1: Audio System (Low Risk, Quick Win)
**Rationale:** SoundSystem.ts framework exists; just needs asset loading
**Delivers:** Sound effects for combat, UI, voice responses
**Effort:** ~1 week
**Avoids:** Silent game feels broken to players

### Phase 2: Texture Atlas Migration (Medium Risk, High Impact)
**Rationale:** Prerequisite for performant animations; reduces 1600+ HTTP requests to 6-10
**Delivers:** Packed sprite atlases, updated SpriteLoader.ts
**Effort:** ~1 week (includes tooling setup)
**Avoids:** Load time issues, draw call overhead with many units

### Phase 3: Animation System Completion (Low Risk, Core Feature)
**Rationale:** Depends on Phase 2 for efficient sprite loading
**Delivers:** Walk, attack, death animations for all 17 unit types
**Effort:** ~2 weeks
**Avoids:** Units looking static/broken

### Phase 4: UI Polish (Medium Risk, User-Facing)
**Rationale:** Builds on working game systems
**Delivers:** Victory/defeat screens, game pause, map selection, settings
**Effort:** ~2 weeks

### Phase 5: Extended Polish (Optional)
**Rationale:** Distinctive features that complete the experience
**Delivers:** Unit portraits, production UI improvements, computer voice
**Effort:** ~2-3 weeks

---

## Open Questions

1. **Atlas Generation Tooling** - TexturePacker (paid) vs ShoeBox (free) vs free-tex-packer? Decision impacts asset pipeline setup.

2. **Animation State Machine Adoption** - Migrate existing units to new FSM pattern or leave working code alone? Risk vs maintainability tradeoff.

3. **Multiplayer Determinism** - Current implementation may have floating-point drift. Is lockstep multiplayer a future goal? If yes, need stricter determinism now.

4. **Portrait System Complexity** - 65 animation sequences, 11 eye positions, 16 mouth shapes. Build now or defer? HIGH complexity but distinctive to Z.O.D.

5. **Fixed Timestep Implementation** - Use Phaser's built-in fps config or implement custom accumulator? Current code uses default Phaser timing.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All Phaser 3.70 built-in, official docs verified |
| Features | HIGH | Derived from original C++ source code analysis |
| Architecture | HIGH | Based on existing codebase patterns + Phaser best practices |
| Pitfalls | HIGH | Well-documented C++/JS differences with verified solutions |

**Overall confidence:** HIGH

The research is grounded in original source code analysis and official Phaser documentation. Pitfalls are well-understood problems in the game porting domain with established solutions.

---

## Sources

### Primary (HIGH confidence)
- Original C++ source code (262 files in ./source/)
- Phaser 3.70 official documentation (animations, audio, textures)
- Existing TypeScript codebase analysis

### Secondary (MEDIUM confidence)
- TexturePacker tutorials for Phaser atlas creation
- Community articles on Phaser optimization
- Game porting patterns from MDN and game dev blogs

---
*Research completed: 2026-01-25*
*Ready for roadmap: yes*
