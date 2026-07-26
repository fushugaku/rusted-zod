use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashSet;

use crate::{
    components::*,
    constants::TILE_SIZE,
    object_sync::{
        CraneAnimPacketQueue, ProducedObjectMemberRoute, RepairBuildingAnimPacketQueue,
        RepairedObjectBatchPending, SourceDeleteGroupMember, SourceObjectEvent,
        SourceObjectEventQueue, relay_crane_anim_state, relay_deferred_delete_object,
        relay_repair_building_anim_state, relay_repaired_object_batch, source_new_object_refs,
        source_new_object_refs_available,
    },
    original::{objects::ObjectKind, types::TeamType},
    units::{buildings, vehicles},
};

#[derive(Clone, Copy)]
pub(crate) struct RepairCommandUnit {
    pub(crate) ref_id: u32,
    pub(crate) position: Vec2,
    pub(crate) move_speed: f32,
}

pub(crate) struct CraneRepairCommand {
    pub(crate) target: CraneRepairTargetInfo,
    pub(crate) cranes: Vec<RepairCommandUnit>,
}

#[derive(Clone, Copy)]
pub(crate) struct CraneRepairTargetInfo {
    pub(crate) ref_id: u32,
    pub(crate) entrance_point: Vec2,
    pub(crate) center_point: Vec2,
}

pub(crate) struct UnitRepairCommand {
    pub(crate) target: UnitRepairTargetInfo,
    pub(crate) units: Vec<RepairCommandUnit>,
}

#[derive(Clone, Copy)]
pub(crate) struct UnitRepairTargetInfo {
    pub(crate) ref_id: u32,
    pub(crate) entrance_point: Vec2,
    pub(crate) center_point: Vec2,
}

type RepairObjectQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GameObjectEntity,
        &'static ObjectTeam,
        &'static ObjectStats,
        Option<&'static CraneRepairTarget>,
        Option<&'static UnitRepairTarget>,
        Option<&'static MovementPath>,
        Option<&'static DriverHealth>,
        Option<&'static RobotGroup>,
        Option<&'static GrenadeInventory>,
        Option<&'static RepairResumeWaypoints>,
    ),
>;

type RepairTargetQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GameObjectEntity,
        &'static ObjectTeam,
        &'static MapGridPosition,
        &'static BuildingLevel,
        Option<&'static BridgeFootprint>,
        Option<&'static RepairBuildingOccupancy>,
        &'static mut ObjectStats,
    ),
>;

type RepairLayerQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static ObjectLayerRef,
        Option<&'static mut Sprite>,
        &'static mut Visibility,
        Option<&'static RepairResumeWaypoints>,
    ),
>;

#[derive(SystemParam)]
pub(crate) struct RepairQueries<'w, 's> {
    set: ParamSet<
        'w,
        's,
        (
            RepairObjectQuery<'w, 's>,
            RepairTargetQuery<'w, 's>,
            RepairLayerQuery<'w, 's>,
        ),
    >,
}

pub(crate) fn crane_repair_target_for_right_click(
    world_pos: Vec2,
    selected_refs: &[u32],
    repairer_team: TeamType,
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&RobotGroup>,
        Option<&BridgeFootprint>,
    )>,
) -> Option<CraneRepairCommand> {
    let cranes: Vec<_> = object_query
        .iter()
        .filter_map(|(object, transform, selectable, team, stats, _, _)| {
            (selected_refs.contains(&object.ref_id)
                && selectable.mobile
                && team.0 == repairer_team
                && vehicles::crane::can_issue_repair_command(object.kind, team.0, *stats))
            .then_some(RepairCommandUnit {
                ref_id: object.ref_id,
                position: transform.translation.truncate(),
                move_speed: stats.move_speed,
            })
        })
        .collect();

    let crane_positions: Vec<_> = cranes.iter().map(|crane| crane.position).collect();
    let target =
        crane_repairable_at_position(world_pos, repairer_team, &crane_positions, object_query)?;

    (!cranes.is_empty()).then_some(CraneRepairCommand { target, cranes })
}

pub(crate) fn unit_repair_target_for_right_click(
    world_pos: Vec2,
    selected_refs: &[u32],
    repairer_team: TeamType,
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&RobotGroup>,
        Option<&BridgeFootprint>,
    )>,
) -> Option<UnitRepairCommand> {
    let target = unit_repairable_at_position(world_pos, repairer_team, object_query)?;
    let units: Vec<_> = object_query
        .iter()
        .filter_map(|(object, transform, selectable, team, stats, _, _)| {
            (selected_refs.contains(&object.ref_id)
                && selectable.mobile
                && buildings::can_repair_target_unit(object.kind, team.0, *stats)
                && team.0 == repairer_team
                && stats.move_speed > 0.0)
                .then_some(RepairCommandUnit {
                    ref_id: object.ref_id,
                    position: transform.translation.truncate(),
                    move_speed: stats.move_speed,
                })
        })
        .collect();

    (!units.is_empty()).then_some(UnitRepairCommand { target, units })
}

fn crane_repairable_at_position(
    world_pos: Vec2,
    repairer_team: TeamType,
    repairer_positions: &[Vec2],
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&RobotGroup>,
        Option<&BridgeFootprint>,
    )>,
) -> Option<CraneRepairTargetInfo> {
    object_query
        .iter()
        .filter_map(|(object, transform, selectable, team, stats, _, bridge)| {
            if !vehicles::crane::can_repair_target(object.kind, team.0, repairer_team, *stats) {
                return None;
            }
            let position = transform.translation.truncate();
            if !point_in_repairable_rect(
                world_pos,
                position,
                selectable.selection_size,
                bridge.copied(),
            ) {
                return None;
            }
            let (entrance_point, center_point) = buildings::crane_repair_points(
                position,
                selectable.selection_size,
                object.kind,
                bridge.copied(),
                repairer_positions,
            )?;
            Some(CraneRepairTargetInfo {
                ref_id: object.ref_id,
                entrance_point,
                center_point,
            })
        })
        .min_by(|a, b| {
            a.entrance_point
                .distance_squared(world_pos)
                .total_cmp(&b.entrance_point.distance_squared(world_pos))
        })
}

fn unit_repairable_at_position(
    world_pos: Vec2,
    repairer_team: TeamType,
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&RobotGroup>,
        Option<&BridgeFootprint>,
    )>,
) -> Option<UnitRepairTargetInfo> {
    object_query
        .iter()
        .filter_map(|(object, transform, selectable, team, stats, _, _)| {
            if !buildings::can_repair_unit(object.kind, team.0, repairer_team, *stats) {
                return None;
            }
            let position = transform.translation.truncate();
            if !point_in_object_rect(world_pos, position, selectable.selection_size) {
                return None;
            }
            let (entrance_point, center_point) = buildings::repair_building_points(
                position,
                selectable.selection_size,
                object.kind,
            )?;
            Some(UnitRepairTargetInfo {
                ref_id: object.ref_id,
                entrance_point,
                center_point,
            })
        })
        .min_by(|a, b| {
            a.entrance_point
                .distance_squared(world_pos)
                .total_cmp(&b.entrance_point.distance_squared(world_pos))
        })
}

pub(crate) fn process_repair_targets(
    mut commands: Commands,
    time: Res<Time>,
    mut next_ref: ResMut<NextObjectRefId>,
    ref_reservations: Res<DynamicObjectRefReservations>,
    mut object_events: ResMut<SourceObjectEventQueue>,
    mut crane_anim_packets: ResMut<CraneAnimPacketQueue>,
    mut repair_anim_packets: ResMut<RepairBuildingAnimPacketQueue>,
    mut queries: RepairQueries,
) {
    let mut reserved_ref_ids: HashSet<u32> = queries
        .set
        .p0()
        .iter()
        .map(|(_, object, ..)| object.ref_id)
        .collect();
    reserved_ref_ids.extend(queries.set.p1().iter().map(|(_, object, ..)| object.ref_id));
    reserved_ref_ids.extend(object_events.pending.iter().filter_map(|event| {
        let SourceObjectEvent::AddNewObject { packet, .. } = event else {
            return None;
        };
        u32::try_from(packet.ref_id).ok()
    }));

    let mut occupancy_steps: Vec<_> = queries
        .set
        .p1()
        .iter()
        .filter_map(|(entity, object, team, _, _, _, occupancy, stats)| {
            occupancy
                .cloned()
                .map(|occupancy| RepairStep::RepairBuilding {
                    entity,
                    building_ref_id: object.ref_id,
                    team: team.0,
                    building_destroyed: stats.destroyed(),
                    occupancy,
                })
        })
        .collect();
    occupancy_steps.sort_by_key(|step| match step {
        RepairStep::RepairBuilding {
            building_ref_id, ..
        } => *building_ref_id,
        _ => u32::MAX,
    });
    let mut busy_repair_buildings = Vec::new();
    for step in occupancy_steps {
        let RepairStep::RepairBuilding {
            entity,
            building_ref_id,
            team,
            building_destroyed,
            occupancy,
        } = step
        else {
            unreachable!();
        };
        if process_repair_building_step(
            &mut commands,
            &mut object_events,
            &mut repair_anim_packets,
            &mut next_ref,
            &mut reserved_ref_ids,
            ref_reservations
                .first_ref_by_building
                .get(&building_ref_id)
                .copied(),
            time.delta_secs(),
            entity,
            building_ref_id,
            team,
            building_destroyed,
            occupancy,
        ) {
            busy_repair_buildings.push(building_ref_id);
        }
    }

    let delete_group_members: Vec<_> = queries
        .set
        .p0()
        .iter()
        .filter_map(|(_, object, _, _, _, _, _, _, group, inventory, _)| {
            let group = group?;
            Some(SourceDeleteGroupMember {
                ref_id: object.ref_id,
                leader_ref_id: group.leader_ref_id,
                member_index: group.member_index,
                grenade_amount: inventory.map_or(0, |inventory| inventory.amount),
            })
        })
        .collect();

    let mut steps: Vec<RepairStep> = queries
        .set
        .p0()
        .iter()
        .filter_map(
            |(entity, object, team, stats, crane, unit, movement, driver, _, _, repair_resume)| {
                if let Some(target) = crane {
                    return Some(RepairStep::Crane {
                        entity,
                        ref_id: object.ref_id,
                        team: team.0,
                        stats: *stats,
                        target: *target,
                        movement_active: movement.is_some(),
                    });
                }
                unit.map(|target| RepairStep::Unit {
                    entity,
                    ref_id: object.ref_id,
                    kind: object.kind,
                    team: team.0,
                    stats: *stats,
                    target: target.clone(),
                    movement_active: movement.is_some(),
                    driver: driver.cloned(),
                    repair_resume: repair_resume.cloned(),
                })
            },
        )
        .collect();
    steps.sort_by_key(|step| match step {
        RepairStep::Crane { ref_id, .. } | RepairStep::Unit { ref_id, .. } => *ref_id,
        RepairStep::RepairBuilding {
            building_ref_id, ..
        } => *building_ref_id,
    });

    for step in steps {
        match step {
            RepairStep::Crane {
                entity,
                ref_id,
                team,
                stats,
                target,
                movement_active,
            } => process_crane_step(
                &mut commands,
                &mut crane_anim_packets,
                &mut queries,
                entity,
                ref_id,
                team,
                stats,
                target,
                movement_active,
            ),
            RepairStep::Unit {
                entity,
                ref_id,
                team,
                kind,
                stats,
                target,
                movement_active,
                driver,
                repair_resume,
            } => process_unit_repair_step(
                &mut commands,
                &mut queries,
                &mut repair_anim_packets,
                &mut busy_repair_buildings,
                &mut object_events,
                &delete_group_members,
                entity,
                ref_id,
                kind,
                team,
                stats,
                target,
                movement_active,
                driver,
                repair_resume,
            ),
            RepairStep::RepairBuilding { .. } => unreachable!(),
        }
    }
}

enum RepairStep {
    Crane {
        entity: Entity,
        ref_id: u32,
        team: TeamType,
        stats: ObjectStats,
        target: CraneRepairTarget,
        movement_active: bool,
    },
    Unit {
        entity: Entity,
        ref_id: u32,
        kind: ObjectKind,
        team: TeamType,
        stats: ObjectStats,
        target: UnitRepairTarget,
        movement_active: bool,
        driver: Option<DriverHealth>,
        repair_resume: Option<RepairResumeWaypoints>,
    },
    RepairBuilding {
        entity: Entity,
        building_ref_id: u32,
        team: TeamType,
        building_destroyed: bool,
        occupancy: RepairBuildingOccupancy,
    },
}

fn advance_repair_building_occupancy(
    occupancy: &mut RepairBuildingOccupancy,
    delta_secs: f32,
    building_destroyed: bool,
) -> bool {
    occupancy.remaining -= delta_secs;
    occupancy.remaining <= 0.0 && !building_destroyed
}

fn repaired_driver_state(
    unit: ObjectKind,
    team: TeamType,
    stored_driver: Option<DriverHealth>,
) -> Option<DriverHealth> {
    stored_driver.or_else(|| crate::units::initial_driver_health(unit, team))
}

#[allow(clippy::too_many_arguments)]
fn process_repair_building_step(
    commands: &mut Commands,
    object_events: &mut SourceObjectEventQueue,
    repair_anim_packets: &mut RepairBuildingAnimPacketQueue,
    next_ref: &mut NextObjectRefId,
    reserved_ref_ids: &mut HashSet<u32>,
    reserved_first_ref_id: Option<u32>,
    delta_secs: f32,
    entity: Entity,
    building_ref_id: u32,
    team: TeamType,
    building_destroyed: bool,
    mut occupancy: RepairBuildingOccupancy,
) -> bool {
    if !advance_repair_building_occupancy(&mut occupancy, delta_secs, building_destroyed) {
        commands.entity(entity).insert(occupancy);
        return true;
    }

    let object_count = usize::from(crate::units::produced_object_count(occupancy.unit));
    let Some(first_ref_id) = reserved_first_ref_id else {
        commands.entity(entity).insert(occupancy);
        return true;
    };
    let Some(candidate_refs) = source_new_object_refs(first_ref_id, object_count) else {
        commands.entity(entity).insert(occupancy);
        return true;
    };
    if !source_new_object_refs_available(&candidate_refs, reserved_ref_ids) {
        commands.entity(entity).insert(occupancy);
        return true;
    }
    let Some((source_top_left_map, initial_move_target_map)) =
        buildings::repaired_unit_source_points(
            occupancy.unit,
            occupancy.center_point,
            occupancy.entrance_point,
        )
    else {
        commands.entity(entity).insert(occupancy);
        return true;
    };
    let event_checkpoint = object_events.pending.len();
    let repaired_driver = repaired_driver_state(occupancy.unit, team, occupancy.driver.clone());
    let Some(member_refs) = relay_repaired_object_batch(
        object_events,
        building_ref_id,
        first_ref_id,
        occupancy.unit,
        team,
        source_top_left_map,
        initial_move_target_map,
        object_count,
        repaired_driver.as_ref(),
        &occupancy.resume_waypoints,
    ) else {
        commands.entity(entity).insert(occupancy);
        return true;
    };
    if member_refs != candidate_refs {
        object_events.pending.truncate(event_checkpoint);
        commands.entity(entity).insert(occupancy);
        return true;
    }

    let waypoints =
        buildings::repaired_unit_waypoints(occupancy.entrance_point, &occupancy.resume_waypoints);
    let member_routes = member_refs
        .iter()
        .copied()
        .map(|ref_id| ProducedObjectMemberRoute {
            ref_id,
            waypoints: waypoints.clone(),
        })
        .collect();
    commands.entity(entity).insert(RepairedObjectBatchPending {
        unit: occupancy.unit,
        member_routes,
    });
    commands.entity(entity).remove::<RepairBuildingOccupancy>();
    relay_repair_building_anim_state(repair_anim_packets, building_ref_id, false, 0.0, true);
    reserved_ref_ids.extend(member_refs.iter().copied());
    next_ref.0 = next_ref.0.max(
        member_refs
            .last()
            .copied()
            .unwrap_or(first_ref_id)
            .saturating_add(1),
    );
    false
}

fn process_crane_step(
    commands: &mut Commands,
    crane_anim_packets: &mut CraneAnimPacketQueue,
    queries: &mut RepairQueries,
    entity: Entity,
    ref_id: u32,
    team: TeamType,
    stats: ObjectStats,
    target: CraneRepairTarget,
    movement_active: bool,
) {
    if stats.destroyed() || stats.move_speed <= 0.0 {
        commands.entity(entity).remove::<CraneRepairTarget>();
        return;
    }

    match source_crane_repair_waypoint_action(
        target.stage,
        movement_active,
        target_can_be_crane_repaired_state(queries, target.ref_id, team),
    ) {
        SourceCraneRepairWaypointAction::KeepMoving => {}
        SourceCraneRepairWaypointAction::KillWaypoint => {
            commands.entity(entity).remove::<CraneRepairTarget>();
        }
        SourceCraneRepairWaypointAction::StageEnterBuilding => {
            relay_crane_anim_state(crane_anim_packets, ref_id, target.ref_id, true);
            commands.entity(entity).insert(CraneRepairTarget {
                stage: CraneRepairStage::EnterBuilding,
                ..target
            });
            insert_layer_paths_for_ref(
                queries,
                commands,
                ref_id,
                target.center_point,
                stats.move_speed,
            );
        }
        SourceCraneRepairWaypointAction::StageExitBuilding => {
            commands.entity(entity).insert(CraneRepairTarget {
                stage: CraneRepairStage::ExitBuilding,
                ..target
            });
            insert_layer_paths_for_ref(
                queries,
                commands,
                ref_id,
                target.exit_point,
                stats.move_speed,
            );
        }
        SourceCraneRepairWaypointAction::FinishAutoRepair => {
            set_auto_repair_now(commands, queries, target.ref_id);
            relay_crane_anim_state(crane_anim_packets, ref_id, target.ref_id, false);
            commands.entity(entity).remove::<CraneRepairTarget>();
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceCraneRepairWaypointAction {
    KeepMoving,
    KillWaypoint,
    StageEnterBuilding,
    StageExitBuilding,
    FinishAutoRepair,
}

fn source_crane_repair_waypoint_action(
    stage: CraneRepairStage,
    movement_active: bool,
    target_can_be_repaired: Option<bool>,
) -> SourceCraneRepairWaypointAction {
    let Some(target_can_be_repaired) = target_can_be_repaired else {
        return SourceCraneRepairWaypointAction::KillWaypoint;
    };

    if !target_can_be_repaired {
        return match stage {
            CraneRepairStage::GotoEntrance => SourceCraneRepairWaypointAction::KillWaypoint,
            CraneRepairStage::EnterBuilding => SourceCraneRepairWaypointAction::StageExitBuilding,
            CraneRepairStage::ExitBuilding => {
                if movement_active {
                    SourceCraneRepairWaypointAction::KeepMoving
                } else {
                    SourceCraneRepairWaypointAction::FinishAutoRepair
                }
            }
        };
    }

    if movement_active {
        return SourceCraneRepairWaypointAction::KeepMoving;
    }

    match stage {
        CraneRepairStage::GotoEntrance => SourceCraneRepairWaypointAction::StageEnterBuilding,
        CraneRepairStage::EnterBuilding => SourceCraneRepairWaypointAction::StageExitBuilding,
        CraneRepairStage::ExitBuilding => SourceCraneRepairWaypointAction::FinishAutoRepair,
    }
}

fn process_unit_repair_step(
    commands: &mut Commands,
    queries: &mut RepairQueries,
    repair_anim_packets: &mut RepairBuildingAnimPacketQueue,
    busy_repair_buildings: &mut Vec<u32>,
    object_events: &mut SourceObjectEventQueue,
    delete_group_members: &[SourceDeleteGroupMember],
    entity: Entity,
    ref_id: u32,
    kind: ObjectKind,
    team: TeamType,
    stats: ObjectStats,
    target: UnitRepairTarget,
    movement_active: bool,
    driver: Option<DriverHealth>,
    repair_resume: Option<RepairResumeWaypoints>,
) {
    let target_busy = busy_repair_buildings.contains(&target.ref_id);
    let target_can_repair = target_can_repair_unit_state(queries, target.ref_id, team);
    let unit_can_be_repaired = buildings::can_repair_target_unit(kind, team, stats);

    match source_unit_repair_waypoint_action(
        target.stage,
        movement_active,
        unit_can_be_repaired,
        target_can_repair,
        target_busy,
    ) {
        SourceUnitRepairWaypointAction::KeepMoving => {}
        SourceUnitRepairWaypointAction::KeepWaiting => {}
        SourceUnitRepairWaypointAction::KillWaypoint => {
            restore_repair_resume_paths(
                queries,
                commands,
                entity,
                ref_id,
                repair_resume
                    .as_ref()
                    .map_or(&[], |resume| resume.0.as_slice()),
                stats.move_speed,
            );
            commands
                .entity(entity)
                .remove::<UnitRepairTarget>()
                .remove::<RepairResumeWaypoints>();
        }
        SourceUnitRepairWaypointAction::StageWait => {
            commands.entity(entity).insert(UnitRepairTarget {
                stage: UnitRepairStage::Wait,
                ..target
            });
        }
        SourceUnitRepairWaypointAction::StageEnterBuilding => {
            commands.entity(entity).insert(UnitRepairTarget {
                stage: UnitRepairStage::EnterBuilding,
                ..target
            });
            insert_layer_paths_for_ref(
                queries,
                commands,
                ref_id,
                target.center_point,
                stats.move_speed,
            );
        }
        SourceUnitRepairWaypointAction::StageExitBuilding => {
            commands.entity(entity).insert(UnitRepairTarget {
                stage: UnitRepairStage::ExitBuilding,
                ..target
            });
            insert_layer_paths_for_ref(
                queries,
                commands,
                ref_id,
                target.entrance_point,
                stats.move_speed,
            );
        }
        SourceUnitRepairWaypointAction::StartRepairing => {
            let Some(building_entity) =
                queries
                    .set
                    .p1()
                    .iter()
                    .find_map(|(building_entity, object, _, _, _, _, _, _)| {
                        (object.ref_id == target.ref_id).then_some(building_entity)
                    })
            else {
                commands.entity(entity).remove::<UnitRepairTarget>();
                return;
            };
            if !relay_deferred_delete_object(object_events, ref_id, delete_group_members) {
                commands.entity(entity).insert(UnitRepairTarget {
                    stage: UnitRepairStage::ExitBuilding,
                    ..target
                });
                insert_layer_paths_for_ref(
                    queries,
                    commands,
                    ref_id,
                    target.entrance_point,
                    stats.move_speed,
                );
                return;
            }
            commands
                .entity(building_entity)
                .insert(RepairBuildingOccupancy {
                    unit: kind,
                    driver,
                    center_point: target.center_point,
                    entrance_point: target.entrance_point,
                    resume_waypoints: repair_resume
                        .map(|resume| resume.0)
                        .unwrap_or_else(|| target.resume_waypoints.clone()),
                    remaining: buildings::REPAIR_BUILDING_SECONDS,
                });
            commands
                .entity(entity)
                .remove::<UnitRepairTarget>()
                .remove::<RepairResumeWaypoints>()
                .remove::<AttackTargetLifecycleComponents>();
            relay_repair_building_anim_state(
                repair_anim_packets,
                target.ref_id,
                true,
                buildings::REPAIR_BUILDING_SECONDS,
                true,
            );
            busy_repair_buildings.push(target.ref_id);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceUnitRepairWaypointAction {
    KeepMoving,
    KeepWaiting,
    KillWaypoint,
    StageWait,
    StageEnterBuilding,
    StageExitBuilding,
    StartRepairing,
}

fn source_unit_repair_waypoint_action(
    stage: UnitRepairStage,
    movement_active: bool,
    unit_can_be_repaired: bool,
    target_can_repair: Option<bool>,
    target_busy: bool,
) -> SourceUnitRepairWaypointAction {
    let Some(target_can_repair) = target_can_repair else {
        return SourceUnitRepairWaypointAction::KillWaypoint;
    };

    if !target_can_repair || !unit_can_be_repaired {
        return match stage {
            UnitRepairStage::EnterBuilding => SourceUnitRepairWaypointAction::StageExitBuilding,
            _ => SourceUnitRepairWaypointAction::KillWaypoint,
        };
    }

    if target_busy {
        return match stage {
            UnitRepairStage::EnterBuilding => SourceUnitRepairWaypointAction::StageExitBuilding,
            UnitRepairStage::Wait => SourceUnitRepairWaypointAction::KeepWaiting,
            _ if movement_active => SourceUnitRepairWaypointAction::KeepMoving,
            UnitRepairStage::GotoEntrance => SourceUnitRepairWaypointAction::StageWait,
            UnitRepairStage::ExitBuilding => SourceUnitRepairWaypointAction::StageWait,
        };
    }

    if movement_active {
        return SourceUnitRepairWaypointAction::KeepMoving;
    }

    match stage {
        UnitRepairStage::GotoEntrance => SourceUnitRepairWaypointAction::StageWait,
        UnitRepairStage::Wait => SourceUnitRepairWaypointAction::StageEnterBuilding,
        UnitRepairStage::EnterBuilding => SourceUnitRepairWaypointAction::StartRepairing,
        UnitRepairStage::ExitBuilding => SourceUnitRepairWaypointAction::StageWait,
    }
}

fn target_can_be_crane_repaired_state(
    queries: &mut RepairQueries,
    target_ref_id: u32,
    repairer_team: TeamType,
) -> Option<bool> {
    queries
        .set
        .p1()
        .iter()
        .find_map(|(_, object, team, _, _, _, _, stats)| {
            (object.ref_id == target_ref_id).then(|| {
                vehicles::crane::can_repair_target(object.kind, team.0, repairer_team, *stats)
            })
        })
}

fn set_auto_repair_now(commands: &mut Commands, queries: &mut RepairQueries, target_ref_id: u32) {
    let Some((entity, _, _, _, _, _, _, _)) = queries
        .set
        .p1()
        .iter_mut()
        .find(|(_, object, _, _, _, _, _, _)| object.ref_id == target_ref_id)
    else {
        return;
    };
    commands.entity(entity).insert(AutoRepair { timer: 0.0 });
}

fn target_can_repair_unit_state(
    queries: &mut RepairQueries,
    target_ref_id: u32,
    unit_team: TeamType,
) -> Option<bool> {
    queries
        .set
        .p1()
        .iter()
        .find_map(|(_, object, team, _, _, _, _, stats)| {
            (object.ref_id == target_ref_id)
                .then(|| buildings::can_repair_unit(object.kind, team.0, unit_team, *stats))
        })
}

fn insert_layer_paths_for_ref(
    queries: &mut RepairQueries,
    commands: &mut Commands,
    ref_id: u32,
    target: Vec2,
    move_speed: f32,
) {
    let Some(base_position) =
        queries
            .set
            .p2()
            .iter()
            .find_map(|(_, transform, layer_ref, _, _, _)| {
                (layer_ref.0 == ref_id).then_some(transform.translation.truncate())
            })
    else {
        return;
    };

    for (entity, transform, layer_ref, _, _, _) in &mut queries.set.p2() {
        if layer_ref.0 == ref_id {
            let layer_offset = transform.translation.truncate() - base_position;
            commands
                .entity(entity)
                .insert(MovementPath::new(vec![target + layer_offset], move_speed));
        }
    }
}

fn restore_repair_resume_paths(
    queries: &mut RepairQueries,
    commands: &mut Commands,
    root_entity: Entity,
    ref_id: u32,
    root_waypoints: &[MovementWaypoint],
    move_speed: f32,
) {
    let Some(base_position) =
        queries
            .set
            .p2()
            .iter()
            .find_map(|(entity, transform, _, _, _, _)| {
                (entity == root_entity).then_some(transform.translation.truncate())
            })
    else {
        return;
    };

    for (entity, transform, layer_ref, _, _, layer_resume) in &mut queries.set.p2() {
        if layer_ref.0 != ref_id {
            continue;
        }
        let waypoints = layer_resume.map_or_else(
            || {
                let offset = transform.translation.truncate() - base_position;
                root_waypoints
                    .iter()
                    .copied()
                    .map(|waypoint| waypoint.with_position(waypoint.position + offset))
                    .collect()
            },
            |resume| resume.0.clone(),
        );
        let mut entity_commands = commands.entity(entity);
        entity_commands.remove::<RepairResumeWaypoints>();
        if waypoints.is_empty() || move_speed <= 0.0 {
            entity_commands.remove::<MovementPath>();
        } else {
            entity_commands.insert(MovementPath::from_typed(waypoints, move_speed));
        }
    }
}

fn point_in_repairable_rect(
    point: Vec2,
    center: Vec2,
    size: Vec2,
    bridge: Option<BridgeFootprint>,
) -> bool {
    let Some(bridge) = bridge else {
        return point_in_object_rect(point, center, size);
    };
    buildings::bridge_world_rect(bridge).is_some_and(|rect| {
        point.x >= rect.min_x
            && point.x <= rect.max_x
            && point.y >= rect.min_y
            && point.y <= rect.max_y
    })
}

fn point_in_object_rect(point: Vec2, center: Vec2, size: Vec2) -> bool {
    let half = size.max(Vec2::splat(TILE_SIZE)) * 0.5;
    point.x >= center.x - half.x
        && point.x <= center.x + half.x
        && point.y >= center.y - half.y
        && point.y <= center.y + half.y
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::BuildingType;

    #[test]
    fn bridge_repair_hit_rect_uses_full_footprint() {
        let bridge = BridgeFootprint {
            x: 2,
            y: 3,
            building: BuildingType::BridgeHorz,
            extra_links: 2,
        };

        assert!(point_in_repairable_rect(
            Vec2::new(120.0, -80.0),
            Vec2::ZERO,
            Vec2::splat(18.0),
            Some(bridge)
        ));
        assert!(!point_in_repairable_rect(
            Vec2::new(160.0, -80.0),
            Vec2::ZERO,
            Vec2::splat(18.0),
            Some(bridge)
        ));
        assert_eq!(crate::buildings::BRIDGE_REVIVE_RERENDER_DELAY, 2.25);
    }

    #[test]
    fn crane_repair_waypoint_action_matches_source_stage_rules() {
        assert_eq!(
            source_crane_repair_waypoint_action(CraneRepairStage::GotoEntrance, false, None),
            SourceCraneRepairWaypointAction::KillWaypoint
        );
        assert_eq!(
            source_crane_repair_waypoint_action(CraneRepairStage::GotoEntrance, true, Some(false),),
            SourceCraneRepairWaypointAction::KillWaypoint
        );
        assert_eq!(
            source_crane_repair_waypoint_action(CraneRepairStage::GotoEntrance, true, Some(true),),
            SourceCraneRepairWaypointAction::KeepMoving
        );
        assert_eq!(
            source_crane_repair_waypoint_action(CraneRepairStage::GotoEntrance, false, Some(true),),
            SourceCraneRepairWaypointAction::StageEnterBuilding
        );
        assert_eq!(
            source_crane_repair_waypoint_action(
                CraneRepairStage::EnterBuilding,
                false,
                Some(false),
            ),
            SourceCraneRepairWaypointAction::StageExitBuilding
        );
        assert_eq!(
            source_crane_repair_waypoint_action(CraneRepairStage::EnterBuilding, false, Some(true),),
            SourceCraneRepairWaypointAction::StageExitBuilding
        );
        assert_eq!(
            source_crane_repair_waypoint_action(CraneRepairStage::ExitBuilding, true, Some(false)),
            SourceCraneRepairWaypointAction::KeepMoving
        );
        assert_eq!(
            source_crane_repair_waypoint_action(CraneRepairStage::ExitBuilding, false, Some(true)),
            SourceCraneRepairWaypointAction::FinishAutoRepair
        );
    }

    #[test]
    fn unit_repair_waypoint_action_matches_source_stage_rules() {
        assert_eq!(
            source_unit_repair_waypoint_action(
                UnitRepairStage::GotoEntrance,
                true,
                true,
                Some(true),
                false,
            ),
            SourceUnitRepairWaypointAction::KeepMoving
        );
        assert_eq!(
            source_unit_repair_waypoint_action(
                UnitRepairStage::GotoEntrance,
                false,
                true,
                Some(true),
                false,
            ),
            SourceUnitRepairWaypointAction::StageWait
        );
        assert_eq!(
            source_unit_repair_waypoint_action(
                UnitRepairStage::Wait,
                false,
                true,
                Some(true),
                true,
            ),
            SourceUnitRepairWaypointAction::KeepWaiting
        );
        assert_eq!(
            source_unit_repair_waypoint_action(
                UnitRepairStage::Wait,
                false,
                true,
                Some(true),
                false,
            ),
            SourceUnitRepairWaypointAction::StageEnterBuilding
        );
        assert_eq!(
            source_unit_repair_waypoint_action(
                UnitRepairStage::EnterBuilding,
                false,
                true,
                Some(true),
                false,
            ),
            SourceUnitRepairWaypointAction::StartRepairing
        );
        assert_eq!(
            source_unit_repair_waypoint_action(
                UnitRepairStage::EnterBuilding,
                true,
                true,
                Some(true),
                true,
            ),
            SourceUnitRepairWaypointAction::StageExitBuilding
        );
        assert_eq!(
            source_unit_repair_waypoint_action(
                UnitRepairStage::EnterBuilding,
                false,
                true,
                Some(false),
                false,
            ),
            SourceUnitRepairWaypointAction::StageExitBuilding
        );
        assert_eq!(
            source_unit_repair_waypoint_action(
                UnitRepairStage::ExitBuilding,
                false,
                true,
                Some(true),
                false,
            ),
            SourceUnitRepairWaypointAction::StageWait
        );
        assert_eq!(
            source_unit_repair_waypoint_action(UnitRepairStage::Wait, false, true, None, false,),
            SourceUnitRepairWaypointAction::KillWaypoint
        );
    }

    #[test]
    fn repair_deadline_advances_but_source_driver_attack_time_is_preserved() {
        let mut occupancy = RepairBuildingOccupancy {
            unit: ObjectKind::Vehicle(crate::original::objects::VehicleType::Light),
            driver: Some(DriverHealth::with_driver_states(
                crate::original::objects::RobotType::Sniper,
                vec![37.0],
                vec![3.0],
            )),
            center_point: Vec2::new(100.0, -200.0),
            entrance_point: Vec2::new(120.0, -240.0),
            resume_waypoints: Vec::new(),
            remaining: buildings::REPAIR_BUILDING_SECONDS,
        };

        assert!(!advance_repair_building_occupancy(
            &mut occupancy,
            6.0,
            true,
        ));
        assert_eq!(occupancy.remaining, -1.0);
        let driver = occupancy.driver.as_ref().unwrap();
        assert_eq!(driver.driver_healths, vec![37.0]);
        assert_eq!(driver.next_attack_cooldowns, vec![3.0]);
        assert!(advance_repair_building_occupancy(
            &mut occupancy,
            0.0,
            false,
        ));
    }

    #[test]
    fn repaired_vehicle_keeps_stored_drivers_or_source_initial_drivers_when_empty() {
        let unit = ObjectKind::Vehicle(crate::original::objects::VehicleType::Light);
        let stored = DriverHealth::with_driver_states(
            crate::original::objects::RobotType::Sniper,
            vec![31.0],
            vec![0.75],
        );
        let kept = repaired_driver_state(unit, TeamType::Red, Some(stored.clone())).unwrap();
        assert_eq!(
            kept.driver_kind,
            crate::original::objects::RobotType::Sniper
        );
        assert_eq!(kept.driver_healths, vec![31.0]);
        assert_eq!(kept.next_attack_cooldowns, vec![0.75]);

        let initial = repaired_driver_state(unit, TeamType::Red, None).unwrap();
        assert_eq!(
            initial.driver_kind,
            crate::original::objects::RobotType::Grunt
        );
        assert_eq!(initial.driver_healths.len(), 1);
        assert!(repaired_driver_state(unit, TeamType::Null, None).is_none());
    }
}
