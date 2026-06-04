use bevy::{camera::visibility::RenderLayers, prelude::*};

use crate::components::*;
use crate::constants::{HUD_LAYER, TILE_SIZE};
use crate::original::map::{MapObject, MapObjectType, ZMap};
use crate::original::objects::{BuildingType, ItemType, ObjectKind, RobotType, VehicleType};
use crate::original::settings::{GRENADES_PER_BOX, object_max_health, unit_settings};
use crate::original::types::{PlanetType, TeamType};
use crate::production::{initial_production_for_building, initial_spawn_count};
use crate::render::atlas::{
    FactoryOverlayKind, GameAtlases, MobileSpriteRole, RadarOverlayKind, RepairOverlayKind,
    SpriteFrame,
};

pub(crate) fn spawn_objects(commands: &mut Commands, map: &ZMap, atlases: &GameAtlases) -> u32 {
    let mut sprite_count = 0;
    let mut fallback_count = 0;
    let mut hidden_count = 0;
    let mut next_ref_id = 1;

    for (map_ref_id, object) in map.objects.iter().enumerate() {
        let x = object.x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
        let y = -(object.y as f32 * TILE_SIZE + TILE_SIZE * 0.5);
        let kind = ObjectKind::from_map_parts(object.object_type, object.object_id)
            .unwrap_or(ObjectKind::MapItem(object.object_id));
        let owner = normalized_object_owner(kind, object.owner);
        let spawn_count = initial_spawn_count(kind);
        let group_leader_ref_id =
            (matches!(kind, ObjectKind::Robot(_)) && spawn_count > 1).then_some(next_ref_id);

        for group_member in 0..spawn_count {
            let ref_id = next_ref_id;
            next_ref_id += 1;
            let mut stats = ObjectStats::from_kind(kind, object.health_percent);
            if matches!(kind, ObjectKind::Cannon(_))
                && area_is_fort_turret_tile(map, object.x as i32, object.y as i32)
            {
                stats.cannon_ejectable = false;
            }
            let building_level = BuildingLevel::from_original(object.building_level);
            let mut production =
                initial_production_for_building(kind, building_level.0, owner, stats);
            let production_active = production
                .as_ref()
                .map(|production| production.status != BuildingProductionStatus::Select)
                .unwrap_or(false);
            let components = (
                GameObjectEntity { ref_id, kind },
                ObjectTeam(owner),
                HealthPercent(object.health_percent),
                DamageCauseTimers::default(),
                stats,
                building_level,
                MapGridPosition {
                    x: object.x,
                    y: object.y,
                },
            );

            let layers = atlases.sprite_layers_for_object(object, map.basics.terrain_type);
            if !layers.is_empty() {
                for (layer_index, frame) in layers.into_iter().enumerate() {
                    let (sprite_x, sprite_y) = sprite_position(object.x, object.y, &frame);
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
                        Transform::from_xyz(sprite_x, sprite_y, 5.0 + layer_index as f32 * 0.01),
                        ObjectLayerRef(ref_id),
                        object_layer_name(map_ref_id, group_member, object, layer_index),
                    ));

                    if layer_index == 0 {
                        entity.insert(components);
                        if let Some(group) = robot_group_for(kind, ref_id, group_leader_ref_id) {
                            entity.insert(group);
                        }
                        if let Some(selectable) = selectable_for(kind, frame.frame_size) {
                            entity.insert(selectable);
                        }
                        if let Some(footprint) = bridge_footprint_for(object, kind) {
                            entity.insert(footprint);
                        }
                        insert_grenade_components(&mut entity, kind);
                        insert_hut_components(&mut entity, kind);
                        if let Some(stamina) = initial_movement_stamina(kind) {
                            entity.insert(stamina);
                        }
                        if vehicle_drops_effects(kind) {
                            entity.insert(VehicleEffectDropTimer { elapsed: 0.2 });
                        }
                        if let Some(driver) = initial_driver_health(kind, object.owner) {
                            entity.insert(driver);
                        }
                        if let Some(production) = production.take() {
                            entity.insert(production);
                        }
                    }

                    if let Some(role) = mobile_sprite_role(kind, layer_index) {
                        entity.insert(MobileSpriteLayer {
                            kind,
                            team: object.owner,
                            role,
                            rotation: 180,
                            frame: 0,
                            elapsed: 0.0,
                        });
                    }

                    if let Some(frames) = animation_indices {
                        entity.insert(AtlasAnimation {
                            frames,
                            frame_time: 0.2,
                            elapsed: 0.0,
                            current: 0,
                        });
                    }

                    sprite_count += 1;
                }
                spawn_radar_overlay_layers(
                    commands, atlases, ref_id, kind, owner, stats, object, map_ref_id,
                );
                spawn_repair_overlay_layers(
                    commands, atlases, ref_id, kind, owner, stats, object, map_ref_id,
                );
                spawn_factory_overlay_layers(
                    commands,
                    atlases,
                    ref_id,
                    kind,
                    owner,
                    stats,
                    production_active,
                    object,
                    map_ref_id,
                );
                continue;
            }

            if let Some(size) = fallback_marker_size(object.object_type, object.object_id) {
                let mut entity = commands.spawn((
                    Sprite::from_color(object_color(owner, object.object_type), size),
                    Transform::from_xyz(x, y, 5.0),
                    ObjectLayerRef(ref_id),
                    object_name(map_ref_id, group_member, object, false),
                    components,
                ));
                if let Some(group) = robot_group_for(kind, ref_id, group_leader_ref_id) {
                    entity.insert(group);
                }
                if let Some(selectable) = selectable_for(kind, size) {
                    entity.insert(selectable);
                }
                if let Some(footprint) = bridge_footprint_for(object, kind) {
                    entity.insert(footprint);
                }
                insert_grenade_components(&mut entity, kind);
                insert_hut_components(&mut entity, kind);
                if let Some(stamina) = initial_movement_stamina(kind) {
                    entity.insert(stamina);
                }
                if vehicle_drops_effects(kind) {
                    entity.insert(VehicleEffectDropTimer { elapsed: 0.2 });
                }
                if let Some(driver) = initial_driver_health(kind, owner) {
                    entity.insert(driver);
                }
                if let Some(production) = production.take() {
                    entity.insert(production);
                }
                fallback_count += 1;
            } else {
                let mut entity = commands.spawn((
                    Transform::from_xyz(x, y, 5.0),
                    ObjectLayerRef(ref_id),
                    object_name(map_ref_id, group_member, object, false),
                    components,
                ));
                if let Some(group) = robot_group_for(kind, ref_id, group_leader_ref_id) {
                    entity.insert(group);
                }
                if let Some(production) = production.take() {
                    entity.insert(production);
                }
                insert_grenade_components(&mut entity, kind);
                insert_hut_components(&mut entity, kind);
                if let Some(stamina) = initial_movement_stamina(kind) {
                    entity.insert(stamina);
                }
                if vehicle_drops_effects(kind) {
                    entity.insert(VehicleEffectDropTimer { elapsed: 0.2 });
                }
                hidden_count += 1;
            }
        }
    }

    let object_count = next_ref_id - 1;
    println!(
        "Object render: {sprite_count} atlas sprites, {fallback_count} fallback markers, {hidden_count} hidden pending renderers, {object_count} gameplay objects"
    );
    next_ref_id
}

pub(crate) fn spawn_runtime_object(
    commands: &mut Commands,
    atlases: &GameAtlases,
    planet: PlanetType,
    hud_layout: &HudLayout,
    ref_id: u32,
    kind: ObjectKind,
    team: TeamType,
    create_center: Vec2,
    health_percent: i32,
    cannon_ejectable: bool,
    just_left_cannon: bool,
    move_target: Option<Vec2>,
    robot_group_leader_ref_id: Option<u32>,
) -> bool {
    let Some((object_type, object_id)) = object_kind_to_map_parts(kind) else {
        return false;
    };
    let object = MapObject {
        x: (create_center.x / TILE_SIZE).floor().max(0.0) as u16,
        y: (-create_center.y / TILE_SIZE).floor().max(0.0) as u16,
        owner: team,
        object_type,
        object_id,
        building_level: 0,
        extra_links: 0,
        health_percent,
    };
    let mut stats = ObjectStats::from_kind(kind, object.health_percent);
    if matches!(kind, ObjectKind::Cannon(_)) {
        stats.cannon_ejectable = cannon_ejectable;
    }
    let layers = atlases.sprite_layers_for_object(&object, planet);
    let top_left_map = layers
        .first()
        .map(|frame| Vec2::new(create_center.x, -create_center.y) - frame.frame_size * 0.5);

    if !layers.is_empty() {
        let top_left_map = top_left_map.unwrap();
        let mut base_position = create_center;
        for (layer_index, frame) in layers.into_iter().enumerate() {
            let layer_position = sprite_position_from_map_top_left(top_left_map, &frame);
            if layer_index == 0 {
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

            if layer_index == 0 {
                entity.insert((
                    GameObjectEntity { ref_id, kind },
                    ObjectTeam(team),
                    HealthPercent(object.health_percent),
                    DamageCauseTimers::default(),
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
                if let Some(stamina) = initial_movement_stamina(kind) {
                    entity.insert(stamina);
                }
                if vehicle_drops_effects(kind) {
                    entity.insert(VehicleEffectDropTimer { elapsed: 0.2 });
                }
                if just_left_cannon {
                    entity.insert(JustLeftCannon);
                }
                if let Some(driver) = initial_driver_health(kind, team) {
                    entity.insert(driver);
                }
            }

            if let Some(role) = mobile_sprite_role(kind, layer_index) {
                entity.insert(MobileSpriteLayer {
                    kind,
                    team,
                    role,
                    rotation: 180,
                    frame: 0,
                    elapsed: 0.0,
                });
            }

            if let Some(move_target) = move_target {
                if stats.move_speed > 0.0 {
                    let layer_offset = layer_position - base_position;
                    entity.insert(MovementPath::new(
                        vec![move_target + layer_offset],
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
    } else if let Some(size) = fallback_marker_size(object_type, object_id) {
        let mut entity = commands.spawn((
            Sprite::from_color(object_color(team, object_type), size),
            Transform::from_xyz(create_center.x, create_center.y, 5.0),
            ObjectLayerRef(ref_id),
            Name::new(format!("runtime #{ref_id} {:?}", kind)),
            GameObjectEntity { ref_id, kind },
            ObjectTeam(team),
            HealthPercent(object.health_percent),
            DamageCauseTimers::default(),
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
        if let Some(selectable) = selectable_for(kind, size) {
            entity.insert(selectable);
        }
        if let Some(footprint) = bridge_footprint_for(&object, kind) {
            entity.insert(footprint);
        }
        insert_grenade_components(&mut entity, kind);
        insert_hut_components(&mut entity, kind);
        if let Some(stamina) = initial_movement_stamina(kind) {
            entity.insert(stamina);
        }
        if vehicle_drops_effects(kind) {
            entity.insert(VehicleEffectDropTimer { elapsed: 0.2 });
        }
        if let Some(move_target) = move_target {
            if stats.move_speed > 0.0 {
                entity.insert(MovementPath::new(vec![move_target], stats.move_speed));
            }
        }
    } else {
        return false;
    }

    spawn_minimap_dot(commands, ref_id, create_center, team, *hud_layout);
    true
}

fn insert_grenade_components(entity: &mut EntityCommands, kind: ObjectKind) {
    match kind {
        ObjectKind::Robot(_) => {
            entity.insert(GrenadeInventory { amount: 0 });
        }
        ObjectKind::MapItem(id) if id == ItemType::Grenades as u8 => {
            entity.insert(GrenadeBox {
                amount: GRENADES_PER_BOX,
            });
        }
        _ => {}
    }
}

fn insert_hut_components(entity: &mut EntityCommands, kind: ObjectKind) {
    if matches!(kind, ObjectKind::MapItem(id) if id == ItemType::Hut as u8) {
        entity.insert(HutAnimalSpawner {
            max_animals: 0,
            animal_timer: 0.0,
            max_timer: 0.0,
        });
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
    if kind != ObjectKind::Building(BuildingType::Radar) {
        return;
    }

    let top_left_map = Vec2::new(object.x as f32 * TILE_SIZE, object.y as f32 * TILE_SIZE);
    let kinds = [
        RadarOverlayKind::FrontLight,
        RadarOverlayKind::SideLight,
        RadarOverlayKind::BoxSpinner,
        RadarOverlayKind::Dish,
    ];

    for (layer_index, overlay_kind) in kinds.into_iter().enumerate() {
        let Some(frames) = atlases.radar_overlay_frames(overlay_kind) else {
            continue;
        };
        let Some(first) = frames.first().cloned() else {
            continue;
        };
        let (sprite_x, sprite_y) = sprite_position(object.x, object.y, &first);
        let visible = radar_overlay_visible(overlay_kind, owner, stats);

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
                frame_time: 0.25,
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

fn radar_overlay_visible(kind: RadarOverlayKind, owner: TeamType, stats: ObjectStats) -> bool {
    !stats.destroyed() && (owner != TeamType::Null || matches!(kind, RadarOverlayKind::FrontLight))
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
    if kind != ObjectKind::Building(BuildingType::Repair) {
        return;
    }

    let top_left_map = Vec2::new(object.x as f32 * TILE_SIZE, object.y as f32 * TILE_SIZE);
    let kinds = [
        RepairOverlayKind::TextBox,
        RepairOverlayKind::Bulb,
        RepairOverlayKind::SmokeStack,
        RepairOverlayKind::FrontLight,
        RepairOverlayKind::SideLight,
    ];

    for (layer_index, overlay_kind) in kinds.into_iter().enumerate() {
        let Some(frames) = atlases.repair_overlay_frames(overlay_kind) else {
            continue;
        };
        let current = repair_overlay_initial_frame(overlay_kind, owner, false);
        let Some(first) = frames
            .get(current.min(frames.len().saturating_sub(1)))
            .cloned()
        else {
            continue;
        };
        let (sprite_x, sprite_y) = sprite_position(object.x, object.y, &first);
        let visible = repair_overlay_visible(overlay_kind, owner, stats, false);

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
                frame_time: 0.35,
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

fn repair_overlay_visible(
    kind: RepairOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    repairing_unit: bool,
) -> bool {
    if stats.destroyed() {
        return false;
    }

    if owner == TeamType::Null {
        return matches!(kind, RepairOverlayKind::SmokeStack);
    }

    match kind {
        RepairOverlayKind::SmokeStack => repairing_unit,
        RepairOverlayKind::FrontLight
        | RepairOverlayKind::SideLight
        | RepairOverlayKind::Bulb
        | RepairOverlayKind::TextBox => true,
    }
}

fn repair_overlay_initial_frame(
    kind: RepairOverlayKind,
    owner: TeamType,
    repairing_unit: bool,
) -> usize {
    match kind {
        RepairOverlayKind::FrontLight | RepairOverlayKind::SideLight => 1,
        RepairOverlayKind::Bulb | RepairOverlayKind::SmokeStack
            if owner == TeamType::Null || !repairing_unit =>
        {
            0
        }
        _ => 0,
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
    let overlay_kinds: &[FactoryOverlayKind] = match kind {
        ObjectKind::Building(BuildingType::RobotFactory) => &[
            FactoryOverlayKind::RobotExhaust,
            FactoryOverlayKind::RobotGreenBox,
            FactoryOverlayKind::RobotSingleLight0,
            FactoryOverlayKind::RobotSingleLight1,
            FactoryOverlayKind::RobotSingleLight2,
            FactoryOverlayKind::RobotDoubleLight,
            FactoryOverlayKind::RobotBody,
            FactoryOverlayKind::RobotSpin,
        ],
        ObjectKind::Building(BuildingType::VehicleFactory) => &[
            FactoryOverlayKind::VehicleExhaust,
            FactoryOverlayKind::VehicleTank,
            FactoryOverlayKind::VehicleVent,
            FactoryOverlayKind::VehicleBulb,
            FactoryOverlayKind::VehicleLight0,
            FactoryOverlayKind::VehicleLight1,
            FactoryOverlayKind::VehicleSpin,
        ],
        _ => return,
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
        let visible = factory_overlay_visible(overlay_kind, owner, stats, production_active)
            && factory_overlay_initial_frame_visible(overlay_kind);

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
                frame_time: 0.25,
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

fn factory_overlay_visible(
    kind: FactoryOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    production_active: bool,
) -> bool {
    if stats.destroyed() {
        return false;
    }

    if owner == TeamType::Null || !production_active {
        return matches!(
            kind,
            FactoryOverlayKind::RobotBody | FactoryOverlayKind::VehicleTank
        );
    }

    true
}

fn factory_overlay_initial_frame_visible(kind: FactoryOverlayKind) -> bool {
    !matches!(
        kind,
        FactoryOverlayKind::RobotSingleLight0
            | FactoryOverlayKind::RobotSingleLight1
            | FactoryOverlayKind::RobotSingleLight2
    )
}

fn normalized_object_owner(kind: ObjectKind, owner: TeamType) -> TeamType {
    match kind {
        ObjectKind::Rock => TeamType::Null,
        ObjectKind::MapItem(id) if id != ItemType::Flag as u8 => TeamType::Null,
        _ => owner,
    }
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

pub(crate) fn initial_movement_stamina(kind: ObjectKind) -> Option<MovementStamina> {
    let max = unit_settings(kind)?.max_run_time;
    (max > 0.0).then_some(MovementStamina {
        max,
        current: max,
        running: false,
    })
}

fn vehicle_drops_effects(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Vehicle(
            VehicleType::Jeep
                | VehicleType::Light
                | VehicleType::Medium
                | VehicleType::Heavy
                | VehicleType::Apc
                | VehicleType::MissileLauncher
                | VehicleType::Crane
        )
    )
}

pub(crate) fn robot_group_for(
    kind: ObjectKind,
    ref_id: u32,
    leader_ref_id: Option<u32>,
) -> Option<RobotGroup> {
    matches!(kind, ObjectKind::Robot(_)).then_some(RobotGroup {
        leader_ref_id: leader_ref_id.unwrap_or(ref_id),
    })
}

pub(crate) fn initial_driver_health(kind: ObjectKind, team: TeamType) -> Option<DriverHealth> {
    (team != TeamType::Null && can_have_driver(kind)).then_some(grunt_driver_health())
}

pub(crate) fn grunt_driver_health() -> DriverHealth {
    let max_health = object_max_health(ObjectKind::Robot(RobotType::Grunt));
    DriverHealth::new(RobotType::Grunt, max_health)
}

pub(crate) fn can_have_driver(kind: ObjectKind) -> bool {
    matches!(kind, ObjectKind::Vehicle(_) | ObjectKind::Cannon(_))
}

pub(crate) fn object_kind_to_map_parts(kind: ObjectKind) -> Option<(MapObjectType, u8)> {
    match kind {
        ObjectKind::Bridge(building) => Some((MapObjectType::Bridge, building as u8)),
        ObjectKind::Building(building) => Some((MapObjectType::Building, building as u8)),
        ObjectKind::Cannon(cannon) => Some((MapObjectType::Cannon, cannon as u8)),
        ObjectKind::Vehicle(vehicle) => Some((MapObjectType::Vehicle, vehicle as u8)),
        ObjectKind::Robot(robot) => Some((MapObjectType::Robot, robot as u8)),
        ObjectKind::Animal(id) => Some((MapObjectType::Animal, id)),
        ObjectKind::MapItem(id) => Some((MapObjectType::MapItem, id)),
        ObjectKind::Rock => Some((MapObjectType::MapItem, ItemType::Rock as u8)),
    }
}

fn sprite_position_from_map_top_left(top_left_map: Vec2, frame: &SpriteFrame) -> Vec2 {
    Vec2::new(
        top_left_map.x + frame.world_offset.x + frame.source_offset.x + frame.frame_size.x * 0.5,
        -(top_left_map.y + frame.world_offset.y + frame.source_offset.y + frame.frame_size.y * 0.5),
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
    match kind {
        ObjectKind::Robot(_) if layer_index == 0 => Some(MobileSpriteRole::Robot),
        ObjectKind::Vehicle(_) if layer_index == 0 => Some(MobileSpriteRole::VehicleBase),
        ObjectKind::Vehicle(_) if layer_index == 1 => Some(MobileSpriteRole::VehicleTop),
        _ => None,
    }
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

fn object_name(
    map_ref_id: usize,
    group_member: u32,
    object: &MapObject,
    atlas_sprite: bool,
) -> Name {
    Name::new(format!(
        "#{}.{} {:?}:{:?} team {:?} level {} extra {} hp {} {}",
        map_ref_id + 1,
        group_member + 1,
        object.object_type,
        object.object_id,
        object.owner,
        object.building_level,
        object.extra_links,
        object.health_percent,
        if atlas_sprite { "atlas" } else { "fallback" }
    ))
}

fn object_layer_name(
    map_ref_id: usize,
    group_member: u32,
    object: &MapObject,
    layer_index: usize,
) -> Name {
    Name::new(format!(
        "#{}.{} {:?}:{:?} team {:?} layer {} atlas",
        map_ref_id + 1,
        group_member + 1,
        object.object_type,
        object.object_id,
        object.owner,
        layer_index,
    ))
}

fn fallback_marker_size(object_type: MapObjectType, object_id: u8) -> Option<Vec2> {
    if object_type == MapObjectType::MapItem && object_id == ItemType::Rock as u8 {
        return None;
    }

    match object_type {
        MapObjectType::Building | MapObjectType::Bridge => Some(Vec2::splat(18.0)),
        MapObjectType::Vehicle => Some(Vec2::splat(10.0)),
        MapObjectType::Robot => Some(Vec2::splat(7.0)),
        MapObjectType::Cannon => Some(Vec2::splat(12.0)),
        _ => Some(Vec2::splat(6.0)),
    }
}

fn object_color(team: TeamType, object_type: MapObjectType) -> Color {
    if team != TeamType::Null {
        return team.color();
    }

    match object_type {
        MapObjectType::Building | MapObjectType::Bridge => Color::srgb(0.75, 0.75, 0.65),
        MapObjectType::MapItem => Color::srgb(0.1, 0.9, 0.2),
        MapObjectType::Rock => Color::srgb(0.45, 0.42, 0.38),
        _ => Color::srgb(0.9, 0.9, 0.9),
    }
}

fn selectable_for(kind: ObjectKind, selection_size: Vec2) -> Option<Selectable> {
    match kind {
        ObjectKind::Robot(_) => Some(Selectable {
            radius: 10.0,
            selection_size,
            mobile: true,
        }),
        ObjectKind::Vehicle(_) => Some(Selectable {
            radius: 16.0,
            selection_size,
            mobile: true,
        }),
        ObjectKind::Cannon(_) => Some(Selectable {
            radius: 18.0,
            selection_size,
            mobile: false,
        }),
        ObjectKind::Building(_) | ObjectKind::Bridge(_) => Some(Selectable {
            radius: 42.0,
            selection_size,
            mobile: false,
        }),
        ObjectKind::Rock | ObjectKind::Animal(_) | ObjectKind::MapItem(_) => None,
    }
}
