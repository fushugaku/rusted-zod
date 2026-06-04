use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use crate::original::map::MapObject;
use crate::original::objects::{BuildingType, CannonType, ItemType, ObjectKind, VehicleType};
use crate::original::types::{PlanetType, TeamType};

const DEFAULT_ROTATION: &str = "180";

#[derive(Resource)]
pub struct GameAtlases {
    robots: HashMap<TeamType, NamedAtlas>,
    vehicles: HashMap<TeamType, NamedAtlas>,
    cannons: HashMap<TeamType, NamedAtlas>,
    buildings: HashMap<TeamType, NamedAtlas>,
    bridges: BridgeAtlases,
    map_items: NamedAtlas,
}

#[derive(Clone)]
pub struct SpriteFrame {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub index: usize,
    pub animation_indices: Option<Vec<usize>>,
    pub world_offset: Vec2,
    pub source_offset: Vec2,
    pub frame_size: Vec2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MobileSpriteRole {
    Robot,
    VehicleBase,
    VehicleTop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeVisualState {
    Live,
    Damaged,
    Destroyed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadarOverlayKind {
    FrontLight,
    SideLight,
    BoxSpinner,
    Dish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepairOverlayKind {
    FrontLight,
    SideLight,
    Bulb,
    SmokeStack,
    TextBox,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactoryOverlayKind {
    RobotExhaust,
    RobotGreenBox,
    RobotSingleLight0,
    RobotSingleLight1,
    RobotSingleLight2,
    RobotDoubleLight,
    RobotBody,
    RobotSpin,
    VehicleExhaust,
    VehicleTank,
    VehicleVent,
    VehicleBulb,
    VehicleLight0,
    VehicleLight1,
    VehicleSpin,
}

struct NamedAtlas {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    frames: HashMap<String, AtlasFrameInfo>,
}

struct AtlasFrameInfo {
    index: usize,
    source_offset: Vec2,
    frame_size: Vec2,
}

struct BridgeAtlases {
    desert: Handle<Image>,
    volcanic: Handle<Image>,
    arctic: Handle<Image>,
    jungle: Handle<Image>,
    city: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

impl GameAtlases {
    pub fn build(asset_server: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>) -> Self {
        let teams = [
            TeamType::Red,
            TeamType::Blue,
            TeamType::Green,
            TeamType::Yellow,
        ];

        let mut robots = HashMap::new();
        let mut vehicles = HashMap::new();
        let mut cannons = HashMap::new();
        let mut buildings = HashMap::new();
        for team in teams {
            let name = team.asset_name();
            robots.insert(
                team,
                load_named_atlas(
                    asset_server,
                    layouts,
                    &format!("atlases/robots_{name}.png"),
                    atlas_json("robots", team),
                ),
            );
            vehicles.insert(
                team,
                load_named_atlas(
                    asset_server,
                    layouts,
                    &format!("atlases/vehicles_{name}.png"),
                    atlas_json("vehicles", team),
                ),
            );
            cannons.insert(
                team,
                load_named_atlas(
                    asset_server,
                    layouts,
                    &format!("atlases/cannons_{name}.png"),
                    atlas_json("cannons", team),
                ),
            );
            buildings.insert(
                team,
                load_named_atlas(
                    asset_server,
                    layouts,
                    &format!("atlases/buildings_{name}.png"),
                    atlas_json("buildings", team),
                ),
            );
        }

        Self {
            robots,
            vehicles,
            cannons,
            buildings,
            bridges: BridgeAtlases::build(asset_server, layouts),
            map_items: load_named_atlas(
                asset_server,
                layouts,
                "atlases/map_items.png",
                include_str!("../../assets/atlases/map_items.json"),
            ),
        }
    }

    pub fn sprite_layers_for_object(
        &self,
        object: &MapObject,
        planet: PlanetType,
    ) -> Vec<SpriteFrame> {
        let Ok(kind) = ObjectKind::from_map_parts(object.object_type, object.object_id) else {
            return Vec::new();
        };
        let team = object.owner.atlas_team();
        let team_name = team.asset_name();
        let planet_name = planet.asset_name();

        match kind {
            ObjectKind::Robot(_) => self
                .robots
                .get(&team)
                .and_then(|atlas| {
                    atlas.frame(&format!("robot_stand_{team_name}_r{DEFAULT_ROTATION}"))
                })
                .into_iter()
                .collect(),
            ObjectKind::Vehicle(vehicle_type) => {
                let mut layers = Vec::with_capacity(2);
                if let Some(frame) = self.vehicles.get(&team).and_then(|atlas| {
                    atlas.frame(&format!(
                        "vehicle_{}_base_{team_name}_r{DEFAULT_ROTATION}_n00",
                        vehicle_type.folder()
                    ))
                }) {
                    layers.push(frame);
                }

                if let Some(frame) = self.vehicle_top_frame(vehicle_type, team) {
                    layers.push(frame);
                }

                layers
            }
            ObjectKind::Cannon(cannon_type) => {
                let (atlas_team, frame) = match cannon_type {
                    CannonType::Gatling => (
                        TeamType::Red,
                        format!("cannon_gatling_empty_r{DEFAULT_ROTATION}"),
                    ),
                    CannonType::Gun => (
                        team,
                        format!("cannon_gun_equiped_{team_name}_r{DEFAULT_ROTATION}"),
                    ),
                    CannonType::Howitzer => (
                        TeamType::Red,
                        format!("cannon_howitzer_empty_r{DEFAULT_ROTATION}"),
                    ),
                    CannonType::MissileCannon => (
                        team,
                        format!("cannon_missile_cannon_equiped_{team_name}_r{DEFAULT_ROTATION}"),
                    ),
                };
                self.cannons
                    .get(&atlas_team)
                    .and_then(|atlas| atlas.frame(&frame))
                    .into_iter()
                    .collect()
            }
            ObjectKind::Building(building_type) => match building_type {
                BuildingType::BridgeVert | BuildingType::BridgeHorz => self.bridge_layers(
                    building_type,
                    planet,
                    object.extra_links,
                    bridge_visual_state(object.health_percent),
                ),
                _ => self.building_layers(building_type, team, planet_name),
            },
            ObjectKind::Bridge(building_type) => self.bridge_layers(
                building_type,
                planet,
                object.extra_links,
                bridge_visual_state(object.health_percent),
            ),
            ObjectKind::MapItem(item_id) => {
                if item_id == ItemType::Flag as u8 {
                    let team_name = object.owner.asset_name();
                    let flag_frames = [
                        format!("flag_{team_name}_0"),
                        format!("flag_{team_name}_1"),
                        format!("flag_{team_name}_2"),
                        format!("flag_{team_name}_3"),
                    ];
                    return self
                        .map_items
                        .frame_with_world_offset_and_animation(
                            &flag_frames[0],
                            Vec2::ZERO,
                            &flag_frames,
                        )
                        .into_iter()
                        .collect();
                }

                let frame = match item_id {
                    id if id == ItemType::Grenades as u8 => "item_grenades".to_string(),
                    id if id == ItemType::Rockets as u8 => "item_rockets".to_string(),
                    id if id == ItemType::Hut as u8 => format!("hut_{planet_name}"),
                    id if id >= ItemType::MapObjectStart as u8 => {
                        format!("map_object{}", id - ItemType::MapObjectStart as u8)
                    }
                    _ => return Vec::new(),
                };
                self.map_items.frame(&frame).into_iter().collect()
            }
            ObjectKind::Rock | ObjectKind::Animal(_) => Vec::new(),
        }
    }

    fn vehicle_top_frame(&self, vehicle_type: VehicleType, team: TeamType) -> Option<SpriteFrame> {
        self.vehicle_top_frame_for_rotation(vehicle_type, team, DEFAULT_ROTATION)
    }

    pub fn mobile_frame(
        &self,
        kind: ObjectKind,
        team: TeamType,
        role: MobileSpriteRole,
        rotation: u16,
        frame: usize,
        moving: bool,
    ) -> Option<SpriteFrame> {
        let team = team.atlas_team();
        let team_name = team.asset_name();
        let rotation = format!("{rotation:03}");

        match (kind, role) {
            (ObjectKind::Robot(_), MobileSpriteRole::Robot) => {
                let name = if moving {
                    format!("robot_walk_{team_name}_r{rotation}_n{:02}", frame % 4)
                } else {
                    format!("robot_stand_{team_name}_r{rotation}")
                };
                self.robots.get(&team).and_then(|atlas| atlas.frame(&name))
            }
            (ObjectKind::Vehicle(vehicle_type), MobileSpriteRole::VehicleBase) => {
                let frame_count = vehicle_move_frame_count(vehicle_type);
                let name = format!(
                    "vehicle_{}_base_{team_name}_r{rotation}_n{:02}",
                    vehicle_type.folder(),
                    frame % frame_count
                );
                self.vehicles
                    .get(&team)
                    .and_then(|atlas| atlas.frame(&name))
            }
            (ObjectKind::Vehicle(vehicle_type), MobileSpriteRole::VehicleTop) => {
                self.vehicle_top_frame_for_rotation(vehicle_type, team, &rotation)
            }
            _ => None,
        }
    }

    pub fn mobile_frame_index(
        &self,
        kind: ObjectKind,
        team: TeamType,
        role: MobileSpriteRole,
        rotation: u16,
        frame: usize,
        moving: bool,
    ) -> Option<usize> {
        self.mobile_frame(kind, team, role, rotation, frame, moving)
            .map(|frame| frame.index)
    }

    pub fn captured_cannon_frame(
        &self,
        cannon_type: CannonType,
        team: TeamType,
        rotation: u16,
    ) -> Option<SpriteFrame> {
        let team = team.atlas_team();
        let team_name = team.asset_name();
        let rotation = format!("{rotation:03}");
        let frame = match cannon_type {
            CannonType::Gatling => format!("cannon_gatling_empty_r{rotation}"),
            CannonType::Gun => format!("cannon_gun_equiped_{team_name}_r{rotation}"),
            CannonType::Howitzer => format!("cannon_howitzer_empty_r{rotation}"),
            CannonType::MissileCannon => {
                format!("cannon_missile_cannon_equiped_{team_name}_r{rotation}")
            }
        };
        let atlas_team = match cannon_type {
            CannonType::Gatling | CannonType::Howitzer => TeamType::Red,
            CannonType::Gun | CannonType::MissileCannon => team,
        };

        self.cannons
            .get(&atlas_team)
            .and_then(|atlas| atlas.frame(&frame))
    }

    pub fn flag_animation_indices(&self, team: TeamType) -> Option<Vec<usize>> {
        let team_name = team.asset_name();
        let frames = [
            format!("flag_{team_name}_0"),
            format!("flag_{team_name}_1"),
            format!("flag_{team_name}_2"),
            format!("flag_{team_name}_3"),
        ];
        self.map_items.animation_indices(&frames)
    }

    pub fn radar_overlay_frames(&self, kind: RadarOverlayKind) -> Option<Vec<SpriteFrame>> {
        let atlas = self.buildings.get(&TeamType::Red)?;
        radar_overlay_frame_names(kind)
            .iter()
            .zip(radar_overlay_offsets(kind).iter())
            .map(|(name, offset)| atlas.frame_with_world_offset(name, *offset))
            .collect()
    }

    pub fn repair_overlay_frames(&self, kind: RepairOverlayKind) -> Option<Vec<SpriteFrame>> {
        let atlas = self.buildings.get(&TeamType::Red)?;
        repair_overlay_frame_names(kind)
            .iter()
            .zip(repair_overlay_offsets(kind).iter())
            .map(|(name, offset)| atlas.frame_with_world_offset(name, *offset))
            .collect()
    }

    pub fn factory_overlay_frames(&self, kind: FactoryOverlayKind) -> Option<Vec<SpriteFrame>> {
        let atlas = self.buildings.get(&TeamType::Red)?;
        factory_overlay_frame_names(kind)
            .iter()
            .zip(factory_overlay_offsets(kind).iter())
            .map(|(name, offset)| atlas.frame_with_world_offset(name, *offset))
            .collect()
    }

    fn vehicle_top_frame_for_rotation(
        &self,
        vehicle_type: VehicleType,
        team: TeamType,
        rotation: &str,
    ) -> Option<SpriteFrame> {
        let team_name = team.asset_name();

        match vehicle_type {
            VehicleType::Light => self
                .vehicles
                .get(&TeamType::Red)
                .and_then(|atlas| atlas.frame(&format!("vehicle_light_top_r{rotation}"))),
            VehicleType::Medium => self
                .vehicles
                .get(&TeamType::Red)
                .and_then(|atlas| atlas.frame(&format!("vehicle_medium_top_r{rotation}"))),
            VehicleType::Heavy => self.vehicles.get(&team).and_then(|atlas| {
                atlas.frame(&format!("vehicle_heavy_top_{team_name}_r{rotation}"))
            }),
            VehicleType::Apc => self
                .vehicles
                .get(&TeamType::Red)
                .and_then(|atlas| atlas.frame(&format!("vehicle_apc_top_r{rotation}"))),
            VehicleType::MissileLauncher => self.vehicles.get(&team).and_then(|atlas| {
                atlas.frame(&format!(
                    "vehicle_missile_launcher_top_{team_name}_r{rotation}"
                ))
            }),
            VehicleType::Jeep | VehicleType::Crane => None,
        }
    }

    fn building_layers(
        &self,
        building_type: BuildingType,
        team: TeamType,
        planet_name: &str,
    ) -> Vec<SpriteFrame> {
        let Some(base_atlas) = self.buildings.get(&TeamType::Red) else {
            return Vec::new();
        };
        let team_name = team.asset_name();
        let mut layers = Vec::with_capacity(3);

        match building_type {
            BuildingType::FortFront | BuildingType::FortBack => {
                let side = match building_type {
                    BuildingType::FortFront => "front",
                    BuildingType::FortBack => "back",
                    _ => unreachable!(),
                };

                if let Some(frame) = base_atlas.frame(&format!("fort_{planet_name}_{side}")) {
                    layers.push(frame);
                }

                let fort_flag_frames = [
                    format!("fort_flag_{team_name}_n00"),
                    format!("fort_flag_{team_name}_n01"),
                    format!("fort_flag_{team_name}_n02"),
                    format!("fort_flag_{team_name}_n03"),
                ];
                if let Some(flag) = self.buildings.get(&team).and_then(|atlas| {
                    atlas.frame_with_world_offset_and_animation(
                        &fort_flag_frames[0],
                        Vec2::new(85.0, 29.0),
                        &fort_flag_frames,
                    )
                }) {
                    layers.push(flag);
                }
            }
            BuildingType::Radar => {
                push_building_base_and_overlay(
                    &mut layers,
                    base_atlas,
                    self.buildings.get(&team),
                    &format!("building_radar_base_{planet_name}"),
                    &format!("building_radar_{team_name}"),
                    Vec2::new(0.0, 32.0),
                );
            }
            BuildingType::Repair => {
                push_building_base_and_overlay(
                    &mut layers,
                    base_atlas,
                    self.buildings.get(&team),
                    &format!("building_repair_base_{planet_name}"),
                    &format!("building_repair_{team_name}"),
                    Vec2::new(0.0, 48.0),
                );
            }
            BuildingType::RobotFactory => {
                push_building_base_and_overlay(
                    &mut layers,
                    base_atlas,
                    self.buildings.get(&team),
                    &format!("building_robot_base_{planet_name}"),
                    &format!("building_robot_{team_name}"),
                    Vec2::new(16.0, 64.0),
                );
            }
            BuildingType::VehicleFactory => {
                push_building_base_and_overlay(
                    &mut layers,
                    base_atlas,
                    self.buildings.get(&team),
                    &format!("building_vehicle_base_{planet_name}"),
                    &format!("building_vehicle_{team_name}"),
                    Vec2::new(32.0, 48.0),
                );
            }
            BuildingType::BridgeVert | BuildingType::BridgeHorz => {}
        }

        layers
    }

    pub fn bridge_layers(
        &self,
        building_type: BuildingType,
        planet: PlanetType,
        extra_links: u16,
        state: BridgeVisualState,
    ) -> Vec<SpriteFrame> {
        match building_type {
            BuildingType::BridgeVert => self.vertical_bridge_layers(planet, extra_links, state),
            BuildingType::BridgeHorz => self.horizontal_bridge_layers(planet, extra_links, state),
            _ => Vec::new(),
        }
    }

    fn vertical_bridge_layers(
        &self,
        planet: PlanetType,
        extra_links: u16,
        state: BridgeVisualState,
    ) -> Vec<SpriteFrame> {
        let image = self.bridges.image(planet);
        let total_rows = 5 + extra_links as usize;
        let mut layers = Vec::with_capacity(total_rows);

        for row in 0..total_rows {
            let index = match row {
                0 => BRIDGE_VERT_TOP,
                1 => BRIDGE_VERT_SECOND,
                r if r == total_rows - 2 => BRIDGE_VERT_PENULTIMATE,
                r if r == total_rows - 1 => BRIDGE_VERT_BOTTOM,
                _ => bridge_vert_fill_index(state, row),
            };
            layers.push(bridge_frame(
                image.clone(),
                self.bridges.layout.clone(),
                index,
                Vec2::new(0.0, row as f32 * 16.0),
                Vec2::new(64.0, 16.0),
            ));
        }

        layers
    }

    fn horizontal_bridge_layers(
        &self,
        planet: PlanetType,
        extra_links: u16,
        state: BridgeVisualState,
    ) -> Vec<SpriteFrame> {
        let image = self.bridges.image(planet);
        let total_cols = 5 + extra_links as usize;
        let mut layers = Vec::with_capacity(total_cols);

        for col in 0..total_cols {
            let index = match col {
                0 => BRIDGE_HORZ_LEFT,
                1 => BRIDGE_HORZ_SECOND,
                c if c == total_cols - 2 => BRIDGE_HORZ_PENULTIMATE,
                c if c == total_cols - 1 => BRIDGE_HORZ_RIGHT,
                _ => bridge_horz_fill_index(state, col),
            };
            layers.push(bridge_frame(
                image.clone(),
                self.bridges.layout.clone(),
                index,
                Vec2::new(col as f32 * 16.0, 0.0),
                Vec2::new(16.0, 64.0),
            ));
        }

        layers
    }
}

const BRIDGE_VERT_TOP: usize = 0;
const BRIDGE_VERT_SECOND: usize = 1;
const BRIDGE_VERT_PENULTIMATE: usize = 2;
const BRIDGE_VERT_BOTTOM: usize = 3;
const BRIDGE_VERT_FILL_LIVE: usize = 4;
const BRIDGE_VERT_FILL_DAMAGED_0: usize = 5;
const BRIDGE_VERT_FILL_DAMAGED_1: usize = 6;
const BRIDGE_VERT_FILL_DESTROYED: usize = 7;
const BRIDGE_HORZ_LEFT: usize = 8;
const BRIDGE_HORZ_SECOND: usize = 9;
const BRIDGE_HORZ_PENULTIMATE: usize = 10;
const BRIDGE_HORZ_RIGHT: usize = 11;
const BRIDGE_HORZ_FILL_LIVE: usize = 12;
const BRIDGE_HORZ_FILL_DAMAGED_0: usize = 13;
const BRIDGE_HORZ_FILL_DAMAGED_1: usize = 14;
const BRIDGE_HORZ_FILL_DESTROYED: usize = 15;

fn bridge_visual_state(health_percent: i32) -> BridgeVisualState {
    if health_percent <= 0 {
        BridgeVisualState::Destroyed
    } else if health_percent < 50 {
        BridgeVisualState::Damaged
    } else {
        BridgeVisualState::Live
    }
}

fn bridge_vert_fill_index(state: BridgeVisualState, row: usize) -> usize {
    match state {
        BridgeVisualState::Live => BRIDGE_VERT_FILL_LIVE,
        BridgeVisualState::Damaged if row % 2 == 0 => BRIDGE_VERT_FILL_DAMAGED_0,
        BridgeVisualState::Damaged => BRIDGE_VERT_FILL_DAMAGED_1,
        BridgeVisualState::Destroyed => BRIDGE_VERT_FILL_DESTROYED,
    }
}

fn bridge_horz_fill_index(state: BridgeVisualState, col: usize) -> usize {
    match state {
        BridgeVisualState::Live => BRIDGE_HORZ_FILL_LIVE,
        BridgeVisualState::Damaged if col % 2 == 0 => BRIDGE_HORZ_FILL_DAMAGED_0,
        BridgeVisualState::Damaged => BRIDGE_HORZ_FILL_DAMAGED_1,
        BridgeVisualState::Destroyed => BRIDGE_HORZ_FILL_DESTROYED,
    }
}

fn vehicle_move_frame_count(vehicle_type: VehicleType) -> usize {
    match vehicle_type {
        VehicleType::Jeep => 2,
        _ => 3,
    }
}

impl BridgeAtlases {
    fn build(asset_server: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>) -> Self {
        let mut layout = TextureAtlasLayout::new_empty(UVec2::new(64, 256));

        for row in [0, 16, 32, 48, 64, 80, 96, 112] {
            layout.add_texture(URect {
                min: UVec2::new(0, row),
                max: UVec2::new(64, row + 16),
            });
        }

        for col in [0, 16, 32, 48] {
            layout.add_texture(URect {
                min: UVec2::new(col, 128),
                max: UVec2::new(col + 16, 192),
            });
        }

        layout.add_texture(URect {
            min: UVec2::new(0, 192),
            max: UVec2::new(16, 256),
        });
        layout.add_texture(URect {
            min: UVec2::new(16, 192),
            max: UVec2::new(32, 256),
        });
        layout.add_texture(URect {
            min: UVec2::new(32, 192),
            max: UVec2::new(48, 256),
        });
        layout.add_texture(URect {
            min: UVec2::new(48, 192),
            max: UVec2::new(64, 256),
        });

        Self {
            desert: asset_server.load("planets/bridge_desert.png"),
            volcanic: asset_server.load("planets/bridge_volcanic.png"),
            arctic: asset_server.load("planets/bridge_arctic.png"),
            jungle: asset_server.load("planets/bridge_jungle.png"),
            city: asset_server.load("planets/bridge_city.png"),
            layout: layouts.add(layout),
        }
    }

    fn image(&self, planet: PlanetType) -> Handle<Image> {
        match planet {
            PlanetType::Desert => self.desert.clone(),
            PlanetType::Volcanic => self.volcanic.clone(),
            PlanetType::Arctic => self.arctic.clone(),
            PlanetType::Jungle => self.jungle.clone(),
            PlanetType::City => self.city.clone(),
        }
    }
}

pub fn radar_overlay_frame_names(kind: RadarOverlayKind) -> Vec<String> {
    let (stem, count) = match kind {
        RadarOverlayKind::FrontLight => ("front_light", 2),
        RadarOverlayKind::SideLight => ("side_light", 2),
        RadarOverlayKind::BoxSpinner => ("box_spinner", 12),
        RadarOverlayKind::Dish => ("dish", 8),
    };

    (0..count)
        .map(|frame| format!("building_radar_{stem}_{frame}"))
        .collect()
}

pub fn radar_overlay_offsets(kind: RadarOverlayKind) -> Vec<Vec2> {
    match kind {
        RadarOverlayKind::FrontLight => vec![Vec2::new(16.0, 22.0); 2],
        RadarOverlayKind::SideLight => vec![Vec2::new(41.0, 0.0); 2],
        RadarOverlayKind::BoxSpinner => vec![Vec2::new(18.0, 13.0); 12],
        RadarOverlayKind::Dish => [-5.0, -6.0, -10.0, -13.0, -15.0, -13.0, -10.0, -6.0]
            .into_iter()
            .map(|y| Vec2::new(15.0, y))
            .collect(),
    }
}

pub fn repair_overlay_frame_names(kind: RepairOverlayKind) -> Vec<String> {
    let (stem, count) = match kind {
        RepairOverlayKind::FrontLight => ("front_light", 2),
        RepairOverlayKind::SideLight => ("side_light", 2),
        RepairOverlayKind::Bulb => ("bulb", 2),
        RepairOverlayKind::SmokeStack => ("smoke_stack", 5),
        RepairOverlayKind::TextBox => ("text_box", 3),
    };

    (0..count)
        .map(|frame| format!("building_repair_{stem}_{frame}"))
        .collect()
}

pub fn repair_overlay_offsets(kind: RepairOverlayKind) -> Vec<Vec2> {
    match kind {
        RepairOverlayKind::FrontLight => vec![Vec2::new(6.0, 16.0); 2],
        RepairOverlayKind::SideLight => vec![Vec2::new(18.0, 6.0); 2],
        RepairOverlayKind::Bulb => vec![Vec2::new(32.0, 0.0); 2],
        RepairOverlayKind::SmokeStack => vec![Vec2::new(61.0, 0.0); 5],
        RepairOverlayKind::TextBox => vec![Vec2::new(16.0, 32.0); 3],
    }
}

pub fn factory_overlay_frame_names(kind: FactoryOverlayKind) -> Vec<String> {
    match kind {
        FactoryOverlayKind::RobotExhaust | FactoryOverlayKind::VehicleExhaust => {
            (0..13).map(|frame| format!("exhaust_{frame}")).collect()
        }
        FactoryOverlayKind::RobotGreenBox => (0..6)
            .map(|frame| format!("building_robot_green_box_{frame}"))
            .collect(),
        FactoryOverlayKind::RobotSingleLight0
        | FactoryOverlayKind::RobotSingleLight1
        | FactoryOverlayKind::RobotSingleLight2 => (0..2)
            .map(|frame| format!("building_robot_light_{frame}"))
            .collect(),
        FactoryOverlayKind::RobotDoubleLight => vec!["building_robot_double_light_1".to_string()],
        FactoryOverlayKind::RobotBody => (0..2)
            .map(|frame| format!("building_robot_robot_{frame}"))
            .collect(),
        FactoryOverlayKind::RobotSpin => (0..8)
            .map(|frame| format!("building_robot_spin_{frame}"))
            .collect(),
        FactoryOverlayKind::VehicleTank => (0..2)
            .map(|frame| format!("building_vehicle_tank_{frame}"))
            .collect(),
        FactoryOverlayKind::VehicleVent => (0..4)
            .map(|frame| format!("building_vehicle_vent_{frame}"))
            .collect(),
        FactoryOverlayKind::VehicleBulb => (0..2)
            .map(|frame| format!("building_vehicle_bulb_{frame}"))
            .collect(),
        FactoryOverlayKind::VehicleLight0 | FactoryOverlayKind::VehicleLight1 => {
            vec!["building_vehicle_lights_1".to_string()]
        }
        FactoryOverlayKind::VehicleSpin => (0..8)
            .map(|frame| format!("building_vehicle_spin_{frame}"))
            .collect(),
    }
}

pub fn factory_overlay_offsets(kind: FactoryOverlayKind) -> Vec<Vec2> {
    match kind {
        FactoryOverlayKind::RobotExhaust => (0..13)
            .map(|frame| Vec2::new(28.0, -24.0 - frame as f32 * 2.0))
            .collect(),
        FactoryOverlayKind::RobotGreenBox => vec![Vec2::new(38.0, 39.0); 6],
        FactoryOverlayKind::RobotSingleLight0 => vec![Vec2::new(13.0, 68.0); 2],
        FactoryOverlayKind::RobotSingleLight1 => vec![Vec2::new(16.0, 68.0); 2],
        FactoryOverlayKind::RobotSingleLight2 => vec![Vec2::new(19.0, 68.0); 2],
        FactoryOverlayKind::RobotDoubleLight => vec![Vec2::new(16.0, 32.0)],
        FactoryOverlayKind::RobotBody => vec![Vec2::new(16.0, 48.0); 2],
        FactoryOverlayKind::RobotSpin => vec![Vec2::new(9.0, -2.0); 8],
        FactoryOverlayKind::VehicleExhaust => (0..13)
            .map(|frame| Vec2::new(28.0, -22.0 - frame as f32 * 2.0))
            .collect(),
        FactoryOverlayKind::VehicleTank => vec![Vec2::new(16.0, 48.0); 2],
        FactoryOverlayKind::VehicleVent => vec![Vec2::new(16.0, 32.0); 4],
        FactoryOverlayKind::VehicleBulb => vec![Vec2::new(24.0, 39.0); 2],
        FactoryOverlayKind::VehicleLight0 => vec![Vec2::new(13.0, 47.0)],
        FactoryOverlayKind::VehicleLight1 => vec![Vec2::new(42.0, 47.0)],
        FactoryOverlayKind::VehicleSpin => vec![Vec2::new(9.0, -2.0); 8],
    }
}

fn bridge_frame(
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    index: usize,
    world_offset: Vec2,
    frame_size: Vec2,
) -> SpriteFrame {
    SpriteFrame {
        image,
        layout,
        index,
        animation_indices: None,
        world_offset,
        source_offset: Vec2::ZERO,
        frame_size,
    }
}

impl NamedAtlas {
    fn frame(&self, name: &str) -> Option<SpriteFrame> {
        self.frame_with_world_offset(name, Vec2::ZERO)
    }

    fn frame_with_world_offset(&self, name: &str, world_offset: Vec2) -> Option<SpriteFrame> {
        self.frame_with_world_offset_and_animation(name, world_offset, &[])
    }

    fn frame_with_world_offset_and_animation(
        &self,
        name: &str,
        world_offset: Vec2,
        animation_names: &[String],
    ) -> Option<SpriteFrame> {
        let frame = self.frames.get(name)?;
        Some(SpriteFrame {
            image: self.image.clone(),
            layout: self.layout.clone(),
            index: frame.index,
            animation_indices: self.animation_indices(animation_names),
            world_offset,
            source_offset: frame.source_offset,
            frame_size: frame.frame_size,
        })
    }

    fn animation_indices(&self, names: &[String]) -> Option<Vec<usize>> {
        if names.is_empty() {
            return None;
        }

        names
            .iter()
            .map(|name| self.frames.get(name).map(|frame| frame.index))
            .collect()
    }
}

fn push_building_base_and_overlay(
    layers: &mut Vec<SpriteFrame>,
    base_atlas: &NamedAtlas,
    team_atlas: Option<&NamedAtlas>,
    base_frame: &str,
    overlay_frame: &str,
    overlay_offset: Vec2,
) {
    if let Some(frame) = base_atlas.frame(base_frame) {
        layers.push(frame);
    }

    if let Some(frame) =
        team_atlas.and_then(|atlas| atlas.frame_with_world_offset(overlay_frame, overlay_offset))
    {
        layers.push(frame);
    }
}

fn load_named_atlas(
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
    image_path: &str,
    json: &str,
) -> NamedAtlas {
    let parsed: AtlasJson = serde_json::from_str(json).expect("valid texture atlas json");
    let texture = parsed
        .textures
        .first()
        .expect("texture atlas json should contain one texture");

    let mut layout = TextureAtlasLayout::new_empty(UVec2::new(texture.size.w, texture.size.h));
    let mut frames = HashMap::with_capacity(texture.frames.len());

    for frame in &texture.frames {
        let index = layout.add_texture(URect {
            min: UVec2::new(frame.frame.x, frame.frame.y),
            max: UVec2::new(frame.frame.x + frame.frame.w, frame.frame.y + frame.frame.h),
        });
        frames.insert(
            frame.filename.clone(),
            AtlasFrameInfo {
                index,
                source_offset: Vec2::new(
                    frame.sprite_source_size.x as f32,
                    frame.sprite_source_size.y as f32,
                ),
                frame_size: Vec2::new(frame.frame.w as f32, frame.frame.h as f32),
            },
        );
    }

    NamedAtlas {
        image: asset_server.load(image_path.to_string()),
        layout: layouts.add(layout),
        frames,
    }
}

fn atlas_json(kind: &str, team: TeamType) -> &'static str {
    match (kind, team) {
        ("robots", TeamType::Red) => include_str!("../../assets/atlases/robots_red.json"),
        ("robots", TeamType::Blue) => include_str!("../../assets/atlases/robots_blue.json"),
        ("robots", TeamType::Green) => include_str!("../../assets/atlases/robots_green.json"),
        ("robots", TeamType::Yellow) => include_str!("../../assets/atlases/robots_yellow.json"),
        ("vehicles", TeamType::Red) => include_str!("../../assets/atlases/vehicles_red.json"),
        ("vehicles", TeamType::Blue) => include_str!("../../assets/atlases/vehicles_blue.json"),
        ("vehicles", TeamType::Green) => include_str!("../../assets/atlases/vehicles_green.json"),
        ("vehicles", TeamType::Yellow) => include_str!("../../assets/atlases/vehicles_yellow.json"),
        ("cannons", TeamType::Red) => include_str!("../../assets/atlases/cannons_red.json"),
        ("cannons", TeamType::Blue) => include_str!("../../assets/atlases/cannons_blue.json"),
        ("cannons", TeamType::Green) => include_str!("../../assets/atlases/cannons_green.json"),
        ("cannons", TeamType::Yellow) => include_str!("../../assets/atlases/cannons_yellow.json"),
        ("buildings", TeamType::Red) => include_str!("../../assets/atlases/buildings_red.json"),
        ("buildings", TeamType::Blue) => include_str!("../../assets/atlases/buildings_blue.json"),
        ("buildings", TeamType::Green) => include_str!("../../assets/atlases/buildings_green.json"),
        ("buildings", TeamType::Yellow) => {
            include_str!("../../assets/atlases/buildings_yellow.json")
        }
        _ => unreachable!("unsupported atlas {kind}/{team:?}"),
    }
}

#[derive(Deserialize)]
struct AtlasJson {
    textures: Vec<AtlasTexture>,
}

#[derive(Deserialize)]
struct AtlasTexture {
    size: AtlasSize,
    frames: Vec<AtlasFrame>,
}

#[derive(Deserialize)]
struct AtlasSize {
    w: u32,
    h: u32,
}

#[derive(Deserialize)]
struct AtlasFrame {
    filename: String,
    frame: AtlasRect,
    #[serde(rename = "spriteSourceSize")]
    sprite_source_size: AtlasRect,
}

#[derive(Deserialize)]
struct AtlasRect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl PlanetType {
    fn asset_name(self) -> &'static str {
        match self {
            Self::Desert => "desert",
            Self::Volcanic => "volcanic",
            Self::Arctic => "arctic",
            Self::Jungle => "jungle",
            Self::City => "city",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_visual_state_matches_original_health_bands() {
        assert_eq!(bridge_visual_state(100), BridgeVisualState::Live);
        assert_eq!(bridge_visual_state(50), BridgeVisualState::Live);
        assert_eq!(bridge_visual_state(49), BridgeVisualState::Damaged);
        assert_eq!(bridge_visual_state(1), BridgeVisualState::Damaged);
        assert_eq!(bridge_visual_state(0), BridgeVisualState::Destroyed);
    }

    #[test]
    fn bridge_fill_indices_cover_live_damaged_and_destroyed_frames() {
        assert_eq!(
            bridge_vert_fill_index(BridgeVisualState::Live, 2),
            BRIDGE_VERT_FILL_LIVE
        );
        assert_eq!(
            bridge_vert_fill_index(BridgeVisualState::Damaged, 2),
            BRIDGE_VERT_FILL_DAMAGED_0
        );
        assert_eq!(
            bridge_vert_fill_index(BridgeVisualState::Damaged, 3),
            BRIDGE_VERT_FILL_DAMAGED_1
        );
        assert_eq!(
            bridge_vert_fill_index(BridgeVisualState::Destroyed, 2),
            BRIDGE_VERT_FILL_DESTROYED
        );
        assert_eq!(
            bridge_horz_fill_index(BridgeVisualState::Destroyed, 2),
            BRIDGE_HORZ_FILL_DESTROYED
        );
    }

    #[test]
    fn radar_overlay_frame_names_match_original_assets() {
        assert_eq!(
            radar_overlay_frame_names(RadarOverlayKind::FrontLight),
            vec![
                "building_radar_front_light_0".to_string(),
                "building_radar_front_light_1".to_string()
            ]
        );
        assert_eq!(
            radar_overlay_frame_names(RadarOverlayKind::BoxSpinner).len(),
            12
        );
        assert_eq!(
            radar_overlay_frame_names(RadarOverlayKind::Dish)
                .last()
                .map(String::as_str),
            Some("building_radar_dish_7")
        );
    }

    #[test]
    fn radar_overlay_offsets_match_original_do_after_effects() {
        assert_eq!(
            radar_overlay_offsets(RadarOverlayKind::FrontLight),
            vec![Vec2::new(16.0, 22.0); 2]
        );
        assert_eq!(
            radar_overlay_offsets(RadarOverlayKind::SideLight),
            vec![Vec2::new(41.0, 0.0); 2]
        );
        assert_eq!(
            radar_overlay_offsets(RadarOverlayKind::BoxSpinner),
            vec![Vec2::new(18.0, 13.0); 12]
        );
        assert_eq!(
            radar_overlay_offsets(RadarOverlayKind::Dish),
            vec![
                Vec2::new(15.0, -5.0),
                Vec2::new(15.0, -6.0),
                Vec2::new(15.0, -10.0),
                Vec2::new(15.0, -13.0),
                Vec2::new(15.0, -15.0),
                Vec2::new(15.0, -13.0),
                Vec2::new(15.0, -10.0),
                Vec2::new(15.0, -6.0),
            ]
        );
    }

    #[test]
    fn repair_overlay_frame_names_match_original_assets() {
        assert_eq!(
            repair_overlay_frame_names(RepairOverlayKind::SmokeStack).len(),
            5
        );
        assert_eq!(
            repair_overlay_frame_names(RepairOverlayKind::TextBox),
            vec![
                "building_repair_text_box_0".to_string(),
                "building_repair_text_box_1".to_string(),
                "building_repair_text_box_2".to_string()
            ]
        );
        assert_eq!(
            repair_overlay_frame_names(RepairOverlayKind::Bulb),
            vec![
                "building_repair_bulb_0".to_string(),
                "building_repair_bulb_1".to_string()
            ]
        );
    }

    #[test]
    fn repair_overlay_offsets_match_original_do_after_effects() {
        assert_eq!(
            repair_overlay_offsets(RepairOverlayKind::FrontLight),
            vec![Vec2::new(6.0, 16.0); 2]
        );
        assert_eq!(
            repair_overlay_offsets(RepairOverlayKind::SideLight),
            vec![Vec2::new(18.0, 6.0); 2]
        );
        assert_eq!(
            repair_overlay_offsets(RepairOverlayKind::Bulb),
            vec![Vec2::new(32.0, 0.0); 2]
        );
        assert_eq!(
            repair_overlay_offsets(RepairOverlayKind::SmokeStack),
            vec![Vec2::new(61.0, 0.0); 5]
        );
        assert_eq!(
            repair_overlay_offsets(RepairOverlayKind::TextBox),
            vec![Vec2::new(16.0, 32.0); 3]
        );
    }

    #[test]
    fn factory_overlay_frame_names_match_original_assets() {
        assert_eq!(
            factory_overlay_frame_names(FactoryOverlayKind::RobotSpin).len(),
            8
        );
        assert_eq!(
            factory_overlay_frame_names(FactoryOverlayKind::RobotGreenBox)
                .last()
                .map(String::as_str),
            Some("building_robot_green_box_5")
        );
        assert_eq!(
            factory_overlay_frame_names(FactoryOverlayKind::RobotSingleLight0),
            vec![
                "building_robot_light_0".to_string(),
                "building_robot_light_1".to_string()
            ]
        );
        assert_eq!(
            factory_overlay_frame_names(FactoryOverlayKind::VehicleVent).len(),
            4
        );
        assert_eq!(
            factory_overlay_frame_names(FactoryOverlayKind::VehicleLight1),
            vec!["building_vehicle_lights_1".to_string()]
        );
        assert_eq!(
            factory_overlay_frame_names(FactoryOverlayKind::VehicleExhaust)
                .last()
                .map(String::as_str),
            Some("exhaust_12")
        );
    }

    #[test]
    fn factory_overlay_offsets_match_original_do_after_effects() {
        assert_eq!(
            factory_overlay_offsets(FactoryOverlayKind::RobotExhaust)
                .last()
                .copied(),
            Some(Vec2::new(28.0, -48.0))
        );
        assert_eq!(
            factory_overlay_offsets(FactoryOverlayKind::RobotSingleLight2),
            vec![Vec2::new(19.0, 68.0); 2]
        );
        assert_eq!(
            factory_overlay_offsets(FactoryOverlayKind::RobotDoubleLight),
            vec![Vec2::new(16.0, 32.0)]
        );
        assert_eq!(
            factory_overlay_offsets(FactoryOverlayKind::VehicleExhaust)
                .last()
                .copied(),
            Some(Vec2::new(28.0, -46.0))
        );
        assert_eq!(
            factory_overlay_offsets(FactoryOverlayKind::VehicleLight0),
            vec![Vec2::new(13.0, 47.0)]
        );
        assert_eq!(
            factory_overlay_offsets(FactoryOverlayKind::VehicleLight1),
            vec![Vec2::new(42.0, 47.0)]
        );
    }
}
