# Что уже вынесено в рамках переноса

## Unit UI layout

- Для роботов, машин, пушек, зданий и items выделены `*_ui.rs` модули в `src/units/**`.
- В них перенесены asset paths, atlas frame names, selection/default sizes, HUD names, damage/death visuals, projectile visuals, vehicle track visuals, factory overlays, bridge/fort geometry.
- `src/units/unit_ui.rs` стал общим facade для визуального отображения и object-to-map conversion.

## Unit stats/settings

- `UnitSettings` и unit stats перенесены в `src/units/unit_stats.rs`.
- Per-unit `.rs` файлы владеют настройками здоровья, скорости, радиуса, урона, cooldown и snipe chance.
- `src/original/settings.rs` оставлен как compatibility shim, но runtime imports оттуда убраны.

## Combat and behavior

- Attack delivery, snipe policy, grenade damage, driver attack multipliers и attack sounds вынесены в `src/units/attack.rs`.
- Driver/eject/enter policy вынесены в `src/units/unit_driver.rs` и `src/units/unit_enter.rs`.
- Passive engage, passive agro, auto-enter, grenade pickup и enter-fort target choice перенесены в `src/units/unit_behavior.rs`.
- Fort elimination, capture and movement multipliers также живут в `unit_behavior`.

## Buildings/items/vehicles visual policy

- Fort entrance/entry points, fort turret slots и fort turret map tile policy перенесены в `src/units/buildings/building_ui.rs`.
- Fallback collision geometry перенесена из placement в `src/units/unit_ui.rs` с делегированием в building/cannon/item UI.
- Vehicle track points, road-neighbor checks и track roll gate перенесены в `src/units/vehicles/vehicle_ui.rs`.
- Vehicle lid/driver visual asset policy and tank offsets перенесены в `src/units/vehicles/vehicle_ui.rs` и per-tank `light_ui.rs`, `medium_ui.rs`, `heavy_ui.rs`.
- Atlas runtime теперь грузит source-backed specs из `src/units/robots/*_ui.rs`, `src/units/vehicles/*_ui.rs`, `src/units/cannons/*_ui.rs` и `src/units/buildings/*_ui.rs`, вместо локальных helper-имен в `render/atlas.rs`.

## Unit file layout

- Concrete units теперь лежат в формате `src/units/[type]/[unit_name]/[unit_name]_logic.rs`, `[unit_name]_ui.rs`, `[unit_name]_mod.rs`.
- Перенесены группы `robots`, `vehicles`, `cannons`, `buildings`, `items`.
- Family-level shared modules оставлены на месте: `robot_ui.rs`, `robot_state.rs`, `vehicle_ui.rs`, `building_ui.rs`, `cannon_ui.rs`, `item_ui.rs`.
- Family `mod.rs` сохраняют compatibility facade для старых call sites (`robots::tough_ui`, `vehicles::light_ui`, `buildings::radar_ui`, etc.), чтобы behavioral wiring не смешивался с механическим layout-срезом.

## Dead code cleanup

- 2026-06-05: `cargo check -q` доведен до чистого состояния без rustc warnings.
- Runtime-worthy visual helpers были подключены в atlas wiring: robot stand/mobile specs, vehicle spawn/mobile specs, building overlay specs, cannon captured frame profile.
- Test-only facades в `main.rs`, `render/atlas.rs`, `src/units/buildings`, `src/units/cannons`, `src/units/items`, `src/units/robots`, `src/units/vehicles` закрыты `#[cfg(test)]` или удалены.
- `UnitSettings::max_health` удален; compatibility tests используют `object_max_health`.
- Проверка среза: `cargo check -q`, `cargo test -q`, `./scripts/build-wasm.sh`, browser smoke на `http://127.0.0.1:4173/` прошли.

## Still risky / needs source-backed audit

- `main.rs` все еще содержит много orchestration, effects and gameplay loops; их надо делить по source-backed slices.
- `placement.rs`, `production.rs`, `production_ui.rs`, `pathing.rs` еще содержат unit-specific policy.
- Остались не dead-code, а source-backed behavior gaps: robot attack/grenade state machines, vehicle lid/turrent/death wiring details, production/building factory behavior and effects families.

## Completed slice: robot grenade throw animation

- Source: `source/zrobot.cpp` `ZRobot::FireMissile`, `ZRobot::Common_Process`, `ZRobot::CanThrowGrenades`; `source/rgrunt.cpp` `RGrunt::DoRender`.
- Rust owner: `src/units/robots/robot_state.rs` for timing/state, `src/units/robots/robot_ui.rs` for `throw_something` atlas frame names.
- Runtime call site: `src/main.rs::process_attack_targets` starts the animation when a grenade delivery is fired; a Bevy animation system advances/removes it.
- Known difference for this slice: Rust still has no full `R_ATTACKING` mode and does not yet hold throw frame 0 during the whole grenade-ready attack state; this slice covers the actual fired grenade animation.
- Done: `RobotGrenadeThrowAnimation` is inserted on fired grenade delivery, uses `robot_throw_*` atlas frames, advances every `0.15` seconds for 4 frames, then restores normal mobile frame.
- Verification: `cargo check -q`, `cargo test -q` (383 passed), `./scripts/build-wasm.sh`, browser smoke (`boot=started`, canvas non-black) passed.

## Completed slice: group-leader grenade ownership

- Source: `source/zobject.cpp` `ZObject::ProcessAttackDamage`, `ZObject::HasExplosives`, `ZObject::CanAttackObject`; `source/zrobot.cpp` `ZRobot::CanThrowGrenades`.
- Rust owner: `src/units/attack.rs` owns `GrenadeAttackSource` and own-first/group-leader fallback; `src/units/unit_behavior.rs` accepts effective grenade availability for passive target choice.
- Runtime call sites: `src/main.rs::process_passive_engage`, `src/main.rs::process_attack_targets`, and `src/cursor.rs` cursor explosive affordance.
- Done: grenade attacks now use own grenades first, then the group leader's grenade inventory; `process_attack_targets` uses a per-tick grenade ledger so multiple minions cannot overspend one leader grenade; passive attack/agro and cursor `HasExplosives` checks see leader grenades too.
- Removed: stale `grenade_attack_amount` helper that only modeled own inventory and triggered dead-code after wiring the source owner.
- Verification: `cargo check -q`, `cargo test -q` (385 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/` passed with canvas non-black and no console errors.

## Completed slice: robot grenade-ready attack pose

- Source: `source/zrobot.cpp` `ZRobot::Common_Process`, `ZRobot::SetAttackObject`, `ZRobot::CanThrowGrenades`; `source/rgrunt.cpp`, `source/rpsycho.cpp`, `source/rpyro.cpp`, `source/rlaser.cpp`, `source/rsniper.cpp` `DoRender`/`Process` attack branches.
- Rust owner: `src/units/robots/robot_state.rs` for robot attack-pose policy; `src/units/robots/robot_ui.rs` still owns `throw_something` frame names.
- Runtime call site: `src/main.rs::sync_robot_grenade_ready_attack_poses`, scheduled after `process_attack_targets` and `move_commanded_objects`, before `animate_robot_grenade_throw_animations`.
- Done: non-Tough robots with an active attack target hold `throw_something[direction][0]` while they can throw own/leader grenades or while the target is `AttackedOnlyByExplosives`; moving robots keep walk frames; the fired grenade animation still owns frames 0..3 after an actual grenade launch.
- Known difference: the general non-grenade `R_ATTACKING` fire-frame state machine and `SetAttackObject` initial attack delay are still not fully modeled.
- Verification: `cargo check -q`, `cargo test -q` (386 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/` passed with canvas non-black and no console errors.

## Completed slice: non-grenade robot attack fire frames

- Source: `source/zrobot.cpp` `ZRobot::SetAttackObject`/`Common_Process`; `source/rgrunt.cpp`, `source/rpsycho.cpp`, `source/rpyro.cpp`, `source/rlaser.cpp`, `source/rsniper.cpp`, `source/rtough.cpp` `DoRender` and `Process`.
- Rust owner: per-robot `src/units/robots/*_ui.rs` owns fire atlas names; `src/units/robots/robot_state.rs` owns shared `R_ATTACKING` frame progression and source timing.
- Runtime call site: `src/main.rs::animate_robot_fire_animations`, scheduled after movement/attack and after grenade-ready pose, before fired grenade animation.
- Done: non-grenade robot attack targets render original fire frame families: Grunt/Sniper 5 frames, Psycho 2, Pyro/Laser/Tough 3. Frame timing follows source ranges, including `SetAttackObject` initial `0.1` delay for non-Tough robots; Tough holds frame 0 until its rocket is fired, then plays frame 1 -> 2 -> 0.
- Known difference: direct/special projectile effects and damage are still triggered by existing combat cooldowns, not by callbacks from the source fire-frame animation frame (`action_i == 1/2/4`).
- Verification: `cargo check -q`, `cargo test -q` (388 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/` passed with canvas non-black and no console errors.

## Completed slice: robot fire-frame visual callbacks

- Source: `source/zobject.cpp` `ZObject::ProcessAttackDamage`; `source/rgrunt.cpp`, `source/rpsycho.cpp`, `source/rpyro.cpp`, `source/rlaser.cpp`, `source/rsniper.cpp` `Process`; `source/rtough.cpp` `FireMissile`/`Process`.
- Rust owner: `src/components.rs::RobotFireVisualCue` stores pending visual/sound callbacks; `src/units/robots/robot_state.rs` owns source callback frames; `src/main.rs::animate_robot_fire_animations` emits effects when entering those frames.
- Runtime call site: `src/main.rs::process_attack_targets` keeps direct damage timing in the combat cooldown path and defers only non-grenade, non-missile robot sound/bullet/flame/laser visuals to the fire animation callback.
- Done: Grunt/Sniper visual callbacks emit on frame 4, Psycho on frame 1, Pyro/Laser on frame 2. Grenades keep immediate missile/throw wiring, and Tough keeps missile delivery tied to its frame-1 rocket start path.
- Known difference: callback payload is a single current cue per robot, not a queue; this matches the source's current `attack_object` callback shape closely enough for this slice, but should be rechecked if attack timing is made more server-authoritative.
- Verification: `cargo check -q`, `cargo test -q` (389 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/` passed with canvas non-black and no console errors.

## Completed slice: robot grenade pickup animation

- Source: `source/zrobot.cpp` `ZRobot::DoPickupGrenadeAnim`, `ZRobot::Common_Process`, robot `Init`; `source/rgrunt.cpp` `DoRender` pickup branches; `source/rtough.h` pickup exclusion.
- Rust owner: `src/units/robots/robot_ui.rs` owns pickup atlas frame names; `src/units/robots/robot_state.rs` owns pickup direction/timing; `src/components.rs::RobotGrenadePickupAnimation` stores the one-shot runtime state.
- Runtime call site: `src/grenades.rs::process_grenade_pickups` inserts pickup animation after a successful grenade transfer; `src/main.rs::animate_robot_grenade_pickup_animations` advances/restores the robot sprite.
- Done: successful grenade pickup now plays source-backed `pickup-up/down` 4-frame robot animation at the shared robot process tick (`0.3s`); direction follows the original `direction < 4` split via current mobile rotation. Tough remains excluded by existing grenade pickup rules.
- Known difference: original scales the `Common_Process` interval by `SpeedOffsetPercentInv`; Rust uses the base `0.3s` for this slice until the broader robot action-mode process is ported.
- Verification: `cargo check -q`, `cargo test -q` (390 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/` passed with canvas non-black and no console errors.

## Completed slice: robot standing idle action modes

- Source: `source/zrobot.cpp` `ZRobot::Common_Process` and robot `Init`; `source/rgrunt.cpp` `DoRender` standing/action branches.
- Rust owner: `src/units/robots/robot_state.rs` owns idle random branch policy, action frame counts and timing; `src/units/robots/robot_ui.rs` owns idle action atlas frame names; `src/components.rs::RobotIdleProcessTimer` and `RobotIdleActionAnimation` store runtime state.
- Runtime call site: `src/main.rs::animate_robot_idle_actions`, scheduled after movement/attack/grenade visual systems, gates itself off whenever robot movement, task, attack, fire, throw, pickup or grenade-ready state owns the sprite.
- Done: standing robots now tick the source `0.3s` idle process: 90% no-op, otherwise 2/3 random turn or 1/3 action. Wired source action families `cigarette` (11 frames), `beer` (10), `full_area_scan` (12), `head_stretch` (11), with direction reset to original direction 6 before action frames.
- Known difference: original applies `SpeedOffsetPercentInv` to the idle process tick; Rust uses the base `0.3s` until full robot speed-offset process state is ported. `look_around` assets remain intentionally unwired because this `Common_Process` branch does not select them.
- Verification: `cargo check -q`, `cargo test -q` (392 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/` passed with canvas non-black and no console errors.

## Completed slice: robot attack target lifecycle reset

- Source: `source/zrobot.cpp` `ZRobot::SetAttackObject` and `ZRobot::Common_Process`; `source/zobject.cpp` `ZObject::Engage`, `ZObject::Disengage`, `ZObject::ProcessAttackDamage`; `source/rgrunt.cpp`, `source/rpsycho.cpp`, `source/rpyro.cpp`, `source/rlaser.cpp`, `source/rsniper.cpp` fire-frame callbacks.
- Rust owner: `src/components.rs` owns attack-target lifecycle helpers and cue target binding; `src/units/robots/robot_state.rs` owns source reset profile for frame `0` and `next_attack_time = now + 0.1`.
- Runtime call sites: passive target assignment in `src/main.rs`, player right-click assignment in `src/selection.rs`, invalid/clear target paths in combat, repair, eject and destruction cleanup, plus `src/main.rs::animate_robot_fire_animations`.
- Done: new/changed attack targets now clear stale robot fire animation, pending visual cue, grenade-ready pose, pickup animation and idle state while preserving the combat damage cooldown; clear target paths remove the same visual lifecycle state. `RobotFireVisualCue` is bound to `target_ref_id`, so old queued visuals cannot fire after retarget/disengage.
- Known difference: source server/client attack-object flags and target-destroyed portrait callbacks are still not modeled as packets/events; this slice only fixes local target lifecycle and stale visual state.
- Verification: `cargo check -q`, `cargo test -q` (395 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/` passed with `boot=started`, canvas non-black and no console errors.

## Completed slice: robot Common_Process speed offset

- Source: `source/zrobot.cpp` `ZRobot::Common_Process`; `source/zobject.cpp` `ZObject::SpeedOffsetPercent`, `ZObject::IsMoving`, `ZObject::SetVelocity`, `ZObject::ProcessMoveOrKillWP`.
- Rust owner: `src/units/unit_behavior.rs` owns source velocity/base-speed ratio; `src/units/robots/robot_state.rs` owns scaled Common_Process delta; `src/units/unit_ui.rs` applies the scaled delta only to robot mobile frames.
- Runtime call site: `src/main.rs::move_commanded_objects` computes `actual_move_speed = path.speed * terrain_speed * movement_multiplier`, passes `speed_offset_percent` into `update_mobile_sprite`, and advances robot walk frames with source-equivalent scaled time.
- Done: robot walk animation now advances faster/slower with run, terrain and damage velocity ratio like `next_process_time = process_time_int * SpeedOffsetPercentInv()` in the original. Vehicle mobile frames stay on their own raw timing. Stationary idle/pickup/attack-direction modes still see ratio `1.0`, matching `SpeedOffsetPercent()` returning `1.0` when not moving.
- Known difference: this slice covers local movement/animation timing; source packet-level velocity updates and client smoothing are still not fully modeled.
- Verification: `cargo check -q`, `cargo test -q` (397 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/` passed with `boot=started`, canvas non-black and no console errors.

## Completed slice: vehicle lid state and sniping gate

- Source: `source/zobject.cpp` `ZObject::Engage`, `ZObject::Disengage`, `ZObject::RemoveObject`; `source/zvehicle.cpp` `ZVehicle::SignalLidShouldOpen`, `SignalLidShouldClose`, `ProcessServerLid`, `ProcessLid`, `CanBeSniped`.
- Rust owner: `src/components.rs::VehicleLidState` stores runtime lid state; `src/units/vehicles/mod.rs` owns source lid signal/timer/animation policy; `src/units/attack.rs` owns the sniping gate.
- Runtime call sites: `src/world_objects.rs::initial_vehicle_lid_state` adds lid state to Light/Medium/Heavy vehicles; `src/main.rs::process_vehicle_lids` runs after flag capture and before damage; `src/main.rs::process_attack_targets` reads `VehicleLidState.open` for driver sniping.
- Done: Light/Medium/Heavy now open lids with the source 80% roll only when a new snipe-capable attack target is engaged, schedule close on disengage/no valid target with `0.1 * (rand() % 15)`, advance lid frame/show-driver timing at `0.2s`, and require actual open lid state for sniping instead of the old attack-target approximation.
- Known difference: `SET_LID_OPEN` packet/event parity is not modeled yet, and visual `tank_lid_r*_n*` / driver overlay layers are still not rendered.
- Verification: `cargo check -q`, `cargo test -q` (398 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/` passed with `boot=started`, canvas non-black (`nonblack_ratio=0.4991`) and no console errors.

## Completed slice: vehicle lid visual overlays and unit layout

- Source: `source/zvehicle.cpp` `ZVehicle::RenderLid`, `ProcessLid`; tank render offsets from `source/vlight.cpp`, `source/vmedium.cpp`, `source/vheavy.cpp`.
- Rust owner: shared lid/driver visual policy in `src/units/vehicles/vehicle_ui.rs`; per-tank offsets in `src/units/vehicles/light_ui.rs`, `medium_ui.rs`, `heavy_ui.rs`; runtime overlay state in `src/components.rs`.
- Runtime call sites: `src/world_objects.rs` spawns lid/driver overlay layers for Light/Medium/Heavy vehicles; `src/main.rs::sync_vehicle_lid_visual_layers` follows vehicle/turrent rotation, source render order, lid frame, and driver visibility.
- Done: `tank_lid_r*_n*` and `tank_fire_team_r*_n*` visual layers are loaded, hidden/shown from `VehicleLidState`, positioned from source top-left offsets, and synchronized against base/turrent transforms.
- Layout done: concrete unit files across `robots`, `vehicles`, `cannons`, `buildings`, and `items` now use `src/units/[type]/[unit_name]/[unit_name]_logic.rs`, `[unit_name]_ui.rs`, `[unit_name]_mod.rs`; shared family facades stay separate.
- Known difference: `SET_LID_OPEN` packet/event parity is still not modeled.
- Verification: `cargo check -q`, `cargo test -q` (401 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1780645236378` passed with `boot=started`, canvas non-black (`nonblack_ratio=0.9962`, screenshot `output/playwright/units-structure-smoke-converted.png`).

## Completed slice: nested concrete unit layout

- Source: project ownership requirement from current goal; no C/C++ behavior changed.
- Rust owner: `src/units/{robots,vehicles,cannons,buildings,items}` family modules.
- Done: concrete units were moved from family root into per-unit directories:
  `src/units/[type]/[unit_name]/[unit_name]_logic.rs`,
  `src/units/[type]/[unit_name]/[unit_name]_ui.rs`,
  `src/units/[type]/[unit_name]/[unit_name]_mod.rs`.
- Family `mod.rs` files now use `#[path = "[unit_name]/[unit_name]_mod.rs"]` and keep old UI compatibility wrappers, so runtime call sites stay stable.
- Layout audit: no flat concrete `*_logic.rs` or `*_mod.rs` files remain at family-root depth; no flat `#[path = "..._mod.rs"]` attrs remain.
- Verification: `cargo check -q`, `cargo test -q` (401 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781279250452` passed with `boot=started`, canvas non-black (`nonblack_ratio=0.5363`, screenshot `output/playwright/nested-unit-layout-smoke.png`).
- Runtime note: root `index.html` imports `./pkg/...`; recreated local `pkg -> web/pkg` symlink before browser smoke because the current workspace only had `web/pkg` after wasm build.

## Completed slice: building production queue order and single-completion tick

- Source: `source/zbuilding.cpp` `ZBuilding::SetBuildingProduction`, `AddBuildingQueue`, `BuildUnit`, `ResetProduction`; `source/zserver_events.cpp` `ZServer::add_building_queue_event`; `source/zserver.cpp` completion path.
- Rust owner: `src/units/buildings/production_logic.rs` owns queue progression/completion timing; `src/production.rs` is now only a compatibility facade for existing call sites; `src/production_ui.rs` owns local UI queue commands; `src/main.rs::process_building_production` owns Bevy spawning/storage orchestration.
- Done: UI queue add now mirrors the original server event default by inserting active-production queue items at the front, so the newly queued unit is built next instead of after the current loop item.
- Done: `advance_production` now completes at most one unit per call and starts the next queued unit with zero elapsed time, matching C `BuildUnit` followed by `ResetProduction`; long browser frames no longer overproduce multiple queued units in one Bevy update.
- Done: removed Rust-only `BuildingProduction::ready_units`; production completion now exists only as the returned per-tick event, matching C `BuildUnit` out-params. Cannon storage remains represented by `stored_cannons`, matching `built_cannon_list`.
- Done: moved building production queue/storage/completion policy out of generic `src/production.rs` into `src/units/buildings/production_logic.rs`; runtime and test re-exports in `src/production.rs` are split so rustc stays warning-clean.
- Done: removed stale `#[allow(dead_code)]` from production helpers that are runtime-wired through production UI and cannon placement.
- Tests: added source-backed coverage for front-insert queue order and long-delta single completion.
- Verification: `cargo check -q`, `cargo test -q` (403 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781280500001` rendered a non-black game screen (screenshot `output/playwright/building-production-owner-smoke.png`) with no browser console errors; only pre-existing wasm-bindgen init deprecation warnings appeared.

## Completed slice: factory overlay source tick timing

- Source: `source/brobot.cpp` `BRobot::Process`/`DoAfterEffects`; `source/bvehicle.cpp` `BVehicle::Process`/`DoAfterEffects`.
- Rust owner: `src/units/buildings/building_ui.rs` owns factory overlay timing policy; `src/main.rs::animate_factory_overlays` remains the Bevy orchestration call site.
- Done: factory overlays now use a source-style bounded `0.25s` process tick that advances at most once per update and discards long-frame overshoot, matching `last_process_time = the_time` in both original factory `Process` methods.
- Done: robot factory spin/green_box/robot/exhaust and vehicle factory spin/vent/exhaust/bulb/tank frame families keep their existing source-backed frame lists and visibility; this slice only changes the timer semantics from modulo carry-over to source discard.
- Tests: added `factory_overlay_process_tick` coverage for no-tick, threshold tick and long-delta single-tick cases.
- Verification: `cargo check -q`, `cargo test -q` (403 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781280900001` rendered a non-black game screen (screenshot `output/playwright/factory-overlay-timing-smoke.png`) with no browser console errors; only pre-existing wasm-bindgen init deprecation warnings appeared.

## Completed slice: cannon storage zone-count parity

- Source: `source/zbuilding.cpp` `ZBuilding::StoreBuiltCannon`, `CannonsInZone`, `RemoveStoredCannon`, `HaveStoredCannon`; `source/zserver.cpp` `ZServer::BuildingCreateUnit`, `BuildingCreateCannon`; `source/zserver_events.cpp` `place_cannon_event`.
- Rust owner: `src/units/buildings/production_logic.rs` owns cannon zone snapshots, zone counting and `StoreBuiltCannon` gating; `src/main.rs::process_building_production` remains Bevy orchestration.
- Done: zone-wide cannon capacity now uses a building-owned `CannonZoneSnapshot` and `store_built_cannon_in_zone` helper instead of inline counting in `main.rs`.
- Done: cannon completion stores only when placed cannons plus unplaced stored cannons in the same zone are below `MAX_STORED_CANNONS`; when full, the completed cannon is dropped after production reset, matching original `BuildingCreateUnit` returning `NULL` after `CannonsInZone >= MAX_STORED_CANNONS`.
- Tests: added owner-local tests for zone count, successful store snapshot update, and full-zone drop without mutating `stored_cannons`.
- Verification: `cargo check -q`, `cargo test -q` (406 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781281300001` rendered a non-black game screen (screenshot `output/playwright/cannon-storage-zone-parity-smoke.png`) with no browser console errors; only pre-existing wasm-bindgen init deprecation warnings appeared.

## Completed slice: building create-unit outcome ownership

- Source: `source/zserver.cpp` `ZServer::ProcessObjects`, `BuildingCreateUnit`, `BuildingCreateCannon`, `CreateRobotGroup`; `source/zbuilding.cpp` `BuildUnit`, `ResetProduction`, `GetBuildingCreationPoint`, `GetBuildingCreationMovePoint`; `source/zobject.cpp` `CloneMinionWayPoints`.
- Rust owner: `src/units/buildings/production_logic.rs` owns the source-backed `BuildingCreateUnit` decision: spawn non-cannon object groups, store completed cannons, or drop completed cannons when zone capacity is full. `src/production.rs` remains a compatibility facade; `src/main.rs::process_building_production` only performs Bevy entity spawning from the owner outcome.
- Done: added `BuildingCreateUnitOutcome` and `building_create_unit_outcome`, moving cannon/non-cannon completion branching out of `main.rs` and into the building production owner.
- Done: non-cannon completions now return an explicit spawn batch with source group count; cannon completions return `StoredCannon` or `DroppedCannon`, matching original `BuildingCreateUnit` returning a new object only for non-cannons and `NULL` for cannons.
- Tests: added owner-local coverage for non-cannon spawn outcome, stored cannon outcome, and dropped cannon outcome.
- Known difference: original appends building rally points to the created leader after the initial factory-exit waypoint relay. Rust still only assigns the factory-exit `MovementPath`; typed waypoint/rally point parity remains a separate source slice.
- Verification: `cargo check -q`, `cargo test -q` (408 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781281800001` rendered a non-black game screen (screenshot `output/playwright/building-create-unit-outcome-smoke.png`) with no browser console errors.

## Completed slice: production building rally points

- Source: `source/zplayer.cpp` `ZPlayer::AddDevWayPointToSelected`, `SendDevWayPointsOfSelected`; `source/zcore.cpp` `ProcessRallypointData`, `CheckRallypoint`; `source/zserver_events.cpp` `rcv_object_rallypoints_event`; `source/zserver.cpp` production completion rally append; `source/zobject.cpp` `CloneMinionWayPoints`.
- Rust owner: `src/units/buildings/production_logic.rs` owns rally point command semantics and production spawn route policy; `src/components.rs::BuildingRallyPoints` stores per-building rally state; `src/selection.rs::handle_building_rally_point_commands` is the local UI event bridge; `src/world_objects.rs::spawn_runtime_object_with_route` performs route-aware spawning.
- Done: producing buildings now carry `BuildingRallyPoints`; map-spawned and newly captured production buildings receive the component with production state.
- Done: while a production window is open, right-click stores a building rally point; normal right-click replaces the list and Shift-right-click appends, matching the original dev-waypoint send/accumulate flow.
- Done: production spawn route now starts with the original factory-exit move point and appends building rally points for the produced leader/new object.
- Done: robot-group minions keep only the factory-exit route, matching the original order where `CloneMinionWayPoints` runs before the caller appends rally points to `new_obj`.
- Tests: added owner-local coverage for replace/append rally point commands, route append order, and leader/minion route split.
- Known difference: Rust still stores rally points as world `Vec2` route points, not full typed `waypoint` records with `mode`, `ref_id`, `attack_to`, and `player_given`; rally marker rendering and network packet parity are still separate slices.
- Verification: `cargo check -q`, `cargo test -q` (411 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781282400001` rendered a non-black game screen (screenshot `output/playwright/building-rally-points-smoke.png`) with no browser console errors.

## Completed slice: production typed waypoint layer

- Source: `source/zobject.h` `waypoint` fields and `waypoint_mode`; `source/zserver.cpp` `BuildingCreateUnit` factory-exit `FORCE_MOVE_WP`; `source/zplayer.cpp` rally `MOVE_WP` creation; `source/zcore.cpp` rally validation accepting only `MOVE_WP`; `source/zobject.cpp` `ProcessMoveOrKillWP` split between stoppable `MOVE_WP` and non-stoppable `FORCE_MOVE_WP`.
- Rust owner: `src/components.rs::MovementWaypoint`/`MovementPath` stores source-backed waypoint metadata; `src/units/buildings/production_logic.rs` owns production route construction; `src/world_objects.rs::spawn_runtime_object_with_waypoints` owns typed route insertion for runtime spawns.
- Done: `MovementPath` keeps the existing compatibility `Vec<Vec2>` route and now carries parallel typed waypoints with `mode`, `ref_id`, `attack_to`, and `player_given`.
- Done: production building exits are emitted as `MovementWaypointMode::ForceMove` with `ref_id = None`; rally tails are emitted as `MovementWaypointMode::Move` with `attack_to = true` and `player_given = true`.
- Done: robot-group minions still receive only the factory-exit `ForceMove` waypoint; only the leader/new object receives the rally tail, preserving the original `CloneMinionWayPoints` ordering.
- Done: runtime production spawn now calls `spawn_runtime_object_with_waypoints`; old `Vec2` route helpers were removed from runtime API or restricted to tests.
- Tests: added owner-local coverage for production waypoint modes/flags and minion force-exit-only route.
- Known difference: movement processing still uses positions for actual movement; full `MOVE_WP` vs `FORCE_MOVE_WP` stoppable/pathfinder behavior, packet relay, and rally marker rendering remain separate source-backed slices.
- Verification: `cargo check -q`, `cargo test -q` (413 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781283000001` loaded the game screen (screenshot `output/playwright/typed-waypoint-layer-smoke.png`); console only showed the pre-existing wasm-bindgen init deprecation warning plus favicon 404.

## Completed slice: ForceMove movement gate

- Source: `source/zobject.cpp` dispatch maps `MOVE_WP` to `ProcessMoveWP(..., stoppable=true)` and `FORCE_MOVE_WP` to `stoppable=false`; `ProcessMoveWP` uses pathfinding/wait only for stoppable move, while force move does direct `SetTarget` + `SetVelocity`; `ProcessMove` checks impassable collision only when `stoppable` is true.
- Rust owner: `src/components.rs::MovementWaypoint::stoppable` owns the mode-to-stoppable mapping; `src/main.rs::move_commanded_objects` owns the current runtime gate.
- Done: current movement target selection now goes through typed waypoint helpers, preserving fallback compatibility for old `Vec2` routes.
- Done: `FORCE_MOVE_WP` no longer uses the stoppable terrain halt gate in the Rust movement loop; it keeps direct movement toward the typed target, matching the source production-exit route behavior.
- Done: ordinary `MOVE_WP` routes keep the current stoppable terrain gate and pathing-produced route shape.
- Tests: added coverage for ForceMove bypassing the terrain halt gate and Move keeping it.
- Known difference: Rust still does not model full `cur_wp_info`, async pathfinder response, impassable-object attack, or `CheckAttackTo` during movement; this slice only wires the source-backed `stoppable` boundary into the current Bevy movement loop.
- Verification: `cargo check -q`, `cargo test -q` (415 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781283600001` loaded the game screen (screenshot `output/playwright/force-move-gate-smoke.png`); console only showed the pre-existing wasm-bindgen init deprecation warning plus favicon 404.

## Completed slice: movement CheckAttackTo hook

- Source: `source/zobject.cpp` `ZObject::CheckAttackTo`, `WithinAgroRadius`, `CanOverwriteWP`, `ProcessMoveWP`; `source/zcore.cpp` `CheckWaypoint` clearing `attack_to` when the object cannot attack.
- Rust owner: `src/units/unit_behavior.rs::attack_to_target_choices` owns the source candidate filter; `src/main.rs::move_commanded_objects` owns the Bevy movement interruption and `AttackTarget` assignment.
- Done: current typed movement waypoint metadata now drives an `attack_to` check before movement advances.
- Done: only stoppable current waypoints can be interrupted, so `FORCE_MOVE_WP` production exits preserve the source `CanOverwriteWP` rule.
- Done: candidate targets follow source filters: enemy non-null team, robot/vehicle/cannon only, inside `attack_radius + AGRO_DISTANCE`, and chosen by the shared game RNG instead of nearest-target ordering.
- Done: the hook stops/removes `MovementPath` and assigns `AttackTarget` for every entity layer sharing the same object ref, avoiding split movement across sprite layers.
- Tests: added owner-local coverage for `attack_to` candidate radius/kind/team filters.
- Known difference: Rust still does not insert a real `ATTACK_WP` at the front of a typed waypoint queue, so original resume-after-attack waypoint behavior is not fully modeled yet. Full `cur_wp_info`, async pathfinder response, impassable-object attack, and attack waypoint chasing remain separate slices.
- Verification: `cargo check -q`, `cargo test -q` (416 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781284200001` loaded the game screen (screenshot `output/playwright/check-attack-to-smoke.png`); console only showed the pre-existing wasm-bindgen init deprecation warning plus favicon 404.

## Completed slice: movement front-insert ATTACK_WP

- Source: `source/zobject.cpp` `ZObject::CheckAttackTo`, `ProcessServer` `ATTACK_WP` dispatch, `ProcessAttackWP`, `KillWP`; `source/zobject.h` `waypoint`.
- Rust owner: `src/components.rs::MovementWaypoint` / `MovementPath` owns typed `ATTACK_WP` queue state; `src/main.rs::move_commanded_objects` owns runtime processing and layer synchronization.
- Done: `CheckAttackTo` now inserts a typed `MovementWaypointMode::Attack` at the front of every visual layer path for the object instead of deleting `MovementPath` and assigning `AttackTarget` immediately.
- Done: the inserted auto-attack waypoint uses the source default `attack_to=false`, `player_given=false`, stores the target `ref_id`, and preserves the interrupted `MOVE_WP` / rally route behind it.
- Done: current `ATTACK_WP` processing keeps the attack waypoint first while the target is valid; when the target is in attack radius it assigns/keeps `AttackTarget` without resetting attack visual lifecycle every tick.
- Done: if the target disappears or becomes invalid, only the front `ATTACK_WP` is popped for every layer and the previous route resumes.
- Done: `ATTACK_WP` is not eligible for another `CheckAttackTo` overwrite, but it still uses the normal terrain halt gate rather than `FORCE_MOVE_WP` direct movement.
- Done: when a target leaves attack radius after an `AttackTarget` was already assigned, all layers clear/pause on the same object-ref tick before chase movement resumes, avoiding base/visual-layer drift.
- Tests: added coverage for front-insert preserving resume route and for `ATTACK_WP` terrain/overwrite semantics.
- Known difference: Rust still does not model full `cur_wp_info`, async pathfinder responses, impassable-object attack insertion, direct player/network `ATTACK_WP` relay parity, or exact path-to-nearest-attack-location behavior.
- Verification: `cargo check -q`, `cargo test -q` (418 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781284800002` rendered a non-black viewport (`nonblack_ratio=0.9814`, screenshot `output/playwright/attack-wp-front-insert-smoke.png`); console only showed the pre-existing wasm-bindgen init deprecation and favicon 404.

## Completed slice: player attack-command ATTACK_WP route

- Source: `source/zplayer.cpp` `ZPlayer::AddDevWayPointToSelected`; `source/zcore.cpp` `ProcessWaypointData` / `CheckWaypoint`; `source/zobject.h` `waypoint`.
- Rust owner: `src/components.rs::MovementWaypoint` owns source `ATTACK_WP` metadata; `src/selection.rs` owns local player right-click command construction; `src/main.rs::move_commanded_objects` owns runtime validation and execution.
- Done: right-clicking an enemy no longer assigns `AttackTarget` directly. It clears the old task/attack lifecycle and inserts a typed route ending in `MovementWaypointMode::Attack`.
- Done: player command `ATTACK_WP` uses source command metadata: `ref_id=target`, `player_given=true`, `attack_to=true`; auto `CheckAttackTo` attack waypoints still keep `attack_to=false`.
- Done: mobile attackers keep the existing route-to-attack-range points as temporary `MOVE_WP` path points, then process the final typed `ATTACK_WP`; stationary attackers receive only the attack waypoint.
- Done: typed attack waypoint validation now uses the same own-first/group-leader grenade availability model as combat, so explosive-only targets are not dropped when a robot attacks with leader grenades available.
- Tests: added coverage for player attack route metadata and leader-grenade `ATTACK_WP` validation.
- Known difference: Rust still does not model packet-level `ProcessWaypointData`, bot direct `ATTACK_WP` coordinate rules (`GetCords()+8`), full `cur_wp_info` path state, async pathfinder response, or impassable-object attack insertion.
- Verification: `cargo check -q`, `cargo test -q` (420 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781285400001` rendered a non-black viewport (`nonblack_ratio=0.9815`, screenshot `output/playwright/player-attack-wp-route-smoke.png`); console only showed the pre-existing wasm-bindgen init deprecation and favicon 404.

## Completed slice: player move-command MOVE_WP attack_to metadata

- Source: `source/zplayer.cpp` `ZPlayer::UnitNearHostiles`, `AddDevWayPointToSelected`; `source/zobject.cpp` `ZObject::CheckAttackTo`; `source/zobject.h` `waypoint`.
- Rust owner: `src/components.rs::MovementWaypoint` owns source `MOVE_WP` metadata; `src/selection.rs` owns local player right-click move command construction and the near-hostile/Ctrl/Alt policy; `src/main.rs::move_commanded_objects` consumes `attack_to` through the existing `CheckAttackTo` runtime hook.
- Done: ordinary right-click move commands now mark the final typed movement waypoint as `player_given=true` and preserve internal route points as non-player path waypoints.
- Done: final `MOVE_WP.attack_to` now follows the original rule: default true, false when the selected unit is near hostile targets, forced true by Ctrl, and forced false by Alt after Ctrl.
- Done: near-hostile detection reuses the same source-backed passive attack candidate filters as `CheckAttackTo`, so robots, vehicles and cannons in aggro range suppress default attack-move behavior consistently.
- Tests: added coverage for final-waypoint-only metadata and Ctrl/Alt/near-hostile precedence.
- Known difference: this slice covers local player move command construction only; packet relay, exact server `CheckWaypoint` processing, full `cur_wp_info`, async pathfinder response, impassable-object attack insertion, and waypoint cursor/render parity are still separate source-backed slices.
- Verification: `cargo check -q`, `cargo test -q` (422 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781286000001` rendered a non-black viewport (`nonblack_ratio=0.9814`, screenshot `output/playwright/player-move-attack-to-smoke.png`); console only showed the pre-existing wasm-bindgen init deprecation and favicon 404.

## Completed slice: minion non-stoppable movement gate

- Source: `source/zobject.cpp` `ZObject::ProcessMove`, `ProcessMoveOrKillWP`; `source/zobject.cpp` `ZObject::IsMinion`.
- Rust owner: `src/main.rs::MovementSpeedSnapshot` carries source minion identity into movement runtime; `src/main.rs::movement_path_current_waypoint_uses_terrain_halt` owns the current terrain/collision halt gate.
- Done: movement runtime now treats `RobotGroup` minions (`leader_ref_id != ref_id`) as non-stoppable for terrain halt, matching `if(IsMinion()) stoppable = false`.
- Done: the `CheckAttackTo` overwrite gate still uses waypoint stoppability before this runtime movement gate, matching the original order where `CheckAttackTo` runs before `ProcessMove`.
- Done: `FORCE_MOVE_WP` keeps its existing non-stoppable behavior; leader/non-minion `MOVE_WP` and `ATTACK_WP` still use the terrain halt gate.
- Tests: added coverage that a minion `MOVE_WP` bypasses the stoppable terrain halt gate while the same waypoint remains stoppable for non-minions.
- Known difference: this slice only ports the minion-specific `ProcessMove` stoppable override. Full `cur_wp_info`, async pathfinder response, impassable-object attack insertion, and minion waypoint clone packet parity remain separate source-backed slices.
- Verification: `cargo check -q`, `cargo test -q` (423 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781286600001` rendered a non-black viewport (`nonblack_ratio=0.5685`, screenshot `output/playwright/minion-nonstoppable-smoke-rgba.png`); console only showed the pre-existing wasm-bindgen init deprecation warning.

## Completed slice: production manufactured computer sounds

- Source: `source/zserver.cpp` `ZServer::ProcessObjects`, `BuildingCreateUnit`, `BuildingCreateCannon`, `RelayObjectManufacturedSound`; `source/zplayer_events.cpp` `set_computer_message_event`; `source/zsound_engine.cpp` computer manufactured sound asset loading.
- Rust owner: `src/main.rs::GameSoundKind` owns local audio event variants; `src/main.rs::manufactured_sound_for_unit` owns source object-type-to-computer-sound mapping; `src/main.rs::process_building_production` owns completion-time playback.
- Done: successful non-cannon production completions now play source computer manufactured sounds once per produced leader/object: robot -> `comp_robot_manufactured.wav`, vehicle -> `comp_vehicle_manufactured.wav`.
- Done: successful stored cannon completions now play `comp_gun_manufactured.wav`, matching `BuildingCreateCannon`; dropped cannons in a full zone stay silent because the source never calls `BuildingCreateCannon` there.
- Done: playback is filtered to the local Red team, mirroring source `RelayTeamMessage(owner, COMP_MSG, ...)` in the current single-local-player model.
- Tests: added coverage for sound asset paths and source object-type mapping.
- Known difference: this slice ports only sound playback. Computer message UI text, spacebar events, start/cancel manufacture sounds, and a future local-player-team abstraction remain separate slices.
- Verification: `cargo check -q`, `cargo test -q` (424 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781287200001` rendered a non-black viewport (`nonblack_ratio=0.5212`, screenshot `output/playwright/manufactured-sound-smoke.png`); console only showed the pre-existing wasm-bindgen init deprecation warning.

## Completed slice: production start/cancel manufacture sounds

- Source: `source/gwproduction.cpp` `GWProduction::DoOkButton`/`DoCancelButton`; `source/zplayer.cpp` START/STOP sends; `source/zserver_events.cpp` `start_building_event`/`stop_building_event`; `source/zplayer_events.cpp` `set_computer_message_event`; `source/zsound_engine.cpp` sound load/play mapping.
- Rust owner: `src/production_ui.rs::handle_production_window_input` local UI/server merged ACK point; constants/helpers own asset paths/playback.
- Done: successful OK/full-selector production starts play `comp_starting_manufacture.wav`.
- Done: successful cancel from active `BUILDING_BUILDING` plays `comp_manufacturing_canceled.wav`.
- Done: queue add/cancel and non-changing cancel/select paths stay silent, matching source.
- Tests: added asset path coverage.
- Known difference: no full `COMP_MSG` server ACK pipeline, message UI, spacebar events, or local-player-team abstraction yet; this is local production window feedback only.
- Verification: `cargo check -q`, `cargo test -q` (425 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781287800001` rendered a non-black viewport (`nonblack_ratio=0.5211`, screenshot `output/playwright/production-start-cancel-sound-smoke.png`); console only showed the pre-existing wasm-bindgen init deprecation warning.

## Completed slice: production manufactured `COMP_MSG` UI and spacebar events

- Source: `source/event_handler.h` `computer_msg_packet`/`COMP_MSG`; `source/zserver.cpp` `RelayObjectManufacturedSound` and `BuildingCreateCannon`; `source/zplayer_events.cpp` `set_computer_message_event`; `source/zcomp_message_engine.cpp` `Init`/`DisplayMessage`/`Process`/click focus; `source/zplayer.h`/`source/zplayer.cpp` `SpaceBarEvent` queue and `DoSpaceBarEvent`.
- Rust owner: `src/components.rs` owns `ComputerMessageKind`, `ComputerMessageDisplay`, and `SpaceBarEventQueue`; `src/hud.rs` owns message image/timing/click flags; `src/selection.rs::process_space_bar_events` owns source-style Space behavior; `src/main.rs::relay_local_manufactured_feedback` owns local completion-time feedback.
- Done: manufactured robot/vehicle completions now show `robot_manufactured.png` / `vehicle_manufactured.png`, queue Space/click focus with `select_obj=true`, and target the produced object `ref_id`.
- Done: stored cannon completions now show `gun_manufactured.png`, queue Space/click focus with `open_gui=true`, and target the producing building `ref_id`.
- Done: `SpaceBarEventQueue` matches source max 5, lifetime 10 seconds, dedupe by `ref_id`, newest-first insert, and rotate-to-back after a successful Space event.
- Done: fort-under-attack message now shares the same computer message display and Space queue with `select_obj=false` / `open_gui=false`.
- Tests: added coverage for manufactured message kind mapping, local feedback `ref_id`/team rules, message focus flags, and queue dedupe/limit/lifetime.
- Known difference: this slice keeps the current local Red-team model. Full `COMP_MSG` packet decode/server ACK pipeline, territory/radar messages, stored-gun left HUD stack, pause resume banner, and local-player-team abstraction are separate slices.
- Verification: `cargo check -q`, `cargo test -q` (429 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781776200001` rendered a non-black viewport (`nonblack_ratio=0.9997`, screenshot `output/playwright/comp-msg-smoke.png`); console only showed the pre-existing wasm-bindgen init deprecation warning.

## Completed slice: stored-gun left HUD stack

- Source: `source/zcomp_message_engine.h` `MAX_RENDERABLE_STORED_GUNS`; `source/zcomp_message_engine.cpp` `Init`, `RenderGuns`, `AbsorbedLClick`, `AbsorbedLUnClick`; `source/zplayer_events.cpp` GUI/focus handling after `zcomp_msg.GetFlags()`.
- Rust owner: `src/hud.rs` owns stored-gun stack layout/filter/render/click policy; `src/components.rs` owns HUD stack markers and click state; `src/main.rs` wires `gun.png`, update, and input systems.
- Done: owned live production buildings with non-empty `BuildingProduction.stored_cannons` render up to 8 `other/comp_messages/gun.png` icons at source positions `(8, 8 + slot*16)`.
- Done: multipliers render as `X2..X4` at source offset `+20,+3`; count `1` stays icon-only.
- Done: clicking a rendered stack icon pushes `HudCommand::FocusObject { select_obj: false, open_gui: true }`, so the camera focuses the source building and opens its production GUI.
- Done: stack clicks set the existing production-window input capture flag for that frame, avoiding same-frame map selection/placement.
- Tests: added coverage for slot coordinates/icon hit rect, eligible building filters/order/cap, multiplier count gate, and click ref resolution.
- Known difference: this slice uses local Red-team filtering and existing local `BuildingProduction.stored_cannons`. Network `SET_BUILT_CANNON_AMOUNT`, packet sync, pause/resume banner, and broader team abstraction remain separate slices.
- Verification: `cargo check -q`, `cargo test -q` (433 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781776900001` rendered a non-black viewport (`nonblack_ratio=1.0000`, screenshot `output/playwright/stored-gun-hud-smoke.png`); `gun.png` was requested successfully and console only showed the pre-existing wasm-bindgen init deprecation warning.

## Completed slice: AwardZone territory/radar computer sounds

- Source: `source/zserver.cpp` `AwardZone`; `source/oflag.cpp` `OFlag::HasRadar`; `source/zplayer_events.cpp` `set_computer_message_event`; `source/zsound_engine.cpp` sound load/play mapping.
- Rust owner: `src/main.rs::award_zone_to_team` owns local normal-capture feedback; `award_zone_computer_sounds` owns old/new team relay rules.
- Done: local Red old-team zone loss plays `comp_territory_lost.wav` when old owner is non-null.
- Done: local Red new-team capture of a linked-radar zone plays `comp_radar_activated.wav`.
- Done: linked radar detection matches source `HasRadar` by linked building kind only, not owner/destroyed state.
- Done: both events stay sound-only, with no banner or `SpaceBarEvent`.
- Tests: added asset path and relay-rule coverage.
- Known difference: current local player is still hard-coded as Red and no network `COMP_MSG` packet parser/relay exists; destroyed-fort elimination path and radar looping `RADAR_SND` are separate slices.
- Verification: `cargo check -q`, `cargo test -q` (434 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781777600001` rendered a non-black viewport (`nonblack_ratio=1.0000`, screenshot `output/playwright/award-zone-sounds-smoke.png`); console only showed the pre-existing wasm-bindgen init deprecation warning.

## Completed slice: pause/resume computer banner

- Source: `source/zcomp_message_engine.cpp` `Init`, `RenderResume`, `AbsorbedLClick`, `AbsorbedLUnClick`; `source/zplayer_events.cpp` left-click/release flow; `source/zplayer.cpp` `SendSetPaused`; `source/zclient.cpp` `ProcessUpdateGamePaused`; `source/event_handler.h` pause packet ids.
- Rust owner: `src/components.rs::GamePauseState` owns local pause state; `src/hud.rs` owns centered resume prompt render and press/release hit testing; `src/selection.rs::process_hud_commands` owns local resume command application.
- Done: source `click_to_resume.png` is loaded and rendered centered while the local game pause state is active.
- Done: mouse press must start over the prompt and release must still be inside before `HudCommand::ResumeGame` clears the pause state, matching source `started_over_gui` + `cmflags.resume_game` flow.
- Done: prompt clicks capture same-frame map/placement input through the existing production-window input-capture guard.
- Done: no sound, no `SpaceBarEvent`, and no normal blinking computer-message timeout are attached to pause/resume, matching source behavior.
- Done: fixed the stored-gun HUD runtime `Query` conflict by making icon and multiplier queries explicitly disjoint, unblocking native and browser rendering after the HUD slices.
- Tests: added centered prompt hit-rect coverage.
- Known difference: this is the local visual/input slice only. Source pause/resume vote packets, initial `GET_GAME_PAUSED`/`UPDATE_GAME_PAUSED`, `/pause` and `/resume`, and a separate frozen game-time model are still separate slices.
- Verification: `cargo check -q`, `cargo test -q` (435 passed), `./scripts/build-wasm.sh`, native screenshot `output/native-pause-resume.png`, and browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781780000001` rendered a non-black viewport (`nonblack_ratio=0.9996`, screenshot `output/playwright/pause-resume-banner-smoke.png`); console only showed the pre-existing wasm-bindgen init deprecation warning.

## Completed slice: pause packet constants and bool layout

- Source: `source/event_handler.h` `tcp_event` enum and `update_game_paused_packet`; `source/zplayer.cpp` `SendSetPaused`; `source/zclient.cpp` `ProcessUpdateGamePaused`.
- Rust owner: `src/network_commands.rs` owns packet ids and wire layout for the future pause network path.
- Done: `TcpEventId` now includes `UPDATE_GAME_PAUSED=50`, `GET_GAME_PAUSED=51`, and `SET_GAME_PAUSED=52`.
- Done: `SetGamePausedCommand` encodes source `update_game_paused_packet` as a one-byte C++ bool payload inside the existing length/event packet envelope.
- Done: `UpdateGamePausedPacket::decode_payload` decodes only exact one-byte `0`/`1` payloads.
- Tests: added source enum-position and packet-layout coverage.
- Known difference: this slice does not yet route packets through runtime input/server vote/client update systems; local resume still directly clears `GamePauseState`.
- Verification: `cargo check -q`, `cargo test -q` (437 passed).

## Completed slice: pause runtime packet path

- Source: `source/zplayer.cpp` `SendSetPaused`; `source/zserver_events.cpp` `set_pause_game_event`; `source/zserver.cpp` `PauseGame`/`ResumeGame`/`RelayGamePaused`; `source/zclient.cpp` `ProcessUpdateGamePaused`; `source/event_handler.h` `tcp_event` and `update_game_paused_packet`.
- Rust owner: `src/components.rs` owns pause request/update queues; `src/selection.rs::process_hud_commands` owns local resume request emission; `src/main.rs` owns local server/client packet round-trip and the single pause-state writer; `src/network_commands.rs` owns pause packet layout.
- Done: `HudCommand::ResumeGame` now queues `GamePauseRequest` instead of mutating `GamePauseState` directly.
- Done: startup emits a local `GET_GAME_PAUSED` packet envelope and relays the current pause state through `GamePauseUpdateQueue`.
- Done: request handling round-trips `SET_GAME_PAUSED`, applies the source same-state no-op guard, and emits `UPDATE_GAME_PAUSED` only for real state changes.
- Done: authoritative update handling round-trips `UPDATE_GAME_PAUSED` and keeps `apply_game_pause_update` as the only writer to `GamePauseState`.
- Tests: added no-op guard and client apply coverage.
- Known difference: local runtime auto-accepts pause/resume instead of modeling `PAUSE_VOTE`/`RESUME_VOTE`, and there is still no real socket transport or separate frozen game-time model.
- Verification: `cargo check -q`, `cargo test -q` (439 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781781300002` rendered a non-black viewport after click (`nonblack_ratio=0.9996`, screenshot `output/playwright/pause-runtime-packet-after-smoke.png`).

## Completed slice: pause vote/runtime parity

- Source: `source/zserver_events.cpp` `set_pause_game_event`; `source/zserver.cpp` `StartVote`, `VoteYes`, `CheckVote`, `KillVote`, `ProcessVote`, `VoteRequired`; `source/zcore.cpp` `VotesNeeded`, `VotesFor`, `VotesAgainst`; `source/zvote.cpp` `ZVote::StartVote`/`ResetVote`.
- Rust owner: `src/vote.rs` owns local vote state and quorum rules; `src/main.rs::process_game_pause_requests` owns pause request handoff from decoded `SET_GAME_PAUSED` into vote processing.
- Done: pause/resume requests now enter `pause_vote_update_for_request` instead of directly becoming a `GamePauseUpdate`.
- Done: local vote rules preserve source same-state no-op, login gate shape, `VoteRequired` single-player skip, voting-power fast path, active same-vote `VoteYes`, clear-on-kill behavior, even rounding in `VotesNeeded`, and pass-vote exclusion from quorum math.
- Done: the default one-player runtime still resumes immediately, but now because source `VoteRequired()==false`, not because pause bypasses vote ownership.
- Tests: added single-player direct process, same-state no-op, multi-player wait/second-yes majority, and quorum rounding/pass exclusion coverage.
- Known difference: vote UI, `/pause` and `/resume` chat commands, and real socket transport are still separate slices.
- Verification: `cargo check -q`, `cargo test -q` (443 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781782500001` rendered a non-black viewport after click (`nonblack_ratio=0.9996`, screenshot `output/playwright/pause-vote-runtime-after-smoke.png`).

## Completed slice: `VOTE_INFO` packet/client state

- Source: `source/event_handler.h` `tcp_event` and packed `vote_info_packet`; `source/zserver.cpp` `RelayVoteInfo`; `source/zclient.cpp` `ProcessVoteInfo`; `source/zplayer_events.cpp` `set_vote_info_event`.
- Rust owner: `src/network_commands.rs` owns `TcpEventId::VoteInfo` and packed `VoteInfoPacket`; `src/vote.rs` owns local relay/apply of vote state.
- Done: vote event ids around pause are covered as `START_VOTE=53`, `VOTE_YES=54`, `VOTE_NO=55`, `VOTE_PASS=56`, `VOTE_INFO=57`.
- Done: `VoteInfoPacket` now mirrors source `#pragma pack(1)` layout: one bool byte followed by `vote_type` and `value` as little-endian `i32`, for a 9-byte payload.
- Done: local vote state changes round-trip through `VOTE_INFO` encode/decode before applying client-visible vote state, matching `RelayVoteInfo -> ProcessVoteInfo` at state level.
- Tests: added packed packet layout coverage and invalid payload rejection.
- Known difference: this does not cover non-pause vote descriptions, exact font surface truncation, news strings, or real socket transport.
- Verification: `cargo check -q`, `cargo test -q` (444 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781783300001` rendered a non-black viewport after click (`nonblack_ratio=0.9996`, screenshot `output/playwright/vote-info-packet-smoke.png`).

## Completed slice: `SET_LPLAYER_VOTEINFO` player vote-choice path

- Source: `source/event_handler.h` `tcp_event` and packed `set_player_int_packet`; `source/zserver.cpp` `RelayLPlayerVoteChoice`, `VoteYes`, `VoteNo`, `VotePass`, `ClearPlayerVotes`; `source/zclient.cpp` `ProcessSetLPlayerVoteInfo`; `source/zplayer_events.cpp` `set_player_voteinfo_event`.
- Rust owner: `src/network_commands.rs` owns `TcpEventId::SetLocalPlayerVoteInfo` and `SetLocalPlayerVoteInfoPacket`; `src/vote.rs` owns local per-player vote-choice relay/apply.
- Done: `SET_LPLAYER_VOTEINFO=48` is represented with the source packed 8-byte payload: `p_id` then `value`, both little-endian `i32`.
- Done: `LocalVotePlayer` now carries source-style `p_id`, and local `VoteYes` relays the changed vote choice through `SetLocalPlayerVoteInfoPacket` before checking the vote result.
- Done: `ClearPlayerVotes` resets and relays each player choice, matching the source cleanup path.
- Done: client apply validates `0 <= value < P_MAX_VOTE_CHOICES`, finds players by `p_id`, and leaves unmatched ids as no-op.
- Tests: added packet layout coverage and client apply validation/matching coverage.
- Known difference: this does not handle tray UI, news strings, or real socket transport.
- Verification: `cargo check -q`, `cargo test -q` (446 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781784400001` rendered a non-black viewport after click (`nonblack_ratio=0.9999`, screenshot `output/playwright/vote-choice-packet-smoke.png`).

## Completed slice: vote expiration timer

- Source: `source/zvote.h` `MAX_VOTE_TIME`; `source/zvote.cpp` `ZVote::StartVote`/`TimeExpired`; `source/zserver.cpp` main loop `CheckVoteExpired`, `KillVote`, `ClearPlayerVotes`, `RelayVoteInfo`.
- Rust owner: `src/vote.rs` owns active vote elapsed time and expiration; `src/main.rs::process_vote_expiration` owns the Bevy runtime tick.
- Done: `GameVoteState` now tracks `elapsed_seconds`, reset to `0.0` when an active vote starts.
- Done: `tick_vote_expiration` mirrors `current_time() >= end_time` using `MAX_VOTE_TIME_SECONDS = 30.0`.
- Done: expiration runs after pause/vote request handling and before pause updates, matching the source loop shape where events process before `CheckVoteExpired`.
- Done: expired votes call the existing `kill_vote` path, which clears player choices and relays `VOTE_INFO` as inactive.
- Tests: added active vote expiration coverage for the 30-second boundary.
- Known difference: `BroadCastNews("vote has expired")`, news strings, and real socket transport remain separate slices.
- Verification: `cargo check -q`, `cargo test -q` (447 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781785200001` rendered a non-black viewport after click (`nonblack_ratio=0.9996`, screenshot `output/playwright/vote-expiration-smoke.png`).

## Completed slice: vote UI panel/count rendering

- Source: `source/zvote.cpp` `ZVote::Init`, `SetupImages`, `DoRender`; `source/zvote.h` `vote_type_string`; `source/zcore.cpp` `VotesNeeded`, `VotesFor`, `VotesAgainst`, `VoteAppendDescription`.
- Rust owner: `src/hud.rs` owns Bevy HUD entities/anchors/text update; `src/vote.rs::vote_display_snapshot` owns the source `SetupImages` inputs; `src/components.rs` owns HUD markers/assets/anchor.
- Done: `other/menus/vote_in_progress.png` is loaded and spawned as a hidden HUD panel.
- Done: `HudAnchor::ScreenTopRight` models the source top-right visible-view placement; the vote panel uses source `112x73`, right margin `4`, top margin `4`, and alpha `200/255`.
- Done: description/have/needed/for/against text entities use the source offsets `(57,41)`, `(57,53)`, `(57,64)`, `(22,64)`, `(91,64)` and yellow menu-style color/alpha.
- Done: pause/resume active votes display `Pause Game` / `Resume Game` and counts from local vote state: have, needed, for, against.
- Tests: added display snapshot coverage and HUD top-right/source-offset/text field coverage.
- Known difference: this slice does not implement non-pause vote descriptions, exact font surface width truncation, tray UI, news strings, or real socket transport.
- Verification: `cargo check -q`, `cargo test -q` (450 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781786400001` rendered a non-black viewport after click (`nonblack_ratio=0.9999`, screenshot `output/playwright/vote-ui-panel-smoke.png`).

## Completed slice: vote F1/F2/F3 input

- Source: `source/zplayer_events.cpp` keydown handling; `source/zplayer.cpp` `SendVoteYes`/`SendVoteNo`/`SendVotePass`; `source/zserver_events.cpp` `vote_yes_event`/`vote_no_event`/`vote_pass_event`; `source/zserver.cpp` `VoteYes`/`VoteNo`/`VotePass`/`CheckVote`.
- Rust owner: `src/main.rs::process_vote_choice_input` owns local keyboard input and packet round-trip; `src/network_commands.rs` owns empty vote command packets; `src/vote.rs::submit_vote_choice` owns local server-style vote choice handling.
- Done: `F1`, `F2`, and `F3` map to `VoteChoice::Yes`, `VoteChoice::No`, and `VoteChoice::Pass`, matching the source keyboard-only input path.
- Done: `VOTE_YES=54`, `VOTE_NO=55`, and `VOTE_PASS=56` encode as zero-payload command packets before local handling.
- Done: local `VoteYes`/`VoteNo`/`VotePass` reject inactive, duplicate, or non-votable input; set `P_YES_VOTE`/`P_NO_VOTE`/`P_PASS_VOTE`; relay player vote choice; run `CheckVote`; and relay inactive `VOTE_INFO` when `KillVote` happens.
- Done: `VoteNo` can kill a vote through against majority, and `VotePass` can lower `VotesNeeded` enough for existing yes votes to pass, matching source `CheckVote` after every choice.
- Tests: added empty packet coverage and vote no/pass behavior coverage.
- Known difference: source news strings were moved into the next local news-log slice; real socket transport remains separate.
- Verification: `cargo check -q`, `cargo test -q` (453 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781787600001` rendered a non-black viewport after click (`nonblack_ratio=0.9999`, screenshot `output/playwright/vote-input-smoke.png`).

## Completed slice: vote news strings/local HUD log

- Source: `source/zserver.cpp` `VoteYes`/`VoteNo`/`VotePass`/`CheckVoteExpired`/`BroadCastNews`; `source/zplayer.cpp` `AddNewsEntry`/`RenderNews`; `source/zplayer_events.cpp` `display_news_event`.
- Rust owner: `src/news.rs` owns local news entries, source black-color adjustment, lifetime, fade, and history cap; `src/vote.rs::submit_vote_choice` owns vote outcome news intents; `src/main.rs::{process_vote_choice_input,process_vote_expiration}` routes local news into the log; `src/hud.rs` owns bottom-left news rendering.
- Done: vote choices now emit exact source strings for success, duplicate vote, and vote-login rejection: `"you have voted yes"`, `"you have voted no"`, `"you have passed on voting"`, `"you have already voted"`, and `"you must be logged in to vote, please type /help"`.
- Done: expired votes now add the source `"vote has expired"` news message after `KillVote`/inactive `VOTE_INFO` relay.
- Done: `NewsLog` mirrors source `AddNewsEntry` basics: newest-first insert, max history 50, 17 second lifetime, last-5-second fade, and `(0,0,0)` color becoming `(1,0,0)`.
- Done: HUD news text uses source bottom-left stack position: x `5`, first row top at `height - 51`, row gap `15`, hidden after expiry.
- Tests: added news owner coverage, vote news string coverage, and HUD bottom-left anchor coverage.
- Known difference: `NEWS_EVENT` and vote-start broadcast strings were moved into the next local packet-relay slice; `SEND_CHAT`, chat history toggle, `/pause` and `/resume` chat commands, factory-list x-shift, and real socket transport remain separate slices.
- Verification: `cargo check -q`, `cargo test -q` (458 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781789200001` rendered a non-black viewport after click (`nonblack_ratio=0.5062`, screenshot `output/playwright/vote-news-smoke.png`).

## Completed slice: `NEWS_EVENT` packet/local relay and vote-start broadcast

- Source: `source/event_handler.h` `NEWS_EVENT=10`; `source/zserver.cpp` `SendNews`/`BroadCastNews`/`StartVote`; `source/zplayer_events.cpp` `display_news_event`; `source/zplayer.cpp` `AddNewsEntry`.
- Rust owner: `src/network_commands.rs` owns `TcpEventId::NewsEvent` and `NewsEventPacket`; `src/news.rs` owns local `NEWS_EVENT` round-trip/apply; `src/vote.rs::pause_vote_outcome_for_request` owns source-style start-vote news outcome; `src/main.rs::process_game_pause_requests` routes it into `NewsLog`.
- Done: `NEWS_EVENT` uses the source hand-built payload layout: `r`, `g`, `b`, then nul-terminated message bytes inside the existing `i32 payload_len + i32 event_id` envelope.
- Done: local news additions now pass through `NewsEventPacket` encode/decode before entering `NewsLog`, preserving the source client guard that ignores payloads with `size <= 5`.
- Done: pause/resume vote creation now emits source-style start broadcast strings when a new vote actually starts: `vote started by {player} to {VoteType}`; login rejection uses `you must be logged in to start a vote, please type /help`.
- Done: active same-vote pause/resume requests reuse the vote-yes news path, matching source `StartVote` calling `VoteYes(player)` for the same active vote.
- Done: stale runtime-only update wrappers were moved behind `#[cfg(test)]` after the dead_code audit.
- Tests: added `NEWS_EVENT` id/layout/decode coverage, news relay coverage, and pause vote-start/login broadcast coverage.
- Known difference: this is still local packet relay only; `SEND_CHAT`, chat history toggle, `/pause` and `/resume` chat commands, non-pause vote append descriptions, real socket transport, factory-list x-shift, and local-player/team abstraction remain separate slices.
- Verification: `cargo check -q`, `cargo test -q` (463 passed), `./scripts/build-wasm.sh`, browser smoke on `http://127.0.0.1:4173/?canvas_fix=1781790600001` rendered a non-black viewport after click (`nonblack_ratio=0.5062`, screenshot `output/playwright/news-event-start-vote-smoke.png`).

## Completed slice: `SEND_CHAT` packet/local relay and HUD chat draft

- Source: `source/event_handler.h` `SEND_CHAT=27`; `source/socket_handler.cpp` ASCII message envelope; `source/zplayer.cpp` `ProcessUnicode`; `source/zserver_events.cpp` `relay_chat_event`; `source/zserver_commands.cpp` `ProcessPlayerCommand`; `source/zhud.cpp` chat draft render/history toggle.
- Rust owner: `src/network_commands.rs` owns `TcpEventId::SendChat` and `SendChatCommand`; `src/chat.rs` owns local chat input state, local relay and slash command dispatch; `src/news.rs` owns chat history visibility; `src/hud.rs` owns `Say::` chat draft display; `src/main.rs::process_chat_input` wires keyboard input and pause command handoff.
- Done: `SEND_CHAT` uses source id `27` and the source string payload shape: message bytes plus final nul inside the shared `i32 payload_len + i32 event_id` envelope.
- Done: local chat submit round-trips through `SendChatCommand` encode/decode before relay, then normal messages broadcast as `{player}:: {message}` through the local `NEWS_EVENT` path with source team-color dimming.
- Done: `Enter` toggles chat collection/submission, text input appends while collecting, Backspace removes the last char, and `/` outside collection starts a command draft with `/`, matching `ProcessUnicode`.
- Done: HUD chat draft renders `Say:: {message}` at source position `chat_start_x + 3`, `off_y + 460 + 5`, represented as bottom-left `(209, 19)`; long drafts keep the visible tail to approximate source left clipping.
- Done: `H` outside chat collection toggles chat history visibility; expired news entries render without fade while history is visible.
- Done: local `/pause` and `/resume` commands feed the existing source-style pause vote/request path; unknown commands emit `command not found, please type /help or /listcommands`.
- Tests: added packet id/layout/decode coverage, chat edit/command/relay coverage, source red chat color coverage, chat history coverage, and HUD chat draft position/prefix/tail coverage.
- Known difference: this remains a local relay with no real socket transport, no full `ProcessPlayerCommand` surface (login, map/bot/team/reset commands, etc.), and a placeholder local player name/team abstraction. `SendChatCommand` is stricter than the source's unfinished ASCII check because it rejects invalid UTF-8 and interior nul bytes.
- Verification: `cargo check -q`, `cargo test -q` (473 passed), `./scripts/build-wasm.sh`, Playwright DOM/canvas smoke on `http://127.0.0.1:4173/?canvas_fix=1781867000001` loaded `Zod Rust Bevy Port` with one 800x600 canvas; console had only the existing wasm-bindgen init warning and favicon 404.

## Completed slice: `ProcessPlayerCommand` help/listcommands output

- Source: `source/zserver_commands.cpp` `ProcessPlayerCommand`, `PlayerCommand_Help`, `PlayerCommand_ListCommands`, `PlayerCommand_NotFound`.
- Rust owner: `src/chat.rs` owns command parsing and local command news emission through the existing `NEWS_EVENT` relay path.
- Done: slash command parsing now preserves the source split shape: command name before the first space and raw contents after that first space.
- Done: `/listcommands` emits the two exact source command-list news lines.
- Done: `/help` and `/help help` emit source help usage/purpose plus the command list; `/help <known-command>` emits the exact source usage/purpose pair for `listcommands`, login/logout/createuser, pause/resume, map/bot/player/team/speed/version commands.
- Done: `/help <unknown>` remains a source-style no-op, while unknown top-level commands still emit `command not found, please type /help or /listcommands`.
- Tests: added command-list ordering coverage, generic help coverage, specific help coverage, and unknown-help no-op coverage.
- Known difference: this slice only ports news-only command output. Login/logout/createuser, changemap, bot/team/reset side effects and their server state remain separate source slices.
- Verification: `cargo check -q`, `cargo test -q` (477 passed), `./scripts/build-wasm.sh`, Playwright DOM/canvas smoke on `http://127.0.0.1:4173/?canvas_fix=1781869000001` loaded `Zod Rust Bevy Port` with one 800x600 canvas; console had only favicon, wasm-bindgen init and browser audio-autoplay warnings.

## Completed slice: `ProcessPlayerCommand` playerinfo/currentmap output

- Source: `source/zserver_commands.cpp` `PlayerCommand_PlayerInfo` and `PlayerCommand_CurrentMap`; `source/zserver_events.cpp` local player info relay shape; `source/zmap.h`/`source/zmap.cpp` map name basics.
- Rust owner: `src/chat.rs` owns `ChatCommandContext` and command news emission; `src/main.rs::process_chat_input` builds the context from `ChatInputState` plus `CurrentMap`.
- Done: `/playerinfo` emits the source message sequence for local player name, team, and logged-in state. The default runtime path is the current honest local state: `Player`, Red team, not logged in.
- Done: the logged-in branch is represented in `ChatCommandContext` and mirrors the source output sequence, including the original bug where activated prints `yes` in both branches.
- Done: `/currentmap` emits `current map: {map_name}` using the parsed `ZMap.basics.map_name`, not a hardcoded Rust label.
- Done: normal chat relay now reads player name/team from the same command context used by slash commands, preparing the later local-player packet abstraction.
- Tests: added not-logged-in playerinfo coverage, logged-in playerinfo coverage, and currentmap coverage.
- Known difference: login/logout/createuser, server-side player id/loginfo packets, map list/change-map vote path, and `/version` packet relay remain separate slices.
- Verification: `cargo check -q`, `cargo test -q` (480 passed), `./scripts/build-wasm.sh`, Playwright DOM/canvas smoke on `http://127.0.0.1:4173/?canvas_fix=1781870600001` loaded `Zod Rust Bevy Port` with one 800x600 canvas; console had only favicon, wasm-bindgen init and browser audio-autoplay warnings.

## Completed slice: `ProcessPlayerCommand` version packet/output

- Source: `source/zserver_commands.cpp` `PlayerCommand_Version`; `source/zserver.cpp` `RelayVersion`; `source/zplayer_events.cpp` `get_version_event`; `source/event_handler.h` `version_packet`; `source/constants.h` `GAME_VERSION`.
- Rust owner: `src/network_commands.rs` owns `TcpEventId::GiveVersion`, `GiveVersionPacket`, and source `GAME_VERSION`; `src/chat.rs` owns `/version` local relay and client news output.
- Done: `GIVE_VERSION=85` is represented with the source fixed `char version[50]` payload inside the shared packet envelope.
- Done: source `GAME_VERSION` is represented as `2011-09-06`, matching `source/constants.h`.
- Done: `/version` now runs through a local `GiveVersionPacket` encode/decode before adding client-local news.
- Done: the client output matches both source branches: `the server version is {version}` for equal versions and `the server version is {server}, which mismatches our version {local}` for mismatch.
- Tests: added event id coverage, fixed char-array packet layout/decode coverage, `/version` output coverage, and mismatch-branch coverage.
- Known difference: startup `REQUEST_VERSION` handshake and real socket routing remain separate network slices; this slice models the `/version` command path that directly relays `GIVE_VERSION`.
- Verification: `cargo check -q`, `cargo test -q` (484 passed), `./scripts/build-wasm.sh`, Playwright DOM/canvas smoke on `http://127.0.0.1:4173/?canvas_fix=1781871800001` loaded `Zod Rust Bevy Port` with one 800x600 canvas; console had only favicon, wasm-bindgen init and browser audio-autoplay warnings.

## Completed slice: startup `REQUEST_VERSION` handshake

- Source: `source/zclient.cpp` `ZClient::ProcessConnect`; `source/zserver_events.cpp` `request_version_event`; `source/zserver.cpp` `RelayVersion`; `source/zplayer_events.cpp` `get_version_event`.
- Rust owner: `src/version.rs` owns the local version request relay and client apply; `src/network_commands.rs` owns `REQUEST_VERSION=84` and `GIVE_VERSION=85`; `src/main.rs` schedules the startup query.
- Done: startup now sends a local empty `REQUEST_VERSION` envelope before the pause query, matching the original `ProcessConnect` order.
- Done: local server handling validates the empty request payload shape and responds through the existing `GIVE_VERSION` packet relay.
- Done: client apply is shared by startup handshake and `/version`, so both paths emit the same source version news and mismatch branch.
- Done: version code was moved out of `src/chat.rs` into `src/version.rs`; chat only invokes the shared `/version` relay.
- Tests: added startup request relay coverage and kept packet/client mismatch coverage under the version owner.
- Known difference: this is still local packet relay; real socket transport and the rest of `ProcessConnect` after game speed (settings, player info/list, selectable maps, map request) remain separate slices.
- Verification: `cargo check -q`, `cargo test -q` (485 passed), `./scripts/build-wasm.sh`, Playwright DOM/canvas smoke on `http://127.0.0.1:4173/?canvas_fix=1781873300001` loaded `Zod Rust Bevy Port` with one 800x600 canvas; console had only favicon, wasm-bindgen init and browser audio-autoplay warnings.

## Completed slice: startup `GET_GAME_SPEED` handshake

- Source: `source/zclient.cpp` `ZClient::ProcessConnect`; `source/zserver_events.cpp` `get_game_speed_event`; `source/zserver.cpp` `RelayGameSpeed`; `source/zclient.cpp` `ProcessUpdateGameSpeed`; `source/ztime.cpp` `ZTime::SetGameSpeed`.
- Rust owner: `src/game_speed.rs` owns local game-speed request relay and client state apply; `src/network_commands.rs` owns `GET_GAME_SPEED=73` and `UPDATE_GAME_SPEED=75`; `src/main.rs` schedules the startup query after pause query.
- Done: startup now sends a local empty `GET_GAME_SPEED` envelope after `GET_GAME_PAUSED`, matching the original `ProcessConnect` order.
- Done: local server handling validates the empty request payload and responds through `UPDATE_GAME_SPEED` with the source `float_packet` payload layout.
- Done: `GameSpeedState` defaults to `1.0`, matching `ZTime::ZTime`, and client apply clamps negative speeds to `0.0`, matching `ZTime::SetGameSpeed`.
- Done: this slice stores the authoritative local speed state only; it does not silently scale Bevy simulation time.
- Tests: added event id coverage, `UPDATE_GAME_SPEED` float packet layout/decode coverage, default state coverage, request relay coverage, and negative-speed clamp coverage.
- Known difference: full ztime-scaled simulation/effects remain separate.
- Verification: `cargo check -q`, `cargo test -q` (489 passed), `./scripts/build-wasm.sh`, Playwright DOM/canvas smoke on `http://127.0.0.1:4173/?canvas_fix=1781874700001` loaded `Zod Rust Bevy Port` with one 800x600 canvas; console had only favicon, wasm-bindgen init and browser audio-autoplay warnings.

## Completed slice: startup `SendPlayerInfo` local-player packet path

- Source: `source/zclient.cpp` `ZClient::ProcessConnect`, `SendPlayerInfo`, `SendPlayerTeam`, `SendPlayerMode`; `source/zserver_events.cpp` `set_player_name_event`, `set_player_team_event`, `set_player_mode_event`; `source/zserver.cpp` `RelayLPlayerName`, `RelayLPlayerTeam`, `RelayLPlayerMode`; `source/zclient.cpp` `ProcessSetLPlayerName`, `ProcessSetLPlayerTeam`, `ProcessSetLPlayerMode`; `source/event_handler.h` player packet ids/layouts.
- Rust owner: `src/local_player.rs` owns local player state, source player mode values, startup `SendPlayerInfo` relay/apply, and current player info used by chat; `src/network_commands.rs` owns `SET_NAME=8`, `SET_TEAM=9`, `SET_PLAYER_MODE=38`, `SET_LPLAYER_NAME=43`, `SET_LPLAYER_TEAM=44`, and `SET_LPLAYER_MODE=45`; `src/chat.rs` reads local player info through `ChatCommandContext`.
- Runtime call site: `src/main.rs` schedules `process_initial_player_info_send` after startup game-speed query and before chat input, mirroring the source `ProcessConnect` ordering around `SendPlayerInfo` while `REQUEST_SETTINGS` remains unported.
- Done: startup now sends local `SET_NAME`, `SET_TEAM`, and `SET_PLAYER_MODE` envelopes, decodes them with source payload guards, applies server-side player state, then relays the corresponding `SET_LPLAYER_*` packets back into the local client state.
- Done: `LocalPlayerState` separates source client intent (`player_name`, `desired_team`, `our_mode`) from server/list state (`name`, `team`, `mode`), so `/playerinfo` and normal chat no longer carry their own placeholder player fields in `ChatInputState`.
- Done: `SET_NAME` accepts source-style nul-terminated ASCII payloads and truncates at `MAX_PLAYER_NAME_SIZE=30`; `SET_TEAM` and `SET_PLAYER_MODE` reject invalid wire values like the source server/client guards.
- Tests: added packet id/layout/decode coverage for `SET_NAME`, `SET_TEAM`, `SET_PLAYER_MODE`, `SET_LPLAYER_NAME`, `SET_LPLAYER_TEAM`, `SET_LPLAYER_MODE`, plus local player startup relay, name truncation, and invalid team/mode guards.
- Known difference: this remains local single-player relay; real socket transport, `REQUEST_SETTINGS`, `REQUEST_PLAYER_ID`, `REQUEST_PLAYER_LIST`, `CLEAR_PLAYER_LIST`/`ADD_LPLAYER`/delete/loginfo/ignored, selectable map list, and map request are still separate `ProcessConnect` slices. `ZPlayer` CLI/default-name handling is not fully modeled; the browser runtime keeps the existing local `Player`/Red default.
- Verification: `cargo check -q`, `cargo test -q` (495 passed), `./scripts/build-wasm.sh`, Playwright screenshot smoke on `http://127.0.0.1:4173/?canvas_fix=1781880000001` rendered the game viewport (screenshot `output/playwright/player-info-smoke.png`); console only had favicon 404 and the existing wasm-bindgen init warning.

## Completed slice: startup `RequestPlayerList` local roster path

- Source: `source/zclient.cpp` `ZClient::RequestPlayerList` and `ProcessPlayerID`; `source/zserver_events.cpp` `request_player_list_event`; `source/zserver.cpp` `RelayLAdd`, `RelayLPlayerName`, `RelayLPlayerTeam`, `RelayLPlayerMode`; `source/zclient.cpp` `ProcessAddLPlayer`, `ProcessSetLPlayerName`, `ProcessSetLPlayerTeam`, `ProcessSetLPlayerMode`; `source/event_handler.h` player id/list packet ids/layouts.
- Rust owner: `src/local_player.rs` owns `LocalPlayerInfo` roster, startup `RequestPlayerList` relay/apply, and current `OurPInfo`-style lookup for chat/playerinfo; `src/network_commands.rs` owns `REQUEST_PLAYER_LIST=39`, `CLEAR_PLAYER_LIST=40`, `ADD_LPLAYER=41`, `GIVE_PLAYER_ID=58`, and `REQUEST_PLAYER_ID=59`.
- Runtime call site: `src/main.rs` schedules `process_initial_player_list_request` immediately after `process_initial_player_info_send` and before chat input, matching `ZClient::ProcessConnect` order after `SendPlayerInfo`.
- Done: startup now sends empty `REQUEST_PLAYER_ID` and receives `GIVE_PLAYER_ID`, then sends empty `REQUEST_PLAYER_LIST`, clears the local roster, adds the local player through `ADD_LPLAYER`, and applies the already source-backed `SET_LPLAYER_NAME`/`TEAM`/`MODE` relay into the roster.
- Done: `LocalPlayerState::name()` and `team()` now prefer the current roster entry when present, falling back to server-side fields and then the original browser local defaults.
- Done: packet guards match source shape for exact 4-byte `player_id_packet` / `add_remove_player_packet` and empty clear/request payloads.
- Tests: added packet id/layout/decode coverage for request/id/clear/add packets, local roster request coverage, clear/add vector-shape coverage, and player-id apply coverage.
- Known difference: this still models a single local server/client path. `DELETE_LPLAYER`, `SET_LPLAYER_IGNORED`, `SET_LPLAYER_LOGINFO`, full player list UI, real socket transport, and the rest of `ProcessConnect` (`REQUEST_SETTINGS`, selectable maps, map request) remain separate slices.
- Verification: `cargo check -q`, `cargo test -q` (500 passed), `./scripts/build-wasm.sh`, Playwright screenshot smoke on `http://127.0.0.1:4173/?canvas_fix=1781883200001` rendered the game viewport (screenshot `output/playwright/player-list-smoke.png`).

## Completed slice: player roster delete/ignored/loginfo packets

- Source: `source/zserver_events.cpp` `disconnect_event`; `source/zserver.cpp` `RelayLPlayerIgnored`, `RelayLPlayerLoginInfo`; `source/zclient.cpp` `ProcessDeleteLPlayer`, `ProcessSetLPlayerIgnored`, `ProcessSetLPlayerLogInfo`; `source/event_handler.h` `add_remove_player_packet`, `set_player_int_packet`, `set_player_loginfo_packet`.
- Rust owner: `src/network_commands.rs` owns `DELETE_LPLAYER=42`, `SET_LPLAYER_IGNORED=46`, and `SET_LPLAYER_LOGINFO=47`; `src/local_player.rs` owns roster delete/ignored/loginfo apply and the runtime queue bridge for incoming delete packets.
- Runtime call site: `src/main.rs` schedules `process_local_player_packet_queue` in the startup/network chain after player-list request and before chat input; startup list relay also applies ignored/loginfo after name/team/mode.
- Done: `DELETE_LPLAYER` uses the source `add_remove_player_packet` 4-byte `p_id` payload and removes every matching roster entry, matching the source erase loop.
- Done: `SET_LPLAYER_IGNORED` uses the source `p_id,value` layout and preserves the original client guard (`0 <= value < MAX_PLAYER_MODES`), treating nonzero values as ignored.
- Done: `SET_LPLAYER_LOGINFO` uses packed source layout `int p_id, int db_id, int voting_power, int total_games, bool activated, bool logged_in, bool bot_logged_in` (`19` bytes under `#pragma pack(1)`), and local `/playerinfo` getters now prefer roster loginfo for logged-in, activated, voting power and real voting power.
- Done: delete packet handling is runtime-wired through `LocalPlayerPacketQueue` instead of being left as test-only packet layout.
- Tests: added id/layout/decode coverage for delete/ignored/loginfo, source boolean guard coverage, roster delete coverage, ignored guard coverage, and loginfo-backed playerinfo getter coverage.
- Known difference: real socket receive is still not modeled, and full player-list UI remains separate.
- Verification: `cargo check -q`, `cargo test -q` (504 passed), `./scripts/build-wasm.sh`, Playwright screenshot smoke on `http://127.0.0.1:4173/?canvas_fix=1781886200001` rendered the game viewport (screenshot `output/playwright/player-roster-loginfo-smoke.png`).

## Completed slice: startup `REQUEST_SELECTABLE_MAP_LIST` and `/listmaps`

- Source: `source/zclient.cpp` `ZClient::ProcessConnect`; `source/zserver_events.cpp` `request_selectable_map_list_event`; `source/zserver.cpp` `ReadSelectableMapListFromFolder` and `GivePlayerSelectableMapList`; `source/zclient.cpp` `ProcessSelectableMapList`; `source/zserver_commands.cpp` `PlayerCommand_ListMaps`; `source/event_handler.h` packet ids.
- Rust owner: `src/selectable_maps.rs` owns generated source map-list state, startup local request relay, and client apply; `src/network_commands.rs` owns `REQUEST_SELECTABLE_MAP_LIST=60`, `GIVE_SELECTABLE_MAP_LIST=61`, `RequestSelectableMapListCommand`, and `GiveSelectableMapListPacket`; `src/chat.rs` owns `/listmaps` source-style news output.
- Runtime call site: `src/main.rs` schedules `process_initial_selectable_map_list_request` after player-list startup and before chat input, matching the original `ProcessConnect` order before `REQUEST_MAP`.
- Done: startup now sends an empty `REQUEST_SELECTABLE_MAP_LIST`, validates the request payload, builds the server response from the project `maps/*.map` list generated at build time, and applies `GIVE_SELECTABLE_MAP_LIST` back into local client state.
- Done: `GIVE_SELECTABLE_MAP_LIST` uses the source comma-joined nul-terminated string payload for non-empty lists and empty payload for empty lists; decode mirrors the source split loop for leading/consecutive separators and the 500-byte temporary buffer truncation.
- Done: `/listmaps` now emits source-formatted `map list: {i}. {name}` news, grouped four entries per line and silent for an empty selectable list.
- Tests: added packet id/layout/decode coverage, source split-loop edge coverage, generated map-list startup relay coverage, client-list replacement coverage, and `/listmaps` grouping/no-op coverage.
- Known difference: the selectable list is generated from checked-in `maps/*.map` at Rust build time instead of reading `psettings.selectable_map_list` or scanning an arbitrary runtime server folder. Real socket transport, `REQUEST_SETTINGS`, `REQUEST_MAP`, `/changemap` vote path, and the map-select GUI remain separate slices.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (510 passed), `./scripts/build-wasm.sh`.

## Completed slice: startup `REQUEST_MAP` / `STORE_MAP` transfer path

- Source: `source/zclient.cpp` `ZClient::ProcessConnect` and `ProcessMapDownload`; `source/zserver_events.cpp` `request_map_event`; `source/zplayer_events.cpp` `store_map_event`; `source/event_handler.h` map packet ids.
- Rust owner: `src/network_commands.rs` owns `REQUEST_MAP=1`, `STORE_MAP=3`, `RequestMapCommand`, and `StoreMapPacket`; `src/map_transfer.rs` owns source-sized 4096-byte map chunks, client download accumulation, final `pack_num=-1` completion, and local request relay.
- Runtime call site: `src/main.rs::load_original_map` now obtains the current `STARTING_MAP`/`ZOD_MAP` bytes through the local `REQUEST_MAP` -> `STORE_MAP` chunk relay before `ZMap::parse`, preserving the existing Bevy startup map spawn while replacing direct parse-only loading with the source transfer shape.
- Done: `REQUEST_MAP` uses the source empty payload request and `STORE_MAP` uses the source `int pack_num` prefix plus raw map bytes, including the final 4-byte `-1` packet.
- Done: client apply resets the accumulated map buffer on packet `0`, appends later chunks, and completes on `-1`, matching the original `ProcessMapDownload` shape.
- Tests: added map event id/layout/decode coverage, chunk reassembly coverage across the 4096-byte boundary, final-packet completion coverage, and first-packet buffer reset coverage.
- Known difference: the original client asks for the map after selectable maps during socket `ProcessConnect`; the Rust Bevy app still needs a map before `spawn_map`, so the source-style transfer wraps the startup loader rather than becoming a later Update-system download. Real socket transport, post-map `REQUEST_OBJECTS`/`REQUEST_ZONES`, full client map reset flow, and runtime map switching remain separate slices.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (514 passed), `./scripts/build-wasm.sh`.

## Completed slice: startup `REQUEST_SETTINGS` / `SET_SETTINGS` handshake

- Source: `source/zclient.cpp` `ZClient::ProcessConnect` and `ProcessZSettings`; `source/zserver_events.cpp` `request_settings_event`; `source/zsettings.h` packed `ZSettings` / `ZUnit_Settings`; `source/zsettings.cpp` `SetDefaults`; `source/event_handler.h` packet ids.
- Rust owner: `src/network_commands.rs` owns `REQUEST_SETTINGS=33`, `SET_SETTINGS=34`, `RequestSettingsCommand`, `SetSettingsPacket`, and the packed source payload size `1420`; `src/settings_sync.rs` owns source-default `ZSettings` byte construction, local request relay, and client byte apply.
- Runtime call site: `src/main.rs` schedules `process_initial_settings_request` after `GET_GAME_SPEED` and before `SendPlayerInfo`, matching source `ProcessConnect`.
- Done: startup now sends an empty `REQUEST_SETTINGS`, validates it, locally produces a source-sized raw `SET_SETTINGS` payload from the current source-backed unit/building/item defaults, encodes/decodes it, and stores the client settings bytes in `SourceSettingsState`.
- Done: `SET_SETTINGS` preserves the source raw `sizeof(ZSettings)` model instead of inventing a JSON/serde schema; `ZUnit_Settings` is encoded in source field order with packed `i32`/`double` layout.
- Correction: this slice originally documented the ids as `36/37`; the later `DRIVER_HIT_EFFECT` slice corrected the adjacent source enum range to `REQUEST_SETTINGS=33`, `SET_SETTINGS=34`, `SET_LID_OPEN=35`, `SNIPE_OBJECT=36`, `DRIVER_HIT_EFFECT=37`.
- Tests: added settings packet id/layout/size/decode coverage, source `ZSettings` payload size/first-unit field coverage, request round-trip coverage, and client byte replacement coverage.
- Known difference: the runtime still reads gameplay stats through existing unit owners (`src/units/**`) rather than dynamically looking up values from `SourceSettingsState`; this preserves the current Bevy gameplay while establishing the source handshake. Loading a custom `zsettings` text file and applying server-modified settings into all stat call sites remain separate slices.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (518 passed), `./scripts/build-wasm.sh`.

## Completed slice: post-map `REQUEST_ZONES` / `SET_ZONE_INFO` ownership relay

- Source: `source/zclient.cpp` `ProcessMapDownload`, `RequestZoneList`, and `ProcessZoneInfo`; `source/zserver_events.cpp` `send_zone_info_list_event`; `source/zserver.cpp` `AwardZone`; `source/event_handler.h` packet ids/layout order.
- Rust owner: `src/network_commands.rs` owns `REQUEST_ZONES=5`, `SET_ZONE_INFO=7`, `RequestZonesCommand`, and `SetZoneInfoPacket`; `src/zone_sync.rs` owns source-style local zone-list request relay and client-side ownership apply.
- Runtime call site: `src/main.rs::spawn_map` now gets `ZoneOwnership` through `relay_request_zone_ownership(&CurrentMap)` before spawning HUD/minimap and inserting the resource, instead of reading ownership directly into runtime state.
- Done: zone sync sends the source empty `REQUEST_ZONES` payload and applies one packed 5-byte `int zone_number, char owner` packet per map zone, preserving the existing `ZoneLink` data while replacing owners through packet apply.
- Done: client apply mirrors source guards for negative owner/zone ids, out-of-range zone index, and invalid team values before mutating `ZoneOwnership`.
- Tests: added zone packet id/layout/decode coverage, request packet coverage, map-zone owner relay coverage, and invalid apply guard coverage.
- Known difference: the source requests zones after the map download completes and then after object list request; Rust still computes zone links from the already parsed map during startup. Real socket transport, post-map `REQUEST_OBJECTS`, runtime `AwardZone` network broadcast parity, and full object-team reset packet effects remain separate slices.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (521 passed), `./scripts/build-wasm.sh`.

## Completed slice: post-map `REQUEST_OBJECTS` / `ADD_NEW_OBJECT` object-init relay

- Source: `source/zclient.cpp` `ProcessMapDownload`, `RequestObjectList`, and `ProcessNewObject`; `source/zserver_events.cpp` `send_object_list_event`; `source/zserver.cpp` `InitObjects`, `CreateRobotGroup`, and `RelayNewObject`; `source/event_handler.h` packed `object_init_packet`.
- Rust owner: `src/network_commands.rs` owns `REQUEST_OBJECTS=4`, `ADD_NEW_OBJECT=6`, `RequestObjectsCommand`, and `ObjectInitPacket`; `src/object_sync.rs` owns source-style local object-list request relay, object-init packet generation, client-side object-init validation, and ref-id sequence calculation.
- Runtime call site: `src/main.rs::spawn_map` now runs `relay_request_object_inits(&CurrentMap)` before `spawn_objects` and asserts the source object-init ref-id sequence matches the current Bevy spawn sequence.
- Done: `ADD_NEW_OBJECT` uses source packed layout `int x, int y, int ref_id, char owner, unsigned char object_type, unsigned char object_id, char blevel, unsigned short extra_links, int health` (`22` bytes).
- Done: server-side map object relay emits source pixel coordinates (`tile * 16`), source owner/type/id/level/extra_links/health fields, and source-style robot group expansion count so generated ref ids line up with current runtime entities.
- Done: client-side object-init validation rejects negative coordinates/ref ids, invalid owner values, and invalid object type ids before accepting the decoded packet.
- Tests: added request packet coverage, object-init id/layout/decode coverage, map object-init relay/ref-id coverage, and invalid wire guard coverage.
- Known difference: Rust still spawns from `ZMap` after the object-init relay instead of constructing every entity directly from `ObjectInitPacket`, because source robot grouping also requires `OBJECT_GROUP_INFO` packets. This slice establishes packet/ref-id parity and gates the current spawn sequence; full object-list construction from packets, `OBJECT_GROUP_INFO`, initial health/building/grenade/rally follow-up packets, and real socket transport remain separate slices.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (524 passed), `./scripts/build-wasm.sh`.

## Completed slice: `OBJECT_GROUP_INFO` robot group metadata relay

- Source: `source/zserver.cpp` `RelayNewObject` and `RelayObjectGroupInfo`; `source/zobject.cpp` `CreateGroupInfoData` and `ProcessGroupInfoData`; `source/zplayer_events.cpp` `set_object_group_info_event`; `source/event_handler.h` packet id order.
- Rust owner: `src/network_commands.rs` owns `OBJECT_GROUP_INFO=29` and `ObjectGroupInfoPacket`; `src/object_sync.rs` owns source-style group-info relay/validation for the object-init stream.
- Runtime call site: `src/main.rs::spawn_map` now runs `relay_object_group_infos(&CurrentMap, &object_inits)` immediately after `REQUEST_OBJECTS` / `ADD_NEW_OBJECT` relay and asserts robot object-init streams are followed by group metadata.
- Done: `OBJECT_GROUP_INFO` uses the client-expected variable layout `int ref_id, int leader_ref_id, int minions, int[minions] minion_refs`, with source-style `leader_ref_id=-1` for leaders.
- Done: group relay emits leader packets with minion refs and minion packets pointing back to their leader for the current map-derived robot groups; validation rejects unknown object, leader, or minion refs.
- Tests: added packet id/layout/decode coverage, group metadata relay coverage for robot groups, and unknown-ref validation coverage.
- Known difference: the C source server writes minion refs at `((int*)data)[2+i]`, which overwrites the `minions` field for leaders, while the C client expects the count at `[2]`. Rust currently encodes the client-expected logical layout to preserve existing playable robot group behavior. Exact replication or compatibility handling of that source indexing bug remains a separate audit item before full socket parity.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (527 passed), `./scripts/build-wasm.sh`.

## Completed slice: object-list `UPDATE_HEALTH` relay

- Source: `source/event_handler.h` `object_health_packet`; `source/zserver.cpp` `RelayObjectHealth` and `UpdateObjectHealth`; `source/zclient.cpp` `ProcessObjectHealthTeam`; `source/zobject.cpp` `SetHealthPercent` and `SetHealth`.
- Rust owner: `src/network_commands.rs` owns `UPDATE_HEALTH=17` and `ObjectHealthPacket`; `src/object_sync.rs` owns source-style health packet relay/validation and map health-percent to source actual-health conversion.
- Runtime call site: `src/main.rs::spawn_map` now runs `relay_object_health_updates(&object_inits)` after `ADD_NEW_OBJECT` and `OBJECT_GROUP_INFO`, before the current Bevy object spawn, and asserts every object-init packet is followed by a health packet.
- Done: `UPDATE_HEALTH` uses source layout `int ref_id, int health` (`8` bytes) inside the existing packet envelope.
- Done: map object init health now carries source actual health (`health_percent * max_health / 100`, clamped to `0..100`) instead of storing the map percent in the packet's `health` field.
- Done: client-side health apply validation mirrors the source object lookup guard by rejecting packets for unknown `ref_id`s.
- Tests: added packet id/layout/decode coverage, object-list health relay coverage, unknown-ref validation coverage, and source health-percent clamp coverage.
- Known difference: Rust still spawns runtime `ObjectStats` from the parsed `ZMap` path; this slice gates the source health packet stream and packet actual-health values before spawn. Later slices wire the same packets into live `ObjectStats`, revive/passability side effects, and hit-effect visual handling.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (531 passed), `./scripts/build-wasm.sh`.

## Completed slice: dynamic `UPDATE_HEALTH` apply path

- Source: `source/zclient.cpp` `ProcessObjectHealthTeam`; `source/zobject.cpp` `SetHealth`; `source/zserver.cpp` `UpdateObjectHealth`.
- Rust owner: `src/object_sync.rs` owns `ObjectHealthPacketQueue`, live packet apply, and source `SetHealth` clamp semantics for `ObjectStats`; `src/network_commands.rs` continues to own the packet layout.
- Runtime call site: `src/main.rs` inserts `ObjectHealthPacketQueue`, seeds it from startup object-list health packets after `spawn_objects`, and schedules `process_object_health_packet_queue` after local missile damage and before building production / destroyed-object lifecycle processing.
- Done: incoming `ObjectHealthPacket` entries now look up live objects by `ref_id` and apply source `SetHealth` clamp (`0..max_health`) into `ObjectStats.health`.
- Done: negative packet refs and unknown refs are ignored, preserving the source `GetObjectFromID` guard before `SetHealth`.
- Done: startup `UPDATE_HEALTH` packets now pass through the same live apply queue used for later dynamic packets instead of remaining only a pre-spawn parity gate.
- Tests: added owner-local coverage for live health apply, over-max clamp, negative clamp, unknown ref rejection, and negative ref rejection.
- Known difference: this slice only applies source-clamped health values. Later slices wire revive/passability/rerender side effects and per-packet hit-effect visual handling.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (533 passed), `./scripts/build-wasm.sh`.

## Completed slice: `UPDATE_HEALTH` revive/passability side effects

- Source: `source/zobject.cpp` `SetHealth`, `DoReviveEffect`, `SetDestroyMapImpassables`, and `UnSetDestroyMapImpassables`; `source/bbridge.cpp` bridge destroy/revive impassable center handling; `source/zbuilding.cpp` `DoReviveEffect`.
- Rust owner: `src/object_sync.rs` now marks health-packet revives through `ObjectHealthReviveQueue`; `src/main.rs` owns the Bevy side effects using the existing destroyed-object/auto-repair restore helpers.
- Runtime call site: `src/main.rs` chains `process_object_health_packet_queue` and `process_object_health_revives` after local missile damage and before production/destroyed-object lifecycle processing.
- Done: when `UPDATE_HEALTH` raises a `DestroyedObject` above zero health, Rust removes `DestroyedObject` and `AutoRepair`, clears destroyed markers from all object layers, and restores live atlas frames.
- Done: bridge revives reopen the bridge center in `PassabilityGrid` and reuse the existing delayed `BridgeRevivePending` rerender/effect path, matching source `BBridge::UnSetDestroyMapImpassables` plus delayed rerender behavior.
- Done: non-bridge building revives reuse source-backed live building frames from `GameAtlases`, matching the source `ZBuilding::DoReviveEffect` base-rerender intent.
- Known difference: `ZPlayer::set_object_health_event` still calls `DoHitEffect` after every accepted health packet; Rust has not modeled that per-packet hit flash/effect yet.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (533 passed), `./scripts/build-wasm.sh`.

## Completed slice: `UPDATE_HEALTH` `DoHitEffect` visual flag

- Source: `source/zplayer_events.cpp` `set_object_health_event`; `source/zobject.cpp` `DoHitEffect`; renderer call sites using `BlitHitSurface(..., do_hit_effect)` and resetting the flag after render.
- Rust owner: `src/object_sync.rs` now emits accepted-health hit refs through `ObjectHealthHitEffectQueue`; `src/components.rs::ObjectHitFlash` stores one-frame render state; `src/main.rs` owns layer color apply/reset.
- Runtime call site: `src/main.rs` chains accepted health packet apply, revive side effects, and hit-effect enqueue/apply after local missile damage; `animate_object_hit_flashes` restores original sprite colors on the following update.
- Done: every accepted `ObjectHealthPacket` now triggers a client-side one-frame hit flash for all visual layers with the matching `ObjectLayerRef`, matching source `ProcessObjectHealthTeam` success followed by `DoHitEffect`.
- Done: active flashes preserve each layer's original `Sprite.color` before applying the flash, so team-colored fallback layers and ordinary atlas layers restore cleanly.
- Known difference: source `BlitHitSurface` paints only opaque source pixels white/black depending render path; Rust currently approximates this with an overbright layer color because there is no source-style per-pixel blit shader yet.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (533 passed), `./scripts/build-wasm.sh`.

## Completed slice: `DRIVER_HIT_EFFECT` driver visual flag

- Source: `source/event_handler.h` `driver_hit_packet` and `tcp_event`; `source/zserver.cpp` `UpdateObjectDriverHealth`; `source/zplayer_events.cpp` `driver_hit_effect_event`; `source/zobject.cpp` `DoDriverHitEffect`; `source/zvehicle.cpp` `RenderLid`.
- Rust owner: `src/network_commands.rs` owns `DRIVER_HIT_EFFECT=37` and `DriverHitEffectPacket`; `src/object_sync.rs` owns local relay/apply queues; `src/main.rs` owns combat relay and driver-layer visual apply.
- Runtime call site: `src/main.rs::process_attack_targets` emits the packet when a snipe hits a live lead driver; `process_driver_hit_effect_packet_queue` validates the object ref before `process_driver_hit_effects` flashes only `VehicleLidVisualRole::Driver`.
- Done: `DRIVER_HIT_EFFECT` uses source layout `int ref_id` (`4` bytes) inside the packet envelope and round-trips through local encode/decode before client apply.
- Done: the adjacent source enum range is corrected in Rust: `REQUEST_SETTINGS=33`, `SET_SETTINGS=34`, `SET_LID_OPEN=35`, `SNIPE_OBJECT=36`, `DRIVER_HIT_EFFECT=37`, `SET_PLAYER_MODE=38`.
- Done: driver hit visuals no longer reuse the generic object hit queue; they target only the tank driver overlay, matching source `RenderZSurface(robot_surface, ..., do_driver_hit_effect)`.
- Known difference: source `BlitHitSurface` paints only opaque source pixels white/black depending render path; Rust currently approximates this with an overbright layer color because there is no source-style per-pixel blit shader yet.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (536 passed), `./scripts/build-wasm.sh`.

## Completed slice: `SNIPE_OBJECT` dead-driver visual packet

- Source: `source/event_handler.h` `snipe_object_packet` and `tcp_event`; `source/zserver.cpp` dead-driver branch in `UpdateObjectDriverHealth`; `source/zplayer_events.cpp` `snipe_object_event`; `source/erobotturrent.cpp` `ERobotTurrent`.
- Rust owner: `src/network_commands.rs` owns `SNIPE_OBJECT=36` and `SnipeObjectPacket`; `src/object_sync.rs` owns local packet round-trip; `src/main.rs` owns source-order visual spawn before local driverless neutralization.
- Runtime call site: `src/main.rs::process_attack_targets` relays `SNIPE_OBJECT` when a snipe reduces the lead driver to zero, spawns the source robot-turrent visual, then runs the existing `neutralize_driverless_object` owner/team reset path.
- Done: `SNIPE_OBJECT` uses source layout `int ref_id` (`4` bytes) and now round-trips through encode/decode before applying the client visual.
- Done: the client visual reuses the already ported `die5_<team>_nXX` robot turrent trajectory and applies the source `GetCenterCords(cx, cy); ERobotTurrent(cx, cy - 4, owner)` offset as `world_y + 4`.
- Known difference: the server branch also relays exact attack-object and waypoint clears for the sniped object and every attacker; Rust still relies on existing local target invalidation/neutralization rather than modeling those socket packet relays.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (539 passed), `./scripts/build-wasm.sh`.

## Completed slice: `SET_OBJECT_TEAM` dead-driver reset packet

- Source: `source/event_handler.h` `object_team_packet` and `driver_info_s`; `source/zobject.cpp` `DamageDriverHealth` and `CreateTeamData`; `source/zserver.cpp` `ResetObjectTeam`; `source/zclient.cpp` `ProcessObjectTeam`; `source/zplayer_events.cpp` `set_object_team_event`.
- Rust owner: `src/network_commands.rs` owns `SET_OBJECT_TEAM=14`, `ObjectTeamPacket`, and packed `ObjectTeamDriverInfo`; `src/object_sync.rs` owns local relay/apply validation; `src/main.rs` consumes the packet in the dead-driver neutralization path.
- Runtime call site: `src/main.rs::process_attack_targets` now relays a source-style `SET_OBJECT_TEAM` packet after `SNIPE_OBJECT` for sniped dead drivers, then `neutralize_driverless_object` applies the decoded owner to object/team layers and minimap state.
- Done: `object_team_packet` uses source header layout `int ref_id, char owner, char driver_type, char driver_amount` (`7` bytes) and `driver_info_s` entries use packed `int driver_health, double next_attack_time` (`12` bytes each).
- Done: the dead-driver path sends `owner=NULL_TEAM` with an empty driver list, matching source `DamageDriverHealth` clearing drivers and setting owner before server `ResetObjectTeam/CreateTeamData`.
- Done: client apply mirrors source guards for payload size, owner range, object ref lookup, and driver array decode before mutating Rust runtime state.
- Known difference: non-snipe ownership changes such as robot enter/capture, zone award building/flag ownership, and repair/eject driver packet relays still need to use this `SET_OBJECT_TEAM` path.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (542 passed), `./scripts/build-wasm.sh`.

## Completed slice: `RobotEnterObject` `SET_OBJECT_TEAM` capture relay

- Source: `source/zserver.cpp` `RobotEnterObject`; `source/zobject.cpp` `AddDriver`, `CreateTeamData`; `source/zclient.cpp` `ProcessObjectTeam`.
- Rust owner: `src/enter.rs` owns enter/capture orchestration; `src/object_sync.rs` owns `SET_OBJECT_TEAM` local relay/apply; `src/network_commands.rs` owns the packet layout.
- Runtime call site: `src/enter.rs::process_enter_targets` now builds target `DriverHealth` from the entering robot/group, round-trips it through `relay_object_team_update`, and applies the decoded packet in `capture_target_layers`.
- Done: vehicle/cannon capture now uses the same packed `SET_OBJECT_TEAM` path as dead-driver reset for target owner, driver type, driver healths, layer team colors, mobile frames, captured cannon frame, and minimap color.
- Done: APC driver attack stats now read the driver kind from the decoded packet state, keeping the source `SetDriverType`/`AddDriver` order tied to packet apply.
- Known difference: source `RelayPortraitAnim` for `VEHICLE_CAPTURED_ANIM` / `GUN_CAPTURED_ANIM` remains a separate visual packet slice.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (542 passed), `./scripts/build-wasm.sh`.

## Completed slice: `DO_PORTRAIT_ANIM` capture portrait packet

- Source: `source/zserver.cpp` `RobotEnterObject` and `RelayPortraitAnim`; `source/zplayer_events.cpp` `do_portrait_anim_event`; `source/zportrait.h` `portrait_anim`.
- Rust owner: `src/network_commands.rs` owns `DO_PORTRAIT_ANIM=68` and `DoPortraitAnimPacket`; `src/object_sync.rs` owns local relay/apply guards; `src/components.rs` owns the minimal `PortraitAnimationState`; `src/enter.rs` owns the capture call site.
- Runtime call site: `src/enter.rs::process_enter_targets` now relays `DO_PORTRAIT_ANIM` after capture `SET_OBJECT_TEAM` for vehicles and cannons.
- Done: vehicle capture emits `VEHICLE_CAPTURED_ANIM=61`; cannon capture emits `GUN_CAPTURED_ANIM=60`.
- Done: client apply mirrors source guards for target ref, owner matching local player team, and busy portrait state before starting the active portrait event.
- Done: accepted portrait packets add `SpaceBarEvent(ref_id, true, false)`, matching source `AddSpaceBarEvent(SpaceBarEvent(pi->ref_id, true))`.
- Known difference: full `ZPortrait` frame rendering is not ported yet; Rust currently stores the active source portrait event as runtime state so later portrait renderer work has an authoritative input.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (546 passed), `./scripts/build-wasm.sh`.

## Completed slice: portrait animation source sounds

- Source: `source/zportrait.cpp` `ZPortrait::StartAnim` and `PlayAnimSound`; `source/zsound_engine.cpp` portrait sound asset loading.
- Rust owner: `src/components.rs` owns `PortraitAnimationSoundQueue`; `src/main.rs` owns `GameSoundKind::Portrait`, source asset mapping, and queue playback; `src/enter.rs` enqueues sounds only after accepted portrait events.
- Runtime call site: accepted `DO_PORTRAIT_ANIM` capture events in `src/enter.rs::process_enter_targets` now enqueue a portrait sound after `PortraitAnimationState::start`.
- Done: source accepted portrait anim ids map to original sound files: `TARGET_DESTROYED_ANIM -> ROB37.wav`, `TERRITORY_TAKEN_ANIM -> ROB49.wav`, `GUN_CAPTURED_ANIM -> ROB51.wav`, `VEHICLE_CAPTURED_ANIM -> ROB52.wav`.
- Done: capture vehicle/gun portrait events now play the same source sounds through the existing Bevy audio path, and they are still gated by target owner/local team and busy portrait state.
- Known difference: full `ZPortrait` frame rendering, APortrait panel composition, and animation lifetime are still not ported; this slice ports the sound side-effect of accepted portrait starts.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (546 passed), `./scripts/build-wasm.sh`.

## Completed slice: `AwardZone` object-team and territory portrait relay

- Source: `source/zserver.cpp` `CheckFlagCaptures`, `AwardZone(OFlag*, ZObject*)`, `AwardZone(OFlag*, team_type)`; `source/oflag.cpp` linked building/radar behavior.
- Rust owner: `src/main.rs` owns `process_flag_captures`, `award_zone_to_team`, and local portrait/sound enqueue; `src/object_sync.rs` provides `SET_OBJECT_TEAM` relay/apply; `src/zone_sync.rs` provides runtime `SET_ZONE_INFO` relay/apply.
- Runtime call site: `src/main.rs::process_flag_captures` now carries the conquering mobile ref id into AwardZone handling, matching source `RelayPortraitAnim(conquerer->GetRefID(), TERRITORY_TAKEN_ANIM)`.
- Done: territory capture now relays/applies `DO_PORTRAIT_ANIM` with `TERRITORY_TAKEN_ANIM=58` for the conquerer, guarded by local-player team and busy portrait state, and emits the source spacebar event/sound path.
- Done: linked flag and buildings now round-trip through `SET_OBJECT_TEAM` with empty driver data before applying owner colors/flag frames/minimap dots.
- Done: zone owner changes now round-trip through runtime `SET_ZONE_INFO` before minimap zone color and production ownage recalculation.
- Known difference: real socket transport remains separate.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (549 passed), `./scripts/build-wasm.sh`.

## Completed slice: `COMP_MSG` manufactured and AwardZone feedback relay

- Source: `source/event_handler.h` `COMP_MSG`; `source/zplayer_events.cpp` `set_computer_message_event`; `source/zserver.cpp` `RelayObjectManufacturedSound`, `BuildingCreateCannon`, and `AwardZone`.
- Rust owner: `src/network_commands.rs` owns `COMP_MSG=28` and `ComputerMessagePacket`; `src/main.rs` owns source sound ids, local relay/apply, manufactured feedback, and AwardZone territory/radar feedback.
- Runtime call sites: `src/main.rs::process_building_production` uses `relay_local_manufactured_feedback`; `src/main.rs::award_zone_to_team` uses `award_zone_computer_feedback`.
- Done: manufactured vehicle/robot/gun feedback now round-trips through source `computer_msg_packet { ref_id, sound }` before playing audio, starting the computer banner, and adding the source spacebar event.
- Done: AwardZone territory-lost and radar-activated feedback now round-trips through `COMP_MSG` packets with `ref_id=-1` and sound-only client apply.
- Done: source sound ids represented for this slice: `COMP_VEHICLE_SND=19`, `COMP_ROBOT_SND=20`, `COMP_GUN_SND=21`, `COMP_TERRITORY_LOST_SND=26`, `COMP_RADAR_ACTIVATED_SND=27`.
- Known difference: real socket team filtering remains separate.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (550 passed), `./scripts/build-wasm.sh`.

## Completed slice: production-window start/cancel `COMP_MSG` relay

- Source: `source/zserver_events.cpp` `start_building_event` and `stop_building_event`; `source/zplayer_events.cpp` `set_computer_message_event`; `source/zsound_engine.h` computer sound ids.
- Rust owner: `src/production_ui.rs` owns production-window input and the source `COMP_MSG` round-trip for start/cancel sounds; `src/network_commands.rs` owns `ComputerMessagePacket`.
- Runtime call site: `src/production_ui.rs::handle_production_window_input` now sends OK/full-selector start and Cancel success feedback through `ComputerMessagePacket` before playing sound.
- Done: starting manufacture uses source `COMP_STARTING_MANUFACTURE_SND=22` and carries the building `ref_id` through `computer_msg_packet`.
- Done: canceling manufacture uses source `COMP_MANUFACTURING_CANCELED_SND=23` and carries the building `ref_id` through `computer_msg_packet`.
- Done: client apply remains sound-only for these two ids, matching source `set_computer_message_event`.
- Known difference: real socket player-only delivery and server-side login/ignored rejection paths remain separate.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (551 passed), `./scripts/build-wasm.sh`.

## Completed slice: `SET_LPLAYER_VOTEINFO` local roster integration

- Source: `source/zclient.cpp` `ProcessSetLPlayerVoteInfo`; `source/zplayer_events.cpp` `set_player_voteinfo_event`; `source/zserver.cpp` `VoteYes`, `VoteNo`, `VotePass`, `ClearPlayerVotes`, and `KillVote`.
- Rust owner: `src/local_player.rs` owns `LocalPlayerState`/roster `vote_choice` and source value guards; `src/vote.rs` owns source vote/clear packet sequencing; `src/network_commands.rs` owns `SetLocalPlayerVoteInfoPacket`.
- Runtime call sites: `src/main.rs::process_game_pause_requests`, `src/main.rs::process_vote_choice_input`, and `src/main.rs::process_vote_expiration` now apply emitted `SET_LPLAYER_VOTEINFO` packets into `LocalPlayerState`.
- Done: local roster entries now carry source `p_info.vote_choice`, defaulting to `P_NULL_VOTE`, and `apply_set_local_player_voteinfo` mirrors the source client guard (`0 <= value < P_MAX_VOTE_CHOICES`) while accepting unknown player ids without mutation.
- Done: vote choice outcomes now expose the source packet sequence, so a player vote packet is followed by `ClearPlayerVotes` null-vote packets when `CheckVote`/expiration kills the vote.
- Done: pause-vote start also emits roster null-vote packets before the initiator's auto-yes packet, matching the source `StartVote`/`ClearPlayerVotes` then `VoteYes` flow.
- Known difference: full tray/player-list UI refresh and real socket delivery remain separate; `LocalVotePlayers` still owns active vote math while `LocalPlayerState` now mirrors the client roster side of the same packets.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (552 passed), `./scripts/build-wasm.sh`.

## Completed slice: `SET_GAME_SPEED` / `CHANGE_GAME_SPEED` vote path

- Source: `source/zserver_events.cpp` `set_game_speed_event`; `source/zserver_commands.cpp` `PlayerCommand_ChangeSpeed`; `source/zserver.cpp` `StartVote`, `ProcessVote`, `ChangeGameSpeed`, and `RelayGameSpeed`; `source/ztime.cpp` `SetGameSpeed`.
- Rust owner: `src/network_commands.rs` owns runtime `SET_GAME_SPEED=74` and `UPDATE_GAME_SPEED=75` float payloads; `src/game_speed.rs` owns speed request/update queues and source percent/float casts; `src/vote.rs` owns `CHANGE_GAME_SPEED=7`; `src/chat.rs` owns `/changespeed` parsing.
- Runtime call sites: `src/main.rs::process_game_speed_requests` decodes `SET_GAME_SPEED`, starts the local source vote, and `src/main.rs::process_game_speed_updates` applies successful changes through `UPDATE_GAME_SPEED` before source news.
- Done: `/changespeed` now follows source command parsing shape: one comma-separated value, leading-space trim, `atoi`-style conversion, and `command error: invalid input(s)` only when the value is missing.
- Done: `CHANGE_GAME_SPEED` uses source vote id `7`, label `Set Game Speed`, default server setting `allow_game_speed_change=true`, and source rejection news for non-positive values.
- Done: successful speed votes apply `value / 100.0` through `UPDATE_GAME_SPEED` and emit `game speed changed to N%`, matching `ChangeGameSpeed`.
- Known difference: real socket transport and dynamic server config loading for `allow_game_speed_change` remain separate; the broader frozen/scaled game-time model still needs a later source-backed pass.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (559 passed), `./scripts/build-wasm.sh`.

## Completed slice: `SET_LID_OPEN` vehicle lid packet parity

- Source: `source/event_handler.h` `set_lid_state_packet` / `SET_LID_OPEN=35`; `source/zvehicle.cpp` `SignalLidShouldOpen`, `SignalLidShouldClose`, `ProcessServerLid`, and `SetLidState`; `source/zclient.cpp` `ProcessObjectLidState`; `source/zplayer_events.cpp` `set_lid_open_event`.
- Rust owner: `src/network_commands.rs` owns `SetLidOpenPacket` packed as `i32 ref_id + bool`; `src/object_sync.rs` owns `VehicleLidPacketQueue`, relay, and client apply; `src/units/vehicles/mod.rs` owns source lid signal/timer/state mutation; `src/main.rs` owns Bevy scheduling and packet production from the local server lid flow.
- Runtime call site: `src/main.rs::process_vehicle_lids` now emits `SET_LID_OPEN` on successful open signals and delayed server close; `src/object_sync.rs::process_vehicle_lid_packet_queue` applies packets before `sync_vehicle_lid_visual_layers`.
- Done: Light/Medium/Heavy lid open/close state now round-trips through source packet layout before the visual overlay reads `VehicleLidState`, matching the source server `sflags.updated_open_lid` -> client `SetLidState` path.
- Done: packet decode rejects non-source payload sizes and non-0/1 bool bytes; relay rejects ref ids that cannot fit the source `int`.
- Known difference: this is still local relay, not real socket delivery; source `do_open_lid` delayed-open branch remains untriggered by current Rust gameplay, matching the already-modeled attack-target open signal path for this slice.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (562 passed), `./scripts/build-wasm.sh`.

## Completed slice: `TARGET_DESTROYED_ANIM` portrait relay

- Source: `source/zobject.cpp` `ZObject::ProcessAttackDamage` target-destroyed `sflags.portrait_anim_*`; `source/zserver.cpp` `ProcessMissileDamage` target-destroyed relay and `RelayPortraitAnim`; `source/zplayer_events.cpp` `do_portrait_anim_event`; `source/zportrait.h` `TARGET_DESTROYED_ANIM`.
- Rust owner: `src/components.rs::DamageMissile` stores source attacker/player-given/target metadata for delayed missile damage; `src/main.rs::process_attack_targets` owns direct attack damage and `src/main.rs::process_damage_missiles` owns missile explosion damage; existing `src/object_sync.rs` `relay_portrait_anim` / `apply_portrait_anim_packet` remains the source packet path.
- Runtime call sites: player-given direct damage that destroys the target now relays `DO_PORTRAIT_ANIM` with the attacker ref id; player-given missile explosions relay it only when the damaged object is the original missile target and is destroyed.
- Done: target-destroyed portrait events now use source `TARGET_DESTROYED_ANIM=30`, source client guards (attacker object must belong to local team and portrait must not already be busy), source spacebar event shape, and the existing source portrait sound queue.
- Done: neutral/grenade-box/map-object missiles carry no attacker metadata, so ambient or neutral explosions cannot produce player target-destroyed portrait events.
- Known difference: Rust still applies the local relay immediately at the damage site instead of batching object `sflags` until the later server object update pass; full attack-object flag relay/timing remains a separate source slice.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (563 passed), `./scripts/build-wasm.sh`.

## Completed slice: `SET_ATTACK_OBJECT` attack target packet parity

- Source: `source/event_handler.h` `attack_object_packet` / `SET_ATTACK_OBJECT=15`; `source/zobject.cpp` `Engage`, `Disengage`, and `CreateAttackObjectData`; `source/zserver.cpp` `RelayObjectAttackObject`; `source/zclient.cpp` `ProcessObjectAttackObject`; `source/zplayer_events.cpp` `set_object_attack_object_event`.
- Rust owner: `src/network_commands.rs` owns `AttackObjectPacket`; `src/object_sync.rs` owns local relay/apply validation; `src/main.rs` owns Bevy attack target assignment/clear wiring through the existing `AttackTarget` component.
- Runtime call sites: passive/player/movement `ATTACK_WP` assignment and combat/movement clear paths now round-trip through `SET_ATTACK_OBJECT` before mutating `AttackTarget`.
- Done: packet layout matches source packed `int ref_id, int attack_object_ref_id`, including `attack_object_ref_id=-1` for clear/disengage.
- Done: client apply mirrors source lookup shape: missing object packet is ignored, missing/negative attack target clears the local attack target, valid target assigns it.
- Known difference: Rust still keeps `AttackTarget.player_given` as local server-side metadata because source `attack_object_packet` does not carry it; full split between server attack state and client visual attack state remains a later real-socket/state-authority pass.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (566 passed), `./scripts/build-wasm.sh`.

## Completed slice: `SET_GRENADE_AMOUNT` object grenade packet parity

- Source: `source/event_handler.h` `obj_grenade_amount_packet` / `SET_GRENADE_AMOUNT=66`; `source/zserver.cpp` `RelayObjectGrenadeAmount`; `source/zclient.cpp` `ProcessSetGrenadeState`; `source/zobject.cpp` grenade pickup and grenade-consume flags; `source/zrobot.cpp` `SetGrenadeAmount`.
- Rust owner: `src/network_commands.rs` owns `ObjectGrenadeAmountPacket`; `src/object_sync.rs` owns relay/apply and source robot clamp; `src/grenades.rs` owns pickup transfer; `src/main.rs::process_attack_targets` owns own/leader grenade consumption.
- Runtime call sites: grenade pickup now round-trips the robot's new amount through `SET_GRENADE_AMOUNT`; grenade attacks now round-trip consumed own or group-leader amounts before mutating live `GrenadeInventory`.
- Done: packet layout matches source packed `int ref_id, int grenade_amount`; source invalid robot values (`<0` or `>99`) clamp to `0` on client apply.
- Done: grenade boxes still do not emit `SET_GRENADE_AMOUNT`, matching `RelayObjectGrenadeAmount` returning early when `CanHaveGrenades()` is false; box deletion/pickup animation remain separate packet slices.
- Known difference at this slice: `PICKUP_GRENADE_ANIM` and delete-grenade-box cleanup were not modeled yet; they are covered by the following pickup animation and destroy-object cleanup slices.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (569 passed), `./scripts/build-wasm.sh`.

## Completed slice: `PICKUP_GRENADE_ANIM` pickup animation packet parity

- Source: `source/event_handler.h` `PICKUP_GRENADE_ANIM=67` via `int_packet`; `source/zserver.cpp` object update pass sending `do_pickup_grenade_anim`; `source/zplayer_events.cpp` `pickup_grenade_event`; `source/zrobot.cpp` `DoPickupGrenadeAnim`; `source/zportrait.h` `GRENADES_COLLECTED_ANIM`; `source/zportrait.cpp` / `source/zsound_engine.cpp` collected-grenades sound.
- Rust owner: `src/network_commands.rs` owns `PickupGrenadeAnimationPacket`; `src/object_sync.rs` owns relay/apply guards; `src/grenades.rs` owns pickup animation start and collected-grenades portrait side effect; `src/components.rs` / `src/main.rs` own `PortraitAnimationKind::GrenadesCollected` and sound mapping.
- Runtime call site: `src/grenades.rs::process_grenade_pickups` now performs `SET_GRENADE_AMOUNT`, then round-trips `PICKUP_GRENADE_ANIM` before inserting `RobotGrenadePickupAnimation` and local-team `GRENADES_COLLECTED_ANIM` portrait/spacebar side effect.
- Done: packet layout matches source `int ref_id`; client apply rejects negative/mismatched refs, non-grenade-capable objects, and attacking robots like source `DoPickupGrenadeAnim`.
- Done: accepted local-team pickup events start `GRENADES_COLLECTED_ANIM=62`, enqueue source `ROB53.wav`, and add the source spacebar event.
- Known difference at this slice: delete-grenade-box cleanup remained separate and still used the existing local despawn; it is covered by the following destroy-object cleanup slice.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (573 passed), `./scripts/build-wasm.sh`.

## Completed slice: grenade-box pickup `DESTROY_OBJECT` cleanup parity

- Source: `source/zobject.cpp` pickup branch sets `delete_grenade_box_ref_id` after zeroing the grenade box; `source/zserver.cpp` object update pass resolves that ref, calls `SetHealth(0)`, then `UpdateObjectHealth`; because grenade boxes have item health, `UpdateObjectHealth` takes the destroyed branch and calls `RelayObjectDeath`; `source/zplayer_events.cpp` applies `DESTROY_OBJECT`.
- Rust owner: `src/network_commands.rs` owns `DestroyObjectPacket`; `src/object_sync.rs` owns `ObjectDestroyPacketQueue`, relay/apply validation, and health-zero application; `src/grenades.rs` owns pickup transfer and box cleanup request; `src/main.rs` wires the destroy packet queue before the existing destroyed-object lifecycle.
- Runtime call site: `src/grenades.rs::process_grenade_pickups` now enqueues `DESTROY_OBJECT` for the picked grenade box after transfer instead of manually despawning object layers and minimap dots.
- Done: packet layout matches source packed `int ref_id, int fire_missile_amount, int killer_ref_id, bool destroy_object, bool do_fire_death, bool do_missile_death` plus optional `fire_missile_info` tail.
- Done: grenade-box pickup emits `killer_ref_id=-1`, `destroy_object=true`, no death flags, and no turret missiles because the source box amount has already been set to `0` before `RelayObjectDeath`.
- Done: removed the pickup-only direct layer/minimap despawn helpers from `src/grenades.rs`; the existing destroyed-object lifecycle owns layer/minimap cleanup.
- Known difference: source sends the packet from the later server object update pass; Rust queues it locally and applies it on the next packet pass until real socket/server timing exists. Generic `DELETE_OBJECT` remains separate because this pickup path does not use it in the source.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (578 passed), `./scripts/build-wasm.sh`.

## Completed slice: live destroyed-object `DESTROY_OBJECT` lifecycle flags

- Source: `source/zserver.cpp` `UpdateObjectHealth` destroyed branch and `RelayObjectDeath`; `source/zplayer_events.cpp` `destroy_object_event`; `source/event_handler.h` `destroy_object_packet`.
- Rust owner: `src/object_sync.rs` owns reusable decoded `relay_destroy_object_packet`; `src/main.rs::process_destroyed_objects` owns the existing Bevy destroyed-object lifecycle.
- Runtime call site: `src/main.rs::process_destroyed_objects` now round-trips every newly destroyed live object through decoded `DESTROY_OBJECT` data before constructing `DestroyedObjectSnapshot`.
- Done: destroyed-object lifecycle now reads `destroy_object`, `do_fire_death`, and `do_missile_death` from the decoded source packet instead of directly from Rust helpers/timers.
- Done: source `destroy_object` controls whether layers/minimap are despawned or converted to persistent destroyed markers, preserving the existing lifecycle while making the packet field authoritative.
- Known difference at this slice: Rust still did not serialize source `ServerFireTurrentMissile` tail data for general deaths; grenade-box and map-object tails are covered by the following effect-family slices, while cannon/vehicle tails remain separate.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (578 passed), `./scripts/build-wasm.sh`.

## Completed slice: grenade-box `DESTROY_OBJECT` `fire_missile_info` tail

- Source: `source/ogrenades.cpp` `OGrenades::ServerFireTurrentMissile`; `source/zserver.cpp` `RelayObjectDeath` serializes `fire_missile_info` after `destroy_object_packet`; `source/zplayer_events.cpp` `destroy_object_event` calls `FireTurrentMissile` for each tail entry.
- Rust owner: `src/units/items/grenades/grenades_logic.rs` owns grenade-box missile target/delay rules; `src/object_sync.rs` owns tail-aware `relay_destroy_object_packet_with_missiles`; `src/main.rs` owns Bevy visual spawn from decoded packet tail.
- Runtime call site: `src/main.rs::process_destroyed_objects` now builds grenade-box `fire_missile_info` tail before `DESTROY_OBJECT` encode/decode, stores decoded tail in `DestroyedObjectSnapshot`, then spawns grenade death missiles from that tail.
- Done: one packet tail entry is generated per remaining grenade, preserving source offset-time and target fields through `DestroyObjectPacket.fire_missiles`.
- Done: `spawn_grenade_box_destroy_missiles` no longer rolls random target/delay values itself; it applies decoded `fire_missile_info` values.
- Known difference: coordinate origin still follows the current Rust object transform semantics; source uses `loc + 16` for target scatter and `loc + 2` as the visual missile origin. Cannon, vehicle, and map-object turrent tails remain separate at this slice; map-object is covered by the following slice.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (578 passed), `./scripts/build-wasm.sh`.

## Completed slice: map-object `DESTROY_OBJECT` `fire_missile_info` tail

- Source: `source/omapobject.cpp` `OMapObject::ServerFireTurrentMissile` and `FireTurrentMissile`; `source/emapobjectturrent.cpp` constructor start jitter/rise/spin; `source/zserver.cpp` `RelayObjectDeath`; `source/zplayer_events.cpp` `destroy_object_event`.
- Rust owner: `src/units/items/map_object/map_object_ui.rs` owns map-object turrent target/delay rules; `src/object_sync.rs` owns tail-aware `relay_destroy_object_packet_with_missiles`; `src/main.rs` owns Bevy visual spawn from decoded packet tail.
- Runtime call site: `src/main.rs::process_destroyed_objects` now adds one map-object `fire_missile_info` tail entry before `DESTROY_OBJECT` encode/decode and passes decoded tail into `spawn_map_object_turrent_missile`.
- Done: map-object turrent target/delay are packet-authoritative; `spawn_map_object_turrent_missile` no longer rolls those values directly.
- Done: client-side start jitter, rise, and spin stay in the visual effect spawn path, matching `EMapObjectTurrent` constructor ownership.
- Known difference: Rust still uses the current map/grid conversion helpers for object origin; exact per-pixel source `loc` audit remains a later effect-position slice. Cannon turrent tails remain a separate effect-family slice; vehicle is covered by the following slice.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (579 passed), `./scripts/build-wasm.sh`.

## Completed slice: vehicle `DESTROY_OBJECT` `fire_missile_info` tail

- Source: `source/vlight.cpp`, `source/vmedium.cpp`, and `source/vheavy.cpp` `ServerFireTurrentMissile` / `FireTurrentMissile`; `source/eturrentmissile.cpp` constructor owns start jitter, rise, and spin; `source/zserver.cpp` `RelayObjectDeath`; `source/zplayer_events.cpp` `destroy_object_event`.
- Rust owner: `src/units/vehicles/vehicle_ui.rs` owns vehicle turrent target/delay/start/rise visual rules; `src/object_sync.rs` owns tail-aware `relay_destroy_object_packet_with_missiles`; `src/main.rs` owns Bevy visual spawn from decoded packet tail.
- Runtime call site: `src/main.rs::process_destroyed_objects` now adds one Light/Medium/Heavy vehicle `fire_missile_info` tail entry before `DESTROY_OBJECT` encode/decode and passes decoded tail into `spawn_vehicle_death_effect`.
- Done: Light/Medium/Heavy vehicle turrent target/delay are packet-authoritative; `spawn_vehicle_turrent_missile_effect` no longer calls a local aggregate launch helper for those values.
- Done: client-side `ETurrentMissile` constructor behavior stays in the visual effect spawn path: start is `loc + 8`, while jitter, rise, and spin are still generated client-side.
- Removed: stale `VehicleTurrentLaunch` / `vehicles::turrent_launch` aggregate helper, because it mixed server-owned target/delay with client-owned rise after the packet tail became authoritative.
- Known difference: Rust still uses the current vehicle death top-left conversion from the Bevy object center; exact source `loc` parity remains a later effect-position audit. Cannon is covered by the following slice.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (580 passed), `./scripts/build-wasm.sh`.

## Completed slice: cannon `DESTROY_OBJECT` `fire_missile_info` tail

- Source: `source/cgatling.cpp`, `source/cgun.cpp`, `source/chowitzer.cpp`, and `source/cmissilecannon.cpp` `ServerFireTurrentMissile` / `FireTurrentMissile`; `source/ecannondeath.cpp` owns wasted-cannon visual timing; `source/zserver.cpp` `RelayObjectDeath`; `source/zplayer_events.cpp` `destroy_object_event`.
- Rust owner: `src/units/cannons/cannon_ui.rs` owns cannon turrent target/delay/rise/jitter visual rules; `src/object_sync.rs` owns tail-aware `relay_destroy_object_packet_with_missiles`; `src/main.rs` owns Bevy visual spawn from decoded packet tail.
- Runtime call site: `src/main.rs::process_destroyed_objects` now adds one cannon `fire_missile_info` tail entry before `DESTROY_OBJECT` encode/decode and passes decoded tail into `spawn_cannon_death_effect`.
- Done: cannon turrent target/offset-time are packet-authoritative; `cannon_ui::death_visual_policy` no longer rolls target/delay directly and instead receives decoded packet target/offset.
- Done: client-side `ECannonDeath` constructor behavior stays in the visual effect spawn path: cannon death delay, start jitter, rise, spin, sparks, crater, and impact effects remain generated by the Bevy visual path.
- Known difference: Rust still uses the current cannon center/map conversion for source `loc + 16`; exact source `loc` parity remains a later effect-position audit.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (581 passed), `./scripts/build-wasm.sh`.

## Completed slice: `DESTROY_OBJECT` turrent tail server damage

- Source: `source/zserver.cpp` `RelayObjectDeath` pushes every `fire_missile_info` into `new_damage_missile_list` with source `damage`, `radius`, `team=NULL_TEAM`, `attacker_ref_id=-1`, `attack_player_given=false`, and `explode_time = now + missile_offset_time`.
- Rust owner: `src/main.rs` owns the server-style logical `DeathTurrentDamageMissile` component and reuses the existing `process_damage_missiles` explosion damage resolver; `src/units/vehicles/vehicle_ui.rs` and `src/units/cannons/cannon_ui.rs` own source `damage/radius` values.
- Runtime call site: `src/main.rs::process_destroyed_objects` now creates logical damage missiles from decoded vehicle/cannon `DESTROY_OBJECT` tail entries while leaving their existing Bevy visual effects unchanged.
- Done: Light/Medium/Heavy vehicle and cannon turrent death missiles now apply NULL-team AoE damage at packet offset time, matching the source server damage list instead of being visual-only.
- Done: no duplicate client visual/sound/crater is spawned for this logical server damage; vehicle/cannon `ETurrentMissile`/`ECannonDeath` visuals still own presentation, while the logical component owns damage.
- Known difference: grenade-box and map-object tails already use `DamageMissile` visual components that carry their damage path; this slice only adds the missing logical damage for vehicle/cannon visual-only effects. Exact server tick ordering vs local Bevy schedule remains tied to current local relay timing until real socket/server transport exists.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (582 passed), `./scripts/build-wasm.sh`.

## Completed slice: `DESTROY_OBJECT` own-fort focus side effect

- Source: `source/zplayer_events.cpp` `ZPlayer::destroy_object_event` checks `obj->GetRefID() == p->fort_ref_id`, reads object center cords, and calls `FocusCameraTo(x, y)` after applying death effects/turrent missiles.
- Rust owner: `src/main.rs::process_destroyed_objects` owns decoded `DESTROY_OBJECT` lifecycle side effects; `src/camera.rs::focus_camera_to_world_point` owns camera centering/clamping.
- Runtime call site: after decoded destroyed-object snapshots are built, Rust now checks whether the destroyed object is the local player's fort and focuses the main camera on its current world center.
- Done: local-team fort destruction now triggers the source client-side camera focus instead of leaving camera state unchanged.
- Known difference: source `FocusCameraTo` animates the move over about `0.7s`; Rust currently applies an instant clamped center through the existing camera model. Last-enemy-fort focus remains a separate `destroy_object_event` branch.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (583 passed), `./scripts/build-wasm.sh`.

## Completed slice: `DESTROY_OBJECT` last-enemy-fort focus side effect

- Source: `source/zplayer_events.cpp` `ZPlayer::destroy_object_event` adjacent `else` branch checks destroyed `FORT_FRONT`/`FORT_BACK`, `obj->GetOwner() != p->our_team`, `team_units_available[p->our_team] > 0`, and no other non-owner enemy team with available units before `FocusCameraTo(x, y)`.
- Rust owner: `src/main.rs::process_destroyed_objects` owns decoded `DESTROY_OBJECT` focus side effects and `destroy_object_team_units_available`; `src/camera.rs::focus_camera_to_world_point` owns camera centering/clamping.
- Runtime call site: after decoded destroyed-object snapshots are built, Rust now computes source-style available unit counts from live robot/vehicle/cannon snapshots, preserves own-fort priority, then focuses the main camera on the destroyed last enemy fort when the source conditions match.
- Done: destroyed enemy FortFront/FortBack can now trigger source client-side camera focus when the local team still has available units and no third enemy team has available units.
- Done: the destroyed fort owner's remaining units are ignored for the `other_teams_exist` check, matching the source `i != obj->GetOwner()` condition.
- Known difference: source `FocusCameraTo` animates the move over about `0.7s`; Rust still applies an instant clamped center through the existing camera model. The lower `was this the target of our "A" unit?` portrait branch remains a separate `destroy_object_event` slice.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (586 passed), `./scripts/build-wasm.sh`, browser boot-state/canvas smoke.

## Completed slice: `DESTROY_OBJECT` A-unit good-hit portrait side effect

- Source: `source/zplayer_events.cpp` `ZPlayer::destroy_object_event` lower branch checks `pi->killer_ref_id != -1`, `zhud.GetARefID() != -1`, `!GetAPortrait().DoingAnim()`, killer owner equals `our_team`, and killer center is either the A-ref object or within `100` pixels before starting `GOOD_HIT_ANIM + rand()%7` without a spacebar event.
- Rust owner: `src/main.rs::process_destroyed_objects` owns decoded `DESTROY_OBJECT` client side effects; `src/components.rs::PortraitAnimationKind::GoodHit` owns the source anim id range; `DamageCauseTimers` now carries recent `killer_ref_id` into the local source packet relay.
- Runtime call site: direct and missile damage record the attacker ref id in `DamageCauseTimers`; `process_destroyed_objects` serializes that into `DestroyObjectPacket.killer_ref_id`, then consumes the decoded packet plus the current `HudAttackAlert.target_ref_id` A-button state to start a good-hit portrait sound/animation.
- Done: source good-hit variants `GOOD_HIT_ANIM..WIPE_OUT_ANIM` map to source ids `50..56` and sounds `ROB40.wav..ROB46.wav`.
- Done: the source `static last_anim` non-repeat rule is represented by `DestroyObjectGoodHitState`.
- Known difference: Rust's broader portrait renderer/timer is still incomplete, so `PortraitAnimationState` busy lifetime remains the existing simplified model. Existing player-given target-destroyed portrait relay still fires earlier than source server batching in some paths; full target-destroyed batching remains a separate source slice.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (589 passed), `./scripts/build-wasm.sh`, browser boot-state/canvas smoke.

## Completed slice: `SET_ATTACK_OBJECT` A-button under-attack client side effect

- Source: `source/zplayer_events.cpp` `ZPlayer::set_object_attack_object_event`; `source/zhud.cpp` `ZHud::ProcessA`; `source/zportrait.h` `WERE_UNDER_ATTACK_ANIM=22`; `source/zsound_engine.cpp` `ROB25.wav`.
- Rust owner: `src/components.rs::{AttackAlertPacketQueue,HudAttackAlert,PortraitAnimationKind::WereUnderAttack}` owns queued packet side effects, A-ref mutation/reset, and source portrait anim id; `src/main.rs` owns the local client event apply; `src/hud.rs::update_hud_attack_alert` now only mirrors `ProcessA` blink/check/clear behavior for an existing A-ref.
- Runtime call site: successful local `SET_ATTACK_OBJECT` assignment relays from passive engage and movement `ATTACK_WP` paths into `AttackAlertPacketQueue`; `process_attack_alert_packet_side_effects` consumes accepted target refs, checks local target ownership and empty A-ref before rolling the source `rand()%5` chance, then starts `WERE_UNDER_ATTACK_ANIM` if the A-portrait is idle and adds the source `SpaceBarEvent(ref_id, true)`.
- Done: removed the previous Rust-only HUD auto-pick of the first under-attack local object, because source assigns A-ref only from `set_object_attack_object_event`.
- Done: `WERE_UNDER_ATTACK_ANIM` maps to source wire id `22` and portrait sound `sounds/ROB25.wav`.
- Known difference: the repeated `I_SAID_WERE_UNDER_ATTACK_ANIM + rand()%6` branch from `ZHud::ProcessA` is still not modeled; this slice covers the initial packet-side A-ref assignment and immediate portrait event.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (591 passed), `./scripts/build-wasm.sh`, browser boot-state/canvas/non-black smoke.

## Completed slice: `ZHud::ProcessA` repeated A-portrait under-attack warning

- Source: `source/zhud.cpp` `ZHud::ProcessA` repeat branch and `ZHud::SetARefID`; `source/zportrait.h` `I_SAID_WERE_UNDER_ATTACK_ANIM=23`; `source/zsound_engine.cpp` `ROB26.wav..ROB30.wav` and `ROB32.wav`.
- Rust owner: `src/components.rs::HudAttackAlert` now stores `next_a_anim_time`-equivalent elapsed/delay and `last_a_anim`; `src/hud.rs::update_hud_attack_alert` owns the source repeat timing and non-repeat variant choice; `src/components.rs::PortraitAnimationKind::UnderAttackRepeat` owns the source wire ids `23..28`.
- Runtime call site: `process_attack_alert_packet_side_effects` schedules the first repeat delay when A-ref is assigned, matching `SetARefID`; every active `update_hud_attack_alert` tick advances that timer only while the A-ref still exists, schedules the next `5 + rand()%300*0.01` delay before checking portrait busy state, chooses `rand()%6` without repeating `last_a_anim`, and starts the A-portrait animation only when idle.
- Done: repeat variants map to source ids `23..28` and sounds `ROB26.wav`, `ROB27.wav`, `ROB28.wav`, `ROB29.wav`, `ROB30.wav`, and `ROB32.wav`.
- Done: when the portrait is busy, Rust now keeps `last_a_anim` unchanged while still scheduling the next repeat window, matching the source branch shape.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (595 passed), `./scripts/build-wasm.sh`. Browser smoke was attempted, but Browser Use blocked direct navigation to the local URL by URL policy; the local server answered `200 OK` on `127.0.0.1:4173`.

## Completed slice: `ZPortrait::Process` busy lifetime for wired A-portrait events

- Source: `source/zportrait.cpp` `ZPortrait::StartAnim`, `ZPortrait::Process`, and `ZPortrait_Anim::AddFrame`; `source/zportrait.h` currently wired event ids `22..30`, `50..58`, and `60..62`.
- Rust owner: `src/components.rs::PortraitAnimationState` owns active event elapsed time; `src/components.rs::PortraitAnimationKind::source_total_duration_secs` owns source `total_duration` values for every currently wired event animation; `src/main.rs::process_portrait_animation_state` owns the per-frame `Process` tick.
- Runtime call site: Bevy update now processes portrait lifetime before systems that check `doing_anim()` and may start new portrait events, including flag capture, target destroyed, damage missile, destroyed-object good-hit, grenade pickup, enter/capture, and HUD A-alert repeat paths.
- Done: `StartAnim` now resets elapsed time, and active portrait state clears only when elapsed time is greater than source total duration, matching `if(time_in > anim_info[cur_anim].total_duration) cur_anim = -1`.
- Done: source duration totals are represented for `WERE_UNDER_ATTACK`, `I_SAID/HELP/THEYRE_ALL_OVER_US/WERE_LOSING_IT/AAAHHH/FOR_CHRIST_SAKE`, `TARGET_DESTROYED`, `GOOD_HIT..WIPE_OUT`, `TERRITORY_TAKEN`, `GUN_CAPTURED`, `VEHICLE_CAPTURED`, and `GRENADES_COLLECTED`.
- Known difference: this still does not render the actual portrait frame list/hand/head/mouth/eye animation; it only fixes the source busy lifetime so already ported portrait side effects can sequence correctly.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (598 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: selected object `PlaySelectedAnim` portrait feedback

- Source: `source/zobject.cpp` `ZObject::PlaySelectedAnim`; `source/rgrunt.cpp`, `source/rpsycho.cpp`, `source/rsniper.cpp`, `source/rtough.cpp`, `source/rlaser.cpp`, `source/rpyro.cpp` robot `PlaySelectedAnim` overrides; `source/zportrait.cpp` `ZPortrait::StartAnim`, `PlayAnimSound`, and `SetupFrames`.
- Rust owner: `src/components.rs::PortraitAnimationKind` owns selected portrait animation ids and source frame-total durations; `src/units/unit_sound.rs` owns generic selected portrait/sound policy; `src/units/robots/mod.rs` owns robot-specific selected reporting policy; per-robot `*_ui.rs` keep reporting wav assets.
- Runtime call site: `src/main.rs::play_selected_portrait_feedback`, scheduled on selection changes after eject commands and before passive engage.
- Done: local selection now starts `PortraitAnimationState` with source selected animation kinds instead of only playing the old Rust voice shortcut.
- Done: vehicle/cannon selections with a live driver use the base `ZObject::PlaySelectedAnim` generic selected branch; robots use the source override `rand()%2` split between generic selected anims and robot-specific reporting anims.
- Done: selected portrait sounds now come from the chosen `PortraitAnimationKind`, so `YES_SIR_ANIM` and `UNIT_REPORTING2_ANIM` consume their extra random sound variant in the same sequence as source `PlayAnimSound`.
- Known difference: full portrait frame rendering is still not ported; this slice wires authoritative selected animation state and sound only. `PlayAcknowledgeAnim` command feedback remains a separate source slice.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (600 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: selected command `PlayAcknowledgeAnim` portrait feedback

- Source: `source/zplayer.cpp` `ZPlayer::AddDevWayPointToSelected`, `DevWayPointsNoWay`, `SendDevWayPointsOfSelected`; `source/zhud.cpp` `ZHud::GiveSelectedCommand`; `source/zobject.cpp` `ZObject::PlayAcknowledgeAnim`; `source/zunitrating.cpp` `ZUnitRating::InitPopulateUCR`; `source/zportrait.cpp` `PlayAnimSound` and `SetupFrames`.
- Rust owner: `src/components.rs::PortraitAnimationKind` owns acknowledge/no-way source ids and frame-total durations; `src/units/unit_sound.rs` owns `PlayAcknowledgeAnim` random selection; `src/units/attack.rs` owns the source `UCR_WILL_DIE` cross-reference table used by `DevWayPointsNoWay`.
- Runtime call site: `src/selection.rs::handle_mouse_commands` starts acknowledge feedback for recognized right-click commands through `MouseCommandPortraitFeedback`, queuing sound through `PortraitAnimationSoundQueue`.
- Done: successful right-click unit commands now start `Acknowledge(0..11)` portrait animations and sounds `ROB13.wav..ROB24.wav`.
- Done: single-selected attack commands against source `UCR_WILL_DIE` targets now start `AcknowledgeNoWay(0..2)` with `ROB35.wav`, `ROB36.wav`, or `ROB34.wav`, matching the source `FORGET_IT`, `GET_OUTTA_HERE`, and `NO_WAY` branches.
- Done: the source `ZUnitRating` weak-vs-strong direction is represented for robot/vehicle/cannon unit pairs used by `DevWayPointsNoWay`; reverse `UCR_WILL_KILL` directions do not trigger no-way feedback.
- Known difference: full portrait frame rendering is still not ported; this slice wires authoritative command feedback state and sound only. The source `AsciiDown('z')` one-unit-per-target waypoint send branch remains separate.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (603 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: `AsciiDown('z')` one-unit-per-target command send

- Source: `source/zplayer.cpp` `ZPlayer::SendDevWayPointsOfSelected`; `source/zplayer_events.cpp` `ZPlayer::runclick_event`.
- Rust owner: `src/selection.rs::handle_mouse_commands` owns local right-click command fanout; `one_unit_command_ref` mirrors source nearest-selected-to-target choice; `remove_one_unit_command_selection` mirrors the source post-send selection removal.
- Runtime call site: on right mouse release, after the eject special case and before order expansion, Rust now checks `KeyCode::KeyZ`.
- Done: when `Z` is held, unit repair, crane repair, grenade pickup, enter, enter fort, attack, and move commands are issued only to the selected object nearest to the command point.
- Done: `Z` bypasses robot group/minion expansion for the command recipient, matching source sending only the chosen selected object's waypoint list.
- Done: after a recognized command branch, the chosen object is removed from `SelectionState` and its selection marker/health bar entities are despawned, matching source `RemoveFromSelected` after `SendDevWayPointsOfObj`.
- Known difference: source stores full dev waypoint lists and shows the final `Pcursor` before clearing them; Rust now round-trips current movement paths through `SEND_WAYPOINTS`, but still does not render source `Pcursor` feedback or apply every command through full server-side `ProcessWaypointData`.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (605 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: Shift-held terrain `MOVE_WP` accumulation

- Source: `source/zplayer_events.cpp` `ZPlayer::runclick_event` and `keyup_event`; `source/zplayer.cpp` `ZPlayer::AddDevWayPointToSelected`, `SendDevWayPointsOfSelected`, and terrain `MOVE_WP` waypoint creation.
- Rust owner: `src/selection.rs::PendingMouseMoveCommands` stores local pending terrain move commands; `handle_mouse_commands` queues them while Shift is held and sends them when the final Shift key is released.
- Runtime call site: Shift + right-click on empty terrain records a source-style player-given move point without immediately assigning `MovementPath`; Shift release converts the pending list into a chained typed `MovementPath` for the selected mobile refs.
- Done: pending terrain moves preserve selected refs, Ctrl/Alt `attack_to` policy, group fanout through leader refs, run-near-flag detection, target marker feedback, and the existing acknowledge portrait/sound feedback on send.
- Done: queued movement segments use the previous queued target as the next route start, so multiple Shift terrain clicks produce a route chain instead of overwriting the earlier point.
- Known difference: this slice covers empty-terrain `MOVE_WP` only. Source can also accumulate `ATTACK_WP`, `UNIT_REPAIR_WP`, `CRANE_REPAIR_WP`, `PICKUP_GRENADES_WP`, `ENTER_WP`, and `ENTER_FORT_WP` in the same dev-list; those target-mode Shift branches still need a richer queued waypoint representation.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (607 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: Shift-held `ATTACK_WP` accumulation

- Source: `source/zplayer.cpp` `ZPlayer::AddDevWayPointToSelected`, `DevWayPointsNoWay`, and `SendDevWayPointsOfSelected`; `source/zplayer_events.cpp` `ZPlayer::runclick_event` and Shift `keyup_event`.
- Rust owner: `src/selection.rs::PendingMouseCommand` now stores typed pending `MOVE_WP` and `ATTACK_WP` commands; the Shift release sender appends attack commands through the existing typed `MovementWaypoint::player_attack_target` path.
- Runtime call site: Shift + right-click on an enemy target queues `ATTACK_WP` instead of assigning `MovementPath` immediately; releasing the final Shift sends the queued path and starts acknowledge/no-way portrait feedback from the first queued command.
- Done: queued attack commands preserve selected refs, source group fanout, target ref id, route-to-attack-range movement for mobile attackers, stationary attacker support, and final `ATTACK_WP` metadata with `player_given=true` / `attack_to=true`.
- Done: pending attack no-way feedback uses the same source `DevWayPointsNoWay` weak-vs-strong gate as immediate attack commands when the first queued command is an attack.
- Known difference: queued attack route chaining after the target resolves uses the precomputed range end as the next route start. Source processes the later waypoint from the unit's actual position after `ATTACK_WP` completes. Shift accumulation for `UNIT_REPAIR_WP`, `CRANE_REPAIR_WP`, `PICKUP_GRENADES_WP`, `ENTER_WP`, and `ENTER_FORT_WP` remains separate.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (608 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: Shift-held `PICKUP_GRENADES_WP` accumulation

- Source: `source/zplayer.cpp` `ZPlayer::AddDevWayPointToSelected` `PICKUP_GRENADES_WP` branch and `SendDevWayPointsOfSelected`; `source/zplayer_events.cpp` `ZPlayer::runclick_event` and Shift `keyup_event`; `source/zobject.cpp` `ProcessPickupGrenadesWP`.
- Rust owner: `src/selection.rs::PendingMouseCommand::PickupGrenades` stores the queued pickup target and eligible robot refs; Shift release appends movement to the grenade box and inserts `PickupGrenadesTarget`.
- Runtime call site: Shift + right-click on a grenade box queues pickup instead of assigning movement immediately; releasing the final Shift sends the pending route through the same pickup processing used by immediate commands.
- Done: queued pickup preserves the source selected-object owner list for HUD acknowledge, stores only robots that can pick up at click time, validates that the target still exists as a grenade map item at send time, and chains from earlier pending route endpoints.
- Done: pickup is checked before attack in the Shift target path, matching the source robot hover-object branch where grenade pickup wins over attack.
- Known difference: source stores a typed `PICKUP_GRENADES_WP` in each object's dev list and processes it later from live object state. Rust still stores a local pending command snapshot and inserts `PickupGrenadesTarget` when Shift is released. Shift accumulation for `UNIT_REPAIR_WP`, `CRANE_REPAIR_WP`, `ENTER_WP`, and `ENTER_FORT_WP` remains separate.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (609 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: Shift-held repair waypoint accumulation

- Source: `source/zplayer.cpp` `ZPlayer::AddDevWayPointToSelected` `UNIT_REPAIR_WP` / `CRANE_REPAIR_WP` branches and `SendDevWayPointsOfSelected`; `source/zplayer_events.cpp` `ZPlayer::runclick_event` and Shift `keyup_event`; source server waypoint validation in `source/zserver.cpp`.
- Rust owner: `src/selection.rs::PendingMouseCommand::{UnitRepair,CraneRepair}` stores queued repair target info and eligible repairer refs; Shift release appends movement to the repair entrance and inserts `UnitRepairTarget` or `CraneRepairTarget`.
- Runtime call site: Shift + right-click on a repair building / crane-repairable object queues the repair waypoint instead of assigning movement immediately; releasing the final Shift sends the pending repair route through the same repair processing used by immediate commands.
- Done: queued unit repair preserves source order before crane/pickup/attack, stores only repair-capable selected units from click time, validates target liveness at send time, and starts `UnitRepairStage::GotoEntrance`.
- Done: queued crane repair stores crane repair target geometry, bridge/building map bounds, target size, and starts `CraneRepairStage::GotoEntrance` after chained route insertion.
- Known difference: source validates `CanBeRepaired`, `CanRepairUnit`, and `CanBeRepairedByCrane` again when applying the waypoint. Rust currently validates target liveness on Shift release and lets the existing repair target processors drop invalid states afterward. Shift accumulation for `ENTER_WP` and `ENTER_FORT_WP` remains separate.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (610 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: Shift-held enter waypoint accumulation

- Source: `source/zplayer.cpp` `ZPlayer::AddDevWayPointToSelected` `ENTER_WP` / `ENTER_FORT_WP` branches and `SendDevWayPointsOfSelected`; `source/zplayer_events.cpp` `ZPlayer::runclick_event` and Shift `keyup_event`; source `ProcessEnterWP` / `ProcessEnterFortWP`.
- Rust owner: `src/selection.rs::PendingMouseCommand::{Enter,EnterFort}` stores queued enter target info and eligible robot refs; Shift release appends movement to the enter waypoint / fort exit point and inserts `EnterTarget` or `EnterFortTarget`.
- Runtime call site: Shift + right-click on an enterable neutral vehicle/cannon or enemy fort entrance queues the enter waypoint instead of assigning movement immediately; releasing the final Shift sends the pending enter route through the existing enter processing.
- Done: queued `ENTER_WP` validates that the target is still enterable on send, chains from earlier pending route endpoints, and starts the same `EnterTarget` component used by immediate commands.
- Done: queued `ENTER_FORT_WP` validates fort enterability for the local team on send, preserves inside/exit points, and starts `EnterFortStage::GotoEntrance`.
- Known difference: Rust now round-trips shifted movement paths through packet-level `SEND_WAYPOINTS` data, but special target commands still attach Rust target components outside full source `ProcessWaypointData`, so exact source validation/apply semantics remain separate.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (611 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: `SEND_WAYPOINTS` packet layout and movement relay

- Source: `source/zcore.cpp` `ZCore::CreateWaypointSendData` / `ProcessWaypointData`; `source/zobject.h` packed `waypoint`; `source/zplayer.cpp` `SendDevWayPointsOfObj` / `SendDevWayPointsOfSelected`; `source/zserver_events.cpp` `rcv_object_waypoints_event`; `source/zplayer_events.cpp` `set_object_waypoints_event`.
- Rust owner: `src/network_commands.rs::SendWaypointsPacket` owns the source payload layout; `src/selection.rs::source_relay_movement_path` owns the local packet round-trip for current typed movement paths.
- Runtime call site: immediate right-click move/attack/repair/enter/pickup movement routes and Shift-released pending movement route accumulation now pass through `SEND_WAYPOINTS` encode/decode before inserting `MovementPath`.
- Done: `TcpEventId::SendWaypoints=11` is represented, with source header `ref_id + waypoint_amount` and packed 15-byte waypoints (`mode`, `ref_id`, `x`, `y`, `attack_to`, `player_given`).
- Done: source waypoint modes `MOVE_WP`, `ENTER_WP`, `ATTACK_WP`, `FORCE_MOVE_WP`, `CRANE_REPAIR_WP`, `UNIT_REPAIR_WP`, `AGRO_WP`, `ENTER_FORT_WP`, `DODGE_WP`, and `PICKUP_GRENADES_WP` decode through the packet layer instead of being implicit Rust-only names.
- Done: current Rust `MovementWaypoint` paths for `MOVE_WP`, `ATTACK_WP`, and `FORCE_MOVE_WP` are runtime-wired through a local `SEND_WAYPOINTS` packet round-trip before movement assignment, preserving the source packet boundary for future server/client relay.
- Known difference: `PICKUP_GRENADES_WP`, `ENTER_WP`, `ENTER_FORT_WP`, `UNIT_REPAIR_WP`, and `CRANE_REPAIR_WP` now have source special waypoint relays but still bridge the decoded target into Rust components until full source waypoint state machines are ported.
- Known difference: `ProcessWaypointData` `CanOverwriteWP` first-waypoint preservation, `CloneMinionWayPoints`, `SetJustLeftCannon(false)`, relay-to-team delivery, `ShowWaypoints`, and source `Pcursor` rendering are still incomplete.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (613 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: `PICKUP_GRENADES_WP` special waypoint relay

- Source: `source/zplayer.cpp` `ZPlayer::AddDevWayPointToSelected` `PICKUP_GRENADES_WP` branch; `source/zcore.cpp` `CheckWaypoint` pickup validation; `source/zobject.cpp` `ProcessPickupWP`.
- Rust owner: `src/selection.rs::source_relay_pickup_grenades_path` encodes route movement plus a final source `PICKUP_GRENADES_WP` waypoint, decodes it, strips the special waypoint from `MovementPath`, and returns the decoded grenade-box ref for the current pickup target component.
- Runtime call site: immediate right-click grenade pickup and Shift-released pending grenade pickup now require the source special waypoint packet round-trip before inserting `PickupGrenadesTarget`.
- Done: manual pickup commands now encode `mode=PICKUP_GRENADES_WP`, grenade-box `ref_id`, source-style `attack_to=true`, `player_given=true`, and the clicked waypoint coordinate into the same `SEND_WAYPOINTS` payload as the route.
- Done: decoded movement waypoints remain the Bevy `MovementPath`, while the decoded final pickup waypoint becomes the authoritative target ref for `PickupGrenadesTarget`.
- Known difference: full source `ProcessPickupWP` is not ported as the owner state machine yet; arrival, `CanPickupGrenades` recheck, `CheckAttackTo`, run attempt, grenade transfer, animation packet, and box cleanup still execute through the current Rust pickup systems after the packet bridge.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (615 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: `ENTER_WP` / `ENTER_FORT_WP` special waypoint relay

- Source: `source/zplayer.cpp` `ZPlayer::AddDevWayPointToSelected` `ENTER_WP` / `ENTER_FORT_WP` branches; `source/zcore.cpp` `CheckWaypoint` enter validation; `source/zobject.cpp` `ProcessEnterWP` and `ProcessEnterFortWP`.
- Rust owner: `src/selection.rs::source_relay_special_waypoint_path` now handles final source special waypoints for enter modes, decodes the target ref, strips the special waypoint from `MovementPath`, and bridges the decoded ref into `EnterTarget` / `EnterFortTarget`.
- Runtime call site: immediate right-click enter / enter-fort commands and Shift-released pending enter / enter-fort commands now require a `SEND_WAYPOINTS` special waypoint round-trip before inserting `EnterTarget` or `EnterFortTarget`.
- Done: manual enter commands now encode `mode=ENTER_WP`, target vehicle/cannon `ref_id`, source-style `attack_to=true`, `player_given=true`, and clicked waypoint coordinate into the same `SEND_WAYPOINTS` payload as the route.
- Done: manual fort-enter commands now encode `mode=ENTER_FORT_WP`, target fort `ref_id`, source-style `attack_to=true`, `player_given=true`, and clicked waypoint coordinate before bridging to the current `EnterFortTarget` stages.
- Known difference: full source `ProcessEnterWP` and `ProcessEnterFortWP` are not ported as the owner state machines yet; `cur_wp_info`, pathfinder response, staged fort entrance/inside/exit velocity, fort destruction flag, and `CheckAttackTo` still execute through current Rust enter systems after the packet bridge.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (617 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: `UNIT_REPAIR_WP` / `CRANE_REPAIR_WP` special waypoint relay

- Source: `source/zplayer.cpp` `ZPlayer::AddDevWayPointToSelected` `UNIT_REPAIR_WP` / `CRANE_REPAIR_WP` branches; `source/zcore.cpp` `CheckWaypoint` repair validation; `source/zobject.cpp` `ProcessUnitRepairWP` and `ProcessCraneRepairWP`.
- Rust owner: `src/selection.rs::source_relay_special_waypoint_path` now handles final source special waypoints for repair modes, decodes the target ref, strips the special waypoint from `MovementPath`, and bridges the decoded ref into `UnitRepairTarget` / `CraneRepairTarget`.
- Runtime call site: immediate right-click unit repair / crane repair commands and Shift-released pending repair commands now require a `SEND_WAYPOINTS` special waypoint round-trip before inserting repair target components.
- Done: manual unit-repair commands now encode `mode=UNIT_REPAIR_WP`, repair-building `ref_id`, source-style `attack_to=true`, `player_given=true`, and repair entrance coordinate into the same `SEND_WAYPOINTS` payload as the route.
- Done: manual crane-repair commands now encode `mode=CRANE_REPAIR_WP`, repairable target `ref_id`, source-style `attack_to=true`, `player_given=true`, and crane repair center coordinate before bridging to the current `CraneRepairTarget` stages.
- Known difference: full source `ProcessUnitRepairWP` and `ProcessCraneRepairWP` are not ported as owner state machines yet; `cur_wp_info`, pathfinder response, repair-building animation packets, staged enter/exit velocity, and auto-repair flags still execute through current Rust repair systems after the packet bridge.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (619 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: source `ProcessWaypointData` waypoint validation

- Source: `source/zcore.cpp` `ZCore::ProcessWaypointData` and `CheckWaypoint`; `source/zserver_events.cpp` `rcv_object_waypoints_event`.
- Rust owner: `src/selection.rs::source_process_waypoint_data`, `source_check_waypoint`, and the existing `source_relay_*` helpers.
- Runtime call site: immediate right-click move/attack/pickup/enter/repair commands and Shift-released pending command chains now pass live object snapshots into the local `SEND_WAYPOINTS` relay before inserting `MovementPath` or special target components.
- Done: local relay now rejects wrong-team, `NULL_TEAM`, destroyed/non-waypoint-capable objects and minion objects before applying decoded waypoints, matching the server gates around `ProcessWaypointData`.
- Done: `CheckWaypoint` validation now runs for decoded `MOVE_WP`, `FORCE_MOVE_WP`, `DODGE_WP`, `AGRO_WP`, `ATTACK_WP`, `ENTER_WP`, `ENTER_FORT_WP`, `PICKUP_GRENADES_WP`, `UNIT_REPAIR_WP`, and `CRANE_REPAIR_WP`, including source client rewrites from force/dodge/agro modes into allowed runtime modes.
- Done: validation uses the already ported unit-domain gates for attack identity, enterable targets, fort entry, grenade pickup, repair-building repair, and crane repair. `MouseCommandObjectSnapshot` now carries grenade amount so source packet validation can mirror current grenade attack/pickup availability.
- Known difference: this slice validates and filters the packet payload but still does not port full waypoint apply semantics: `CanOverwriteWP` first-waypoint preservation, `CloneMinionWayPoints`, `SetJustLeftCannon(false)`, source relay-to-team delivery, `ShowWaypoints`/`Pcursor`, and full `ProcessPickupWP` / `ProcessEnterWP` / `ProcessEnterFortWP` / `ProcessUnitRepairWP` / `ProcessCraneRepairWP` ownership remain separate.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (621 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: `SEND_RALLYPOINTS` packet relay

- Source: `source/zcore.cpp` `CreateWaypointSendData`, `ProcessRallypointData`, and `CheckRallypoint`; `source/zplayer.cpp` rallypoint send path in `SendDevWayPointsOfSelected`; `source/zserver_events.cpp` `rcv_object_rallypoints_event`; `source/zserver.cpp` `RelayObjectRallyPoints`.
- Rust owner: `src/network_commands.rs::SendRallypointsPacket` owns the source packet layout; `src/units/buildings/production_logic.rs::can_set_rallypoints` owns source `BFort`/`BRobot`/`BVehicle` rally capability; `src/selection.rs::source_relay_rally_points` owns the local packet round-trip and validation.
- Runtime call site: `src/selection.rs::handle_building_rally_point_commands` now builds the candidate rally list, encodes it as `SEND_RALLYPOINTS`, decodes it, runs source-style owner/team/capability and `MOVE_WP` validation, then replaces `BuildingRallyPoints` with the accepted decoded coordinates.
- Done: `TcpEventId::SendRallypoints=12` is represented with the same `ref_id + waypoint_amount + packed waypoint[15]` payload as `SEND_WAYPOINTS`.
- Done: source `CanSetRallypoints` is represented for FortFront, FortBack, RobotFactory, and VehicleFactory; Repair/Radar/other objects reject rally packets.
- Done: rally packets preserve source `MOVE_WP`, `ref_id=-1`, `attack_to=true`, `player_given=true` waypoint shape and filter out non-`MOVE_WP` waypoint modes like `CheckRallypoint`.
- Known difference: source rallypoint line/cursor rendering through `DoRenderWaypoints(..., is_rally_points=true)` and `Pcursor` placement feedback are still not ported; this slice covers packet relay/validation/apply into current `BuildingRallyPoints`.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (626 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: source `Pcursor` command feedback

- Source: `source/zplayer.cpp` `SetPcursor`, `ShowPcursor`, and `RenderPreviousCursor`; `source/zobject.cpp` `ShowWaypoints` and `DoRenderWaypoints` final-waypoint cursor mapping; `source/cursor.cpp` `ZCursor::Process` / command cursor render shift.
- Rust owner: `src/cursor.rs::PreviousCursorState`, `PreviousCursorSprite`, and `update_previous_cursor`; `src/selection.rs::previous_cursor_kind_for_source_waypoint` owns the source waypoint-mode to previous-cursor mapping.
- Runtime call site: accepted immediate right-click move/attack/pickup/enter/repair commands, Shift-released pending command chains, and accepted production-building rally commands now show the previous command cursor for the source 3 second lifetime.
- Done: previous command cursor uses existing source cursor assets, null-team result cursor families, 4-frame `0.2s` animation, and world-position placement matching the source map-coordinate render path.
- Done: final waypoint modes map like source `ShowWaypoints`: `MOVE_WP` / `FORCE_MOVE_WP` / `ENTER_FORT_WP` / `DODGE_WP` -> `PLACED_C`, `PICKUP_GRENADES_WP` -> `GRABBED_C`, `ENTER_WP` -> `ENTERED_C`, `ATTACK_WP` / `AGRO_WP` -> `ATTACKED_C`, and repair modes -> `REPAIRED_C`.
- Done: attack command feedback uses the target object's current center when available, matching `DoRenderWaypoints` using the target object center for `ATTACK_WP`/`AGRO_WP`.
- Known difference: this slice ports the final command cursor feedback only. Full waypoint/rally path line rendering from `DoRenderWaypoints`, per-object `show_waypoints`, and exact rally rendering remain separate.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (628 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: source `DoRenderWaypoints` dotted path feedback

- Source: `source/zobject.cpp` `ProcessObject`, `ShowWaypoints`, `DoRenderWaypoints`, and `RenderWaypointLine`; `source/zplayer.cpp` `RenderObjects` rallypoint render branch.
- Rust owner: `src/selection.rs::WaypointFeedbackState`, `WaypointFeedbackPath`, `update_waypoint_feedback`, and source waypoint line helpers.
- Runtime call site: accepted immediate right-click waypoint commands, Shift-released pending waypoint chains, and open production-window rally points now render source-style dotted path feedback.
- Done: waypoint dots use source color `170,170,170`, 2x2 size, 4px segment step, `waypoint_i` phase `0..3`, and source `0.1s` phase timing.
- Done: ordinary command waypoint feedback keeps the source `ShowWaypoints` 3 second lifetime; production-window rally paths render while the building GUI is open, matching `RenderObjects` calling `DoRenderWaypoints(..., is_rally_points=true)` for the open building.
- Done: rally rendering starts at the building creation point, includes the creation move point when available, then draws stored rally points.
- Done: attack/agro waypoint line endpoints resolve through the target object's current center when the ref still exists, matching the source `GetObjectFromID` / `GetCenterCords` branch.
- Known difference at this slice: per-object overlapping `show_waypoints` lifetimes were still simplified here and are closed by the later per-object `ShowWaypoints` feedback lifetime slice. Full `cur_wp_info`, server apply semantics, and special waypoint state-machine ownership remain separate.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (630 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: source `ProcessWaypointData` first-waypoint preservation

- Source: `source/zcore.cpp` `ZCore::ProcessWaypointData`; `source/zobject.cpp` `ZObject::CanOverwriteWP`; `source/zserver_events.cpp` `rcv_object_waypoints_event`.
- Rust owner: `src/selection.rs::source_process_waypoint_data`, `source_relay_movement_path_with_existing`, `source_relay_special_waypoint_path_with_existing`, and `MouseCommandLayerSnapshot`.
- Runtime call site: accepted immediate right-click move/attack/pickup/enter/repair commands and Shift-released pending command chains now pass the current layer `MovementPath` into the local `SEND_WAYPOINTS` relay before replacing movement/special target state.
- Done: if the current first waypoint cannot be overwritten, the relay preserves that first existing waypoint before appending accepted packet waypoints, matching the source `CanOverwriteWP` preservation branch. Repeated local Shift-chain relays skip the duplicate preserved first waypoint so it is not revalidated as a client-sent `FORCE_MOVE_WP`. Current Rust can represent this fully for `FORCE_MOVE_WP`; special current-stage gates still need the full source waypoint state machines.
- Done: accepted immediate and Shift-released waypoint commands now clear `JustLeftCannon` only after the local `ProcessWaypointData` relay accepts and inserts runtime path/special-target state, matching the source `SetJustLeftCannon(false)` placement after successful `ProcessWaypointData`.
- Done: stale runtime-only relay wrapper helpers were restricted to `#[cfg(test)]`, leaving runtime wired through the existing-path variants so `dead_code` stays useful.
- Tests: added source-backed coverage for preserving an existing `FORCE_MOVE_WP` ahead of a new player move, skipping a duplicate preserved first waypoint during repeated local relays, and overwriting a normal existing `MOVE_WP`.
- Known difference at this slice: `CanOverwriteWP` current-stage rules for `CRANE_REPAIR_WP`, `UNIT_REPAIR_WP`, and `ENTER_FORT_WP` were still separate here and are closed by the later source `CanOverwriteWP` special stage gates slice. Real relay-to-team delivery and full special waypoint state-machine ownership remain separate slices.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (633 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: source `CloneMinionWayPoints` after accepted waypoints

- Source: `source/zobject.cpp` `ZObject::CloneMinionWayPoints`; `source/zserver_events.cpp` `rcv_object_waypoints_event`; production/repair creation call sites in `source/zserver.cpp`.
- Rust owner: `src/selection.rs::source_clone_minion_waypoint_paths`, `movement_path_retargeted_to_layer`, and accepted immediate/Shift `SEND_WAYPOINTS` command branches.
- Runtime call site: after a leader's local `SEND_WAYPOINTS` relay accepts a move/attack/pickup/enter/enter-fort/unit-repair/crane-repair command, Rust now clones the accepted leader path onto every live minion layer in the same robot group instead of trying to send a separate minion packet.
- Done: cloned minion paths use the same accepted waypoint list semantics as the leader, copy `attempt_run`, retarget visual-layer offsets from leader layer to minion layer, and use each minion's own `move_speed`, matching source `waypoint_list = leader.waypoint_list` followed by `SetVelocity()`.
- Done: cloned minion command state also receives the same bridged special target component (`PickupGrenadesTarget`, `EnterTarget`, `EnterFortTarget`, `UnitRepairTarget`, or `CraneRepairTarget`) where the current Rust runtime still represents special waypoint processing through components.
- Done: cloned minions clear `JustLeftCannon` after the accepted leader relay, matching source `SetJustLeftCannon(false)` before `CloneMinionWayPoints` and minion `SetJustLeftCannon(just_left_cannon)`.
- Tests: added source-backed coverage for layer-offset retargeting, minion speed usage, run-attempt copying, destroyed-minion skip, and other-group skip. The direct minion packet rejection test remains in place to preserve the source server gate.
- Known difference: this slice clones local authoritative Rust state; real `RelayObjectWayPoints` delivery and exact client/server separation are still future network slices. Full special waypoint state-machine ownership remains separate.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (635 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: per-object `ShowWaypoints` feedback lifetime

- Source: `source/zobject.cpp` `ZObject::ProcessObject`, `ShowWaypoints`, and `DoRenderWaypoints`; `source/zplayer_events.cpp` `set_object_waypoints_event`.
- Rust owner: `src/selection.rs::WaypointFeedbackState`, `WaypointFeedbackPath`, `WaypointFeedbackEntry`, and `update_waypoint_feedback`.
- Runtime call site: accepted immediate right-click waypoint commands and Shift-released pending command chains still build source-backed feedback paths keyed by object ref id; `WaypointFeedbackState::show_object_paths` now stores those paths per ref instead of replacing a single global transient list.
- Done: command waypoint feedback now mirrors source object-local `show_waypoints` lifetime by refreshing only the accepted object's 3 second render window while preserving other objects' active feedback windows.
- Done: repeated accepted commands for the same object replace that object's path and refresh its 3 second lifetime without clearing other object feedback, matching `ShowWaypoints` updating one `ZObject`.
- Done: dotted rendering still shares the existing source `RenderWaypointLine` dot shape, 4px phase, target-center resolution for attack/agro refs, and open production-window rally rendering from the prior `DoRenderWaypoints` slice.
- Tests: added source-backed coverage for overlapping object lifetimes and same-object replacement without clearing another object's remaining feedback.
- Known difference: the visual feedback is still represented in one Bevy resource rather than as fields on every runtime object entity. Full `cur_wp_info`, special waypoint state-machine ownership, and real relay-to-team packet separation remain separate slices.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (637 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: source `CanOverwriteWP` special stage gates

- Source: `source/zobject.cpp` `ZObject::CanOverwriteWP`; `source/zobject.h` `crane_repair_wp_stage`, `unit_repair_wp_stage`, and `enter_fort_wp_stage`; `source/zcore.cpp` `ProcessWaypointData` preservation gate.
- Rust owner: `src/selection.rs::SourceCurrentWaypointStage`, `source_current_waypoint_stage`, `source_can_overwrite_current_stage`, `MouseCommandLayerSnapshot`, and local `SEND_WAYPOINTS` relay helpers.
- Runtime call site: immediate right-click move/attack/pickup/enter/enter-fort/unit-repair/crane-repair commands and Shift-released pending command chains now pass current `CraneRepairTarget`, `UnitRepairTarget`, and `EnterFortTarget` stages into the source-style relay before replacing runtime path/special target state.
- Done: current `CRANE_REPAIR_WP` can only be overwritten while still in `GotoEntrance`; `EnterBuilding` and `ExitBuilding` block replacement like source.
- Done: current `UNIT_REPAIR_WP` can be overwritten during `GotoEntrance` and `Wait`; `EnterBuilding` and `ExitBuilding` block replacement like source.
- Done: current `ENTER_FORT_WP` can only be overwritten while still in `GotoEntrance`; `EnterBuilding` and `ExitBuilding` block replacement like source.
- Tests: added source-backed coverage for all current-stage overwrite rules and for local relay blocking a new waypoint while a non-overwritable `ENTER_FORT_WP` stage is active.
- Known difference at this slice: source preserves the current special waypoint and appends new packet waypoints behind it; movement/attack-compatible tails are closed by the later queue-behind slice, while queued special-after-special ownership remains future work.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (639 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: source `ProcessWaypointData` queue behind protected special stage

- Source: `source/zcore.cpp` `ProcessWaypointData` keep-first-and-push loop; `source/zobject.cpp` `CanOverwriteWP`; `source/zobject.cpp` `ProcessServer` waypoint dispatch.
- Rust owner: `src/selection.rs::source_relay_movement_path_with_existing`, `movement_path_has_prefix`, `source_should_queue_behind_current_stage`, and `insert_waypoint_command_path`.
- Runtime call site: immediate move/attack commands and Shift-released pending move/attack command chains now append movement-compatible tails behind a current non-overwritable `CRANE_REPAIR_WP`, `UNIT_REPAIR_WP`, or `ENTER_FORT_WP` stage instead of dropping the command or clearing the active special target component.
- Done: protected-stage movement relay validates new packet waypoints through the same source-style `SEND_WAYPOINTS` path, then returns `existing_path + accepted_tail`, matching the source shape where the first special waypoint remains and new waypoints are pushed after it.
- Done: repeated Shift-style relays strip an existing-path prefix before packet validation, so the current protected path is not duplicated each time the local pending route is revalidated.
- Done: command apply now preserves active `CraneRepairTarget`, `UnitRepairTarget`, and `EnterFortTarget` components when queueing behind their non-overwritable stages; normal overwriteable stages still clear old command state before applying the new path.
- Tests: updated the stage-gate relay test to assert protected-stage queueing, overwriteable-stage replacement, and no duplicate existing-path prefix on repeated relay.
- Known difference: queued special-after-special waypoints still require full special waypoint ownership; this slice only covers movement/attack-compatible tails that can be represented in `MovementPath`.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (639 passed), `./scripts/build-wasm.sh`; local `127.0.0.1:4173` server returned `200 OK`.

## Completed slice: source `DoAttackImpassableAtCoords`

- Source: `source/zobject.cpp` `ProcessMoveOrKillWP`, `ProcessMove`, and `DoAttackImpassableAtCoords`; `source/orock.cpp`, `source/ohut.cpp`, and `source/omapobject.cpp` `CausesImpassAtCoord` / `SetMapImpassables`.
- Rust owner: `src/pathing.rs::blocked_tile_at_world_for_object_kind` exposes the source-like stop tile for a robot/vehicle footprint; `src/main.rs::movement_impassable_attack_target_choice`, `source_destroyable_impassable_stop_tile`, and `insert_impassable_attack_waypoints_for_layers` own the movement-to-attack conversion.
- Runtime call site: `src/main.rs::move_commanded_objects` checks a stoppable movement probe against the passability footprint before moving. If the blocked tile belongs to a live destroyable impassable rock/hut/map-object target and the moving unit has own/leader explosives or explosive weapon damage, it inserts a front `ATTACK_WP` and preserves the previous route behind it.
- Done: rock stop coordinates match source `loc.y + 32`; hut and generic map objects use their own top-left tile. Null-team destroyable impassables remain valid `ATTACK_WP` targets, while non-explosive attackers do not get the inserted attack waypoint.
- Done: front-inserted impassable attack paths are applied to all layers of the moving object, clear stale local attack-target components if needed, and clone the resulting path to live robot minions when the blocked object is a group leader, matching the source `CloneMinionWayPoints` call after insertion.
- Tests: added source-backed coverage for blocked footprint tile reporting, rock/hut/map-object stop-tile mapping, explosives-required target choice, and matching blocked-tile selection.
- Known difference: this slice plugs the movement-time attack insertion into the current synchronous `MovementPath` model. Full source `cur_wp_info`, async pathfinder response behavior, and pathfinder planning that intentionally routes explosive units through destroyable barriers are still separate slices.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (642 passed), `./scripts/build-wasm.sh`; browser boot reached `started` and screenshot pixels were non-black on `http://127.0.0.1:4173/`.

## Completed slice: source `ProcessPickupWP` owner behavior

- Source: `source/zobject.cpp` `ProcessPickupWP`; `source/zrobot.h` `CanPickupGrenades`; `source/zobject.cpp` `IsMinion`; `source/zrobot.cpp` `DoPickupGrenadeAnim` packet side effects already wired through the existing pickup animation relay.
- Rust owner: `src/grenades.rs::process_grenade_pickups`, `source_pickup_waypoint_action`, and `find_pickup_waypoint_target`.
- Runtime call site: after `move_commanded_objects`, a live `PickupGrenadesTarget` now behaves like the source `PICKUP_GRENADES_WP` terminal waypoint instead of only being a Rust pickup bridge.
- Done: failed `CanPickupGrenades`, missing target, or non-grenade target removes `PickupGrenadesTarget`, matching source `KillWP`.
- Done: arrival still uses the source `UnderCursor(center_x, center_y)` shape via the grenade-box tile bounds; cloned robot minions now only remove their pickup waypoint on arrival and do not transfer grenades, emit pickup animation packets, or delete the grenade box.
- Done: non-minion arrival keeps the existing source packet relays for `SET_GRENADE_AMOUNT`, `PICKUP_GRENADE_ANIM`, portrait feedback, and `DESTROY_OBJECT` grenade-box cleanup, with the transfer helper still runtime-wired so `dead_code` catches future disconnects.
- Tests: added source-backed coverage for capability failure, non-grenade target, in-flight movement, minion arrival, and non-minion arrival decisions.
- Known difference: this slice ports the terminal owner behavior for the current synchronous movement model. Source `cur_wp_info`, pathfinder response waiting, and exact `CheckAttackTo` ordering inside `ProcessPickupWP` remain part of the broader waypoint/pathfinder parity work.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (643 passed), `./scripts/build-wasm.sh`; browser boot reached `started` and screenshot pixels were non-black on `http://127.0.0.1:4173/`.

## Completed slice: source `ProcessEnterWP` owner behavior

- Source: `source/zobject.cpp` `ProcessEnterWP`; `source/zobject.cpp` `IsMinion`; `source/zobject.cpp` `RobotEnterObject` side effects already wired through the existing object-team and portrait packet relays.
- Rust owner: `src/enter.rs::process_enter_targets`, `source_enter_waypoint_action`, and `EnterRequestSnapshot::is_minion`.
- Runtime call site: after `move_commanded_objects`, a live `EnterTarget` now behaves like the source ordinary `ENTER_WP` terminal waypoint for neutral vehicle/cannon entry.
- Done: missing or no-longer-enterable vehicle/cannon target removes `EnterTarget`, matching source `KillWP` after failed `CanBeEntered`.
- Done: while the robot has not reached the target `UnderCursor` rectangle, `EnterTarget` remains attached instead of being removed on the first frame after command insertion.
- Done: cloned robot minions now only remove their own enter waypoint on arrival and do not trigger capture side effects; non-minion arrival still drives the existing `SET_OBJECT_TEAM`, driver-health transfer, robot removal, group promotion, minimap update, and capture portrait packet relays.
- Tests: added source-backed coverage for the enter waypoint action helper: in-flight movement keeps the waypoint, minion arrival kills only the waypoint, and non-minion arrival applies enter.
- Known difference: this slice ports terminal owner behavior for the current synchronous movement model. Source `cur_wp_info`, pathfinder response waiting, and exact `CheckAttackTo` ordering inside `ProcessEnterWP` remain part of the broader waypoint/pathfinder parity work.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (644 passed), `./scripts/build-wasm.sh`; browser boot reached `started` and screenshot pixels were non-black on `http://127.0.0.1:4173/`.

## Completed slice: source `ProcessEnterFortWP` stage owner behavior

- Source: `source/zobject.cpp` `ProcessEnterFortWP`; `source/zobject.h` `enter_fort_wp_stage`; `source/zserver.cpp` `destroy_fort_building_ref_id` handling through fort health destruction.
- Rust owner: `src/enter.rs::process_enter_fort_targets`, `source_enter_fort_waypoint_action`, and `FortEnterStep`.
- Runtime call site: after `move_commanded_objects`, a live `EnterFortTarget` now follows the source `ENTER_FORT_WP` stage transitions for fort entry/exit in the current synchronous movement model.
- Done: missing fort targets remove `EnterFortTarget`, matching source `GetObjectFromID(..., building_olist)` failure.
- Done: `CanEnterFort` failure is checked before movement completion: during `GotoEntrance` it kills the waypoint; during `EnterBuilding` it forces `ExitBuilding` and routes to the saved exit point without destroying the fort; during `ExitBuilding` it keeps moving until the exit path finishes.
- Done: valid `GotoEntrance` completion routes to the inside point and changes to `EnterBuilding`; valid `EnterBuilding` completion sets fort health to zero, changes to `ExitBuilding`, and routes to the exit point; `ExitBuilding` completion removes the waypoint.
- Tests: added source-backed coverage for the enter-fort action helper across missing target, invalid target at each stage, active movement, valid stage transitions, and exit completion.
- Known difference: this slice ports stage owner behavior for the current synchronous movement model. Source `cur_wp_info`, pathfinder response waiting, exact `CheckAttackTo` ordering, and full source `UpdateObjectHealth`/`RelayObjectDeath` separation for fort destruction remain part of broader waypoint/network parity work.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (645 passed), `./scripts/build-wasm.sh`; browser boot reached `started` and screenshot pixels were non-black on `http://127.0.0.1:4173/`.

## Completed slice: source `ProcessUnitRepairWP` stage owner behavior

- Source: `source/zobject.cpp` `ProcessUnitRepairWP`; `source/brepair.cpp` `CanRepairUnit`, `RepairingAUnit`, `SetRepairUnit`, `GetRepairEntrance`, and `GetRepairCenter`.
- Rust owner: `src/repair.rs::process_repair_targets`, `process_unit_repair_step`, `source_unit_repair_waypoint_action`, and `target_can_repair_unit_state`.
- Runtime call site: after `move_commanded_objects`, a live `UnitRepairTarget` now follows the source `UNIT_REPAIR_WP` stage decisions for entering a repair building in the current synchronous movement model.
- Done: missing repair-building targets kill the waypoint; invalid repair building or no-longer-repairable unit kills the waypoint unless the unit is already in `EnterBuilding`, where source sends it to `ExitBuilding`.
- Done: `GotoEntrance` completion now stages to `Wait` instead of directly entering the building; `Wait` remains stable while the repair building is busy and routes to the repair center once it is free.
- Done: busy repair-building detection is evaluated even before movement completion, so a unit already in `EnterBuilding` exits back to the entrance instead of being accepted while another unit is repairing.
- Done: successful `EnterBuilding` still hides the unit, clears attack target lifecycle, starts the current `RepairingUnit` bridge for the source 5 second repair duration, and marks the building busy. `ExitBuilding` returns to `Wait` for normal rejected/queued attempts.
- Tests: added source-backed coverage for the unit-repair action helper across active movement, `GotoEntrance`, busy `Wait`, free `Wait`, valid/invalid/busy `EnterBuilding`, `ExitBuilding`, and missing targets.
- Known difference at completion of this historical slice: source `cur_wp_info`, pathfinder response waiting, exact `CheckAttackTo` ordering, repair-building animation packets, and full `SetRepairUnit` storage were still broader work. The later `BuildingRepairUnit` replacement FIFO slice removed the temporary zero-ref post-repair bridge and closed the storage/animation replacement path.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (646 passed), `./scripts/build-wasm.sh`; browser boot reached `started` and screenshot pixels were non-black on `http://127.0.0.1:4173/`.

## Completed slice: source `ProcessCraneRepairWP` stage owner behavior

- Source: `source/zobject.cpp` `ProcessCraneRepairWP`; `source/zobject.h` `crane_repair_wp_stage`; source `CanBeRepairedByCrane`; crane animation flags and the target `do_auto_repair` side effect.
- Rust owner: `src/repair.rs::process_repair_targets`, `process_crane_step`, `source_crane_repair_waypoint_action`, and `target_can_be_crane_repaired_state`.
- Runtime call site: after `move_commanded_objects`, a live `CraneRepairTarget` now follows the source `CRANE_REPAIR_WP` stage decisions in the current synchronous movement model.
- Done: missing target kills the waypoint, matching the source `GetObjectFromID(..., building_olist)` failure branch.
- Done: invalid targets kill during `GotoEntrance`, force `ExitBuilding` during `EnterBuilding`, and still finish auto-repair after `ExitBuilding` movement completes.
- Done: target validity is checked even while movement is active, matching the source ordering before `ProcessMoveOrKillWP`.
- Done: valid `GotoEntrance` completion stages into `EnterBuilding`, valid `EnterBuilding` routes to the saved exit point, and `ExitBuilding` calls the current `set_auto_repair_now` bridge before removing the waypoint.
- Tests: added source-backed coverage for missing target, invalid target per stage, active movement, valid stage transitions, and exit completion decisions.
- Known difference: at this slice Rust still used `CraneRepairTarget` presence/stage to drive the crane conco visual; that visual packet boundary is closed by the later `DO_CRANE_ANIM` slice. Full `cur_wp_info`, async pathfinder response, exact target routing/pathfinder semantics, and source `CheckAttackTo` timing remain broader waypoint/network parity work.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (647 passed), `./scripts/build-wasm.sh`; browser screenshot pixels were non-black on `http://127.0.0.1:4173/`.

## Completed slice: source `DO_CRANE_ANIM` crane repair visual packet

- Source: `source/event_handler.h` `crane_anim_packet` and `DO_CRANE_ANIM`; `source/zserver.cpp` `set_crane_anim` relay; `source/zplayer_events.cpp` `do_crane_anim_event`; `source/vcrane.cpp` `VCrane::DoCraneAnim`.
- Rust owner: `src/network_commands.rs::CraneAnimPacket`, `src/object_sync.rs::{CraneAnimPacketQueue,relay_crane_anim_state,process_crane_anim_packet_queue,apply_crane_anim_packet}`, `src/units/vehicles/crane/crane_ui.rs::CraneConcoVisualTarget`, and `src/repair.rs::process_crane_step`.
- Runtime call site: `ProcessCraneRepairWP` stage transition into `EnterBuilding` now queues source `DO_CRANE_ANIM on=true`; `ExitBuilding` completion queues `on=false`; `process_crane_anim_packet_queue` applies the packet before `sync_crane_conco_effects`.
- Done: `crane_anim_packet` uses packed source layout `ref_id:i32 + rep_ref_id:i32 + on:bool` with `DO_CRANE_ANIM=31`.
- Done: crane conco visual no longer reads `CraneRepairTarget` directly; it reads packet-applied `CraneConcoVisualTarget`, rebuilt from the current repair target object like source `rep_obj->GetCords/GetDimensionsPixel`.
- Done: old visual-only bounds were removed from `CraneRepairTarget` and `CraneRepairTargetInfo`; logic state now owns only waypoint stage/target movement points.
- Tests: added packet layout coverage and relay/apply coverage for `DO_CRANE_ANIM`.
- Known difference: this is still a local packet round-trip, not real socket transport. Source crane hook animation itself is still not modeled separately from the existing crane conco effect.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (650 passed), `./scripts/build-wasm.sh`; browser screenshot pixels were non-black on `http://127.0.0.1:4173/`.

## Completed slice: source `SET_REPAIR_ANIM` repair-building visual packet

- Source: `source/event_handler.h` `repair_building_anim_packet` and `SET_REPAIR_ANIM`; `source/brepair.cpp` `CreateRepairAnimData` / `DoRepairBuildingAnim`; `source/zserver.cpp` `UnitEnterRepairBuilding` and `RelayBuildingState`; `source/zplayer_events.cpp` `set_repair_building_anim_event`.
- Rust owner: `src/network_commands.rs::RepairBuildingAnimPacket`, `src/object_sync.rs::{RepairBuildingAnimPacketQueue,relay_repair_building_anim_state,apply_repair_building_anim_packet}`, `src/components.rs::RepairBuildingAnimState`, and `src/main.rs::process_repair_building_anim_packet_queue`.
- Runtime call site: `ProcessUnitRepairWP` successful `EnterBuilding` queues source `SET_REPAIR_ANIM on=true`; timed repair completion queues `on=false`; `animate_repair_overlays` now reads packet-applied `RepairBuildingAnimState` instead of deriving repair-building visual state directly from `RepairingUnit`.
- Done: `repair_building_anim_packet` uses packed source layout `ref_id:i32 + on:bool + remaining_time:f64 + play_sound:bool` with `SET_REPAIR_ANIM=32`.
- Done: source `DoRepairBuildingAnim(true, remaining_time)` shape is represented by inserting `RepairBuildingAnimState`, preserving/decrementing the remaining-time value and resetting the bulb/smokestack animation counters to frame 0 on start.
- Done: source client side effects are wired for local team: `COMP_STARTING_REPAIR_SND` on start, `COMP_VEHICLE_REPAIRED_SND` and `SpaceBarEvent(ref_id)` on stop.
- Tests: added packet layout coverage, relay/apply coverage, sound-path coverage, and reset-policy coverage for the repair-building animation packet.
- Known difference: this remains a local packet round-trip, not real socket transport; source `RelayBuildingState` startup/rejoin delivery is represented by the packet owner but full building-state packet follow-up streams are still part of broader object construction/socket work.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (654 passed), `./scripts/build-wasm.sh`; browser screenshot pixels were non-black on `http://127.0.0.1:4173/?canvas_fix=1782064050000`.

## Completed slice: source `EJECT_VEHICLE` driver eject packet

- Source: `source/event_handler.h` `eject_vehicle_packet` and `EJECT_VEHICLE`; `source/zplayer_events.cpp` right-click eject send; `source/zserver_events.cpp` `exit_vehicle_event`.
- Rust owner: `src/network_commands.rs::EjectVehiclePacket`, `src/object_sync.rs::{EjectVehiclePacketQueue,relay_eject_vehicle_command,apply_eject_vehicle_packet}`, `src/selection.rs::handle_mouse_commands`, and `src/main.rs::process_eject_vehicle_packet_queue`.
- Runtime call site: local right-click eject no longer inserts `EjectDriversCommand` directly; it now sends source `EJECT_VEHICLE` locally, then the packet queue applies source server gates before the existing ejection runtime creates driver robots, clears drivers, clears movement/attack state, and resets the source object to `NULL_TEAM`.
- Done: `eject_vehicle_packet` uses source layout `ref_id:i32` with `EJECT_VEHICLE=30`.
- Done: local apply mirrors the server event gates: valid packet ref, non-null object owner, player team matches owner, and object passes `CanEjectDrivers`.
- Tests: added packet layout coverage and relay/apply gate coverage for `EJECT_VEHICLE`.
- Known difference: this remains a local packet round-trip, not real socket transport; source `RelayNewObject` / `UpdateObjectHealth` announcements for newly ejected driver robots still use the existing direct runtime spawn path instead of full object-init packet streams.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (657 passed), `./scripts/build-wasm.sh`; browser screenshot pixels were non-black on `http://127.0.0.1:4173/?canvas_fix=1782064449000`.

## Completed slice: source eject-driver `SET_OBJECT_TEAM` reset

- Source: `source/zserver_events.cpp` `exit_vehicle_event`; after driver robot creation and waypoint/attack clears, source calls `ResetObjectTeam(obj, NULL_TEAM)`.
- Rust owner: `src/main.rs::{queue_eject_driver_commands,commit_eject_driver_commands,eject_object_team_reset_packet}` and existing `src/object_sync.rs::{relay_object_team_update,apply_object_team_packet,EjectDriverBatchPending,EjectDriverBatchReady}`.
- Runtime call site: after accepted `EJECT_VEHICLE`, driver ejection now builds a source `SET_OBJECT_TEAM` packet with `owner=NULL_TEAM` and empty driver info, applies it to the root object, then applies the same packet to visual layers and minimap dots.
- Done: root `ObjectTeam` now changes through the same source packet apply path as snipe, robot enter/capture, and zone ownership changes instead of being left to visual-layer-only local assignment.
- Done: ejected driver robot spawning still uses the original pre-reset owner from the accepted eject command, matching the source order where drivers are created before `ResetObjectTeam`.
- Tests: added coverage for the eject reset packet shape (`ref_id`, `NULL_TEAM`, empty driver list).
- Known difference: this still uses direct runtime spawn for the created driver robots; full `RelayNewObject` / `UpdateObjectHealth` packet streams for those new robots remain part of broader object construction/socket work.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (658 passed), `./scripts/build-wasm.sh`; browser screenshot pixels were non-black on `http://127.0.0.1:4173/?canvas_fix=1782064703000`.

## Completed slice: source eject-driver `SET_ATTACK_OBJECT` clear

- Source: `source/zserver_events.cpp` `exit_vehicle_event`; after waypoint/movement cleanup, source calls `SetAttackObject(NULL)` and `RelayObjectAttackObject(obj)`.
- Rust owner: `src/main.rs::{queue_eject_driver_commands,commit_eject_driver_commands,eject_attack_target_clear_packet,apply_attack_target_clear_packet}` and existing `src/object_sync.rs::{relay_object_attack_target,apply_object_attack_packet}`.
- Runtime call site: after accepted `EJECT_VEHICLE`, driver ejection now builds source `SET_ATTACK_OBJECT` with `attack_object_ref_id=-1` and applies it to the root object plus visual layers before clearing movement/special waypoint state.
- Done: root and layer attack target lifecycle cleanup now goes through the same source packet owner used for normal attack assignment/clear instead of direct component removal.
- Tests: added coverage for the eject attack-clear packet shape (`ref_id`, `attack_object_ref_id=-1`).
- Known difference: source also relays the emptied waypoint list and object location after `StopMove`; those packet boundaries are still separate eject cleanup slices.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (659 passed), `./scripts/build-wasm.sh`; browser screenshot pixels were non-black on `http://127.0.0.1:4173/?canvas_fix=1782064954000`.

## Completed slice: source eject-driver empty `SEND_WAYPOINTS` cleanup

- Source: `source/zserver_events.cpp` `exit_vehicle_event`; `source/zcore.cpp` `CreateWaypointSendData` / `ProcessWaypointData`; `source/zserver.cpp` `RelayObjectWayPoints`.
- Rust owner: `src/object_sync.rs::{relay_object_empty_waypoints,apply_empty_object_waypoints_packet}` and `src/main.rs::{queue_eject_driver_commands,commit_eject_driver_commands,eject_waypoint_clear_packet,apply_waypoint_clear_packet}`.
- Runtime call site: after accepted `EJECT_VEHICLE`, driver ejection now builds source `SEND_WAYPOINTS` with `waypoint_amount=0`, applies it to the root object and visual layers, and only then removes `MovementPath` plus bridged special waypoint state.
- Done: the eject cleanup no longer deletes movement/special waypoint components directly; it goes through the same packet owner used by source `GetWayPointList().clear()` / `RelayObjectWayPoints`.
- Done: the local cleanup order now keeps the source shape for the neutralized object: empty waypoint relay, attack-object clear relay, then `NULL_TEAM` object-team reset.
- Tests: added coverage for the eject waypoint-clear packet shape (`ref_id`, empty waypoint list).
- Known difference: source also calls `StopMove()` and relays `SEND_LOC` when movement was active; that object-location packet boundary is still a separate eject cleanup slice.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (662 passed), `./scripts/build-wasm.sh`; browser boot reached `started` and screenshot pixels were non-black on `http://127.0.0.1:4173/?canvas_fix=1782065484`.

## Completed slice: source eject-driver `StopMove()` / `SEND_LOC` cleanup

- Source: `source/zserver_events.cpp` `exit_vehicle_event`; `source/zobject.cpp` `StopMove` / `CreateLocationData`; `source/zserver.cpp` `RelayObjectLoc`; `source/zclient.cpp` `ProcessObjectLoc`.
- Rust owner: `src/network_commands.rs::{TcpEventId::SendLoc,ObjectLocationPacket}`, `src/object_sync.rs::{relay_object_location,apply_object_location_packet}`, and `src/main.rs::{queue_eject_driver_commands,commit_eject_driver_commands,eject_stop_move_location_packet,apply_stop_move_location_packet}`.
- Runtime call site: after accepted `EJECT_VEHICLE`, driver ejection now sends source-shaped `SEND_LOC` for objects that had active Rust movement, applies the zero-velocity location packet to the root and visual layers, then continues with `SET_ATTACK_OBJECT` clear and `SET_OBJECT_TEAM` reset.
- Done: `SEND_LOC` is represented with source payload layout `ref_id:i32 + x:i32 + y:i32 + dx:f32 + dy:f32` and event id `13`.
- Done: source `StopMove()`'s conditional relay is mapped to runtime `MovementVelocity::is_moving` with the original `0.00001` component epsilon; a queued route at rest no longer emits `SEND_LOC`, while actual movement emits a zero-velocity packet.
- Done: root and visual layers retain their relative offsets while applying the shared object-location packet, instead of collapsing all layer transforms onto the root position.
- Tests: added layout coverage for `SEND_LOC`, object-sync relay/apply coverage, and eject stop-move packet/apply coverage.
- Known difference: Rust still stores object transforms as world centers while source `object_location` stores object map coordinates; this slice preserves current Rust position through a source-shaped local packet. Full top-left source loc ownership belongs with broader object construction/location parity.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (667 passed), `./scripts/build-wasm.sh`; browser boot reached `started` and screenshot pixels were non-black on `http://127.0.0.1:4173/?canvas_fix=1783683664`.

## Completed slice: full dead-driver `UpdateObjectDriverHealth` dependency cleanup

- Source: `source/zserver.cpp::UpdateObjectDriverHealth`; `source/zobject.cpp::{Disengage,StopMove,IsMoving}`; `source/zserver.cpp::{RelayObjectAttackObject,RelayObjectWayPoints,RelayObjectLoc}`.
- Rust owner: `src/components.rs::MovementVelocity`; `src/object_sync.rs::{DriverlessObjectCleanupSnapshot,DriverlessObjectCleanupPlan,driverless_object_cleanup_plan,relay_object_waypoints,movement_path_from_object_waypoints_packet}`; `src/main.rs::{neutralize_driverless_object,apply_driverless_attack_cleanup_event,apply_driverless_waypoint_cleanup_event,move_commanded_objects}`.
- Runtime call site: after a lethal driver snipe emits `SNIPE_OBJECT`, `process_attack_targets` now applies the source event sequence `SET_ATTACK_OBJECT(target) -> SET_OBJECT_TEAM(target) -> SEND_WAYPOINTS(target) -> optional SEND_LOC(target)`, then walks all live objects in ref-id/source creation order and applies their attack clear and first target-waypoint removal events.
- Done: the dead target's own attack state is cleared only when `Disengage()` would change it; its own waypoint state is cleared only when a movement or bridged special waypoint exists.
- Done: every other object with `AttackTarget == dead_ref` receives its own `SET_ATTACK_OBJECT(-1)` clear packet.
- Done: every object whose first runtime waypoint is the normalized source `ATTACK_WP`/`AGRO_WP` for the dead ref removes only that first waypoint, relays the remaining source-shaped waypoint list, and preserves speed, run state, waypoint flags, and per-layer offsets.
- Done: `MovementVelocity` is initialized on movable object layers, updated from actual movement, zeroed for idle layers, and uses the original `IsMoving()` epsilon. `SEND_LOC` is now conditional on real velocity for both snipe and eject cleanup.
- Tests: added packet round-trip coverage for non-empty `SEND_WAYPOINTS`, full target/dependent cleanup-plan coverage, stationary attack-waypoint no-`SEND_LOC` coverage, route-tail/layer-offset apply coverage, and source movement epsilon coverage.
- Known difference: Rust still stores object transforms as world-centered layer transforms while source owns one top-left/map `object_location`; packet application preserves current root position plus explicit layer offsets. Full source top-left location ownership remains part of object construction/location parity.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (672 passed), `./scripts/build-wasm.sh`; in-app browser boot reached `started`, reported no runtime errors, and rendered a non-black game frame on `http://127.0.0.1:4173/?canvas_fix=1783685515570`.

## Completed slice: accepted empty `SEND_WAYPOINTS` group stop

- Source: `source/zserver_events.cpp::rcv_object_waypoints_event`; after accepted `ProcessWaypointData`, `SetJustLeftCannon(false)`, `CloneMinionWayPoints`, and `RelayObjectWayPoints`, an empty resulting list calls `StopMove` and unconditionally relays `SEND_LOC` for the leader and every minion.
- Rust owner: `src/components.rs::AcceptedEmptyWaypointCommand`; `src/selection.rs::insert_waypoint_command_path`; `src/object_sync.rs::{EmptyWaypointObjectSnapshot,accepted_empty_waypoint_location_packets,process_accepted_empty_waypoint_commands,ObjectLocationPacketQueue}`.
- Runtime call site: the four accepted plain immediate/Shift move and attack paths mark a server-filtered empty route after local `SEND_WAYPOINTS` round-trip; the chained object-sync system runs directly after `handle_mouse_commands`, and movement plus special-waypoint processors are ordered after it.
- Done: empty accepted routes clear movement, bridged special waypoint state, and `JustLeftCannon` on every visual layer of the leader and all live minions while preserving the current `attack_object`, matching the absence of `Disengage` in the source event.
- Done: unlike dead-driver/eject cleanup, this branch does not gate `SEND_LOC` on current velocity, matching the source's unconditional relay after `StopMove`; packets carry zero velocity and preserve every layer's relative offset.
- Done: a marker made stale by a later accepted special command in the same deferred Shift batch is discarded without stopping the group or deleting the later command.
- Tests: added server-filtered-empty acceptance, current-attack preservation, mixed empty/special deferred apply, world/map axis conversion and layer-offset coverage, deterministic leader/minion packet coverage, destroyed/other-group exclusion, direct-minion rejection, and a Bevy runtime group-stop regression.
- Known difference: relay/apply is still local; real team/socket delivery and unified cross-event ordering remain future engine slices. Full source top-left location ownership is closed by the following generic client `SEND_LOC` slice.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (678 passed), `./scripts/build-wasm.sh`; in-app browser boot reached `started`, reported no runtime errors, and rendered a non-black game frame on `http://127.0.0.1:4173/?canvas_fix=1783687883689`.

## Completed slice: generic client `SEND_LOC` apply / `SetLoc` + `SmoothMove`

- Source: `source/zclient.cpp::ProcessObjectLoc`; `source/zobject.cpp::{SetLoc,SmoothMove,ProcessMove,CreateLocationData}`; `source/common.h::isz`.
- Rust owner: `src/components.rs::{SourceObjectLocation,SourceLocationInterpolation}`; `src/object_sync.rs::{ObjectLocationPacketQueue,relay_object_location,process_object_location_packet_queue,smooth_object_locations}`; source-location initialization in `src/world_objects.rs`; movement authority and scheduling in `src/main.rs`; packet-interpolation cancellation for new commands in `src/selection.rs`.
- Runtime call site: all queued local `SEND_LOC` packets now enter one client apply path before movement; accepted-empty group stops use that same queue instead of a cleanup-specific transform bridge.
- Done: `object_location.x/y` are canonical source top-left map coordinates; visual root anchors and per-layer offsets are stored separately, so packet apply no longer confuses source loc with Bevy sprite centers.
- Done: packet velocity updates source/world velocity, recalculates direction only when velocity changes, and installs source-shaped interpolation with the first tick held at the packet location.
- Done: `SmoothMove` uses `floor(dx * elapsed)` / `floor(dy * elapsed)`, while local `ProcessMove` synchronization keeps integer loc plus separate `xover/yover` remainders.
- Done: packet-driven entities are excluded from local `MovementPath` simulation until a newly accepted local command explicitly removes packet interpolation; pending or completed destruction rejects later location apply.
- Tests: added packet anchor/layer-offset, same-velocity direction, first-tick/interpolation floor, pending-destroy, accepted-empty shared-queue, stale-marker, destroyed-marker, and local remainder accumulation regressions.
- Known difference: the local packet resources still approximate cross-event ordering instead of consuming one real socket event stream, and Bevy time is not yet split into the source-compatible scaled simulation clock used by every subsystem.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (682 passed), `./scripts/build-wasm.sh`; in-app browser reported no runtime errors and rendered a non-black game frame on `http://127.0.0.1:4173/?canvas_fix=1783690577634`.

## Completed slice: runtime `ADD_NEW_OBJECT` and stored-cannon placement

- Source: `source/zclient.cpp::ProcessNewObject`; `source/zplayer_events.cpp::add_new_object_event`; `source/zserver.cpp::RelayNewObject`; `source/zserver_events.cpp::place_cannon_event`; `source/zbuilding.cpp::{CreateBuiltCannonData,ProcessSetBuiltCannonData}`; cannon `JUST_PLACED_MODE` render/process owners.
- Rust owner: `src/network_commands.rs::{ObjectInitPacket,BuiltCannonListPacket}`; `src/object_sync.rs::{SourceObjectEventQueue,relay_new_object,relay_built_cannon_list,process_source_object_event_queue}`; `src/world_objects.rs::spawn_runtime_object_from_source_init`; `src/placement.rs::process_cannon_placement`; placement animation state/profiles in `src/units/cannons/**/**_ui.rs` and atlas lookup in `src/render/atlas.rs`.
- Runtime call site: accepted local cannon placement emits FIFO events `ADD_NEW_OBJECT -> SET_BUILT_CANNON_AMOUNT`; the chained client apply runs immediately after placement and before ordinary mouse commands.
- Done: the 22-byte object-init packet now constructs runtime ECS objects from source top-left x/y, ref id, owner, building level, and extra links; packet health is intentionally ignored until a health event, matching `ProcessNewObject`.
- Done: source-created objects use a separate gameplay root at `loc + logical_size/2`; sprite layers retain source render offsets, so selection, attack distance, obstacles, minimap, movement sync, destruction, and visuals no longer share a cropped-frame center. Cannon team capture updates the separated visual layer and cancels any obsolete placement animation.
- Done: placement rechecks current local-player ownership, clears placement mode after any left-click request, rejects duplicate refs before server mutation, and commits the post-placement built-cannon list only when its dependent ADD succeeds.
- Done: `SET_BUILT_CANNON_AMOUNT=25` uses source `ref_id + count + cannon ids` layout and replaces the building list through the production owner.
- Done: locally owned placed cannons run source `JUST_PLACED_MODE`: three shared init frames plus four cannon/team frames at 0.1 seconds with overshoot discard, then the occupied passive frame; enemy cannons start directly on the passive frame.
- Tests: packet layout/length, source type mapping, negative coordinates, duplicate refs, FIFO event order, dependent cannon-list apply, exact top-left center, render offsets, and placement animation profile/timing regressions.
- Known difference: startup map objects still bypass this runtime constructor, and production/ejection/repair creation still need their source-ordered `OBJECT_GROUP_INFO`, health, waypoint, driver, building, grenade, queue, and rally follow-ups migrated into the same FIFO.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (688 passed), `./scripts/build-wasm.sh`; in-app browser reported no runtime errors and rendered a non-black game frame on `http://127.0.0.1:4173/?canvas_fix=1783694220208`.

## Completed slice: ejected-driver robot group source FIFO batch

- Source: `source/zserver_events.cpp::exit_vehicle_event`; `source/zserver.cpp::{CreateRobotGroup,RelayNewObject,RelayObjectGroupInfo,UpdateObjectHealth}`; `source/zobject.cpp::{CreateGroupInfoData,ProcessGroupInfoData}`; `source/zplayer_events.cpp::set_object_team_event`.
- Rust owner: `src/object_sync.rs::{SourceObjectEventQueue,relay_ejected_driver_group,relay_ejected_driver_batch_commit,process_source_object_event_queue,EjectDriverBatchPending,EjectDriverBatchReady}`; `src/world_objects.rs::spawn_runtime_object_from_source_init`; `src/main.rs::{queue_eject_driver_commands,commit_eject_driver_commands}`.
- Runtime call site: an accepted local `EJECT_VEHICLE` now reserves the complete driver group, emits each member's source events, applies all new objects and follow-ups in the same chained frame, and only then neutralizes the carrier through the already packetized waypoint/location/attack/team cleanup.
- Done: every leader/minion emits `ADD_NEW_OBJECT -> OBJECT_GROUP_INFO -> UPDATE_HEALTH` in source member order; health packets carry actual integer driver health rather than a Rust health percentage, and every object uses the carrier's canonical source top-left coordinates.
- Done: client apply pre-reserves every accepted gameplay entity id, then replays the original event queue in exact order. Leader group metadata can therefore target not-yet-populated minion roots without moving `OBJECT_GROUP_INFO`/`UPDATE_HEALTH` behind later `ADD_NEW_OBJECT` events; health still requires its own completed ADD, and carrier cleanup requires the complete member set.
- Done: duplicate/live/queued refs and signed wire overflow are rejected before producer mutation. `NextObjectRefId`, pending carrier cleanup, source selection removal, and the `NULL_TEAM` reset are committed only for a successfully relayed batch; a failed client batch drops the pending carrier commit instead of deleting its drivers.
- Done: the source empty-driver branch emits no robot/waypoint/location/attack work and performs only the team reset. A successful reset deselects the carrier without selecting the new leader, matching `set_object_team_event`.
- Compatibility policy: Rust keeps the client-intended `ref_id, leader_ref_id, count, minion_refs...` layout and resolves forward refs. It intentionally does not copy the source defects that overwrite the count at `[2+i]`, read minions from `[2+i]`, set minion health/`JustLeftCannon` on the leader, or fail to increment `driver_i`; each ejected member receives its own health and cannon-exit marker.
- Tests: added exact nine-wire-event FIFO coverage for a three-member group, commit ordering, ref collision/wire overflow and dependency readiness, full queue rollback, corrected group assignments, actual-health conversion, and source carrier deselection regressions.
- Known difference: transport remains the shared local source-event queue rather than a real socket stream; the documented corrections preserve playable source intent instead of reproducing memory-index and wrong-target bugs literally.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (693 passed), `./scripts/build-wasm.sh`; independent source-parity re-review returned no findings, and the in-app browser reached `started` with no runtime errors and a non-black game frame on `http://127.0.0.1:4173/?canvas_fix=1783702800000`.

## Completed slice: production robot/vehicle source FIFO batch

- Source: `source/zserver.cpp::{BuildingCreateUnit,RelayNewObject,RelayObjectManufacturedSound,ProcessObjects}`; `source/zobject.cpp::CloneMinionWayPoints`; `source/zserver.cpp` constructor/`CreateObject` ref-id ownership.
- Rust owner: `src/object_sync.rs::{relay_produced_object_batch,process_source_object_event_queue,ProducedObjectBatchPending,ProducedObjectBatchReady}`; `src/units/buildings/production_logic.rs::production_source_map_points`; `src/main.rs::{process_building_production,commit_produced_object_batches}`; source-owned spawning in `src/world_objects.rs`.
- Runtime call site: a completed robot/vehicle factory item reserves the complete ref range, queues the source creation stream, drains it after production, then commits manufactured feedback and authoritative local routes only after every new gameplay root exists.
- Done: robot groups emit `ADD_NEW_OBJECT -> OBJECT_GROUP_INFO` for leader and each minion in exact member order; vehicles emit one `ADD_NEW_OBJECT`; no Rust-only production `UPDATE_HEALTH` event is added because source does not send one here.
- Done: the leader alone receives the source `SEND_WAYPOINTS` packet with one `FORCE_MOVE_WP`; all members retain the cloned factory-exit route locally, while only the leader receives the building rally tail after manufactured feedback, matching source server state/order.
- Done: packet coordinates use source top-left object dimensions (16x16 robots, 32x32 vehicles), source map/world axis conversion, and source random vehicle direction. Visual layer paths preserve their offsets and exclude non-moving lid/driver overlays.
- Done: manufactured `COMP_MSG` now targets the actual `LocalPlayerState` team instead of assuming Red, and source-owned dynamic vehicles join root lid state to separate sprite layers by ref id.
- Done: startup and dynamic object refs now begin at source ref `0`; post-repair exit uses an explicit state flag instead of reserving zero as a sentinel.

## Completed slice: source `BuildingRepairUnit` replacement FIFO

- Source: `source/brepair.cpp::{SetRepairUnit,BuildingRepairUnit,CreateRepairAnimData}`; `source/zserver.cpp::{UnitEnterRepairBuilding,DeleteObject,RemoveObjectFromGroup,RelayBuildingState}`; `source/zobject.cpp` driver and waypoint storage.
- Rust owner: `src/components.rs::{UnitRepairTarget,RepairBuildingOccupancy}`; `src/repair.rs::{process_repair_targets,advance_repair_building_occupancy,process_repair_building_step,repaired_driver_state}`; `src/object_sync.rs::{relay_deferred_delete_object,relay_repaired_object_batch,process_source_object_event_queue_early,process_source_object_event_queue_late}`; `src/units/buildings/repair/repair_logic.rs::{repaired_unit_source_points,repaired_unit_waypoints}`; `src/main.rs::commit_repaired_object_batches`.
- Done: accepted repair entry snapshots the unit kind, full driver state, repair center/entrance, and queued waypoint tail into the repair building, then defers source group cleanup, grenade transfer, old group clear, and `DELETE_OBJECT` until the next early source drain.
- Done: the absolute five-second deadline continues while the repair building is destroyed, while saved source driver attack timestamps remain unchanged; object recreation waits for revival and allocates fresh refs only when completion can run. An empty saved driver list keeps the source-created default driver.
- Done: completion emits robot members as `ADD_NEW_OBJECT -> OBJECT_GROUP_INFO` or a vehicle as `ADD_NEW_OBJECT`, sends the exit `FORCE_MOVE_WP` plus preserved route metadata only to the owner, applies leader `SET_OBJECT_TEAM` with driver info to all clients, and commits routes only after the guarded FIFO succeeds.
- Done: repaired-object creation is applied before `SET_REPAIR_ANIM on=false`, repaired sound, and spacebar feedback; simultaneous building completions use stable source/building ref order.
- Done: after current-frame damage/health apply, mixed repair/factory completions reserve fresh refs in one source building/object-ref order without consuming `NextObjectRefId` until a batch succeeds; protected repair stages store later move/attack commands as the post-repair route, and restore that route if the repair waypoint is cancelled.
- Done: `OBJECT_GROUP_INFO` apply stores explicit packet-derived member indexes, so leader promotion and grenade transfer follow source `minion_list` order after ECS archetype changes; `DELETE_OBJECT` also clears the complete robot attack lifecycle from attackers.
- Known difference: the local in-process FIFO represents transport ordering until real sockets exist. Queued special-after-special waypoint semantics remain part of the broader `ProcessWaypointData` gap.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, 709 tests, WASM release build, independent re-review with no findings, and non-black in-app browser frame at `http://127.0.0.1:4173/?canvas_fix=1783716600000`.
- Tests: exact robot/vehicle FIFO order, rollback, unsupported kinds, map coordinates, no production health packet, source-zero object-init/group/health refs, local-team manufactured feedback, route/layer offsets, and separated dynamic vehicle lid ownership.
- Known difference: transport is still the shared local source-event FIFO rather than a real socket stream; source server rally state is represented directly in the unified local simulation after the client-visible initial waypoint event.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, `cargo test -q` (698 passed), `./scripts/build-wasm.sh`; independent source-parity findings were closed, and the in-app browser reached `started` with a non-black game frame on `http://127.0.0.1:4173/?canvas_fix=1783711200000`.

## Completed slice: typed `SET_SETTINGS` runtime ownership

- Source: `source/zsettings.h` packed `ZUnit_Settings`/`ZSettings`; `source/zserver_events.cpp::request_settings_event`; `source/zclient.cpp::ProcessZSettings`; `source/zobject.cpp` constructor stat copy; `source/vapc.cpp::SetAttackObjectDriver`; production group/build-time reads through `ZSettings`.
- Rust owner: `src/settings_sync.rs::SourceSettingsState` decodes the exact 1420-byte layout into unit settings, building/item health ratios and grenade settings; per-unit defaults remain in `src/units/**/**_logic.rs` as the source-default serializer and fallback owner.
- Runtime call sites: settings handshake runs before startup object construction; `src/world_objects.rs` creates `ObjectStats` and stamina from the decoded state; `src/object_sync.rs` uses it for startup/dynamic object health and movement; combat uses it for APC driver stats and grenades; production uses it for build time, group count and ref reservation; zone links use the same group counts.
- Done: custom packet values now change max/current health scaling, movement, attack damage/radius/chance/speed, missile/snipe values, run stamina, grenade delivery, robot group sizes, startup/dynamic ref allocation and production timing/counts instead of being stored as opaque bytes.
- Done: startup zone-link refs now share source ref `0` ownership and source-settings robot expansion, removing the stale one-based zone reference calculation.
- Compatibility: invalid negative/chance/non-finite values are censored at the typed boundary like `ZSettings::CensorSettings`; unsupported object kinds retain their concrete unit/item defaults.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, 715 tests, WASM release build, and a rendered non-black WASM game frame in `output/playwright/source-settings-runtime.png`.

## Completed slice: source `GWFactoryList`

- Source: `source/gwfactory_list.cpp::{Process,DoRender,DoRenderEntries,DetermineHeight,Click,UnClick,CollectEntries,AddEntry}`; `source/zplayer.cpp::{B_Button,HandleKeyPress,RenderNews}`.
- Rust owner: `src/factory_list.rs` owns visibility, source asset paths, entry collection/order, screen geometry, rows, scrolling, pointer hit testing and focus/open commands; `src/selection.rs::process_hud_commands` routes the B HUD command to the list.
- Done: B HUD and keyboard B toggle the list; owned Fort factories precede Robot then Vehicle factories; every entry renders source health, production and tech rows with percent/time text and original `factory_gui` assets.
- Done: panel height follows the source available-view calculation, anchors to the game viewport bottom-left, scrolls by wheel/buttons, consumes covered pointer input, and clicking an entry focuses its building and opens the matching production window.
- Removed: the old B-button shortcut that opened only the currently selected production building; source B behavior is now authoritative.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, 716 tests, WASM release build, and the visible runtime list in `output/playwright/factory-list-runtime-settled.png`.

## Completed slice: startup minimap source-ref ownership

- Source: `source/zmini_map.cpp` reads the live `object_list`; startup object identity comes from `source/zserver.cpp::RelayNewObject` rather than map-file indexes.
- Rust owner: `src/world_objects.rs::spawn_minimap_dot` is now the only minimap-dot constructor for startup and dynamic objects through the shared runtime `ADD_NEW_OBJECT` path.
- Done: removed the parallel `src/hud.rs` map-enumeration dots that used stale `ref_id + 1`, duplicated runtime dots and could not represent expanded robot groups.
- Done: startup/dynamic dots now use exact source-zero refs, one dot per created robot group member, canonical runtime positions and team updates; map-item rocks retain the original minimap exclusion.
- Verification: `RUSTFLAGS="-Ddead_code" cargo check -q`, 716 tests and WASM release build.

## Completed slice: source `ZTime` virtual game clock

- Source: `source/ztime.cpp::{UpdateTime,SetGameSpeed,Pause,Resume}` and the client/server frame loops that keep wall time moving while simulation time is paused or scaled.
- Rust owner: `src/game_speed.rs::{sync_source_game_time,apply_source_game_time_control}` applies `GamePauseState` and `GameSpeedState` to Bevy `Time<Virtual>` in `First`, after Bevy updates its clocks; pause and speed packet handlers also apply the new clock state immediately.
- Done: gameplay movement, combat, projectiles, production, repairs, object animations and simulation effects now consume one frozen/scaled virtual clock, matching source `ztime.ztime` ownership instead of independently advancing on wall time.
- Done: vote expiration, HUD news lifetime, cursor/previous-cursor timing, spacebar event selection and camera input explicitly consume `Time<Real>`, so client interaction and server-style deadlines remain available while gameplay is paused.
- Compatibility: invalid/non-finite speed becomes zero and valid speed is clamped non-negative at the clock boundary; the packet/state resources remain the authoritative source for pause and relative speed.
- Runtime evidence: the browser game starts paused with a stable non-black frame and `/resume` removes the pause gate and resumes world simulation (`output/playwright/source-time-paused.png`, `output/playwright/source-time-resumed.png`).

## Completed slice: source `BlitHitSurface` pixel parity

- Source: `source/zsdl.cpp::ZSDL_BlitHitSurface`, `source/zsdl_opengl.cpp::ZSDL_Surface::BlitHitSurface`, robot render paths and `source/zvehicle.cpp` body/driver hit layers.
- Rust owner: `src/render/hit_surface.rs` owns source alpha-to-white conversion, one cached hit image per original Bevy image asset, one-frame swap metadata and animation-safe restoration; `src/main.rs` owns the existing `UPDATE_HEALTH` and `DRIVER_HIT_EFFECT` packet-to-layer routing.
- Done: the old `Color::srgba(4,4,4,1)` multiplication was removed because dark source pixels did not become white. RGBA/BGRA assets now keep alpha-zero pixels transparent and replace every nonzero-alpha pixel with opaque white exactly like the SDL pixel loop; luminance-alpha and luminance images have equivalent handling.
- Done: hit surfaces are applied in `PostUpdate`, after gameplay/unit animation has selected the current frame, and restored on the following `PostUpdate`; atlas coordinates and standalone-frame changes remain authoritative, repeated refs are deduplicated, and overlapping whole-object/driver hits retain the same original image.
- Compatibility: unsupported GPU pixel formats are rejected rather than reinterpreted; the game's loaded BMP/PNG sprite formats use the covered 8-bit layouts. The cache preserves original image dimensions, sampler, atlas indexing and WASM-compatible ordinary sprite rendering.
- Runtime evidence: `output/playwright/source-hit-surface-runtime.png` records the resumed non-black WASM world with the new render owner active.

## Completed slice: non-pause vote descriptions

- Source: `source/zvote.h::vote_type_string`, `source/zcore.cpp::VoteAppendDescription`, `source/zvote.cpp::SetupImages`, `source/zserver.cpp::StartVote` and `source/zplayer_events.cpp::set_player_voteinfo_event`.
- Rust owner: `src/vote.rs::{VoteType,source_vote_append_description,source_vote_description,vote_display_snapshot}` owns all eight wire ids/labels and append rules; `src/hud.rs::update_hud_vote_display` supplies live `SelectableMapListState` to that owner.
- Done: `VOTE_INFO` types 0 through 7 now decode as Pause, Resume, Change Map, Start Bot, Stop Bot, Reset Game, Reshuffle Teams and Set Game Speed instead of dropping every non-pause/non-speed vote.
- Done: Change Map renders `Change Map: <index>. <map name>` only for a valid current selectable-list index; Start/Stop Bot render the exact lower-case source team name for ids 0 through 8; types without source append text render only their base label.
- Done: vote display snapshots own a dynamic string rather than a static partial label, so later player vote-info refreshes preserve the source append description while counts change.
- Remaining boundary: this closes client-side packet/UI description parity. Chat request commands and passed-vote runtime actions for map/bot/reset/reshuffle remain in the stateful `ProcessPlayerCommand` and map/bot lifecycle slices.

## Completed slice: non-pause `ProcessPlayerCommand` vote requests

- Source: `source/zserver_commands.cpp::{PlayerCommand_ChangeMap,PlayerCommand_StartBot,PlayerCommand_StopBot,PlayerCommand_ResetGame,PlayerCommand_ReshuffleTeams}`, `source/zserver_events.cpp` event handlers and `source/event_handler.h` ids 79 through 83.
- Rust owners: `src/chat.rs` owns source argument parsing/team errors; `src/network_commands.rs` owns `ReshuffleTeamsCommand`, `StartBotCommand`, `StopBotCommand`, `SelectMapCommand`, and `ResetMapCommand`; `src/vote.rs` owns typed request/action queues and the common vote server path; `src/main.rs` owns local packet relay/apply order.
- Done: `/changemap` uses source `atoi`, `/startbot` accepts the exact eight non-null lower-case team names, `/stopbot` filters against active bot teams and preserves the original source's `start bot error` wording, and missing input uses `command error: invalid input(s)`.
- Done: requests round-trip through source payload shapes (`team`/`map_num` as one little-endian `int_packet`, reset/reshuffle as empty payload) before `StartVote`; invalid map indexes produce `invalid map choice, please type /listmaps`, and invalid direct team packets are rejected silently.
- Done: multi-player votes broadcast the already-wired map/team append description; passed Start/Stop Bot actions update `LocalBotTeams`, so later `/stopbot` validation reads live state rather than a hardcoded list. Map/Reset/Reshuffle passed actions remain ordered in `NonPauseVoteActionQueue` for their world-lifecycle owner.
- Runtime evidence: `output/playwright/non-pause-vote-command-runtime.png` records the WASM command path with the source invalid-map response and a live non-black world.
- Remaining boundary: real bot processes/AI and the map reset/reshuffle world mutation are separate engine slices; this slice owns their command, packet, vote and ordered action boundary only.

## Completed slice: stateful `/changeteam`

- Source: `source/zserver_commands.cpp::PlayerCommand_ChangeTeam`, `source/zserver.cpp::ChangePlayerTeam`, `source/zserver_events.cpp::set_team_event`, `source/zplayer.cpp::SetPlayerTeam` and `SET_TEAM`/`SET_LPLAYER_TEAM` packet layouts.
- Rust owners: `src/chat.rs::player_command_change_team` owns command parsing/errors; `src/local_player.rs::relay_change_player_team` owns server request, local-player roster relay and news; `src/main.rs::process_chat_input` owns client team-change side effects.
- Done: missing input, invalid exact lower-case team names and same-team requests emit the source strings; all nine source team names including `null` are valid command values, while bot commands retain their separate non-null rule.
- Done: a valid command round-trips the existing four-byte `SetTeamCommand`, applies the authoritative team, then round-trips `SetLocalPlayerTeamPacket` for the roster before emitting `you have been set to the <team> team` and `<name> has changed from ...` in source order.
- Done: successful team apply clears selected refs and spacebar events like `ZPlayer::SetPlayerTeam`; cursor/HUD/computer/factory owners already read `LocalPlayerState::team`, so no parallel palette or UI team state is introduced.
- Compatibility: this local transport has one connected player, but the roster packet remains the same broadcast-shaped `(player_id, team)` owner used by future socket transport.

## Completed slice: account `ProcessPlayerCommand` lifecycle

- Source: `source/zserver_commands.cpp::{PlayerCommand_Login,PlayerCommand_Logout,PlayerCommand_CreateUser}`, `source/zserver_events.cpp::{player_login_event,request_login_required_event,create_user_event}`, `source/zserver.cpp::{AttemptPlayerLogin,AttemptCreateUser,LoginPlayer,LogoutPlayer,SendPlayerLoginRequired}`, `source/zcore.h::p_info::logout` and ids 62-65.
- Rust owners: `src/chat.rs` owns source comma parsing; `src/network_commands.rs` owns `SendLoginCommand`, `RequestLoginOffCommand`, `GiveLoginOffPacket`, and `CreateUserCommand`; `src/account.rs` owns server settings/store/validation; `src/local_player.rs` owns roster-visible login/logout apply.
- Done: `/login` requires two comma fields, `/createuser` four, trims only initial spaces per field like `ParseCommandContents`, and reports missing values as `command error: invalid input(s)`; `/logout` requires no payload.
- Done: login/create ASCII payloads are NUL-terminated source CSV, login-required request is empty and response is one packed bool. Account fields preserve source allowed characters, double/edge-space rejection and username/login/password/email size limits.
- Done: default `use_database=false` returns the original `no database used` messages. `ZOD_USE_DATABASE=1` enables a deterministic in-process server store: duplicate/invalid/login-state checks use source strings, create auto-logs in, and login/logout relay player name, loginfo and current vote choice through the existing roster packets.
- Done: `ZOD_REQUIRE_LOGIN=1` drives `GIVE_LOGINOFF.show_login = !logged_in`; `LoginPromptState` is the packet apply boundary for the future original login/create-user windows.
- Compatibility: MySQL persistence, affiliate/history updates and cross-connection `LogoutOthers` require the real server/database transport; they are not replaced by invented browser persistence.

## Completed slice: `/buyregistration` key lifecycle

- Source: `source/zserver_commands.cpp::PlayerCommand_BuyRegistration`, `source/zplayer_events.cpp::{poll_buy_regkey,set_regkey}`, `source/zserver_events.cpp::buy_regkey_event`, `source/zencrypt_aes.cpp`, `source/zcore.cpp::InitEncryption`, `buy_registration_packet`, `REGISTRATION_COST=1` and ids 70-72.
- Rust owners: `src/network_commands.rs::{PollBuyRegistrationKeyPacket,BuyRegistrationKeyCommand,ReturnRegistrationKeyPacket}` owns exact wire shapes; `src/account.rs::{RegistrationState,process_buy_registration}` owns client/server flow; `src/local_player.rs::relay_spend_voting_power` owns roster-visible cost apply.
- Done: `/buyregistration` emits an empty poll; already-registered clients stop with the source message, otherwise a deterministic platform device id is sent as exactly 16 bytes. The server applies source configured/logged-in/activated/one-VP gates and messages before changing state.
- Done: successful purchase subtracts one voting power from both account server state and `SET_LPLAYER_LOGINFO`, encrypts the 16-byte device block with the original fixed AES-128 key, round-trips `RETURN_REGKEY`, decrypt-verifies it client-side and emits the source congratulations/error result.
- Done: ids 70/71/72 and empty/16-byte payload rejection are covered explicitly; the AES implementation is shared by native and WASM rather than replacing registration with a flag-only shortcut.
- Compatibility: registration packet/AES ownership is shared by the native `registration.zkey` and browser localStorage platform stores documented in the later persistence slice; the deterministic default device id replaces unavailable browser MAC access and is isolated at that boundary.

## Completed slice: final dead-driver relay-clear audit

- Source: `source/zserver.cpp::UpdateObjectDriverHealth`, specifically `SNIPE_OBJECT -> target Disengage/SET_ATTACK_OBJECT -> ResetObjectTeam -> optional SEND_WAYPOINTS/SEND_LOC -> dependent Disengage/first ATTACK_WP-or-AGRO_WP removal`.
- Rust owner: `src/object_sync.rs::driverless_object_cleanup_plan` produces target/dependent packets; `src/main.rs::neutralize_driverless_object` applies attack clear before team reset, then target waypoint/location, then source-ref-ordered dependents.
- Confirmed: dependent objects clear `AttackTarget` independently from removing only a matching first normalized attack/agro waypoint; objects satisfying both relay attack clear before waypoint clear, preserve the remaining route, and stop/relay location only when movement was active.
- Fixed: the sniped target previously produced a stop-location packet whenever it was moving, even with no source waypoints. Source calls `StopMove()` only inside `if(GetWayPointList().size())`; Rust now requires the same `target_has_waypoints && is_moving` condition.
- Done: the stale engine-gap entry for exact dead-driver attack/waypoint clears is closed; the regression covers a moving target with no waypoints and therefore no `SEND_WAYPOINTS` or `SEND_LOC`.

## Completed slice: passed Change Map, Reset Game and Reshuffle Teams actions

- Source: `source/zserver.cpp::{DoResetGame,ResetGame,LoadNextMap,ReshuffleTeams,ChangePlayerTeam,ProcessVote}`, including `ClearMap -> DeleteAllObjects -> Read map -> InitObjects -> InitZones -> PauseGame`, `A new game has started`, empty `RESET_GAME` event 19, logged-in-first player ordering, available object-team discovery and active-bot exclusion.
- Rust owners: `build.rs`/`src/selectable_maps.rs` embed every selectable `.map` for native and WASM lookup; `CurrentMapSource` retains the authoritative current bytes/name/generation; `src/main.rs::{process_world_vote_actions,queue_runtime_map_reset,apply_runtime_map_reset,spawn_map_contents}` owns the common startup/runtime reconstruction path; `src/local_player.rs::relay_reshuffle_player_teams` owns roster packet application.
- Done: a passed `/resetgame` replays the source chunked map transfer against the current bytes, parses fresh map/tile metadata, round-trips event 19, clears session visuals and object queues, pauses source virtual time, then rebuilds terrain, rocks, ambient objects, passability, startup object packets, zones, minimap and HUD through the same owner used at startup.
- Done: a passed `/changemap <zero-based index>` resolves the generated source selectable list to embedded bytes and runs the same reset path. All 57 source maps are available in WASM without filesystem access; the camera is recentered and the selected map becomes the next reset source.
- Done: teardown is restricted to session entities with transforms. The Bevy window, cameras, cursor entities and non-visual engine/observer entities survive; reconstruction runs as one exclusive world transaction so no gameplay system can observe the cleared half-state.
- Done: `/reshuffleteams` discovers teams represented by robot/vehicle/cannon objects, removes teams with active bots, assigns logged-in players before other player-mode clients, replenishes the available-team bag exactly when exhausted, relays `SET_LPLAYER_TEAM` and local `SET_TEAM`, and emits the three source success/error strings.
- Runtime evidence: `output/playwright/runtime-reset-exclusive-after.png` shows the complete same-map world after Reset; `output/playwright/runtime-changemap-exclusive-after.png` shows a distinct map after Change Map; `output/playwright/runtime-reshuffle-after.png` shows the roster/team news over the rebuilt map.
- Remaining boundary: real remote socket peers/server processes, configured server map-list folders and real bot AI/processes remain separate. The local/native/WASM world action is no longer a queued placeholder.

## Completed slice: persistent `registration.zkey` ownership

- Source: `source/zcore.cpp::CheckRegistration`, `source/common.cpp::file_can_be_written` and `source/zplayer_events.cpp::{poll_buy_regkey,set_regkey}`.
- Rust owner: `src/account.rs::RegistrationState` now distinguishes test/in-memory storage from the runtime platform store, checks writability before sending `BUY_REGKEY`, persists the returned exact 16-byte encrypted block, reloads it through the same validation path and compares the AES-decrypted bytes with the device id.
- Native: startup reads `registration.zkey` from the working directory, or `ZOD_REGISTRATION_KEY` when supplied; write uses the source 16-byte file shape and corrupt/short files remain unregistered. The source save-failure and troubleshooting messages are preserved.
- WASM: the same encrypted block is encoded as 32 lowercase hex characters under `zod.registration.zkey` in browser localStorage. A write/remove probe supplies the browser equivalent of source append-open writability; startup decode rejects any non-16-byte representation.
- Done: `RegistrationState::load_platform` runs during Bevy app construction, so persisted registration affects the first `/buyregistration` poll rather than only the current purchase call. Successful return storage is re-read before the congratulations result.
- Runtime evidence: `output/playwright/registration-localstorage-runtime.png` starts from a pre-existing valid source-encrypted localStorage key and takes the source `already registered on this computer` branch in a rendered WASM game.
- Remaining boundary: browser MAC access is intentionally replaced by the existing stable 16-byte device-id boundary; a future user-configurable platform identity may replace that value without changing storage or packet ownership.

## Completed slice: source selectable-map server catalog

- Source: `source/zserver.cpp::{InitSelectableMapList,ReadSelectableMapList,ReadSelectableMapListFromFolder,GivePlayerSelectableMapList,ProcessVote}`, `source/zclient.cpp::ProcessSelectableMapList` and the configured/folder/map-list fallback order.
- Rust owners: `src/selectable_maps.rs::SelectableMapCatalog` owns server-side names and bytes; `SelectableMapListState` remains the client packet result only. `src/main.rs::process_world_vote_actions` resolves the passed Change Map index against both owners before entering the common reset transaction.
- Done: `REQUEST_SELECTABLE_MAP_LIST` no longer manufactures the client list directly from a build constant. It serializes the active server catalog through `GIVE_SELECTABLE_MAP_LIST`, preserving file order, blank-line removal, carriage-return cleanup and exact path/name strings.
- Native: `ZOD_SELECTABLE_MAP_LIST` reads an explicit source-style list and loads each named map path from the current working directory semantics. `ZOD_SELECTABLE_MAP_FOLDER` supplies the `ReadSelectableMapListFromFolder` branch: regular `.map` files only, case-insensitive extension, bare filenames and bytewise-sorted order. An unreadable listed map remains visible to the client but cannot enter the parsed reset transaction.
- WASM/default: `build.rs` remains the platform packager for every repository map; `SelectableMapCatalog::embedded` exposes those static bytes through the same server packet and Change Map owner without filesystem calls.
- Runtime evidence: `output/native-selectable-map-catalog.png` is the native game with the folder catalog active; `output/playwright/selectable-map-catalog-wasm.png` is the rendered embedded-catalog WASM branch.
- Remaining boundary: parsing `ZPSettings::selectable_map_list` from the complete perpetual-server settings file and automatic fallback to a separate rotating `map_list` belong to the real server-process/configuration slice.

## Completed slice: typed perpetual-server `ZPSettings`

- Source: `source/zpsettings.{h,cpp}::{LoadDefaults,LoadSettings}`, `source/zserver.cpp::{InitPerpetualServerSettings,InitSelectableMapList,LoadNextMap}`, account/login handlers and `StartVote` policy reads.
- Rust owner: `src/perpetual_settings.rs::PerpetualServerSettings` is constructed once before the Bevy app and inserted as the shared server resource. Native loads the optional source-equivalent file selected by `ZOD_PSETTINGS`; WASM uses source defaults plus supported build/runtime overrides.
- Done: defaults match source for `ignore_activation`, `require_login`, `use_database`, `use_mysql`, `start_map_paused`, `bots_start_ignored`, `allow_game_speed_change` and empty `selectable_map_list`. Parsing removes CR/LF, ignores only lines whose first character is `#`, preserves exact string values, applies C-style `atoi` truth and treats any nonempty non-comment line as a loaded file.
- Done: the resource now constructs `LocalAccountStore`, `LocalVoteSettings`, `LocalBotTeams`, `SelectableMapCatalog` and initial `GamePauseState`; the former account-local environment reads and independent vote defaults are removed. Existing environment controls are compatibility overrides applied in this one owner rather than scattered consumers.
- Done: map Reset reads `start_map_paused` from the live resource after teardown. `start_map_paused=0` no longer recreates the source resume banner; `allow_game_speed_change`, login-required/database/MySQL policy and configured selectable-list path reach their existing packet owners.
- Done: bot state now distinguishes a started bot thread from its ignored state like `StartBot`/`SetTeamsBotsIgnored`; Start Bot unhides a started team and Stop Bot ignores it without pretending the source thread was destroyed.
- Runtime evidence: `output/native-perpetual-settings.png` is a rendered native world started from a settings file with `start_map_paused=0`, showing the active world without the resume overlay.
- Remaining boundary: MySQL credential fields wait for real database transport; `ignore_activation` is parsed and retained, but its `UnitRequiresActivation` waypoint rejection joins the separate registration/activation command-gate slice.

## Completed slice: registration and account-activation waypoint gates

- Source: `source/zcore.cpp::UnitRequiresActivation`, `source/zplayer.cpp::{SendDevWayPointsOfObj,SendDevWayPointsOfSelected}`, `source/zserver.cpp::ActivationCheckPassed` and `source/zserver_events.cpp::set_waypoints_event`.
- Unit ownership: every robot and vehicle logic file now declares its own `REQUIRES_ACTIVATION` parameter. Pyro/Laser and Medium/Heavy/APC/Missile Launcher/Crane are restricted; Grunt/Psycho/Sniper/Tough and Jeep/Light are unrestricted. `units::{robots,vehicles}::requires_activation` only dispatches to those per-unit owners.
- Done: `MouseCommandInput` reads persisted `RegistrationState`, `PerpetualServerSettings` and roster login/activation state at command-send time. The client check runs first and emits `move unit error: registration required, please visit www.nighsoft.com`; only a client-accepted packet reaches the server-equivalent activation check and its `activation required` message.
- Done: server activation matches source: pass when login is not required, activation is ignored, the client is bot-logged-in, or it is both logged-in and activated. `LocalPlayerState::bot_logged_in` reads the roster packet owner rather than a parallel flag.
- Done: immediate orders filter restricted refs before route/special-target mutation; Shift dev-waypoint chains are checked once per selected object at Shift release, then remove that ref from move/attack/pickup/repair/enter variants. A mixed selection still commands unrestricted units exactly like the source per-object send loop.
- Runtime evidence: `output/playwright/activation-gate-wasm-800.png` shows the rendered unregistered WASM client with the new command owner scheduled; source classification/order/mixed-selection behavior is covered at the pure packet-policy boundary.
- Remaining boundary: the original client also uses registration state to gate some menu/production presentation outside `SEND_WAYPOINTS`; those UI affordances remain part of the menu/production parity audit.

## Completed slice: perpetual-server end-game lifecycle and rotating map list

- Source: `source/zserver.cpp::{CheckEndGame,EndGameRequirementsMet,ProcessEndGame,RelayTeamsWonEnded,CheckDestroyedFort,CheckResetGame,DoResetGame,LoadNextMap,NextInMapList,ReadMapList}`, `source/zserver.cpp::RelayTeamEnded`, `source/zplayer_events.cpp::team_ended_event`, and events `END_GAME=18`, `RESET_GAME=19`, `TEAM_ENDED=69`.
- Rust owners: `src/network_commands.rs::{EndGamePacket,TeamEndedPacket}` owns the exact wire shapes; `src/selectable_maps.rs::MapRotationState` owns the source random/sequential list cursor and native map bytes; `src/main.rs::{GameLifecycleState,process_game_lifecycle,process_destroyed_fort_eliminations}` owns the server clock/order and feeds the existing exclusive reset transaction.
- Done: the live robot/vehicle/cannon owner set is sampled at most once per source virtual second. Null owners, destroyed objects and non-combat families do not keep a match alive; zero or one remaining team enters the ended state.
- Done: destroyed forts relay a losing `TEAM_ENDED` packet before their team objects are destroyed. Final surviving teams relay winning packets before the exact `The game has ended` news and empty `END_GAME` event; decoded outcomes are retained as the client boundary for the later original HUD end-animation renderer.
- Done: native `ZOD_MAP_LIST` parses the first line with C `atoi`, preserves subsequent nonempty map path order and owns each file's bytes. Startup consumes the first sequential/random entry like `LoadNextMap`; later games increment/wrap or select the supplied random index by source modulo behavior.
- Done: end-game detection uses virtual game time, while the ten-second reset deadline uses real time like source `current_time()`. At expiry the next list entry runs through the existing map transfer, parse, `A new game has started`, `RESET_GAME`, teardown and complete reconstruction transaction. Manual Change Map and Reset also restore `game_on` and clear stale team outcomes.
- Done: native asset root resolution now canonicalizes the working-directory `assets/` folder before Bevy constructs its file reader. A directly launched `target/debug/zod-source-rust` therefore renders the same terrain/HUD as `cargo run` instead of looking beside the executable and producing a gray asset-missing window.
- Runtime evidence: `output/end-game-lifecycle-native-fixed.png` records the directly launched native binary with the complete rendered world, HUD and resume prompt after lifecycle wiring.
- Remaining boundary: `TEAM_ENDED` outcome storage is wired, but the source `ZHud::StartEndAnimations` winner/loser unit parade is part of the dedicated HUD parity slice; MySQL win/loss persistence remains part of real database transport.

## Completed slice: `TEAM_ENDED` HUD portrait parade

- Source: `source/zplayer_events.cpp::team_ended_event`, `source/zhud.cpp::{StartEndAnimations,Process,ResetGame,RenderBackdrop}`, `source/zportrait.cpp::{Process,DoRender,RenderFace,PlayAnimSound,SetupFrames}`, `zhud_end_unit`, copied `ENDW1..3`/`ENDL1..3` sequences and sounds `ROB61..ROB73`.
- Rust owners: `src/portrait.rs` owns the client packet queue, parade state, preloaded portrait layers, exact copied end-frame sequences and HUD renderer. Every robot's `_ui.rs` owns its source face id, shoulder height and portrait asset path; `units::robots` only dispatches to those unit owners.
- Done: decoded `TEAM_ENDED` now carries a source-time snapshot of the affected team's robot/vehicle/cannon list. Robot entries use their own type; vehicle/cannon entries use the current driver type with the source Grunt fallback. Refs are sorted by source creation order and consumed from the back like the C++ vector.
- Done: the client ignores outcomes for other teams, starts the first local portrait immediately, then advances one unit every `1.2` virtual seconds. Reset clears the active parade and queue while preserving the source-static last end-voice choice.
- Done: winner/loser animation variants independently choose among three copied sequences: `LETS_GET_EM` (`0.525s`), `GUN_CAPTURED` (`0.855s`) and `GOING_IN` (`0.675s`). Portrait frames advance on real time like `ZPortrait::Process`; the parade scheduler remains on source virtual time like `ZHud::Process`.
- Done: winner voice choice uses `ROB61..ROB66`, loser choice `ROB67..ROB73`, and the shared last choice cannot repeat consecutively. `PortraitAnimationKind` now covers source wire ids 63-68 and exact duration/sound routing.
- Done: backdrop, head, eyes, mouth and shoulders are composed in the source draw order at base HUD `(556,44)`. Vehicle/cannon drivers use `backdrop_vehicle`; robots use the current planet backdrop. Grunt/Sniper shoulders are 32px, Psycho/Pyro/Laser 36px and Tough 26px as owned by their unit UI files.
- Runtime QA: `ZOD_DEBUG_END_ANIMATION=win|lose` injects one local client outcome without changing server state. `output/end-game-portrait-win-native.png` records the rendered native winner parade in the original portrait aperture.
- Remaining boundary: the same layer renderer currently activates for end animations only. Rendering every ordinary selection/acknowledgement/alert portrait frame is the next portrait parity expansion, while MySQL win/loss persistence remains a server/database concern.

## Completed slice: ordinary `ZPortrait` frame renderer and dual HUD portrait ownership

- Source: `source/zportrait.cpp::{SetupFrames,Process,DoRender,RenderFace,GetBlitInfo,StartRandomAnim}`, `source/zhud.cpp::{SetSelectedObject,GiveSelectedCommand,Process,ProcessA,RenderBackdrop}`, and source animation ids `0..62` plus the winner/loser aliases.
- Rust owners: `src/portrait.rs` owns the common layered renderer, subject resolution, the primary portrait idle clock and A/end overlay priority; `src/components.rs` keeps independent `SelectedPortraitAnimationState` and `PortraitAnimationState` resources; `scripts/generate-portrait-frames.awk` converts the original mutable `SetupFrames` construction into `src/portrait_frames_generated.rs`.
- Done: all source frame lists `0..62` are represented as typed Rust tables with exact `0.015s` duration units, look direction, head/mouth/eye indexes and hand coordinates. End ids `63..68` reuse their original `LETS_GET_EM`, `GUN_CAPTURED` and `GOING_IN` frame lists instead of duplicating approximations.
- Done: the renderer resolves a robot directly and a vehicle/cannon through its live driver type with the Grunt fallback, then composes backdrop, head, eyes, mouth, shoulders and optional hand at the original `(556,44)` aperture. Negative hand coordinates use source-equivalent source/destination clipping against the 86x74 portrait bounds.
- Done: selected and acknowledge animations now run in the primary source `portrait`, while attack warnings, good-hit, capture, territory, grenade and end events run in the independent `aportrait`. An active A/end animation temporarily replaces the visible primary portrait without advancing, clearing or overwriting its animation state; end-parade gaps retain the current A-portrait still frame like `do_end_animations`.
- Done: only the primary portrait starts one of the 13 source random idle animations. Completion schedules the original wall-clock `0.5 + rand()%50 * 0.1` delay, idles remain silent, selection changes rebind the idle subject, and non-robot/vehicle/cannon selections clear the portrait like `SetObject -> ClearRobotID`. The A-portrait keeps source `SetDoRandomAnims(false)` behavior.
- Corrected while importing the authoritative tables: `AcknowledgeNoWay` uses ids `48..50`, `GoodHit` uses `51..57`, and their typed total durations now match the generated source lists rather than the earlier hand-copied offsets.
- Runtime QA: `ZOD_DEBUG_PORTRAIT=1` selects a local combat unit and starts the source selected animation; `ZOD_SCREENSHOT_FRAMES` controls capture settling. `output/ordinary-portrait-split-native-settled.png` records a fully rendered native world while the independent selected portrait is inside an ordinary animated face frame.
- Verification: native and `wasm32-unknown-unknown` dead-code-deny passes are clean, the full suite has 772 passing tests, release WASM packaging succeeds, and the direct native binary renders a non-black game window with the animated portrait.
- Remaining boundary: full login/create-user and map-select GUIs, dynamic team/player-list presentation, real socket/server transport, MySQL win/loss persistence and real bot AI remain separate engine slices; ordinary/end portrait frame composition is no longer one of the renderer gaps.

## Completed slice: source login and create-user windows

- Source: `source/gwlogin.cpp`, `source/gwcreateuser.cpp`, `source/zgui_window.cpp::{ZGuiTextBox,ZGuiButton}`, `source/zplayer.cpp::{GuiAbsorbLClick,GuiAbsorbLUnClick,GuiAbsorbKeys}`, and the existing account events `62..65`.
- Rust owner: `src/account_ui.rs` owns the Bevy window state, original image assets, text fields, pointer/key handling and visual synchronization; `src/account.rs::process_account_command` remains the single packet/account lifecycle owner shared by chat and GUI submissions.
- Done: the 112x100 login and 112x157 create-user source bitmaps are centered inside the live game viewport excluding the 100px side HUD and 36px bottom HUD. Login/Create and Ok/Cancel use the original 38x14 normal/pressed images and exact offsets `(8,83)/(66,83)` and `(8,140)/(66,140)`.
- Done: login name/password fields and user/login/password/email fields keep the source offsets, 99x11 hit boxes, initial focus, Tab cycles and Enter actions. Password rendering uses asterisks; selected fields append the source `{` cursor; visible text keeps the right-hand tail when wider than the box.
- Done: input accepts the same ASCII alphanumeric/space/`@._-` set, enforces source 30-character player/login/password and 250-character email limits, supports Backspace, click focus, press-inside/release-inside button semantics, Login->Create and Cancel->Login switching.
- Done: GUI Login and Create User run through the already typed `SEND_LOGIN`/`CREATE_USER` packets and existing server validation/news/roster relays. `GIVE_LOGINOFF` controls visibility; the menu captures keyboard input for the frame even when successful login closes it, and pointer hits inside the active menu cannot leak into selection or the pause-resume banner.
- Done: menu entities are engine-owned and survive exclusive map teardown/rebuild, matching the source active-menu lifetime rather than being recreated as map objects. `ZOD_DEBUG_LOGIN=1|create` exposes the login or create-user branch without changing server settings.
- Runtime evidence: `output/login-menu-native.png` and `output/create-user-menu-native.png` record both original windows over a rendered native game; the latter shows exact centered geometry and the selected `{` cursor.
- Verification: native/WASM dead-code-deny passes are clean, the full suite has 775 passing tests and release WASM packaging includes the menu assets.
- Remaining boundary: the real remote account/database transport and MySQL persistence remain separate; the local typed server boundary is unchanged.
