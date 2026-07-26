use bevy::{camera::visibility::RenderLayers, prelude::*};

use crate::components::*;
use crate::constants::{HUD_LAYER, TILE_SIZE};
use crate::original::map::MapObject;
use crate::original::objects::{ItemType, ObjectKind};
use crate::original::types::{PlanetType, TeamType};
use crate::render::atlas::{GameAtlases, MobileSpriteRole, SpriteFrame};
use crate::settings_sync::SourceSettingsState;

#[derive(Clone, Copy)]
pub(crate) struct RuntimeSpawnLayer {
    pub(crate) entity: Entity,
    pub(crate) world_offset: Vec2,
}

pub(crate) struct RuntimeSpawnedObject {
    pub(crate) gameplay_entity: Entity,
    pub(crate) visual_layers: Vec<RuntimeSpawnLayer>,
}

pub(crate) fn spawn_runtime_object_from_source_init(
    commands: &mut Commands,
    atlases: &GameAtlases,
    asset_server: &AssetServer,
    planet: PlanetType,
    hud_layout: &HudLayout,
    settings: &SourceSettingsState,
    ref_id: u32,
    kind: ObjectKind,
    team: TeamType,
    source_map_position: Vec2,
    building_level: i8,
    extra_links: u16,
    cannon_ejectable: bool,
    initial_rotation: u16,
    just_left_cannon: bool,
    just_placed_cannon: bool,
    reserved_gameplay_entity: Entity,
) -> Option<RuntimeSpawnedObject> {
    spawn_runtime_object_with_source_position(
        commands,
        atlases,
        asset_server,
        planet,
        hud_layout,
        settings,
        ref_id,
        kind,
        team,
        None,
        Some(source_map_position),
        100,
        cannon_ejectable,
        just_left_cannon,
        None,
        None,
        building_level,
        extra_links,
        initial_rotation,
        just_placed_cannon,
        Some(reserved_gameplay_entity),
    )
}

#[allow(clippy::too_many_arguments)]
fn spawn_runtime_object_with_source_position(
    commands: &mut Commands,
    atlases: &GameAtlases,
    asset_server: &AssetServer,
    planet: PlanetType,
    hud_layout: &HudLayout,
    settings: &SourceSettingsState,
    ref_id: u32,
    kind: ObjectKind,
    team: TeamType,
    create_center: Option<Vec2>,
    source_map_position: Option<Vec2>,
    health_percent: i32,
    cannon_ejectable: bool,
    just_left_cannon: bool,
    move_waypoints: Option<&[MovementWaypoint]>,
    robot_group_leader_ref_id: Option<u32>,
    building_level: i8,
    extra_links: u16,
    initial_rotation: u16,
    just_placed_cannon: bool,
    reserved_gameplay_entity: Option<Entity>,
) -> Option<RuntimeSpawnedObject> {
    let Some((object_type, object_id)) = crate::units::object_kind_to_map_parts(kind) else {
        return None;
    };
    let grid_map_position = source_map_position.unwrap_or_else(|| {
        let create_center = create_center.unwrap_or(Vec2::ZERO);
        Vec2::new(create_center.x, -create_center.y)
    });
    let object = MapObject {
        x: (grid_map_position.x / TILE_SIZE).floor().max(0.0) as u16,
        y: (grid_map_position.y / TILE_SIZE).floor().max(0.0) as u16,
        owner: normalized_object_owner(kind, team),
        object_type,
        object_id,
        building_level,
        extra_links,
        health_percent,
    };
    let mut stats = settings.object_stats(kind, object.health_percent);
    stats.cannon_ejectable =
        crate::units::cannon_ejectable_for_runtime_spawn(kind, cannon_ejectable);
    let source_owned = source_map_position.is_some();
    let mut layers = atlases.sprite_layers_for_object(&object, planet);
    if source_owned
        && let ObjectKind::Cannon(cannon) = kind
        && let Some(frame) = atlases.captured_cannon_frame(cannon, team, initial_rotation)
    {
        layers = vec![frame];
    }
    let fallback_size = fallback_marker_size(kind);
    if !source_owned && layers.is_empty() && fallback_size.is_none() {
        return None;
    }
    let display_size = layers
        .first()
        .map(|frame| frame.frame_size)
        .or(fallback_size)
        .unwrap_or_else(|| crate::units::fallback_collision_size(kind));
    let gameplay_size = if source_owned {
        crate::units::source_mobile_dimensions(kind)
            .unwrap_or_else(|| crate::units::combat_object_default_size(kind))
    } else {
        display_size
    };
    let top_left_map = source_map_position.or_else(|| {
        create_center.map(|center| Vec2::new(center.x, -center.y) - display_size * 0.5)
    });
    let create_center = create_center.or_else(|| {
        top_left_map.map(|top_left| source_top_left_world_center(top_left, gameplay_size))
    });
    let Some(create_center) = create_center else {
        return None;
    };
    let top_left_map = top_left_map
        .unwrap_or_else(|| Vec2::new(create_center.x, -create_center.y) - gameplay_size * 0.5);
    let cannon_render_offset = match kind {
        ObjectKind::Cannon(cannon) if source_owned => {
            crate::units::cannons::render_offset(cannon, usize::from(initial_rotation / 45) % 8)
        }
        _ => Vec2::ZERO,
    };
    let visual_top_left_map = top_left_map + cannon_render_offset;
    let mut production =
        crate::units::buildings::can_set_rallypoints(kind).then(|| BuildingProduction {
            status: BuildingProductionStatus::Select,
            current: None,
            queue: Default::default(),
            elapsed: 0.0,
            duration: 0.0,
            zone_ownage: 0.0,
            unit_limit_reached: false,
            stored_cannons: Vec::new(),
        });
    let production_active = false;

    let mut gameplay_entity = None;
    let mut spawned_visual_layers = Vec::new();
    if source_owned {
        let mut entity = match reserved_gameplay_entity {
            Some(entity) => commands.entity(entity),
            None => commands.spawn_empty(),
        };
        entity.insert((
            Transform::from_xyz(create_center.x, create_center.y, 5.0),
            Visibility::Visible,
            ObjectLayerRef(ref_id),
            Name::new(format!("runtime #{ref_id} {:?} gameplay root", kind)),
            GameObjectEntity { ref_id, kind },
            ObjectTeam(team),
            HealthPercent(object.health_percent),
            DamageCauseTimers::default(),
            SourceObjectLocation {
                map_position: top_left_map,
                map_velocity: Vec2::ZERO,
                map_remainder: Vec2::ZERO,
                world_anchor: create_center - Vec2::new(top_left_map.x, -top_left_map.y),
            },
            stats,
            BuildingLevel::from_original(object.building_level),
            MapGridPosition {
                x: object.x,
                y: object.y,
            },
        ));
        if stats.move_speed > 0.0 {
            entity.insert(MovementVelocity::default());
        }
        if let Some(selectable) = selectable_for(kind, gameplay_size) {
            entity.insert(selectable);
        }
        if let Some(footprint) = bridge_footprint_for(&object, kind) {
            entity.insert(footprint);
        }
        insert_grenade_components(&mut entity, kind);
        insert_hut_components(&mut entity, kind);
        if let Some(stamina) = initial_movement_stamina(settings, kind) {
            entity.insert(stamina);
        }
        if let Some(timer) = initial_vehicle_effect_drop_timer(kind) {
            entity.insert(timer);
        }
        if let Some(lid) = initial_vehicle_lid_state(kind) {
            entity.insert(lid);
        }
        if just_left_cannon {
            entity.insert(JustLeftCannon);
        }
        if let Some(driver) = initial_driver_health(kind, team) {
            entity.insert(driver);
        }
        if let Some(production) = production.take() {
            entity.insert((production, BuildingRallyPoints::default()));
        }
        if let Some(move_waypoints) = move_waypoints
            && stats.move_speed > 0.0
        {
            entity.insert(MovementPath::from_typed(
                move_waypoints.to_vec(),
                stats.move_speed,
            ));
        }
        gameplay_entity = Some(entity.id());
    }

    if !layers.is_empty() {
        let mut base_position = create_center;
        for (layer_index, frame) in layers.into_iter().enumerate() {
            let layer_position = sprite_position_from_map_top_left(visual_top_left_map, &frame);
            if layer_index == 0 && !source_owned {
                base_position = layer_position;
            }
            let animation_indices = frame.animation_indices.clone();
            let mut entity = commands.spawn((
                Sprite {
                    image: frame.image,
                    texture_atlas: Some(TextureAtlas {
                        layout: frame.layout,
                        index: frame.index,
                    }),
                    ..default()
                },
                Transform::from_xyz(
                    layer_position.x,
                    layer_position.y,
                    5.0 + layer_index as f32 * 0.01,
                ),
                ObjectLayerRef(ref_id),
                Name::new(format!("runtime #{ref_id} {:?} layer {layer_index}", kind)),
            ));
            if source_owned {
                spawned_visual_layers.push(RuntimeSpawnLayer {
                    entity: entity.id(),
                    world_offset: layer_position - create_center,
                });
            }
            if stats.move_speed > 0.0 {
                entity.insert(MovementVelocity::default());
            }

            if layer_index == 0 && !source_owned {
                entity.insert((
                    GameObjectEntity { ref_id, kind },
                    ObjectTeam(team),
                    HealthPercent(object.health_percent),
                    DamageCauseTimers::default(),
                    SourceObjectLocation {
                        map_position: top_left_map,
                        map_velocity: Vec2::ZERO,
                        map_remainder: Vec2::ZERO,
                        world_anchor: layer_position - Vec2::new(top_left_map.x, -top_left_map.y),
                    },
                    stats,
                    BuildingLevel::from_original(object.building_level),
                    MapGridPosition {
                        x: object.x,
                        y: object.y,
                    },
                ));
                if let Some(group) = robot_group_for(kind, ref_id, robot_group_leader_ref_id) {
                    entity.insert(group);
                }
                if let Some(selectable) = selectable_for(kind, frame.frame_size) {
                    entity.insert(selectable);
                }
                if let Some(footprint) = bridge_footprint_for(&object, kind) {
                    entity.insert(footprint);
                }
                insert_grenade_components(&mut entity, kind);
                if let Some(stamina) = initial_movement_stamina(settings, kind) {
                    entity.insert(stamina);
                }
                if let Some(timer) = initial_vehicle_effect_drop_timer(kind) {
                    entity.insert(timer);
                }
                if let Some(lid) = initial_vehicle_lid_state(kind) {
                    entity.insert(lid);
                }
                if just_left_cannon {
                    entity.insert(JustLeftCannon);
                }
                if let Some(driver) = initial_driver_health(kind, team) {
                    entity.insert(driver);
                }
                gameplay_entity = Some(entity.id());
            }

            if layer_index == 0
                && just_placed_cannon
                && let ObjectKind::Cannon(cannon) = kind
            {
                entity.insert(crate::units::cannons::CannonPlacementAnimation::new(
                    cannon,
                    team,
                    top_left_map,
                ));
            }

            if let Some(role) = mobile_sprite_role(kind, layer_index) {
                entity.insert(MobileSpriteLayer {
                    kind,
                    team,
                    role,
                    rotation: initial_rotation,
                    frame: 0,
                    elapsed: 0.0,
                });
            }

            if let Some(move_waypoints) = move_waypoints {
                if stats.move_speed > 0.0 {
                    let layer_offset = layer_position - base_position;
                    entity.insert(MovementPath::from_typed(
                        move_waypoints
                            .iter()
                            .map(|waypoint| {
                                waypoint.with_position(waypoint.position + layer_offset)
                            })
                            .collect(),
                        stats.move_speed,
                    ));
                }
            }

            if let Some(frames) = animation_indices {
                entity.insert(AtlasAnimation {
                    frames,
                    frame_time: 0.2,
                    elapsed: 0.0,
                    current: 0,
                });
            }
        }
        spawn_vehicle_lid_visual_layers(
            commands,
            asset_server,
            ref_id,
            kind,
            team,
            &format!("runtime #{ref_id}"),
        );
        if source_owned {
            spawn_radar_overlay_layers(
                commands,
                atlases,
                ref_id,
                kind,
                team,
                stats,
                &object,
                ref_id as usize,
            );
            spawn_repair_overlay_layers(
                commands,
                atlases,
                ref_id,
                kind,
                team,
                stats,
                &object,
                ref_id as usize,
            );
            spawn_factory_overlay_layers(
                commands,
                atlases,
                ref_id,
                kind,
                team,
                stats,
                production_active,
                &object,
                ref_id as usize,
            );
        }
    } else if let Some(size) = fallback_size {
        let source_map_position = top_left_map;
        let mut entity = commands.spawn((
            Sprite::from_color(object_color(kind, team), size),
            Transform::from_xyz(create_center.x, create_center.y, 5.0),
            ObjectLayerRef(ref_id),
            Name::new(format!("runtime #{ref_id} {:?}", kind)),
        ));
        if source_owned {
            spawned_visual_layers.push(RuntimeSpawnLayer {
                entity: entity.id(),
                world_offset: Vec2::ZERO,
            });
        }
        if !source_owned {
            entity.insert((
                GameObjectEntity { ref_id, kind },
                ObjectTeam(team),
                HealthPercent(object.health_percent),
                DamageCauseTimers::default(),
                SourceObjectLocation {
                    map_position: source_map_position,
                    map_velocity: Vec2::ZERO,
                    map_remainder: Vec2::ZERO,
                    world_anchor: create_center
                        - Vec2::new(source_map_position.x, -source_map_position.y),
                },
                stats,
                BuildingLevel::from_original(object.building_level),
                MapGridPosition {
                    x: object.x,
                    y: object.y,
                },
            ));
            if stats.move_speed > 0.0 {
                entity.insert(MovementVelocity::default());
            }
            if let Some(group) = robot_group_for(kind, ref_id, robot_group_leader_ref_id) {
                entity.insert(group);
            }
            if let Some(selectable) = selectable_for(kind, size) {
                entity.insert(selectable);
            }
            if let Some(footprint) = bridge_footprint_for(&object, kind) {
                entity.insert(footprint);
            }
            insert_grenade_components(&mut entity, kind);
            insert_hut_components(&mut entity, kind);
            if let Some(stamina) = initial_movement_stamina(settings, kind) {
                entity.insert(stamina);
            }
            if let Some(timer) = initial_vehicle_effect_drop_timer(kind) {
                entity.insert(timer);
            }
            if let Some(move_waypoints) = move_waypoints
                && stats.move_speed > 0.0
            {
                entity.insert(MovementPath::from_typed(
                    move_waypoints.to_vec(),
                    stats.move_speed,
                ));
            }
            gameplay_entity = Some(entity.id());
        }
    }

    if !matches!(kind, ObjectKind::MapItem(item) if item == ItemType::Rock as u8) {
        spawn_minimap_dot(commands, ref_id, create_center, team, *hud_layout);
    }
    Some(RuntimeSpawnedObject {
        gameplay_entity: gameplay_entity?,
        visual_layers: spawned_visual_layers,
    })
}

fn insert_grenade_components(entity: &mut EntityCommands, kind: ObjectKind) {
    if crate::units::items::grenades::can_have_grenades(kind) {
        entity.insert(GrenadeInventory { amount: 0 });
    }

    if crate::units::items::grenades::is_grenade_box(kind) {
        entity.insert(GrenadeBox {
            amount: crate::units::items::grenades::default_box_amount(),
        });
    }
}

fn insert_hut_components(entity: &mut EntityCommands, kind: ObjectKind) {
    if let Some(spawner) = crate::units::items::hut_ui::initial_animal_spawner(kind) {
        entity.insert(spawner);
    }
}

fn spawn_radar_overlay_layers(
    commands: &mut Commands,
    atlases: &GameAtlases,
    ref_id: u32,
    kind: ObjectKind,
    owner: TeamType,
    stats: ObjectStats,
    object: &MapObject,
    map_ref_id: usize,
) {
    let Some(kinds) = crate::units::buildings::radar_overlay_kinds_for_object(kind) else {
        return;
    };

    let top_left_map = Vec2::new(object.x as f32 * TILE_SIZE, object.y as f32 * TILE_SIZE);

    for (layer_index, overlay_kind) in kinds.iter().copied().enumerate() {
        let Some(frames) = atlases.radar_overlay_frames(overlay_kind) else {
            continue;
        };
        let Some(first) = frames.first().cloned() else {
            continue;
        };
        let (sprite_x, sprite_y) = sprite_position(object.x, object.y, &first);
        let visible =
            crate::units::buildings::radar_overlay_should_be_visible(overlay_kind, owner, stats);

        commands.spawn((
            Sprite {
                image: first.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: first.layout.clone(),
                    index: first.index,
                }),
                ..default()
            },
            Transform::from_xyz(sprite_x, sprite_y, 5.05 + layer_index as f32 * 0.01),
            if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            ObjectLayerRef(ref_id),
            RadarOverlayLayer {
                ref_id,
                kind: overlay_kind,
                top_left_map,
                frames,
                frame_time: crate::units::buildings::radar_overlay_frame_time(),
                elapsed: 0.0,
                current: 0,
            },
            Name::new(format!(
                "#{}.radar_overlay {:?}:{:?}",
                map_ref_id + 1,
                ref_id,
                overlay_kind
            )),
        ));
    }
}

fn spawn_repair_overlay_layers(
    commands: &mut Commands,
    atlases: &GameAtlases,
    ref_id: u32,
    kind: ObjectKind,
    owner: TeamType,
    stats: ObjectStats,
    object: &MapObject,
    map_ref_id: usize,
) {
    let Some(kinds) = crate::units::buildings::repair_overlay_kinds_for_object(kind) else {
        return;
    };

    let top_left_map = Vec2::new(object.x as f32 * TILE_SIZE, object.y as f32 * TILE_SIZE);

    for (layer_index, overlay_kind) in kinds.iter().copied().enumerate() {
        let Some(frames) = atlases.repair_overlay_frames(overlay_kind) else {
            continue;
        };
        let current =
            crate::units::buildings::repair_overlay_initial_frame(overlay_kind, owner, false);
        let Some(first) = frames
            .get(current.min(frames.len().saturating_sub(1)))
            .cloned()
        else {
            continue;
        };
        let (sprite_x, sprite_y) = sprite_position(object.x, object.y, &first);
        let visible = crate::units::buildings::repair_overlay_should_be_visible(
            overlay_kind,
            owner,
            stats,
            false,
        );

        commands.spawn((
            Sprite {
                image: first.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: first.layout.clone(),
                    index: first.index,
                }),
                ..default()
            },
            Transform::from_xyz(sprite_x, sprite_y, 5.05 + layer_index as f32 * 0.01),
            if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            ObjectLayerRef(ref_id),
            RepairOverlayLayer {
                ref_id,
                kind: overlay_kind,
                top_left_map,
                frames,
                frame_time: crate::units::buildings::repair_overlay_frame_time(),
                elapsed: 0.0,
                current,
            },
            Name::new(format!(
                "#{}.repair_overlay {:?}:{:?}",
                map_ref_id + 1,
                ref_id,
                overlay_kind
            )),
        ));
    }
}

fn spawn_factory_overlay_layers(
    commands: &mut Commands,
    atlases: &GameAtlases,
    ref_id: u32,
    kind: ObjectKind,
    owner: TeamType,
    stats: ObjectStats,
    production_active: bool,
    object: &MapObject,
    map_ref_id: usize,
) {
    let Some(overlay_kinds) = crate::units::buildings::factory_overlay_kinds_for_object(kind)
    else {
        return;
    };

    let top_left_map = Vec2::new(object.x as f32 * TILE_SIZE, object.y as f32 * TILE_SIZE);

    for (layer_index, overlay_kind) in overlay_kinds.iter().copied().enumerate() {
        let Some(frames) = atlases.factory_overlay_frames(overlay_kind) else {
            continue;
        };
        let Some(first) = frames.first().cloned() else {
            continue;
        };
        let (sprite_x, sprite_y) = sprite_position(object.x, object.y, &first);
        let visible =
            crate::units::buildings::factory_overlay_should_be_visible(
                overlay_kind,
                owner,
                stats,
                production_active,
                0,
            ) && crate::units::buildings::factory_overlay_initial_frame_visible(overlay_kind);

        commands.spawn((
            Sprite {
                image: first.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: first.layout.clone(),
                    index: first.index,
                }),
                ..default()
            },
            Transform::from_xyz(sprite_x, sprite_y, 5.05 + layer_index as f32 * 0.01),
            if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            ObjectLayerRef(ref_id),
            FactoryOverlayLayer {
                ref_id,
                kind: overlay_kind,
                top_left_map,
                frames,
                frame_time: crate::units::buildings::factory_overlay_frame_time(),
                elapsed: 0.0,
                current: 0,
            },
            Name::new(format!(
                "#{}.factory_overlay {:?}:{:?}",
                map_ref_id + 1,
                ref_id,
                overlay_kind
            )),
        ));
    }
}

fn normalized_object_owner(kind: ObjectKind, owner: TeamType) -> TeamType {
    crate::units::items::object_display_owner(kind, owner)
}

fn bridge_footprint_for(object: &MapObject, kind: ObjectKind) -> Option<BridgeFootprint> {
    let ObjectKind::Bridge(building) = kind else {
        return None;
    };

    Some(BridgeFootprint {
        x: object.x,
        y: object.y,
        building,
        extra_links: object.extra_links,
    })
}

pub(crate) fn initial_movement_stamina(
    settings: &SourceSettingsState,
    kind: ObjectKind,
) -> Option<MovementStamina> {
    let max = settings.unit_settings(kind)?.max_run_time;
    (max > 0.0).then_some(MovementStamina {
        max,
        current: max,
        running: false,
    })
}

fn initial_vehicle_effect_drop_timer(kind: ObjectKind) -> Option<VehicleEffectDropTimer> {
    let ObjectKind::Vehicle(vehicle) = kind else {
        return None;
    };

    crate::units::vehicles::initial_effect_drop_timer_elapsed(vehicle)
        .map(|elapsed| VehicleEffectDropTimer { elapsed })
}

pub(crate) fn robot_group_for(
    kind: ObjectKind,
    ref_id: u32,
    leader_ref_id: Option<u32>,
) -> Option<RobotGroup> {
    matches!(kind, ObjectKind::Robot(_)).then_some(RobotGroup {
        leader_ref_id: crate::units::robots::group_leader_ref_id(ref_id, leader_ref_id),
        member_index: leader_ref_id
            .and_then(|leader_ref_id| ref_id.checked_sub(leader_ref_id))
            .and_then(|index| u16::try_from(index).ok())
            .unwrap_or(0),
    })
}

pub(crate) fn initial_driver_health(kind: ObjectKind, team: TeamType) -> Option<DriverHealth> {
    crate::units::initial_driver_health(kind, team)
}

pub(crate) fn initial_vehicle_lid_state(kind: ObjectKind) -> Option<VehicleLidState> {
    match kind {
        ObjectKind::Vehicle(vehicle) if crate::units::vehicles::has_lid(vehicle) => {
            Some(VehicleLidState::closed())
        }
        _ => None,
    }
}

fn spawn_vehicle_lid_visual_layers(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ref_id: u32,
    kind: ObjectKind,
    team: TeamType,
    name_prefix: &str,
) {
    let ObjectKind::Vehicle(vehicle) = kind else {
        return;
    };
    if team == TeamType::Null || !crate::units::vehicles::has_lid(vehicle) {
        return;
    }

    spawn_vehicle_lid_visual_layer(
        commands,
        asset_server,
        ref_id,
        vehicle,
        VehicleLidVisualRole::Lid,
        crate::units::vehicles::lid_frame_paths(),
        crate::units::vehicles::LID_FRAME_COUNT,
        name_prefix,
    );
    spawn_vehicle_lid_visual_layer(
        commands,
        asset_server,
        ref_id,
        vehicle,
        VehicleLidVisualRole::Driver,
        crate::units::vehicles::tank_driver_frame_paths(team),
        crate::units::vehicles::TANK_DRIVER_FRAME_COUNT,
        name_prefix,
    );
}

fn spawn_vehicle_lid_visual_layer(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ref_id: u32,
    vehicle: crate::original::objects::VehicleType,
    role: VehicleLidVisualRole,
    frame_paths: Vec<String>,
    frames_per_direction: usize,
    name_prefix: &str,
) {
    let frames: Vec<_> = frame_paths
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();
    let Some(first_frame) = frames.first().cloned() else {
        return;
    };

    commands.spawn((
        Sprite::from_image(first_frame),
        Transform::from_xyz(0.0, 0.0, 5.02),
        Visibility::Hidden,
        ObjectLayerRef(ref_id),
        VehicleLidVisualLayer { vehicle, role },
        VehicleLidVisualFrames {
            frames,
            frames_per_direction,
        },
        Name::new(format!("{name_prefix} vehicle lid {role:?}")),
    ));
}

fn sprite_position_from_map_top_left(top_left_map: Vec2, frame: &SpriteFrame) -> Vec2 {
    Vec2::new(
        top_left_map.x + frame.world_offset.x + frame.source_offset.x + frame.frame_size.x * 0.5,
        -(top_left_map.y + frame.world_offset.y + frame.source_offset.y + frame.frame_size.y * 0.5),
    )
}

fn source_top_left_world_center(top_left_map: Vec2, size: Vec2) -> Vec2 {
    Vec2::new(
        top_left_map.x + size.x * 0.5,
        -(top_left_map.y + size.y * 0.5),
    )
}

fn spawn_minimap_dot(
    commands: &mut Commands,
    ref_id: u32,
    position: Vec2,
    team: TeamType,
    layout: HudLayout,
) {
    let local = layout.map_pixel_to_minimap_local(Vec2::new(position.x, -position.y));
    commands.spawn((
        Sprite::from_color(team.color(), Vec2::splat(2.0)),
        Transform::from_xyz(0.0, 0.0, 713.0),
        RenderLayers::layer(HUD_LAYER),
        HudAnchor::BottomRight {
            offset: layout.bottom_right_offset_for_minimap_local(local),
        },
        MinimapDot { ref_id },
        Name::new("minimap_object"),
    ));
}

fn mobile_sprite_role(kind: ObjectKind, layer_index: usize) -> Option<MobileSpriteRole> {
    crate::units::mobile_sprite_role(kind, layer_index)
}

fn sprite_position(tile_x: u16, tile_y: u16, frame: &SpriteFrame) -> (f32, f32) {
    let x = tile_x as f32 * TILE_SIZE
        + frame.world_offset.x
        + frame.source_offset.x
        + frame.frame_size.x * 0.5;
    let y = -(tile_y as f32 * TILE_SIZE
        + frame.world_offset.y
        + frame.source_offset.y
        + frame.frame_size.y * 0.5);
    (x, y)
}

fn fallback_marker_size(kind: ObjectKind) -> Option<Vec2> {
    crate::units::fallback_marker_size(kind)
}

fn object_color(kind: ObjectKind, team: TeamType) -> Color {
    crate::units::fallback_marker_color(kind, team)
}

fn selectable_for(kind: ObjectKind, selection_size: Vec2) -> Option<Selectable> {
    crate::units::selectable_for(kind, selection_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_runtime_spawn_keeps_packet_top_left_without_tile_quantization() {
        assert_eq!(
            source_top_left_world_center(Vec2::new(49.0, 65.0), Vec2::new(32.0, 32.0)),
            Vec2::new(65.0, -81.0)
        );
    }
}
