---
phase: 05-production-polish
plan: 02
subsystem: ui
tags: [rally-point, phaser, visualization, production-system, flag-marker]

# Dependency graph
requires:
  - phase: 04-production
    provides: ProductionSystem with rally point storage
provides:
  - RallyPointVisualizer component with flag and dashed line
  - Rally button in ProductionWindow UI
  - GameScene integration for rally point setting and display
affects: [unit-production, spawn-system, ai-behavior]

# Tech tracking
tech-stack:
  added: []
  patterns: [event-driven UI updates, cursor state management]

key-files:
  created:
    - client/src/ui/RallyPointVisualizer.ts
  modified:
    - client/src/ui/ProductionWindow.ts
    - client/src/ui/index.ts
    - client/src/scenes/GameScene.ts
    - client/src/scenes/HUDScene.ts

key-decisions:
  - "Rally button toggles mode, left/right click to set position"
  - "Dashed line connects building to rally flag for visual clarity"
  - "Crosshair cursor indicates rally mode is active"

patterns-established:
  - "Event-driven pattern: HUD emits events, GameScene handles them"
  - "Visualizer pattern: separate class for world-space visualization"

# Metrics
duration: 18min
completed: 2026-01-25
---

# Phase 5 Plan 2: Rally Point Visualization Summary

**Rally point UI with flag marker visualization, dashed connection line, and click-to-set interaction in production buildings**

## Performance

- **Duration:** 18 min
- **Started:** 2026-01-25
- **Completed:** 2026-01-25
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Created RallyPointVisualizer with team-colored flag marker and dashed line
- Added "Set Rally" button to ProductionWindow with visual mode toggle
- Integrated rally point display and setting in GameScene
- Rally point shows when production building is selected
- Click anywhere on map (in rally mode) to set new rally position

## Task Commits

Each task was committed atomically:

1. **Task 1: Create RallyPointVisualizer component** - `de37746` (feat)
2. **Task 2: Add rally button to ProductionWindow** - `3744502` (feat)
3. **Task 3: Wire rally point system in GameScene** - `dbcc6d2` (feat)
4. **Fix: Restore ProductionModifiers import** - `3cff927` (fix)

## Files Created/Modified
- `client/src/ui/RallyPointVisualizer.ts` - Flag marker with dashed line visualization
- `client/src/ui/ProductionWindow.ts` - Rally button and mode toggle
- `client/src/ui/index.ts` - Export RallyPointVisualizer
- `client/src/scenes/GameScene.ts` - Rally point integration and event handling
- `client/src/scenes/HUDScene.ts` - Rally mode event wiring

## Decisions Made
- Rally button text changes to "Click Map..." when in rally mode for clear user feedback
- Both left and right click can set rally point (consistent with RTS conventions)
- Cursor changes to crosshair during rally mode for visual indication
- Rally mode auto-cancels when production window closes
- Dashed line uses 8px dash / 4px gap for visibility without distraction

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Restored ProductionModifiers import**
- **Found during:** Final verification
- **Issue:** Import was removed but method using type still existed from another feature
- **Fix:** Re-added ProductionModifiers to import statement
- **Files modified:** client/src/ui/ProductionWindow.ts
- **Verification:** Build passes
- **Committed in:** 3cff927

---

**Total deviations:** 1 auto-fixed (1 bug fix)
**Impact on plan:** Minor import fix, no scope creep.

## Issues Encountered
- External file modifications during execution caused brief build failures; resolved by re-reading and adapting to changes

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Rally point visualization complete and functional
- Ready for testing with actual unit production
- Future: Consider adding rally point drag-to-reposition

---
*Phase: 05-production-polish*
*Completed: 2026-01-25*
