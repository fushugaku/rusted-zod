# Phase 7: Game Flow Screens - Context

## Phase Goal
Complete game flow from launch to gameplay with map selection, team choice, settings, and end screens matching the original C engine implementation.

## Source References

### Victory/Defeat Screens (zplayer.cpp, zserver.cpp)
- **Victory conditions:** Total domination (all zones), Zone majority (60%+ for time), Elimination (all enemies destroyed), Time limit
- **Game stats tracked:** Units killed, units lost, zones owned, buildings owned, game duration
- **Victory trigger:** GameStateSystem.onGameEnded callback with GameResult containing winner, condition, teamStats

### Map Selection (zmenu.cpp, zgame_menu.cpp)
- **Map listing:** Scan maps/ directory for .map files
- **Map preview:** Parse header to get map name, dimensions, terrain type, player count
- **Map info display:** Name, size (WxH tiles), terrain type (desert/arctic/volcanic/city/jungle), player count

### Team Selection (zteam.cpp, zgame_menu.cpp)
- **Available teams:** RED, BLUE, GREEN, YELLOW (plus PURPLE, TEAL, WHITE, BLACK in extended)
- **Team colors:** Defined in GameConfig.ts TEAM_COLORS
- **Selection UI:** Color swatch buttons, one per available team

### Settings (zoptions.cpp, zsound_engine.cpp)
- **Audio settings:** Sound volume (0-100), Music volume (0-100), Voice volume (0-100)
- **Game speed:** Normal (1.0x), Fast (1.5x), Faster (2.0x), Slow (0.5x)
- **Graphics options:** Show health bars, show unit names, minimap size

## Existing Implementation

### GameStateSystem.ts
- Already tracks: GameState (NOT_STARTED, PLAYING, PAUSED, ENDED), VictoryCondition types
- TeamStats interface: zonesOwned, unitsAlive, buildingsOwned, unitsKilled, unitsLost, isAlive
- GameResult interface: winner, condition, teamStats, gameDuration
- Callbacks: onGameStateChanged, onGameEnded, onTeamEliminated
- Methods: hasPlayerWon(), hasPlayerLost(), getTeamStats(), getGameDuration()

### Scene Architecture
- BootScene -> PreloaderScene -> GameScene (current flow)
- HUDScene runs as overlay on GameScene
- Scenes use Phaser scene management: scene.start(), scene.launch(), scene.stop()

### MapLoader.ts
- loadFromUrl(url) -> MapBasics
- parseMapBuffer(buffer) -> MapBasics
- getMapInfo(map) -> string summary
- MapBasics contains: width, height, mapName, playerCount, terrainType, zones, objects

### SoundSystem.ts
- setVolume(volume: number) method exists
- Uses Phaser sound manager internally
- Sound categories: weapon, explosion, voice, announcement, ambient, UI

## Success Criteria

1. **Victory/Defeat Screens**
   - Victory screen shows when all enemy units/buildings destroyed
   - Screen displays game stats (time, kills, losses, zones)
   - "Victory!" or "Defeat!" header with team color
   - Button to return to main menu

2. **Map Selection Screen**
   - Lists all .map files from maps/ directory
   - Shows map preview info (name, size, terrain, player count)
   - Terrain icon or color indicator
   - Click to select, double-click or button to start

3. **Team Selection UI**
   - Color swatches for available teams (RED, BLUE, GREEN, YELLOW)
   - Visual indicator of selected team
   - Integrated with map selection or separate screen

4. **Settings Screen**
   - Volume sliders for sound effects and announcements
   - Game speed selection (slider or buttons)
   - Graphics toggles (health bars, unit names)
   - Save/load settings from localStorage

## Scene Flow Design

```
                   +----------------+
                   | PreloaderScene |
                   +-------+--------+
                           |
                   +-------v--------+
                   |   MainMenu     |
                   | (new scene)    |
                   +-------+--------+
                           |
        +------------------+------------------+
        |                  |                  |
+-------v--------+ +-------v--------+ +-------v--------+
| MapSelection   | | TeamSelection  | |   Settings     |
| (new scene)    | | (new scene)    | | (new scene)    |
+-------+--------+ +----------------+ +----------------+
        |
+-------v--------+
|   GameScene    |
|  + HUDScene    |
+-------+--------+
        |
        | (game ends)
        |
+-------v--------+
| EndGame Screen |
| (victory/def)  |
+----------------+
        |
        v
   Return to MainMenu
```

## UI Patterns from Existing Code

### Container-based UI (from HUDScene, ProductionWindow)
```typescript
// Create container for positioning
const container = this.add.container(x, y);

// Background with rounded corners
const background = this.add.graphics();
background.fillStyle(0x1a1a2e, 0.9);
background.fillRoundedRect(0, 0, width, height, 4);
background.lineStyle(2, 0x3a3a5e, 1);
background.strokeRoundedRect(0, 0, width, height, 4);
container.add(background);

// Text styling
const titleText = this.add.text(x, y, "Title", {
  fontFamily: "Courier New",
  fontSize: "16px",
  color: "#ffffff",
  fontStyle: "bold",
});
```

### Interactive Elements (from ProductionWindow)
```typescript
// Clickable button with hover state
const button = this.add.graphics();
button.fillStyle(0x3a5a3a, 1);
button.fillRoundedRect(0, 0, buttonWidth, buttonHeight, 4);
button.setInteractive(
  new Phaser.Geom.Rectangle(0, 0, buttonWidth, buttonHeight),
  Phaser.Geom.Rectangle.Contains
);
button.on("pointerover", () => { /* hover effect */ });
button.on("pointerout", () => { /* normal state */ });
button.on("pointerdown", () => { /* click action */ });
```

### Color Constants
```typescript
// From GameConfig.ts
const TEAM_COLORS = {
  [TeamType.NULL]: 0x808080,
  [TeamType.RED]: 0xff0000,
  [TeamType.BLUE]: 0x0000ff,
  [TeamType.GREEN]: 0x00ff00,
  [TeamType.YELLOW]: 0xffff00,
};

// UI colors (from HUDScene)
const UI_COLORS = {
  background: 0x1a1a2e,
  border: 0x3a3a5e,
  text: "#ffffff",
  textMuted: "#888888",
  highlight: "#00ff66",
};
```

## Implementation Notes

### Scene Transition Pattern
```typescript
// Start new scene (replaces current)
this.scene.start("MapSelectionScene");

// Start scene with data
this.scene.start("GameScene", { mapPath: selectedMap, team: selectedTeam });

// Launch overlay scene (keeps current running)
this.scene.launch("HUDScene");

// Stop current scene
this.scene.stop();
```

### LocalStorage for Settings
```typescript
// Save
localStorage.setItem("zod_settings", JSON.stringify(settings));

// Load
const saved = localStorage.getItem("zod_settings");
const settings = saved ? JSON.parse(saved) : defaultSettings;
```

### Map Preview Generation
Since we don't have thumbnail images, show text-based preview:
- Map name from header
- Terrain type icon or colored square
- Dimensions: "128x96 tiles"
- Player count: "2-4 players"

## Dependencies
- Phase 6 complete (HUD patterns established)
- Existing GameStateSystem.ts infrastructure
- Existing MapLoader.ts for map parsing
- Existing SoundSystem.ts for volume control

## Risks
- LOW complexity - mostly UI work with existing patterns
- Map preview without images may feel sparse
- Settings persistence via localStorage (no server save)
