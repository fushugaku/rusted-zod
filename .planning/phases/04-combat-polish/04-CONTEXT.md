# Phase 4: Combat Polish - Context

## Phase Goal

Complete combat mechanics with grenades, driver health, fort firing, and visual feedback. These are the remaining combat features needed for mechanical parity with the original C engine.

## Current State Analysis

### Already Implemented (CombatSystem.ts)
- Full combat stat tables for all robots, vehicles, cannons (lines 85-308)
- GRENADE_SETTINGS already defined (damage, radius, speed, attack_speed)
- MissileType.GRENADE exists in enum
- Snipe mechanics: `canSnipe`, `snipeChance` per unit type
- `onDriverSniped` callback exists but is NOT connected to driver health
- Area damage with falloff formula matching original
- Missile/projectile system with travel time

### Partially Implemented
- **Robot.ts**: `grenadeAmount`, `maxGrenades`, `throwGrenade()` method exists but:
  - No pickup system
  - No grenade box objects
  - No THROW animation integration with actual grenade projectiles

- **Vehicle.ts**: `hasDriver`, `driverType`, `removeDriver()` exist but:
  - No `driverHealth` separate from `vehicleHealth`
  - Snipe just calls `onDriverSniped` callback, doesn't track driver damage
  - Vehicle can have driver removed but no health tracking

### Not Implemented
- Grenade box map items (pickup objects)
- Fort firing mechanics (garrisoned units fire from fort)
- Repair visual effects (sparks/wrench animation during UNIT_REPAIR waypoint)

## Source Code Reference

### Grenades (source/ogrenades.cpp, source/zrobot.cpp)
```cpp
// ogrenades.cpp - Grenade box object
grenade_amount = zsettings->grenades_per_box;  // Amount in each box
radius = 40;  // Grenade damage radius
damage = zsettings->grenade_damage * MAX_UNIT_HEALTH;

// zrobot.cpp - Robot grenade handling
#define GRENADE_TIME_INT 0.15  // 150ms per throw animation frame
int grenade_amount;  // Robots track their grenade count
void DoPickupGrenadeAnim();  // Pickup animation (up/down based on direction)
bool CanPickupGrenades() { return grenade_amount <= 0; }  // Only if empty
bool CanHaveGrenades() { return true; }  // All robots can have grenades
bool CanThrowGrenades();  // Check if can throw
void FireMissile(int x_, int y_);  // Actually throws grenade
```

### Driver Health (source/zvehicle.cpp)
```cpp
// Vehicles track driver info separately
vector<driver_info_s> driver_info;
driver_type = GRUNT;  // Default driver type
AddDriver(zsettings->GetUnitSettings(ROBOT_OBJECT, GRUNT).health * MAX_UNIT_HEALTH);

// Snipe-able check requires:
// 1. Vehicle has `can_be_sniped` flag
// 2. Lid is open (for lidded vehicles)
// 3. Has driver(s)
bool ZVehicle::CanBeSniped() {
  if(has_lid)
    return can_be_sniped && lid_open && driver_info.size();
  else
    return can_be_sniped && driver_info.size();
}
```

### Fort Firing
Not explicitly in bfort.cpp - forts themselves don't fire. The mechanic is:
1. Robots inside fort (garrisoned) can attack
2. Fort provides cover/protection
3. Garrisoned units use their own weapons from inside

### Repair Visual Effects (source/brepair.cpp)
```cpp
// Repair building visual feedback
if(repairing_unit) {
  // Smoke stack animation when repairing
  lx = x + smoke_stack_x;
  ly = y + smoke_stack_y;
  the_map.RenderZSurface(&smoke_stack[smoke_stack_i], lx, ly);
}
```

The UNIT_REPAIR waypoint (crane repairing units in field) should show similar effects on the unit being repaired.

## Key Implementation Notes

1. **Grenade Max**: Original allows up to 99 grenades per robot (with adjustment check), but typical max from zsettings is 5 per box pickup.

2. **Driver Health**: Drivers are essentially "health pools" inside the vehicle. When sniped, one driver is removed. Vehicle becomes driverless when all drivers killed.

3. **Fort Garrison**: The ENTER_FORT waypoint already exists. Fort firing is about letting garrisoned units attack enemies within range while protected by fort.

4. **Repair Effects**: The EDeathSparks class shows particle effects. Repair should show similar sparks/wrench visuals at the repair location.

## Dependencies

- Phase 3 (Animation System) should be complete for grenade throw animation to work properly
- Existing WaypointSystem has ENTER_FORT, CRANE_REPAIR, UNIT_REPAIR modes
- CombatSystem has all the damage calculation infrastructure

## Technical Approach

1. **Grenade System**: Create GrenadeBox map item class, integrate with WaypointSystem for pickup, connect robot throw animation to actual grenade missile creation.

2. **Driver Health**: Add `driverHealth` to Vehicle state, update snipe logic to damage driver instead of calling callback, handle driver death.

3. **Fort Firing**: Extend Fort building class to track garrisoned units, delegate attack commands to garrisoned units, provide attack range from fort position.

4. **Repair Effects**: Create RepairEffect visual class with sparks/wrench animation, trigger from WaypointSystem when UNIT_REPAIR is active.
