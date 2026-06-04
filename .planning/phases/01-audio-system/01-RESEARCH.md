# Phase 1: Audio System - Research

**Researched:** 2026-01-25
**Domain:** Game audio (Phaser WebAudio, sound effects, voice announcements)
**Confidence:** HIGH

## Summary

This research investigates implementing the audio system for Z.O.D. by examining the original C engine sound architecture (`zsound_engine.cpp/h`) and mapping it to the existing TypeScript `SoundSystem.ts` skeleton. The original game has 261 WAV files covering weapon sounds, explosions, unit voices (ROB01-ROB75), computer announcements, and ambient effects.

The existing TypeScript codebase has a well-structured `SoundSystem.ts` (415 lines) with positional audio already implemented, but no audio assets are currently loaded. The C source reveals precise sound-to-event mappings, volume settings, and a rate-limiting system (`play_time_shift`) that prevents sound spam.

**Primary recommendation:** Extend `SoundSystem.ts` to load all original WAV files, map them to the exact sound enum from `zsound_engine.h`, and trigger sounds from combat/unit systems using the established patterns from the C source.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Phaser WebAudioSoundManager | 3.70 (built-in) | All game audio playback | Already in codebase, handles browser quirks, supports spatial audio |
| Individual WAV files | N/A | Sound effect source | Original assets exist; 261 WAV files in `/assets/sounds/` |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Audio Sprites (JSON) | Phaser format | Combine multiple sounds | Future optimization; not required for Phase 1 |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Individual WAVs | Audio Sprites | Sprites reduce HTTP requests but add build step complexity; defer to Phase 5 |
| Phaser Audio | Howler.js | Howler is redundant; Phaser's wrapper is sufficient |

**Installation:**
```bash
# No new dependencies - using existing Phaser 3.70
# Assets already exist at /assets/sounds/*.wav
```

## Architecture Patterns

### Recommended Project Structure
```
client/src/
  sound/
    SoundSystem.ts          # Extend existing (415 lines)
    SoundTypes.ts           # NEW: Enum matching zsound_engine.h
    ComputerVoice.ts        # NEW: Announcement queue system
  types/
    interfaces.ts           # Add SoundConfig interface
```

### Pattern 1: Sound Event Enum (from C source)
**What:** Enum of all sound events matching original engine
**When to use:** All sound playback calls
**Example:**
```typescript
// Source: zsound_engine.h lines 11-41
export enum SoundEvent {
  // Weapon sounds
  PSYCHO_FIRE = 'MACHGUN2',
  RIFLE_FIRE = 'RIFLE3',
  GUN_FIRE = 'LTGUN',
  GATLING_FIRE = 'GATTGUN',
  JEEP_FIRE = 'JEEPMGUN',
  LIGHT_FIRE = 'LTANKGUN',
  MEDIUM_FIRE = 'MTANKGUN',
  HEAVY_FIRE = 'HTANKGUN',
  MOMISSILE_FIRE = 'MOBIMIS2',
  TOUGH_FIRE = 'MOBIMISS',
  PYRO_FIRE = 'FLAMER',
  LASER_FIRE = 'LASERGUN',

  // Explosions (random selection from 5)
  EXPLOSION_00 = 'explosion_00',
  EXPLOSION_01 = 'explosion_01',
  EXPLOSION_02 = 'explosion_02',
  EXPLOSION_03 = 'explosion_03',
  EXPLOSION_04 = 'explosion_04',
  TURRENT_EXPLOSION = 'METGRND',

  // Computer voice announcements
  COMP_VEHICLE = 'comp_vehicle_manufactured',
  COMP_ROBOT = 'comp_robot_manufactured',
  COMP_GUN = 'comp_gun_manufactured',
  COMP_STARTING_MANUFACTURE = 'comp_starting_manufacture',
  COMP_MANUFACTURING_CANCELED = 'comp_manufacturing_canceled',
  COMP_STARTING_REPAIR = 'comp_starting_repair',
  COMP_VEHICLE_REPAIRED = 'comp_vehicle_repaired',
  COMP_TERRITORY_LOST = 'comp_territory_lost',
  COMP_RADAR_ACTIVATED = 'comp_radar_activated',
  COMP_FORT_UNDER_ATTACK = 'comp_fort_under_attack',
  // COMP_YOUR_LOSING_00 through _09 (10 variants)

  // Unit voice responses (ROB01-ROB75)
  YES_SIR1 = 'ROB01',
  YES_SIR2 = 'ROB02',
  YES_SIR3 = 'ROB03',
  UNIT_REPORTING1 = 'ROB04',
  UNIT_REPORTING2 = 'ROB05',
  UNIT_REPORTING3 = 'ROB06',
  GRUNTS_REPORTING = 'ROB07',
  PSYCHOS_REPORTING = 'ROB08',
  SNIPERS_REPORTING = 'ROB09',
  TOUGHS_REPORTING = 'ROB10',
  LASERS_REPORTING = 'ROB11',
  PYROS_REPORTING = 'ROB12',
  // ... continues through ROB75

  // Ambient sounds
  BAT_CHIRP = 'BATCHIRP',
  CROW = 'CROW2',
  RICOCHET = 'RICOCH1',
  THROW_GRENADE = 'GRENLOBX',

  // Building/factory sounds
  RADAR = 'radar_sound',
  ROBOT_FACTORY = 'ROBFACT5',
  VEHICLE_FACTORY = 'ROBFACT5', // Same as robot factory in original
}
```

### Pattern 2: Volume and Rate Limiting (from C source)
**What:** Per-sound volume and cooldown settings
**When to use:** Configuring sound playback
**Example:**
```typescript
// Source: zsound_engine.cpp lines 126-172
interface SoundConfig {
  filename: string;
  baseVolume: number;      // 0-100, default 40
  volumeShift: number;     // Random variation, default 20
  playTimeShift: number;   // Cooldown in seconds, default 0
}

const SOUND_CONFIG: Record<SoundEvent, SoundConfig> = {
  // Weapon sounds with custom volumes
  [SoundEvent.PSYCHO_FIRE]: { filename: 'MACHGUN2', baseVolume: 40, volumeShift: 20, playTimeShift: 0 },
  [SoundEvent.JEEP_FIRE]: { filename: 'JEEPMGUN', baseVolume: 40, volumeShift: 20, playTimeShift: 0.15 },
  [SoundEvent.RIFLE_FIRE]: { filename: 'RIFLE3', baseVolume: 10, volumeShift: 10, playTimeShift: 0 },
  [SoundEvent.PYRO_FIRE]: { filename: 'FLAMER', baseVolume: 20, volumeShift: 10, playTimeShift: 0 },
  [SoundEvent.LASER_FIRE]: { filename: 'LASERGUN', baseVolume: 20, volumeShift: 10, playTimeShift: 0 },
  [SoundEvent.LIGHT_FIRE]: { filename: 'LTANKGUN', baseVolume: 25, volumeShift: 20, playTimeShift: 0 },
  [SoundEvent.MEDIUM_FIRE]: { filename: 'MTANKGUN', baseVolume: 25, volumeShift: 20, playTimeShift: 0 },
  [SoundEvent.HEAVY_FIRE]: { filename: 'HTANKGUN', baseVolume: 25, volumeShift: 20, playTimeShift: 0 },

  // Explosions with lower base volume
  [SoundEvent.EXPLOSION_00]: { filename: 'explosion_00', baseVolume: 30, volumeShift: 20, playTimeShift: 0 },

  // Radar has low volume
  [SoundEvent.RADAR]: { filename: 'radar_sound', baseVolume: 20, volumeShift: 20, playTimeShift: 0 },
  [SoundEvent.ROBOT_FACTORY]: { filename: 'ROBFACT5', baseVolume: 5, volumeShift: 20, playTimeShift: 0 },
};
```

### Pattern 3: PlayWavRestricted (Positional Audio)
**What:** Only play sound if within camera view
**When to use:** All combat/unit sounds
**Example:**
```typescript
// Source: zsound_engine.cpp lines 284-293
public playPositional(
  sound: SoundEvent,
  x: number,
  y: number,
  width: number = 0,
  height: number = 0
): void {
  // Already implemented in SoundSystem.ts lines 136-153
  // Uses Phaser.Math.Distance.Between for falloff
  if (!this.isWithinCameraView(x, y, width, height)) return;
  this.play(sound, { x, y });
}
```

### Pattern 4: Random Explosion Selection
**What:** Pick random explosion sound from pool
**When to use:** Explosion effects
**Example:**
```typescript
// Source: zsound_engine.cpp lines 343-348
public playRandomExplosion(x: number, y: number, isLight: boolean = false): void {
  const maxIndex = isLight ? 2 : 5;  // Light uses first 2, full uses all 5
  const index = Math.floor(Math.random() * maxIndex);
  this.playPositional(`explosion_0${index}`, x, y);
}
```

### Anti-Patterns to Avoid
- **Playing every sound:** Original uses `play_time_shift` cooldown - respect it
- **Ignoring camera bounds:** Always use `playPositional` for world sounds
- **Loading all sounds upfront:** Load in PreloaderScene with progress tracking
- **Hardcoding filenames:** Use enum mapping like original `switch(snd)` pattern

## Sound Event Mapping from C Source

### Weapon Sounds (from specific unit files)

| Unit | C Source File | Sound Event | WAV File |
|------|---------------|-------------|----------|
| Grunt | rgrunt.cpp:175 | RIFLE_FIRE | RIFLE3.wav |
| Psycho | rpsycho.cpp:171 | PSYCHO_FIRE | MACHGUN2.wav |
| Sniper | rsniper.cpp:174 | RIFLE_FIRE | RIFLE3.wav |
| Tough | rtough.cpp:191 | TOUGH_FIRE | MOBIMISS.wav |
| Pyro | rpyro.cpp:217 | PYRO_FIRE | FLAMER.wav |
| Laser | rlaser.cpp:217 | LASER_FIRE | LASERGUN.wav |
| Jeep | vjeep.cpp:264 | JEEP_FIRE | JEEPMGUN.wav |
| Light Tank | vlight.cpp:240 | LIGHT_FIRE | LTANKGUN.wav |
| Medium Tank | vmedium.cpp:234 | MEDIUM_FIRE | MTANKGUN.wav |
| Heavy Tank | vheavy.cpp:223 | HEAVY_FIRE | HTANKGUN.wav |
| Missile Launcher | vmissilelauncher.cpp:201 | MOMISSILE_FIRE | MOBIMIS2.wav |
| Gatling | cgatling.cpp:223 | GATLING_FIRE | GATTGUN.wav |
| Gun | cgun.cpp:205 | GUN_FIRE | LTGUN.wav |
| Howitzer | chowitzer.cpp:222 | HEAVY_FIRE | HTANKGUN.wav |
| Missile Cannon | cmissilecannon.cpp:226 | MOMISSILE_FIRE | MOBIMIS2.wav |

### Computer Voice Announcements (from zplayer_events.cpp)

| Event | C Source Line | Sound | Trigger |
|-------|---------------|-------|---------|
| Vehicle manufactured | 994 | comp_vehicle_manufactured | Factory completion |
| Robot manufactured | 999 | comp_robot_manufactured | Factory completion |
| Gun manufactured | 1004 | comp_gun_manufactured | Factory completion |
| Manufacturing started | 1009 | comp_starting_manufacture | Production queued |
| Manufacturing canceled | 1012 | comp_manufacturing_canceled | Production canceled |
| Territory lost | 1015 | comp_territory_lost | Zone captured by enemy |
| Radar activated | 1018 | comp_radar_activated | Radar comes online |
| Starting repair | 1078 | comp_starting_repair | Crane begins repair |
| Vehicle repaired | 1081 | comp_vehicle_repaired | Repair complete |
| Fort under attack | zplayer.cpp:1090 | comp_fort_under_attack | Fort taking damage |
| You're losing | zplayer.cpp:1138 | comp_youre_losing_0X | Score disadvantage |

### Unit Voice Responses (from zportrait.cpp)

| Action | Sound Group | WAV Files |
|--------|-------------|-----------|
| Selection acknowledgment | YES_SIR | ROB01, ROB02, ROB03 (random) |
| Unit reporting (generic) | UNIT_REPORTING | ROB04, ROB05, ROB06 |
| Grunts reporting | GRUNTS_REPORTING | ROB07 |
| Psychos reporting | PSYCHOS_REPORTING | ROB08 |
| Snipers reporting | SNIPERS_REPORTING | ROB09 |
| Toughs reporting | TOUGHS_REPORTING | ROB10 |
| Lasers reporting | LASERS_REPORTING | ROB11 |
| Pyros reporting | PYROS_REPORTING | ROB12 |
| Movement commands | Various | ROB13-ROB24 |
| Under attack | UNDER_ATTACK | ROB25-ROB29 |
| Target destroyed | SUCCESS | ROB37-ROB46 |
| Victory | WIN | ROB61-ROB66 |
| Defeat | LOSE | ROB67-ROB73 |

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Sound cooldown | Custom timer per sound | Copy ZSound.next_play_time pattern | Original handles random jitter |
| Positional audio | Complex 3D audio | SoundSystem.ts already has falloff | Lines 134-153 are correct |
| Volume randomization | Fixed volumes | base_volume + rand() % volume_shift | Creates natural variation |
| Audio context unlock | Manual user interaction | SoundSystem.resumeContext() | Already implemented correctly |

**Key insight:** The original C engine has a well-designed sound system with rate limiting, volume variation, and positional audio. Copy these patterns exactly rather than simplifying.

## Common Pitfalls

### Pitfall 1: Sound Spam
**What goes wrong:** Multiple units firing creates cacophony of overlapping sounds
**Why it happens:** No cooldown between same sound type
**How to avoid:** Implement `play_time_shift` from original - Jeep fire has 0.15s cooldown
**Warning signs:** Audio distortion, clicking, overwhelming volume

### Pitfall 2: Web Audio Context Suspended
**What goes wrong:** No sounds play until user interaction
**Why it happens:** Browser autoplay policy
**How to avoid:** Call `SoundSystem.resumeContext()` on first click (already implemented)
**Warning signs:** Sounds work in dev but not fresh page load

### Pitfall 3: Loading All Sounds Synchronously
**What goes wrong:** Long loading time, no progress feedback
**Why it happens:** 261 WAV files loaded one-by-one
**How to avoid:** Use Phaser's load queue with progress callback
**Warning signs:** Loading bar stuck, browser tab unresponsive

### Pitfall 4: Missing Positional Audio Bounds
**What goes wrong:** Sounds from far-away units still play at full volume
**Why it happens:** Not using PlayWavRestricted pattern
**How to avoid:** All combat sounds use positional audio with camera check
**Warning signs:** Hearing explosions from offscreen battles

## Code Examples

### Loading Sounds in PreloaderScene
```typescript
// Source: Extend existing loadSoundAssets() in PreloaderScene.ts
private loadSoundAssets(): void {
  // Weapon sounds
  this.load.audio('RIFLE3', 'assets/sounds/RIFLE3.wav');
  this.load.audio('MACHGUN2', 'assets/sounds/MACHGUN2.wav');
  this.load.audio('GATTGUN', 'assets/sounds/GATTGUN.wav');
  this.load.audio('JEEPMGUN', 'assets/sounds/JEEPMGUN.wav');
  this.load.audio('LTANKGUN', 'assets/sounds/LTANKGUN.wav');
  this.load.audio('MTANKGUN', 'assets/sounds/MTANKGUN.wav');
  this.load.audio('HTANKGUN', 'assets/sounds/HTANKGUN.wav');
  this.load.audio('MOBIMIS2', 'assets/sounds/MOBIMIS2.wav');
  this.load.audio('MOBIMISS', 'assets/sounds/MOBIMISS.wav');
  this.load.audio('FLAMER', 'assets/sounds/FLAMER.wav');
  this.load.audio('LASERGUN', 'assets/sounds/LASERGUN.wav');
  this.load.audio('LTGUN', 'assets/sounds/LTGUN.wav');

  // Explosions (5 variants)
  for (let i = 0; i < 5; i++) {
    this.load.audio(`explosion_0${i}`, `assets/sounds/explosion_0${i}.wav`);
  }
  this.load.audio('METGRND', 'assets/sounds/METGRND.wav');

  // Computer voice announcements
  this.load.audio('comp_vehicle_manufactured', 'assets/sounds/comp_vehicle_manufactured.wav');
  this.load.audio('comp_robot_manufactured', 'assets/sounds/comp_robot_manufactured.wav');
  this.load.audio('comp_gun_manufactured', 'assets/sounds/comp_gun_manufactured.wav');
  this.load.audio('comp_starting_manufacture', 'assets/sounds/comp_starting_manufacture.wav');
  this.load.audio('comp_manufacturing_canceled', 'assets/sounds/comp_manufacturing_canceled.wav');
  this.load.audio('comp_starting_repair', 'assets/sounds/comp_starting_repair.wav');
  this.load.audio('comp_vehicle_repaired', 'assets/sounds/comp_vehicle_repaired.wav');
  this.load.audio('comp_territory_lost', 'assets/sounds/comp_territory_lost.wav');
  this.load.audio('comp_radar_activated', 'assets/sounds/comp_radar_activated.wav');
  this.load.audio('comp_fort_under_attack', 'assets/sounds/comp_fort_under_attack.wav');

  // "You're losing" variants (10)
  for (let i = 0; i < 10; i++) {
    const padded = i.toString().padStart(2, '0');
    this.load.audio(`comp_youre_losing_${padded}`, `assets/sounds/comp_youre_losing_${padded}.wav`);
  }

  // Unit voice responses (ROB01-ROB75)
  for (let i = 1; i <= 75; i++) {
    const padded = i.toString().padStart(2, '0');
    this.load.audio(`ROB${padded}`, `assets/sounds/ROB${padded}.wav`);
  }

  // Misc sounds
  this.load.audio('RICOCH1', 'assets/sounds/RICOCH1.wav');
  this.load.audio('GRENLOBX', 'assets/sounds/GRENLOBX.wav');
  this.load.audio('BATCHIRP', 'assets/sounds/BATCHIRP.wav');
  this.load.audio('CROW2', 'assets/sounds/CROW2.wav');
  this.load.audio('radar_sound', 'assets/sounds/radar_sound.wav');
  this.load.audio('ROBFACT5', 'assets/sounds/ROBFACT5.wav');
}
```

### Integration with CombatSystem
```typescript
// In CombatSystem.ts, extend onAttackEffect callback
combatSystem.setOnAttackEffect((attackerRefId, targetRefId, missileType) => {
  const attacker = getUnitInfo(attackerRefId);
  if (!attacker) return;

  // Map unit type to sound
  const sound = getSoundForUnit(attacker.objectType, attacker.objectId);
  soundSystem.playPositional(sound, attacker.x, attacker.y);
});

function getSoundForUnit(objectType: ObjectType, objectId: number): string {
  if (objectType === ObjectType.OBJECT_ROBOT) {
    switch (objectId as RobotType) {
      case RobotType.GRUNT:
      case RobotType.SNIPER: return 'RIFLE3';
      case RobotType.PSYCHO: return 'MACHGUN2';
      case RobotType.TOUGH: return 'MOBIMISS';
      case RobotType.PYRO: return 'FLAMER';
      case RobotType.LASER: return 'LASERGUN';
    }
  }
  // ... vehicles and cannons
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| HTML5 Audio | Web Audio API | 2018+ | Web Audio is standard for games |
| Individual audio files | Audio Sprites | Ongoing | Reduces HTTP requests; optional for Phase 1 |

**Deprecated/outdated:**
- HTML5 Audio fallback: Not needed; all modern browsers support Web Audio
- Flash audio: Extinct

## Open Questions

1. **Music streaming vs preload**
   - What we know: 3 OGG music files exist (desert, volcanic, jungle)
   - What's unclear: Should music stream or preload?
   - Recommendation: Use `this.load.audio()` for preload; music files are small (~1-3MB each)

2. **UI click sounds**
   - What we know: Original has CLICK1L.wav, BEEP*.wav files
   - What's unclear: Which specific sounds for which UI actions?
   - Recommendation: Use CLICK1L for button clicks, examine original menus if needed

3. **Selected_grunt.wav etc. vs ROB07 etc.**
   - What we know: Both selected_*.wav AND ROB07-12 reporting sounds exist
   - What's unclear: When to use which?
   - Recommendation: Use selected_* for initial selection, ROB* for subsequent selection (match C source)

## Sources

### Primary (HIGH confidence)
- `/source/zsound_engine.cpp` - Sound loading and playback implementation
- `/source/zsound_engine.h` - Sound enum definitions (lines 11-41)
- `/source/zportrait.cpp` - Unit voice response triggers (lines 450-535)
- `/source/zplayer_events.cpp` - Computer announcement triggers (lines 992-1082)
- `/source/rgrunt.cpp`, `/source/vjeep.cpp`, etc. - Unit-specific sound calls

### Secondary (MEDIUM confidence)
- `.planning/research/STACK.md` - Audio architecture recommendations (verified)
- `/client/src/sound/SoundSystem.ts` - Existing implementation (415 lines)

### Tertiary (LOW confidence)
- None - all findings verified against C source

## Metadata

**Confidence breakdown:**
- Sound event mapping: HIGH - directly from C source code
- Volume/timing settings: HIGH - exact values from zsound_engine.cpp
- Architecture patterns: HIGH - matches existing SoundSystem.ts structure
- Integration points: MEDIUM - requires CombatSystem callback wiring

**Research date:** 2026-01-25
**Valid until:** Indefinite (original C source is stable reference)

---

## Quick Reference: File to Sound Mapping

### Essential Weapon Sounds (12 files)
```
RIFLE3.wav      - Grunt, Sniper
MACHGUN2.wav    - Psycho
GATTGUN.wav     - Gatling cannon
JEEPMGUN.wav    - Jeep
LTANKGUN.wav    - Light tank
MTANKGUN.wav    - Medium tank
HTANKGUN.wav    - Heavy tank, Howitzer
MOBIMIS2.wav    - Missile Launcher, Missile Cannon
MOBIMISS.wav    - Tough robot
FLAMER.wav      - Pyro
LASERGUN.wav    - Laser robot
LTGUN.wav       - Gun cannon
```

### Essential Explosion Sounds (6 files)
```
explosion_00.wav through explosion_04.wav
METGRND.wav     - Turret destruction
```

### Essential Computer Voice (10 files)
```
comp_vehicle_manufactured.wav
comp_robot_manufactured.wav
comp_gun_manufactured.wav
comp_starting_manufacture.wav
comp_manufacturing_canceled.wav
comp_territory_lost.wav
comp_fort_under_attack.wav
comp_starting_repair.wav
comp_vehicle_repaired.wav
comp_radar_activated.wav
```

### Unit Voice (Priority files for MVP)
```
ROB01-03.wav    - "Yes sir" acknowledgments
ROB04-06.wav    - "Unit reporting"
ROB07-12.wav    - Unit-type-specific reporting
ROB13-24.wav    - Movement acknowledgments
```

**Total essential sounds for Phase 1 MVP:** ~40 files
