# Phase 2: Texture Atlas Migration — Context

## Goal

Sprites load from packed atlases instead of 1600+ individual files, with team color tinting support.

## User Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| Team colors | Pre-baked atlases | Keep current approach: separate atlas per team. Matches existing asset structure, simpler rendering, no shader work needed |
| Tooling | free-tex-packer | Open source, Node.js based, Phaser JSON export, CLI automation, MIT license |
| Atlas organization | By category | robots.json, vehicles.json, buildings.json, effects.json — logical grouping at 2048x2048 |

## Current State Analysis

### Asset Counts

- **9,624 total image files** in assets/
- SpriteLoader.ts: 1,623 lines loading individual images
- Current load time: 30+ seconds (user report from research)

### Sprite Categories (for atlas organization)

| Category | Description | Team-colored? |
|----------|-------------|---------------|
| robots | Stand, walk, fire, death animations | Yes (4 teams) |
| vehicles | Base, turret, damaged, animations | Partially (some turrets neutral) |
| cannons | Empty, equipped, fire, place, wasted | Yes (4 teams) |
| buildings | Factories, forts, animations | Yes (4 teams) |
| effects | Explosions, fire, particles, exhaust | No |
| cursors | Team cursors, reaction cursors | Partially |
| map_items | Rocks, huts, items, flags | Partially |
| terrain | Tiles per planet type | No |

### Original C Engine Approach

From `source/zteam.cpp`:
- Uses **16-color palette replacement** system
- Palette files: `assets/teams/{team}_palette.bmp`
- `ZTeam::Make()` does pixel-by-pixel replacement at load time
- Base team is RED_TEAM — other teams derived from red base

**We are NOT replicating this** — we keep pre-baked team sprites.

### Phaser Atlas Requirements

Phaser 3 uses JSON atlas format:
```json
{
  "frames": {
    "sprite_name": {
      "frame": {"x": 0, "y": 0, "w": 64, "h": 64},
      "sourceSize": {"w": 64, "h": 64},
      "spriteSourceSize": {"x": 0, "y": 0, "w": 64, "h": 64}
    }
  },
  "meta": {
    "image": "atlas.png",
    "size": {"w": 2048, "h": 2048}
  }
}
```

### SpriteLoader API Compatibility

Current public API to preserve:
- `getRobotSpriteKey(animation, team, rotation, frame, robotType?)`
- `getVehicleBodyKey(vehicleType, team, rotation, frame, damaged?)`
- `getVehicleTurretKey(vehicleType, rotation)`
- `SpriteLoader.loadRobotSprites()`, `loadVehicleSprites()`, etc.

Unit code expects these key patterns:
- `robot_stand_blue_r045`
- `robot_walk_red_r180_n02`
- `vehicle_light_base_green_r000_n01`

**Key insight:** Atlas frame names should match existing key patterns exactly.

## Success Criteria (from ROADMAP)

1. All unit sprites load from texture atlases (not individual files)
2. Load time reduced significantly (target: under 5 seconds vs current 30+)
3. Team colors (red, blue, green, etc.) render correctly on all units
4. SpriteLoader API remains compatible with existing unit code
5. WebGL batching enabled (fewer draw calls visible in dev tools)

## Atlas Plan

| Atlas | Contents | Teams | Est. Frames |
|-------|----------|-------|-------------|
| robots_red.json | All robot sprites | Red | ~2000 |
| robots_blue.json | All robot sprites | Blue | ~2000 |
| robots_green.json | All robot sprites | Green | ~2000 |
| robots_yellow.json | All robot sprites | Yellow | ~2000 |
| vehicles_red.json | Vehicle body/turret sprites | Red | ~600 |
| vehicles_blue.json | Vehicle body/turret sprites | Blue | ~600 |
| vehicles_green.json | Vehicle body/turret sprites | Green | ~600 |
| vehicles_yellow.json | Vehicle body/turret sprites | Yellow | ~600 |
| cannons_red.json | Cannon sprites | Red | ~300 |
| cannons_blue.json | Cannon sprites | Blue | ~300 |
| cannons_green.json | Cannon sprites | Green | ~300 |
| cannons_yellow.json | Cannon sprites | Yellow | ~300 |
| buildings_red.json | Building sprites | Red | ~400 |
| buildings_blue.json | Building sprites | Blue | ~400 |
| buildings_green.json | Building sprites | Green | ~400 |
| buildings_yellow.json | Building sprites | Yellow | ~400 |
| effects.json | Explosions, fire, particles | Neutral | ~200 |
| cursors.json | All cursor sprites | Mixed | ~150 |
| map_items.json | Rocks, huts, flags | Mixed | ~100 |

**Total: ~19 atlas files** (vs 9,624 individual files)

## Technical Considerations

### free-tex-packer Setup

```bash
npm install --save-dev free-tex-packer-core
```

Script to pack sprites:
```javascript
const texturePacker = require('free-tex-packer-core');
// Configure for Phaser 3 JSON hash format
```

### Build Integration

Add npm script: `npm run pack-sprites`
- Runs before build
- Reads from assets/units/, assets/buildings/, etc.
- Outputs to assets/atlases/

### Migration Path

1. Generate atlases alongside existing individual files
2. Update SpriteLoader to load from atlases
3. Keep frame key names identical for compatibility
4. Remove individual load calls, replace with atlas load
5. Verify all units render correctly
6. Delete individual files (optional, for repo size)

## Risks

| Risk | Mitigation |
|------|------------|
| Atlas exceeds texture limits | Keep under 2048x2048, split if needed |
| Frame naming mismatch | Auto-generate names from path structure |
| Missing sprites in atlas | Validate atlas contains all expected keys |
| Build time increase | Cache atlas generation, only regenerate on change |

---
*Created: 2026-01-25 during /gsd:discuss-phase 2*
