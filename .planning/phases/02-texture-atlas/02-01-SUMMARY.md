---
phase: 02-texture-atlas
plan: 01
subsystem: asset-pipeline
tags: [texture-atlas, free-tex-packer, npm-scripts, build-tooling]

dependency-graph:
  requires: []
  provides: [atlas-generation-tooling, pack-atlases-script]
  affects: [02-02-atlas-loader-migration]

tech-stack:
  added: [free-tex-packer-core, glob, tsx]
  patterns: [Phaser3-JSON-atlas-format, category-based-atlas-organization]

key-files:
  created:
    - client/scripts/pack-atlases.ts
    - client/tsconfig.node.json
    - .gitignore
  modified:
    - client/package.json

decisions:
  - id: frame-name-matching
    summary: Frame names match SpriteLoader.ts patterns exactly
    rationale: Enables drop-in atlas loading without changing sprite key references
  - id: generated-atlases-gitignored
    summary: Atlases excluded from git (regenerated on build)
    rationale: Avoid bloating repo with generated binaries
  - id: neutral-sprites-in-red-atlas
    summary: Neutral sprites (turrets, effects) packed into red team atlas
    rationale: Avoids duplication across team atlases

metrics:
  duration: 6 min
  completed: 2026-01-25
---

# Phase 02 Plan 01: Atlas Generation Tooling Summary

**One-liner:** Created free-tex-packer script that packs 9,509 sprites into 19 atlases with Phaser 3 JSON format

## What Was Built

### Atlas Packer Script (`client/scripts/pack-atlases.ts`)

A comprehensive Node.js script using free-tex-packer-core that:

1. **Scans asset directories** for sprites organized by category:
   - `assets/units/robots/` - Robot animations (stand, walk, fire, death, idle)
   - `assets/units/vehicles/` - Vehicle base, turret, damaged sprites
   - `assets/units/cannons/` - Cannon empty, equipped, fire, place sprites
   - `assets/buildings/` - Factory buildings, forts, animations
   - `assets/other/` - Effects, cursors, map items

2. **Generates team-colored atlases** (4 teams x 4 categories = 16 atlases):
   - `robots_red.json/png`, `robots_blue.json/png`, etc.
   - `vehicles_red.json/png`, `vehicles_blue.json/png`, etc.
   - `cannons_red.json/png`, `cannons_blue.json/png`, etc.
   - `buildings_red.json/png`, `buildings_blue.json/png`, etc.

3. **Generates neutral atlases** (3 atlases):
   - `effects.json/png` - Explosions, fire, particles, grenades
   - `cursors.json/png` - All cursor sprites
   - `map_items.json/png` - Rocks, huts, flags, team markers

4. **Frame naming convention** matches SpriteLoader.ts exactly:
   - `robot_stand_{team}_r{rot}` - Stand frames
   - `robot_walk_{team}_r{rot}_n{frame}` - Walk animation
   - `robot_{type}_fire_{team}_r{rot}_n{frame}` - Fire animation
   - `vehicle_{type}_base_{team}_r{rot}_n{frame}` - Vehicle body
   - `vehicle_{type}_top_r{rot}` - Turret (neutral)
   - `cannon_{type}_{state}_{team}_r{rot}_n{frame}` - Cannon states

### Atlas Statistics

| Category | Atlases | Total Frames | Max Size |
|----------|---------|--------------|----------|
| Robots | 4 | 2,236 | 256x2048 |
| Vehicles | 4 | 824 | 1024x2048 |
| Cannons | 4 | 318 | 1024x256 |
| Buildings | 4 | 296 | 2048x2048 |
| Effects | 1 | 55 | 64x512 |
| Cursors | 1 | 196 | 1024x128 |
| Map Items | 1 | 93 | 256x512 |
| **Total** | **19** | **4,018** | - |

**Reduction:** 9,509 individual PNG files -> 19 atlas PNGs + 19 JSON files

### NPM Integration

```json
{
  "scripts": {
    "pack-atlases": "tsx scripts/pack-atlases.ts"
  }
}
```

Run with: `npm run pack-atlases`
Duration: ~2.5 seconds

## Commits

| Hash | Type | Description |
|------|------|-------------|
| 4bf62f6 | feat | Add atlas packer script and npm commands |
| dcecb50 | chore | Add .gitignore with generated atlas exclusion |

## Deviations from Plan

None - plan executed exactly as written.

## Technical Notes

### Phaser 3 Atlas Format

free-tex-packer generates the newer Phaser 3 "textures" array format:

```json
{
  "textures": [{
    "image": "atlas.png",
    "frames": [
      { "filename": "sprite_name", "frame": {...} }
    ]
  }]
}
```

This is compatible with `this.load.atlas()` in Phaser 3.

### Neutral Sprites Strategy

Neutral sprites (non-team-colored) are packed into the `_red` team atlas to avoid duplication:
- Light/medium tank turrets (no team color)
- Jeep empty/fire/under sprites
- Cannon empty sprites (gatling, howitzer rotation)
- Building animations (spin, lights, etc.)
- Fort terrain sprites

Plan 02 will need to load both team-specific and neutral atlases.

### ES Module Compatibility

Script uses ES modules with `import.meta.url` for `__dirname` equivalent:

```typescript
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
```

## Next Phase Readiness

Plan 02-02 can now:
1. Load generated atlases using `this.load.atlas()`
2. Reference frames by the exact keys SpriteLoader.ts expects
3. Remove individual sprite loading calls
4. Verify rendering matches original

No blockers identified.
