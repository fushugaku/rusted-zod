# Стратегия переноса движка без потерь

## Принцип

Переносить не "по ощущению", а по trace:

- C/C++ source function/class/constant -> Rust owner module -> tests -> runtime wiring.
- Каждый срез должен иметь ссылку на оригинальные файлы в плане или commit notes.
- Если поведение пока не подключено, оно считается incomplete, даже если тесты на чистую функцию проходят.

## Единица работы

Один переносимый срез должен быть маленьким:

- Один C++ class method или связанная группа методов.
- Один visual state machine.
- Одна gameplay policy.
- Один asset naming/loading family.

Для каждого среза фиксировать:

- Source: `source/...`
- Rust owner: `src/units/...` или engine subsystem.
- Runtime call site.
- Tests.
- Known differences.

## Engine split

- `src/units/**`: unit/building/item stats, visual policy, target/enter/driver/combat behavior.
- Concrete units use `src/units/[type]/[unit_name]/[unit_name]_logic.rs`, `[unit_name]_ui.rs`, `[unit_name]_mod.rs`.
- Shared family policy stays in family facades such as `robot_ui.rs`, `robot_state.rs`, `vehicle_ui.rs`, `building_ui.rs`, `cannon_ui.rs`, `item_ui.rs`.
- `src/pathing.rs`: algorithm only. Unit footprint/shape rules должны приходить из `src/units`.
- `src/placement.rs`: input/query/spawn orchestration only. Cannon/building placement policy должна жить в `src/units`.
- `src/production.rs`: queue/state orchestration only. Unit count/storage/production limits должны жить в `src/units` или dedicated domain facade.
- `src/main.rs`: Bevy scheduling and entity wiring only. No hardcoded unit policy.
- `src/original/**`: parsers and compatibility only. Runtime game logic не должен брать настройки из original shims.

## Order

1. Keep `dead_code` clean. It is currently clean as of 2026-06-05, and any new warning must be resolved in the same slice.
2. Lock unit ownership boundaries: placement/pathing/production leaks.
3. Port source-backed state machines by original owner; vehicle lid state is runtime-wired, remaining lid work is visual/network parity.
4. Port remaining engine systems by original source files, not by current Rust convenience.
5. Keep browser/wasm smoke after each batch.

## Next source-backed slices

1. `source/zrobot.cpp` -> robot action state:
   - `ZRobot::Common_Process`
   - done: player-given direct kills and missile target kills now relay `TARGET_DESTROYED_ANIM` through the existing source `DO_PORTRAIT_ANIM` path.
   - done: `SET_ATTACK_OBJECT` packet/event parity now round-trips local attack target assignment and clear paths through source `attack_object_packet` layout.
   - done: `SET_ATTACK_OBJECT` client event now applies the source A-button/APortrait under-attack branch: local target, empty A-ref, `rand()%5`, `WERE_UNDER_ATTACK_ANIM`, and source-style spacebar event.
   - done: `ZHud::ProcessA` now repeats source A-portrait under-attack warnings after `5 + rand()%300*0.01` seconds with `I_SAID_WERE_UNDER_ATTACK_ANIM + rand()%6` and non-repeating `last_a_anim`.
   - done: local selection now runs the source robot `PlaySelectedAnim` override shape: `rand()%2` picks either generic selected portrait anims or robot-specific reporting anims, with the selected portrait kind driving the same sound path.
   - done: `SET_GRENADE_AMOUNT` packet/event parity now round-trips robot grenade pickup and own/leader grenade consumption through source `obj_grenade_amount_packet` layout.
   - done: `PICKUP_GRENADE_ANIM` packet/event parity now starts robot pickup animation and source `GRENADES_COLLECTED_ANIM` client side effects through the source int packet path.
   - done: grenade-box pickup cleanup now follows source `delete_grenade_box_ref_id` through `UpdateObjectHealth`'s destroyed branch by relaying `DESTROY_OBJECT` into the existing destroyed-object lifecycle instead of direct visual despawn.
   - remaining: full server/client attack-flag sync around visual callbacks, target destroyed event batching, and exact server timing.
2. `source/zvehicle.cpp` -> remaining vehicle lid/network parity:
   - done: `SET_LID_OPEN` packet/event parity round-trips vehicle lid open/close changes through source `set_lid_state_packet` layout before lid/driver visual sync.
   - remaining: audit whether lid visual overlay should become fully server-authoritative once real socket transport exists.
3. `source/zobject.cpp` -> shared object state:
   - full snipe/sniped checks;
   - driver/eject hooks.
   - done: `EJECT_VEHICLE` packet/event parity now round-trips player eject commands through source `eject_vehicle_packet` layout before applying server-style team/CanEjectDrivers gates and existing driver ejection side effects.
   - done: eject-driver neutralization now follows source `ResetObjectTeam(obj, NULL_TEAM)` by relaying/applying `SET_OBJECT_TEAM` with empty driver info after drivers are ejected.
   - done: eject-driver attack cleanup now follows source `SetAttackObject(NULL)` / `RelayObjectAttackObject` by relaying/applying `SET_ATTACK_OBJECT` clear packets during eject cleanup.
   - done: eject-driver waypoint cleanup now follows source `GetWayPointList().clear()` / `RelayObjectWayPoints` by relaying/applying an empty `SEND_WAYPOINTS` packet before removing movement and special waypoint runtime state.
   - done: eject-driver movement stop cleanup now follows source `StopMove()` / `RelayObjectLoc` by relaying/applying `SEND_LOC` with zero velocity for objects that had active movement during eject cleanup.
   - done: live-driver snipe hits now relay source `DRIVER_HIT_EFFECT` and apply the one-frame driver-only visual flag to the vehicle driver overlay.
   - done: dead-driver snipe hits now relay source `SNIPE_OBJECT` and spawn the client `ERobotTurrent`-style visual before local driverless neutralization.
   - done: dead-driver `NULL_TEAM` reset now relays/applies source `SET_OBJECT_TEAM` with packed `driver_info_s` layout after `SNIPE_OBJECT`.
   - done: dead-driver cleanup now follows the complete `UpdateObjectDriverHealth` cascade: the target disengages through `SET_ATTACK_OBJECT`, clears its own waypoint list through `SEND_WAYPOINTS`, and every other object attacking it disengages through its own packet.
   - done: dead-driver target removal now erases only the first matching runtime `ATTACK_WP` (source `AGRO_WP` is normalized to `ATTACK_WP` on acceptance), relays the preserved route tail through `SEND_WAYPOINTS`, and applies it to every visual layer with its layer offset intact.
   - done: source `IsMoving()` / `StopMove()` is represented by runtime `MovementVelocity` with the original `0.00001` epsilon; dead-driver and eject cleanup relay `SEND_LOC` only for genuinely moving objects, then zero velocity without collapsing visual layers.
   - done: robot enter/capture now relays/applies source `SET_OBJECT_TEAM` for vehicle/cannon owner and driver state before updating layers/minimap.
   - done: robot enter/capture now relays/applies source `DO_PORTRAIT_ANIM` for `VEHICLE_CAPTURED_ANIM` / `GUN_CAPTURED_ANIM`, with local-team and busy-portrait guards plus source-style spacebar event.
   - done: accepted portrait events now play source `ZPortrait::PlayAnimSound` audio for target destroyed, territory taken, gun captured, and vehicle captured animations.
   - done: `PortraitAnimationState` now clears busy state after source `ZPortrait_Anim::total_duration` for all currently wired A-portrait event ids, instead of staying busy forever after `StartAnim`.
   - done: local object selection now starts source selected portrait animations for base `ZObject::PlaySelectedAnim` users with live drivers and for robot overrides, replacing the Rust-only voice shortcut.
   - done: local right-click object commands now start source `PlayAcknowledgeAnim` portrait feedback, including the `DevWayPointsNoWay`/`ZUnitRating::UCR_WILL_DIE` no-way branch for single-object attack commands.
   - done: live destroyed-object lifecycle now round-trips `DESTROY_OBJECT` packet flags before deciding source `destroy_object` despawn vs persistent destroyed marker and fire/missile death visual flags.
   - done: grenade-box death missiles now travel through `DESTROY_OBJECT` `fire_missile_info` tail before Bevy spawns grenade explosion missiles.
   - done: map-object turrent death missiles now travel through `DESTROY_OBJECT` `fire_missile_info` tail before Bevy spawns `EMapObjectTurrent`-style missiles.
   - done: Light/Medium/Heavy vehicle turrent death missiles now travel through `DESTROY_OBJECT` `fire_missile_info` tail before Bevy spawns `ETurrentMissile`-style top-pop missiles.
   - done: cannon turrent death missiles now travel through `DESTROY_OBJECT` `fire_missile_info` tail before Bevy spawns `ECannonDeath`-style wasted cannon missiles.
   - done: vehicle/cannon `DESTROY_OBJECT` turrent tails now also create source server-style NULL-team logical damage missiles instead of remaining visual-only effects.
   - done: accepted `DESTROY_OBJECT` for the local player's fort now focuses the game camera on the destroyed fort like source `destroy_object_event`.
   - done: accepted `DESTROY_OBJECT` for the last enemy FortFront/FortBack now focuses the game camera when the local team still has available units and no other enemy team has available units, matching the adjacent source `destroy_object_event` branch.
   - done: accepted `DESTROY_OBJECT` now preserves `killer_ref_id` from recent local damage and can start the source good-hit A-portrait branch when the killer belongs to the local team and is within 100px of the current A-ref.
4. `source/zbuilding.cpp`, `source/zbuildlist.cpp`, `source/brobot.cpp`, `source/bvehicle.cpp` -> production/building state:
   - done: queue front-insert order and one-completion-per-server-pass timing;
   - done: removed Rust-only `BuildingProduction::ready_units`; completed units are per-tick events, not persistent building state;
   - done: moved queue/storage/completion policy out of generic `src/production.rs` into `src/units/buildings/production_logic.rs`; `src/production.rs` remains a compatibility facade.
   - done: factory overlay frame timers use source bounded tick with long-frame overshoot discard.
   - done: cannon storage/drop capacity is building-owned and counts placed plus stored cannons in-zone before storing.
   - done: source `BuildingCreateUnit` cannon/non-cannon completion outcome is owned by `src/units/buildings/production_logic.rs`, with `main.rs` reduced to Bevy spawning from the outcome.
   - done: building rally points are stored on production buildings and appended to produced leader routes after the factory-exit move point; robot minions keep only the initial cloned factory-exit route.
   - done: production spawn routes now preserve typed waypoint metadata (`MOVE_WP`, `FORCE_MOVE_WP`, `ref_id`, `attack_to`, `player_given`) alongside the existing `Vec2` path compatibility layer.
   - done: `FORCE_MOVE_WP` now drives the runtime movement `stoppable=false` gate for direct production-exit movement, while ordinary `MOVE_WP` keeps the stoppable/pathing gate.
   - done: movement `attack_to` waypoints now run a source-backed `CheckAttackTo` candidate hook and interrupt movement into `AttackTarget`.
   - done: movement `CheckAttackTo` now inserts a real typed `ATTACK_WP` at the front of every object layer path, keeps the previous `MOVE_WP` route behind it, and pops only that front attack waypoint when the target disappears or becomes invalid.
   - done: player right-click attack commands now create typed `ATTACK_WP` routes with source `player_given=true` / `attack_to=true` metadata instead of assigning `AttackTarget` directly.
   - done: ordinary player right-click move commands now create source-style final `MOVE_WP` metadata with `player_given=true` and `attack_to` set by the original Ctrl/Alt/near-hostile rule, so `CheckAttackTo` can interrupt local move commands.
   - done: source `AsciiDown('z')` one-unit-per-target command send now limits local right-click command fanout to the nearest selected object and removes that object from selection after the command.
   - done: source `ShiftDown` terrain move waypoint accumulation now stores local pending `MOVE_WP` commands while Shift is held and sends the chained route when the last Shift key is released.
   - done: source `ShiftDown` attack waypoint accumulation now stores local pending `ATTACK_WP` commands while Shift is held and appends them to the chained route on Shift release.
   - done: source `ShiftDown` grenade pickup waypoint accumulation now stores local pending `PICKUP_GRENADES_WP` commands while Shift is held and appends pickup movement/target state to the chained route on Shift release.
   - done: source `ShiftDown` repair waypoint accumulation now stores local pending `UNIT_REPAIR_WP` and `CRANE_REPAIR_WP` commands while Shift is held and appends repair movement/target state to the chained route on Shift release.
   - done: source `ShiftDown` enter waypoint accumulation now stores local pending `ENTER_WP` and `ENTER_FORT_WP` commands while Shift is held and appends enter movement/target state to the chained route on Shift release.
   - done: minions now follow the original `ProcessMove` terrain/collision rule by treating movement as non-stoppable even when the current waypoint is `MOVE_WP` or `ATTACK_WP`.
   - done: source `SEND_WAYPOINTS` payload layout is represented with packed 15-byte waypoints, and current `MOVE_WP` / `ATTACK_WP` / `FORCE_MOVE_WP` `MovementPath` assignment now round-trips through that packet layer locally.
   - done: `PICKUP_GRENADES_WP` is now encoded/decoded as a special `SEND_WAYPOINTS` waypoint for immediate and shifted pickup commands before bridging into the current pickup component.
   - done: `ENTER_WP` and `ENTER_FORT_WP` are now encoded/decoded as special `SEND_WAYPOINTS` waypoints for immediate and shifted enter commands before bridging into current enter components.
   - done: `UNIT_REPAIR_WP` and `CRANE_REPAIR_WP` are now encoded/decoded as special `SEND_WAYPOINTS` waypoints for immediate and shifted repair commands before bridging into current repair components.
   - done: local `SEND_WAYPOINTS` relay now runs source `ProcessWaypointData` server gates and `CheckWaypoint` validation for immediate and shifted waypoint commands before inserting runtime movement/special target state.
   - done: `SEND_RALLYPOINTS` packet layout and local `ProcessRallypointData` relay/validation are represented for production buildings that source `CanSetRallypoints`.
   - done: source `Pcursor` / final waypoint cursor feedback now appears for accepted immediate commands, Shift-released pending command chains, and accepted rally commands with source cursor mapping and 3 second lifetime.
   - done: source `DoRenderWaypoints` dotted waypoint/rally line feedback now renders for accepted immediate commands, Shift-released pending command chains, and the open production building's rally points.
   - done: source `ProcessWaypointData` now preserves the current first `FORCE_MOVE_WP` when local `CanOverwriteWP` would reject overwriting it, and accepted waypoint commands clear `JustLeftCannon` only after the local packet relay accepts the command.
   - done: source `CloneMinionWayPoints` is represented after accepted immediate and Shift `SEND_WAYPOINTS` relays by cloning the accepted leader path and bridged special target state to live robot minions with minion layer offsets and minion movement speed.
   - done: source `ShowWaypoints` / `DoRenderWaypoints` transient feedback now keeps independent 3 second lifetimes per object ref, so rapid accepted waypoint packets for different objects overlap instead of replacing a single global path list.
   - done: source `CanOverwriteWP` current-stage gates are represented for `CRANE_REPAIR_WP`, `UNIT_REPAIR_WP`, and `ENTER_FORT_WP` by passing current special target stages into local `SEND_WAYPOINTS` relay.
   - done: source `ProcessWaypointData` queue-behind behavior is represented for movement/attack-compatible tails behind active non-overwritable special stages by appending to the current `MovementPath` without clearing the active special target component.
   - done: source `DoAttackImpassableAtCoords` is represented when a stoppable movement footprint hits a destroyable impassable rock/hut/map-object tile: Rust finds the matching source stop tile, requires explosives, inserts an `ATTACK_WP` at the front of the movement path, and clones it to live robot minions.
   - done: source `ProcessPickupWP` arrival/side-effect ownership is represented for `PICKUP_GRENADES_WP`: invalid capability/target kills the waypoint, minions only kill the cloned waypoint on arrival, and only a non-minion applies grenade amount, pickup animation, portrait feedback, and grenade-box cleanup packets.
   - done: source `ProcessEnterWP` terminal owner behavior is represented for ordinary vehicle/cannon enter waypoints: target invalidation kills the waypoint, movement keeps the waypoint alive until `UnderCursor`, cloned minions only kill their own waypoint on arrival, and only non-minions emit capture side effects.
   - done: source `ProcessEnterFortWP` stage owner behavior is represented for `ENTER_FORT_WP`: missing targets kill the waypoint, `CanEnterFort` failure kills during `GOTO_ENTRANCE`, failure during `ENTER_BUILDING` forces the exit stage, valid `ENTER_BUILDING` destroys the fort and exits, and `EXIT_BUILDING` kills the waypoint after movement finishes.
   - done: source `ProcessUnitRepairWP` stage owner behavior is represented for unit repair waypoints: `GOTO_ENTRANCE` completes into `WAIT`, busy repair buildings keep `WAIT`, invalid/busy `ENTER_BUILDING` exits back to the entrance, successful `ENTER_BUILDING` transfers the source snapshot into building-owned occupancy, and `EXIT_BUILDING` returns to `WAIT`.
   - done: source `ProcessCraneRepairWP` stage owner behavior is represented for crane repair waypoints: missing targets kill the waypoint, invalid targets kill during `GOTO_ENTRANCE`, invalid `ENTER_BUILDING` forces `EXIT_BUILDING`, `EXIT_BUILDING` finishes auto-repair after movement completion, and target validity is checked even while the current movement is still active.
   - done: source `DO_CRANE_ANIM` packet parity now round-trips crane repair visual on/off through `crane_anim_packet` before driving the crane conco visual target.
   - done: source `SET_REPAIR_ANIM` packet parity now round-trips repair-building visual on/off through `repair_building_anim_packet`, including source start-frame reset, local-team sound side effects, and repaired-object spacebar event.
   - done: accepted plain `SEND_WAYPOINTS` commands whose server-filtered waypoint list is empty now follow `rcv_object_waypoints_event`: the leader and every live minion clear movement/special waypoint state while preserving `attack_object`, run source-shaped zero-velocity `SEND_LOC` relay/apply unconditionally, retain per-layer visual offsets, and discard a marker superseded by a later special command in the same deferred batch.
   - done: stored-cannon placement now emits `ADD_NEW_OBJECT -> SET_BUILT_CANNON_AMOUNT` through the shared FIFO after source owner/zone/placement gates; the list commit depends on successful object creation, failed clicks leave placement mode, and the source seven-frame `JUST_PLACED_MODE` with cannon-specific render offsets finishes on the occupied passive frame.
   - done: generic client `SEND_LOC` apply now follows `ZClient::ProcessObjectLoc`, `ZObject::SetLoc`, `SmoothMove`, and the integer `ProcessMove` accumulator: source top-left map coordinates are canonical, visual anchors/layer offsets remain separate, direction changes only with packet velocity, interpolation uses source `floor(velocity * elapsed)`, and local motion preserves `xover/yover` remainders.
   - done: runtime `ADD_NEW_OBJECT` now has a shared source-event FIFO and client apply owner; packet x/y remain canonical top-left coordinates, gameplay roots use source object dimensions, visual layers keep per-unit render offsets, packet health remains ignored like `ProcessNewObject`, duplicate refs are rejected, and source ref/owner/level/extra-links initialize the new object.
   - done: `EJECT_VEHICLE` driver creation now uses the shared source FIFO as one guarded same-frame batch: every leader/minion relays `ADD_NEW_OBJECT -> OBJECT_GROUP_INFO -> UPDATE_HEALTH`, gameplay entity ids are pre-reserved so forward group refs resolve without reordering the source events, actual driver health and canonical source top-left coordinates reach those roots, and carrier cleanup/team reset commits only after every member succeeds. The corrected client-intended group layout and per-member health/`JustLeftCannon` behavior are explicit compatibility policy; the source index/target/increment defects are not copied.
   - done: `BuildingCreateUnit` robot/vehicle completion now uses the same source FIFO: robot members relay `ADD_NEW_OBJECT -> OBJECT_GROUP_INFO` in member order, vehicles relay `ADD_NEW_OBJECT`, and only the leader receives the source `SEND_WAYPOINTS` initial `FORCE_MOVE_WP`; successful client construction gates manufactured feedback and the local authoritative factory-exit/rally routes.
   - done: `BuildingRepairUnit` now replaces the entering object through source-ordered ownership: the old ref and its group/grenade state are removed at the next early server drain, the repair building owns the saved unit/driver/waypoint snapshot for the absolute five-second deadline, and a destroyed repair building defers completion until revival while preserving source driver attack timestamps unchanged.
   - done: repaired robot/vehicle completion allocates fresh refs only at the deadline and relays `ADD_NEW_OBJECT`, robot `OBJECT_GROUP_INFO`, owner-only leader `SEND_WAYPOINTS`, and leader `SET_OBJECT_TEAM` before the repaired batch commit; saved driver state, default source drivers, exit `FORCE_MOVE_WP`, queued route metadata, repair animation stop, repaired sound, and spacebar feedback follow source order.
   - done: repair and factory completions share a per-pass ref reservation sorted by source building/object ref, protected `UNIT_REPAIR_WP` stages keep newly sent move/attack waypoints as the post-repair tail, and robot groups retain explicit packet-derived member order for leader promotion/grenade transfer.
   - done: source object refs now begin at `0` for startup and dynamic construction, matching `ZServer::next_ref_id = 0`; the former repair-exit `ref_id == 0` sentinel was replaced with explicit state so object zero remains a valid runtime target.
   - done: source-owned dynamic vehicle roots and separate visual layers now join lid/driver/turrent visual state by `ObjectLayerRef`, so produced Light/Medium/Heavy vehicles receive the same lid visual processing as startup vehicles.
   - done: source `ZTime` is represented by Bevy `Time<Virtual>`: the first-stage owner applies server pause and non-negative game speed every frame, gameplay timers consume the frozen/scaled clock, and vote expiry, HUD news, cursor/input timing and camera control explicitly consume `Time<Real>`.
   - remaining: full `cur_wp_info`, async pathfinder response, remaining `ProcessWaypointData` apply semantics (relay-to-team delivery and queued special-after-special waypoints), source `ProcessPickupWP` / `ProcessEnterWP` / `ProcessEnterFortWP` / `ProcessUnitRepairWP` / `ProcessCraneRepairWP` pathfinder/attack-to timing under the future async model, bot/network `ATTACK_WP` relay parity, and unified socket-event ordering for cross-packet queues are still missing.
   - done: source `RelayObjectManufacturedSound` / successful `BuildingCreateCannon` sound-event parity is runtime-wired for local produced robot/vehicle/stored cannon completions.
   - done: source `START_BUILDING`/`STOP_BUILDING` computer feedback sounds are wired for local production window OK/full-selector start and `BUILDING_BUILDING` cancel; queue add/cancel remain silent like source.
   - done: production-window start/cancel computer feedback sounds now run through source `COMP_MSG` packet layout before local sound-only apply.
   - done: manufactured `COMP_MSG` UI parity is runtime-wired locally: robot/vehicle/gun manufactured banners, click behavior, and source-style spacebar events use produced object/building `ref_id` rules.
   - done: manufactured robot/vehicle/gun feedback now runs through source `COMP_MSG` packet layout before local client apply.
   - done: stored-gun left HUD stack renders source `gun.png` indicators for owned production buildings with stored cannons and opens the source building GUI on click.
   - done: AwardZone now plays source sound-only `COMP_TERRITORY_LOST`/`COMP_RADAR_ACTIVATED` feedback locally for old/new Red team rules; linked radar detection follows `OFlag::HasRadar` without owner/destroyed checks.
   - done: AwardZone flag/building owner changes now round-trip through source `SET_OBJECT_TEAM`, zone owner changes round-trip through runtime `SET_ZONE_INFO`, and conquerer portrait events relay `TERRITORY_TAKEN_ANIM`.
   - done: AwardZone territory-lost/radar-activated feedback now runs through source `COMP_MSG` sound-only packet layout before local client apply.
   - done: pause/resume computer banner renders centered source `click_to_resume.png` while the local pause state is active and resumes through source-style press/release hit testing; `paused.png` stays unused like the source load path.
   - done: pause packet ids and `update_game_paused_packet` bool payload layout are represented in `src/network_commands.rs` for `UPDATE_GAME_PAUSED`, `GET_GAME_PAUSED`, and `SET_GAME_PAUSED`.
   - done: local pause requests now flow through source-style `SET_GAME_PAUSED`/`GET_GAME_PAUSED`/`UPDATE_GAME_PAUSED` packet layout and same-state guard before `GamePauseState` changes.
   - done: pause requests now run through a source-style local `StartVote`/`VoteYes`/`CheckVote`/`ProcessVote` owner before emitting `UPDATE_GAME_PAUSED`; single-player default still resolves immediately like source `VoteRequired()==false`.
   - done: `VOTE_INFO` now uses the source packed `bool+i32+i32` packet layout and round-trips vote state through local relay/apply code when local vote state changes.
   - done: `SET_LPLAYER_VOTEINFO` now uses the source packed `p_id/value` packet layout and vote-choice changes round-trip through local relay/apply code keyed by player id.
   - done: active votes now track source `MAX_VOTE_TIME=30` and expire through `KillVote` plus `VOTE_INFO` relay before simulation work.
   - done: vote panel/count rendering now uses source `vote_in_progress.png`, alpha 200, top-right 4px placement, source offsets for description/have/needed/for/against, and pause/resume labels.
   - done: vote input uses source keyboard-only `F1`/`F2`/`F3` mapping, empty `VOTE_YES`/`VOTE_NO`/`VOTE_PASS` packets, and local server-style `VoteYes`/`VoteNo`/`VotePass` handlers.
   - done: local vote/news strings now flow through a source-style `NewsLog`/HUD stack for vote success, duplicate/login rejection, and vote expiration broadcast.
   - done: `NEWS_EVENT=10` is represented with source `r,g,b + nul-terminated message` payload and local news additions round-trip through that packet before display.
   - done: pause/resume vote creation emits source-style vote-start broadcast strings through the local `NEWS_EVENT` path.
   - done: `SEND_CHAT=27` is represented with source nul-terminated message payload and local chat submission round-trips through the packet before normal chat broadcast.
   - done: local chat input now mirrors source `Enter` collect/submit, Backspace edit, `/` quick command draft, and `H` chat history toggle.
   - done: HUD chat draft renders source `Say::` text at the original bottom-left chat position.
   - done: local `/pause` and `/resume` chat commands feed the existing pause vote/request path; unknown slash commands emit the source command-not-found news string.
   - done: `ProcessPlayerCommand` news-only `/help`, `/help <command>`, and `/listcommands` output uses exact source strings and source no-op behavior for unknown help topics.
   - done: `ProcessPlayerCommand` `/playerinfo` and `/currentmap` output is locally wired from `ChatCommandContext` and parsed `CurrentMap` data.
   - done: `ProcessPlayerCommand` `/version` now relays source `GIVE_VERSION` fixed-char packet locally and emits source client version news.
   - done: `ProcessPlayerCommand` `/changespeed` now feeds the source `SET_GAME_SPEED` / `CHANGE_GAME_SPEED` vote path, applies successful speed changes through `UPDATE_GAME_SPEED`, and emits the source speed-change news.
   - done: startup local `REQUEST_VERSION` handshake now runs before pause query and shares the `/version` client apply path.
   - done: startup local `GET_GAME_SPEED` handshake now runs after pause query and applies source `UPDATE_GAME_SPEED` float packet into local game-speed state.
   - done: startup local `SendPlayerInfo` now runs through source-style `SET_NAME`, `SET_TEAM`, `SET_PLAYER_MODE`, and `SET_LPLAYER_NAME`/`TEAM`/`MODE` packet relay into a shared local-player state used by chat/playerinfo.
   - done: startup local `RequestPlayerList` now runs through source-style `REQUEST_PLAYER_ID`, `GIVE_PLAYER_ID`, `REQUEST_PLAYER_LIST`, `CLEAR_PLAYER_LIST`, `ADD_LPLAYER`, and existing `SET_LPLAYER_NAME`/`TEAM`/`MODE` relay into a local roster.
   - done: player roster delete/ignored/loginfo packets now use source `DELETE_LPLAYER`, `SET_LPLAYER_IGNORED`, and packed `SET_LPLAYER_LOGINFO`; `/playerinfo` reads roster loginfo for logged-in and voting-power data.
   - done: `SET_LPLAYER_VOTEINFO` now updates the `LocalPlayerState` roster `vote_choice` from source vote/clear packet sequences, including `ClearPlayerVotes` null-vote relays after vote resolution or expiration.
   - done: startup local `REQUEST_SELECTABLE_MAP_LIST` now runs through source-style request/response packet relay, stores generated `maps/*.map` state in `src/selectable_maps.rs`, and `/listmaps` emits source-style four-map grouped news lines.
   - done: startup map bytes now run through a source-style local `REQUEST_MAP` / chunked `STORE_MAP` transfer before `ZMap::parse`, including source `pack_num=-1` completion.
   - done: startup local `REQUEST_SETTINGS` now runs before `SendPlayerInfo`, receives a raw source-sized `SET_SETTINGS` payload, and stores it in `SourceSettingsState`.
   - done: post-map local `REQUEST_ZONES` now sends source-style `SET_ZONE_INFO` packets and applies ownership through `src/zone_sync.rs` before HUD/minimap/resource setup.
   - done: post-map local `REQUEST_OBJECTS` now sends source-style `ADD_NEW_OBJECT` object-init packets and gates the current Bevy spawn ref-id sequence against the decoded object-init stream.
   - done: `OBJECT_GROUP_INFO` now relays client-expected robot leader/minion metadata for the local object-init stream and gates robot packet streams before Bevy spawn.
   - done: object-list `UPDATE_HEALTH` now relays source actual-health packets for the local object-init stream and gates one health packet per object before Bevy spawn.
   - done: dynamic `UPDATE_HEALTH` packets now apply source-clamped health into live `ObjectStats` through a runtime queue before destroyed-object lifecycle processing.
   - done: `UPDATE_HEALTH` revives now remove `DestroyedObject`/`AutoRepair`, restore live layers, and reopen bridge passability through the existing bridge revive path.
   - done: accepted `UPDATE_HEALTH` packets now trigger source-style one-frame `DoHitEffect` visual flag handling on object layers.
   - done: source `ZSDL_Surface::BlitHitSurface` pixel semantics replace the former overbright tint: each loaded sprite/atlas gets a cached white silhouette where alpha zero remains transparent and every nonzero-alpha pixel becomes opaque white; object and vehicle-driver hit flags swap that silhouette for exactly one extracted frame and then restore the current animation image.
   - done: startup `REQUEST_OBJECTS` now constructs every gameplay root through the shared runtime `ADD_NEW_OBJECT` FIFO, followed in source order by group, health, building-state/queue/repair-animation, grenade and rally packets.
   - done: the packed `SET_SETTINGS` payload is decoded into typed source settings and drives startup/dynamic `ObjectStats`, robot group counts/ref allocation, movement stamina/speed, APC driver combat stats, grenade delivery, production counts/timers and zone-link ref ids.
   - done: source `GWFactoryList` is runtime-wired to the B HUD button and keyboard shortcut with source ordering, bottom-left/view-height placement, health/production/tech rows, progress/time text, scrolling and click-to-focus/open-factory behavior; the screen-space anchor follows the source view shift instead of map coordinates.
   - done: the vote client recognizes all eight source wire types/labels; `VoteAppendDescription` map-number/name and start/stop-bot team-color strings are derived from live selectable-map state and rendered in the HUD description exactly as `ZVote::SetupImages` expects.
   - done: `/changemap`, `/startbot`, `/stopbot`, `/resetgame`, and `/reshuffleteams` now use source command parsing/error strings, exact events 79-83 and the shared `StartVote` path; start/stop bot passed actions update the local source team-bot registry used by later command validation.
   - done: passed Change Map and Reset actions now share a source-ordered runtime transaction: map bytes transfer/parse, empty `RESET_GAME` event 19, session-entity teardown, pause, terrain/object/zone/HUD reconstruction and camera recenter. `build.rs` embeds the generated selectable-map byte lookup so the same path works in WASM.
   - done: passed Reshuffle Teams now derives eligible teams from robot/vehicle/cannon owners, excludes active-bot teams, assigns logged-in players first, and relays roster/local team packets plus source result news.
   - done: `/changeteam` now validates all source team names/current-team state, round-trips `SET_TEAM`, relays `SET_LPLAYER_TEAM`, emits the direct/broadcast news pair in source order and clears selection/space events so HUD, cursor and factory list rebind from one local-team owner.
   - done: `/login`, `/logout`, and `/createuser` now preserve source comma parsing, account validation/news, events 62-65, login-required response and roster name/loginfo/vote relays; default no-database behavior matches `ZPSettings`, with an opt-in deterministic local store for the database branch.
   - done: `/buyregistration` now follows source poll/buy/return ids 70-72, 16-byte device packet, database/login/activation/voting-power gates, roster deduction, original AES-128 key and client verification; browser storage remains an explicit platform boundary.
   - done: dead-driver `UpdateObjectDriverHealth` cleanup now matches the complete source sequence for target/dependent attack and first attack/agro waypoint clears; target `SEND_LOC` is emitted only inside the non-empty waypoint-clear branch, not merely because interpolation reports motion.
   - done: `registration.zkey` uses the source exact 16-byte encrypted payload and startup validation on native; WASM stores the same payload as strict hex in localStorage behind a write-probed platform boundary.
   - done: selectable-map server ownership is separate from client packet state; native explicit-list/folder catalogs load runtime map bytes and order, while WASM uses the generated embedded catalog through the same request/change-map path.
   - done: a typed `PerpetualServerSettings` resource owns source `ZPSettings` defaults/file parsing and supplies account, login, vote-speed, initial/reset pause, bot-ignore and selectable-catalog policy instead of duplicated assumptions.
   - done: source `UnitRequiresActivation`, client registration rejection and server login/activation policy now gate immediate and Shift waypoint sends; the restriction bit lives in every affected unit logic file.
   - done: source virtual-time end detection, losing/winning `TEAM_ENDED`, empty `END_GAME`, real-time ten-second reset delay and native sequential/random rotating map-list ownership now feed the common reset transaction.
   - done: local `TEAM_ENDED` packets snapshot source combat/driver identity and run the exact `ZHud::StartEndAnimations` reverse-order 1.2-second portrait parade with copied winner/loser frames, backdrops and nonrepeating voices.
   - done: generated Rust tables now cover every source ordinary portrait frame list `0..62`; the shared layer renderer applies exact head/eye/mouth/hand frames and source clipping for selected, acknowledge, warning, good-hit, capture, territory, grenade and end animations.
   - done: source `portrait` and `aportrait` are independent runtime resources. Selected/acknowledge plus the 13 random idle animations belong to the primary portrait; A/end events use the non-random overlay and temporarily win `RenderBackdrop` priority without destroying primary animation state.
   - done: source `GWLogin` and `GWCreateUser` are Bevy HUD windows with original assets, dimensions, field/button offsets, focus/Tab/Enter/text rules, pressed-state hit testing and Login/Create/Cancel transitions; submissions reuse the typed account packet owner and `GIVE_LOGINOFF` visibility state.
   - remaining: real socket transport for pause/vote/news/chat/version/speed/settings/player-info/selectable-map-list/map/zone/object/group/health/snipe/driver-hit/object-team/portrait/comp-msg/account/end-game packets, dynamic team palette/full player-list UI abstraction, map-select GUI, remaining registration-gated UI affordances, MySQL win/loss updates, and real bot process/AI ownership are not modeled yet.
5. `source/e*.cpp`, cannon and vehicle death code -> effects:
   - direct fire and missile impacts;
   - vehicle/cannon/robot death effects;
   - bridge/building destruction effects.
