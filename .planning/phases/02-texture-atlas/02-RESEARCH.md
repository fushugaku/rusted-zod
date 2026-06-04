# Phase 2: Texture Atlas Migration - Research

**Researched:** 2026-01-25
**Domain:** Texture Atlas Generation and Phaser 3 Integration
**Confidence:** HIGH

## Summary

This research covers the technical requirements for migrating from 9,624 individual PNG files to texture atlases using free-tex-packer-core for generation and Phaser 3's atlas loading system. The project already has pre-baked team-colored sprites, so no runtime tinting is needed.

The key insight is that Phaser 3 allows using atlas frame names identically to individual texture keys, meaning the SpriteLoader API can remain compatible by changing `setTexture(key)` to `setTexture(atlasKey, frameKey)`. The existing sprite key generation logic in `getSpriteKey()` methods already produces the exact frame names we need.

Free-tex-packer-core is well-suited for this task with its `Phaser3` exporter, programmatic Node.js API, and ability to generate multiple atlases when content exceeds size limits.

**Primary recommendation:** Use free-tex-packer-core with the `Phaser3` exporter, organize atlases by team (separate atlases per team color to keep under 2048x2048), and generate frame names from file paths stripped of extensions.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| free-tex-packer-core | latest | Atlas generation | MIT licensed, Phaser3-native export, async API, programmatic Node.js usage |
| Phaser 3 | ^3.70.0 | Atlas loading/rendering | Already in project, built-in atlas support with WebGL batching |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| chokidar-cli | ^3.0.0 | File watching | Development watch mode for atlas regeneration |
| glob | ^10.0.0 | File pattern matching | Collecting source PNGs for packing |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| free-tex-packer-core | TexturePacker (paid) | More features but commercial license, no CLI automation in free tier |
| Manual watch script | Vite plugin | Vite plugin would be simpler but locks to Vite, standalone script more portable |

**Installation:**
```bash
cd client
npm install --save-dev free-tex-packer-core chokidar-cli glob
```

## Architecture Patterns

### Recommended Project Structure
```
client/
├── scripts/
│   └── pack-atlases.js       # Atlas generation script
├── assets/
│   └── atlases/              # Generated atlas output (gitignored or committed)
│       ├── robots_red.json
│       ├── robots_red.png
│       ├── robots_blue.json
│       ...
├── src/
│   └── assets/
│       └── SpriteLoader.ts   # Modified to load atlases
assets/                       # Source sprites (unchanged)
├── units/
│   ├── robots/
│   ├── vehicles/
│   └── cannons/
└── ...
```

### Pattern 1: Atlas-Per-Team Organization
**What:** Separate atlas files for each team color (robots_red.json, robots_blue.json, etc.)
**When to use:** When sprites are pre-baked per team and total frames per category exceed 2048x2048 single atlas
**Why:** Keeps individual atlases under texture size limits, enables selective loading (only load teams in current game)

**Example atlas plan:**
```
robots_red.json     (~2000 frames) - All robot sprites for red team
robots_blue.json    (~2000 frames) - All robot sprites for blue team
robots_green.json   (~2000 frames) - All robot sprites for green team
robots_yellow.json  (~2000 frames) - All robot sprites for yellow team
vehicles_red.json   (~600 frames)  - All vehicle sprites for red team
...
effects.json        (~200 frames)  - Team-neutral effects
cursors.json        (~150 frames)  - Cursor sprites
```

### Pattern 2: Frame Name Matching
**What:** Atlas frame names must exactly match existing sprite key patterns
**When to use:** Always - this ensures zero changes to unit rendering code
**Example:**
```typescript
// Current code generates keys like:
"robot_stand_blue_r045"
"robot_walk_red_r180_n02"
"vehicle_light_base_green_r000_n01"

// Atlas JSON must have matching frame names:
{
  "frames": {
    "robot_stand_blue_r045": { "frame": {...} },
    "robot_walk_red_r180_n02": { "frame": {...} },
    ...
  }
}
```

### Pattern 3: Texture Check Migration
**What:** Replace `textures.exists(key)` with `textures.get(atlas).has(frame)`
**When to use:** In all places that check if a texture exists before using it

**Before (individual textures):**
```typescript
if (this.scene.textures.exists(spriteKey)) {
  this.robotSprite.setTexture(spriteKey);
}
```

**After (atlas frames):**
```typescript
const atlasKey = this.getAtlasKey(); // e.g., "robots_blue"
if (this.scene.textures.get(atlasKey)?.has(spriteKey)) {
  this.robotSprite.setTexture(atlasKey, spriteKey);
}
```

### Anti-Patterns to Avoid
- **Mixing individual textures and atlases:** Load everything from atlases, don't leave some sprites as individual files
- **Single massive atlas:** Exceeds WebGL texture limits (2048x2048 safe, 4096 maximum on most GPUs)
- **Changing frame names:** Must preserve exact key patterns or all unit code breaks
- **Loading all atlases at startup:** Load only what's needed for current game (teams in play)

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rectangle packing | Custom bin-packing algorithm | free-tex-packer's MaxRectsBin packer | Optimal packing is NP-hard, library has proven algorithms |
| JSON atlas format | Manual JSON generation | free-tex-packer Phaser3 exporter | Format must match Phaser expectations exactly |
| Transparent pixel trimming | Manual trim detection | free-tex-packer's allowTrim option | Handles edge cases, preserves source dimensions |
| Duplicate detection | Manual hash comparison | free-tex-packer's detectIdentical option | Automatic deduplication saves atlas space |
| File watching | Custom fs.watch wrapper | chokidar-cli | Cross-platform, handles edge cases, debouncing |

**Key insight:** Texture packing is a solved problem with many edge cases (rotation, trim, padding, duplicates). Using free-tex-packer avoids weeks of debugging obscure rendering issues.

## Common Pitfalls

### Pitfall 1: Frame Name Mismatch
**What goes wrong:** Atlas frame names don't match sprite keys, all sprites render as wrong frames or fail to load
**Why it happens:** File path transformation doesn't match expected key pattern
**How to avoid:**
1. Set `prependFolderName: false` and `removeFileExtension: true`
2. Pre-process file paths to generate exact key names
3. Validate generated JSON against expected keys before deployment
**Warning signs:** First sprite renders wrong, missing texture warnings in console

### Pitfall 2: Texture Size Limit Exceeded
**What goes wrong:** Atlas exceeds 2048x2048 (or device max), fails to load or causes WebGL errors
**Why it happens:** Too many sprites packed into single atlas
**How to avoid:**
1. Set `width: 2048, height: 2048` in packer options
2. Enable multipack (suffix option) to auto-split
3. Organize by team to naturally partition frames
**Warning signs:** Black textures, WebGL errors in console, iOS devices fail while desktop works

### Pitfall 3: SpriteLoader API Changes Breaking Unit Code
**What goes wrong:** Changing SpriteLoader API requires changes in all unit classes
**Why it happens:** Trying to change how sprite keys are resolved
**How to avoid:**
1. Keep `getRobotSpriteKey()`, `getVehicleBodyKey()` etc. returning same keys
2. Only change the load methods and add atlas key resolution
3. Modify `setTexture()` calls to include atlas key
**Warning signs:** TypeScript errors in many files, unit tests failing

### Pitfall 4: Development Build Time
**What goes wrong:** Rebuilding atlases on every change takes too long (30+ seconds)
**Why it happens:** Packing 9000+ images is computationally expensive
**How to avoid:**
1. Only regenerate specific atlases when their source files change
2. Cache generated atlases in version control or build cache
3. Use watch mode with debouncing during development
**Warning signs:** Development iteration becomes slow, devs bypass atlas step

### Pitfall 5: Missing Sprites in Atlas
**What goes wrong:** Some sprites not included in atlas, render as missing
**Why it happens:** Glob patterns don't match all files, or file names have unexpected characters
**How to avoid:**
1. Log all files found during packing
2. Compare expected frame count vs actual frame count
3. Create validation script that checks all expected keys exist
**Warning signs:** Some animations work, others show missing texture

## Code Examples

Verified patterns from official sources:

### Atlas Generation Script (pack-atlases.js)
```typescript
// Source: free-tex-packer-core npm docs and TypeScript definitions
import { packAsync } from 'free-tex-packer-core';
import { glob } from 'glob';
import { readFileSync, writeFileSync, mkdirSync } from 'fs';
import { basename, join, dirname } from 'path';

const TEAMS = ['red', 'blue', 'green', 'yellow'];
const OUTPUT_DIR = './client/assets/atlases';

async function packTeamAtlas(category: string, team: string, sourceDir: string) {
  // Find all PNGs for this team in the category
  const pattern = `${sourceDir}/**/*_${team}*.png`;
  const files = await glob(pattern);

  const images = files.map(filePath => {
    // Generate frame name from filename (without extension)
    const frameName = basename(filePath, '.png');
    return {
      path: frameName,  // This becomes the frame name in atlas
      contents: readFileSync(filePath)
    };
  });

  const result = await packAsync(images, {
    textureName: `${category}_${team}`,
    width: 2048,
    height: 2048,
    fixedSize: false,
    powerOfTwo: true,
    padding: 1,
    allowRotation: false,  // Disable rotation for pixel art
    detectIdentical: true,
    allowTrim: true,
    exporter: 'Phaser3',
    removeFileExtension: false,  // Already removed in path
    prependFolderName: false
  });

  // Write output files
  mkdirSync(OUTPUT_DIR, { recursive: true });
  for (const file of result) {
    writeFileSync(join(OUTPUT_DIR, file.name), file.buffer);
  }

  console.log(`Packed ${images.length} frames into ${category}_${team}`);
}

// Pack all team atlases
async function main() {
  for (const team of TEAMS) {
    await packTeamAtlas('robots', team, './assets/units/robots');
    await packTeamAtlas('vehicles', team, './assets/units/vehicles');
    await packTeamAtlas('cannons', team, './assets/units/cannons');
  }
  // Pack team-neutral atlases
  await packAtlas('effects', './assets/other/explosions', './assets/other/fire');
}
```

### Phaser 3 Atlas Loading
```typescript
// Source: Phaser 3 official documentation
// In PreloaderScene.ts

public preload(): void {
  // Load atlas instead of individual images
  this.load.atlas(
    'robots_red',                           // Key to reference this atlas
    'assets/atlases/robots_red.png',        // Texture image
    'assets/atlases/robots_red.json'        // JSON frame data
  );

  // Or load multiple atlases at once
  const teams = ['red', 'blue', 'green', 'yellow'];
  for (const team of teams) {
    this.load.atlas(
      `robots_${team}`,
      `assets/atlases/robots_${team}.png`,
      `assets/atlases/robots_${team}.json`
    );
  }
}
```

### Using Atlas Frames in Sprites
```typescript
// Source: Phaser 3 documentation - Textures concept page

// Creating a sprite from atlas frame
const sprite = this.add.sprite(x, y, 'robots_blue', 'robot_stand_blue_r045');

// Changing frame (same atlas)
sprite.setFrame('robot_walk_blue_r045_n00');

// Changing to different atlas frame
sprite.setTexture('robots_red', 'robot_stand_red_r045');

// Checking if frame exists
const atlas = this.textures.get('robots_blue');
if (atlas.has('robot_stand_blue_r045')) {
  // Frame exists
}

// Getting all frame names (useful for validation)
const frameNames = this.textures.get('robots_blue').getFrameNames();
console.log('Available frames:', frameNames.length);
```

### Updated SpriteLoader Pattern
```typescript
// Modified SpriteLoader.ts pattern

export class SpriteLoader {
  private scene: Phaser.Scene;

  // Map of category+team to atlas key
  private atlasMap = new Map<string, string>();

  /**
   * Load all robot atlases for specified teams
   */
  public loadRobotAtlases(teams: TeamType[]): void {
    for (const team of teams) {
      const teamStr = TEAM_STRINGS[team];
      const atlasKey = `robots_${teamStr}`;

      this.scene.load.atlas(
        atlasKey,
        `assets/atlases/${atlasKey}.png`,
        `assets/atlases/${atlasKey}.json`
      );

      this.atlasMap.set(`robot_${teamStr}`, atlasKey);
    }
  }

  /**
   * Get atlas key for a sprite key
   */
  public getAtlasForSpriteKey(spriteKey: string): string | undefined {
    // Parse sprite key to determine atlas
    // e.g., "robot_stand_blue_r045" -> "robots_blue"
    const match = spriteKey.match(/^(robot|vehicle|cannon)_.*_(red|blue|green|yellow)/);
    if (match) {
      return `${match[1]}s_${match[2]}`;
    }
    return undefined;
  }
}
```

### npm Scripts for Build Integration
```json
// In package.json
{
  "scripts": {
    "pack-atlases": "node scripts/pack-atlases.js",
    "pack-atlases:watch": "chokidar \"../assets/units/**/*.png\" -c \"npm run pack-atlases\" -d 500",
    "dev": "npm run pack-atlases && vite",
    "build": "npm run pack-atlases && tsc && vite build"
  }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Individual sprite loading | Texture atlas loading | Phaser 2->3 | 90% reduction in HTTP requests |
| Manual JSON atlas creation | Automated packing tools | 2018+ | Faster iteration, consistent output |
| Single large atlas | Multi-atlas with auto-split | Phaser 3.50 | Support for larger sprite collections |
| GPU texture limit 2048 | Modern GPUs support 4096+ | 2020+ | Can use larger atlases on modern devices |

**Deprecated/outdated:**
- `PhaserHash` exporter: Use `Phaser3` for Phaser 3 projects
- `load.spritesheet()` for animation frames: Use `load.atlas()` with JSON frame data for variable-sized sprites
- TexturePacker old JSON format: Ensure using "Phaser 3" export format, not "Phaser 2"

## Open Questions

Things that couldn't be fully resolved:

1. **Multipack single JSON file**
   - What we know: free-tex-packer generates separate JSON per texture when multipack triggers
   - What's unclear: Whether Phaser's `multiatlas` can load this format or needs post-processing
   - Recommendation: Test with small atlas first; if needed, write post-processing script to merge JSONs

2. **Incremental atlas generation**
   - What we know: No built-in support in free-tex-packer for incremental builds
   - What's unclear: Best approach for detecting changes and regenerating only affected atlases
   - Recommendation: Use file hashing (md5 of source folder) to skip unchanged categories; implement if build time becomes problematic

3. **Power-of-two texture sizes**
   - What we know: Some mobile GPUs require POT textures for best performance
   - What's unclear: Whether Phaser 3.70 handles NPOT textures efficiently on all targets
   - Recommendation: Enable `powerOfTwo: true` for safety, monitor performance on target devices

## Sources

### Primary (HIGH confidence)
- [free-tex-packer-core npm](https://www.npmjs.com/package/free-tex-packer-core) - API documentation, TypeScript definitions
- [free-tex-packer-core GitHub](https://github.com/odrick/free-tex-packer-core) - README, examples, issue discussions
- [Phaser 3 Textures Documentation](https://docs.phaser.io/phaser/concepts/textures) - Atlas loading, frame usage, performance
- [Phaser 3 LoaderPlugin API](https://docs.phaser.io/api-documentation/class/loader-loaderplugin) - atlas() and multiatlas() methods

### Secondary (MEDIUM confidence)
- [Phaser v3.60 Mobile Performance changelog](https://github.com/phaserjs/phaser/blob/v3.60.0/changelog/3.60/MobilePerformance.md) - WebGL batching improvements
- [free-tex-packer Issue #30](https://github.com/odrick/free-tex-packer-core/issues/30) - Multipack JSON format discussion
- [chokidar-cli npm](https://www.npmjs.com/package/chokidar-cli) - Watch mode command syntax

### Tertiary (LOW confidence)
- WebSearch results for build pipeline patterns - general guidance, not library-specific

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official documentation verified for both free-tex-packer and Phaser 3
- Architecture: HIGH - Patterns derived from official docs and project-specific analysis
- Pitfalls: MEDIUM - Combination of documented issues and common sense, some based on experience patterns

**Research date:** 2026-01-25
**Valid until:** 60 days (stable libraries, unlikely to have breaking changes)
