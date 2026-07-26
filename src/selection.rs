use bevy::{ecs::system::SystemParam, prelude::*};
use std::collections::{HashMap, HashSet};

use crate::{
    account::{LoginPromptState, RegistrationState},
    account_ui::{AccountMenuState, account_menu_contains_cursor},
    camera::cursor_world_position,
    components::*,
    constants::{HUD_HEIGHT, HUD_WIDTH, TILE_SIZE},
    cursor::{PreviousCursorState, ZCursorKind},
    enter::{
        EnterFortTargetInfo, EnterTargetInfo, can_be_entered, can_enter_fort,
        enter_fort_target_for_right_click, enter_target_for_right_click,
    },
    factory_list::FactoryListState,
    grenades::{can_pickup_grenades, is_grenade_box, pickup_target_for_right_click},
    local_player::LocalPlayerState,
    network_commands::{
        CommandPayload, SendRallypointsPacket, SendWaypointsPacket, SourceWaypoint,
        SourceWaypointMode,
    },
    news::NewsLog,
    object_sync::{
        EjectVehiclePacketQueue, movement_waypoint_from_source, relay_eject_vehicle_command,
        source_waypoint_from_movement, source_waypoints_from_existing_path,
        source_waypoints_from_movement_path,
    },
    original::{
        objects::{BuildingType, ItemType, ObjectKind, VehicleType},
        types::TeamType,
    },
    pathing::RouteFootprint,
    perpetual_settings::PerpetualServerSettings,
    production::apply_building_rally_point,
    repair::{
        CraneRepairTargetInfo, UnitRepairTargetInfo, crane_repair_target_for_right_click,
        unit_repair_target_for_right_click,
    },
    units::{
        self, buildings,
        unit_behavior::{self, PassiveCombatTargetSnapshot},
        unit_stats::MAX_UNIT_HEALTH,
    },
};

pub(crate) fn process_hud_commands(
    mut commands: Commands,
    mut queue: ResMut<HudCommandQueue>,
    mut command_state: ResMut<HudCommandState>,
    mut pause_requests: ResMut<GamePauseRequestQueue>,
    mut production_window: ResMut<ProductionWindowState>,
    mut factory_list: ResMut<FactoryListState>,
    mut selection: ResMut<SelectionState>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    object_query: Query<
        (
            &GameObjectEntity,
            &Transform,
            &Selectable,
            &ObjectTeam,
            &ObjectStats,
            Option<&BuildingProduction>,
            Option<&RobotGroup>,
        ),
        Without<MainCamera>,
    >,
    selection_markers: Query<Entity, With<SelectionMarker>>,
    selection_health_bars: Query<Entity, With<SelectionHealthBar>>,
) {
    let pending = std::mem::take(&mut queue.pending);
    for command in pending {
        match command {
            HudCommand::SelectGroup(group) => select_next_group_object(
                &mut commands,
                group,
                TeamType::Red,
                &mut command_state,
                &mut selection,
                &object_query,
                &selection_markers,
                &selection_health_bars,
            ),
            HudCommand::BeginBuildingAction => factory_list.toggle(),
            HudCommand::JumpToObject(ref_id) => select_and_focus_object(
                &mut commands,
                ref_id,
                &mut camera_query,
                &mut selection,
                &object_query,
                &selection_markers,
                &selection_health_bars,
            ),
            HudCommand::ResumeGame => {
                pause_requests
                    .pending
                    .push(GamePauseRequest { game_paused: false });
            }
            HudCommand::FocusObject {
                ref_id,
                select_obj,
                open_gui,
            } => {
                focus_object_with_options(
                    &mut commands,
                    ref_id,
                    select_obj,
                    open_gui,
                    &mut camera_query,
                    &mut selection,
                    &object_query,
                    &selection_markers,
                    &selection_health_bars,
                    Some(&mut production_window),
                );
            }
        }
    }
}

pub(crate) fn process_space_bar_events(
    time: Res<Time<Real>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut events: ResMut<SpaceBarEventQueue>,
    mut commands: Commands,
    mut production_window: ResMut<ProductionWindowState>,
    mut selection: ResMut<SelectionState>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    object_query: Query<
        (
            &GameObjectEntity,
            &Transform,
            &Selectable,
            &ObjectTeam,
            &ObjectStats,
            Option<&BuildingProduction>,
            Option<&RobotGroup>,
        ),
        Without<MainCamera>,
    >,
    selection_markers: Query<Entity, With<SelectionMarker>>,
    selection_health_bars: Query<Entity, With<SelectionHealthBar>>,
) {
    events.advance(time.delta_secs());

    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    let attempts = events.events.len();
    for _ in 0..attempts {
        let Some(event) = events.events.pop_front() else {
            break;
        };
        if event.expired() {
            continue;
        }
        if focus_object_with_options(
            &mut commands,
            event.ref_id,
            event.select_obj,
            event.open_gui,
            &mut camera_query,
            &mut selection,
            &object_query,
            &selection_markers,
            &selection_health_bars,
            Some(&mut production_window),
        ) {
            events.events.push_back(event);
            break;
        }
    }
}

fn open_production_window_for_ref(
    ref_id: u32,
    team: TeamType,
    object_query: &Query<
        (
            &GameObjectEntity,
            &Transform,
            &Selectable,
            &ObjectTeam,
            &ObjectStats,
            Option<&BuildingProduction>,
            Option<&RobotGroup>,
        ),
        Without<MainCamera>,
    >,
    production_window: &mut ProductionWindowState,
) -> bool {
    let Some((object, _, _, _, _, _, _)) =
        object_query
            .iter()
            .find(|(object, _, _, object_team, stats, production, _)| {
                object.ref_id == ref_id
                    && object_team.0 == team
                    && !stats.destroyed()
                    && production.is_some()
                    && buildings::production_window_kind(object.kind).is_some()
            })
    else {
        return false;
    };

    production_window.open =
        buildings::production_window_kind(object.kind).map(|window_kind| ProductionWindow {
            building_ref_id: object.ref_id,
            kind: window_kind,
            selected_index: 0,
            queue_selected_index: 0,
            expanded: false,
            full_selector: None,
            pressed_button: None,
        });
    production_window.open.is_some()
}

fn select_and_focus_object(
    commands: &mut Commands,
    ref_id: u32,
    camera_query: &mut Query<&mut Transform, With<MainCamera>>,
    selection: &mut SelectionState,
    object_query: &Query<
        (
            &GameObjectEntity,
            &Transform,
            &Selectable,
            &ObjectTeam,
            &ObjectStats,
            Option<&BuildingProduction>,
            Option<&RobotGroup>,
        ),
        Without<MainCamera>,
    >,
    selection_markers: &Query<Entity, With<SelectionMarker>>,
    selection_health_bars: &Query<Entity, With<SelectionHealthBar>>,
) {
    focus_object_with_options(
        commands,
        ref_id,
        true,
        false,
        camera_query,
        selection,
        object_query,
        selection_markers,
        selection_health_bars,
        None,
    );
}

fn focus_object_with_options(
    commands: &mut Commands,
    ref_id: u32,
    select_obj: bool,
    open_gui: bool,
    camera_query: &mut Query<&mut Transform, With<MainCamera>>,
    selection: &mut SelectionState,
    object_query: &Query<
        (
            &GameObjectEntity,
            &Transform,
            &Selectable,
            &ObjectTeam,
            &ObjectStats,
            Option<&BuildingProduction>,
            Option<&RobotGroup>,
        ),
        Without<MainCamera>,
    >,
    selection_markers: &Query<Entity, With<SelectionMarker>>,
    selection_health_bars: &Query<Entity, With<SelectionHealthBar>>,
    production_window: Option<&mut ProductionWindowState>,
) -> bool {
    let ref_id = normalized_selection_ref(ref_id, object_query);
    let Some((object, transform, selectable, team, stats, _, _)) = object_query
        .iter()
        .find(|(object, _, _, _, stats, _, _)| object.ref_id == ref_id && !stats.destroyed())
    else {
        return false;
    };

    let position = transform.translation.truncate();
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        camera_transform.translation.x = position.x;
        camera_transform.translation.y = position.y;
    }

    if select_obj {
        selection.selected_refs.clear();
        selection.selected_refs.push(ref_id);

        for marker in selection_markers {
            commands.entity(marker).despawn();
        }
        for bar in selection_health_bars {
            commands.entity(bar).despawn();
        }

        spawn_selection_marker(
            commands,
            object.ref_id,
            position,
            selectable.selection_size,
            team.0,
            *stats,
        );
    }

    if open_gui {
        if let Some(production_window) = production_window {
            open_production_window_for_ref(ref_id, TeamType::Red, object_query, production_window);
        }
    }

    true
}

fn select_next_group_object(
    commands: &mut Commands,
    group: ObjectSelectionGroup,
    team: TeamType,
    command_state: &mut HudCommandState,
    selection: &mut SelectionState,
    object_query: &Query<
        (
            &GameObjectEntity,
            &Transform,
            &Selectable,
            &ObjectTeam,
            &ObjectStats,
            Option<&BuildingProduction>,
            Option<&RobotGroup>,
        ),
        Without<MainCamera>,
    >,
    selection_markers: &Query<Entity, With<SelectionMarker>>,
    selection_health_bars: &Query<Entity, With<SelectionHealthBar>>,
) {
    let last_ref = last_selected_group_ref(command_state, group);
    let candidates: Vec<SelectionCandidate> = object_query
        .iter()
        .map(
            |(object, _, _, object_team, stats, _, group)| SelectionCandidate {
                ref_id: object.ref_id,
                kind: object.kind,
                team: object_team.0,
                destroyed: stats.destroyed(),
                leader_ref_id: group.map(|group| group.leader_ref_id),
            },
        )
        .collect();
    let Some(next_ref) = next_ordered_selection_ref(&candidates, group, team, last_ref) else {
        return;
    };

    set_last_selected_group_ref(command_state, group, Some(next_ref));
    selection.selected_refs.clear();
    selection.selected_refs.push(next_ref);

    for marker in selection_markers {
        commands.entity(marker).despawn();
    }
    for bar in selection_health_bars {
        commands.entity(bar).despawn();
    }

    if let Some((object, transform, selectable, object_team, stats, _, _)) = object_query
        .iter()
        .find(|(object, _, _, _, _, _, _)| object.ref_id == next_ref)
    {
        spawn_selection_marker(
            commands,
            object.ref_id,
            transform.translation.truncate(),
            selectable.selection_size,
            object_team.0,
            *stats,
        );
    }
}

fn last_selected_group_ref(state: &HudCommandState, group: ObjectSelectionGroup) -> Option<u32> {
    match group {
        ObjectSelectionGroup::Robot => state.last_robot_ref,
        ObjectSelectionGroup::Vehicle => state.last_vehicle_ref,
        ObjectSelectionGroup::Cannon => state.last_cannon_ref,
    }
}

fn set_last_selected_group_ref(
    state: &mut HudCommandState,
    group: ObjectSelectionGroup,
    ref_id: Option<u32>,
) {
    match group {
        ObjectSelectionGroup::Robot => state.last_robot_ref = ref_id,
        ObjectSelectionGroup::Vehicle => state.last_vehicle_ref = ref_id,
        ObjectSelectionGroup::Cannon => state.last_cannon_ref = ref_id,
    }
}

#[derive(Clone, Copy)]
pub(crate) struct SelectionCandidate {
    pub(crate) ref_id: u32,
    pub(crate) kind: ObjectKind,
    pub(crate) team: TeamType,
    pub(crate) destroyed: bool,
    pub(crate) leader_ref_id: Option<u32>,
}

pub(crate) fn next_ordered_selection_ref(
    candidates: &[SelectionCandidate],
    group: ObjectSelectionGroup,
    team: TeamType,
    last_ref: Option<u32>,
) -> Option<u32> {
    let mut refs: Vec<u32> = candidates
        .iter()
        .filter(|candidate| {
            candidate.team == team
                && !candidate.destroyed
                && object_matches_selection_group(candidate.kind, group)
                && candidate
                    .leader_ref_id
                    .is_none_or(|leader_ref_id| leader_ref_id == candidate.ref_id)
        })
        .map(|candidate| candidate.ref_id)
        .collect();
    refs.sort_unstable();

    if refs.is_empty() {
        return None;
    }

    let Some(last_ref) = last_ref else {
        return refs.first().copied();
    };

    refs.iter()
        .copied()
        .find(|ref_id| *ref_id > last_ref)
        .or_else(|| refs.first().copied())
}

fn object_matches_selection_group(kind: ObjectKind, group: ObjectSelectionGroup) -> bool {
    matches!(
        (group, kind),
        (ObjectSelectionGroup::Robot, ObjectKind::Robot(_))
            | (ObjectSelectionGroup::Vehicle, ObjectKind::Vehicle(_))
            | (ObjectSelectionGroup::Cannon, ObjectKind::Cannon(_))
    )
}

fn normalized_selection_ref(
    ref_id: u32,
    object_query: &Query<
        (
            &GameObjectEntity,
            &Transform,
            &Selectable,
            &ObjectTeam,
            &ObjectStats,
            Option<&BuildingProduction>,
            Option<&RobotGroup>,
        ),
        Without<MainCamera>,
    >,
) -> u32 {
    object_query
        .iter()
        .find(|(object, _, _, _, stats, _, _)| object.ref_id == ref_id && !stats.destroyed())
        .and_then(|(_, _, _, _, _, _, group)| group.map(|group| group.leader_ref_id))
        .and_then(|leader_ref_id| {
            object_query
                .iter()
                .any(|(object, _, _, _, stats, _, _)| {
                    object.ref_id == leader_ref_id && !stats.destroyed()
                })
                .then_some(leader_ref_id)
        })
        .unwrap_or(ref_id)
}

fn expand_selected_refs_for_orders(
    selected_refs: &[u32],
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&RobotGroup>,
        Option<&BridgeFootprint>,
    )>,
) -> Vec<u32> {
    let mut refs = Vec::new();
    for (object, _, _, _, stats, group, _) in object_query {
        if stats.destroyed() {
            continue;
        }

        let selected = if let Some(group) = group {
            selected_refs.contains(&group.leader_ref_id)
        } else {
            selected_refs.contains(&object.ref_id)
        };

        if selected && !refs.contains(&object.ref_id) {
            refs.push(object.ref_id);
        }
    }
    refs
}

fn movement_path_for_route(
    route: &[Vec2],
    layer_offset: Vec2,
    speed: f32,
    attempt_run: bool,
) -> MovementPath {
    let path = MovementPath::new(
        route
            .iter()
            .map(|waypoint| *waypoint + layer_offset)
            .collect(),
        speed,
    );
    if attempt_run {
        path.with_run_attempt()
    } else {
        path
    }
}

fn movement_player_move_path_for_route(
    route: &[Vec2],
    layer_offset: Vec2,
    speed: f32,
    attempt_run: bool,
    attack_to: bool,
) -> MovementPath {
    let Some((last, route_prefix)) = route.split_last() else {
        return movement_path_for_route(route, layer_offset, speed, attempt_run);
    };
    let mut waypoints: Vec<_> = route_prefix
        .iter()
        .map(|waypoint| MovementWaypoint::move_to(*waypoint + layer_offset))
        .collect();
    waypoints.push(MovementWaypoint::player_move_to(
        *last + layer_offset,
        attack_to,
    ));
    let path = MovementPath::from_typed(waypoints, speed);
    if attempt_run {
        path.with_run_attempt()
    } else {
        path
    }
}

fn movement_attack_path_for_route(
    route: &[Vec2],
    layer_offset: Vec2,
    target_ref_id: u32,
    target_position: Vec2,
    speed: f32,
) -> MovementPath {
    let mut waypoints: Vec<_> = route
        .iter()
        .map(|waypoint| MovementWaypoint::move_to(*waypoint + layer_offset))
        .collect();
    waypoints.push(MovementWaypoint::player_attack_target(
        target_ref_id,
        target_position + layer_offset,
    ));
    MovementPath::from_typed(waypoints, speed)
}

fn source_can_overwrite_waypoint(waypoint: SourceWaypoint) -> bool {
    waypoint.mode != SourceWaypointMode::ForceMove
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceCurrentWaypointStage {
    CraneRepair(CraneRepairStage),
    UnitRepair(UnitRepairStage),
    EnterFort(EnterFortStage),
}

fn source_current_waypoint_stage(
    crane_repair: Option<&CraneRepairTarget>,
    unit_repair: Option<&UnitRepairTarget>,
    enter_fort: Option<&EnterFortTarget>,
) -> Option<SourceCurrentWaypointStage> {
    crane_repair
        .map(|target| SourceCurrentWaypointStage::CraneRepair(target.stage))
        .or_else(|| unit_repair.map(|target| SourceCurrentWaypointStage::UnitRepair(target.stage)))
        .or_else(|| enter_fort.map(|target| SourceCurrentWaypointStage::EnterFort(target.stage)))
}

fn source_can_overwrite_current_stage(stage: Option<SourceCurrentWaypointStage>) -> bool {
    match stage {
        None => true,
        Some(SourceCurrentWaypointStage::CraneRepair(stage)) => {
            stage == CraneRepairStage::GotoEntrance
        }
        Some(SourceCurrentWaypointStage::UnitRepair(stage)) => {
            matches!(stage, UnitRepairStage::GotoEntrance | UnitRepairStage::Wait)
        }
        Some(SourceCurrentWaypointStage::EnterFort(stage)) => stage == EnterFortStage::GotoEntrance,
    }
}

fn source_should_queue_behind_current_stage(stage: Option<SourceCurrentWaypointStage>) -> bool {
    !source_can_overwrite_current_stage(stage)
}

fn protected_unit_repair_resume_waypoints(
    path: &MovementPath,
    current_stage: Option<SourceCurrentWaypointStage>,
    existing_path: Option<&MovementPath>,
) -> Option<Vec<MovementWaypoint>> {
    if !matches!(
        current_stage,
        Some(SourceCurrentWaypointStage::UnitRepair(
            UnitRepairStage::EnterBuilding | UnitRepairStage::ExitBuilding
        ))
    ) {
        return None;
    }
    let prefix_len = existing_path
        .filter(|existing| movement_path_has_prefix(path, existing))
        .map_or(0, |existing| existing.typed_waypoints.len());
    Some(path.typed_waypoints[prefix_len..].to_vec())
}

fn insert_waypoint_command_path(
    commands: &mut Commands,
    entity: Entity,
    path: MovementPath,
    current_stage: Option<SourceCurrentWaypointStage>,
    existing_path: Option<&MovementPath>,
) {
    let mut entity_commands = commands.entity(entity);
    let queue_behind_current = source_should_queue_behind_current_stage(current_stage);
    if let Some(resume_waypoints) =
        protected_unit_repair_resume_waypoints(&path, current_stage, existing_path)
    {
        entity_commands
            .remove::<AcceptedEmptyWaypointCommand>()
            .insert(RepairResumeWaypoints(resume_waypoints));
        return;
    }
    let accepted_empty = path.is_empty() && !queue_behind_current;
    entity_commands.remove::<SourceLocationInterpolation>();
    if !queue_behind_current {
        if !accepted_empty {
            entity_commands.remove::<AttackTargetLifecycleComponents>();
        }
        entity_commands
            .remove::<PickupGrenadesTarget>()
            .remove::<EnterTarget>()
            .remove::<EnterFortTarget>()
            .remove::<CraneRepairTarget>()
            .remove::<UnitRepairTarget>();
    }
    entity_commands.remove::<JustLeftCannon>();
    if accepted_empty {
        entity_commands
            .remove::<MovementPath>()
            .insert(AcceptedEmptyWaypointCommand);
    } else {
        entity_commands
            .remove::<AcceptedEmptyWaypointCommand>()
            .insert(path);
    }
}

fn source_preserved_existing_waypoint(
    existing_waypoints: &[SourceWaypoint],
) -> Option<SourceWaypoint> {
    existing_waypoints
        .first()
        .copied()
        .filter(|waypoint| !source_can_overwrite_waypoint(*waypoint))
}

fn source_waypoint_matches_preserved(left: SourceWaypoint, right: SourceWaypoint) -> bool {
    left.mode == right.mode
        && left.ref_id == right.ref_id
        && left.x == right.x
        && left.y == right.y
        && left.attack_to == right.attack_to
        && left.player_given == right.player_given
}

fn movement_path_has_prefix(path: &MovementPath, prefix: &MovementPath) -> bool {
    path.typed_waypoints.len() >= prefix.typed_waypoints.len()
        && path
            .typed_waypoints
            .iter()
            .zip(&prefix.typed_waypoints)
            .all(|(left, right)| left == right)
}

fn source_snapshot_for_ref(
    ref_id: u32,
    snapshots: &[MouseCommandObjectSnapshot],
) -> Option<&MouseCommandObjectSnapshot> {
    snapshots.iter().find(|snapshot| snapshot.ref_id == ref_id)
}

fn source_can_set_waypoints(snapshot: &MouseCommandObjectSnapshot) -> bool {
    !snapshot.stats.destroyed()
        && matches!(
            snapshot.kind,
            ObjectKind::Robot(_) | ObjectKind::Vehicle(_) | ObjectKind::Cannon(_)
        )
}

fn source_can_move(snapshot: &MouseCommandObjectSnapshot) -> bool {
    snapshot.mobile && snapshot.stats.move_speed > 0.0 && !snapshot.stats.destroyed()
}

fn source_is_minion(snapshot: &MouseCommandObjectSnapshot) -> bool {
    snapshot
        .group_leader_ref_id
        .is_some_and(|leader_ref_id| leader_ref_id != snapshot.ref_id)
}

fn source_waypoint_target<'a>(
    waypoint: SourceWaypoint,
    snapshots: &'a [MouseCommandObjectSnapshot],
) -> Option<&'a MouseCommandObjectSnapshot> {
    let ref_id = u32::try_from(waypoint.ref_id).ok()?;
    source_snapshot_for_ref(ref_id, snapshots)
}

fn source_can_attack_object(
    attacker: &MouseCommandObjectSnapshot,
    target: &MouseCommandObjectSnapshot,
) -> bool {
    units::attack::can_attack_target_identity(
        attacker.team,
        attacker.stats,
        attacker.grenade_amount.unwrap_or(0),
        target.team,
        target.stats,
    )
}

fn source_check_waypoint(
    object: &MouseCommandObjectSnapshot,
    mut waypoint: SourceWaypoint,
    snapshots: &[MouseCommandObjectSnapshot],
) -> Option<SourceWaypoint> {
    if !object.stats.can_attack() {
        waypoint.attack_to = false;
    }

    match waypoint.mode {
        SourceWaypointMode::Move => {
            if !source_can_move(object) {
                return None;
            }
        }
        SourceWaypointMode::ForceMove | SourceWaypointMode::Dodge => {
            if !source_can_move(object) {
                return None;
            }
            waypoint.mode = SourceWaypointMode::Move;
        }
        SourceWaypointMode::Agro => {
            waypoint.mode = SourceWaypointMode::Attack;
        }
        SourceWaypointMode::Attack => {
            let target = source_waypoint_target(waypoint, snapshots)?;
            if !source_can_attack_object(object, target) {
                return None;
            }
        }
        SourceWaypointMode::Enter => {
            if !source_can_move(object) || !matches!(object.kind, ObjectKind::Robot(_)) {
                return None;
            }
            let target = source_waypoint_target(waypoint, snapshots)?;
            if !can_be_entered(target.kind, target.team, target.stats) {
                return None;
            }
        }
        SourceWaypointMode::CraneRepair => {
            if !source_can_move(object)
                || !matches!(object.kind, ObjectKind::Vehicle(VehicleType::Crane))
            {
                return None;
            }
            let target = source_waypoint_target(waypoint, snapshots)?;
            if !units::vehicles::crane::can_repair_target(
                target.kind,
                target.team,
                object.team,
                target.stats,
            ) {
                return None;
            }
        }
        SourceWaypointMode::UnitRepair => {
            if !source_can_move(object)
                || !buildings::can_repair_target_unit(object.kind, object.team, object.stats)
            {
                return None;
            }
            let target = source_waypoint_target(waypoint, snapshots)?;
            if !buildings::can_repair_unit(target.kind, target.team, object.team, target.stats) {
                return None;
            }
        }
        SourceWaypointMode::EnterFort => {
            if !source_can_move(object) {
                return None;
            }
            let target = source_waypoint_target(waypoint, snapshots)?;
            if !can_enter_fort(target.kind, target.team, object.team, target.stats) {
                return None;
            }
        }
        SourceWaypointMode::PickupGrenades => {
            if !source_can_move(object)
                || !can_pickup_grenades(object.kind, object.grenade_amount.unwrap_or(0))
            {
                return None;
            }
            let target = source_waypoint_target(waypoint, snapshots)?;
            if target.stats.destroyed() || !is_grenade_box(target.kind) {
                return None;
            }
        }
    }

    Some(waypoint)
}

fn source_process_waypoint_data(
    packet: &SendWaypointsPacket,
    ok_team: TeamType,
    snapshots: &[MouseCommandObjectSnapshot],
    existing_waypoints: &[SourceWaypoint],
) -> Option<Vec<SourceWaypoint>> {
    let ref_id = u32::try_from(packet.ref_id).ok()?;
    let object = source_snapshot_for_ref(ref_id, snapshots)?;
    if object.team == TeamType::Null || object.team != ok_team {
        return None;
    }
    if !source_can_set_waypoints(object) || source_is_minion(object) {
        return None;
    }

    let mut accepted = Vec::new();
    let preserved_first = source_preserved_existing_waypoint(existing_waypoints);
    if let Some(first_waypoint) = preserved_first {
        accepted.push(first_waypoint);
    }
    accepted.extend(
        packet
            .waypoints
            .iter()
            .copied()
            .enumerate()
            .filter(|(index, waypoint)| {
                *index != 0
                    || preserved_first
                        .map(|first| !source_waypoint_matches_preserved(first, *waypoint))
                        .unwrap_or(true)
            })
            .map(|(_, waypoint)| waypoint)
            .filter_map(|waypoint| source_check_waypoint(object, waypoint, snapshots)),
    );
    Some(accepted)
}

fn source_rally_waypoint_from_point(point: Vec2) -> SourceWaypoint {
    SourceWaypoint {
        mode: SourceWaypointMode::Move,
        ref_id: -1,
        x: point.x.round() as i32,
        y: (-point.y).round() as i32,
        attack_to: true,
        player_given: true,
    }
}

fn source_rally_point_from_waypoint(waypoint: SourceWaypoint) -> Vec2 {
    Vec2::new(waypoint.x as f32, -(waypoint.y as f32))
}

fn source_check_rallypoint(waypoint: SourceWaypoint) -> Option<SourceWaypoint> {
    (waypoint.mode == SourceWaypointMode::Move).then_some(waypoint)
}

fn source_process_rallypoint_data(
    packet: &SendRallypointsPacket,
    ok_team: TeamType,
    snapshots: &[MouseCommandObjectSnapshot],
) -> Option<Vec<SourceWaypoint>> {
    let ref_id = u32::try_from(packet.ref_id).ok()?;
    let object = source_snapshot_for_ref(ref_id, snapshots)?;
    if object.team == TeamType::Null || object.team != ok_team {
        return None;
    }
    if object.stats.destroyed() || !buildings::can_set_rallypoints(object.kind) {
        return None;
    }

    Some(
        packet
            .waypoints
            .iter()
            .copied()
            .filter_map(source_check_rallypoint)
            .collect(),
    )
}

fn source_relay_rally_points(
    ref_id: u32,
    points: &[Vec2],
    ok_team: TeamType,
    snapshots: &[MouseCommandObjectSnapshot],
) -> Option<Vec<Vec2>> {
    let source_ref_id = i32::try_from(ref_id).ok()?;
    let packet = SendRallypointsPacket {
        ref_id: source_ref_id,
        waypoints: points
            .iter()
            .copied()
            .map(source_rally_waypoint_from_point)
            .collect(),
    };
    let wire_packet = packet.encode_packet();
    let relayed_packet = SendRallypointsPacket::decode_payload(&wire_packet[8..])?;
    if relayed_packet.ref_id != source_ref_id {
        return None;
    }
    Some(
        source_process_rallypoint_data(&relayed_packet, ok_team, snapshots)?
            .into_iter()
            .map(source_rally_point_from_waypoint)
            .collect(),
    )
}

fn previous_cursor_kind_for_source_waypoint(mode: SourceWaypointMode) -> ZCursorKind {
    match mode {
        SourceWaypointMode::Move
        | SourceWaypointMode::ForceMove
        | SourceWaypointMode::EnterFort
        | SourceWaypointMode::Dodge => ZCursorKind::Placed,
        SourceWaypointMode::PickupGrenades => ZCursorKind::Grabbed,
        SourceWaypointMode::Enter => ZCursorKind::Entered,
        SourceWaypointMode::Attack | SourceWaypointMode::Agro => ZCursorKind::Attacked,
        SourceWaypointMode::CraneRepair | SourceWaypointMode::UnitRepair => ZCursorKind::Repaired,
    }
}

const WAYPOINT_FEEDBACK_DURATION: f32 = 3.0;
const WAYPOINT_FEEDBACK_TICK: f32 = 0.1;
const WAYPOINT_FEEDBACK_DOT_STEP: f32 = 4.0;
const WAYPOINT_FEEDBACK_DOT_SIZE: Vec2 = Vec2::splat(2.0);
const WAYPOINT_FEEDBACK_Z: f32 = 29.0;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WaypointFeedbackPath {
    start: Vec2,
    waypoints: Vec<SourceWaypoint>,
}

impl WaypointFeedbackPath {
    fn new(start: Vec2, waypoints: Vec<SourceWaypoint>) -> Self {
        Self { start, waypoints }
    }
}

#[derive(Default, Resource)]
pub(crate) struct WaypointFeedbackState {
    transient_paths: HashMap<u32, WaypointFeedbackEntry>,
    waypoint_i: usize,
    elapsed: f32,
    dirty: bool,
}

#[derive(Clone, Debug)]
struct WaypointFeedbackEntry {
    path: WaypointFeedbackPath,
    remaining: f32,
}

impl WaypointFeedbackState {
    pub(crate) fn show_object_paths(&mut self, paths: HashMap<u32, WaypointFeedbackPath>) {
        if paths.is_empty() {
            return;
        }

        for (ref_id, path) in paths {
            self.transient_paths.insert(
                ref_id,
                WaypointFeedbackEntry {
                    path,
                    remaining: WAYPOINT_FEEDBACK_DURATION,
                },
            );
        }
        self.dirty = true;
    }

    fn tick(&mut self, delta: f32) {
        if !self.transient_paths.is_empty() {
            for entry in self.transient_paths.values_mut() {
                entry.remaining = (entry.remaining - delta).max(0.0);
            }
            let before = self.transient_paths.len();
            self.transient_paths
                .retain(|_, entry| entry.remaining > 0.0);
            if before != self.transient_paths.len() {
                self.dirty = true;
            }
        }

        self.elapsed += delta;
        if self.elapsed >= WAYPOINT_FEEDBACK_TICK {
            self.elapsed %= WAYPOINT_FEEDBACK_TICK;
            self.waypoint_i = (self.waypoint_i + 1) % 4;
            self.dirty = true;
        }
    }

    fn transient_paths(&self) -> impl Iterator<Item = &WaypointFeedbackPath> {
        self.transient_paths.values().map(|entry| &entry.path)
    }
}

#[derive(Component)]
pub(crate) struct WaypointFeedbackDot;

fn source_waypoint_at(position: Vec2, mode: SourceWaypointMode, ref_id: i32) -> SourceWaypoint {
    SourceWaypoint {
        mode,
        ref_id,
        x: position.x.round() as i32,
        y: position.y.round() as i32,
        attack_to: true,
        player_given: true,
    }
}

fn waypoint_feedback_path_from_movement(
    start: Vec2,
    path: &MovementPath,
    layer_offset: Vec2,
    final_waypoint: Option<SourceWaypoint>,
) -> WaypointFeedbackPath {
    let mut waypoints = source_waypoints_from_movement_path(path, layer_offset);
    if let Some(final_waypoint) = final_waypoint {
        waypoints.push(final_waypoint);
    }
    WaypointFeedbackPath::new(start, waypoints)
}

fn source_waypoint_position(
    waypoint: SourceWaypoint,
    target_positions: &HashMap<u32, Vec2>,
) -> Vec2 {
    if matches!(
        waypoint.mode,
        SourceWaypointMode::Attack | SourceWaypointMode::Agro
    ) {
        if let Ok(ref_id) = u32::try_from(waypoint.ref_id) {
            if let Some(position) = target_positions.get(&ref_id) {
                return *position;
            }
        }
    }
    Vec2::new(waypoint.x as f32, waypoint.y as f32)
}

fn source_waypoint_line_points(start: Vec2, end: Vec2, waypoint_i: usize) -> Vec<Vec2> {
    let delta = end - start;
    let dist = delta.length();
    if dist <= f32::EPSILON {
        return Vec::new();
    }

    let step = delta * (WAYPOINT_FEEDBACK_DOT_STEP / dist);
    let mut point = start + step * (waypoint_i as f32 / WAYPOINT_FEEDBACK_DOT_STEP);
    let mut count = (dist / WAYPOINT_FEEDBACK_DOT_STEP) as usize + 1;
    if waypoint_i > 0 {
        count = count.saturating_sub(1);
    }

    let mut points = Vec::with_capacity(count);
    for _ in 0..count {
        points.push(point);
        point += step;
    }
    points
}

fn source_waypoint_feedback_points(
    path: &WaypointFeedbackPath,
    waypoint_i: usize,
    target_positions: &HashMap<u32, Vec2>,
) -> Vec<Vec2> {
    let mut points = Vec::new();
    let mut start = path.start;
    for waypoint in &path.waypoints {
        let end = source_waypoint_position(*waypoint, target_positions);
        points.extend(source_waypoint_line_points(start, end, waypoint_i));
        start = end;
    }
    points
}

fn rally_feedback_path_for_open_building(
    production_window: &ProductionWindowState,
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        Option<&MapGridPosition>,
        Option<&BuildingRallyPoints>,
    )>,
) -> Option<WaypointFeedbackPath> {
    let open = production_window.open?;
    let (object, transform, grid, rally_points) = object_query
        .iter()
        .find(|(object, _, _, _)| object.ref_id == open.building_ref_id)?;
    let rally_points = rally_points?;
    if rally_points.points.is_empty() {
        return None;
    }

    let start = if let (ObjectKind::Building(building), Some(grid)) = (object.kind, grid) {
        buildings::production_world_points(building, grid.x, grid.y)
            .map(|(create_center, _)| create_center)
            .unwrap_or_else(|| transform.translation.truncate())
    } else {
        transform.translation.truncate()
    };

    let mut waypoints = Vec::new();
    if let (ObjectKind::Building(building), Some(grid)) = (object.kind, grid) {
        if let Some((_, move_target)) = buildings::production_world_points(building, grid.x, grid.y)
        {
            waypoints.push(source_waypoint_at(
                move_target,
                SourceWaypointMode::ForceMove,
                -1,
            ));
        }
    }
    waypoints.extend(
        rally_points
            .points
            .iter()
            .copied()
            .map(source_rally_waypoint_from_point),
    );
    Some(WaypointFeedbackPath::new(start, waypoints))
}

pub(crate) fn update_waypoint_feedback(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<WaypointFeedbackState>,
    production_window: Res<ProductionWindowState>,
    dot_query: Query<Entity, With<WaypointFeedbackDot>>,
    object_query: Query<(
        &GameObjectEntity,
        &Transform,
        Option<&MapGridPosition>,
        Option<&BuildingRallyPoints>,
    )>,
) {
    state.tick(time.delta_secs());
    let rally_path = rally_feedback_path_for_open_building(&production_window, &object_query);
    if !state.dirty && rally_path.is_none() {
        return;
    }

    for entity in &dot_query {
        commands.entity(entity).despawn();
    }

    let target_positions: HashMap<u32, Vec2> = object_query
        .iter()
        .map(|(object, transform, _, _)| (object.ref_id, transform.translation.truncate()))
        .collect();
    let mut paths: Vec<WaypointFeedbackPath> = state.transient_paths().cloned().collect();
    if let Some(rally_path) = rally_path {
        paths.push(rally_path);
    }

    for point in paths
        .iter()
        .flat_map(|path| source_waypoint_feedback_points(path, state.waypoint_i, &target_positions))
    {
        commands.spawn((
            Sprite::from_color(Color::srgb_u8(170, 170, 170), WAYPOINT_FEEDBACK_DOT_SIZE),
            Transform::from_xyz(point.x, point.y, WAYPOINT_FEEDBACK_Z),
            WaypointFeedbackDot,
            Name::new("waypoint_feedback_dot"),
        ));
    }

    state.dirty = false;
}

#[cfg(test)]
fn source_relay_movement_path(
    ref_id: u32,
    path: MovementPath,
    ok_team: TeamType,
    snapshots: &[MouseCommandObjectSnapshot],
) -> Option<MovementPath> {
    source_relay_movement_path_with_existing(ref_id, path, ok_team, snapshots, None, None)
}

fn source_relay_movement_path_with_existing(
    ref_id: u32,
    path: MovementPath,
    ok_team: TeamType,
    snapshots: &[MouseCommandObjectSnapshot],
    existing_path: Option<&MovementPath>,
    current_stage: Option<SourceCurrentWaypointStage>,
) -> Option<MovementPath> {
    let queue_behind_current = source_should_queue_behind_current_stage(current_stage);
    let packet_waypoints = if queue_behind_current {
        if let Some(existing_path) = existing_path {
            if movement_path_has_prefix(&path, existing_path) {
                &path.typed_waypoints[existing_path.typed_waypoints.len()..]
            } else {
                &path.typed_waypoints
            }
        } else {
            &path.typed_waypoints
        }
    } else {
        &path.typed_waypoints
    };
    let source_ref_id = i32::try_from(ref_id).ok()?;
    let packet = SendWaypointsPacket {
        ref_id: source_ref_id,
        waypoints: packet_waypoints
            .iter()
            .copied()
            .map(source_waypoint_from_movement)
            .collect(),
    };
    let wire_packet = packet.encode_packet();
    let relayed_packet = SendWaypointsPacket::decode_payload(&wire_packet[8..])?;
    if relayed_packet.ref_id != source_ref_id {
        return None;
    }
    let existing_waypoints = if queue_behind_current {
        Vec::new()
    } else {
        source_waypoints_from_existing_path(existing_path)
    };
    let accepted_waypoints =
        source_process_waypoint_data(&relayed_packet, ok_team, snapshots, &existing_waypoints)?;
    let mut typed_waypoints = Vec::with_capacity(accepted_waypoints.len());
    for waypoint in accepted_waypoints {
        typed_waypoints.push(movement_waypoint_from_source(waypoint)?);
    }
    let relayed = if queue_behind_current {
        let mut queued = existing_path
            .cloned()
            .unwrap_or_else(|| MovementPath::from_typed(Vec::new(), path.speed));
        let appended = MovementPath::from_typed(typed_waypoints, path.speed);
        queued.waypoints.extend(appended.waypoints);
        queued.typed_waypoints.extend(appended.typed_waypoints);
        queued.attempt_run |= path.attempt_run;
        queued
    } else {
        MovementPath::from_typed(typed_waypoints, path.speed)
    };
    if path.attempt_run {
        Some(relayed.with_run_attempt())
    } else {
        Some(relayed)
    }
}

#[cfg(test)]
fn source_relay_pickup_grenades_path(
    ref_id: u32,
    path: MovementPath,
    target_ref_id: u32,
    waypoint_position: Vec2,
    ok_team: TeamType,
    snapshots: &[MouseCommandObjectSnapshot],
) -> Option<(MovementPath, u32)> {
    source_relay_special_waypoint_path(
        ref_id,
        path,
        target_ref_id,
        waypoint_position,
        SourceWaypointMode::PickupGrenades,
        ok_team,
        snapshots,
    )
}

#[cfg(test)]
fn source_relay_special_waypoint_path(
    ref_id: u32,
    path: MovementPath,
    target_ref_id: u32,
    waypoint_position: Vec2,
    mode: SourceWaypointMode,
    ok_team: TeamType,
    snapshots: &[MouseCommandObjectSnapshot],
) -> Option<(MovementPath, u32)> {
    source_relay_special_waypoint_path_with_existing(
        ref_id,
        path,
        target_ref_id,
        waypoint_position,
        mode,
        ok_team,
        snapshots,
        None,
        None,
    )
}

fn source_relay_special_waypoint_path_with_existing(
    ref_id: u32,
    path: MovementPath,
    target_ref_id: u32,
    waypoint_position: Vec2,
    mode: SourceWaypointMode,
    ok_team: TeamType,
    snapshots: &[MouseCommandObjectSnapshot],
    existing_path: Option<&MovementPath>,
    current_stage: Option<SourceCurrentWaypointStage>,
) -> Option<(MovementPath, u32)> {
    if !source_can_overwrite_current_stage(current_stage) {
        return None;
    }

    let source_ref_id = i32::try_from(ref_id).ok()?;
    let source_target_ref_id = i32::try_from(target_ref_id).ok()?;
    let mut waypoints: Vec<_> = path
        .typed_waypoints
        .iter()
        .copied()
        .map(source_waypoint_from_movement)
        .collect();
    waypoints.push(SourceWaypoint {
        mode,
        ref_id: source_target_ref_id,
        x: waypoint_position.x.round() as i32,
        y: waypoint_position.y.round() as i32,
        attack_to: true,
        player_given: true,
    });

    let packet = SendWaypointsPacket {
        ref_id: source_ref_id,
        waypoints,
    };
    let wire_packet = packet.encode_packet();
    let relayed_packet = SendWaypointsPacket::decode_payload(&wire_packet[8..])?;
    if relayed_packet.ref_id != source_ref_id {
        return None;
    }
    let existing_waypoints = source_waypoints_from_existing_path(existing_path);
    let accepted_waypoints =
        source_process_waypoint_data(&relayed_packet, ok_team, snapshots, &existing_waypoints)?;
    let (special_waypoint, movement_waypoints) = accepted_waypoints.split_last()?;
    if special_waypoint.mode != mode {
        return None;
    }
    let target_ref_id = u32::try_from(special_waypoint.ref_id).ok()?;
    let mut typed_waypoints = Vec::with_capacity(movement_waypoints.len());
    for waypoint in movement_waypoints {
        typed_waypoints.push(movement_waypoint_from_source(*waypoint)?);
    }
    let relayed = MovementPath::from_typed(typed_waypoints, path.speed);
    let relayed = if path.attempt_run {
        relayed.with_run_attempt()
    } else {
        relayed
    };
    Some((relayed, target_ref_id))
}

fn move_target_is_near_flag(
    world_pos: Vec2,
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&RobotGroup>,
        Option<&BridgeFootprint>,
    )>,
) -> bool {
    object_query
        .iter()
        .any(|(object, transform, _, _, stats, _, _)| {
            !stats.destroyed()
                && matches!(object.kind, ObjectKind::MapItem(id) if id == ItemType::Flag as u8)
                && transform.translation.truncate().distance(world_pos) <= 32.0
        })
}

fn player_move_waypoint_attack_to(
    ref_id: u32,
    position: Vec2,
    team: TeamType,
    stats: ObjectStats,
    targets: &[PassiveCombatTargetSnapshot],
    ctrl_down: bool,
    alt_down: bool,
) -> bool {
    if !stats.can_attack() || team == TeamType::Null || alt_down {
        return false;
    }
    if ctrl_down {
        return true;
    }

    unit_behavior::attack_to_target_choices(ref_id, position, team, stats, targets.iter().copied())
        .is_empty()
}

fn eject_target_for_right_click(
    world_pos: Vec2,
    selected_refs: &[u32],
    team: TeamType,
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&RobotGroup>,
        Option<&BridgeFootprint>,
    )>,
) -> Option<u32> {
    let [selected_ref] = selected_refs else {
        return None;
    };

    object_query
        .iter()
        .find(
            |(object, transform, selectable, object_team, stats, _, _)| {
                object.ref_id == *selected_ref
                    && object_team.0 == team
                    && can_eject_drivers(object.kind, **stats)
                    && transform.translation.truncate().distance(world_pos) <= selectable.radius
            },
        )
        .map(|(object, _, _, _, _, _, _)| object.ref_id)
}

pub(crate) fn handle_building_rally_point_commands(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    hud_layout: Res<HudLayout>,
    production_window: Res<ProductionWindowState>,
    mut previous_cursor: ResMut<PreviousCursorState>,
    mut buildings: Query<(
        &GameObjectEntity,
        &ObjectTeam,
        &ObjectStats,
        &mut BuildingRallyPoints,
    )>,
) {
    if production_window.input_captured || !mouse.just_released(MouseButton::Right) {
        return;
    }
    let Some(window_state) = production_window.open else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(screen_pos) = window.cursor_position() else {
        return;
    };
    let window_size = Vec2::new(window.width(), window.height());
    let in_hud =
        screen_pos.x >= window_size.x - HUD_WIDTH || screen_pos.y >= window_size.y - HUD_HEIGHT;
    let world_pos = if in_hud {
        let Some(minimap_world) = hud_layout.minimap_screen_to_world(screen_pos, window_size)
        else {
            return;
        };
        minimap_world
    } else {
        let Some(world_pos) = cursor_world_position(&windows, &camera_query) else {
            return;
        };
        world_pos
    };

    if !world_pos.is_finite() {
        return;
    }

    let append = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    for (object, team, stats, mut rally_points) in &mut buildings {
        if object.ref_id == window_state.building_ref_id
            && team.0 == TeamType::Red
            && !stats.destroyed()
        {
            let mut candidate_rally_points = BuildingRallyPoints {
                points: rally_points.points.clone(),
            };
            apply_building_rally_point(&mut candidate_rally_points, world_pos, append);
            let snapshot = MouseCommandObjectSnapshot {
                ref_id: object.ref_id,
                kind: object.kind,
                position: Vec2::ZERO,
                selection_size: Vec2::ZERO,
                team: team.0,
                stats: *stats,
                grenade_amount: None,
                mobile: false,
                group_leader_ref_id: None,
            };
            let Some(relayed_points) = source_relay_rally_points(
                object.ref_id,
                &candidate_rally_points.points,
                TeamType::Red,
                &[snapshot],
            ) else {
                break;
            };
            rally_points.points = relayed_points;
            previous_cursor.show(ZCursorKind::Placed, world_pos);
            break;
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct MouseCommandInput<'w> {
    mouse: Res<'w, ButtonInput<MouseButton>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    eject_vehicle_packets: ResMut<'w, EjectVehiclePacketQueue>,
    registration: Res<'w, RegistrationState>,
    settings: Res<'w, PerpetualServerSettings>,
    local_player: Res<'w, LocalPlayerState>,
    news_log: ResMut<'w, NewsLog>,
    login_prompt: Res<'w, LoginPromptState>,
    account_menu: Res<'w, AccountMenuState>,
}

#[derive(Clone, Copy)]
struct SourceCommandActivationAccess {
    registered: bool,
    server_activation_passed: bool,
}

fn source_command_activation_access(input: &MouseCommandInput) -> SourceCommandActivationAccess {
    SourceCommandActivationAccess {
        registered: input.registration.is_registered,
        server_activation_passed: !input.settings.require_login
            || input.settings.ignore_activation
            || input.local_player.bot_logged_in()
            || (input.local_player.logged_in() && input.local_player.activated()),
    }
}

fn source_activation_rejection(
    kind: ObjectKind,
    access: SourceCommandActivationAccess,
) -> Option<&'static str> {
    if !units::requires_activation(kind) {
        return None;
    }
    if !access.registered {
        return Some("move unit error: registration required, please visit www.nighsoft.com");
    }
    if !access.server_activation_passed {
        return Some("move unit error: activation required, please visit www.nighsoft.com");
    }
    None
}

fn source_filter_activation_refs(
    refs: &[u32],
    kinds: &HashMap<u32, ObjectKind>,
    access: SourceCommandActivationAccess,
    news_log: &mut NewsLog,
) -> Vec<u32> {
    refs.iter()
        .copied()
        .filter(|ref_id| {
            let Some(kind) = kinds.get(ref_id).copied() else {
                return false;
            };
            if let Some(message) = source_activation_rejection(kind, access) {
                news_log.relay_source_news(message, 0, 0, 0);
                false
            } else {
                true
            }
        })
        .collect()
}

fn source_filter_pending_activation(
    pending: &mut [PendingMouseCommand],
    kinds: &HashMap<u32, ObjectKind>,
    access: SourceCommandActivationAccess,
    news_log: &mut NewsLog,
) {
    let attempted_refs = pending
        .iter()
        .flat_map(pending_command_selected_refs)
        .copied()
        .collect::<HashSet<_>>();
    let allowed = source_filter_activation_refs(
        &attempted_refs.iter().copied().collect::<Vec<_>>(),
        kinds,
        access,
        news_log,
    )
    .into_iter()
    .collect::<HashSet<_>>();

    for command in pending {
        match command {
            PendingMouseCommand::Move(command) => {
                command.selected_refs.retain(|r| allowed.contains(r))
            }
            PendingMouseCommand::Attack(command) => {
                command.selected_refs.retain(|r| allowed.contains(r))
            }
            PendingMouseCommand::PickupGrenades(command) => {
                command.selected_refs.retain(|r| allowed.contains(r));
                command.robot_refs.retain(|r| allowed.contains(r));
            }
            PendingMouseCommand::UnitRepair(command) => {
                command.selected_refs.retain(|r| allowed.contains(r));
                command.unit_refs.retain(|r| allowed.contains(r));
            }
            PendingMouseCommand::CraneRepair(command) => {
                command.selected_refs.retain(|r| allowed.contains(r));
                command.crane_refs.retain(|r| allowed.contains(r));
            }
            PendingMouseCommand::Enter(command) => {
                command.selected_refs.retain(|r| allowed.contains(r));
                command.robot_refs.retain(|r| allowed.contains(r));
            }
            PendingMouseCommand::EnterFort(command) => {
                command.selected_refs.retain(|r| allowed.contains(r));
                command.robot_refs.retain(|r| allowed.contains(r));
            }
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct MouseCommandPortraitFeedback<'w> {
    rng: ResMut<'w, CombatRng>,
    portrait_state: ResMut<'w, SelectedPortraitAnimationState>,
    portrait_sounds: ResMut<'w, PortraitAnimationSoundQueue>,
}

impl MouseCommandPortraitFeedback<'_> {
    fn start_acknowledge(&mut self, ref_id: Option<u32>, no_way: bool) {
        let Some(ref_id) = ref_id else {
            return;
        };
        let kind = units::acknowledge_portrait_animation(no_way, &mut *self.rng);
        self.portrait_state
            .start(PortraitAnimationEvent { ref_id, kind });
        self.portrait_sounds.pending.push(kind);
    }
}

#[derive(SystemParam)]
pub(crate) struct MouseCommandVisualFeedback<'w> {
    previous_cursor: ResMut<'w, PreviousCursorState>,
    waypoint_feedback: ResMut<'w, WaypointFeedbackState>,
}

#[derive(SystemParam)]
pub(crate) struct MouseCommandUiQueries<'w, 's> {
    selection_markers: Query<'w, 's, (Entity, &'static SelectionMarker)>,
    selection_health_bars: Query<'w, 's, (Entity, &'static SelectionHealthBar)>,
    target_markers: Query<'w, 's, Entity, With<TargetMarker>>,
    drag_boxes: Query<'w, 's, Entity, With<DragSelectionBox>>,
}

#[derive(Clone)]
enum PendingMouseCommand {
    Move(PendingMouseMoveCommand),
    Attack(PendingMouseAttackCommand),
    PickupGrenades(PendingMousePickupGrenadesCommand),
    UnitRepair(PendingMouseUnitRepairCommand),
    CraneRepair(PendingMouseCraneRepairCommand),
    Enter(PendingMouseEnterCommand),
    EnterFort(PendingMouseEnterFortCommand),
}

#[derive(Clone)]
struct PendingMouseMoveCommand {
    world_pos: Vec2,
    selected_refs: Vec<u32>,
    ctrl_down: bool,
    alt_down: bool,
}

#[derive(Clone)]
struct PendingMouseAttackCommand {
    world_pos: Vec2,
    selected_refs: Vec<u32>,
    target_ref_id: u32,
}

#[derive(Clone)]
struct PendingMousePickupGrenadesCommand {
    world_pos: Vec2,
    selected_refs: Vec<u32>,
    robot_refs: Vec<u32>,
    target_ref_id: u32,
    target_position: Vec2,
}

#[derive(Clone)]
struct PendingMouseUnitRepairCommand {
    world_pos: Vec2,
    selected_refs: Vec<u32>,
    unit_refs: Vec<u32>,
    target: UnitRepairTargetInfo,
}

#[derive(Clone)]
struct PendingMouseCraneRepairCommand {
    world_pos: Vec2,
    selected_refs: Vec<u32>,
    crane_refs: Vec<u32>,
    target: CraneRepairTargetInfo,
}

#[derive(Clone)]
struct PendingMouseEnterCommand {
    world_pos: Vec2,
    selected_refs: Vec<u32>,
    robot_refs: Vec<u32>,
    target: EnterTargetInfo,
}

#[derive(Clone)]
struct PendingMouseEnterFortCommand {
    world_pos: Vec2,
    selected_refs: Vec<u32>,
    robot_refs: Vec<u32>,
    target: EnterFortTargetInfo,
}

#[derive(Default, Resource)]
pub(crate) struct PendingMouseMoveCommands {
    pending: Vec<PendingMouseCommand>,
}

#[derive(Clone, Copy)]
struct MouseCommandObjectSnapshot {
    ref_id: u32,
    kind: ObjectKind,
    position: Vec2,
    selection_size: Vec2,
    team: TeamType,
    stats: ObjectStats,
    grenade_amount: Option<u8>,
    mobile: bool,
    group_leader_ref_id: Option<u32>,
}

type MouseCommandLayerSnapshot = (
    Entity,
    u32,
    Vec2,
    Option<MovementPath>,
    Option<SourceCurrentWaypointStage>,
);

#[derive(Clone)]
struct SourceClonedMinionPath {
    entity: Entity,
    ref_id: u32,
    path: MovementPath,
}

fn movement_path_retargeted_to_layer(
    path: &MovementPath,
    source_layer_offset: Vec2,
    target_layer_offset: Vec2,
    speed: f32,
) -> MovementPath {
    let typed_waypoints = path
        .typed_waypoints
        .iter()
        .copied()
        .map(|waypoint| {
            waypoint.with_position(waypoint.position - source_layer_offset + target_layer_offset)
        })
        .collect();
    let cloned = MovementPath::from_typed(typed_waypoints, speed);
    if path.attempt_run {
        cloned.with_run_attempt()
    } else {
        cloned
    }
}

fn source_clone_minion_waypoint_paths(
    leader_ref_id: u32,
    leader_base_position: Vec2,
    leader_layer_position: Vec2,
    leader_path: &MovementPath,
    object_snapshots: &[MouseCommandObjectSnapshot],
    layer_snapshots: &[MouseCommandLayerSnapshot],
) -> Vec<SourceClonedMinionPath> {
    let leader_layer_offset = leader_layer_position - leader_base_position;
    let mut cloned = Vec::new();
    for minion in object_snapshots.iter().filter(|snapshot| {
        snapshot.group_leader_ref_id == Some(leader_ref_id)
            && snapshot.ref_id != leader_ref_id
            && !snapshot.stats.destroyed()
            && snapshot.mobile
    }) {
        for (entity, ref_id, layer_position, _, _) in layer_snapshots {
            if *ref_id != minion.ref_id {
                continue;
            }
            let minion_layer_offset = *layer_position - minion.position;
            cloned.push(SourceClonedMinionPath {
                entity: *entity,
                ref_id: minion.ref_id,
                path: movement_path_retargeted_to_layer(
                    leader_path,
                    leader_layer_offset,
                    minion_layer_offset,
                    minion.stats.move_speed,
                ),
            });
        }
    }
    cloned
}

fn shift_down(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
}

fn shift_send_released(keys: &ButtonInput<KeyCode>) -> bool {
    (keys.just_released(KeyCode::ShiftLeft) || keys.just_released(KeyCode::ShiftRight))
        && !shift_down(keys)
}

pub(crate) fn handle_mouse_commands(
    mut commands: Commands,
    mut input: MouseCommandInput,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    passability: Res<PassabilityGrid>,
    hud_layout: Res<HudLayout>,
    cannon_placement: Res<CannonPlacementState>,
    production_window: Res<ProductionWindowState>,
    mut selection: ResMut<SelectionState>,
    mut mouse_state: ResMut<MouseCommandState>,
    mut pending_move_commands: ResMut<PendingMouseMoveCommands>,
    mut visual_feedback: MouseCommandVisualFeedback,
    mut portrait_feedback: MouseCommandPortraitFeedback,
    mut object_queries: ParamSet<(
        Query<(
            &GameObjectEntity,
            &Transform,
            &Selectable,
            &ObjectTeam,
            &ObjectStats,
            Option<&RobotGroup>,
            Option<&BridgeFootprint>,
        )>,
        Query<(
            Entity,
            &Transform,
            &ObjectLayerRef,
            Option<&GameObjectEntity>,
            Option<&AttackTarget>,
            Option<&MovementPath>,
            Option<&CraneRepairTarget>,
            Option<&UnitRepairTarget>,
            Option<&EnterFortTarget>,
        )>,
        Query<(
            &GameObjectEntity,
            &Transform,
            &ObjectTeam,
            &ObjectStats,
            Option<&GrenadeInventory>,
        )>,
    )>,
    attack_selectable_query: Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&MapGridPosition>,
    )>,
    ui_queries: MouseCommandUiQueries,
) {
    if cannon_placement.pending.is_some()
        || production_window.open.is_some()
        || production_window.input_captured
    {
        return;
    }
    let activation_access = source_command_activation_access(&input);

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(screen_pos) = window.cursor_position() else {
        return;
    };
    if account_menu_contains_cursor(&input.login_prompt, &input.account_menu, window, screen_pos) {
        return;
    }
    let window_size = Vec2::new(window.width(), window.height());
    let in_hud =
        screen_pos.x >= window_size.x - HUD_WIDTH || screen_pos.y >= window_size.y - HUD_HEIGHT;
    let world_pos = if in_hud {
        let Some(minimap_world) = hud_layout.minimap_screen_to_world(screen_pos, window_size)
        else {
            return;
        };
        if !input.mouse.just_released(MouseButton::Right) {
            return;
        }
        minimap_world
    } else {
        let Some(world_pos) = cursor_world_position(&windows, &camera_query) else {
            return;
        };
        world_pos
    };

    if !world_pos.is_finite() {
        return;
    }

    if input.mouse.just_pressed(MouseButton::Left) {
        mouse_state.left_start = Some(world_pos);
    }

    if input.mouse.pressed(MouseButton::Left) {
        if let Some(start) = mouse_state.left_start {
            for box_entity in &ui_queries.drag_boxes {
                commands.entity(box_entity).despawn();
            }

            let delta = world_pos - start;
            if delta.length_squared() > 4.0 {
                spawn_drag_selection_box(&mut commands, start, world_pos, TeamType::Red);
            }
        }
    }

    if input.mouse.just_released(MouseButton::Left) {
        let Some(start) = mouse_state.left_start.take() else {
            return;
        };

        for box_entity in &ui_queries.drag_boxes {
            commands.entity(box_entity).despawn();
        }

        for (marker, _) in &ui_queries.selection_markers {
            commands.entity(marker).despawn();
        }
        for (bar, _) in &ui_queries.selection_health_bars {
            commands.entity(bar).despawn();
        }

        selection.selected_refs.clear();

        let selected = if (world_pos - start).abs().max_element() <= 1.0 {
            nearest_selectable(world_pos, &object_queries.p0())
                .into_iter()
                .collect()
        } else {
            selectables_in_rect(start, world_pos, &object_queries.p0())
        };

        for (ref_id, position, selection_size, team, stats) in selected {
            selection.selected_refs.push(ref_id);
            spawn_selection_marker(&mut commands, ref_id, position, selection_size, team, stats);
        }
    }

    if shift_send_released(&input.keys) && !pending_move_commands.pending.is_empty() {
        let mut pending = std::mem::take(&mut pending_move_commands.pending);
        let grenade_amounts: Vec<(u32, u8)> = object_queries
            .p2()
            .iter()
            .filter_map(|(object, _, _, _, inventory)| {
                inventory.map(|inventory| (object.ref_id, inventory.amount))
            })
            .collect();
        let object_snapshots: Vec<MouseCommandObjectSnapshot> = object_queries
            .p0()
            .iter()
            .map(|(object, transform, selectable, team, stats, group, _)| {
                MouseCommandObjectSnapshot {
                    ref_id: object.ref_id,
                    kind: object.kind,
                    position: transform.translation.truncate(),
                    selection_size: selectable.selection_size,
                    team: team.0,
                    stats: *stats,
                    grenade_amount: grenade_amounts
                        .iter()
                        .find_map(|(ref_id, amount)| (*ref_id == object.ref_id).then_some(*amount)),
                    mobile: selectable.mobile,
                    group_leader_ref_id: group.map(|group| group.leader_ref_id),
                }
            })
            .collect();
        let object_kinds = object_snapshots
            .iter()
            .map(|snapshot| (snapshot.ref_id, snapshot.kind))
            .collect::<HashMap<_, _>>();
        source_filter_pending_activation(
            &mut pending,
            &object_kinds,
            activation_access,
            &mut input.news_log,
        );
        let layer_snapshots: Vec<MouseCommandLayerSnapshot> = object_queries
            .p1()
            .iter()
            .map(
                |(
                    entity,
                    transform,
                    layer_ref,
                    _,
                    _,
                    movement_path,
                    crane_repair,
                    unit_repair,
                    enter_fort,
                )| {
                    (
                        entity,
                        layer_ref.0,
                        transform.translation.truncate(),
                        movement_path.cloned(),
                        source_current_waypoint_stage(crane_repair, unit_repair, enter_fort),
                    )
                },
            )
            .collect();

        apply_pending_mouse_move_commands(
            &mut commands,
            &passability,
            &mut portrait_feedback,
            &mut visual_feedback.previous_cursor,
            &mut visual_feedback.waypoint_feedback,
            pending,
            &object_snapshots,
            &layer_snapshots,
            &ui_queries.target_markers,
        );
    }

    if input.mouse.just_released(MouseButton::Right) {
        let shift_pressed = shift_down(&input.keys);
        let mut selected_refs = selection.selected_refs.clone();
        if !shift_pressed {
            let object_kinds = object_queries
                .p0()
                .iter()
                .map(|(object, ..)| (object.ref_id, object.kind))
                .collect::<HashMap<_, _>>();
            selected_refs = source_filter_activation_refs(
                &selected_refs,
                &object_kinds,
                activation_access,
                &mut input.news_log,
            );
        }
        if selected_refs.is_empty() {
            return;
        }
        let source_selected_refs = selected_refs.clone();
        let acknowledge_ref_id = source_selected_refs.first().copied();
        if shift_pressed {
            let expanded_refs =
                expand_selected_refs_for_orders(&selected_refs, &object_queries.p0());
            if let Some(repair) = unit_repair_target_for_right_click(
                world_pos,
                &expanded_refs,
                TeamType::Red,
                &object_queries.p0(),
            ) {
                pending_move_commands
                    .pending
                    .push(PendingMouseCommand::UnitRepair(
                        PendingMouseUnitRepairCommand {
                            world_pos,
                            selected_refs,
                            unit_refs: repair.units.iter().map(|unit| unit.ref_id).collect(),
                            target: repair.target,
                        },
                    ));

                for marker in &ui_queries.target_markers {
                    commands.entity(marker).despawn();
                }
                commands.spawn((
                    Sprite::from_color(Color::srgba(0.2, 0.9, 1.0, 0.35), Vec2::splat(10.0)),
                    Transform::from_xyz(
                        repair.target.entrance_point.x,
                        repair.target.entrance_point.y,
                        31.0,
                    ),
                    TargetMarker,
                    Name::new("unit_repair_target_marker"),
                ));
                return;
            }

            if let Some(repair) = crane_repair_target_for_right_click(
                world_pos,
                &expanded_refs,
                TeamType::Red,
                &object_queries.p0(),
            ) {
                pending_move_commands
                    .pending
                    .push(PendingMouseCommand::CraneRepair(
                        PendingMouseCraneRepairCommand {
                            world_pos,
                            selected_refs,
                            crane_refs: repair.cranes.iter().map(|crane| crane.ref_id).collect(),
                            target: repair.target,
                        },
                    ));

                for marker in &ui_queries.target_markers {
                    commands.entity(marker).despawn();
                }
                commands.spawn((
                    Sprite::from_color(Color::srgba(0.2, 0.9, 1.0, 0.35), Vec2::splat(10.0)),
                    Transform::from_xyz(
                        repair.target.entrance_point.x,
                        repair.target.entrance_point.y,
                        31.0,
                    ),
                    TargetMarker,
                    Name::new("crane_repair_target_marker"),
                ));
                return;
            }

            if let Some(pickup) =
                pickup_target_for_right_click(world_pos, &expanded_refs, &object_queries.p2())
            {
                pending_move_commands
                    .pending
                    .push(PendingMouseCommand::PickupGrenades(
                        PendingMousePickupGrenadesCommand {
                            world_pos,
                            selected_refs,
                            robot_refs: pickup.robots.iter().map(|robot| robot.ref_id).collect(),
                            target_ref_id: pickup.target.ref_id,
                            target_position: pickup.target.position,
                        },
                    ));

                for marker in &ui_queries.target_markers {
                    commands.entity(marker).despawn();
                }
                commands.spawn((
                    Sprite::from_color(Color::srgba(0.2, 0.9, 1.0, 0.35), Vec2::splat(10.0)),
                    Transform::from_xyz(pickup.target.position.x, pickup.target.position.y, 31.0),
                    TargetMarker,
                    Name::new("pickup_grenades_target_marker"),
                ));
                return;
            }

            if let Some(enter) =
                enter_target_for_right_click(world_pos, &expanded_refs, &object_queries.p0())
            {
                pending_move_commands
                    .pending
                    .push(PendingMouseCommand::Enter(PendingMouseEnterCommand {
                        world_pos,
                        selected_refs,
                        robot_refs: enter.robots.iter().map(|robot| robot.ref_id).collect(),
                        target: enter.target,
                    }));

                for marker in &ui_queries.target_markers {
                    commands.entity(marker).despawn();
                }
                commands.spawn((
                    Sprite::from_color(Color::srgba(0.2, 0.9, 1.0, 0.35), Vec2::splat(10.0)),
                    Transform::from_xyz(enter.target.waypoint.x, enter.target.waypoint.y, 31.0),
                    TargetMarker,
                    Name::new("enter_target_marker"),
                ));
                return;
            }

            if let Some(enter_fort) = enter_fort_target_for_right_click(
                world_pos,
                &expanded_refs,
                TeamType::Red,
                &object_queries.p0(),
            ) {
                pending_move_commands
                    .pending
                    .push(PendingMouseCommand::EnterFort(
                        PendingMouseEnterFortCommand {
                            world_pos,
                            selected_refs,
                            robot_refs: enter_fort
                                .robots
                                .iter()
                                .map(|robot| robot.ref_id)
                                .collect(),
                            target: enter_fort.target,
                        },
                    ));

                for marker in &ui_queries.target_markers {
                    commands.entity(marker).despawn();
                }
                commands.spawn((
                    Sprite::from_color(Color::srgba(0.2, 0.9, 1.0, 0.35), Vec2::splat(10.0)),
                    Transform::from_xyz(
                        enter_fort.target.exit_point.x,
                        enter_fort.target.exit_point.y,
                        31.0,
                    ),
                    TargetMarker,
                    Name::new("enter_fort_target_marker"),
                ));
                return;
            }

            let attack_target =
                attack_target_at_position(world_pos, &expanded_refs, &attack_selectable_query);
            if let Some(target_ref_id) = attack_target {
                pending_move_commands
                    .pending
                    .push(PendingMouseCommand::Attack(PendingMouseAttackCommand {
                        world_pos,
                        selected_refs,
                        target_ref_id,
                    }));

                for marker in &ui_queries.target_markers {
                    commands.entity(marker).despawn();
                }
                commands.spawn((
                    Sprite::from_color(Color::srgba(0.95, 0.2, 0.2, 0.35), Vec2::splat(10.0)),
                    Transform::from_xyz(world_pos.x, world_pos.y, 31.0),
                    TargetMarker,
                    Name::new("attack_target_marker"),
                ));
                return;
            }

            let plain_move_command = !expanded_refs.is_empty()
                && unit_repair_target_for_right_click(
                    world_pos,
                    &expanded_refs,
                    TeamType::Red,
                    &object_queries.p0(),
                )
                .is_none()
                && crane_repair_target_for_right_click(
                    world_pos,
                    &expanded_refs,
                    TeamType::Red,
                    &object_queries.p0(),
                )
                .is_none()
                && enter_target_for_right_click(world_pos, &expanded_refs, &object_queries.p0())
                    .is_none()
                && enter_fort_target_for_right_click(
                    world_pos,
                    &expanded_refs,
                    TeamType::Red,
                    &object_queries.p0(),
                )
                .is_none();

            if plain_move_command {
                pending_move_commands
                    .pending
                    .push(PendingMouseCommand::Move(PendingMouseMoveCommand {
                        world_pos,
                        selected_refs,
                        ctrl_down: input.keys.pressed(KeyCode::ControlLeft)
                            || input.keys.pressed(KeyCode::ControlRight),
                        alt_down: input.keys.pressed(KeyCode::AltLeft)
                            || input.keys.pressed(KeyCode::AltRight),
                    }));

                for marker in &ui_queries.target_markers {
                    commands.entity(marker).despawn();
                }
                commands.spawn((
                    Sprite::from_color(Color::srgba(0.2, 0.9, 1.0, 0.35), Vec2::splat(10.0)),
                    Transform::from_xyz(world_pos.x, world_pos.y, 31.0),
                    TargetMarker,
                    Name::new("move_target_marker"),
                ));
                return;
            }
        }

        if !shift_pressed {
            let eject_ref = eject_target_for_right_click(
                world_pos,
                &selected_refs,
                TeamType::Red,
                &object_queries.p0(),
            );
            if let Some(eject_ref) = eject_ref {
                relay_eject_vehicle_command(&mut input.eject_vehicle_packets, eject_ref);
                return;
            }
        }

        let one_unit_command_ref = if input.keys.pressed(KeyCode::KeyZ) {
            let candidates: Vec<(u32, Vec2)> = object_queries
                .p0()
                .iter()
                .filter(|(object, _, _, _, stats, _, _)| {
                    selected_refs.contains(&object.ref_id) && !stats.destroyed()
                })
                .map(|(object, transform, _, _, _, _, _)| {
                    (object.ref_id, transform.translation.truncate())
                })
                .collect();
            one_unit_command_ref(world_pos, candidates)
        } else {
            None
        };
        let selected_refs = if let Some(ref_id) = one_unit_command_ref {
            vec![ref_id]
        } else {
            expand_selected_refs_for_orders(&selected_refs, &object_queries.p0())
        };
        if selected_refs.is_empty() {
            return;
        }
        let source_grenade_amounts: Vec<(u32, u8)> = object_queries
            .p2()
            .iter()
            .filter_map(|(object, _, _, _, inventory)| {
                inventory.map(|inventory| (object.ref_id, inventory.amount))
            })
            .collect();
        let source_object_snapshots: Vec<MouseCommandObjectSnapshot> = object_queries
            .p0()
            .iter()
            .map(|(object, transform, selectable, team, stats, group, _)| {
                MouseCommandObjectSnapshot {
                    ref_id: object.ref_id,
                    kind: object.kind,
                    position: transform.translation.truncate(),
                    selection_size: selectable.selection_size,
                    team: team.0,
                    stats: *stats,
                    grenade_amount: source_grenade_amounts
                        .iter()
                        .find_map(|(ref_id, amount)| (*ref_id == object.ref_id).then_some(*amount)),
                    mobile: selectable.mobile,
                    group_leader_ref_id: group.map(|group| group.leader_ref_id),
                }
            })
            .collect();
        if let Some(repair) = unit_repair_target_for_right_click(
            world_pos,
            &selected_refs,
            TeamType::Red,
            &object_queries.p0(),
        ) {
            portrait_feedback.start_acknowledge(acknowledge_ref_id, false);
            let unit_refs: Vec<u32> = repair.units.iter().map(|unit| unit.ref_id).collect();
            let layer_snapshots: Vec<MouseCommandLayerSnapshot> = object_queries
                .p1()
                .iter()
                .map(
                    |(
                        entity,
                        transform,
                        layer_ref,
                        _,
                        _,
                        movement_path,
                        crane_repair,
                        unit_repair,
                        enter_fort,
                    )| {
                        (
                            entity,
                            layer_ref.0,
                            transform.translation.truncate(),
                            movement_path.cloned(),
                            source_current_waypoint_stage(crane_repair, unit_repair, enter_fort),
                        )
                    },
                )
                .collect();

            for (entity, ref_id, _, _, current_stage) in &layer_snapshots {
                if unit_refs.contains(ref_id)
                    && !source_should_queue_behind_current_stage(*current_stage)
                {
                    commands
                        .entity(*entity)
                        .remove::<AttackTargetLifecycleComponents>()
                        .remove::<PickupGrenadesTarget>()
                        .remove::<EnterTarget>()
                        .remove::<EnterFortTarget>()
                        .remove::<CraneRepairTarget>()
                        .remove::<UnitRepairTarget>();
                }
            }

            let mut issued_any = false;
            let mut feedback_paths = HashMap::new();
            for unit in repair.units {
                let Some(route) = passability.route_with_footprint(
                    unit.position,
                    repair.target.entrance_point,
                    RouteFootprint::Vehicle,
                ) else {
                    continue;
                };

                for (entity, ref_id, layer_position, existing_path, current_stage) in
                    &layer_snapshots
                {
                    if *ref_id == unit.ref_id {
                        let layer_offset = *layer_position - unit.position;
                        let Some((path, repair_ref_id)) =
                            source_relay_special_waypoint_path_with_existing(
                                unit.ref_id,
                                movement_path_for_route(
                                    &route,
                                    layer_offset,
                                    unit.move_speed,
                                    false,
                                ),
                                repair.target.ref_id,
                                repair.target.entrance_point,
                                SourceWaypointMode::UnitRepair,
                                TeamType::Red,
                                &source_object_snapshots,
                                existing_path.as_ref(),
                                *current_stage,
                            )
                        else {
                            continue;
                        };
                        let final_waypoint = source_waypoint_at(
                            repair.target.entrance_point,
                            SourceWaypointMode::UnitRepair,
                            i32::try_from(repair_ref_id).unwrap_or(-1),
                        );
                        feedback_paths.insert(
                            unit.ref_id,
                            waypoint_feedback_path_from_movement(
                                unit.position,
                                &path,
                                layer_offset,
                                Some(final_waypoint),
                            ),
                        );
                        let minion_paths = source_clone_minion_waypoint_paths(
                            unit.ref_id,
                            unit.position,
                            *layer_position,
                            &path,
                            &source_object_snapshots,
                            &layer_snapshots,
                        );
                        commands
                            .entity(*entity)
                            .remove::<AcceptedEmptyWaypointCommand>()
                            .remove::<SourceLocationInterpolation>()
                            .insert(path.clone())
                            .remove::<JustLeftCannon>()
                            .insert(UnitRepairTarget {
                                ref_id: repair_ref_id,
                                stage: UnitRepairStage::GotoEntrance,
                                center_point: repair.target.center_point,
                                entrance_point: repair.target.entrance_point,
                                resume_waypoints: Vec::new(),
                            });
                        for cloned in minion_paths {
                            commands
                                .entity(cloned.entity)
                                .remove::<AcceptedEmptyWaypointCommand>()
                                .remove::<AttackTargetLifecycleComponents>()
                                .remove::<PickupGrenadesTarget>()
                                .remove::<EnterTarget>()
                                .remove::<EnterFortTarget>()
                                .remove::<CraneRepairTarget>()
                                .remove::<UnitRepairTarget>()
                                .remove::<JustLeftCannon>()
                                .remove::<SourceLocationInterpolation>()
                                .insert(cloned.path)
                                .insert(UnitRepairTarget {
                                    ref_id: repair_ref_id,
                                    stage: UnitRepairStage::GotoEntrance,
                                    center_point: repair.target.center_point,
                                    entrance_point: repair.target.entrance_point,
                                    resume_waypoints: Vec::new(),
                                });
                        }
                        issued_any = true;
                    }
                }
            }
            if issued_any {
                visual_feedback.previous_cursor.show(
                    previous_cursor_kind_for_source_waypoint(SourceWaypointMode::UnitRepair),
                    repair.target.entrance_point,
                );
                visual_feedback
                    .waypoint_feedback
                    .show_object_paths(feedback_paths);
            }

            remove_one_unit_command_selection(
                &mut commands,
                &mut selection,
                one_unit_command_ref,
                &ui_queries.selection_markers,
                &ui_queries.selection_health_bars,
            );
            return;
        }

        if let Some(repair) = crane_repair_target_for_right_click(
            world_pos,
            &selected_refs,
            TeamType::Red,
            &object_queries.p0(),
        ) {
            portrait_feedback.start_acknowledge(acknowledge_ref_id, false);
            let crane_refs: Vec<u32> = repair.cranes.iter().map(|crane| crane.ref_id).collect();
            let layer_snapshots: Vec<MouseCommandLayerSnapshot> = object_queries
                .p1()
                .iter()
                .map(
                    |(
                        entity,
                        transform,
                        layer_ref,
                        _,
                        _,
                        movement_path,
                        crane_repair,
                        unit_repair,
                        enter_fort,
                    )| {
                        (
                            entity,
                            layer_ref.0,
                            transform.translation.truncate(),
                            movement_path.cloned(),
                            source_current_waypoint_stage(crane_repair, unit_repair, enter_fort),
                        )
                    },
                )
                .collect();

            for (entity, ref_id, _, _, current_stage) in &layer_snapshots {
                if crane_refs.contains(ref_id)
                    && !source_should_queue_behind_current_stage(*current_stage)
                {
                    commands
                        .entity(*entity)
                        .remove::<AttackTargetLifecycleComponents>()
                        .remove::<PickupGrenadesTarget>()
                        .remove::<EnterTarget>()
                        .remove::<EnterFortTarget>()
                        .remove::<CraneRepairTarget>()
                        .remove::<UnitRepairTarget>();
                }
            }

            let mut issued_any = false;
            let mut feedback_paths = HashMap::new();
            for crane in repair.cranes {
                let Some(route) = passability.route_with_footprint(
                    crane.position,
                    repair.target.entrance_point,
                    RouteFootprint::Vehicle,
                ) else {
                    continue;
                };

                for (entity, ref_id, layer_position, existing_path, current_stage) in
                    &layer_snapshots
                {
                    if *ref_id == crane.ref_id {
                        let layer_offset = *layer_position - crane.position;
                        let Some((path, repair_ref_id)) =
                            source_relay_special_waypoint_path_with_existing(
                                crane.ref_id,
                                movement_path_for_route(
                                    &route,
                                    layer_offset,
                                    crane.move_speed,
                                    false,
                                ),
                                repair.target.ref_id,
                                repair.target.center_point,
                                SourceWaypointMode::CraneRepair,
                                TeamType::Red,
                                &source_object_snapshots,
                                existing_path.as_ref(),
                                *current_stage,
                            )
                        else {
                            continue;
                        };
                        let final_waypoint = source_waypoint_at(
                            repair.target.center_point,
                            SourceWaypointMode::CraneRepair,
                            i32::try_from(repair_ref_id).unwrap_or(-1),
                        );
                        feedback_paths.insert(
                            crane.ref_id,
                            waypoint_feedback_path_from_movement(
                                crane.position,
                                &path,
                                layer_offset,
                                Some(final_waypoint),
                            ),
                        );
                        let minion_paths = source_clone_minion_waypoint_paths(
                            crane.ref_id,
                            crane.position,
                            *layer_position,
                            &path,
                            &source_object_snapshots,
                            &layer_snapshots,
                        );
                        commands
                            .entity(*entity)
                            .remove::<AcceptedEmptyWaypointCommand>()
                            .remove::<SourceLocationInterpolation>()
                            .insert(path.clone())
                            .remove::<JustLeftCannon>()
                            .insert(CraneRepairTarget {
                                ref_id: repair_ref_id,
                                stage: CraneRepairStage::GotoEntrance,
                                center_point: repair.target.center_point,
                                exit_point: repair.target.entrance_point,
                            });
                        for cloned in minion_paths {
                            commands
                                .entity(cloned.entity)
                                .remove::<AcceptedEmptyWaypointCommand>()
                                .remove::<AttackTargetLifecycleComponents>()
                                .remove::<PickupGrenadesTarget>()
                                .remove::<EnterTarget>()
                                .remove::<EnterFortTarget>()
                                .remove::<CraneRepairTarget>()
                                .remove::<UnitRepairTarget>()
                                .remove::<JustLeftCannon>()
                                .remove::<SourceLocationInterpolation>()
                                .insert(cloned.path)
                                .insert(CraneRepairTarget {
                                    ref_id: repair_ref_id,
                                    stage: CraneRepairStage::GotoEntrance,
                                    center_point: repair.target.center_point,
                                    exit_point: repair.target.entrance_point,
                                });
                        }
                        issued_any = true;
                    }
                }
            }
            if issued_any {
                visual_feedback.previous_cursor.show(
                    previous_cursor_kind_for_source_waypoint(SourceWaypointMode::CraneRepair),
                    repair.target.center_point,
                );
                visual_feedback
                    .waypoint_feedback
                    .show_object_paths(feedback_paths);
            }

            remove_one_unit_command_selection(
                &mut commands,
                &mut selection,
                one_unit_command_ref,
                &ui_queries.selection_markers,
                &ui_queries.selection_health_bars,
            );
            return;
        }

        if let Some(pickup) =
            pickup_target_for_right_click(world_pos, &selected_refs, &object_queries.p2())
        {
            portrait_feedback.start_acknowledge(acknowledge_ref_id, false);
            let robot_refs: Vec<u32> = pickup.robots.iter().map(|robot| robot.ref_id).collect();
            let layer_snapshots: Vec<MouseCommandLayerSnapshot> = object_queries
                .p1()
                .iter()
                .map(
                    |(
                        entity,
                        transform,
                        layer_ref,
                        _,
                        _,
                        movement_path,
                        crane_repair,
                        unit_repair,
                        enter_fort,
                    )| {
                        (
                            entity,
                            layer_ref.0,
                            transform.translation.truncate(),
                            movement_path.cloned(),
                            source_current_waypoint_stage(crane_repair, unit_repair, enter_fort),
                        )
                    },
                )
                .collect();

            for (entity, ref_id, _, _, current_stage) in &layer_snapshots {
                if robot_refs.contains(ref_id)
                    && !source_should_queue_behind_current_stage(*current_stage)
                {
                    commands
                        .entity(*entity)
                        .remove::<AttackTargetLifecycleComponents>()
                        .remove::<PickupGrenadesTarget>()
                        .remove::<EnterTarget>()
                        .remove::<EnterFortTarget>()
                        .remove::<CraneRepairTarget>()
                        .remove::<UnitRepairTarget>();
                }
            }

            let mut issued_any = false;
            let mut feedback_paths = HashMap::new();
            for robot in pickup.robots {
                let Some(route) = passability.route(robot.position, pickup.target.position) else {
                    continue;
                };

                for (entity, ref_id, layer_position, existing_path, current_stage) in
                    &layer_snapshots
                {
                    if *ref_id == robot.ref_id {
                        let layer_offset = *layer_position - robot.position;
                        let Some((path, pickup_ref_id)) =
                            source_relay_special_waypoint_path_with_existing(
                                robot.ref_id,
                                movement_path_for_route(
                                    &route,
                                    layer_offset,
                                    robot.move_speed,
                                    true,
                                ),
                                pickup.target.ref_id,
                                world_pos,
                                SourceWaypointMode::PickupGrenades,
                                TeamType::Red,
                                &source_object_snapshots,
                                existing_path.as_ref(),
                                *current_stage,
                            )
                        else {
                            continue;
                        };
                        let final_waypoint = source_waypoint_at(
                            world_pos,
                            SourceWaypointMode::PickupGrenades,
                            i32::try_from(pickup_ref_id).unwrap_or(-1),
                        );
                        feedback_paths.insert(
                            robot.ref_id,
                            waypoint_feedback_path_from_movement(
                                robot.position,
                                &path,
                                layer_offset,
                                Some(final_waypoint),
                            ),
                        );
                        let minion_paths = source_clone_minion_waypoint_paths(
                            robot.ref_id,
                            robot.position,
                            *layer_position,
                            &path,
                            &source_object_snapshots,
                            &layer_snapshots,
                        );
                        commands
                            .entity(*entity)
                            .remove::<AcceptedEmptyWaypointCommand>()
                            .remove::<SourceLocationInterpolation>()
                            .insert(path.clone())
                            .remove::<JustLeftCannon>()
                            .insert(PickupGrenadesTarget {
                                ref_id: pickup_ref_id,
                            });
                        for cloned in minion_paths {
                            commands
                                .entity(cloned.entity)
                                .remove::<AcceptedEmptyWaypointCommand>()
                                .remove::<AttackTargetLifecycleComponents>()
                                .remove::<PickupGrenadesTarget>()
                                .remove::<EnterTarget>()
                                .remove::<EnterFortTarget>()
                                .remove::<CraneRepairTarget>()
                                .remove::<UnitRepairTarget>()
                                .remove::<JustLeftCannon>()
                                .remove::<SourceLocationInterpolation>()
                                .insert(cloned.path)
                                .insert(PickupGrenadesTarget {
                                    ref_id: pickup_ref_id,
                                });
                        }
                        issued_any = true;
                    }
                }
            }
            if issued_any {
                visual_feedback.previous_cursor.show(
                    previous_cursor_kind_for_source_waypoint(SourceWaypointMode::PickupGrenades),
                    world_pos,
                );
                visual_feedback
                    .waypoint_feedback
                    .show_object_paths(feedback_paths);
            }

            remove_one_unit_command_selection(
                &mut commands,
                &mut selection,
                one_unit_command_ref,
                &ui_queries.selection_markers,
                &ui_queries.selection_health_bars,
            );
            return;
        }

        if let Some(enter) =
            enter_target_for_right_click(world_pos, &selected_refs, &object_queries.p0())
        {
            portrait_feedback.start_acknowledge(acknowledge_ref_id, false);
            let robot_refs: Vec<u32> = enter.robots.iter().map(|robot| robot.ref_id).collect();
            let layer_snapshots: Vec<MouseCommandLayerSnapshot> = object_queries
                .p1()
                .iter()
                .map(
                    |(
                        entity,
                        transform,
                        layer_ref,
                        _,
                        _,
                        movement_path,
                        crane_repair,
                        unit_repair,
                        enter_fort,
                    )| {
                        (
                            entity,
                            layer_ref.0,
                            transform.translation.truncate(),
                            movement_path.cloned(),
                            source_current_waypoint_stage(crane_repair, unit_repair, enter_fort),
                        )
                    },
                )
                .collect();

            for (entity, ref_id, _, _, current_stage) in &layer_snapshots {
                if robot_refs.contains(ref_id)
                    && !source_should_queue_behind_current_stage(*current_stage)
                {
                    commands
                        .entity(*entity)
                        .remove::<AttackTargetLifecycleComponents>()
                        .remove::<PickupGrenadesTarget>()
                        .remove::<EnterTarget>()
                        .remove::<EnterFortTarget>()
                        .remove::<CraneRepairTarget>()
                        .remove::<UnitRepairTarget>();
                }
            }

            let mut issued_any = false;
            let mut feedback_paths = HashMap::new();
            for robot in enter.robots {
                let Some(route) = passability.route(robot.position, enter.target.waypoint) else {
                    continue;
                };

                for (entity, ref_id, layer_position, existing_path, current_stage) in
                    &layer_snapshots
                {
                    if *ref_id == robot.ref_id {
                        let layer_offset = *layer_position - robot.position;
                        let Some((path, enter_ref_id)) =
                            source_relay_special_waypoint_path_with_existing(
                                robot.ref_id,
                                movement_path_for_route(
                                    &route,
                                    layer_offset,
                                    robot.move_speed,
                                    true,
                                ),
                                enter.target.ref_id,
                                world_pos,
                                SourceWaypointMode::Enter,
                                TeamType::Red,
                                &source_object_snapshots,
                                existing_path.as_ref(),
                                *current_stage,
                            )
                        else {
                            continue;
                        };
                        let final_waypoint = source_waypoint_at(
                            world_pos,
                            SourceWaypointMode::Enter,
                            i32::try_from(enter_ref_id).unwrap_or(-1),
                        );
                        feedback_paths.insert(
                            robot.ref_id,
                            waypoint_feedback_path_from_movement(
                                robot.position,
                                &path,
                                layer_offset,
                                Some(final_waypoint),
                            ),
                        );
                        let minion_paths = source_clone_minion_waypoint_paths(
                            robot.ref_id,
                            robot.position,
                            *layer_position,
                            &path,
                            &source_object_snapshots,
                            &layer_snapshots,
                        );
                        commands
                            .entity(*entity)
                            .remove::<AcceptedEmptyWaypointCommand>()
                            .remove::<SourceLocationInterpolation>()
                            .insert(path.clone())
                            .remove::<JustLeftCannon>()
                            .insert(EnterTarget {
                                ref_id: enter_ref_id,
                            });
                        for cloned in minion_paths {
                            commands
                                .entity(cloned.entity)
                                .remove::<AcceptedEmptyWaypointCommand>()
                                .remove::<AttackTargetLifecycleComponents>()
                                .remove::<PickupGrenadesTarget>()
                                .remove::<EnterTarget>()
                                .remove::<EnterFortTarget>()
                                .remove::<CraneRepairTarget>()
                                .remove::<UnitRepairTarget>()
                                .remove::<JustLeftCannon>()
                                .remove::<SourceLocationInterpolation>()
                                .insert(cloned.path)
                                .insert(EnterTarget {
                                    ref_id: enter_ref_id,
                                });
                        }
                        issued_any = true;
                    }
                }
            }
            if issued_any {
                visual_feedback.previous_cursor.show(
                    previous_cursor_kind_for_source_waypoint(SourceWaypointMode::Enter),
                    world_pos,
                );
                visual_feedback
                    .waypoint_feedback
                    .show_object_paths(feedback_paths);
            }

            remove_one_unit_command_selection(
                &mut commands,
                &mut selection,
                one_unit_command_ref,
                &ui_queries.selection_markers,
                &ui_queries.selection_health_bars,
            );
            return;
        }

        if let Some(enter_fort) = enter_fort_target_for_right_click(
            world_pos,
            &selected_refs,
            TeamType::Red,
            &object_queries.p0(),
        ) {
            portrait_feedback.start_acknowledge(acknowledge_ref_id, false);
            let robot_refs: Vec<u32> = enter_fort.robots.iter().map(|robot| robot.ref_id).collect();
            let layer_snapshots: Vec<MouseCommandLayerSnapshot> = object_queries
                .p1()
                .iter()
                .map(
                    |(
                        entity,
                        transform,
                        layer_ref,
                        _,
                        _,
                        movement_path,
                        crane_repair,
                        unit_repair,
                        enter_fort,
                    )| {
                        (
                            entity,
                            layer_ref.0,
                            transform.translation.truncate(),
                            movement_path.cloned(),
                            source_current_waypoint_stage(crane_repair, unit_repair, enter_fort),
                        )
                    },
                )
                .collect();

            for (entity, ref_id, _, _, current_stage) in &layer_snapshots {
                if robot_refs.contains(ref_id)
                    && !source_should_queue_behind_current_stage(*current_stage)
                {
                    commands
                        .entity(*entity)
                        .remove::<AttackTargetLifecycleComponents>()
                        .remove::<PickupGrenadesTarget>()
                        .remove::<EnterTarget>()
                        .remove::<EnterFortTarget>()
                        .remove::<CraneRepairTarget>()
                        .remove::<UnitRepairTarget>();
                }
            }

            let mut issued_any = false;
            let mut feedback_paths = HashMap::new();
            for robot in enter_fort.robots {
                let Some(route) = passability.route_for_object_kind(
                    robot.position,
                    enter_fort.target.exit_point,
                    robot.kind,
                ) else {
                    continue;
                };

                for (entity, ref_id, layer_position, existing_path, current_stage) in
                    &layer_snapshots
                {
                    if *ref_id == robot.ref_id {
                        let layer_offset = *layer_position - robot.position;
                        let Some((path, enter_fort_ref_id)) =
                            source_relay_special_waypoint_path_with_existing(
                                robot.ref_id,
                                movement_path_for_route(
                                    &route,
                                    layer_offset,
                                    robot.move_speed,
                                    false,
                                ),
                                enter_fort.target.ref_id,
                                world_pos,
                                SourceWaypointMode::EnterFort,
                                TeamType::Red,
                                &source_object_snapshots,
                                existing_path.as_ref(),
                                *current_stage,
                            )
                        else {
                            continue;
                        };
                        let final_waypoint = source_waypoint_at(
                            world_pos,
                            SourceWaypointMode::EnterFort,
                            i32::try_from(enter_fort_ref_id).unwrap_or(-1),
                        );
                        feedback_paths.insert(
                            robot.ref_id,
                            waypoint_feedback_path_from_movement(
                                robot.position,
                                &path,
                                layer_offset,
                                Some(final_waypoint),
                            ),
                        );
                        let minion_paths = source_clone_minion_waypoint_paths(
                            robot.ref_id,
                            robot.position,
                            *layer_position,
                            &path,
                            &source_object_snapshots,
                            &layer_snapshots,
                        );
                        commands
                            .entity(*entity)
                            .remove::<AcceptedEmptyWaypointCommand>()
                            .remove::<SourceLocationInterpolation>()
                            .insert(path.clone())
                            .remove::<JustLeftCannon>()
                            .insert(EnterFortTarget {
                                ref_id: enter_fort_ref_id,
                                stage: EnterFortStage::GotoEntrance,
                                inside_point: enter_fort.target.inside_point,
                                exit_point: enter_fort.target.exit_point,
                            });
                        for cloned in minion_paths {
                            commands
                                .entity(cloned.entity)
                                .remove::<AcceptedEmptyWaypointCommand>()
                                .remove::<AttackTargetLifecycleComponents>()
                                .remove::<PickupGrenadesTarget>()
                                .remove::<EnterTarget>()
                                .remove::<EnterFortTarget>()
                                .remove::<CraneRepairTarget>()
                                .remove::<UnitRepairTarget>()
                                .remove::<JustLeftCannon>()
                                .remove::<SourceLocationInterpolation>()
                                .insert(cloned.path)
                                .insert(EnterFortTarget {
                                    ref_id: enter_fort_ref_id,
                                    stage: EnterFortStage::GotoEntrance,
                                    inside_point: enter_fort.target.inside_point,
                                    exit_point: enter_fort.target.exit_point,
                                });
                        }
                        issued_any = true;
                    }
                }
            }
            if issued_any {
                visual_feedback.previous_cursor.show(
                    previous_cursor_kind_for_source_waypoint(SourceWaypointMode::EnterFort),
                    world_pos,
                );
                visual_feedback
                    .waypoint_feedback
                    .show_object_paths(feedback_paths);
            }

            remove_one_unit_command_selection(
                &mut commands,
                &mut selection,
                one_unit_command_ref,
                &ui_queries.selection_markers,
                &ui_queries.selection_health_bars,
            );
            return;
        }

        let attack_target =
            attack_target_at_position(world_pos, &selected_refs, &attack_selectable_query);
        if let Some(target_ref_id) = attack_target {
            let object_kinds: Vec<(u32, ObjectKind)> = object_queries
                .p0()
                .iter()
                .map(|(object, _, _, _, _, _, _)| (object.ref_id, object.kind))
                .collect();
            let no_way = attack_command_no_way(&source_selected_refs, target_ref_id, object_kinds);
            let target_position = object_queries
                .p0()
                .iter()
                .find(|(object, _, _, _, _, _, _)| object.ref_id == target_ref_id)
                .map(|(_, transform, _, _, _, _, _)| transform.translation.truncate())
                .unwrap_or(world_pos);

            let attackers: Vec<(u32, ObjectKind, Vec2, f32, f32, bool)> = object_queries
                .p0()
                .iter()
                .filter(|(object, _, _, _, stats, _, _)| {
                    selected_refs.contains(&object.ref_id) && stats.can_attack()
                })
                .map(|(object, transform, selectable, _, stats, _, _)| {
                    (
                        object.ref_id,
                        object.kind,
                        transform.translation.truncate(),
                        stats.move_speed,
                        stats.attack_radius,
                        selectable.mobile,
                    )
                })
                .collect();
            let attacker_refs: Vec<u32> = attackers
                .iter()
                .map(|(ref_id, _, _, _, _, _)| *ref_id)
                .collect();
            if !attacker_refs.is_empty() {
                portrait_feedback.start_acknowledge(acknowledge_ref_id, no_way);
            }
            let layer_snapshots: Vec<MouseCommandLayerSnapshot> = object_queries
                .p1()
                .iter()
                .map(
                    |(
                        entity,
                        transform,
                        layer_ref,
                        _,
                        _,
                        movement_path,
                        crane_repair,
                        unit_repair,
                        enter_fort,
                    )| {
                        (
                            entity,
                            layer_ref.0,
                            transform.translation.truncate(),
                            movement_path.cloned(),
                            source_current_waypoint_stage(crane_repair, unit_repair, enter_fort),
                        )
                    },
                )
                .collect();
            let attack_routes: Vec<(u32, Vec2, Vec<Vec2>, f32)> = attackers
                .iter()
                .map(
                    |(ref_id, kind, base_position, move_speed, attack_radius, mobile)| {
                        let route = if *mobile {
                            passability
                                .route_to_attack_range_for_object_kind(
                                    *base_position,
                                    target_position,
                                    *attack_radius,
                                    *kind,
                                )
                                .unwrap_or_default()
                        } else {
                            Vec::new()
                        };
                        (*ref_id, *base_position, route, *move_speed)
                    },
                )
                .collect();

            let mut issued_any = false;
            let mut feedback_paths = HashMap::new();
            for (entity, ref_id, layer_position, existing_path, current_stage) in &layer_snapshots {
                if attacker_refs.contains(ref_id) {
                    if let Some((_, base_position, route, move_speed)) = attack_routes
                        .iter()
                        .find(|(route_ref_id, _, _, _)| route_ref_id == ref_id)
                    {
                        let layer_offset = *layer_position - *base_position;
                        let Some(path) = source_relay_movement_path_with_existing(
                            *ref_id,
                            movement_attack_path_for_route(
                                route,
                                layer_offset,
                                target_ref_id,
                                world_pos,
                                *move_speed,
                            ),
                            TeamType::Red,
                            &source_object_snapshots,
                            existing_path.as_ref(),
                            *current_stage,
                        ) else {
                            continue;
                        };
                        feedback_paths.insert(
                            *ref_id,
                            waypoint_feedback_path_from_movement(
                                *base_position,
                                &path,
                                layer_offset,
                                None,
                            ),
                        );
                        let minion_paths = source_clone_minion_waypoint_paths(
                            *ref_id,
                            *base_position,
                            *layer_position,
                            &path,
                            &source_object_snapshots,
                            &layer_snapshots,
                        );
                        insert_waypoint_command_path(
                            &mut commands,
                            *entity,
                            path.clone(),
                            *current_stage,
                            existing_path.as_ref(),
                        );
                        for cloned in minion_paths {
                            insert_waypoint_command_path(
                                &mut commands,
                                cloned.entity,
                                cloned.path,
                                None,
                                None,
                            );
                        }
                        issued_any = true;
                    }
                }
            }
            if issued_any {
                visual_feedback.previous_cursor.show(
                    previous_cursor_kind_for_source_waypoint(SourceWaypointMode::Attack),
                    target_position,
                );
                visual_feedback
                    .waypoint_feedback
                    .show_object_paths(feedback_paths);
            }

            if !attacker_refs.is_empty() {
                remove_one_unit_command_selection(
                    &mut commands,
                    &mut selection,
                    one_unit_command_ref,
                    &ui_queries.selection_markers,
                    &ui_queries.selection_health_bars,
                );
            }
            return;
        }

        let move_targets: Vec<PassiveCombatTargetSnapshot> = object_queries
            .p0()
            .iter()
            .map(
                |(object, transform, selectable, team, stats, _, _)| PassiveCombatTargetSnapshot {
                    ref_id: object.ref_id,
                    kind: object.kind,
                    position: transform.translation.truncate(),
                    size: selectable.selection_size,
                    team: team.0,
                    stats: *stats,
                },
            )
            .collect();
        let ctrl_down =
            input.keys.pressed(KeyCode::ControlLeft) || input.keys.pressed(KeyCode::ControlRight);
        let alt_down =
            input.keys.pressed(KeyCode::AltLeft) || input.keys.pressed(KeyCode::AltRight);
        let mobile_bases: Vec<(u32, ObjectKind, Vec2, f32, bool)> = object_queries
            .p0()
            .iter()
            .filter(|(object, _, selectable, _, _, _, _)| {
                selectable.mobile && selected_refs.contains(&object.ref_id)
            })
            .map(|(object, transform, _, team, stats, _, _)| {
                let position = transform.translation.truncate();
                let attack_to = player_move_waypoint_attack_to(
                    object.ref_id,
                    position,
                    team.0,
                    *stats,
                    &move_targets,
                    ctrl_down,
                    alt_down,
                );
                (
                    object.ref_id,
                    object.kind,
                    position,
                    stats.move_speed,
                    attack_to,
                )
            })
            .collect();

        if mobile_bases.is_empty() {
            return;
        }
        portrait_feedback.start_acknowledge(acknowledge_ref_id, false);

        for marker in &ui_queries.target_markers {
            commands.entity(marker).despawn();
        }

        commands.spawn((
            Sprite::from_color(Color::srgba(0.2, 0.9, 1.0, 0.35), Vec2::splat(10.0)),
            Transform::from_xyz(world_pos.x, world_pos.y, 31.0),
            TargetMarker,
            Name::new("move_target_marker"),
        ));

        let attempt_run = move_target_is_near_flag(world_pos, &object_queries.p0());
        let layer_snapshots: Vec<MouseCommandLayerSnapshot> = object_queries
            .p1()
            .iter()
            .map(
                |(
                    entity,
                    transform,
                    layer_ref,
                    _,
                    _,
                    movement_path,
                    crane_repair,
                    unit_repair,
                    enter_fort,
                )| {
                    (
                        entity,
                        layer_ref.0,
                        transform.translation.truncate(),
                        movement_path.cloned(),
                        source_current_waypoint_stage(crane_repair, unit_repair, enter_fort),
                    )
                },
            )
            .collect();
        let mut issued_any = false;
        let mut feedback_paths = HashMap::new();
        for (ref_id, kind, base_position, move_speed, attack_to) in mobile_bases {
            let Some(route) = passability.route_for_object_kind(base_position, world_pos, kind)
            else {
                continue;
            };

            for (entity, layer_ref_id, layer_position, existing_path, current_stage) in
                &layer_snapshots
            {
                if *layer_ref_id == ref_id {
                    let layer_offset = *layer_position - base_position;
                    let Some(path) = source_relay_movement_path_with_existing(
                        ref_id,
                        movement_player_move_path_for_route(
                            &route,
                            layer_offset,
                            move_speed,
                            attempt_run,
                            attack_to,
                        ),
                        TeamType::Red,
                        &source_object_snapshots,
                        existing_path.as_ref(),
                        *current_stage,
                    ) else {
                        continue;
                    };
                    feedback_paths.insert(
                        ref_id,
                        waypoint_feedback_path_from_movement(
                            base_position,
                            &path,
                            layer_offset,
                            None,
                        ),
                    );
                    let minion_paths = source_clone_minion_waypoint_paths(
                        ref_id,
                        base_position,
                        *layer_position,
                        &path,
                        &source_object_snapshots,
                        &layer_snapshots,
                    );
                    insert_waypoint_command_path(
                        &mut commands,
                        *entity,
                        path.clone(),
                        *current_stage,
                        existing_path.as_ref(),
                    );
                    for cloned in minion_paths {
                        insert_waypoint_command_path(
                            &mut commands,
                            cloned.entity,
                            cloned.path,
                            None,
                            None,
                        );
                    }
                    issued_any = true;
                }
            }
        }
        if issued_any {
            visual_feedback.previous_cursor.show(
                previous_cursor_kind_for_source_waypoint(SourceWaypointMode::Move),
                world_pos,
            );
            visual_feedback
                .waypoint_feedback
                .show_object_paths(feedback_paths);
        }

        remove_one_unit_command_selection(
            &mut commands,
            &mut selection,
            one_unit_command_ref,
            &ui_queries.selection_markers,
            &ui_queries.selection_health_bars,
        );
    }
}

fn one_unit_command_ref(
    target: Vec2,
    candidates: impl IntoIterator<Item = (u32, Vec2)>,
) -> Option<u32> {
    candidates
        .into_iter()
        .filter(|(_, position)| position.is_finite())
        .min_by(|(_, left), (_, right)| {
            left.distance_squared(target)
                .total_cmp(&right.distance_squared(target))
        })
        .map(|(ref_id, _)| ref_id)
}

fn remove_one_unit_command_selection(
    commands: &mut Commands,
    selection: &mut SelectionState,
    one_unit_ref: Option<u32>,
    selection_markers: &Query<(Entity, &SelectionMarker)>,
    selection_health_bars: &Query<(Entity, &SelectionHealthBar)>,
) {
    let Some(ref_id) = one_unit_ref else {
        return;
    };

    selection
        .selected_refs
        .retain(|selected_ref| *selected_ref != ref_id);
    for (entity, marker) in selection_markers {
        if marker.ref_id == ref_id {
            commands.entity(entity).despawn();
        }
    }
    for (entity, bar) in selection_health_bars {
        if bar.ref_id == ref_id {
            commands.entity(entity).despawn();
        }
    }
}

fn expand_selected_refs_for_orders_from_snapshots(
    selected_refs: &[u32],
    snapshots: &[MouseCommandObjectSnapshot],
) -> Vec<u32> {
    let mut refs = Vec::new();
    for snapshot in snapshots {
        if snapshot.stats.destroyed() {
            continue;
        }

        let selected = snapshot
            .group_leader_ref_id
            .is_some_and(|leader_ref_id| selected_refs.contains(&leader_ref_id))
            || selected_refs.contains(&snapshot.ref_id);

        if selected && !refs.contains(&snapshot.ref_id) {
            refs.push(snapshot.ref_id);
        }
    }
    refs
}

fn move_target_is_near_flag_from_snapshots(
    world_pos: Vec2,
    snapshots: &[MouseCommandObjectSnapshot],
) -> bool {
    snapshots.iter().any(|snapshot| {
        !snapshot.stats.destroyed()
            && matches!(snapshot.kind, ObjectKind::MapItem(id) if id == ItemType::Flag as u8)
            && snapshot.position.distance(world_pos) <= 32.0
    })
}

fn append_pending_movement_path(
    pending_paths: &mut HashMap<Entity, MovementPath>,
    entity: Entity,
    ref_id: u32,
    segment: MovementPath,
    ok_team: TeamType,
    object_snapshots: &[MouseCommandObjectSnapshot],
    existing_path: Option<&MovementPath>,
    current_stage: Option<SourceCurrentWaypointStage>,
) -> Option<MovementPath> {
    let entry = pending_paths
        .entry(entity)
        .or_insert_with(|| MovementPath::from_typed(Vec::new(), segment.speed));
    entry.waypoints.extend(segment.waypoints);
    entry.typed_waypoints.extend(segment.typed_waypoints);
    entry.attempt_run |= segment.attempt_run;
    let relayed = source_relay_movement_path_with_existing(
        ref_id,
        entry.clone(),
        ok_team,
        object_snapshots,
        existing_path,
        current_stage,
    )?;
    *entry = relayed.clone();
    Some(relayed)
}

fn pending_command_selected_refs(command: &PendingMouseCommand) -> &[u32] {
    match command {
        PendingMouseCommand::Move(command) => &command.selected_refs,
        PendingMouseCommand::Attack(command) => &command.selected_refs,
        PendingMouseCommand::PickupGrenades(command) => &command.selected_refs,
        PendingMouseCommand::UnitRepair(command) => &command.selected_refs,
        PendingMouseCommand::CraneRepair(command) => &command.selected_refs,
        PendingMouseCommand::Enter(command) => &command.selected_refs,
        PendingMouseCommand::EnterFort(command) => &command.selected_refs,
    }
}

fn pending_attack_no_way(
    command: &PendingMouseCommand,
    object_snapshots: &[MouseCommandObjectSnapshot],
) -> bool {
    let PendingMouseCommand::Attack(command) = command else {
        return false;
    };

    attack_command_no_way(
        &command.selected_refs,
        command.target_ref_id,
        object_snapshots
            .iter()
            .map(|snapshot| (snapshot.ref_id, snapshot.kind)),
    )
}

fn pending_repair_resume_waypoints(
    pending: &[PendingMouseCommand],
    unit: MouseCommandObjectSnapshot,
    start_position: Vec2,
    object_snapshots: &[MouseCommandObjectSnapshot],
) -> Vec<MovementWaypoint> {
    let move_targets: Vec<_> = object_snapshots
        .iter()
        .map(|snapshot| PassiveCombatTargetSnapshot {
            ref_id: snapshot.ref_id,
            kind: snapshot.kind,
            position: snapshot.position,
            size: snapshot.selection_size,
            team: snapshot.team,
            stats: snapshot.stats,
        })
        .collect();
    let mut position = start_position;
    let mut waypoints = Vec::new();

    for command in pending {
        match command {
            PendingMouseCommand::Move(command) if command.selected_refs.contains(&unit.ref_id) => {
                let attack_to = player_move_waypoint_attack_to(
                    unit.ref_id,
                    position,
                    unit.team,
                    unit.stats,
                    &move_targets,
                    command.ctrl_down,
                    command.alt_down,
                );
                waypoints.push(MovementWaypoint::player_move_to(
                    command.world_pos,
                    attack_to,
                ));
                position = command.world_pos;
            }
            PendingMouseCommand::Attack(command)
                if command.selected_refs.contains(&unit.ref_id) =>
            {
                if object_snapshots.iter().any(|snapshot| {
                    snapshot.ref_id == command.target_ref_id && !snapshot.stats.destroyed()
                }) {
                    waypoints.push(MovementWaypoint::player_attack_target(
                        command.target_ref_id,
                        command.world_pos,
                    ));
                    position = command.world_pos;
                }
            }
            PendingMouseCommand::PickupGrenades(command)
                if command.selected_refs.contains(&unit.ref_id) =>
            {
                break;
            }
            PendingMouseCommand::UnitRepair(command)
                if command.selected_refs.contains(&unit.ref_id) =>
            {
                break;
            }
            PendingMouseCommand::CraneRepair(command)
                if command.selected_refs.contains(&unit.ref_id) =>
            {
                break;
            }
            PendingMouseCommand::Enter(command) if command.selected_refs.contains(&unit.ref_id) => {
                break;
            }
            PendingMouseCommand::EnterFort(command)
                if command.selected_refs.contains(&unit.ref_id) =>
            {
                break;
            }
            _ => {}
        }
    }

    waypoints
}

fn apply_pending_mouse_move_commands(
    commands: &mut Commands,
    passability: &PassabilityGrid,
    portrait_feedback: &mut MouseCommandPortraitFeedback,
    previous_cursor: &mut PreviousCursorState,
    waypoint_feedback: &mut WaypointFeedbackState,
    pending: Vec<PendingMouseCommand>,
    object_snapshots: &[MouseCommandObjectSnapshot],
    layer_snapshots: &[MouseCommandLayerSnapshot],
    target_markers: &Query<Entity, With<TargetMarker>>,
) {
    let acknowledge_ref_id = pending
        .first()
        .and_then(|command| pending_command_selected_refs(command).first())
        .copied();
    let no_way = pending
        .first()
        .is_some_and(|command| pending_attack_no_way(command, object_snapshots));
    let mut pending_paths = HashMap::new();
    let mut planned_positions: HashMap<u32, Vec2> = HashMap::new();
    let mut issued_any = false;
    let mut final_target = None;
    let mut final_cursor = None;
    let mut feedback_paths: HashMap<u32, WaypointFeedbackPath> = HashMap::new();
    let mut repair_queued_refs = HashSet::new();

    for (command_index, command) in pending.iter().cloned().enumerate() {
        match command {
            PendingMouseCommand::Move(command) => {
                let selected_refs = expand_selected_refs_for_orders_from_snapshots(
                    &command.selected_refs,
                    object_snapshots,
                );
                if selected_refs.is_empty() {
                    continue;
                }

                let move_targets: Vec<PassiveCombatTargetSnapshot> = object_snapshots
                    .iter()
                    .map(|snapshot| PassiveCombatTargetSnapshot {
                        ref_id: snapshot.ref_id,
                        kind: snapshot.kind,
                        position: snapshot.position,
                        size: snapshot.selection_size,
                        team: snapshot.team,
                        stats: snapshot.stats,
                    })
                    .collect();
                let attempt_run =
                    move_target_is_near_flag_from_snapshots(command.world_pos, object_snapshots);

                for snapshot in object_snapshots.iter().filter(|snapshot| {
                    snapshot.mobile
                        && selected_refs.contains(&snapshot.ref_id)
                        && !snapshot.stats.destroyed()
                }) {
                    let route_start = planned_positions
                        .get(&snapshot.ref_id)
                        .copied()
                        .unwrap_or(snapshot.position);
                    if repair_queued_refs.contains(&snapshot.ref_id) {
                        planned_positions.insert(snapshot.ref_id, command.world_pos);
                        issued_any = true;
                        final_target = Some(command.world_pos);
                        final_cursor = Some((
                            previous_cursor_kind_for_source_waypoint(SourceWaypointMode::Move),
                            command.world_pos,
                        ));
                        continue;
                    }
                    let attack_to = player_move_waypoint_attack_to(
                        snapshot.ref_id,
                        route_start,
                        snapshot.team,
                        snapshot.stats,
                        &move_targets,
                        command.ctrl_down,
                        command.alt_down,
                    );
                    let Some(route) = passability.route_for_object_kind(
                        route_start,
                        command.world_pos,
                        snapshot.kind,
                    ) else {
                        continue;
                    };

                    for (entity, ref_id, layer_position, existing_path, current_stage) in
                        layer_snapshots
                    {
                        if *ref_id == snapshot.ref_id {
                            let layer_offset = *layer_position - snapshot.position;
                            let segment = movement_player_move_path_for_route(
                                &route,
                                layer_offset,
                                snapshot.stats.move_speed,
                                attempt_run,
                                attack_to,
                            );
                            let Some(path) = append_pending_movement_path(
                                &mut pending_paths,
                                *entity,
                                snapshot.ref_id,
                                segment,
                                TeamType::Red,
                                object_snapshots,
                                existing_path.as_ref(),
                                *current_stage,
                            ) else {
                                continue;
                            };
                            feedback_paths.insert(
                                snapshot.ref_id,
                                waypoint_feedback_path_from_movement(
                                    snapshot.position,
                                    &path,
                                    layer_offset,
                                    None,
                                ),
                            );
                            let minion_paths = source_clone_minion_waypoint_paths(
                                snapshot.ref_id,
                                snapshot.position,
                                *layer_position,
                                &path,
                                object_snapshots,
                                layer_snapshots,
                            );
                            insert_waypoint_command_path(
                                commands,
                                *entity,
                                path.clone(),
                                *current_stage,
                                existing_path.as_ref(),
                            );
                            for cloned in minion_paths {
                                insert_waypoint_command_path(
                                    commands,
                                    cloned.entity,
                                    cloned.path,
                                    None,
                                    None,
                                );
                            }
                        }
                    }

                    planned_positions.insert(snapshot.ref_id, command.world_pos);
                    issued_any = true;
                    final_target = Some(command.world_pos);
                    final_cursor = Some((
                        previous_cursor_kind_for_source_waypoint(SourceWaypointMode::Move),
                        command.world_pos,
                    ));
                }
            }
            PendingMouseCommand::Attack(command) => {
                let selected_refs = expand_selected_refs_for_orders_from_snapshots(
                    &command.selected_refs,
                    object_snapshots,
                );
                if selected_refs.is_empty() {
                    continue;
                }
                let Some(target) = object_snapshots
                    .iter()
                    .find(|snapshot| {
                        snapshot.ref_id == command.target_ref_id && !snapshot.stats.destroyed()
                    })
                    .copied()
                else {
                    continue;
                };

                for snapshot in object_snapshots.iter().filter(|snapshot| {
                    selected_refs.contains(&snapshot.ref_id)
                        && snapshot.stats.can_attack()
                        && !snapshot.stats.destroyed()
                }) {
                    let route_start = planned_positions
                        .get(&snapshot.ref_id)
                        .copied()
                        .unwrap_or(snapshot.position);
                    if repair_queued_refs.contains(&snapshot.ref_id) {
                        planned_positions.insert(snapshot.ref_id, command.world_pos);
                        issued_any = true;
                        final_target = Some(command.world_pos);
                        final_cursor = Some((
                            previous_cursor_kind_for_source_waypoint(SourceWaypointMode::Attack),
                            target.position,
                        ));
                        continue;
                    }
                    let route = if snapshot.mobile {
                        passability
                            .route_to_attack_range_for_object_kind(
                                route_start,
                                target.position,
                                snapshot.stats.attack_radius,
                                snapshot.kind,
                            )
                            .unwrap_or_default()
                    } else {
                        Vec::new()
                    };

                    for (entity, ref_id, layer_position, existing_path, current_stage) in
                        layer_snapshots
                    {
                        if *ref_id == snapshot.ref_id {
                            let layer_offset = *layer_position - snapshot.position;
                            let segment = movement_attack_path_for_route(
                                &route,
                                layer_offset,
                                command.target_ref_id,
                                command.world_pos,
                                snapshot.stats.move_speed,
                            );
                            let Some(path) = append_pending_movement_path(
                                &mut pending_paths,
                                *entity,
                                snapshot.ref_id,
                                segment,
                                TeamType::Red,
                                object_snapshots,
                                existing_path.as_ref(),
                                *current_stage,
                            ) else {
                                continue;
                            };
                            feedback_paths.insert(
                                snapshot.ref_id,
                                waypoint_feedback_path_from_movement(
                                    snapshot.position,
                                    &path,
                                    layer_offset,
                                    None,
                                ),
                            );
                            let minion_paths = source_clone_minion_waypoint_paths(
                                snapshot.ref_id,
                                snapshot.position,
                                *layer_position,
                                &path,
                                object_snapshots,
                                layer_snapshots,
                            );
                            insert_waypoint_command_path(
                                commands,
                                *entity,
                                path.clone(),
                                *current_stage,
                                existing_path.as_ref(),
                            );
                            for cloned in minion_paths {
                                insert_waypoint_command_path(
                                    commands,
                                    cloned.entity,
                                    cloned.path,
                                    None,
                                    None,
                                );
                            }
                        }
                    }

                    planned_positions.insert(
                        snapshot.ref_id,
                        route.last().copied().unwrap_or(route_start),
                    );
                    issued_any = true;
                    final_target = Some(command.world_pos);
                    final_cursor = Some((
                        previous_cursor_kind_for_source_waypoint(SourceWaypointMode::Attack),
                        target.position,
                    ));
                }
            }
            PendingMouseCommand::PickupGrenades(command) => {
                let target_valid = object_snapshots.iter().any(|snapshot| {
                    snapshot.ref_id == command.target_ref_id
                        && !snapshot.stats.destroyed()
                        && matches!(snapshot.kind, ObjectKind::MapItem(id) if id == ItemType::Grenades as u8)
                });
                if !target_valid {
                    continue;
                }

                for snapshot in object_snapshots.iter().filter(|snapshot| {
                    command.robot_refs.contains(&snapshot.ref_id)
                        && snapshot.mobile
                        && !snapshot.stats.destroyed()
                }) {
                    let route_start = planned_positions
                        .get(&snapshot.ref_id)
                        .copied()
                        .unwrap_or(snapshot.position);
                    let Some(route) = passability.route(route_start, command.target_position)
                    else {
                        continue;
                    };

                    for (entity, ref_id, layer_position, existing_path, current_stage) in
                        layer_snapshots
                    {
                        if *ref_id == snapshot.ref_id {
                            let layer_offset = *layer_position - snapshot.position;
                            let segment = movement_path_for_route(
                                &route,
                                layer_offset,
                                snapshot.stats.move_speed,
                                true,
                            );
                            let Some(path) = append_pending_movement_path(
                                &mut pending_paths,
                                *entity,
                                snapshot.ref_id,
                                segment,
                                TeamType::Red,
                                object_snapshots,
                                existing_path.as_ref(),
                                *current_stage,
                            ) else {
                                continue;
                            };
                            let Some((path, pickup_ref_id)) =
                                source_relay_special_waypoint_path_with_existing(
                                    snapshot.ref_id,
                                    path,
                                    command.target_ref_id,
                                    command.world_pos,
                                    SourceWaypointMode::PickupGrenades,
                                    TeamType::Red,
                                    object_snapshots,
                                    existing_path.as_ref(),
                                    *current_stage,
                                )
                            else {
                                continue;
                            };
                            let final_waypoint = source_waypoint_at(
                                command.world_pos,
                                SourceWaypointMode::PickupGrenades,
                                i32::try_from(pickup_ref_id).unwrap_or(-1),
                            );
                            feedback_paths.insert(
                                snapshot.ref_id,
                                waypoint_feedback_path_from_movement(
                                    snapshot.position,
                                    &path,
                                    layer_offset,
                                    Some(final_waypoint),
                                ),
                            );
                            let minion_paths = source_clone_minion_waypoint_paths(
                                snapshot.ref_id,
                                snapshot.position,
                                *layer_position,
                                &path,
                                object_snapshots,
                                layer_snapshots,
                            );
                            commands
                                .entity(*entity)
                                .remove::<AttackTargetLifecycleComponents>()
                                .remove::<PickupGrenadesTarget>()
                                .remove::<EnterTarget>()
                                .remove::<EnterFortTarget>()
                                .remove::<CraneRepairTarget>()
                                .remove::<UnitRepairTarget>()
                                .remove::<JustLeftCannon>()
                                .remove::<SourceLocationInterpolation>()
                                .insert(path.clone())
                                .insert(PickupGrenadesTarget {
                                    ref_id: pickup_ref_id,
                                });
                            for cloned in minion_paths {
                                commands
                                    .entity(cloned.entity)
                                    .remove::<AttackTargetLifecycleComponents>()
                                    .remove::<PickupGrenadesTarget>()
                                    .remove::<EnterTarget>()
                                    .remove::<EnterFortTarget>()
                                    .remove::<CraneRepairTarget>()
                                    .remove::<UnitRepairTarget>()
                                    .remove::<JustLeftCannon>()
                                    .remove::<SourceLocationInterpolation>()
                                    .insert(cloned.path)
                                    .insert(PickupGrenadesTarget {
                                        ref_id: pickup_ref_id,
                                    });
                            }
                        }
                    }

                    planned_positions.insert(snapshot.ref_id, command.target_position);
                    issued_any = true;
                    final_target = Some(command.world_pos);
                    final_cursor = Some((
                        previous_cursor_kind_for_source_waypoint(
                            SourceWaypointMode::PickupGrenades,
                        ),
                        command.world_pos,
                    ));
                }
            }
            PendingMouseCommand::UnitRepair(command) => {
                let target_valid = object_snapshots.iter().any(|snapshot| {
                    snapshot.ref_id == command.target.ref_id && !snapshot.stats.destroyed()
                });
                if !target_valid {
                    continue;
                }

                for snapshot in object_snapshots.iter().filter(|snapshot| {
                    command.unit_refs.contains(&snapshot.ref_id)
                        && snapshot.mobile
                        && !snapshot.stats.destroyed()
                }) {
                    let route_start = planned_positions
                        .get(&snapshot.ref_id)
                        .copied()
                        .unwrap_or(snapshot.position);
                    let Some(route) = passability.route_with_footprint(
                        route_start,
                        command.target.entrance_point,
                        RouteFootprint::Vehicle,
                    ) else {
                        continue;
                    };
                    let resume_waypoints = pending_repair_resume_waypoints(
                        &pending[command_index + 1..],
                        *snapshot,
                        command.target.entrance_point,
                        object_snapshots,
                    );

                    for (entity, ref_id, layer_position, existing_path, current_stage) in
                        layer_snapshots
                    {
                        if *ref_id == snapshot.ref_id {
                            let layer_offset = *layer_position - snapshot.position;
                            let segment = movement_path_for_route(
                                &route,
                                layer_offset,
                                snapshot.stats.move_speed,
                                false,
                            );
                            let Some(path) = append_pending_movement_path(
                                &mut pending_paths,
                                *entity,
                                snapshot.ref_id,
                                segment,
                                TeamType::Red,
                                object_snapshots,
                                existing_path.as_ref(),
                                *current_stage,
                            ) else {
                                continue;
                            };
                            let Some((path, repair_ref_id)) =
                                source_relay_special_waypoint_path_with_existing(
                                    snapshot.ref_id,
                                    path,
                                    command.target.ref_id,
                                    command.target.entrance_point,
                                    SourceWaypointMode::UnitRepair,
                                    TeamType::Red,
                                    object_snapshots,
                                    existing_path.as_ref(),
                                    *current_stage,
                                )
                            else {
                                continue;
                            };
                            let final_waypoint = source_waypoint_at(
                                command.target.entrance_point,
                                SourceWaypointMode::UnitRepair,
                                i32::try_from(repair_ref_id).unwrap_or(-1),
                            );
                            feedback_paths.insert(
                                snapshot.ref_id,
                                waypoint_feedback_path_from_movement(
                                    snapshot.position,
                                    &path,
                                    layer_offset,
                                    Some(final_waypoint),
                                ),
                            );
                            let minion_paths = source_clone_minion_waypoint_paths(
                                snapshot.ref_id,
                                snapshot.position,
                                *layer_position,
                                &path,
                                object_snapshots,
                                layer_snapshots,
                            );
                            commands
                                .entity(*entity)
                                .remove::<AttackTargetLifecycleComponents>()
                                .remove::<PickupGrenadesTarget>()
                                .remove::<EnterTarget>()
                                .remove::<EnterFortTarget>()
                                .remove::<CraneRepairTarget>()
                                .remove::<UnitRepairTarget>()
                                .remove::<JustLeftCannon>()
                                .remove::<SourceLocationInterpolation>()
                                .insert(path.clone())
                                .insert(UnitRepairTarget {
                                    ref_id: repair_ref_id,
                                    stage: UnitRepairStage::GotoEntrance,
                                    center_point: command.target.center_point,
                                    entrance_point: command.target.entrance_point,
                                    resume_waypoints: resume_waypoints.clone(),
                                });
                            repair_queued_refs.insert(snapshot.ref_id);
                            for cloned in minion_paths {
                                repair_queued_refs.insert(cloned.ref_id);
                                commands
                                    .entity(cloned.entity)
                                    .remove::<AttackTargetLifecycleComponents>()
                                    .remove::<PickupGrenadesTarget>()
                                    .remove::<EnterTarget>()
                                    .remove::<EnterFortTarget>()
                                    .remove::<CraneRepairTarget>()
                                    .remove::<UnitRepairTarget>()
                                    .remove::<JustLeftCannon>()
                                    .remove::<SourceLocationInterpolation>()
                                    .insert(cloned.path)
                                    .insert(UnitRepairTarget {
                                        ref_id: repair_ref_id,
                                        stage: UnitRepairStage::GotoEntrance,
                                        center_point: command.target.center_point,
                                        entrance_point: command.target.entrance_point,
                                        resume_waypoints: resume_waypoints.clone(),
                                    });
                            }
                        }
                    }

                    planned_positions.insert(snapshot.ref_id, command.target.entrance_point);
                    issued_any = true;
                    final_target = Some(command.world_pos);
                    final_cursor = Some((
                        previous_cursor_kind_for_source_waypoint(SourceWaypointMode::UnitRepair),
                        command.target.entrance_point,
                    ));
                }
            }
            PendingMouseCommand::CraneRepair(command) => {
                let target_valid = object_snapshots.iter().any(|snapshot| {
                    snapshot.ref_id == command.target.ref_id && !snapshot.stats.destroyed()
                });
                if !target_valid {
                    continue;
                }

                for snapshot in object_snapshots.iter().filter(|snapshot| {
                    command.crane_refs.contains(&snapshot.ref_id)
                        && snapshot.mobile
                        && !snapshot.stats.destroyed()
                }) {
                    let route_start = planned_positions
                        .get(&snapshot.ref_id)
                        .copied()
                        .unwrap_or(snapshot.position);
                    let Some(route) = passability.route_with_footprint(
                        route_start,
                        command.target.entrance_point,
                        RouteFootprint::Vehicle,
                    ) else {
                        continue;
                    };

                    for (entity, ref_id, layer_position, existing_path, current_stage) in
                        layer_snapshots
                    {
                        if *ref_id == snapshot.ref_id {
                            let layer_offset = *layer_position - snapshot.position;
                            let segment = movement_path_for_route(
                                &route,
                                layer_offset,
                                snapshot.stats.move_speed,
                                false,
                            );
                            let Some(path) = append_pending_movement_path(
                                &mut pending_paths,
                                *entity,
                                snapshot.ref_id,
                                segment,
                                TeamType::Red,
                                object_snapshots,
                                existing_path.as_ref(),
                                *current_stage,
                            ) else {
                                continue;
                            };
                            let Some((path, repair_ref_id)) =
                                source_relay_special_waypoint_path_with_existing(
                                    snapshot.ref_id,
                                    path,
                                    command.target.ref_id,
                                    command.target.center_point,
                                    SourceWaypointMode::CraneRepair,
                                    TeamType::Red,
                                    object_snapshots,
                                    existing_path.as_ref(),
                                    *current_stage,
                                )
                            else {
                                continue;
                            };
                            let final_waypoint = source_waypoint_at(
                                command.target.center_point,
                                SourceWaypointMode::CraneRepair,
                                i32::try_from(repair_ref_id).unwrap_or(-1),
                            );
                            feedback_paths.insert(
                                snapshot.ref_id,
                                waypoint_feedback_path_from_movement(
                                    snapshot.position,
                                    &path,
                                    layer_offset,
                                    Some(final_waypoint),
                                ),
                            );
                            let minion_paths = source_clone_minion_waypoint_paths(
                                snapshot.ref_id,
                                snapshot.position,
                                *layer_position,
                                &path,
                                object_snapshots,
                                layer_snapshots,
                            );
                            commands
                                .entity(*entity)
                                .remove::<AttackTargetLifecycleComponents>()
                                .remove::<PickupGrenadesTarget>()
                                .remove::<EnterTarget>()
                                .remove::<EnterFortTarget>()
                                .remove::<CraneRepairTarget>()
                                .remove::<UnitRepairTarget>()
                                .remove::<JustLeftCannon>()
                                .remove::<SourceLocationInterpolation>()
                                .insert(path.clone())
                                .insert(CraneRepairTarget {
                                    ref_id: repair_ref_id,
                                    stage: CraneRepairStage::GotoEntrance,
                                    center_point: command.target.center_point,
                                    exit_point: command.target.entrance_point,
                                });
                            for cloned in minion_paths {
                                commands
                                    .entity(cloned.entity)
                                    .remove::<AttackTargetLifecycleComponents>()
                                    .remove::<PickupGrenadesTarget>()
                                    .remove::<EnterTarget>()
                                    .remove::<EnterFortTarget>()
                                    .remove::<CraneRepairTarget>()
                                    .remove::<UnitRepairTarget>()
                                    .remove::<JustLeftCannon>()
                                    .remove::<SourceLocationInterpolation>()
                                    .insert(cloned.path)
                                    .insert(CraneRepairTarget {
                                        ref_id: repair_ref_id,
                                        stage: CraneRepairStage::GotoEntrance,
                                        center_point: command.target.center_point,
                                        exit_point: command.target.entrance_point,
                                    });
                            }
                        }
                    }

                    planned_positions.insert(snapshot.ref_id, command.target.entrance_point);
                    issued_any = true;
                    final_target = Some(command.world_pos);
                    final_cursor = Some((
                        previous_cursor_kind_for_source_waypoint(SourceWaypointMode::CraneRepair),
                        command.target.center_point,
                    ));
                }
            }
            PendingMouseCommand::Enter(command) => {
                let target_valid = object_snapshots.iter().any(|snapshot| {
                    snapshot.ref_id == command.target.ref_id
                        && !snapshot.stats.destroyed()
                        && can_be_entered(snapshot.kind, snapshot.team, snapshot.stats)
                });
                if !target_valid {
                    continue;
                }

                for snapshot in object_snapshots.iter().filter(|snapshot| {
                    command.robot_refs.contains(&snapshot.ref_id)
                        && snapshot.mobile
                        && !snapshot.stats.destroyed()
                }) {
                    let route_start = planned_positions
                        .get(&snapshot.ref_id)
                        .copied()
                        .unwrap_or(snapshot.position);
                    let Some(route) = passability.route(route_start, command.target.waypoint)
                    else {
                        continue;
                    };

                    for (entity, ref_id, layer_position, existing_path, current_stage) in
                        layer_snapshots
                    {
                        if *ref_id == snapshot.ref_id {
                            let layer_offset = *layer_position - snapshot.position;
                            let segment = movement_path_for_route(
                                &route,
                                layer_offset,
                                snapshot.stats.move_speed,
                                true,
                            );
                            let Some(path) = append_pending_movement_path(
                                &mut pending_paths,
                                *entity,
                                snapshot.ref_id,
                                segment,
                                TeamType::Red,
                                object_snapshots,
                                existing_path.as_ref(),
                                *current_stage,
                            ) else {
                                continue;
                            };
                            let Some((path, enter_ref_id)) =
                                source_relay_special_waypoint_path_with_existing(
                                    snapshot.ref_id,
                                    path,
                                    command.target.ref_id,
                                    command.world_pos,
                                    SourceWaypointMode::Enter,
                                    TeamType::Red,
                                    object_snapshots,
                                    existing_path.as_ref(),
                                    *current_stage,
                                )
                            else {
                                continue;
                            };
                            let final_waypoint = source_waypoint_at(
                                command.world_pos,
                                SourceWaypointMode::Enter,
                                i32::try_from(enter_ref_id).unwrap_or(-1),
                            );
                            feedback_paths.insert(
                                snapshot.ref_id,
                                waypoint_feedback_path_from_movement(
                                    snapshot.position,
                                    &path,
                                    layer_offset,
                                    Some(final_waypoint),
                                ),
                            );
                            let minion_paths = source_clone_minion_waypoint_paths(
                                snapshot.ref_id,
                                snapshot.position,
                                *layer_position,
                                &path,
                                object_snapshots,
                                layer_snapshots,
                            );
                            commands
                                .entity(*entity)
                                .remove::<AttackTargetLifecycleComponents>()
                                .remove::<PickupGrenadesTarget>()
                                .remove::<EnterTarget>()
                                .remove::<EnterFortTarget>()
                                .remove::<CraneRepairTarget>()
                                .remove::<UnitRepairTarget>()
                                .remove::<JustLeftCannon>()
                                .remove::<SourceLocationInterpolation>()
                                .insert(path.clone())
                                .insert(EnterTarget {
                                    ref_id: enter_ref_id,
                                });
                            for cloned in minion_paths {
                                commands
                                    .entity(cloned.entity)
                                    .remove::<AttackTargetLifecycleComponents>()
                                    .remove::<PickupGrenadesTarget>()
                                    .remove::<EnterTarget>()
                                    .remove::<EnterFortTarget>()
                                    .remove::<CraneRepairTarget>()
                                    .remove::<UnitRepairTarget>()
                                    .remove::<JustLeftCannon>()
                                    .remove::<SourceLocationInterpolation>()
                                    .insert(cloned.path)
                                    .insert(EnterTarget {
                                        ref_id: enter_ref_id,
                                    });
                            }
                        }
                    }

                    planned_positions.insert(snapshot.ref_id, command.target.waypoint);
                    issued_any = true;
                    final_target = Some(command.world_pos);
                    final_cursor = Some((
                        previous_cursor_kind_for_source_waypoint(SourceWaypointMode::Enter),
                        command.world_pos,
                    ));
                }
            }
            PendingMouseCommand::EnterFort(command) => {
                let target_valid = object_snapshots.iter().any(|snapshot| {
                    snapshot.ref_id == command.target.ref_id
                        && !snapshot.stats.destroyed()
                        && can_enter_fort(
                            snapshot.kind,
                            snapshot.team,
                            TeamType::Red,
                            snapshot.stats,
                        )
                });
                if !target_valid {
                    continue;
                }

                for snapshot in object_snapshots.iter().filter(|snapshot| {
                    command.robot_refs.contains(&snapshot.ref_id)
                        && snapshot.mobile
                        && !snapshot.stats.destroyed()
                }) {
                    let route_start = planned_positions
                        .get(&snapshot.ref_id)
                        .copied()
                        .unwrap_or(snapshot.position);
                    let Some(route) = passability.route_for_object_kind(
                        route_start,
                        command.target.exit_point,
                        snapshot.kind,
                    ) else {
                        continue;
                    };

                    for (entity, ref_id, layer_position, existing_path, current_stage) in
                        layer_snapshots
                    {
                        if *ref_id == snapshot.ref_id {
                            let layer_offset = *layer_position - snapshot.position;
                            let segment = movement_path_for_route(
                                &route,
                                layer_offset,
                                snapshot.stats.move_speed,
                                false,
                            );
                            let Some(path) = append_pending_movement_path(
                                &mut pending_paths,
                                *entity,
                                snapshot.ref_id,
                                segment,
                                TeamType::Red,
                                object_snapshots,
                                existing_path.as_ref(),
                                *current_stage,
                            ) else {
                                continue;
                            };
                            let Some((path, enter_fort_ref_id)) =
                                source_relay_special_waypoint_path_with_existing(
                                    snapshot.ref_id,
                                    path,
                                    command.target.ref_id,
                                    command.world_pos,
                                    SourceWaypointMode::EnterFort,
                                    TeamType::Red,
                                    object_snapshots,
                                    existing_path.as_ref(),
                                    *current_stage,
                                )
                            else {
                                continue;
                            };
                            let final_waypoint = source_waypoint_at(
                                command.world_pos,
                                SourceWaypointMode::EnterFort,
                                i32::try_from(enter_fort_ref_id).unwrap_or(-1),
                            );
                            feedback_paths.insert(
                                snapshot.ref_id,
                                waypoint_feedback_path_from_movement(
                                    snapshot.position,
                                    &path,
                                    layer_offset,
                                    Some(final_waypoint),
                                ),
                            );
                            let minion_paths = source_clone_minion_waypoint_paths(
                                snapshot.ref_id,
                                snapshot.position,
                                *layer_position,
                                &path,
                                object_snapshots,
                                layer_snapshots,
                            );
                            commands
                                .entity(*entity)
                                .remove::<AttackTargetLifecycleComponents>()
                                .remove::<PickupGrenadesTarget>()
                                .remove::<EnterTarget>()
                                .remove::<EnterFortTarget>()
                                .remove::<CraneRepairTarget>()
                                .remove::<UnitRepairTarget>()
                                .remove::<JustLeftCannon>()
                                .remove::<SourceLocationInterpolation>()
                                .insert(path.clone())
                                .insert(EnterFortTarget {
                                    ref_id: enter_fort_ref_id,
                                    stage: EnterFortStage::GotoEntrance,
                                    inside_point: command.target.inside_point,
                                    exit_point: command.target.exit_point,
                                });
                            for cloned in minion_paths {
                                commands
                                    .entity(cloned.entity)
                                    .remove::<AttackTargetLifecycleComponents>()
                                    .remove::<PickupGrenadesTarget>()
                                    .remove::<EnterTarget>()
                                    .remove::<EnterFortTarget>()
                                    .remove::<CraneRepairTarget>()
                                    .remove::<UnitRepairTarget>()
                                    .remove::<JustLeftCannon>()
                                    .remove::<SourceLocationInterpolation>()
                                    .insert(cloned.path)
                                    .insert(EnterFortTarget {
                                        ref_id: enter_fort_ref_id,
                                        stage: EnterFortStage::GotoEntrance,
                                        inside_point: command.target.inside_point,
                                        exit_point: command.target.exit_point,
                                    });
                            }
                        }
                    }

                    planned_positions.insert(snapshot.ref_id, command.target.exit_point);
                    issued_any = true;
                    final_target = Some(command.world_pos);
                    final_cursor = Some((
                        previous_cursor_kind_for_source_waypoint(SourceWaypointMode::EnterFort),
                        command.world_pos,
                    ));
                }
            }
        }
    }

    if !issued_any {
        return;
    }

    portrait_feedback.start_acknowledge(acknowledge_ref_id, no_way);
    if let Some((kind, position)) = final_cursor {
        previous_cursor.show(kind, position);
    }
    if !feedback_paths.is_empty() {
        waypoint_feedback.show_object_paths(feedback_paths);
    }
    for marker in target_markers {
        commands.entity(marker).despawn();
    }
    if let Some(final_target) = final_target {
        commands.spawn((
            Sprite::from_color(Color::srgba(0.2, 0.9, 1.0, 0.35), Vec2::splat(10.0)),
            Transform::from_xyz(final_target.x, final_target.y, 31.0),
            TargetMarker,
            Name::new("move_target_marker"),
        ));
    }
}

fn attack_command_no_way(
    source_selected_refs: &[u32],
    target_ref_id: u32,
    object_kinds: impl IntoIterator<Item = (u32, ObjectKind)>,
) -> bool {
    let [selected_ref] = source_selected_refs else {
        return false;
    };
    let mut attacker = None;
    let mut victim = None;
    for (ref_id, kind) in object_kinds {
        if ref_id == *selected_ref {
            attacker = Some(kind);
        }
        if ref_id == target_ref_id {
            victim = Some(kind);
        }
    }
    attacker
        .zip(victim)
        .is_some_and(|(attacker, victim)| units::unit_rating_will_die(attacker, victim))
}

fn spawn_drag_selection_box(commands: &mut Commands, start: Vec2, end: Vec2, team: TeamType) {
    let min = start.min(end);
    let max = start.max(end);
    let color = darkened_team_color(team, 0.2);
    let step = 4.0;
    let dot_size = Vec2::splat(2.0);
    let draw_shift = 0.0;

    let mut x = min.x + (step - draw_shift);
    while x < max.x {
        spawn_drag_dot(commands, Vec2::new(x, min.y), dot_size, color);
        x += step;
    }

    let mut x = min.x + draw_shift;
    while x < max.x {
        spawn_drag_dot(commands, Vec2::new(x, max.y), dot_size, color);
        x += step;
    }

    let mut y = min.y + draw_shift;
    while y < max.y {
        spawn_drag_dot(commands, Vec2::new(min.x, y), dot_size, color);
        y += step;
    }

    let mut y = min.y + (step - draw_shift);
    while y < max.y {
        spawn_drag_dot(commands, Vec2::new(max.x, y), dot_size, color);
        y += step;
    }
}

fn spawn_drag_dot(commands: &mut Commands, position: Vec2, size: Vec2, color: Color) {
    commands.spawn((
        Sprite::from_color(color, size),
        Transform::from_xyz(position.x, position.y, 32.0),
        DragSelectionBox,
        Name::new("drag_selection_box"),
    ));
}

fn darkened_team_color(team: TeamType, amount: f32) -> Color {
    let linear = team.color().to_linear();
    Color::srgb(
        (linear.red * (1.0 - amount)).clamp(0.0, 1.0),
        (linear.green * (1.0 - amount)).clamp(0.0, 1.0),
        (linear.blue * (1.0 - amount)).clamp(0.0, 1.0),
    )
}

pub(crate) fn spawn_selection_marker(
    commands: &mut Commands,
    ref_id: u32,
    position: Vec2,
    selection_size: Vec2,
    team: TeamType,
    stats: ObjectStats,
) {
    let size = selection_size + Vec2::splat(6.0);
    let half = size * 0.5;
    let len = 5.0;
    let color = team.color();

    for (offset, segment_size) in [
        (Vec2::new(-half.x + len * 0.5, half.y), Vec2::new(len, 1.0)),
        (Vec2::new(-half.x, half.y - len * 0.5), Vec2::new(1.0, len)),
        (Vec2::new(half.x - len * 0.5, half.y), Vec2::new(len, 1.0)),
        (Vec2::new(half.x, half.y - len * 0.5), Vec2::new(1.0, len)),
        (Vec2::new(-half.x + len * 0.5, -half.y), Vec2::new(len, 1.0)),
        (Vec2::new(-half.x, -half.y + len * 0.5), Vec2::new(1.0, len)),
        (Vec2::new(half.x - len * 0.5, -half.y), Vec2::new(len, 1.0)),
        (Vec2::new(half.x, -half.y + len * 0.5), Vec2::new(1.0, len)),
    ] {
        commands.spawn((
            Sprite::from_color(color, segment_size),
            Transform::from_xyz(position.x + offset.x, position.y + offset.y, 30.0),
            SelectionMarker { ref_id, offset },
            Name::new("selection_marker"),
        ));
    }

    spawn_selection_health_bar(commands, ref_id, position, stats);
}

fn spawn_selection_health_bar(
    commands: &mut Commands,
    ref_id: u32,
    position: Vec2,
    stats: ObjectStats,
) {
    let max_dist = 36.0;
    let green_dist = (max_dist * stats.health / MAX_UNIT_HEALTH)
        .round()
        .max(if stats.health > 0.0 { 1.0 } else { 0.0 });
    let yellow_dist = (max_dist * stats.max_health / MAX_UNIT_HEALTH)
        .round()
        .max(if stats.max_health > 0.0 { 1.0 } else { 0.0 });
    let background_size = Vec2::new(yellow_dist + 2.0, 4.0);
    let background_offset = Vec2::new(15.0, 6.0);

    for (offset, size, color) in [
        (
            background_offset + Vec2::new(background_size.x * 0.5, -background_size.y * 0.5),
            background_size,
            Color::srgb(0.0, 0.0, 0.0),
        ),
        (
            background_offset + Vec2::new(1.0 + green_dist * 0.5, -2.0),
            Vec2::new(green_dist, 2.0),
            Color::srgb_u8(82, 190, 33),
        ),
        (
            background_offset
                + Vec2::new(1.0 + green_dist + (yellow_dist - green_dist) * 0.5, -2.0),
            Vec2::new((yellow_dist - green_dist).max(0.0), 2.0),
            Color::srgb_u8(247, 203, 107),
        ),
    ] {
        if size.x <= 0.0 || size.y <= 0.0 {
            continue;
        }
        commands.spawn((
            Sprite::from_color(color, size),
            Transform::from_xyz(position.x + offset.x, position.y + offset.y, 31.0),
            SelectionHealthBar { ref_id, offset },
            Name::new("selection_health_bar"),
        ));
    }
}

#[derive(Clone, Copy)]
struct SelectableSnapshot {
    ref_id: u32,
    leader_ref_id: Option<u32>,
    position: Vec2,
    selection_size: Vec2,
    radius: f32,
    team: TeamType,
    stats: ObjectStats,
}

fn normalized_snapshot(
    snapshot: SelectableSnapshot,
    snapshots: &[SelectableSnapshot],
) -> SelectableSnapshot {
    let Some(leader_ref_id) = snapshot.leader_ref_id else {
        return snapshot;
    };

    snapshots
        .iter()
        .find(|candidate| candidate.ref_id == leader_ref_id)
        .copied()
        .unwrap_or(snapshot)
}

fn selection_tuple_for_snapshot(
    snapshot: SelectableSnapshot,
) -> (u32, Vec2, Vec2, TeamType, ObjectStats) {
    (
        snapshot.ref_id,
        snapshot.position,
        snapshot.selection_size,
        snapshot.team,
        snapshot.stats,
    )
}

fn nearest_selectable(
    world_pos: Vec2,
    selectable_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&RobotGroup>,
        Option<&BridgeFootprint>,
    )>,
) -> Option<(u32, Vec2, Vec2, TeamType, ObjectStats)> {
    let snapshots: Vec<SelectableSnapshot> = selectable_query
        .iter()
        .filter_map(|(object, transform, selectable, team, stats, group, _)| {
            if stats.destroyed() {
                return None;
            }

            let position = transform.translation.truncate();
            Some(SelectableSnapshot {
                ref_id: object.ref_id,
                leader_ref_id: group.map(|group| group.leader_ref_id),
                position,
                selection_size: selectable.selection_size,
                radius: selectable.radius,
                team: team.0,
                stats: *stats,
            })
        })
        .collect();

    let hit = snapshots
        .iter()
        .filter_map(|snapshot| {
            let distance = snapshot.position.distance(world_pos);
            (distance <= snapshot.radius).then_some((*snapshot, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(snapshot, _)| snapshot)?;

    Some(selection_tuple_for_snapshot(normalized_snapshot(
        hit, &snapshots,
    )))
}

fn selectables_in_rect(
    start: Vec2,
    end: Vec2,
    selectable_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&RobotGroup>,
        Option<&BridgeFootprint>,
    )>,
) -> Vec<(u32, Vec2, Vec2, TeamType, ObjectStats)> {
    let min = start.min(end);
    let max = start.max(end);

    let snapshots: Vec<SelectableSnapshot> = selectable_query
        .iter()
        .filter_map(|(object, transform, selectable, team, stats, group, _)| {
            if stats.destroyed() {
                return None;
            }

            let position = transform.translation.truncate();
            Some(SelectableSnapshot {
                ref_id: object.ref_id,
                leader_ref_id: group.map(|group| group.leader_ref_id),
                position,
                selection_size: selectable.selection_size,
                radius: selectable.radius,
                team: team.0,
                stats: *stats,
            })
        })
        .collect();

    let mut selected = Vec::new();
    for snapshot in snapshots.iter().filter(|snapshot| {
        snapshot.position.x >= min.x
            && snapshot.position.x <= max.x
            && snapshot.position.y >= min.y
            && snapshot.position.y <= max.y
    }) {
        let normalized = normalized_snapshot(*snapshot, &snapshots);
        if selected
            .iter()
            .any(|(ref_id, _, _, _, _)| *ref_id == normalized.ref_id)
        {
            continue;
        }
        selected.push(selection_tuple_for_snapshot(normalized));
    }

    selected
}

fn attack_target_at_position(
    world_pos: Vec2,
    selected_refs: &[u32],
    selectable_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&MapGridPosition>,
    )>,
) -> Option<u32> {
    let selected_team = selectable_query
        .iter()
        .find(|(object, _, _, _, stats, _)| {
            selected_refs.contains(&object.ref_id) && stats.can_attack()
        })
        .map(|(_, _, _, team, _, _)| team.0)?;

    selectable_query
        .iter()
        .filter_map(
            |(object, transform, selectable, team, stats, grid_position)| {
                if selected_refs.contains(&object.ref_id)
                    || stats.destroyed()
                    || team.0 == selected_team
                    || selected_team == TeamType::Null
                {
                    return None;
                }

                let position = transform.translation.truncate();
                let distance = position.distance(world_pos);
                attack_hit_test(object.kind, world_pos, position, selectable, grid_position)
                    .then_some((object.ref_id, distance))
            },
        )
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(ref_id, _)| ref_id)
}

fn attack_hit_test(
    kind: ObjectKind,
    world_pos: Vec2,
    target_position: Vec2,
    selectable: &Selectable,
    grid_position: Option<&MapGridPosition>,
) -> bool {
    match kind {
        ObjectKind::Building(BuildingType::FortFront | BuildingType::FortBack) => grid_position
            .map(|grid_position| fort_attack_hit_at_world_position(world_pos, grid_position))
            .unwrap_or_else(|| target_position.distance(world_pos) <= selectable.radius),
        _ => target_position.distance(world_pos) <= selectable.radius,
    }
}

fn fort_attack_hit_at_world_position(world_pos: Vec2, grid_position: &MapGridPosition) -> bool {
    let rx = world_pos.x - grid_position.x as f32 * TILE_SIZE;
    let ry = -world_pos.y - grid_position.y as f32 * TILE_SIZE;
    fort_attack_mask_contains(rx, ry)
}

fn fort_attack_mask_contains(rx: f32, ry: f32) -> bool {
    points_within_area_inclusive(rx, ry, 16.0, 16.0, 16.0 * 8.0, 16.0 * 7.0)
        || points_within_area_inclusive(rx, ry, 0.0, 16.0 * 3.0, 16.0 * 10.0, 16.0 * 4.0)
        || points_within_area_inclusive(rx, ry, 16.0, 0.0, 16.0 * 2.0, 16.0)
        || points_within_area_inclusive(rx, ry, 16.0 * 7.0, 0.0, 16.0 * 2.0, 16.0)
        || points_within_area_inclusive(rx, ry, 16.0 * 2.0, 16.0 * 8.0, 16.0, 16.0)
        || points_within_area_inclusive(rx, ry, 16.0 * 7.0, 16.0 * 8.0, 16.0, 16.0)
}

fn points_within_area_inclusive(px: f32, py: f32, ax: f32, ay: f32, aw: f32, ah: f32) -> bool {
    px >= ax && py >= ay && px <= ax + aw && py <= ay + ah
}

pub(crate) fn update_selection_markers(
    mut marker_query: Query<(&SelectionMarker, &mut Transform)>,
    mut health_bar_query: Query<(&SelectionHealthBar, &mut Transform), Without<SelectionMarker>>,
    object_query: Query<
        (&GameObjectEntity, &Transform),
        (
            With<Selectable>,
            Without<SelectionMarker>,
            Without<SelectionHealthBar>,
        ),
    >,
) {
    for (marker, mut marker_transform) in &mut marker_query {
        if let Some((_, object_transform)) = object_query
            .iter()
            .find(|(object, _)| object.ref_id == marker.ref_id)
        {
            marker_transform.translation.x = object_transform.translation.x + marker.offset.x;
            marker_transform.translation.y = object_transform.translation.y + marker.offset.y;
        }
    }

    for (bar, mut bar_transform) in &mut health_bar_query {
        if let Some((_, object_transform)) = object_query
            .iter()
            .find(|(object, _)| object.ref_id == bar.ref_id)
        {
            bar_transform.translation.x = object_transform.translation.x + bar.offset.x;
            bar_transform.translation.y = object_transform.translation.y + bar.offset.y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_activation_classes_match_source_zcore() {
        for robot in [
            crate::original::objects::RobotType::Grunt,
            crate::original::objects::RobotType::Psycho,
            crate::original::objects::RobotType::Sniper,
            crate::original::objects::RobotType::Tough,
        ] {
            assert!(!units::requires_activation(ObjectKind::Robot(robot)));
        }
        for robot in [
            crate::original::objects::RobotType::Pyro,
            crate::original::objects::RobotType::Laser,
        ] {
            assert!(units::requires_activation(ObjectKind::Robot(robot)));
        }
        for vehicle in [VehicleType::Jeep, VehicleType::Light] {
            assert!(!units::requires_activation(ObjectKind::Vehicle(vehicle)));
        }
        for vehicle in [
            VehicleType::Medium,
            VehicleType::Heavy,
            VehicleType::Apc,
            VehicleType::MissileLauncher,
            VehicleType::Crane,
        ] {
            assert!(units::requires_activation(ObjectKind::Vehicle(vehicle)));
        }
    }

    #[test]
    fn command_activation_rejection_preserves_client_then_server_order() {
        let restricted = ObjectKind::Robot(crate::original::objects::RobotType::Pyro);
        let unrestricted = ObjectKind::Vehicle(VehicleType::Jeep);
        let unregistered = SourceCommandActivationAccess {
            registered: false,
            server_activation_passed: false,
        };
        let unactivated = SourceCommandActivationAccess {
            registered: true,
            server_activation_passed: false,
        };
        let allowed = SourceCommandActivationAccess {
            registered: true,
            server_activation_passed: true,
        };

        assert_eq!(
            source_activation_rejection(restricted, unregistered),
            Some("move unit error: registration required, please visit www.nighsoft.com")
        );
        assert_eq!(
            source_activation_rejection(restricted, unactivated),
            Some("move unit error: activation required, please visit www.nighsoft.com")
        );
        assert_eq!(source_activation_rejection(restricted, allowed), None);
        assert_eq!(
            source_activation_rejection(unrestricted, unregistered),
            None
        );
    }

    #[test]
    fn mixed_selection_rejects_only_restricted_unit_and_keeps_other_command_refs() {
        let kinds = HashMap::from([
            (
                1,
                ObjectKind::Robot(crate::original::objects::RobotType::Laser),
            ),
            (2, ObjectKind::Vehicle(VehicleType::Jeep)),
        ]);
        let mut news = NewsLog::default();

        let allowed = source_filter_activation_refs(
            &[1, 2],
            &kinds,
            SourceCommandActivationAccess {
                registered: false,
                server_activation_passed: false,
            },
            &mut news,
        );

        assert_eq!(allowed, vec![2]);
        assert_eq!(
            news.display_entry(0).map(|entry| entry.message),
            Some("move unit error: registration required, please visit www.nighsoft.com")
        );
    }

    fn fort_world_position(grid_position: MapGridPosition, rx: f32, ry: f32) -> Vec2 {
        Vec2::new(
            grid_position.x as f32 * TILE_SIZE + rx,
            -(grid_position.y as f32 * TILE_SIZE + ry),
        )
    }

    fn broad_selectable() -> Selectable {
        Selectable {
            radius: 500.0,
            selection_size: Vec2::splat(TILE_SIZE),
            mobile: false,
        }
    }

    #[test]
    fn previous_cursor_kind_matches_source_waypoint_cursor_modes() {
        assert_eq!(
            previous_cursor_kind_for_source_waypoint(SourceWaypointMode::Move),
            ZCursorKind::Placed
        );
        assert_eq!(
            previous_cursor_kind_for_source_waypoint(SourceWaypointMode::ForceMove),
            ZCursorKind::Placed
        );
        assert_eq!(
            previous_cursor_kind_for_source_waypoint(SourceWaypointMode::EnterFort),
            ZCursorKind::Placed
        );
        assert_eq!(
            previous_cursor_kind_for_source_waypoint(SourceWaypointMode::Dodge),
            ZCursorKind::Placed
        );
        assert_eq!(
            previous_cursor_kind_for_source_waypoint(SourceWaypointMode::PickupGrenades),
            ZCursorKind::Grabbed
        );
        assert_eq!(
            previous_cursor_kind_for_source_waypoint(SourceWaypointMode::Enter),
            ZCursorKind::Entered
        );
        assert_eq!(
            previous_cursor_kind_for_source_waypoint(SourceWaypointMode::Attack),
            ZCursorKind::Attacked
        );
        assert_eq!(
            previous_cursor_kind_for_source_waypoint(SourceWaypointMode::Agro),
            ZCursorKind::Attacked
        );
        assert_eq!(
            previous_cursor_kind_for_source_waypoint(SourceWaypointMode::UnitRepair),
            ZCursorKind::Repaired
        );
        assert_eq!(
            previous_cursor_kind_for_source_waypoint(SourceWaypointMode::CraneRepair),
            ZCursorKind::Repaired
        );
    }

    #[test]
    fn waypoint_feedback_line_points_match_source_four_pixel_phase() {
        assert_eq!(
            source_waypoint_line_points(Vec2::ZERO, Vec2::new(8.0, 0.0), 0),
            vec![Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(8.0, 0.0)]
        );
        assert_eq!(
            source_waypoint_line_points(Vec2::ZERO, Vec2::new(8.0, 0.0), 1),
            vec![Vec2::new(1.0, 0.0), Vec2::new(5.0, 0.0)]
        );
    }

    #[test]
    fn waypoint_feedback_attack_uses_live_target_center_like_source() {
        let path = WaypointFeedbackPath::new(
            Vec2::ZERO,
            vec![SourceWaypoint {
                mode: SourceWaypointMode::Attack,
                ref_id: 7,
                x: 99,
                y: 99,
                attack_to: true,
                player_given: true,
            }],
        );
        let target_positions = HashMap::from([(7, Vec2::new(8.0, 0.0))]);

        assert_eq!(
            source_waypoint_feedback_points(&path, 0, &target_positions),
            vec![Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(8.0, 0.0)]
        );
    }

    fn waypoint_feedback_path_for_test(x: i32) -> WaypointFeedbackPath {
        WaypointFeedbackPath::new(
            Vec2::ZERO,
            vec![SourceWaypoint {
                mode: SourceWaypointMode::Move,
                ref_id: -1,
                x,
                y: 0,
                attack_to: true,
                player_given: true,
            }],
        )
    }

    fn waypoint_feedback_refs(state: &WaypointFeedbackState) -> Vec<u32> {
        let mut refs: Vec<_> = state.transient_paths.keys().copied().collect();
        refs.sort_unstable();
        refs
    }

    fn assert_close(left: f32, right: f32) {
        assert!(
            (left - right).abs() < 0.001,
            "expected {left} to be close to {right}"
        );
    }

    #[test]
    fn waypoint_feedback_keeps_overlapping_source_show_waypoints_lifetimes() {
        let mut state = WaypointFeedbackState::default();
        state.show_object_paths(HashMap::from([(1, waypoint_feedback_path_for_test(8))]));
        state.tick(1.0);

        state.show_object_paths(HashMap::from([(2, waypoint_feedback_path_for_test(16))]));

        assert_eq!(waypoint_feedback_refs(&state), vec![1, 2]);
        assert_close(state.transient_paths.get(&1).unwrap().remaining, 2.0);
        assert_close(state.transient_paths.get(&2).unwrap().remaining, 3.0);

        state.tick(2.1);

        assert_eq!(waypoint_feedback_refs(&state), vec![2]);
        assert_close(state.transient_paths.get(&2).unwrap().remaining, 0.9);
    }

    #[test]
    fn waypoint_feedback_replaces_only_the_same_object_path() {
        let mut state = WaypointFeedbackState::default();
        state.show_object_paths(HashMap::from([
            (1, waypoint_feedback_path_for_test(8)),
            (2, waypoint_feedback_path_for_test(16)),
        ]));
        state.tick(1.0);

        let replacement = waypoint_feedback_path_for_test(24);
        state.show_object_paths(HashMap::from([(1, replacement.clone())]));

        assert_eq!(waypoint_feedback_refs(&state), vec![1, 2]);
        assert_close(state.transient_paths.get(&1).unwrap().remaining, 3.0);
        assert_close(state.transient_paths.get(&2).unwrap().remaining, 2.0);
        assert_eq!(state.transient_paths.get(&1).unwrap().path, replacement);
    }

    #[test]
    fn one_unit_command_ref_chooses_nearest_candidate_to_target() {
        assert_eq!(
            one_unit_command_ref(
                Vec2::new(24.0, -10.0),
                [
                    (1, Vec2::new(0.0, 0.0)),
                    (2, Vec2::new(30.0, -12.0)),
                    (3, Vec2::new(80.0, -80.0)),
                ],
            ),
            Some(2),
        );
    }

    #[test]
    fn one_unit_command_ref_ignores_non_finite_candidates() {
        assert_eq!(
            one_unit_command_ref(
                Vec2::ZERO,
                [(1, Vec2::new(f32::NAN, 0.0)), (2, Vec2::new(8.0, 0.0)),],
            ),
            Some(2),
        );
        assert_eq!(
            one_unit_command_ref(Vec2::ZERO, [(1, Vec2::new(f32::INFINITY, 0.0))]),
            None,
        );
    }

    #[test]
    fn pending_move_expand_uses_robot_group_leader_like_order_fanout() {
        use crate::original::objects::RobotType;

        let kind = ObjectKind::Robot(RobotType::Grunt);
        let stats = ObjectStats::from_kind(kind, 100);
        let snapshots = [
            MouseCommandObjectSnapshot {
                ref_id: 10,
                kind,
                position: Vec2::ZERO,
                selection_size: Vec2::splat(TILE_SIZE),
                team: TeamType::Red,
                stats,
                grenade_amount: None,
                mobile: true,
                group_leader_ref_id: None,
            },
            MouseCommandObjectSnapshot {
                ref_id: 11,
                kind,
                position: Vec2::new(8.0, 0.0),
                selection_size: Vec2::splat(TILE_SIZE),
                team: TeamType::Red,
                stats,
                grenade_amount: None,
                mobile: true,
                group_leader_ref_id: Some(10),
            },
        ];

        assert_eq!(
            expand_selected_refs_for_orders_from_snapshots(&[10], &snapshots),
            vec![10, 11],
        );
        assert_eq!(
            expand_selected_refs_for_orders_from_snapshots(&[11], &snapshots),
            vec![11],
        );
    }

    #[test]
    fn pending_move_near_flag_uses_source_flag_radius() {
        let stats = ObjectStats::from_kind(ObjectKind::MapItem(ItemType::Flag as u8), 100);
        let snapshots = [MouseCommandObjectSnapshot {
            ref_id: 1,
            kind: ObjectKind::MapItem(ItemType::Flag as u8),
            position: Vec2::new(20.0, 0.0),
            selection_size: Vec2::splat(TILE_SIZE),
            team: TeamType::Red,
            stats,
            grenade_amount: None,
            mobile: false,
            group_leader_ref_id: None,
        }];

        assert!(move_target_is_near_flag_from_snapshots(
            Vec2::ZERO,
            &snapshots
        ));
        assert!(!move_target_is_near_flag_from_snapshots(
            Vec2::new(60.0, 0.0),
            &snapshots
        ));
    }

    #[test]
    fn pending_command_selected_refs_preserves_enter_selection_owner() {
        let enter = PendingMouseCommand::Enter(PendingMouseEnterCommand {
            world_pos: Vec2::new(16.0, -16.0),
            selected_refs: vec![3, 4],
            robot_refs: vec![4],
            target: EnterTargetInfo {
                ref_id: 9,
                position: Vec2::new(32.0, -32.0),
                waypoint: Vec2::new(34.0, -34.0),
            },
        });
        let enter_fort = PendingMouseCommand::EnterFort(PendingMouseEnterFortCommand {
            world_pos: Vec2::new(48.0, -48.0),
            selected_refs: vec![7],
            robot_refs: vec![7],
            target: EnterFortTargetInfo {
                ref_id: 10,
                position: Vec2::new(64.0, -64.0),
                inside_point: Vec2::new(72.0, -72.0),
                exit_point: Vec2::new(80.0, -80.0),
            },
        });

        assert_eq!(pending_command_selected_refs(&enter), &[3, 4]);
        assert_eq!(pending_command_selected_refs(&enter_fort), &[7]);
    }

    #[test]
    fn pending_command_selected_refs_preserves_repair_selection_owner() {
        let unit_repair = PendingMouseCommand::UnitRepair(PendingMouseUnitRepairCommand {
            world_pos: Vec2::new(16.0, -16.0),
            selected_refs: vec![3, 4],
            unit_refs: vec![3],
            target: UnitRepairTargetInfo {
                ref_id: 9,
                entrance_point: Vec2::new(32.0, -32.0),
                center_point: Vec2::new(40.0, -40.0),
            },
        });
        let crane_repair = PendingMouseCommand::CraneRepair(PendingMouseCraneRepairCommand {
            world_pos: Vec2::new(48.0, -48.0),
            selected_refs: vec![7],
            crane_refs: vec![7],
            target: CraneRepairTargetInfo {
                ref_id: 10,
                entrance_point: Vec2::new(64.0, -64.0),
                center_point: Vec2::new(72.0, -72.0),
            },
        });

        assert_eq!(pending_command_selected_refs(&unit_repair), &[3, 4]);
        assert_eq!(pending_command_selected_refs(&crane_repair), &[7]);
    }

    #[test]
    fn queued_move_and_attack_after_repair_are_preserved_as_resume_tail() {
        use crate::original::objects::VehicleType;

        let unit = source_relay_snapshot(
            3,
            ObjectKind::Vehicle(VehicleType::Light),
            TeamType::Red,
            50,
            true,
        );
        let target = source_relay_snapshot(
            9,
            ObjectKind::Vehicle(VehicleType::Heavy),
            TeamType::Blue,
            100,
            true,
        );
        let pending = vec![
            PendingMouseCommand::Move(PendingMouseMoveCommand {
                world_pos: Vec2::new(80.0, -96.0),
                selected_refs: vec![3],
                ctrl_down: true,
                alt_down: false,
            }),
            PendingMouseCommand::Attack(PendingMouseAttackCommand {
                world_pos: Vec2::new(112.0, -128.0),
                selected_refs: vec![3],
                target_ref_id: 9,
            }),
        ];

        let tail = pending_repair_resume_waypoints(
            &pending,
            unit,
            Vec2::new(64.0, -64.0),
            &[unit, target],
        );
        assert_eq!(tail.len(), 2);
        assert_eq!(
            tail[0],
            MovementWaypoint::player_move_to(Vec2::new(80.0, -96.0), true)
        );
        assert_eq!(
            tail[1],
            MovementWaypoint::player_attack_target(9, Vec2::new(112.0, -128.0))
        );
    }

    #[test]
    fn pending_command_selected_refs_preserves_pickup_selection_owner() {
        let command = PendingMouseCommand::PickupGrenades(PendingMousePickupGrenadesCommand {
            world_pos: Vec2::new(16.0, -16.0),
            selected_refs: vec![3, 4],
            robot_refs: vec![4],
            target_ref_id: 9,
            target_position: Vec2::new(32.0, -32.0),
        });

        assert_eq!(pending_command_selected_refs(&command), &[3, 4]);
    }

    #[test]
    fn pending_attack_no_way_uses_source_first_attack_waypoint_gate() {
        use crate::original::objects::{RobotType, VehicleType};

        let grunt = ObjectKind::Robot(RobotType::Grunt);
        let heavy = ObjectKind::Vehicle(VehicleType::Heavy);
        let snapshots = [
            MouseCommandObjectSnapshot {
                ref_id: 7,
                kind: grunt,
                position: Vec2::ZERO,
                selection_size: Vec2::splat(TILE_SIZE),
                team: TeamType::Red,
                stats: ObjectStats::from_kind(grunt, 100),
                grenade_amount: None,
                mobile: true,
                group_leader_ref_id: None,
            },
            MouseCommandObjectSnapshot {
                ref_id: 8,
                kind: heavy,
                position: Vec2::new(64.0, 0.0),
                selection_size: Vec2::splat(TILE_SIZE),
                team: TeamType::Blue,
                stats: ObjectStats::from_kind(heavy, 100),
                grenade_amount: None,
                mobile: true,
                group_leader_ref_id: None,
            },
        ];

        assert!(pending_attack_no_way(
            &PendingMouseCommand::Attack(PendingMouseAttackCommand {
                world_pos: Vec2::new(64.0, 0.0),
                selected_refs: vec![7],
                target_ref_id: 8,
            }),
            &snapshots,
        ));
        assert!(!pending_attack_no_way(
            &PendingMouseCommand::Move(PendingMouseMoveCommand {
                world_pos: Vec2::new(64.0, 0.0),
                selected_refs: vec![7],
                ctrl_down: false,
                alt_down: false,
            }),
            &snapshots,
        ));
    }

    #[test]
    fn attack_command_no_way_matches_single_selected_source_gate() {
        use crate::original::objects::{RobotType, VehicleType};

        assert!(attack_command_no_way(
            &[7],
            8,
            [
                (7, ObjectKind::Robot(RobotType::Grunt)),
                (8, ObjectKind::Vehicle(VehicleType::Heavy)),
            ],
        ));
        assert!(!attack_command_no_way(
            &[7, 9],
            8,
            [
                (7, ObjectKind::Robot(RobotType::Grunt)),
                (8, ObjectKind::Vehicle(VehicleType::Heavy)),
            ],
        ));
        assert!(!attack_command_no_way(
            &[7],
            8,
            [
                (7, ObjectKind::Vehicle(VehicleType::Heavy)),
                (8, ObjectKind::Robot(RobotType::Grunt)),
            ],
        ));
    }

    #[test]
    fn player_move_path_marks_only_final_waypoint_as_player_attack_to() {
        let path = movement_player_move_path_for_route(
            &[Vec2::new(16.0, -16.0), Vec2::new(32.0, -32.0)],
            Vec2::new(1.0, -2.0),
            30.0,
            true,
            true,
        );

        assert!(path.attempt_run);
        assert_eq!(path.typed_waypoints.len(), 2);
        assert_eq!(path.typed_waypoints[0].mode, MovementWaypointMode::Move);
        assert!(!path.typed_waypoints[0].attack_to);
        assert!(!path.typed_waypoints[0].player_given);
        assert_eq!(path.typed_waypoints[1].mode, MovementWaypointMode::Move);
        assert!(path.typed_waypoints[1].attack_to);
        assert!(path.typed_waypoints[1].player_given);
        assert_eq!(path.typed_waypoints[1].position, Vec2::new(33.0, -34.0));
    }

    fn source_relay_snapshot(
        ref_id: u32,
        kind: ObjectKind,
        team: TeamType,
        health_percent: i32,
        mobile: bool,
    ) -> MouseCommandObjectSnapshot {
        MouseCommandObjectSnapshot {
            ref_id,
            kind,
            position: Vec2::new(ref_id as f32 * 8.0, -(ref_id as f32 * 4.0)),
            selection_size: Vec2::splat(TILE_SIZE),
            team,
            stats: ObjectStats::from_kind(kind, health_percent),
            grenade_amount: None,
            mobile,
            group_leader_ref_id: None,
        }
    }

    fn source_relay_snapshot_with_grenades(
        ref_id: u32,
        kind: ObjectKind,
        team: TeamType,
        health_percent: i32,
        mobile: bool,
        grenade_amount: u8,
    ) -> MouseCommandObjectSnapshot {
        MouseCommandObjectSnapshot {
            grenade_amount: Some(grenade_amount),
            ..source_relay_snapshot(ref_id, kind, team, health_percent, mobile)
        }
    }

    #[test]
    fn source_clone_minion_waypoints_retargets_layer_offsets_and_speed() {
        use crate::original::objects::RobotType;

        let kind = ObjectKind::Robot(RobotType::Grunt);
        let mut leader = source_relay_snapshot(10, kind, TeamType::Red, 100, true);
        leader.position = Vec2::new(100.0, -100.0);
        leader.group_leader_ref_id = Some(10);
        let mut minion = source_relay_snapshot(11, kind, TeamType::Red, 100, true);
        minion.position = Vec2::new(120.0, -110.0);
        minion.group_leader_ref_id = Some(10);
        minion.stats.move_speed = 47.0;
        let leader_layer_position = leader.position + Vec2::new(2.0, -3.0);
        let minion_layer_position = minion.position + Vec2::new(-4.0, 5.0);
        let path = MovementPath::from_typed(
            vec![
                MovementWaypoint::move_to(Vec2::new(210.0, -210.0)),
                MovementWaypoint::player_attack_target(99, Vec2::new(220.0, -220.0)),
            ],
            30.0,
        )
        .with_run_attempt();
        let snapshots = [leader, minion];
        let layers = [
            (Entity::PLACEHOLDER, 10, leader_layer_position, None, None),
            (Entity::PLACEHOLDER, 11, minion_layer_position, None, None),
        ];

        let cloned = source_clone_minion_waypoint_paths(
            10,
            leader.position,
            leader_layer_position,
            &path,
            &snapshots,
            &layers,
        );

        assert_eq!(cloned.len(), 1);
        assert_eq!(cloned[0].ref_id, 11);
        assert_eq!(cloned[0].path.speed, 47.0);
        assert!(cloned[0].path.attempt_run);
        assert_eq!(
            cloned[0].path.typed_waypoints[0].mode,
            MovementWaypointMode::Move
        );
        assert_eq!(
            cloned[0].path.typed_waypoints[0].position,
            Vec2::new(204.0, -202.0)
        );
        assert_eq!(
            cloned[0].path.typed_waypoints[1].mode,
            MovementWaypointMode::Attack
        );
        assert_eq!(cloned[0].path.typed_waypoints[1].ref_id, Some(99));
        assert_eq!(
            cloned[0].path.typed_waypoints[1].position,
            Vec2::new(214.0, -212.0)
        );
    }

    #[test]
    fn source_clone_minion_waypoints_skips_destroyed_minions_and_other_groups() {
        use crate::original::objects::RobotType;

        let kind = ObjectKind::Robot(RobotType::Grunt);
        let mut leader = source_relay_snapshot(10, kind, TeamType::Red, 100, true);
        leader.group_leader_ref_id = Some(10);
        let mut destroyed_minion = source_relay_snapshot(11, kind, TeamType::Red, 0, true);
        destroyed_minion.group_leader_ref_id = Some(10);
        let mut other_group_minion = source_relay_snapshot(12, kind, TeamType::Red, 100, true);
        other_group_minion.group_leader_ref_id = Some(20);
        let path = MovementPath::from_typed(
            vec![MovementWaypoint::move_to(Vec2::new(32.0, -32.0))],
            30.0,
        );
        let snapshots = [leader, destroyed_minion, other_group_minion];
        let layers = [
            (Entity::PLACEHOLDER, 10, Vec2::ZERO, None, None),
            (Entity::PLACEHOLDER, 11, Vec2::ZERO, None, None),
            (Entity::PLACEHOLDER, 12, Vec2::ZERO, None, None),
        ];

        let cloned = source_clone_minion_waypoint_paths(
            10,
            Vec2::ZERO,
            Vec2::ZERO,
            &path,
            &snapshots,
            &layers,
        );

        assert!(cloned.is_empty());
    }

    #[test]
    fn source_rallypoints_relay_round_trips_move_points() {
        let snapshots = [source_relay_snapshot(
            12,
            ObjectKind::Building(BuildingType::RobotFactory),
            TeamType::Red,
            100,
            false,
        )];

        let relayed = source_relay_rally_points(
            12,
            &[Vec2::new(32.0, -48.0), Vec2::new(64.0, -80.0)],
            TeamType::Red,
            &snapshots,
        )
        .unwrap();

        assert_eq!(
            relayed,
            vec![Vec2::new(32.0, -48.0), Vec2::new(64.0, -80.0)]
        );
    }

    #[test]
    fn source_rallypoints_rejects_wrong_team_and_non_rally_buildings() {
        let points = [Vec2::new(32.0, -48.0)];
        let mut snapshots = [source_relay_snapshot(
            12,
            ObjectKind::Building(BuildingType::RobotFactory),
            TeamType::Red,
            100,
            false,
        )];

        assert!(source_relay_rally_points(12, &points, TeamType::Blue, &snapshots).is_none());

        snapshots[0].kind = ObjectKind::Building(BuildingType::Repair);
        assert!(source_relay_rally_points(12, &points, TeamType::Red, &snapshots).is_none());
    }

    #[test]
    fn source_rallypoints_process_filters_non_move_waypoints() {
        let snapshots = [source_relay_snapshot(
            12,
            ObjectKind::Building(BuildingType::VehicleFactory),
            TeamType::Red,
            100,
            false,
        )];
        let packet = SendRallypointsPacket {
            ref_id: 12,
            waypoints: vec![
                source_rally_waypoint_from_point(Vec2::new(32.0, -48.0)),
                SourceWaypoint {
                    mode: SourceWaypointMode::Attack,
                    ref_id: 99,
                    x: 64,
                    y: -80,
                    attack_to: true,
                    player_given: true,
                },
            ],
        };

        let accepted = source_process_rallypoint_data(&packet, TeamType::Red, &snapshots).unwrap();

        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].mode, SourceWaypointMode::Move);
        assert_eq!(
            source_rally_point_from_waypoint(accepted[0]),
            Vec2::new(32.0, -48.0)
        );
    }

    #[test]
    fn source_send_waypoints_relay_round_trips_movement_path() {
        use crate::original::objects::{RobotType, VehicleType};

        let path = MovementPath::from_typed(
            vec![
                MovementWaypoint::move_to(Vec2::new(16.0, -16.0)),
                MovementWaypoint::player_attack_target(9, Vec2::new(32.0, -32.0)),
            ],
            30.0,
        )
        .with_run_attempt();
        let snapshots = [
            source_relay_snapshot(
                7,
                ObjectKind::Robot(RobotType::Grunt),
                TeamType::Red,
                100,
                true,
            ),
            source_relay_snapshot(
                9,
                ObjectKind::Vehicle(VehicleType::Heavy),
                TeamType::Blue,
                100,
                true,
            ),
        ];

        let relayed = source_relay_movement_path(7, path, TeamType::Red, &snapshots).unwrap();

        assert!(relayed.attempt_run);
        assert_eq!(relayed.typed_waypoints.len(), 2);
        assert_eq!(relayed.typed_waypoints[0].mode, MovementWaypointMode::Move);
        assert_eq!(relayed.typed_waypoints[0].position, Vec2::new(16.0, -16.0));
        assert_eq!(
            relayed.typed_waypoints[1].mode,
            MovementWaypointMode::Attack
        );
        assert_eq!(relayed.typed_waypoints[1].ref_id, Some(9));
        assert!(relayed.typed_waypoints[1].attack_to);
        assert!(relayed.typed_waypoints[1].player_given);
    }

    #[test]
    fn source_send_waypoints_rejects_wrong_team_and_minions() {
        use crate::original::objects::RobotType;

        let path = MovementPath::from_typed(
            vec![MovementWaypoint::player_move_to(
                Vec2::new(16.0, -16.0),
                true,
            )],
            30.0,
        );
        let mut snapshots = [source_relay_snapshot(
            7,
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            100,
            true,
        )];

        assert!(source_relay_movement_path(7, path.clone(), TeamType::Blue, &snapshots).is_none());

        snapshots[0].group_leader_ref_id = Some(6);
        assert!(source_relay_movement_path(7, path, TeamType::Red, &snapshots).is_none());
    }

    #[test]
    fn source_send_waypoints_accepts_route_filtered_to_empty() {
        use crate::original::objects::RobotType;

        let path = MovementPath::from_typed(
            vec![MovementWaypoint::player_attack_target(
                99,
                Vec2::new(32.0, -32.0),
            )],
            30.0,
        );
        let snapshots = [source_relay_snapshot(
            7,
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            100,
            true,
        )];

        let relayed = source_relay_movement_path(7, path, TeamType::Red, &snapshots).unwrap();

        assert!(relayed.is_empty());
    }

    #[test]
    fn accepted_empty_waypoint_apply_preserves_current_attack_object() {
        fn apply_empty_waypoint_path(
            mut commands: Commands,
            query: Query<Entity, With<AttackTarget>>,
        ) {
            for entity in &query {
                insert_waypoint_command_path(
                    &mut commands,
                    entity,
                    MovementPath::from_typed(Vec::new(), 30.0),
                    None,
                    None,
                );
            }
        }

        let mut app = App::new();
        app.add_systems(Update, apply_empty_waypoint_path);
        let entity = app
            .world_mut()
            .spawn((
                AttackTarget {
                    ref_id: 99,
                    cooldown: 0.25,
                    player_given: true,
                },
                MovementPath::new(vec![Vec2::new(16.0, -16.0)], 30.0),
                PickupGrenadesTarget { ref_id: 88 },
                JustLeftCannon,
            ))
            .id();

        app.update();

        let world = app.world();
        assert!(world.get::<AttackTarget>(entity).is_some());
        assert!(world.get::<MovementPath>(entity).is_none());
        assert!(world.get::<PickupGrenadesTarget>(entity).is_none());
        assert!(world.get::<JustLeftCannon>(entity).is_none());
        assert!(world.get::<AcceptedEmptyWaypointCommand>(entity).is_some());
    }

    #[test]
    fn source_check_waypoint_rewrites_client_force_move_to_move() {
        use crate::original::objects::RobotType;

        let path = MovementPath::from_typed(
            vec![MovementWaypoint::force_move(Vec2::new(16.0, -16.0))],
            30.0,
        );
        let snapshots = [source_relay_snapshot(
            7,
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            100,
            true,
        )];

        let relayed = source_relay_movement_path(7, path, TeamType::Red, &snapshots).unwrap();

        assert_eq!(relayed.typed_waypoints.len(), 1);
        assert_eq!(relayed.typed_waypoints[0].mode, MovementWaypointMode::Move);
        assert!(!relayed.typed_waypoints[0].player_given);
    }

    #[test]
    fn source_process_waypoint_preserves_force_move_when_current_cannot_overwrite() {
        use crate::original::objects::RobotType;

        let existing = MovementPath::from_typed(
            vec![MovementWaypoint::force_move(Vec2::new(8.0, -8.0))],
            30.0,
        );
        let path = MovementPath::from_typed(
            vec![MovementWaypoint::player_move_to(
                Vec2::new(16.0, -16.0),
                true,
            )],
            30.0,
        );
        let snapshots = [source_relay_snapshot(
            7,
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            100,
            true,
        )];

        let relayed = source_relay_movement_path_with_existing(
            7,
            path,
            TeamType::Red,
            &snapshots,
            Some(&existing),
            None,
        )
        .unwrap();

        assert_eq!(relayed.typed_waypoints.len(), 2);
        assert_eq!(
            relayed.typed_waypoints[0].mode,
            MovementWaypointMode::ForceMove
        );
        assert_eq!(relayed.typed_waypoints[0].position, Vec2::new(8.0, -8.0));
        assert_eq!(relayed.typed_waypoints[1].mode, MovementWaypointMode::Move);
        assert_eq!(relayed.typed_waypoints[1].position, Vec2::new(16.0, -16.0));
        assert!(relayed.typed_waypoints[1].attack_to);
        assert!(relayed.typed_waypoints[1].player_given);
    }

    #[test]
    fn source_process_waypoint_skips_duplicate_preserved_first_waypoint() {
        use crate::original::objects::RobotType;

        let existing = MovementPath::from_typed(
            vec![MovementWaypoint::force_move(Vec2::new(8.0, -8.0))],
            30.0,
        );
        let path = MovementPath::from_typed(
            vec![
                MovementWaypoint::force_move(Vec2::new(8.0, -8.0)),
                MovementWaypoint::player_move_to(Vec2::new(16.0, -16.0), true),
                MovementWaypoint::player_move_to(Vec2::new(24.0, -24.0), true),
            ],
            30.0,
        );
        let snapshots = [source_relay_snapshot(
            7,
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            100,
            true,
        )];

        let relayed = source_relay_movement_path_with_existing(
            7,
            path,
            TeamType::Red,
            &snapshots,
            Some(&existing),
            None,
        )
        .unwrap();

        assert_eq!(relayed.typed_waypoints.len(), 3);
        assert_eq!(
            relayed.typed_waypoints[0].mode,
            MovementWaypointMode::ForceMove
        );
        assert_eq!(relayed.typed_waypoints[0].position, Vec2::new(8.0, -8.0));
        assert_eq!(relayed.typed_waypoints[1].mode, MovementWaypointMode::Move);
        assert_eq!(relayed.typed_waypoints[1].position, Vec2::new(16.0, -16.0));
        assert_eq!(relayed.typed_waypoints[2].mode, MovementWaypointMode::Move);
        assert_eq!(relayed.typed_waypoints[2].position, Vec2::new(24.0, -24.0));
    }

    #[test]
    fn source_process_waypoint_overwrites_current_move_when_allowed() {
        use crate::original::objects::RobotType;

        let existing =
            MovementPath::from_typed(vec![MovementWaypoint::move_to(Vec2::new(8.0, -8.0))], 30.0);
        let path = MovementPath::from_typed(
            vec![MovementWaypoint::player_move_to(
                Vec2::new(16.0, -16.0),
                true,
            )],
            30.0,
        );
        let snapshots = [source_relay_snapshot(
            7,
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            100,
            true,
        )];

        let relayed = source_relay_movement_path_with_existing(
            7,
            path,
            TeamType::Red,
            &snapshots,
            Some(&existing),
            None,
        )
        .unwrap();

        assert_eq!(relayed.typed_waypoints.len(), 1);
        assert_eq!(relayed.typed_waypoints[0].mode, MovementWaypointMode::Move);
        assert_eq!(relayed.typed_waypoints[0].position, Vec2::new(16.0, -16.0));
        assert!(relayed.typed_waypoints[0].attack_to);
        assert!(relayed.typed_waypoints[0].player_given);
    }

    #[test]
    fn source_can_overwrite_current_stage_matches_repair_and_enter_fort_rules() {
        assert!(source_can_overwrite_current_stage(Some(
            SourceCurrentWaypointStage::CraneRepair(CraneRepairStage::GotoEntrance),
        )));
        assert!(!source_can_overwrite_current_stage(Some(
            SourceCurrentWaypointStage::CraneRepair(CraneRepairStage::EnterBuilding),
        )));
        assert!(!source_can_overwrite_current_stage(Some(
            SourceCurrentWaypointStage::CraneRepair(CraneRepairStage::ExitBuilding),
        )));

        assert!(source_can_overwrite_current_stage(Some(
            SourceCurrentWaypointStage::UnitRepair(UnitRepairStage::GotoEntrance),
        )));
        assert!(source_can_overwrite_current_stage(Some(
            SourceCurrentWaypointStage::UnitRepair(UnitRepairStage::Wait),
        )));
        assert!(!source_can_overwrite_current_stage(Some(
            SourceCurrentWaypointStage::UnitRepair(UnitRepairStage::EnterBuilding),
        )));
        assert!(!source_can_overwrite_current_stage(Some(
            SourceCurrentWaypointStage::UnitRepair(UnitRepairStage::ExitBuilding),
        )));

        assert!(source_can_overwrite_current_stage(Some(
            SourceCurrentWaypointStage::EnterFort(EnterFortStage::GotoEntrance),
        )));
        assert!(!source_can_overwrite_current_stage(Some(
            SourceCurrentWaypointStage::EnterFort(EnterFortStage::EnterBuilding),
        )));
        assert!(!source_can_overwrite_current_stage(Some(
            SourceCurrentWaypointStage::EnterFort(EnterFortStage::ExitBuilding),
        )));
    }

    #[test]
    fn source_relay_queues_move_tail_during_non_overwritable_current_stage() {
        use crate::original::objects::RobotType;

        let existing =
            MovementPath::from_typed(vec![MovementWaypoint::move_to(Vec2::new(8.0, -8.0))], 30.0);
        let path = MovementPath::from_typed(
            vec![MovementWaypoint::player_move_to(
                Vec2::new(16.0, -16.0),
                true,
            )],
            30.0,
        );
        let snapshots = [source_relay_snapshot(
            7,
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            100,
            true,
        )];

        let queued = source_relay_movement_path_with_existing(
            7,
            path.clone(),
            TeamType::Red,
            &snapshots,
            Some(&existing),
            Some(SourceCurrentWaypointStage::EnterFort(
                EnterFortStage::EnterBuilding,
            )),
        )
        .unwrap();

        assert_eq!(queued.typed_waypoints.len(), 2);
        assert_eq!(queued.typed_waypoints[0].position, Vec2::new(8.0, -8.0));
        assert_eq!(queued.typed_waypoints[1].position, Vec2::new(16.0, -16.0));
        assert!(queued.typed_waypoints[1].player_given);

        let accumulated = MovementPath::from_typed(
            vec![
                MovementWaypoint::move_to(Vec2::new(8.0, -8.0)),
                MovementWaypoint::player_move_to(Vec2::new(16.0, -16.0), true),
                MovementWaypoint::player_move_to(Vec2::new(24.0, -24.0), true),
            ],
            30.0,
        );
        let queued = source_relay_movement_path_with_existing(
            7,
            accumulated,
            TeamType::Red,
            &snapshots,
            Some(&existing),
            Some(SourceCurrentWaypointStage::EnterFort(
                EnterFortStage::EnterBuilding,
            )),
        )
        .unwrap();

        assert_eq!(queued.typed_waypoints.len(), 3);
        assert_eq!(queued.typed_waypoints[0].position, Vec2::new(8.0, -8.0));
        assert_eq!(queued.typed_waypoints[1].position, Vec2::new(16.0, -16.0));
        assert_eq!(queued.typed_waypoints[2].position, Vec2::new(24.0, -24.0));

        let replaced = source_relay_movement_path_with_existing(
            7,
            path,
            TeamType::Red,
            &snapshots,
            Some(&existing),
            Some(SourceCurrentWaypointStage::EnterFort(
                EnterFortStage::GotoEntrance,
            )),
        )
        .unwrap();

        assert_eq!(replaced.typed_waypoints.len(), 1);
        assert_eq!(replaced.typed_waypoints[0].position, Vec2::new(16.0, -16.0));
    }

    #[test]
    fn protected_unit_repair_keeps_active_path_out_of_repair_resume_tail() {
        let existing =
            MovementPath::from_typed(vec![MovementWaypoint::move_to(Vec2::new(8.0, -8.0))], 30.0);
        let queued = MovementPath::from_typed(
            vec![
                MovementWaypoint::move_to(Vec2::new(8.0, -8.0)),
                MovementWaypoint::player_move_to(Vec2::new(16.0, -16.0), true),
                MovementWaypoint::player_attack_target(99, Vec2::new(24.0, -24.0)),
            ],
            30.0,
        );

        let tail = protected_unit_repair_resume_waypoints(
            &queued,
            Some(SourceCurrentWaypointStage::UnitRepair(
                UnitRepairStage::EnterBuilding,
            )),
            Some(&existing),
        )
        .unwrap();

        assert_eq!(tail, queued.typed_waypoints[1..]);
    }

    #[test]
    fn source_pickup_grenades_relay_keeps_special_waypoint_out_of_movement_path() {
        use crate::original::objects::RobotType;

        let path = MovementPath::from_typed(
            vec![MovementWaypoint::move_to(Vec2::new(16.0, -16.0))],
            30.0,
        )
        .with_run_attempt();
        let snapshots = [
            source_relay_snapshot_with_grenades(
                7,
                ObjectKind::Robot(RobotType::Grunt),
                TeamType::Red,
                100,
                true,
                0,
            ),
            source_relay_snapshot(
                9,
                ObjectKind::MapItem(ItemType::Grenades as u8),
                TeamType::Null,
                100,
                false,
            ),
        ];

        let (relayed, pickup_ref_id) = source_relay_pickup_grenades_path(
            7,
            path,
            9,
            Vec2::new(31.6, -48.2),
            TeamType::Red,
            &snapshots,
        )
        .unwrap();

        assert_eq!(pickup_ref_id, 9);
        assert!(relayed.attempt_run);
        assert_eq!(relayed.typed_waypoints.len(), 1);
        assert_eq!(relayed.typed_waypoints[0].mode, MovementWaypointMode::Move);
        assert_eq!(relayed.typed_waypoints[0].position, Vec2::new(16.0, -16.0));
    }

    #[test]
    fn source_enter_relay_keeps_special_waypoint_out_of_movement_path() {
        use crate::original::objects::{RobotType, VehicleType};

        let path = MovementPath::from_typed(
            vec![MovementWaypoint::move_to(Vec2::new(16.0, -16.0))],
            30.0,
        )
        .with_run_attempt();
        let snapshots = [
            source_relay_snapshot(
                7,
                ObjectKind::Robot(RobotType::Grunt),
                TeamType::Red,
                100,
                true,
            ),
            source_relay_snapshot(
                9,
                ObjectKind::Vehicle(VehicleType::Jeep),
                TeamType::Null,
                100,
                true,
            ),
        ];

        let (relayed, enter_ref_id) = source_relay_special_waypoint_path(
            7,
            path,
            9,
            Vec2::new(31.6, -48.2),
            SourceWaypointMode::Enter,
            TeamType::Red,
            &snapshots,
        )
        .unwrap();

        assert_eq!(enter_ref_id, 9);
        assert!(relayed.attempt_run);
        assert_eq!(relayed.typed_waypoints.len(), 1);
        assert_eq!(relayed.typed_waypoints[0].mode, MovementWaypointMode::Move);
        assert_eq!(relayed.typed_waypoints[0].position, Vec2::new(16.0, -16.0));
    }

    #[test]
    fn source_enter_fort_relay_keeps_special_waypoint_out_of_movement_path() {
        use crate::original::objects::RobotType;

        let path = MovementPath::from_typed(
            vec![MovementWaypoint::move_to(Vec2::new(16.0, -16.0))],
            30.0,
        );
        let snapshots = [
            source_relay_snapshot(
                7,
                ObjectKind::Robot(RobotType::Grunt),
                TeamType::Red,
                100,
                true,
            ),
            source_relay_snapshot(
                11,
                ObjectKind::Building(BuildingType::FortFront),
                TeamType::Blue,
                100,
                false,
            ),
        ];

        let (relayed, fort_ref_id) = source_relay_special_waypoint_path(
            7,
            path,
            11,
            Vec2::new(64.0, -64.0),
            SourceWaypointMode::EnterFort,
            TeamType::Red,
            &snapshots,
        )
        .unwrap();

        assert_eq!(fort_ref_id, 11);
        assert!(!relayed.attempt_run);
        assert_eq!(relayed.typed_waypoints.len(), 1);
        assert_eq!(relayed.typed_waypoints[0].mode, MovementWaypointMode::Move);
        assert_eq!(relayed.typed_waypoints[0].position, Vec2::new(16.0, -16.0));
    }

    #[test]
    fn source_unit_repair_relay_keeps_special_waypoint_out_of_movement_path() {
        use crate::original::objects::VehicleType;

        let path = MovementPath::from_typed(
            vec![MovementWaypoint::move_to(Vec2::new(16.0, -16.0))],
            30.0,
        );
        let snapshots = [
            source_relay_snapshot(
                7,
                ObjectKind::Vehicle(VehicleType::Light),
                TeamType::Red,
                50,
                true,
            ),
            source_relay_snapshot(
                12,
                ObjectKind::Building(BuildingType::Repair),
                TeamType::Red,
                100,
                false,
            ),
        ];

        let (relayed, repair_ref_id) = source_relay_special_waypoint_path(
            7,
            path,
            12,
            Vec2::new(24.0, -32.0),
            SourceWaypointMode::UnitRepair,
            TeamType::Red,
            &snapshots,
        )
        .unwrap();

        assert_eq!(repair_ref_id, 12);
        assert_eq!(relayed.typed_waypoints.len(), 1);
        assert_eq!(relayed.typed_waypoints[0].mode, MovementWaypointMode::Move);
        assert_eq!(relayed.typed_waypoints[0].position, Vec2::new(16.0, -16.0));
    }

    #[test]
    fn source_crane_repair_relay_keeps_special_waypoint_out_of_movement_path() {
        use crate::original::objects::VehicleType;

        let path = MovementPath::from_typed(
            vec![MovementWaypoint::move_to(Vec2::new(16.0, -16.0))],
            30.0,
        );
        let snapshots = [
            source_relay_snapshot(
                7,
                ObjectKind::Vehicle(VehicleType::Crane),
                TeamType::Red,
                100,
                true,
            ),
            source_relay_snapshot(
                13,
                ObjectKind::Building(BuildingType::Radar),
                TeamType::Red,
                0,
                false,
            ),
        ];

        let (relayed, repair_ref_id) = source_relay_special_waypoint_path(
            7,
            path,
            13,
            Vec2::new(40.0, -48.0),
            SourceWaypointMode::CraneRepair,
            TeamType::Red,
            &snapshots,
        )
        .unwrap();

        assert_eq!(repair_ref_id, 13);
        assert_eq!(relayed.typed_waypoints.len(), 1);
        assert_eq!(relayed.typed_waypoints[0].mode, MovementWaypointMode::Move);
        assert_eq!(relayed.typed_waypoints[0].position, Vec2::new(16.0, -16.0));
    }

    #[test]
    fn player_move_attack_to_follows_source_ctrl_alt_and_near_hostiles() {
        use crate::original::objects::RobotType;

        let attacker_kind = ObjectKind::Robot(RobotType::Grunt);
        let attacker_stats = ObjectStats::from_kind(attacker_kind, 100);
        let hostile = PassiveCombatTargetSnapshot {
            ref_id: 7,
            kind: ObjectKind::Robot(RobotType::Grunt),
            position: Vec2::new(8.0, 0.0),
            size: Vec2::splat(TILE_SIZE),
            team: TeamType::Blue,
            stats: ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100),
        };

        assert!(!player_move_waypoint_attack_to(
            1,
            Vec2::ZERO,
            TeamType::Red,
            attacker_stats,
            &[hostile],
            false,
            false,
        ));
        assert!(player_move_waypoint_attack_to(
            1,
            Vec2::ZERO,
            TeamType::Red,
            attacker_stats,
            &[hostile],
            true,
            false,
        ));
        assert!(!player_move_waypoint_attack_to(
            1,
            Vec2::ZERO,
            TeamType::Red,
            attacker_stats,
            &[hostile],
            true,
            true,
        ));
        assert!(player_move_waypoint_attack_to(
            1,
            Vec2::ZERO,
            TeamType::Red,
            attacker_stats,
            &[],
            false,
            false,
        ));
    }

    #[test]
    fn player_attack_path_appends_source_flagged_attack_waypoint() {
        let path = movement_attack_path_for_route(
            &[Vec2::new(16.0, -16.0), Vec2::new(32.0, -16.0)],
            Vec2::new(2.0, -3.0),
            7,
            Vec2::new(40.0, -24.0),
            12.0,
        );

        assert_eq!(path.waypoints.len(), 3);
        assert_eq!(path.typed_waypoints[0].mode, MovementWaypointMode::Move);
        assert_eq!(path.typed_waypoints[0].position, Vec2::new(18.0, -19.0));
        assert_eq!(path.typed_waypoints[1].mode, MovementWaypointMode::Move);
        assert_eq!(path.typed_waypoints[1].position, Vec2::new(34.0, -19.0));

        let attack = path.typed_waypoints[2];
        assert_eq!(attack.mode, MovementWaypointMode::Attack);
        assert_eq!(attack.ref_id, Some(7));
        assert!(attack.attack_to);
        assert!(attack.player_given);
        assert_eq!(attack.position, Vec2::new(42.0, -27.0));
    }

    #[test]
    fn fort_attack_mask_accepts_original_rectangles_with_inclusive_edges() {
        let grid_position = MapGridPosition { x: 10, y: 20 };
        let hits = [
            (16.0, 16.0),
            (144.0, 128.0),
            (0.0, 48.0),
            (160.0, 112.0),
            (16.0, 0.0),
            (48.0, 16.0),
            (112.0, 0.0),
            (144.0, 16.0),
            (32.0, 128.0),
            (48.0, 144.0),
            (112.0, 128.0),
            (128.0, 144.0),
        ];

        for (rx, ry) in hits {
            assert!(
                fort_attack_hit_at_world_position(
                    fort_world_position(grid_position, rx, ry),
                    &grid_position,
                ),
                "expected fort attack hit at rx={rx}, ry={ry}",
            );
        }
    }

    #[test]
    fn fort_attack_mask_rejects_empty_corners_and_gaps() {
        let grid_position = MapGridPosition { x: 10, y: 20 };
        let misses = [
            (0.0, 0.0),
            (80.0, 0.0),
            (0.0, 16.0),
            (159.0, 47.0),
            (160.1, 112.0),
            (80.0, 144.0),
            (144.1, 128.0),
        ];

        for (rx, ry) in misses {
            assert!(
                !fort_attack_hit_at_world_position(
                    fort_world_position(grid_position, rx, ry),
                    &grid_position,
                ),
                "expected fort attack miss at rx={rx}, ry={ry}",
            );
        }
    }

    #[test]
    fn fort_attack_hit_test_uses_mask_for_both_fort_faces() {
        let grid_position = MapGridPosition { x: 3, y: 4 };
        let selectable = broad_selectable();
        let target_position = fort_world_position(grid_position, 80.0, 72.0);
        let empty_corner = fort_world_position(grid_position, 0.0, 0.0);
        let main_area = fort_world_position(grid_position, 80.0, 72.0);

        for kind in [
            ObjectKind::Building(BuildingType::FortFront),
            ObjectKind::Building(BuildingType::FortBack),
        ] {
            assert!(attack_hit_test(
                kind,
                main_area,
                target_position,
                &selectable,
                Some(&grid_position),
            ));
            assert!(!attack_hit_test(
                kind,
                empty_corner,
                target_position,
                &selectable,
                Some(&grid_position),
            ));
        }
    }

    #[test]
    fn non_fort_attack_hit_test_keeps_radius_selection() {
        let selectable = Selectable {
            radius: 8.0,
            selection_size: Vec2::splat(TILE_SIZE),
            mobile: false,
        };
        let target_position = Vec2::ZERO;

        assert!(attack_hit_test(
            ObjectKind::Building(BuildingType::Radar),
            Vec2::new(8.0, 0.0),
            target_position,
            &selectable,
            None,
        ));
        assert!(!attack_hit_test(
            ObjectKind::Building(BuildingType::Radar),
            Vec2::new(8.1, 0.0),
            target_position,
            &selectable,
            None,
        ));
    }
}
