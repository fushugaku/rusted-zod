use bevy::prelude::*;

use crate::{
    components::*,
    constants::TILE_SIZE,
    original::{
        objects::{BuildingType, ObjectKind, VehicleType},
        types::TeamType,
    },
};

const REPAIR_BUILDING_SECONDS: f32 = 5.0;

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
    pub(crate) target_top_left_map: Vec2,
    pub(crate) target_size: Vec2,
    pub(crate) target_is_bridge: bool,
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
                && matches!(object.kind, ObjectKind::Vehicle(VehicleType::Crane))
                && team.0 == repairer_team
                && team.0 != TeamType::Null
                && !stats.destroyed()
                && stats.move_speed > 0.0)
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
                && can_be_repaired_unit(object.kind, team.0, *stats)
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
            if !can_be_repaired_by_crane(object.kind, team.0, repairer_team, *stats) {
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
            let (entrance_point, center_point) = crane_repair_points(
                position,
                selectable.selection_size,
                object.kind,
                bridge.copied(),
                repairer_positions,
            )?;
            let (target_top_left_map, target_size) = crane_repair_target_map_bounds(
                position,
                selectable.selection_size,
                bridge.copied(),
            );
            Some(CraneRepairTargetInfo {
                ref_id: object.ref_id,
                entrance_point,
                center_point,
                target_top_left_map,
                target_size,
                target_is_bridge: bridge.is_some(),
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
            if !can_repair_unit(object.kind, team.0, repairer_team, *stats) {
                return None;
            }
            let position = transform.translation.truncate();
            if !point_in_object_rect(world_pos, position, selectable.selection_size) {
                return None;
            }
            let (entrance_point, center_point) =
                repair_building_points(position, selectable.selection_size, object.kind)?;
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

pub(crate) fn can_be_repaired_by_crane(
    kind: ObjectKind,
    target_team: TeamType,
    repairer_team: TeamType,
    stats: ObjectStats,
) -> bool {
    matches!(
        kind,
        ObjectKind::Building(
            BuildingType::Radar
                | BuildingType::Repair
                | BuildingType::RobotFactory
                | BuildingType::VehicleFactory
        ) | ObjectKind::Bridge(BuildingType::BridgeVert | BuildingType::BridgeHorz)
    ) && (target_team == TeamType::Null || target_team == repairer_team)
        && stats.destroyed()
}

pub(crate) fn can_be_repaired_unit(kind: ObjectKind, team: TeamType, stats: ObjectStats) -> bool {
    matches!(kind, ObjectKind::Vehicle(_))
        && team != TeamType::Null
        && !stats.destroyed()
        && stats.health < stats.max_health
}

pub(crate) fn can_repair_unit(
    kind: ObjectKind,
    building_team: TeamType,
    unit_team: TeamType,
    stats: ObjectStats,
) -> bool {
    matches!(kind, ObjectKind::Building(BuildingType::Repair))
        && building_team != TeamType::Null
        && building_team == unit_team
        && !stats.destroyed()
}

pub(crate) fn process_repair_targets(
    mut commands: Commands,
    time: Res<Time>,
    mut queries: ParamSet<(
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &ObjectStats,
            Option<&CraneRepairTarget>,
            Option<&UnitRepairTarget>,
            Option<&RepairingUnit>,
            Option<&MovementPath>,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &MapGridPosition,
            &BuildingLevel,
            Option<&BridgeFootprint>,
            &mut ObjectStats,
        )>,
        Query<(
            Entity,
            &Transform,
            &ObjectLayerRef,
            Option<&mut Sprite>,
            &mut Visibility,
        )>,
    )>,
) {
    let mut busy_repair_buildings: Vec<u32> = queries
        .p0()
        .iter()
        .filter_map(|(_, _, _, _, _, _, repairing, _)| {
            repairing.map(|repairing| repairing.building_ref_id)
        })
        .collect();

    let steps: Vec<RepairStep> = queries
        .p0()
        .iter()
        .filter_map(
            |(entity, object, team, stats, crane, unit, repairing, movement)| {
                if let Some(repairing) = repairing {
                    return Some(RepairStep::RepairingUnit {
                        entity,
                        ref_id: object.ref_id,
                        stats: *stats,
                        building_ref_id: repairing.building_ref_id,
                        remaining: repairing.remaining,
                        exit_point: repairing.exit_point,
                        move_speed: stats.move_speed,
                    });
                }
                if movement.is_some() {
                    return None;
                }
                if let Some(target) = crane {
                    return Some(RepairStep::Crane {
                        entity,
                        ref_id: object.ref_id,
                        team: team.0,
                        stats: *stats,
                        target: *target,
                    });
                }
                unit.map(|target| RepairStep::Unit {
                    entity,
                    ref_id: object.ref_id,
                    kind: object.kind,
                    team: team.0,
                    stats: *stats,
                    target: *target,
                })
            },
        )
        .collect();

    for step in steps {
        match step {
            RepairStep::Crane {
                entity,
                ref_id,
                team,
                stats,
                target,
            } => process_crane_step(
                &mut commands,
                &mut queries,
                entity,
                ref_id,
                team,
                stats,
                target,
            ),
            RepairStep::Unit {
                entity,
                ref_id,
                team,
                kind,
                stats,
                target,
            } => process_unit_repair_step(
                &mut commands,
                &mut queries,
                &mut busy_repair_buildings,
                entity,
                ref_id,
                kind,
                team,
                stats,
                target,
            ),
            RepairStep::RepairingUnit {
                entity,
                ref_id,
                stats,
                building_ref_id,
                remaining,
                exit_point,
                move_speed,
            } => process_repairing_unit_step(
                &mut commands,
                &mut queries,
                time.delta_secs(),
                entity,
                ref_id,
                stats,
                building_ref_id,
                remaining,
                exit_point,
                move_speed,
            ),
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
    },
    Unit {
        entity: Entity,
        ref_id: u32,
        kind: ObjectKind,
        team: TeamType,
        stats: ObjectStats,
        target: UnitRepairTarget,
    },
    RepairingUnit {
        entity: Entity,
        ref_id: u32,
        stats: ObjectStats,
        building_ref_id: u32,
        remaining: f32,
        exit_point: Vec2,
        move_speed: f32,
    },
}

fn process_crane_step(
    commands: &mut Commands,
    queries: &mut ParamSet<(
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &ObjectStats,
            Option<&CraneRepairTarget>,
            Option<&UnitRepairTarget>,
            Option<&RepairingUnit>,
            Option<&MovementPath>,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &MapGridPosition,
            &BuildingLevel,
            Option<&BridgeFootprint>,
            &mut ObjectStats,
        )>,
        Query<(
            Entity,
            &Transform,
            &ObjectLayerRef,
            Option<&mut Sprite>,
            &mut Visibility,
        )>,
    )>,
    entity: Entity,
    ref_id: u32,
    team: TeamType,
    stats: ObjectStats,
    target: CraneRepairTarget,
) {
    if stats.destroyed() || stats.move_speed <= 0.0 {
        commands.entity(entity).remove::<CraneRepairTarget>();
        return;
    }

    match target.stage {
        CraneRepairStage::GotoEntrance => {
            if !target_can_be_crane_repaired(queries, target.ref_id, team) {
                commands.entity(entity).remove::<CraneRepairTarget>();
                return;
            }
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
        CraneRepairStage::EnterBuilding => {
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
        CraneRepairStage::ExitBuilding => {
            set_auto_repair_now(commands, queries, target.ref_id);
            commands.entity(entity).remove::<CraneRepairTarget>();
        }
    }
}

fn process_unit_repair_step(
    commands: &mut Commands,
    queries: &mut ParamSet<(
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &ObjectStats,
            Option<&CraneRepairTarget>,
            Option<&UnitRepairTarget>,
            Option<&RepairingUnit>,
            Option<&MovementPath>,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &MapGridPosition,
            &BuildingLevel,
            Option<&BridgeFootprint>,
            &mut ObjectStats,
        )>,
        Query<(
            Entity,
            &Transform,
            &ObjectLayerRef,
            Option<&mut Sprite>,
            &mut Visibility,
        )>,
    )>,
    busy_repair_buildings: &mut Vec<u32>,
    entity: Entity,
    ref_id: u32,
    kind: ObjectKind,
    team: TeamType,
    stats: ObjectStats,
    target: UnitRepairTarget,
) {
    if !can_be_repaired_unit(kind, team, stats) {
        commands.entity(entity).remove::<UnitRepairTarget>();
        return;
    }

    let target_busy = busy_repair_buildings.contains(&target.ref_id);
    let target_can_repair = target_can_repair_unit(queries, target.ref_id, team);
    if !target_can_repair {
        commands.entity(entity).remove::<UnitRepairTarget>();
        return;
    }

    match target.stage {
        UnitRepairStage::GotoEntrance | UnitRepairStage::Wait => {
            if target_busy {
                commands.entity(entity).insert(UnitRepairTarget {
                    stage: UnitRepairStage::Wait,
                    ..target
                });
                return;
            }
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
        UnitRepairStage::EnterBuilding => {
            if target_busy {
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
            set_layers_visibility_for_ref(queries, ref_id, Visibility::Hidden);
            commands
                .entity(entity)
                .remove::<UnitRepairTarget>()
                .remove::<AttackTarget>()
                .insert(RepairingUnit {
                    building_ref_id: target.ref_id,
                    exit_point: target.entrance_point,
                    remaining: REPAIR_BUILDING_SECONDS,
                });
            busy_repair_buildings.push(target.ref_id);
        }
        UnitRepairStage::ExitBuilding => {
            commands.entity(entity).remove::<UnitRepairTarget>();
        }
    }
}

fn process_repairing_unit_step(
    commands: &mut Commands,
    queries: &mut ParamSet<(
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &ObjectStats,
            Option<&CraneRepairTarget>,
            Option<&UnitRepairTarget>,
            Option<&RepairingUnit>,
            Option<&MovementPath>,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &MapGridPosition,
            &BuildingLevel,
            Option<&BridgeFootprint>,
            &mut ObjectStats,
        )>,
        Query<(
            Entity,
            &Transform,
            &ObjectLayerRef,
            Option<&mut Sprite>,
            &mut Visibility,
        )>,
    )>,
    delta_secs: f32,
    entity: Entity,
    ref_id: u32,
    stats: ObjectStats,
    building_ref_id: u32,
    remaining: f32,
    exit_point: Vec2,
    move_speed: f32,
) {
    let remaining = remaining - delta_secs;
    if remaining > 0.0 {
        commands.entity(entity).insert(RepairingUnit {
            building_ref_id,
            exit_point,
            remaining,
        });
        return;
    }

    set_layers_visibility_for_ref(queries, ref_id, Visibility::Visible);
    commands
        .entity(entity)
        .remove::<RepairingUnit>()
        .insert(ObjectStats {
            health: stats.max_health,
            ..stats
        })
        .insert(UnitRepairTarget {
            ref_id: 0,
            stage: UnitRepairStage::ExitBuilding,
            center_point: exit_point,
            entrance_point: exit_point,
        });
    insert_layer_paths_for_ref(queries, commands, ref_id, exit_point, move_speed);
}

fn target_can_be_crane_repaired(
    queries: &mut ParamSet<(
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &ObjectStats,
            Option<&CraneRepairTarget>,
            Option<&UnitRepairTarget>,
            Option<&RepairingUnit>,
            Option<&MovementPath>,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &MapGridPosition,
            &BuildingLevel,
            Option<&BridgeFootprint>,
            &mut ObjectStats,
        )>,
        Query<(
            Entity,
            &Transform,
            &ObjectLayerRef,
            Option<&mut Sprite>,
            &mut Visibility,
        )>,
    )>,
    target_ref_id: u32,
    repairer_team: TeamType,
) -> bool {
    queries
        .p1()
        .iter()
        .any(|(_, object, team, _, _, _, stats)| {
            object.ref_id == target_ref_id
                && can_be_repaired_by_crane(object.kind, team.0, repairer_team, *stats)
        })
}

fn set_auto_repair_now(
    commands: &mut Commands,
    queries: &mut ParamSet<(
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &ObjectStats,
            Option<&CraneRepairTarget>,
            Option<&UnitRepairTarget>,
            Option<&RepairingUnit>,
            Option<&MovementPath>,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &MapGridPosition,
            &BuildingLevel,
            Option<&BridgeFootprint>,
            &mut ObjectStats,
        )>,
        Query<(
            Entity,
            &Transform,
            &ObjectLayerRef,
            Option<&mut Sprite>,
            &mut Visibility,
        )>,
    )>,
    target_ref_id: u32,
) {
    let Some((entity, _, _, _, _, _, _)) = queries
        .p1()
        .iter_mut()
        .find(|(_, object, _, _, _, _, _)| object.ref_id == target_ref_id)
    else {
        return;
    };
    commands.entity(entity).insert(AutoRepair { timer: 0.0 });
}

fn target_can_repair_unit(
    queries: &mut ParamSet<(
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &ObjectStats,
            Option<&CraneRepairTarget>,
            Option<&UnitRepairTarget>,
            Option<&RepairingUnit>,
            Option<&MovementPath>,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &MapGridPosition,
            &BuildingLevel,
            Option<&BridgeFootprint>,
            &mut ObjectStats,
        )>,
        Query<(
            Entity,
            &Transform,
            &ObjectLayerRef,
            Option<&mut Sprite>,
            &mut Visibility,
        )>,
    )>,
    target_ref_id: u32,
    unit_team: TeamType,
) -> bool {
    queries
        .p1()
        .iter()
        .any(|(_, object, team, _, _, _, stats)| {
            object.ref_id == target_ref_id
                && can_repair_unit(object.kind, team.0, unit_team, *stats)
        })
}

fn insert_layer_paths_for_ref(
    queries: &mut ParamSet<(
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &ObjectStats,
            Option<&CraneRepairTarget>,
            Option<&UnitRepairTarget>,
            Option<&RepairingUnit>,
            Option<&MovementPath>,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &MapGridPosition,
            &BuildingLevel,
            Option<&BridgeFootprint>,
            &mut ObjectStats,
        )>,
        Query<(
            Entity,
            &Transform,
            &ObjectLayerRef,
            Option<&mut Sprite>,
            &mut Visibility,
        )>,
    )>,
    commands: &mut Commands,
    ref_id: u32,
    target: Vec2,
    move_speed: f32,
) {
    let Some(base_position) = queries
        .p2()
        .iter()
        .find_map(|(_, transform, layer_ref, _, _)| {
            (layer_ref.0 == ref_id).then_some(transform.translation.truncate())
        })
    else {
        return;
    };

    for (entity, transform, layer_ref, _, _) in &mut queries.p2() {
        if layer_ref.0 == ref_id {
            let layer_offset = transform.translation.truncate() - base_position;
            commands
                .entity(entity)
                .insert(MovementPath::new(vec![target + layer_offset], move_speed));
        }
    }
}

fn set_layers_visibility_for_ref(
    queries: &mut ParamSet<(
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &ObjectStats,
            Option<&CraneRepairTarget>,
            Option<&UnitRepairTarget>,
            Option<&RepairingUnit>,
            Option<&MovementPath>,
        )>,
        Query<(
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &MapGridPosition,
            &BuildingLevel,
            Option<&BridgeFootprint>,
            &mut ObjectStats,
        )>,
        Query<(
            Entity,
            &Transform,
            &ObjectLayerRef,
            Option<&mut Sprite>,
            &mut Visibility,
        )>,
    )>,
    ref_id: u32,
    visibility: Visibility,
) {
    for (_, _, layer_ref, _, mut layer_visibility) in &mut queries.p2() {
        if layer_ref.0 == ref_id {
            *layer_visibility = visibility;
        }
    }
}

fn crane_repair_points(
    center: Vec2,
    size: Vec2,
    kind: ObjectKind,
    bridge: Option<BridgeFootprint>,
    repairer_positions: &[Vec2],
) -> Option<(Vec2, Vec2)> {
    if let Some(bridge) = bridge {
        return bridge_crane_repair_points(bridge, repairer_positions);
    }

    let (entrance, center_point) = match kind {
        ObjectKind::Building(BuildingType::Radar) => {
            (Vec2::new(28.0, size.y + 32.0), Vec2::new(28.0, 24.0))
        }
        ObjectKind::Building(BuildingType::Repair) => {
            (Vec2::new(32.0, size.y + 32.0), Vec2::new(32.0, 32.0))
        }
        ObjectKind::Building(BuildingType::RobotFactory) => {
            (Vec2::new(35.0, size.y + 32.0), Vec2::new(35.0, 32.0))
        }
        ObjectKind::Building(BuildingType::VehicleFactory) => {
            (Vec2::new(31.0, size.y + 32.0), Vec2::new(31.0, 32.0))
        }
        _ => return None,
    };
    Some((
        building_local_point(center, size, entrance),
        building_local_point(center, size, center_point),
    ))
}

fn crane_repair_target_map_bounds(
    center: Vec2,
    size: Vec2,
    bridge: Option<BridgeFootprint>,
) -> (Vec2, Vec2) {
    if let Some(bridge) = bridge {
        if let Some((x, y, width, height)) =
            PassabilityGrid::bridge_bounds(bridge.x, bridge.y, bridge.building, bridge.extra_links)
        {
            return (
                Vec2::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
                Vec2::new(width as f32 * TILE_SIZE, height as f32 * TILE_SIZE),
            );
        }
    }

    (world_object_top_left_map(center, size), size)
}

fn world_object_top_left_map(center: Vec2, size: Vec2) -> Vec2 {
    Vec2::new(center.x - size.x * 0.5, -center.y - size.y * 0.5)
}

fn bridge_crane_repair_points(
    bridge: BridgeFootprint,
    repairer_positions: &[Vec2],
) -> Option<(Vec2, Vec2)> {
    let (x, y, width, height) =
        PassabilityGrid::bridge_bounds(bridge.x, bridge.y, bridge.building, bridge.extra_links)?;
    let left = x as f32 * TILE_SIZE;
    let top = y as f32 * TILE_SIZE;
    let width_pix = width as f32 * TILE_SIZE;
    let height_pix = height as f32 * TILE_SIZE;
    let center = map_point_to_world(Vec2::new(left + width_pix * 0.5, top + height_pix * 0.5));
    let entrances = match bridge.building {
        BuildingType::BridgeVert => [
            map_point_to_world(Vec2::new(left + 32.0, top - 32.0)),
            map_point_to_world(Vec2::new(left + 32.0, top + height_pix + 32.0)),
        ],
        BuildingType::BridgeHorz => [
            map_point_to_world(Vec2::new(left - 31.0, top + 31.0)),
            map_point_to_world(Vec2::new(left + width_pix + 32.0, top + 32.0)),
        ],
        _ => return None,
    };
    let entrance = repairer_positions
        .iter()
        .flat_map(|repairer| {
            entrances
                .into_iter()
                .map(move |entrance| (entrance, entrance.distance_squared(*repairer)))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map_or(entrances[0], |(entrance, _)| entrance);
    Some((entrance, center))
}

fn repair_building_points(center: Vec2, size: Vec2, kind: ObjectKind) -> Option<(Vec2, Vec2)> {
    if !matches!(kind, ObjectKind::Building(BuildingType::Repair)) {
        return None;
    }
    Some((
        building_local_point(center, size, Vec2::new(32.0, size.y + 32.0)),
        building_local_point(center, size, Vec2::new(32.0, 32.0)),
    ))
}

fn building_local_point(center: Vec2, size: Vec2, local: Vec2) -> Vec2 {
    let top_left = Vec2::new(center.x - size.x * 0.5, center.y + size.y * 0.5);
    Vec2::new(top_left.x + local.x, top_left.y - local.y)
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
    let Some((x, y, width, height)) =
        PassabilityGrid::bridge_bounds(bridge.x, bridge.y, bridge.building, bridge.extra_links)
    else {
        return false;
    };
    let min_x = x as f32 * TILE_SIZE;
    let max_x = x.saturating_add(width) as f32 * TILE_SIZE;
    let min_y = -(y.saturating_add(height) as f32 * TILE_SIZE);
    let max_y = -(y as f32 * TILE_SIZE);
    point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
}

fn map_point_to_world(point: Vec2) -> Vec2 {
    Vec2::new(point.x, -point.y)
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

    #[test]
    fn crane_repair_rules_match_original_building_gate() {
        let mut stats = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Radar), 0);
        assert!(can_be_repaired_by_crane(
            ObjectKind::Building(BuildingType::Radar),
            TeamType::Red,
            TeamType::Red,
            stats
        ));
        assert!(can_be_repaired_by_crane(
            ObjectKind::Building(BuildingType::Repair),
            TeamType::Null,
            TeamType::Red,
            stats
        ));
        assert!(!can_be_repaired_by_crane(
            ObjectKind::Building(BuildingType::FortFront),
            TeamType::Red,
            TeamType::Red,
            stats
        ));
        assert!(can_be_repaired_by_crane(
            ObjectKind::Bridge(BuildingType::BridgeVert),
            TeamType::Null,
            TeamType::Red,
            stats
        ));
        stats.health = stats.max_health;
        assert!(!can_be_repaired_by_crane(
            ObjectKind::Building(BuildingType::Radar),
            TeamType::Red,
            TeamType::Red,
            stats
        ));
    }

    #[test]
    fn repair_building_points_match_original_offsets() {
        let center = Vec2::new(40.0, -32.0);
        let size = Vec2::new(80.0, 64.0);
        assert_eq!(
            repair_building_points(center, size, ObjectKind::Building(BuildingType::Repair)),
            Some((Vec2::new(32.0, -96.0), Vec2::new(32.0, -32.0)))
        );
    }

    #[test]
    fn bridge_crane_repair_points_match_original_dual_entrances() {
        let bridge = BridgeFootprint {
            x: 2,
            y: 3,
            building: BuildingType::BridgeVert,
            extra_links: 1,
        };

        assert_eq!(
            bridge_crane_repair_points(bridge, &[Vec2::new(64.0, -10.0)]),
            Some((Vec2::new(64.0, -16.0), Vec2::new(64.0, -96.0)))
        );
        assert_eq!(
            bridge_crane_repair_points(bridge, &[Vec2::new(64.0, -190.0)]),
            Some((Vec2::new(64.0, -176.0), Vec2::new(64.0, -96.0)))
        );

        let horizontal = BridgeFootprint {
            x: 2,
            y: 3,
            building: BuildingType::BridgeHorz,
            extra_links: 1,
        };
        assert_eq!(
            bridge_crane_repair_points(horizontal, &[Vec2::new(0.0, -80.0)]),
            Some((Vec2::new(1.0, -79.0), Vec2::new(80.0, -80.0)))
        );
    }

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
}
