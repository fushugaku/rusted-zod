---
phase: 06-hud-enhancement
plan: 03
subsystem: ui-panels
tags: [hud, multi-selection, building-info, production-queue, phaser]

dependency-graph:
  requires: ["06-01"]
  provides: ["GroupInfoPanel", "BuildingInfoPanel", "panel-switching"]
  affects: ["06-04"]

tech-stack:
  added: []
  patterns:
    - "Panel switching based on selection type (single/multi/building)"
    - "Aggregate health display across multiple units"
    - "Production queue visualization with unit abbreviations"

key-files:
  created:
    - client/src/ui/GroupInfoPanel.ts
    - client/src/ui/BuildingInfoPanel.ts
  modified:
    - client/src/ui/index.ts
    - client/src/scenes/HUDScene.ts

decisions:
  - id: "panel-switching"
    choice: "Show different panels based on selection type"
    reason: "Single unit shows unit info, multi-selection shows group aggregates, buildings show production queue"
  - id: "unit-abbreviations"
    choice: "Two-letter codes for units in queue display"
    reason: "Compact display in queue slots (24x24 pixels)"
  - id: "zone-ownership-bar"
    choice: "Horizontal bar showing zone control percentage"
    reason: "Quick visual indicator of production speed bonus"

metrics:
  duration: "8 min"
  completed: "2026-01-25"
---

# Phase 6 Plan 03: Multi-selection and Building Info Panels Summary

Multi-selection group info panel and building info panel with production queue visualization.

## What Was Built

### GroupInfoPanel (Task 1)
Created `GroupInfoPanel.ts` that displays aggregate information for multi-selection:
- Title showing "GROUP SELECTED"
- Total unit count (e.g., "5 Units")
- Combined health bar with percentage across all selected units
- Unit type breakdown sorted by count descending (e.g., "3x Grunt", "2x Sniper")
- Overflow indicator for more than 5 unit types

Key implementation details:
- Uses `aggregateUnits()` method to count units by type from GameObjectState array
- Health bar color changes: green (>60%), yellow (30-60%), red (<30%)
- Max 5 rows displayed with "+N more..." for additional types

### BuildingInfoPanel (Task 2)
Created `BuildingInfoPanel.ts` for production buildings:
- Building name and level display
- Health bar with current/max values
- Zone ownership section with percentage bar
- Production queue visualization:
  - Progress bar with time remaining
  - Up to 5 queue slots with unit abbreviations
  - Current production shown first, queue items follow

Unit abbreviations for queue slots:
- Robots: GR, PS, SN, TO, PY, LA
- Vehicles: JP, LT, MD, HV, AP, ML, CR
- Cannons: GA, GU, HO, MI

### HUDScene Integration (Task 3)
Updated `HUDScene.ts` with:
- New panel properties (groupInfoPanel, buildingInfoPanel)
- Panel creation in `createUnitPanel()` method
- `updateSelection()` method for panel switching logic:
  - Empty selection: hide all panels
  - Single unit: show unit info panel + portrait
  - Single building: show building info panel
  - Multi-selection: show group info panel
- `updatePortraitOnly()` helper for multi-selection portrait display
- Placeholder methods for production and zone data access
- Resize handling for all new panels

## Technical Decisions

1. **Panel switching on selection type**: Rather than layering panels, we switch between them based on what's selected. This keeps the UI clean and focused.

2. **Two-letter abbreviations**: Queue slots are small (24x24), so two-letter codes (GR for Grunt, ML for Missile Launcher) provide clear identification.

3. **Zone ownership visualization**: Horizontal bar shows percentage fill with color coding (green = owned, yellow = partial, red = contested).

## Files Changed

| File | Changes |
|------|---------|
| `client/src/ui/GroupInfoPanel.ts` | New file - multi-selection aggregate display |
| `client/src/ui/BuildingInfoPanel.ts` | New file - building info with production queue |
| `client/src/ui/index.ts` | Added exports for new panels |
| `client/src/scenes/HUDScene.ts` | Panel integration and selection handling |

## Commits

| Hash | Description |
|------|-------------|
| f4e7dee | Task 1: Create GroupInfoPanel for multi-selection |
| 5c1124a | Task 2: Create BuildingInfoPanel for production buildings |
| 2b03cde | Task 3: Integrate panels with HUDScene |

## Deviations from Plan

None - plan executed exactly as written.

## Verification Results

1. TypeScript compilation: Passed (`npm run build` succeeds)
2. GroupInfoPanel exports correctly from barrel
3. BuildingInfoPanel exports correctly from barrel
4. HUDScene imports and creates all panels
5. Panel switching logic implemented with proper visibility toggling
6. Resize handling updates all panel positions

## Next Phase Readiness

Plan 06-03 is complete. The HUD now supports:
- Single unit selection with portrait and detailed stats
- Multi-selection with aggregate info
- Building selection with production queue

Ready for 06-04 (if it exists) or next phase.
