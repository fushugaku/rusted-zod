use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use crate::original::map::MapObject;
use crate::original::objects::{BuildingType, CannonType, ObjectKind, RobotType, VehicleType};
use crate::original::types::{PlanetType, TeamType};
use crate::units::{buildings as unit_buildings, cannons, items, robots, vehicles};

const DEFAULT_ROTATION: u16 = 180;

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

pub(crate) use crate::units::buildings::BridgeVisualState;

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
        match kind {
            ObjectKind::Robot(_) => {
                let spec = robots::stand_atlas_frame_spec(team, DEFAULT_ROTATION);
                self.robots
                    .get(&spec.atlas_team)
                    .and_then(|atlas| atlas.frame(&spec.frame_name))
                    .into_iter()
                    .collect()
            }
            ObjectKind::Vehicle(vehicle_type) => {
                self.vehicle_spawn_frames(vehicle_type, team, DEFAULT_ROTATION, 0)
            }
            ObjectKind::Cannon(cannon_type) => {
                let profile = cannons::spawn_frame_profile(cannon_type, team, DEFAULT_ROTATION);
                self.cannons
                    .get(&profile.atlas_team)
                    .and_then(|atlas| atlas.frame(&profile.atlas_frame_name))
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
                _ => self.building_layers(building_type, team, planet),
            },
            ObjectKind::Bridge(building_type) => self.bridge_layers(
                building_type,
                planet,
                object.extra_links,
                bridge_visual_state(object.health_percent),
            ),
            ObjectKind::MapItem(item_id) => {
                let Some(spec) = items::map_item_atlas_frame_spec(item_id, object.owner, planet)
                else {
                    return Vec::new();
                };

                if spec.animation_frame_names.is_empty() {
                    self.map_items.frame(&spec.frame_name).into_iter().collect()
                } else {
                    self.map_items
                        .frame_with_world_offset_and_animation(
                            &spec.frame_name,
                            Vec2::ZERO,
                            &spec.animation_frame_names,
                        )
                        .into_iter()
                        .collect()
                }
            }
            ObjectKind::Rock | ObjectKind::Animal(_) => Vec::new(),
        }
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

        match (kind, role) {
            (ObjectKind::Robot(_), MobileSpriteRole::Robot) => {
                let spec = robots::mobile_atlas_frame_spec(team, rotation, frame, moving);
                self.robots
                    .get(&spec.atlas_team)
                    .and_then(|atlas| atlas.frame(&spec.frame_name))
            }
            (ObjectKind::Vehicle(vehicle_type), role) => {
                self.vehicle_mobile_frame(vehicle_type, team, role, rotation, frame, moving)
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

    pub fn robot_throw_frame(
        &self,
        team: TeamType,
        rotation: u16,
        frame: usize,
    ) -> Option<SpriteFrame> {
        let spec = robots::throw_atlas_frame_spec(team, rotation, frame);
        self.robots
            .get(&spec.atlas_team)
            .and_then(|atlas| atlas.frame(&spec.frame_name))
    }

    pub fn robot_grenade_pickup_frame(
        &self,
        team: TeamType,
        upward: bool,
        frame: usize,
    ) -> Option<SpriteFrame> {
        let spec = robots::grenade_pickup_atlas_frame_spec(team, upward, frame);
        self.robots
            .get(&spec.atlas_team)
            .and_then(|atlas| atlas.frame(&spec.frame_name))
    }

    pub fn robot_idle_action_frame(
        &self,
        team: TeamType,
        kind: robots::RobotIdleActionKind,
        frame: usize,
    ) -> Option<SpriteFrame> {
        let spec = robots::idle_action_atlas_frame_spec(team, kind, frame);
        self.robots
            .get(&spec.atlas_team)
            .and_then(|atlas| atlas.frame(&spec.frame_name))
    }

    pub fn robot_fire_frame(
        &self,
        robot: RobotType,
        team: TeamType,
        rotation: u16,
        frame: usize,
    ) -> Option<SpriteFrame> {
        let spec = robots::fire_atlas_frame_spec(robot, team, rotation, frame);
        self.robots
            .get(&spec.atlas_team)
            .and_then(|atlas| atlas.frame(&spec.frame_name))
    }

    pub fn captured_cannon_frame(
        &self,
        cannon_type: CannonType,
        team: TeamType,
        rotation: u16,
    ) -> Option<SpriteFrame> {
        let profile = cannons::captured_frame_profile(cannon_type, team, rotation);

        self.cannons
            .get(&profile.atlas_team)
            .and_then(|atlas| atlas.frame(&profile.atlas_frame_name))
    }

    pub fn cannon_placement_frame(
        &self,
        cannon_type: CannonType,
        team: TeamType,
        frame: usize,
    ) -> Option<SpriteFrame> {
        let profile = cannons::placement_frame_profile(cannon_type, team, frame)?;
        self.cannons
            .get(&profile.atlas_team)
            .and_then(|atlas| atlas.frame(&profile.atlas_frame_name))
    }

    pub fn flag_animation_indices(&self, team: TeamType) -> Option<Vec<usize>> {
        let frames = items::flag_animation_frame_names(team);
        self.map_items.animation_indices(&frames)
    }

    pub fn radar_overlay_frames(&self, kind: RadarOverlayKind) -> Option<Vec<SpriteFrame>> {
        let atlas = self.buildings.get(&TeamType::Red)?;
        unit_buildings::radar_overlay_frame_specs(kind)
            .into_iter()
            .map(|spec| atlas.frame_with_world_offset(&spec.frame_name, spec.world_offset))
            .collect()
    }

    pub fn repair_overlay_frames(&self, kind: RepairOverlayKind) -> Option<Vec<SpriteFrame>> {
        let atlas = self.buildings.get(&TeamType::Red)?;
        unit_buildings::repair_overlay_frame_specs(kind)
            .into_iter()
            .map(|spec| atlas.frame_with_world_offset(&spec.frame_name, spec.world_offset))
            .collect()
    }

    pub fn factory_overlay_frames(&self, kind: FactoryOverlayKind) -> Option<Vec<SpriteFrame>> {
        let atlas = self.buildings.get(&TeamType::Red)?;
        unit_buildings::factory_overlay_frame_specs(kind)
            .into_iter()
            .map(|spec| atlas.frame_with_world_offset(&spec.frame_name, spec.world_offset))
            .collect()
    }

    fn vehicle_spawn_frames(
        &self,
        vehicle_type: VehicleType,
        team: TeamType,
        rotation: u16,
        frame: usize,
    ) -> Vec<SpriteFrame> {
        vehicles::spawn_atlas_frame_specs(vehicle_type, team, rotation, frame)
            .into_iter()
            .filter_map(|spec| {
                self.vehicles
                    .get(&spec.atlas_team)
                    .and_then(|atlas| atlas.frame(&spec.frame_name))
            })
            .collect()
    }

    fn vehicle_mobile_frame(
        &self,
        vehicle_type: VehicleType,
        team: TeamType,
        role: MobileSpriteRole,
        rotation: u16,
        frame: usize,
        moving: bool,
    ) -> Option<SpriteFrame> {
        let spec =
            vehicles::mobile_atlas_frame_spec(vehicle_type, team, role, rotation, frame, moving)?;
        self.vehicles
            .get(&spec.atlas_team)
            .and_then(|atlas| atlas.frame(&spec.frame_name))
    }

    fn building_layers(
        &self,
        building_type: BuildingType,
        team: TeamType,
        planet: PlanetType,
    ) -> Vec<SpriteFrame> {
        unit_buildings::building_layer_specs(building_type, team, planet)
            .into_iter()
            .filter_map(|spec| {
                self.buildings.get(&spec.atlas_team).and_then(|atlas| {
                    atlas.frame_with_world_offset_and_animation(
                        &spec.frame_name,
                        spec.world_offset,
                        &spec.animation_frame_names,
                    )
                })
            })
            .collect()
    }

    pub fn bridge_layers(
        &self,
        building_type: BuildingType,
        planet: PlanetType,
        extra_links: u16,
        state: BridgeVisualState,
    ) -> Vec<SpriteFrame> {
        let image = self.bridges.image(planet);
        unit_buildings::bridge_visual_tile_specs(building_type, extra_links, state)
            .into_iter()
            .map(|spec| {
                bridge_frame(
                    image.clone(),
                    self.bridges.layout.clone(),
                    spec.index,
                    spec.world_offset,
                    spec.frame_size,
                )
            })
            .collect()
    }
}

fn bridge_visual_state(health_percent: i32) -> BridgeVisualState {
    unit_buildings::bridge_visual_state(health_percent)
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
            unit_buildings::bridge_vert_fill_index(BridgeVisualState::Live, 2),
            4
        );
        assert_eq!(
            unit_buildings::bridge_vert_fill_index(BridgeVisualState::Damaged, 2),
            5
        );
        assert_eq!(
            unit_buildings::bridge_vert_fill_index(BridgeVisualState::Damaged, 3),
            6
        );
        assert_eq!(
            unit_buildings::bridge_vert_fill_index(BridgeVisualState::Destroyed, 2),
            7
        );
        assert_eq!(
            unit_buildings::bridge_horz_fill_index(BridgeVisualState::Destroyed, 2),
            15
        );
    }

    #[test]
    fn radar_overlay_frame_names_match_original_assets() {
        assert_eq!(
            unit_buildings::radar_overlay_frame_names(RadarOverlayKind::FrontLight),
            vec![
                "building_radar_front_light_0".to_string(),
                "building_radar_front_light_1".to_string()
            ]
        );
        assert_eq!(
            unit_buildings::radar_overlay_frame_names(RadarOverlayKind::BoxSpinner).len(),
            12
        );
        assert_eq!(
            unit_buildings::radar_overlay_frame_names(RadarOverlayKind::Dish)
                .last()
                .map(String::as_str),
            Some("building_radar_dish_7")
        );
    }

    #[test]
    fn radar_overlay_offsets_match_original_do_after_effects() {
        assert_eq!(
            unit_buildings::radar_overlay_offsets(RadarOverlayKind::FrontLight),
            vec![Vec2::new(16.0, 22.0); 2]
        );
        assert_eq!(
            unit_buildings::radar_overlay_offsets(RadarOverlayKind::SideLight),
            vec![Vec2::new(41.0, 0.0); 2]
        );
        assert_eq!(
            unit_buildings::radar_overlay_offsets(RadarOverlayKind::BoxSpinner),
            vec![Vec2::new(18.0, 13.0); 12]
        );
        assert_eq!(
            unit_buildings::radar_overlay_offsets(RadarOverlayKind::Dish),
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
            unit_buildings::repair_overlay_frame_names(RepairOverlayKind::SmokeStack).len(),
            5
        );
        assert_eq!(
            unit_buildings::repair_overlay_frame_names(RepairOverlayKind::TextBox),
            vec![
                "building_repair_text_box_0".to_string(),
                "building_repair_text_box_1".to_string(),
                "building_repair_text_box_2".to_string()
            ]
        );
        assert_eq!(
            unit_buildings::repair_overlay_frame_names(RepairOverlayKind::Bulb),
            vec![
                "building_repair_bulb_0".to_string(),
                "building_repair_bulb_1".to_string()
            ]
        );
    }

    #[test]
    fn repair_overlay_offsets_match_original_do_after_effects() {
        assert_eq!(
            unit_buildings::repair_overlay_offsets(RepairOverlayKind::FrontLight),
            vec![Vec2::new(6.0, 16.0); 2]
        );
        assert_eq!(
            unit_buildings::repair_overlay_offsets(RepairOverlayKind::SideLight),
            vec![Vec2::new(18.0, 6.0); 2]
        );
        assert_eq!(
            unit_buildings::repair_overlay_offsets(RepairOverlayKind::Bulb),
            vec![Vec2::new(32.0, 0.0); 2]
        );
        assert_eq!(
            unit_buildings::repair_overlay_offsets(RepairOverlayKind::SmokeStack),
            vec![Vec2::new(61.0, 0.0); 5]
        );
        assert_eq!(
            unit_buildings::repair_overlay_offsets(RepairOverlayKind::TextBox),
            vec![Vec2::new(16.0, 32.0); 3]
        );
    }

    #[test]
    fn factory_overlay_frame_names_match_original_assets() {
        assert_eq!(
            unit_buildings::factory_overlay_frame_names(FactoryOverlayKind::RobotSpin).len(),
            8
        );
        assert_eq!(
            unit_buildings::factory_overlay_frame_names(FactoryOverlayKind::RobotGreenBox)
                .last()
                .map(String::as_str),
            Some("building_robot_green_box_5")
        );
        assert_eq!(
            unit_buildings::factory_overlay_frame_names(FactoryOverlayKind::RobotSingleLight0),
            vec![
                "building_robot_light_0".to_string(),
                "building_robot_light_1".to_string()
            ]
        );
        assert_eq!(
            unit_buildings::factory_overlay_frame_names(FactoryOverlayKind::VehicleVent).len(),
            4
        );
        assert_eq!(
            unit_buildings::factory_overlay_frame_names(FactoryOverlayKind::VehicleLight1),
            vec!["building_vehicle_lights_1".to_string()]
        );
        assert_eq!(
            unit_buildings::factory_overlay_frame_names(FactoryOverlayKind::VehicleExhaust)
                .last()
                .map(String::as_str),
            Some("exhaust_12")
        );
    }

    #[test]
    fn factory_overlay_offsets_match_original_do_after_effects() {
        assert_eq!(
            unit_buildings::factory_overlay_offsets(FactoryOverlayKind::RobotExhaust)
                .last()
                .copied(),
            Some(Vec2::new(28.0, -48.0))
        );
        assert_eq!(
            unit_buildings::factory_overlay_offsets(FactoryOverlayKind::RobotSingleLight2),
            vec![Vec2::new(19.0, 68.0); 2]
        );
        assert_eq!(
            unit_buildings::factory_overlay_offsets(FactoryOverlayKind::RobotDoubleLight),
            vec![Vec2::new(16.0, 32.0)]
        );
        assert_eq!(
            unit_buildings::factory_overlay_offsets(FactoryOverlayKind::VehicleExhaust)
                .last()
                .copied(),
            Some(Vec2::new(28.0, -46.0))
        );
        assert_eq!(
            unit_buildings::factory_overlay_offsets(FactoryOverlayKind::VehicleLight0),
            vec![Vec2::new(13.0, 47.0)]
        );
        assert_eq!(
            unit_buildings::factory_overlay_offsets(FactoryOverlayKind::VehicleLight1),
            vec![Vec2::new(42.0, 47.0)]
        );
    }
}
