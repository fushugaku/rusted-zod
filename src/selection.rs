use bevy::prelude::*;

use crate::{
    camera::cursor_world_position,
    components::*,
    constants::{HUD_HEIGHT, HUD_WIDTH, TILE_SIZE},
    enter::{enter_fort_target_for_right_click, enter_target_for_right_click},
    grenades::pickup_target_for_right_click,
    original::{
        objects::{BuildingType, ItemType, ObjectKind},
        settings::MAX_UNIT_HEALTH,
        types::TeamType,
    },
    pathing::RouteFootprint,
    production_ui::production_window_kind_for_building,
    repair::{crane_repair_target_for_right_click, unit_repair_target_for_right_click},
};

pub(crate) fn process_hud_commands(
    mut commands: Commands,
    mut queue: ResMut<HudCommandQueue>,
    mut command_state: ResMut<HudCommandState>,
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
            HudCommand::BeginBuildingAction => open_production_window_for_selection(
                &selection.selected_refs,
                TeamType::Red,
                &object_query,
                &mut production_window,
            ),
            HudCommand::JumpToObject(ref_id) => select_and_focus_object(
                &mut commands,
                ref_id,
                &mut camera_query,
                &mut selection,
                &object_query,
                &selection_markers,
                &selection_health_bars,
            ),
        }
    }
}

fn open_production_window_for_selection(
    selected_refs: &[u32],
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
) {
    if let Some((ref_id, kind)) = selected_refs.iter().find_map(|selected_ref| {
        object_query
            .iter()
            .find(|(object, _, _, object_team, stats, production, _)| {
                object.ref_id == *selected_ref
                    && object_team.0 == team
                    && !stats.destroyed()
                    && production.is_some()
                    && production_window_kind_for_building(object.kind).is_some()
            })
            .map(|(object, _, _, _, _, _, _)| (object.ref_id, object.kind))
    }) {
        production_window.open =
            production_window_kind_for_building(kind).map(|window_kind| ProductionWindow {
                building_ref_id: ref_id,
                kind: window_kind,
                selected_index: 0,
                queue_selected_index: 0,
                expanded: false,
                full_selector: None,
                pressed_button: None,
            });
    }
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
    let ref_id = normalized_selection_ref(ref_id, object_query);
    let Some((object, transform, selectable, team, stats, _, _)) = object_query
        .iter()
        .find(|(object, _, _, _, stats, _, _)| object.ref_id == ref_id && !stats.destroyed())
    else {
        return;
    };

    let position = transform.translation.truncate();
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        camera_transform.translation.x = position.x;
        camera_transform.translation.y = position.y;
    }

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

pub(crate) fn handle_mouse_commands(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    passability: Res<PassabilityGrid>,
    hud_layout: Res<HudLayout>,
    cannon_placement: Res<CannonPlacementState>,
    production_window: Res<ProductionWindowState>,
    mut selection: ResMut<SelectionState>,
    mut mouse_state: ResMut<MouseCommandState>,
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
    selection_markers: Query<Entity, With<SelectionMarker>>,
    selection_health_bars: Query<Entity, With<SelectionHealthBar>>,
    target_markers: Query<Entity, With<TargetMarker>>,
    drag_boxes: Query<Entity, With<DragSelectionBox>>,
) {
    if cannon_placement.pending.is_some()
        || production_window.open.is_some()
        || production_window.input_captured
    {
        return;
    }

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
        if !mouse.just_released(MouseButton::Right) {
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

    if mouse.just_pressed(MouseButton::Left) {
        mouse_state.left_start = Some(world_pos);
    }

    if mouse.pressed(MouseButton::Left) {
        if let Some(start) = mouse_state.left_start {
            for box_entity in &drag_boxes {
                commands.entity(box_entity).despawn();
            }

            let delta = world_pos - start;
            if delta.length_squared() > 4.0 {
                spawn_drag_selection_box(&mut commands, start, world_pos, TeamType::Red);
            }
        }
    }

    if mouse.just_released(MouseButton::Left) {
        let Some(start) = mouse_state.left_start.take() else {
            return;
        };

        for box_entity in &drag_boxes {
            commands.entity(box_entity).despawn();
        }

        for marker in &selection_markers {
            commands.entity(marker).despawn();
        }
        for bar in &selection_health_bars {
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

    if mouse.just_released(MouseButton::Right) {
        let selected_refs = selection.selected_refs.clone();
        if selected_refs.is_empty() {
            return;
        }

        let eject_ref = eject_target_for_right_click(
            world_pos,
            &selected_refs,
            TeamType::Red,
            &object_queries.p0(),
        );
        if let Some(eject_ref) = eject_ref {
            for (entity, _, layer_ref, maybe_object) in &object_queries.p1() {
                if layer_ref.0 == eject_ref && maybe_object.is_some() {
                    commands.entity(entity).insert(EjectDriversCommand);
                }
            }
            return;
        }

        let selected_refs = expand_selected_refs_for_orders(&selected_refs, &object_queries.p0());
        if selected_refs.is_empty() {
            return;
        }
        for (entity, _, layer_ref, _) in &object_queries.p1() {
            if selected_refs.contains(&layer_ref.0) {
                commands.entity(entity).remove::<JustLeftCannon>();
            }
        }

        if let Some(repair) = unit_repair_target_for_right_click(
            world_pos,
            &selected_refs,
            TeamType::Red,
            &object_queries.p0(),
        ) {
            let unit_refs: Vec<u32> = repair.units.iter().map(|unit| unit.ref_id).collect();
            let layer_snapshots: Vec<(Entity, u32, Vec2)> = object_queries
                .p1()
                .iter()
                .map(|(entity, transform, layer_ref, _)| {
                    (entity, layer_ref.0, transform.translation.truncate())
                })
                .collect();

            for (entity, ref_id, _) in &layer_snapshots {
                if unit_refs.contains(ref_id) {
                    commands
                        .entity(*entity)
                        .remove::<AttackTarget>()
                        .remove::<PickupGrenadesTarget>()
                        .remove::<EnterTarget>()
                        .remove::<EnterFortTarget>()
                        .remove::<CraneRepairTarget>()
                        .remove::<UnitRepairTarget>();
                }
            }

            for unit in repair.units {
                let Some(route) = passability.route_with_footprint(
                    unit.position,
                    repair.target.entrance_point,
                    RouteFootprint::Vehicle,
                ) else {
                    continue;
                };

                for (entity, ref_id, layer_position) in &layer_snapshots {
                    if *ref_id == unit.ref_id {
                        let layer_offset = *layer_position - unit.position;
                        commands
                            .entity(*entity)
                            .insert(movement_path_for_route(
                                &route,
                                layer_offset,
                                unit.move_speed,
                                false,
                            ))
                            .insert(UnitRepairTarget {
                                ref_id: repair.target.ref_id,
                                stage: UnitRepairStage::GotoEntrance,
                                center_point: repair.target.center_point,
                                entrance_point: repair.target.entrance_point,
                            });
                    }
                }
            }

            return;
        }

        if let Some(repair) = crane_repair_target_for_right_click(
            world_pos,
            &selected_refs,
            TeamType::Red,
            &object_queries.p0(),
        ) {
            let crane_refs: Vec<u32> = repair.cranes.iter().map(|crane| crane.ref_id).collect();
            let layer_snapshots: Vec<(Entity, u32, Vec2)> = object_queries
                .p1()
                .iter()
                .map(|(entity, transform, layer_ref, _)| {
                    (entity, layer_ref.0, transform.translation.truncate())
                })
                .collect();

            for (entity, ref_id, _) in &layer_snapshots {
                if crane_refs.contains(ref_id) {
                    commands
                        .entity(*entity)
                        .remove::<AttackTarget>()
                        .remove::<PickupGrenadesTarget>()
                        .remove::<EnterTarget>()
                        .remove::<EnterFortTarget>()
                        .remove::<CraneRepairTarget>()
                        .remove::<UnitRepairTarget>();
                }
            }

            for crane in repair.cranes {
                let Some(route) = passability.route_with_footprint(
                    crane.position,
                    repair.target.entrance_point,
                    RouteFootprint::Vehicle,
                ) else {
                    continue;
                };

                for (entity, ref_id, layer_position) in &layer_snapshots {
                    if *ref_id == crane.ref_id {
                        let layer_offset = *layer_position - crane.position;
                        commands
                            .entity(*entity)
                            .insert(movement_path_for_route(
                                &route,
                                layer_offset,
                                crane.move_speed,
                                false,
                            ))
                            .insert(CraneRepairTarget {
                                ref_id: repair.target.ref_id,
                                stage: CraneRepairStage::GotoEntrance,
                                center_point: repair.target.center_point,
                                exit_point: repair.target.entrance_point,
                                target_top_left_map: repair.target.target_top_left_map,
                                target_size: repair.target.target_size,
                                target_is_bridge: repair.target.target_is_bridge,
                            });
                    }
                }
            }

            return;
        }

        if let Some(pickup) =
            pickup_target_for_right_click(world_pos, &selected_refs, &object_queries.p2())
        {
            let robot_refs: Vec<u32> = pickup.robots.iter().map(|robot| robot.ref_id).collect();
            let layer_snapshots: Vec<(Entity, u32, Vec2)> = object_queries
                .p1()
                .iter()
                .map(|(entity, transform, layer_ref, _)| {
                    (entity, layer_ref.0, transform.translation.truncate())
                })
                .collect();

            for (entity, ref_id, _) in &layer_snapshots {
                if robot_refs.contains(ref_id) {
                    commands
                        .entity(*entity)
                        .remove::<AttackTarget>()
                        .remove::<PickupGrenadesTarget>()
                        .remove::<EnterTarget>()
                        .remove::<EnterFortTarget>()
                        .remove::<CraneRepairTarget>()
                        .remove::<UnitRepairTarget>();
                }
            }

            for robot in pickup.robots {
                let Some(route) = passability.route(robot.position, pickup.target.position) else {
                    continue;
                };

                for (entity, ref_id, layer_position) in &layer_snapshots {
                    if *ref_id == robot.ref_id {
                        let layer_offset = *layer_position - robot.position;
                        commands
                            .entity(*entity)
                            .insert(movement_path_for_route(
                                &route,
                                layer_offset,
                                robot.move_speed,
                                true,
                            ))
                            .insert(PickupGrenadesTarget {
                                ref_id: pickup.target.ref_id,
                            });
                    }
                }
            }

            return;
        }

        if let Some(enter) =
            enter_target_for_right_click(world_pos, &selected_refs, &object_queries.p0())
        {
            let robot_refs: Vec<u32> = enter.robots.iter().map(|robot| robot.ref_id).collect();
            let layer_snapshots: Vec<(Entity, u32, Vec2)> = object_queries
                .p1()
                .iter()
                .map(|(entity, transform, layer_ref, _)| {
                    (entity, layer_ref.0, transform.translation.truncate())
                })
                .collect();

            for (entity, ref_id, _) in &layer_snapshots {
                if robot_refs.contains(ref_id) {
                    commands
                        .entity(*entity)
                        .remove::<AttackTarget>()
                        .remove::<PickupGrenadesTarget>()
                        .remove::<EnterTarget>()
                        .remove::<EnterFortTarget>()
                        .remove::<CraneRepairTarget>()
                        .remove::<UnitRepairTarget>();
                }
            }

            for robot in enter.robots {
                let Some(route) = passability.route(robot.position, enter.target.waypoint) else {
                    continue;
                };

                for (entity, ref_id, layer_position) in &layer_snapshots {
                    if *ref_id == robot.ref_id {
                        let layer_offset = *layer_position - robot.position;
                        commands
                            .entity(*entity)
                            .insert(movement_path_for_route(
                                &route,
                                layer_offset,
                                robot.move_speed,
                                true,
                            ))
                            .insert(EnterTarget {
                                ref_id: enter.target.ref_id,
                            });
                    }
                }
            }

            return;
        }

        if let Some(enter_fort) = enter_fort_target_for_right_click(
            world_pos,
            &selected_refs,
            TeamType::Red,
            &object_queries.p0(),
        ) {
            let robot_refs: Vec<u32> = enter_fort.robots.iter().map(|robot| robot.ref_id).collect();
            let layer_snapshots: Vec<(Entity, u32, Vec2)> = object_queries
                .p1()
                .iter()
                .map(|(entity, transform, layer_ref, _)| {
                    (entity, layer_ref.0, transform.translation.truncate())
                })
                .collect();

            for (entity, ref_id, _) in &layer_snapshots {
                if robot_refs.contains(ref_id) {
                    commands
                        .entity(*entity)
                        .remove::<AttackTarget>()
                        .remove::<PickupGrenadesTarget>()
                        .remove::<EnterTarget>()
                        .remove::<EnterFortTarget>()
                        .remove::<CraneRepairTarget>()
                        .remove::<UnitRepairTarget>();
                }
            }

            for robot in enter_fort.robots {
                let Some(route) = passability.route_for_object_kind(
                    robot.position,
                    enter_fort.target.exit_point,
                    robot.kind,
                ) else {
                    continue;
                };

                for (entity, ref_id, layer_position) in &layer_snapshots {
                    if *ref_id == robot.ref_id {
                        let layer_offset = *layer_position - robot.position;
                        commands
                            .entity(*entity)
                            .insert(movement_path_for_route(
                                &route,
                                layer_offset,
                                robot.move_speed,
                                false,
                            ))
                            .insert(EnterFortTarget {
                                ref_id: enter_fort.target.ref_id,
                                stage: EnterFortStage::GotoEntrance,
                                inside_point: enter_fort.target.inside_point,
                                exit_point: enter_fort.target.exit_point,
                            });
                    }
                }
            }

            return;
        }

        let attack_target =
            attack_target_at_position(world_pos, &selected_refs, &attack_selectable_query);
        if let Some(target_ref_id) = attack_target {
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
            let layer_snapshots: Vec<(Entity, u32, Vec2)> = object_queries
                .p1()
                .iter()
                .map(|(entity, transform, layer_ref, _)| {
                    (entity, layer_ref.0, transform.translation.truncate())
                })
                .collect();
            let attack_routes: Vec<(u32, Vec2, Vec<Vec2>, f32)> = attackers
                .iter()
                .filter_map(
                    |(ref_id, kind, base_position, move_speed, attack_radius, mobile)| {
                        if !*mobile {
                            return None;
                        }
                        passability
                            .route_to_attack_range_for_object_kind(
                                *base_position,
                                target_position,
                                *attack_radius,
                                *kind,
                            )
                            .map(|route| (*ref_id, *base_position, route, *move_speed))
                    },
                )
                .collect();

            for (entity, ref_id, _) in &layer_snapshots {
                if attacker_refs.contains(ref_id) {
                    commands
                        .entity(*entity)
                        .remove::<MovementPath>()
                        .remove::<PickupGrenadesTarget>()
                        .remove::<EnterTarget>()
                        .remove::<EnterFortTarget>()
                        .remove::<CraneRepairTarget>()
                        .remove::<UnitRepairTarget>();
                }
            }

            for (entity, ref_id, layer_position) in &layer_snapshots {
                if attacker_refs.contains(ref_id) {
                    if let Some((_, base_position, route, move_speed)) = attack_routes
                        .iter()
                        .find(|(route_ref_id, _, _, _)| route_ref_id == ref_id)
                    {
                        let layer_offset = *layer_position - *base_position;
                        commands.entity(*entity).insert(movement_path_for_route(
                            route,
                            layer_offset,
                            *move_speed,
                            false,
                        ));
                    }

                    commands.entity(*entity).insert(AttackTarget {
                        ref_id: target_ref_id,
                        cooldown: 0.0,
                        player_given: true,
                    });
                }
            }

            return;
        }

        let mobile_bases: Vec<(u32, ObjectKind, Vec2, f32)> = object_queries
            .p0()
            .iter()
            .filter(|(object, _, selectable, _, _, _, _)| {
                selectable.mobile && selected_refs.contains(&object.ref_id)
            })
            .map(|(object, transform, _, _, stats, _, _)| {
                (
                    object.ref_id,
                    object.kind,
                    transform.translation.truncate(),
                    stats.move_speed,
                )
            })
            .collect();

        if mobile_bases.is_empty() {
            return;
        }

        for marker in &target_markers {
            commands.entity(marker).despawn();
        }

        commands.spawn((
            Sprite::from_color(Color::srgba(0.2, 0.9, 1.0, 0.35), Vec2::splat(10.0)),
            Transform::from_xyz(world_pos.x, world_pos.y, 31.0),
            TargetMarker,
            Name::new("move_target_marker"),
        ));

        let attempt_run = move_target_is_near_flag(world_pos, &object_queries.p0());
        for (ref_id, kind, base_position, move_speed) in mobile_bases {
            let Some(route) = passability.route_for_object_kind(base_position, world_pos, kind)
            else {
                continue;
            };

            for (entity, transform, layer_ref, _) in object_queries.p1().iter() {
                if layer_ref.0 == ref_id {
                    let layer_offset = transform.translation.truncate() - base_position;
                    commands
                        .entity(entity)
                        .remove::<AttackTarget>()
                        .remove::<PickupGrenadesTarget>()
                        .remove::<EnterTarget>()
                        .remove::<EnterFortTarget>()
                        .remove::<CraneRepairTarget>()
                        .remove::<UnitRepairTarget>()
                        .insert(movement_path_for_route(
                            &route,
                            layer_offset,
                            move_speed,
                            attempt_run,
                        ));
                }
            }
        }
    }
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
