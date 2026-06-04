use bevy::prelude::*;

use crate::{
    components::*,
    original::{
        objects::{BuildingType, ObjectKind, RobotType, VehicleType},
        types::TeamType,
    },
    production::production_world_points,
    render::atlas::GameAtlases,
    robot_groups::{
        RobotGroupMemberSnapshot, remap_selected_refs_for_group_promotions,
        robot_group_promotions_for_removed_refs,
    },
};

#[derive(Clone, Copy)]
pub(crate) struct EnterTargetInfo {
    pub(crate) ref_id: u32,
    pub(crate) position: Vec2,
    pub(crate) waypoint: Vec2,
}

#[derive(Clone, Copy)]
pub(crate) struct EnteringRobot {
    pub(crate) ref_id: u32,
    pub(crate) kind: ObjectKind,
    pub(crate) position: Vec2,
    pub(crate) move_speed: f32,
}

pub(crate) struct EnterCommand {
    pub(crate) target: EnterTargetInfo,
    pub(crate) robots: Vec<EnteringRobot>,
}

pub(crate) struct EnterFortCommand {
    pub(crate) target: EnterFortTargetInfo,
    pub(crate) robots: Vec<EnteringRobot>,
}

#[derive(Clone, Copy)]
pub(crate) struct EnterFortTargetInfo {
    pub(crate) ref_id: u32,
    pub(crate) position: Vec2,
    pub(crate) inside_point: Vec2,
    pub(crate) exit_point: Vec2,
}

pub(crate) fn can_be_entered(kind: ObjectKind, team: TeamType, stats: ObjectStats) -> bool {
    team == TeamType::Null
        && !stats.destroyed()
        && matches!(kind, ObjectKind::Vehicle(_) | ObjectKind::Cannon(_))
}

fn can_enter_target(kind: ObjectKind, team: TeamType, stats: ObjectStats) -> bool {
    team != TeamType::Null && !stats.destroyed() && matches!(kind, ObjectKind::Robot(_))
}

fn can_enter_fort_unit(kind: ObjectKind, team: TeamType, stats: ObjectStats) -> bool {
    team != TeamType::Null
        && !stats.destroyed()
        && stats.move_speed > 0.0
        && matches!(kind, ObjectKind::Robot(_) | ObjectKind::Vehicle(_))
}

pub(crate) fn can_enter_fort(
    kind: ObjectKind,
    fort_team: TeamType,
    entering_team: TeamType,
    stats: ObjectStats,
) -> bool {
    matches!(
        kind,
        ObjectKind::Building(BuildingType::FortFront | BuildingType::FortBack)
    ) && fort_team != entering_team
        && !stats.destroyed()
}

pub(crate) fn enter_target_for_right_click(
    world_pos: Vec2,
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
) -> Option<EnterCommand> {
    let target = enterable_at_position(world_pos, object_query)?;
    let robots: Vec<EnteringRobot> = object_query
        .iter()
        .filter_map(|(object, transform, _, team, stats, _, _)| {
            (selected_refs.contains(&object.ref_id)
                && can_enter_target(object.kind, team.0, *stats))
            .then_some(EnteringRobot {
                ref_id: object.ref_id,
                kind: object.kind,
                position: transform.translation.truncate(),
                move_speed: stats.move_speed,
            })
        })
        .collect();

    (!robots.is_empty()).then_some(EnterCommand { target, robots })
}

pub(crate) fn enter_fort_target_for_right_click(
    world_pos: Vec2,
    selected_refs: &[u32],
    entering_team: TeamType,
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&RobotGroup>,
        Option<&BridgeFootprint>,
    )>,
) -> Option<EnterFortCommand> {
    let target = enterable_fort_at_position(world_pos, entering_team, object_query)?;
    let robots: Vec<EnteringRobot> = object_query
        .iter()
        .filter_map(|(object, transform, _, team, stats, _, _)| {
            (selected_refs.contains(&object.ref_id)
                && team.0 == entering_team
                && can_enter_fort_unit(object.kind, team.0, *stats))
            .then_some(EnteringRobot {
                ref_id: object.ref_id,
                kind: object.kind,
                position: transform.translation.truncate(),
                move_speed: stats.move_speed,
            })
        })
        .collect();

    (!robots.is_empty()).then_some(EnterFortCommand { target, robots })
}

fn enterable_fort_at_position(
    world_pos: Vec2,
    entering_team: TeamType,
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        &ObjectTeam,
        &ObjectStats,
        Option<&RobotGroup>,
        Option<&BridgeFootprint>,
    )>,
) -> Option<EnterFortTargetInfo> {
    object_query
        .iter()
        .filter_map(|(object, transform, selectable, team, stats, _, _)| {
            if !can_enter_fort(object.kind, team.0, entering_team, *stats) {
                return None;
            }

            let ObjectKind::Building(building) = object.kind else {
                return None;
            };
            let position = transform.translation.truncate();
            if !point_in_fort_entrance_rect(
                world_pos,
                position,
                selectable.selection_size,
                building,
            ) {
                return None;
            }

            let Some((inside_point, exit_point)) =
                fort_entry_points(position, selectable.selection_size, building)
            else {
                return None;
            };

            Some(EnterFortTargetInfo {
                ref_id: object.ref_id,
                position,
                inside_point,
                exit_point,
            })
        })
        .min_by(|a, b| {
            a.position
                .distance_squared(world_pos)
                .total_cmp(&b.position.distance_squared(world_pos))
        })
}

fn enterable_at_position(
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
) -> Option<EnterTargetInfo> {
    object_query
        .iter()
        .filter_map(|(object, transform, selectable, team, stats, _, _)| {
            if !can_be_entered(object.kind, team.0, *stats) {
                return None;
            }

            let position = transform.translation.truncate();
            point_in_enter_rect(world_pos, position, selectable.selection_size).then_some(
                EnterTargetInfo {
                    ref_id: object.ref_id,
                    position,
                    waypoint: world_pos,
                },
            )
        })
        .min_by(|a, b| {
            a.position
                .distance_squared(world_pos)
                .total_cmp(&b.position.distance_squared(world_pos))
        })
}

pub(crate) fn process_enter_targets(
    mut commands: Commands,
    game_atlases: Res<GameAtlases>,
    mut selection: ResMut<SelectionState>,
    mut queries: ParamSet<(
        Query<
            (
                Entity,
                &GameObjectEntity,
                &Transform,
                &ObjectTeam,
                &ObjectStats,
                &EnterTarget,
                Option<&RobotGroup>,
            ),
            (With<EnterTarget>, Without<DestroyedObject>),
        >,
        Query<
            (
                &GameObjectEntity,
                &Transform,
                &Selectable,
                &ObjectTeam,
                &ObjectStats,
            ),
            Without<EnterTarget>,
        >,
        Query<(
            Entity,
            &ObjectLayerRef,
            Option<&GameObjectEntity>,
            Option<&mut ObjectTeam>,
            Option<&mut ObjectStats>,
            Option<&mut Sprite>,
            Option<&mut MobileSpriteLayer>,
        )>,
        Query<(Entity, &MinimapDot, &mut Sprite)>,
        Query<
            (
                &GameObjectEntity,
                &ObjectStats,
                &mut RobotGroup,
                Option<&mut GrenadeInventory>,
            ),
            Without<DestroyedObject>,
        >,
    )>,
    selection_markers: Query<(Entity, &SelectionMarker)>,
    selection_health_bars: Query<(Entity, &SelectionHealthBar)>,
) {
    let entrants: Vec<EnterRequestSnapshot> = queries
        .p0()
        .iter()
        .filter_map(|(entity, object, transform, team, stats, enter, group)| {
            if !can_enter_target(object.kind, team.0, *stats) {
                return None;
            }

            let ObjectKind::Robot(robot_kind) = object.kind else {
                return None;
            };

            Some(EnterRequestSnapshot {
                entity,
                robot_ref_id: object.ref_id,
                robot_kind,
                target_ref_id: enter.ref_id,
                leader_ref_id: group.map(|group| group.leader_ref_id),
                position: transform.translation.truncate(),
                team: team.0,
                health: stats.health,
            })
        })
        .collect();
    let targets: Vec<EnterTargetSnapshot> = queries
        .p1()
        .iter()
        .filter_map(
            |(target_object, target_transform, selectable, target_team, target_stats)| {
                if !can_be_entered(target_object.kind, target_team.0, *target_stats) {
                    return None;
                }

                Some(EnterTargetSnapshot {
                    ref_id: target_object.ref_id,
                    kind: target_object.kind,
                    position: target_transform.translation.truncate(),
                    selection_size: selectable.selection_size,
                })
            },
        )
        .collect();
    let group_members: Vec<EnterGroupMemberSnapshot> = queries
        .p4()
        .iter()
        .map(|(object, stats, group, _)| EnterGroupMemberSnapshot {
            ref_id: object.ref_id,
            leader_ref_id: group.leader_ref_id,
            health: stats.health,
            destroyed: stats.destroyed(),
        })
        .collect();

    let enter_events: Vec<EnterEvent> = entrants
        .iter()
        .filter_map(|entrant| {
            if entrant
                .leader_ref_id
                .is_some_and(|leader_ref_id| leader_ref_id != entrant.robot_ref_id)
            {
                return None;
            }

            let target = targets
                .iter()
                .find(|target| target.ref_id == entrant.target_ref_id)?;

            if !point_in_enter_rect(entrant.position, target.position, target.selection_size) {
                return None;
            }

            let removed_robots = removed_robots_for_enter(entrant, target.kind, &group_members);
            let driver_healths: Vec<f32> =
                removed_robots.iter().map(|robot| robot.health).collect();
            Some(EnterEvent {
                target_ref_id: target.ref_id,
                target_kind: target.kind,
                robot_kind: entrant.robot_kind,
                new_team: entrant.team,
                driver_health: DriverHealth::with_driver_healths(
                    entrant.robot_kind,
                    driver_healths,
                ),
                removed_robot_refs: removed_robots.iter().map(|robot| robot.ref_id).collect(),
            })
        })
        .collect();

    let removed_robot_refs: Vec<u32> = enter_events
        .iter()
        .flat_map(|event| event.removed_robot_refs.iter().copied())
        .collect();
    if !removed_robot_refs.is_empty() {
        let group_snapshots: Vec<RobotGroupMemberSnapshot> = queries
            .p4()
            .iter()
            .map(
                |(object, stats, group, inventory)| RobotGroupMemberSnapshot {
                    ref_id: object.ref_id,
                    leader_ref_id: group.leader_ref_id,
                    destroyed: stats.destroyed(),
                    grenade_amount: inventory.map_or(0, |inventory| inventory.amount),
                },
            )
            .collect();
        let group_promotions =
            robot_group_promotions_for_removed_refs(&removed_robot_refs, &group_snapshots);

        if !group_promotions.is_empty() {
            for (object, stats, mut group, inventory) in &mut queries.p4() {
                let Some(promotion) = group_promotions.iter().find(|promotion| {
                    group.leader_ref_id == promotion.old_leader_ref_id
                        && !stats.destroyed()
                        && !removed_robot_refs.contains(&object.ref_id)
                }) else {
                    continue;
                };

                group.leader_ref_id = promotion.new_leader_ref_id;
                if object.ref_id == promotion.new_leader_ref_id && promotion.grenade_amount > 0 {
                    if let Some(mut inventory) = inventory {
                        inventory.amount = promotion.grenade_amount;
                    }
                }
            }

            remap_selected_refs_for_group_promotions(
                &mut selection.selected_refs,
                &group_promotions,
            );
        }
        selection
            .selected_refs
            .retain(|ref_id| !removed_robot_refs.contains(ref_id));
    }

    for event in enter_events {
        {
            let mut layers = queries.p2();
            capture_target_layers(
                &mut commands,
                &mut layers,
                &game_atlases,
                event.target_ref_id,
                event.target_kind,
                event.robot_kind,
                event.new_team,
                event.driver_health,
            );
            for robot_ref_id in &event.removed_robot_refs {
                remove_robot_after_enter(
                    &mut commands,
                    *robot_ref_id,
                    &mut layers,
                    &selection_markers,
                    &selection_health_bars,
                );
            }
        }

        {
            let mut minimap_dots = queries.p3();
            for (entity, dot, mut sprite) in &mut minimap_dots {
                if event.removed_robot_refs.contains(&dot.ref_id) {
                    commands.entity(entity).despawn();
                    continue;
                }

                if dot.ref_id == event.target_ref_id {
                    sprite.color = event.new_team.color();
                }
            }
        }
    }

    for entrant in entrants {
        commands.entity(entrant.entity).remove::<EnterTarget>();
    }
}

pub(crate) fn process_enter_fort_targets(
    mut commands: Commands,
    entrants: Query<
        (
            Entity,
            &GameObjectEntity,
            &ObjectTeam,
            &ObjectStats,
            &EnterFortTarget,
            Option<&MovementPath>,
        ),
        (With<EnterFortTarget>, Without<DestroyedObject>),
    >,
    mut targets: Query<
        (&GameObjectEntity, &ObjectTeam, &mut ObjectStats),
        Without<EnterFortTarget>,
    >,
    layers: Query<(Entity, &Transform, &ObjectLayerRef)>,
) {
    let fort_steps: Vec<FortEnterStep> = entrants
        .iter()
        .filter_map(|(entity, object, team, stats, target, movement)| {
            if movement.is_some() || !can_enter_fort_unit(object.kind, team.0, *stats) {
                return None;
            }

            Some(FortEnterStep {
                entity,
                ref_id: object.ref_id,
                fort_ref_id: target.ref_id,
                team: team.0,
                stage: target.stage,
                inside_point: target.inside_point,
                exit_point: target.exit_point,
                move_speed: stats.move_speed,
            })
        })
        .collect();

    for step in fort_steps {
        match step.stage {
            EnterFortStage::GotoEntrance => {
                commands.entity(step.entity).insert(EnterFortTarget {
                    ref_id: step.fort_ref_id,
                    stage: EnterFortStage::EnterBuilding,
                    inside_point: step.inside_point,
                    exit_point: step.exit_point,
                });
                insert_layer_paths_for_ref(
                    &mut commands,
                    &layers,
                    step.ref_id,
                    step.inside_point,
                    step.move_speed,
                );
            }
            EnterFortStage::EnterBuilding => {
                let Some((_, _, mut fort_stats)) =
                    targets.iter_mut().find(|(object, team, stats)| {
                        object.ref_id == step.fort_ref_id
                            && can_enter_fort(object.kind, team.0, step.team, **stats)
                    })
                else {
                    commands.entity(step.entity).remove::<EnterFortTarget>();
                    continue;
                };

                fort_stats.health = 0.0;
                commands.entity(step.entity).insert(EnterFortTarget {
                    ref_id: step.fort_ref_id,
                    stage: EnterFortStage::ExitBuilding,
                    inside_point: step.inside_point,
                    exit_point: step.exit_point,
                });
                insert_layer_paths_for_ref(
                    &mut commands,
                    &layers,
                    step.ref_id,
                    step.exit_point,
                    step.move_speed,
                );
            }
            EnterFortStage::ExitBuilding => {
                commands.entity(step.entity).remove::<EnterFortTarget>();
            }
        }
    }
}

#[derive(Clone, Copy)]
struct FortEnterStep {
    entity: Entity,
    ref_id: u32,
    fort_ref_id: u32,
    team: TeamType,
    stage: EnterFortStage,
    inside_point: Vec2,
    exit_point: Vec2,
    move_speed: f32,
}

fn insert_layer_paths_for_ref(
    commands: &mut Commands,
    layers: &Query<(Entity, &Transform, &ObjectLayerRef)>,
    ref_id: u32,
    target: Vec2,
    move_speed: f32,
) {
    let Some(base_position) = layers.iter().find_map(|(_, transform, layer_ref)| {
        (layer_ref.0 == ref_id).then_some(transform.translation.truncate())
    }) else {
        return;
    };

    for (entity, transform, layer_ref) in layers {
        if layer_ref.0 == ref_id {
            let layer_offset = transform.translation.truncate() - base_position;
            commands.entity(entity).insert(
                MovementPath::new(vec![target + layer_offset], move_speed).with_run_attempt(),
            );
        }
    }
}

#[derive(Clone, Copy)]
struct EnterRequestSnapshot {
    entity: Entity,
    robot_ref_id: u32,
    robot_kind: RobotType,
    target_ref_id: u32,
    leader_ref_id: Option<u32>,
    position: Vec2,
    team: TeamType,
    health: f32,
}

#[derive(Clone, Copy)]
struct EnterTargetSnapshot {
    ref_id: u32,
    kind: ObjectKind,
    position: Vec2,
    selection_size: Vec2,
}

#[derive(Clone)]
struct EnterEvent {
    target_ref_id: u32,
    target_kind: ObjectKind,
    robot_kind: RobotType,
    new_team: TeamType,
    driver_health: DriverHealth,
    removed_robot_refs: Vec<u32>,
}

#[derive(Clone, Copy)]
struct EnterGroupMemberSnapshot {
    ref_id: u32,
    leader_ref_id: u32,
    health: f32,
    destroyed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RemovedRobotForEnter {
    ref_id: u32,
    health: f32,
}

fn removed_robots_for_enter(
    entrant: &EnterRequestSnapshot,
    target_kind: ObjectKind,
    group_members: &[EnterGroupMemberSnapshot],
) -> Vec<RemovedRobotForEnter> {
    let mut robots = vec![RemovedRobotForEnter {
        ref_id: entrant.robot_ref_id,
        health: entrant.health,
    }];
    if !matches!(target_kind, ObjectKind::Vehicle(VehicleType::Apc)) {
        return robots;
    }

    for member in group_members.iter().filter(|member| {
        member.ref_id != entrant.robot_ref_id
            && member.leader_ref_id == entrant.robot_ref_id
            && !member.destroyed
            && member.health > 0.0
    }) {
        robots.push(RemovedRobotForEnter {
            ref_id: member.ref_id,
            health: member.health,
        });
    }
    robots
}

fn apply_apc_driver_attack_stats(stats: &mut ObjectStats, robot_kind: RobotType) {
    let driver_stats = ObjectStats::from_kind(ObjectKind::Robot(robot_kind), 100);
    stats.attack_radius = driver_stats.attack_radius;
    stats.attack_damage = driver_stats.attack_damage;
    stats.damage_chance = driver_stats.damage_chance;
    stats.damage_radius = driver_stats.damage_radius;
    stats.missile_speed = driver_stats.missile_speed;
    stats.attack_speed = driver_stats.attack_speed;
    stats.snipe_chance = driver_stats.snipe_chance;
}

fn point_in_enter_rect(point: Vec2, center: Vec2, size: Vec2) -> bool {
    let half = size.max(Vec2::splat(16.0)) * 0.5;
    point.x >= center.x - half.x
        && point.x <= center.x + half.x
        && point.y >= center.y - half.y
        && point.y <= center.y + half.y
}

pub(crate) fn point_in_fort_entrance_rect(
    point: Vec2,
    center: Vec2,
    size: Vec2,
    building: BuildingType,
) -> bool {
    let top_left = Vec2::new(center.x - size.x * 0.5, center.y + size.y * 0.5);
    let (x, y, w, h) = fort_entrance_rect(building);
    let min_x = top_left.x + x;
    let max_x = min_x + w;
    let max_y = top_left.y - y;
    let min_y = max_y - h;

    point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
}

fn fort_entry_points(center: Vec2, size: Vec2, building: BuildingType) -> Option<(Vec2, Vec2)> {
    let top_left = Vec2::new(center.x - size.x * 0.5, center.y + size.y * 0.5);
    let tile_x = (top_left.x / 16.0).round().max(0.0) as u16;
    let tile_y = (-top_left.y / 16.0).round().max(0.0) as u16;
    production_world_points(building, tile_x, tile_y)
}

fn fort_entrance_rect(building: BuildingType) -> (f32, f32, f32, f32) {
    match building {
        BuildingType::FortFront => (64.0, 32.0, 32.0, 96.0),
        BuildingType::FortBack => (64.0, 16.0, 32.0, 64.0),
        _ => (0.0, 0.0, 0.0, 0.0),
    }
}

fn capture_target_layers(
    commands: &mut Commands,
    layers: &mut Query<(
        Entity,
        &ObjectLayerRef,
        Option<&GameObjectEntity>,
        Option<&mut ObjectTeam>,
        Option<&mut ObjectStats>,
        Option<&mut Sprite>,
        Option<&mut MobileSpriteLayer>,
    )>,
    game_atlases: &GameAtlases,
    target_ref_id: u32,
    target_kind: ObjectKind,
    robot_kind: RobotType,
    new_team: TeamType,
    driver_health: DriverHealth,
) {
    for (entity, layer_ref, maybe_object, maybe_team, maybe_stats, maybe_sprite, maybe_mobile) in
        layers.iter_mut()
    {
        if layer_ref.0 != target_ref_id {
            continue;
        }

        if let Some(mut team) = maybe_team {
            team.0 = new_team;
        }

        if maybe_object.is_some() {
            commands.entity(entity).insert(driver_health.clone());
            if let (ObjectKind::Vehicle(VehicleType::Apc), Some(mut stats)) =
                (target_kind, maybe_stats)
            {
                apply_apc_driver_attack_stats(&mut stats, robot_kind);
            }
        }

        if let Some(mut mobile) = maybe_mobile {
            mobile.team = new_team;
            if let Some(mut sprite) = maybe_sprite {
                if let Some(frame) = game_atlases.mobile_frame(
                    mobile.kind,
                    new_team,
                    mobile.role,
                    mobile.rotation,
                    mobile.frame,
                    false,
                ) {
                    apply_sprite_frame(&mut sprite, frame);
                }
            }
            continue;
        }

        if maybe_object.is_some() {
            if let (ObjectKind::Cannon(cannon), Some(mut sprite)) = (target_kind, maybe_sprite) {
                if let Some(frame) = game_atlases.captured_cannon_frame(cannon, new_team, 180) {
                    apply_sprite_frame(&mut sprite, frame);
                }
            }
        }
    }
}

fn apply_sprite_frame(sprite: &mut Sprite, frame: crate::render::atlas::SpriteFrame) {
    sprite.image = frame.image;
    sprite.texture_atlas = Some(TextureAtlas {
        layout: frame.layout,
        index: frame.index,
    });
    sprite.rect = None;
    sprite.custom_size = None;
}

fn remove_robot_after_enter(
    commands: &mut Commands,
    robot_ref_id: u32,
    layers: &mut Query<(
        Entity,
        &ObjectLayerRef,
        Option<&GameObjectEntity>,
        Option<&mut ObjectTeam>,
        Option<&mut ObjectStats>,
        Option<&mut Sprite>,
        Option<&mut MobileSpriteLayer>,
    )>,
    selection_markers: &Query<(Entity, &SelectionMarker)>,
    selection_health_bars: &Query<(Entity, &SelectionHealthBar)>,
) {
    for (entity, layer_ref, _, _, _, _, _) in layers.iter_mut() {
        if layer_ref.0 == robot_ref_id {
            commands.entity(entity).despawn();
        }
    }

    for (entity, marker) in selection_markers {
        if marker.ref_id == robot_ref_id {
            commands.entity(entity).despawn();
        }
    }
    for (entity, bar) in selection_health_bars {
        if bar.ref_id == robot_ref_id {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{CannonType, RobotType, VehicleType};

    #[test]
    fn can_be_entered_matches_original_target_rules() {
        let vehicle = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);
        let destroyed_vehicle = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 0);
        let robot = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);

        assert!(can_be_entered(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Null,
            vehicle
        ));
        assert!(can_be_entered(
            ObjectKind::Cannon(CannonType::Gun),
            TeamType::Null,
            ObjectStats::from_kind(ObjectKind::Cannon(CannonType::Gun), 100)
        ));
        assert!(!can_be_entered(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Red,
            vehicle
        ));
        assert!(!can_be_entered(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Null,
            destroyed_vehicle
        ));
        assert!(!can_be_entered(
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Null,
            robot
        ));
    }

    #[test]
    fn can_enter_target_is_robot_only_like_enter_wp() {
        let grunt = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let jeep = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);

        assert!(can_enter_target(
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            grunt
        ));
        assert!(!can_enter_target(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Red,
            jeep
        ));
        assert!(!can_enter_target(
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Null,
            grunt
        ));
    }

    #[test]
    fn enter_hit_test_uses_target_rectangle_like_original_under_cursor() {
        assert!(point_in_enter_rect(
            Vec2::new(15.0, 0.0),
            Vec2::ZERO,
            Vec2::new(32.0, 16.0)
        ));
        assert!(!point_in_enter_rect(
            Vec2::new(17.0, 0.0),
            Vec2::ZERO,
            Vec2::new(32.0, 16.0)
        ));
    }

    #[test]
    fn can_enter_fort_matches_original_owner_and_destroyed_rules() {
        let fort = ObjectStats::from_kind(ObjectKind::Building(BuildingType::FortFront), 100);
        let destroyed_fort =
            ObjectStats::from_kind(ObjectKind::Building(BuildingType::FortFront), 0);

        assert!(can_enter_fort(
            ObjectKind::Building(BuildingType::FortFront),
            TeamType::Blue,
            TeamType::Red,
            fort
        ));
        assert!(!can_enter_fort(
            ObjectKind::Building(BuildingType::FortFront),
            TeamType::Red,
            TeamType::Red,
            fort
        ));
        assert!(!can_enter_fort(
            ObjectKind::Building(BuildingType::FortFront),
            TeamType::Blue,
            TeamType::Red,
            destroyed_fort
        ));
        assert!(!can_enter_fort(
            ObjectKind::Building(BuildingType::Radar),
            TeamType::Blue,
            TeamType::Red,
            ObjectStats::from_kind(ObjectKind::Building(BuildingType::Radar), 100)
        ));
    }

    #[test]
    fn apc_enter_removes_leader_and_live_minions_like_original() {
        let entrant = EnterRequestSnapshot {
            entity: Entity::PLACEHOLDER,
            robot_ref_id: 10,
            robot_kind: RobotType::Grunt,
            target_ref_id: 20,
            leader_ref_id: Some(10),
            position: Vec2::ZERO,
            team: TeamType::Red,
            health: 8.0,
        };
        let group_members = [
            EnterGroupMemberSnapshot {
                ref_id: 10,
                leader_ref_id: 10,
                health: 8.0,
                destroyed: false,
            },
            EnterGroupMemberSnapshot {
                ref_id: 11,
                leader_ref_id: 10,
                health: 8.0,
                destroyed: false,
            },
            EnterGroupMemberSnapshot {
                ref_id: 12,
                leader_ref_id: 10,
                health: 0.0,
                destroyed: true,
            },
        ];

        assert_eq!(
            removed_robots_for_enter(
                &entrant,
                ObjectKind::Vehicle(VehicleType::Apc),
                &group_members
            ),
            vec![
                RemovedRobotForEnter {
                    ref_id: 10,
                    health: 8.0,
                },
                RemovedRobotForEnter {
                    ref_id: 11,
                    health: 8.0,
                },
            ]
        );
        assert_eq!(
            removed_robots_for_enter(
                &entrant,
                ObjectKind::Vehicle(VehicleType::Jeep),
                &group_members
            ),
            vec![RemovedRobotForEnter {
                ref_id: 10,
                health: 8.0,
            }]
        );
    }

    #[test]
    fn fort_entrance_rects_match_original_front_and_back_zones() {
        let center = Vec2::new(80.0, -96.0);
        let front_size = Vec2::new(160.0, 192.0);
        assert!(point_in_fort_entrance_rect(
            Vec2::new(80.0, -80.0),
            center,
            front_size,
            BuildingType::FortFront
        ));
        assert!(!point_in_fort_entrance_rect(
            Vec2::new(80.0, -140.0),
            center,
            front_size,
            BuildingType::FortFront
        ));

        let back_center = Vec2::new(80.0, -88.0);
        let back_size = Vec2::new(160.0, 176.0);
        assert!(point_in_fort_entrance_rect(
            Vec2::new(80.0, -48.0),
            back_center,
            back_size,
            BuildingType::FortBack
        ));
        assert!(!point_in_fort_entrance_rect(
            Vec2::new(80.0, -90.0),
            back_center,
            back_size,
            BuildingType::FortBack
        ));
    }
}
