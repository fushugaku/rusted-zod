---
phase: 07-game-flow-screens
plan: 01
subsystem: ui
tags: [scenes, victory, defeat, stats, game-flow]
dependency-graph:
  requires: [06-hud-enhancement]
  provides: [end-game-screens, game-result-display]
  affects: [07-02-main-menu]
tech-stack:
  added: []
  patterns: [scene-transition, stats-display, interactive-buttons]
key-files:
  created:
    - client/src/scenes/VictoryScene.ts
    - client/src/scenes/DefeatScene.ts
  modified:
    - client/src/scenes/index.ts
    - client/src/scenes/GameScene.ts
    - client/src/main.ts
decisions:
  - id: victory-defeat-styling
    choice: Green theme for victory, red theme for defeat
    rationale: Clear visual distinction between win/loss states
  - id: stats-panel-layout
    choice: Centered panel with label-value pairs
    rationale: Clean, readable layout for game statistics
  - id: test-shortcuts
    choice: Shift+V and Shift+D for testing end screens
    rationale: Easy development testing without playing full game
metrics:
  duration: 8 min
  completed: 2026-01-25
---

# Phase 7 Plan 01: Victory and Defeat Screens with Stats Summary

Victory/defeat end screens with game statistics display using Phaser scene transitions.

## Summary

Implemented VictoryScene and DefeatScene that display when the game ends, showing comprehensive game statistics and providing navigation back to restart the game.

## Completed Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create VictoryScene with stats display | fd1d5c6 | VictoryScene.ts |
| 2 | Create DefeatScene with stats display | 16b4960 | DefeatScene.ts |
| 3 | Wire end screens to GameScene and register | b403fd3 | index.ts, GameScene.ts, main.ts |

## Implementation Details

### VictoryScene
- Large "VICTORY!" header in team color (green for player victory)
- Victory condition subtitle (Total Domination, Zone Majority, Elimination, Time Limit)
- Stats panel showing:
  - Game Duration (formatted as M:SS)
  - Units Killed
  - Units Lost
  - Zones Controlled
  - Buildings Owned
- Interactive "MAIN MENU" button with hover effects
- Green color theme (#00ff66 accents)

### DefeatScene
- Large "DEFEAT" header in red (#ff3333)
- Winner team announcement
- Player stats panel showing same statistics
- Interactive "MAIN MENU" button with hover effects
- Red color theme (#ff6666 accents)

### GameScene Integration
- Added `handleGameEnded(result: GameResult)` method
- Scene stops HUDScene before transitioning
- Determines winner vs loser to show correct screen
- Debug shortcuts for testing:
  - Shift+V: Show victory screen with test data
  - Shift+D: Show defeat screen with test data

## Verification

- [x] TypeScript compilation passes
- [x] VictoryScene exported and registered
- [x] DefeatScene exported and registered
- [x] Scenes added to main.ts scene array
- [x] handleGameEnded method transitions to correct scene
- [x] Test shortcuts trigger screens with sample data

## Deviations from Plan

None - plan executed exactly as written.

## Next Phase Readiness

Ready for 07-02 (Main Menu Scene) which will:
- Replace the temporary "restart game" behavior with proper menu navigation
- Add map selection before game start
- Provide full game loop: Menu -> Game -> Victory/Defeat -> Menu
