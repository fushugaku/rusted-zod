---
phase: 06-hud-enhancement
plan: 02
subsystem: ui
tags: [hud, unit-info, health-bar, grenades, driver-status]
depends_on:
  requires: ["06-01"]
  provides: ["unit-info-panel", "health-bar-segments", "grenade-display", "driver-health"]
  affects: ["06-03"]
tech-stack:
  added: []
  patterns: ["composite-graphics", "state-update-pattern"]
key-files:
  created:
    - client/src/ui/UnitInfoPanel.ts
  modified:
    - client/src/ui/index.ts
    - client/src/scenes/HUDScene.ts
decisions:
  - id: "grenade-icon-procedural"
    choice: "Procedural graphics for grenade icon"
    reason: "Matches RepairEffect/CraneEffect approach, avoids asset dependencies"
  - id: "health-bar-segments"
    choice: "Three segments: green (current), yellow (damaged), gray (max lost)"
    reason: "Matches C source zhud.cpp RenderHealth implementation"
  - id: "driver-health-blue"
    choice: "Blue color (0x00aaff) for driver health bar"
    reason: "Distinguishes from vehicle health (green), easy visual separation"
metrics:
  duration: "4 min"
  completed: "2026-01-25"
---

# Phase 6 Plan 02: Unit Info Panel with Stats Summary

Unit info panel displays comprehensive stats for selected units including health bars, grenade counts, driver status, and weapon information.

## What Was Built

### UnitInfoPanel Class (client/src/ui/UnitInfoPanel.ts)

Comprehensive stat display panel for all unit types:

**Health Bar System:**
- Three-segment bar matching C source (zhud.cpp)
- Green segment for current health
- Yellow segment for damaged but repairable
- Gray segment for max health lost
- Text display showing current/max values

**Robot-Specific Display:**
- Grenade icon with team-colored fill (procedural graphics)
- Grenade count text (padded to 2 digits)
- Weapon type text (Machine Gun, Sniper Rifle, etc.)

**Vehicle-Specific Display:**
- Vehicle health bar
- Driver health section (blue bar) when driver present
- Hides driver section when driverless
- Weapon/equipment text (Light Cannon, Transport, etc.)

**Cannon Display:**
- Health bar
- Weapon type text (Gatling Gun, Artillery, Howitzer, Missiles)

**Building Display:**
- Health bar
- Level number text

### HUDScene Integration

- UnitInfoPanel positioned left of main unit panel
- Updates automatically via updateSelectedUnit method
- Repositions correctly on window resize
- Shows/hides based on selection state

## Implementation Details

### Constants from C Source

```typescript
const PANEL_WIDTH = 100;
const PANEL_HEIGHT = 120;
const HEALTH_BAR_WIDTH = 74;  // max_dist from zhud.cpp
const HEALTH_BAR_HEIGHT = 8;
```

### Color Scheme

```typescript
const COLORS = {
  background: 0x1a1a2e,
  border: 0x3a3a5e,
  healthFull: 0x00ff00,    // Green
  healthLost: 0xffff00,    // Yellow
  healthEmpty: 0x333333,   // Gray
  driverHealth: 0x00aaff,  // Blue
};
```

### Weapon Info Mapping

| Unit Type | Weapon Text |
|-----------|-------------|
| Grunt | Machine Gun |
| Psycho | Machine Gun |
| Sniper | Sniper Rifle |
| Tough | Rocket Launcher |
| Pyro | Flamethrower |
| Laser | Laser Gun |
| Jeep | Machine Gun |
| Light/Medium/Heavy Tank | Light/Medium/Heavy Cannon |
| APC | Transport |
| Missile Launcher | Missiles |
| Crane | Repair Arm |
| Gatling | Gatling Gun |
| Gun | Artillery |
| Howitzer | Howitzer |
| Missile Cannon | Missiles |

## Commits

| Hash | Type | Description |
|------|------|-------------|
| 885b4c3 | feat | Create UnitInfoPanel class with health bars and stat display |
| b31faa6 | feat | Integrate UnitInfoPanel with HUDScene |

## Deviations from Plan

None - plan executed exactly as written.

## Verification Results

- TypeScript compilation: PASS
- Build: PASS
- UnitInfoPanel exports: updateFromState method available
- Health bar segments: green/yellow/gray rendering verified
- Grenade display: team-colored icon with count
- Driver health: conditional visibility based on hasDriver flag

## Next Phase Readiness

Ready for 06-03 (Group Info Panel) - UnitInfoPanel provides foundation for single-unit display, GroupInfoPanel will extend for multi-selection scenarios.
