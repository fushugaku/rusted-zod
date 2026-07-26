use std::collections::HashMap;

use bevy::prelude::Vec2;

use crate::{
    components::{
        BridgeFootprint, CombatRng, MapGridPosition, ObjectStats, ProductionLevel,
        ProductionWindowKind,
    },
    constants::TILE_SIZE,
    original::{
        map::MapObjectType,
        objects::{BuildingType, ObjectKind},
        types::{PlanetType, TeamType},
    },
    render::atlas::{FactoryOverlayKind, RadarOverlayKind, RepairOverlayKind},
    units::buildings::{
        BuildingDeathProfile, BuildingEffectBox, BuildingPathingSpec, EffectTrajectory,
        bridge_horz_ui, bridge_vert_ui, fort_back_ui, fort_front_ui, radar_ui, repair_ui,
        robot_factory_ui, vehicle_factory_ui,
    },
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ProductionPlacement {
    pub(crate) create_offset: Vec2,
    pub(crate) move_offset: Vec2,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BuildingAtlasFrameSpec {
    pub(crate) atlas_team: TeamType,
    pub(crate) frame_name: String,
    pub(crate) world_offset: Vec2,
    pub(crate) animation_frame_names: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BuildingOverlayFrameSpec {
    pub(crate) frame_name: String,
    pub(crate) world_offset: Vec2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BridgeVisualState {
    Live,
    Damaged,
    Destroyed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BridgeVisualTileSpec {
    pub(crate) index: usize,
    pub(crate) world_offset: Vec2,
    pub(crate) frame_size: Vec2,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ProductionSelectorGeometrySpec {
    pub(crate) object_offset: Vec2,
    pub(crate) object_step: Vec2,
}

pub(crate) const BUILDING_TURRENT_FRAME_TIME: f32 = 0.1;
pub(crate) const BRIDGE_TURRENT_FRAME_TIME: f32 = 0.07;
pub(crate) const BRIDGE_TURRENT_MAX_DISTANCE: f32 = 140.0;
pub(crate) const BRIDGE_ROCK_PARTICLE_FRAME_TIME: f32 = 0.07;
pub(crate) const BRIDGE_REVIVE_RERENDER_DELAY: f32 = 2.25;
const PRODUCTION_SELECTOR_OBJECT_OFFSET: Vec2 = Vec2::new(6.0, 22.0);
const PRODUCTION_SELECTOR_OBJECT_STEP: Vec2 = Vec2::new(47.0, 53.0);

pub(crate) fn default_selection_size(building: BuildingType) -> Vec2 {
    match building {
        BuildingType::FortFront => fort_front_ui::default_selection_size(),
        BuildingType::FortBack => fort_back_ui::default_selection_size(),
        BuildingType::Radar => radar_ui::default_selection_size(),
        BuildingType::Repair => repair_ui::default_selection_size(),
        BuildingType::RobotFactory => robot_factory_ui::default_selection_size(),
        BuildingType::VehicleFactory => vehicle_factory_ui::default_selection_size(),
        BuildingType::BridgeVert => bridge_vert_ui::default_selection_size(),
        BuildingType::BridgeHorz => bridge_horz_ui::default_selection_size(),
    }
}

pub(crate) fn production_placement(building: BuildingType) -> Option<ProductionPlacement> {
    match building {
        BuildingType::FortFront => Some(fort_front_ui::production_placement()),
        BuildingType::FortBack => Some(fort_back_ui::production_placement()),
        BuildingType::RobotFactory => Some(robot_factory_ui::production_placement()),
        BuildingType::VehicleFactory => Some(vehicle_factory_ui::production_placement()),
        BuildingType::Radar
        | BuildingType::Repair
        | BuildingType::BridgeVert
        | BuildingType::BridgeHorz => None,
    }
}

pub(crate) fn production_world_points(
    building: BuildingType,
    tile_x: u16,
    tile_y: u16,
) -> Option<(Vec2, Vec2)> {
    let placement = production_placement(building)?;
    let origin = Vec2::new(tile_x as f32 * TILE_SIZE, tile_y as f32 * TILE_SIZE);
    let create = origin + placement.create_offset;
    let move_to = origin + placement.move_offset;
    Some((
        Vec2::new(create.x, -create.y),
        Vec2::new(move_to.x, -move_to.y),
    ))
}

#[cfg(test)]
pub(crate) fn production_selector_geometry() -> ProductionSelectorGeometrySpec {
    ProductionSelectorGeometrySpec {
        object_offset: PRODUCTION_SELECTOR_OBJECT_OFFSET,
        object_step: PRODUCTION_SELECTOR_OBJECT_STEP,
    }
}

pub(crate) fn production_label_asset_path(kind: ProductionWindowKind) -> &'static str {
    match kind {
        ProductionWindowKind::Robot => robot_factory_ui::production_label_asset_path(),
        ProductionWindowKind::Vehicle => vehicle_factory_ui::production_label_asset_path(),
        ProductionWindowKind::Fort => fort_front_ui::production_label_asset_path(),
    }
}

#[cfg(test)]
pub(crate) fn production_label_asset_path_for_building(
    building: BuildingType,
) -> Option<&'static str> {
    match building {
        BuildingType::FortFront => Some(fort_front_ui::production_label_asset_path()),
        BuildingType::FortBack => Some(fort_back_ui::production_label_asset_path()),
        BuildingType::RobotFactory => Some(robot_factory_ui::production_label_asset_path()),
        BuildingType::VehicleFactory => Some(vehicle_factory_ui::production_label_asset_path()),
        BuildingType::Radar
        | BuildingType::Repair
        | BuildingType::BridgeVert
        | BuildingType::BridgeHorz => None,
    }
}

pub(crate) fn fort_entrance_rect(building: BuildingType) -> Option<(f32, f32, f32, f32)> {
    match building {
        BuildingType::FortFront => Some(fort_front_ui::entrance_rect()),
        BuildingType::FortBack => Some(fort_back_ui::entrance_rect()),
        _ => None,
    }
}

pub(crate) fn point_in_fort_entrance_rect(
    point: Vec2,
    center: Vec2,
    size: Vec2,
    building: BuildingType,
) -> bool {
    let Some((x, y, width, height)) = fort_entrance_rect(building) else {
        return false;
    };
    let top_left = Vec2::new(center.x - size.x * 0.5, center.y + size.y * 0.5);
    let min_x = top_left.x + x;
    let max_x = min_x + width;
    let max_y = top_left.y - y;
    let min_y = max_y - height;

    point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
}

pub(crate) fn fallback_collision_size(building: BuildingType) -> Vec2 {
    match building {
        BuildingType::FortFront | BuildingType::FortBack => {
            Vec2::new(TILE_SIZE * 10.0, TILE_SIZE * 5.0)
        }
        _ => Vec2::splat(TILE_SIZE * 3.0),
    }
}

pub(crate) fn map_object_has_fort_turret_tile(
    object_type: MapObjectType,
    object_id: u8,
    object_tile_x: i32,
    object_tile_y: i32,
    tile_x: i32,
    tile_y: i32,
) -> bool {
    object_type == MapObjectType::Building
        && ObjectKind::from_map_parts(object_type, object_id).is_ok_and(|kind| {
            fort_turret_tile(kind, tile_x - object_tile_x, tile_y - object_tile_y)
        })
}

pub(crate) fn fort_turret_slot_allows(
    kind: ObjectKind,
    center: Vec2,
    size: Vec2,
    map_left: f32,
    map_top: f32,
) -> bool {
    let obstacle_left = center.x - size.x * 0.5;
    let obstacle_top = -center.y - size.y * 0.5;
    let local_x = ((map_left - obstacle_left) / TILE_SIZE).round() as i32;
    let local_y = ((map_top - obstacle_top) / TILE_SIZE).round() as i32;

    fort_turret_tile(kind, local_x, local_y)
}

fn fort_turret_tile(kind: ObjectKind, local_tile_x: i32, local_tile_y: i32) -> bool {
    matches!(
        kind,
        ObjectKind::Building(BuildingType::FortFront | BuildingType::FortBack)
    ) && matches!(local_tile_x, 1 | 7)
        && matches!(local_tile_y, 0 | 3)
}

pub(crate) fn fort_entry_points(
    center: Vec2,
    size: Vec2,
    building: BuildingType,
) -> Option<(Vec2, Vec2)> {
    if fort_entrance_rect(building).is_none() {
        return None;
    }
    let top_left = Vec2::new(center.x - size.x * 0.5, center.y + size.y * 0.5);
    let tile_x = (top_left.x / TILE_SIZE).round().max(0.0) as u16;
    let tile_y = (-top_left.y / TILE_SIZE).round().max(0.0) as u16;
    production_world_points(building, tile_x, tile_y)
}

pub(crate) fn building_layer_specs(
    building: BuildingType,
    team: TeamType,
    planet: PlanetType,
) -> Vec<BuildingAtlasFrameSpec> {
    match building {
        BuildingType::FortFront => fort_front_ui::atlas_layer_specs(team, planet),
        BuildingType::FortBack => fort_back_ui::atlas_layer_specs(team, planet),
        BuildingType::Radar => radar_ui::atlas_layer_specs(team, planet),
        BuildingType::Repair => repair_ui::atlas_layer_specs(team, planet),
        BuildingType::RobotFactory => robot_factory_ui::atlas_layer_specs(team, planet),
        BuildingType::VehicleFactory => vehicle_factory_ui::atlas_layer_specs(team, planet),
        BuildingType::BridgeVert | BuildingType::BridgeHorz => Vec::new(),
    }
}

pub(crate) fn bridge_visual_state(health_percent: i32) -> BridgeVisualState {
    if health_percent <= 0 {
        BridgeVisualState::Destroyed
    } else if health_percent < 50 {
        BridgeVisualState::Damaged
    } else {
        BridgeVisualState::Live
    }
}

pub(crate) fn bridge_visual_tile_specs(
    building: BuildingType,
    extra_links: u16,
    state: BridgeVisualState,
) -> Vec<BridgeVisualTileSpec> {
    match building {
        BuildingType::BridgeVert => bridge_vert_ui::visual_tile_specs(extra_links, state),
        BuildingType::BridgeHorz => bridge_horz_ui::visual_tile_specs(extra_links, state),
        _ => Vec::new(),
    }
}

pub(crate) fn bridge_pathing_dimensions(
    building: BuildingType,
    extra_links: u16,
) -> Option<(u16, u16)> {
    match building {
        BuildingType::BridgeVert => Some(bridge_vert_ui::pathing_dimensions(extra_links)),
        BuildingType::BridgeHorz => Some(bridge_horz_ui::pathing_dimensions(extra_links)),
        _ => None,
    }
}

pub(crate) fn bridge_edge_block_offsets(
    building: BuildingType,
    extra_links: u16,
) -> Vec<(u16, u16)> {
    match building {
        BuildingType::BridgeVert => bridge_vert_ui::edge_block_offsets(extra_links),
        BuildingType::BridgeHorz => bridge_horz_ui::edge_block_offsets(extra_links),
        _ => Vec::new(),
    }
}

pub(crate) fn bridge_center_offsets(building: BuildingType, extra_links: u16) -> Vec<(u16, u16)> {
    match building {
        BuildingType::BridgeVert => bridge_vert_ui::center_offsets(extra_links),
        BuildingType::BridgeHorz => bridge_horz_ui::center_offsets(extra_links),
        _ => Vec::new(),
    }
}

pub(crate) fn building_pathing_spec(building: BuildingType) -> BuildingPathingSpec {
    match building {
        BuildingType::FortFront => fort_front_ui::pathing_spec(),
        BuildingType::FortBack => fort_back_ui::pathing_spec(),
        BuildingType::Radar => radar_ui::pathing_spec(),
        BuildingType::Repair => repair_ui::pathing_spec(),
        BuildingType::RobotFactory => robot_factory_ui::pathing_spec(),
        BuildingType::VehicleFactory => vehicle_factory_ui::pathing_spec(),
        BuildingType::BridgeVert | BuildingType::BridgeHorz => BuildingPathingSpec {
            blocked_rects: Vec::new(),
            blocked_masks: Vec::new(),
            unblocked_tiles: Vec::new(),
        },
    }
}

#[cfg(test)]
pub(crate) fn bridge_vert_fill_index(state: BridgeVisualState, row: usize) -> usize {
    bridge_vert_ui::fill_index(state, row)
}

#[cfg(test)]
pub(crate) fn bridge_horz_fill_index(state: BridgeVisualState, col: usize) -> usize {
    bridge_horz_ui::fill_index(state, col)
}

#[cfg(test)]
pub(crate) fn radar_overlay_frame_names(kind: RadarOverlayKind) -> Vec<String> {
    radar_ui::overlay_frame_names(kind)
}

#[cfg(test)]
pub(crate) fn radar_overlay_offsets(kind: RadarOverlayKind) -> Vec<Vec2> {
    radar_ui::overlay_offsets(kind)
}

pub(crate) fn radar_overlay_frame_specs(kind: RadarOverlayKind) -> Vec<BuildingOverlayFrameSpec> {
    radar_ui::overlay_frame_specs(kind)
}

pub(crate) fn radar_overlay_kinds() -> &'static [RadarOverlayKind] {
    &radar_ui::OVERLAY_KINDS
}

pub(crate) fn radar_overlay_kinds_for_object(
    kind: ObjectKind,
) -> Option<&'static [RadarOverlayKind]> {
    matches!(kind, ObjectKind::Building(BuildingType::Radar)).then_some(radar_overlay_kinds())
}

pub(crate) fn radar_overlay_frame_time() -> f32 {
    radar_ui::OVERLAY_FRAME_TIME
}

#[cfg(test)]
pub(crate) fn repair_overlay_frame_names(kind: RepairOverlayKind) -> Vec<String> {
    repair_ui::overlay_frame_names(kind)
}

#[cfg(test)]
pub(crate) fn repair_overlay_offsets(kind: RepairOverlayKind) -> Vec<Vec2> {
    repair_ui::overlay_offsets(kind)
}

pub(crate) fn repair_overlay_frame_specs(kind: RepairOverlayKind) -> Vec<BuildingOverlayFrameSpec> {
    repair_ui::overlay_frame_specs(kind)
}

pub(crate) fn repair_overlay_kinds() -> &'static [RepairOverlayKind] {
    &repair_ui::OVERLAY_KINDS
}

pub(crate) fn repair_overlay_kinds_for_object(
    kind: ObjectKind,
) -> Option<&'static [RepairOverlayKind]> {
    matches!(kind, ObjectKind::Building(BuildingType::Repair)).then_some(repair_overlay_kinds())
}

pub(crate) fn repair_overlay_frame_time() -> f32 {
    repair_ui::OVERLAY_FRAME_TIME
}

pub(crate) fn repair_overlay_initial_frame(
    kind: RepairOverlayKind,
    owner: TeamType,
    repairing_unit: bool,
) -> usize {
    repair_ui::overlay_initial_frame(kind, owner, repairing_unit)
}

#[cfg(test)]
pub(crate) fn factory_overlay_frame_names(kind: FactoryOverlayKind) -> Vec<String> {
    if robot_factory_ui::overlay_kind_is_robot(kind) {
        robot_factory_ui::overlay_frame_names(kind)
    } else {
        vehicle_factory_ui::overlay_frame_names(kind)
    }
}

#[cfg(test)]
pub(crate) fn factory_overlay_offsets(kind: FactoryOverlayKind) -> Vec<Vec2> {
    if robot_factory_ui::overlay_kind_is_robot(kind) {
        robot_factory_ui::overlay_offsets(kind)
    } else {
        vehicle_factory_ui::overlay_offsets(kind)
    }
}

pub(crate) fn factory_overlay_frame_specs(
    kind: FactoryOverlayKind,
) -> Vec<BuildingOverlayFrameSpec> {
    if robot_factory_ui::overlay_kind_is_robot(kind) {
        robot_factory_ui::overlay_frame_specs(kind)
    } else {
        vehicle_factory_ui::overlay_frame_specs(kind)
    }
}

pub(crate) fn factory_overlay_kinds(
    building: BuildingType,
) -> Option<&'static [FactoryOverlayKind]> {
    match building {
        BuildingType::RobotFactory => Some(&robot_factory_ui::OVERLAY_KINDS),
        BuildingType::VehicleFactory => Some(&vehicle_factory_ui::OVERLAY_KINDS),
        _ => None,
    }
}

pub(crate) fn factory_overlay_kinds_for_object(
    kind: ObjectKind,
) -> Option<&'static [FactoryOverlayKind]> {
    let ObjectKind::Building(building) = kind else {
        return None;
    };

    factory_overlay_kinds(building)
}

pub(crate) fn factory_overlay_frame_time() -> f32 {
    robot_factory_ui::OVERLAY_FRAME_TIME
}

pub(crate) fn factory_overlay_process_tick(
    elapsed: f32,
    delta_secs: f32,
    frame_time: f32,
) -> (bool, f32) {
    let elapsed = elapsed + delta_secs.max(0.0);
    if frame_time <= 0.0 || elapsed >= frame_time {
        (true, 0.0)
    } else {
        (false, elapsed)
    }
}

pub(crate) fn factory_overlay_initial_frame_visible(kind: FactoryOverlayKind) -> bool {
    if robot_factory_ui::overlay_kind_is_robot(kind) {
        robot_factory_ui::overlay_initial_frame_visible(kind)
    } else {
        vehicle_factory_ui::overlay_initial_frame_visible(kind)
    }
}

pub(crate) fn production_window_kind(kind: ObjectKind) -> Option<ProductionWindowKind> {
    let ObjectKind::Building(building) = kind else {
        return None;
    };

    match building {
        BuildingType::RobotFactory => Some(ProductionWindowKind::Robot),
        BuildingType::VehicleFactory => Some(ProductionWindowKind::Vehicle),
        BuildingType::FortFront | BuildingType::FortBack => Some(ProductionWindowKind::Fort),
        BuildingType::Radar
        | BuildingType::Repair
        | BuildingType::BridgeVert
        | BuildingType::BridgeHorz => None,
    }
}

pub(crate) fn selected_production_unit(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    selected_index: usize,
) -> Option<ObjectKind> {
    let building = production_building(kind)?;
    super::default_build_list(building, level)
        .get(selected_index)
        .copied()
        .or_else(|| super::default_production_unit(building, level))
}

pub(crate) fn cycle_production_selection(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    selected_index: usize,
    direction: i32,
) -> usize {
    let Some(building) = production_building(kind) else {
        return selected_index;
    };
    let len = super::default_build_list(building, level).len();
    if len == 0 {
        return 0;
    }

    if direction >= 0 {
        (selected_index + 1) % len
    } else if selected_index == 0 {
        len - 1
    } else {
        selected_index - 1
    }
}

pub(crate) fn production_selector_unit_index(
    kind: ObjectKind,
    level: impl Into<ProductionLevel>,
    unit: ObjectKind,
) -> Option<usize> {
    let building = production_building(kind)?;
    super::default_build_list(building, level)
        .into_iter()
        .position(|candidate| candidate == unit)
}

pub(crate) fn production_selector_units(
    kind: ObjectKind,
    level: impl Into<ProductionLevel>,
) -> Vec<(ObjectKind, Vec2)> {
    let Some(building) = production_building(kind) else {
        return Vec::new();
    };
    let list = super::default_build_list(building, level);
    let mut rows: [Vec<ObjectKind>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for unit in list {
        match unit {
            ObjectKind::Robot(_) => rows[0].push(unit),
            ObjectKind::Vehicle(_) => rows[1].push(unit),
            ObjectKind::Cannon(_) => rows[2].push(unit),
            ObjectKind::Building(_)
            | ObjectKind::Bridge(_)
            | ObjectKind::Animal(_)
            | ObjectKind::MapItem(_)
            | ObjectKind::Rock => {}
        }
    }

    let mut units = Vec::new();
    let mut row = 0;
    for row_units in rows {
        if row_units.is_empty() {
            continue;
        }
        for (col, unit) in row_units.into_iter().enumerate() {
            units.push((
                unit,
                PRODUCTION_SELECTOR_OBJECT_OFFSET
                    + Vec2::new(
                        col as f32 * PRODUCTION_SELECTOR_OBJECT_STEP.x,
                        row as f32 * PRODUCTION_SELECTOR_OBJECT_STEP.y,
                    ),
            ));
        }
        row += 1;
    }
    units
}

fn production_building(kind: ObjectKind) -> Option<BuildingType> {
    match kind {
        ObjectKind::Building(
            building @ (BuildingType::FortFront
            | BuildingType::FortBack
            | BuildingType::RobotFactory
            | BuildingType::VehicleFactory),
        ) => Some(building),
        _ => None,
    }
}

pub(crate) fn crane_repair_points(
    center: Vec2,
    size: Vec2,
    kind: ObjectKind,
    bridge: Option<BridgeFootprint>,
    repairer_positions: &[Vec2],
) -> Option<(Vec2, Vec2)> {
    if let Some(bridge) = bridge {
        return bridge_crane_repair_points(bridge, repairer_positions);
    }

    let ObjectKind::Building(building) = kind else {
        return None;
    };
    let (entrance, center_point) = match building {
        BuildingType::Radar => radar_ui::crane_repair_local_points(size),
        BuildingType::Repair => repair_ui::crane_repair_local_points(size),
        BuildingType::RobotFactory => robot_factory_ui::crane_repair_local_points(size),
        BuildingType::VehicleFactory => vehicle_factory_ui::crane_repair_local_points(size),
        BuildingType::FortFront
        | BuildingType::FortBack
        | BuildingType::BridgeVert
        | BuildingType::BridgeHorz => return None,
    };
    Some((
        building_local_point(center, size, entrance),
        building_local_point(center, size, center_point),
    ))
}

pub(crate) fn repair_building_points(
    center: Vec2,
    size: Vec2,
    kind: ObjectKind,
) -> Option<(Vec2, Vec2)> {
    if !matches!(kind, ObjectKind::Building(BuildingType::Repair)) {
        return None;
    }
    let (entrance, center_point) = repair_ui::unit_repair_local_points(size);
    Some((
        building_local_point(center, size, entrance),
        building_local_point(center, size, center_point),
    ))
}

pub(crate) fn bridge_crane_repair_points(
    bridge: BridgeFootprint,
    repairer_positions: &[Vec2],
) -> Option<(Vec2, Vec2)> {
    let (top_left, width, height) = super::bridge_pixel_bounds(bridge)?;
    let center = map_point_to_world(top_left + Vec2::new(width * 0.5, height * 0.5));
    let entrances = match bridge.building {
        BuildingType::BridgeVert => bridge_vert_ui::crane_repair_entrances(top_left, width, height),
        BuildingType::BridgeHorz => bridge_horz_ui::crane_repair_entrances(top_left, width, height),
        _ => return None,
    };
    let entrance = repairer_positions
        .iter()
        .flat_map(|repairer| {
            entrances.into_iter().map(move |entrance| {
                (
                    entrance,
                    map_point_to_world(entrance).distance_squared(*repairer),
                )
            })
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map_or(entrances[0], |(entrance, _)| entrance);
    Some((map_point_to_world(entrance), center))
}

pub(crate) fn death_profile(kind: ObjectKind, planet: PlanetType) -> Option<BuildingDeathProfile> {
    let ObjectKind::Building(building) = kind else {
        return None;
    };

    match building {
        BuildingType::FortFront => Some(fort_front_ui::death_profile(planet)),
        BuildingType::FortBack => Some(fort_back_ui::death_profile(planet)),
        BuildingType::Radar => Some(radar_ui::death_profile()),
        BuildingType::Repair => Some(repair_ui::death_profile()),
        BuildingType::RobotFactory => Some(robot_factory_ui::death_profile()),
        BuildingType::VehicleFactory => Some(vehicle_factory_ui::death_profile()),
        BuildingType::BridgeVert | BuildingType::BridgeHorz => None,
    }
}

pub(crate) fn destroyed_asset_path(building: BuildingType, planet: PlanetType) -> Option<String> {
    match building {
        BuildingType::FortFront => Some(fort_front_ui::destroyed_asset_path(planet)),
        BuildingType::FortBack => Some(fort_back_ui::destroyed_asset_path(planet)),
        BuildingType::Radar => Some(radar_ui::destroyed_asset_path(planet)),
        BuildingType::Repair => Some(repair_ui::destroyed_asset_path(planet)),
        BuildingType::RobotFactory => Some(robot_factory_ui::destroyed_asset_path(planet)),
        BuildingType::VehicleFactory => Some(vehicle_factory_ui::destroyed_asset_path(planet)),
        BuildingType::BridgeVert | BuildingType::BridgeHorz => None,
    }
}

#[cfg(test)]
pub(crate) fn destroyed_asset_name(kind: ObjectKind, planet: PlanetType) -> Option<String> {
    match kind {
        ObjectKind::Building(building) => destroyed_asset_path(building, planet),
        ObjectKind::Bridge(BuildingType::BridgeVert | BuildingType::BridgeHorz) => None,
        _ => None,
    }
}

pub(crate) fn standard_max_effects(profile: BuildingDeathProfile, rng: &mut CombatRng) -> usize {
    profile.max_effects_base + rng.index(profile.max_effects_random)
}

pub(crate) fn death_fireball_count(profile: BuildingDeathProfile, rng: &mut CombatRng) -> usize {
    profile.fireball_base + rng.index(profile.fireball_random)
}

pub(crate) fn death_piece_count(profile: BuildingDeathProfile, rng: &mut CombatRng) -> usize {
    profile.piece_base + rng.index(profile.piece_random)
}

pub(crate) fn death_effect_point(
    top_left: Vec2,
    effect_box: BuildingEffectBox,
    rng: &mut CombatRng,
) -> Vec2 {
    top_left
        + Vec2::new(
            effect_box.x + rng.index(effect_box.width) as f32,
            effect_box.y + rng.index(effect_box.height) as f32,
        )
}

pub(crate) fn death_piece_target(
    top_left: Vec2,
    profile: BuildingDeathProfile,
    rng: &mut CombatRng,
) -> Vec2 {
    top_left
        + Vec2::new(
            profile.width_pix * 0.5 + 200.0 - rng.index(400) as f32,
            profile.height_pix * 0.5 + 200.0 - rng.index(400) as f32,
        )
}

pub(crate) fn piece_flight_time(profile: BuildingDeathProfile, rng: &mut CombatRng) -> f32 {
    profile.piece_flight_base + rng.index(200) as f32 * 0.01
}

pub(crate) fn building_top_left_from_grid(grid: MapGridPosition) -> Vec2 {
    Vec2::new(grid.x as f32 * TILE_SIZE, grid.y as f32 * TILE_SIZE)
}

pub(crate) fn building_turrent_rise(rng: &mut CombatRng) -> f32 {
    turrent_rise(rng)
}

pub(crate) fn turrent_rise(rng: &mut CombatRng) -> f32 {
    1.0 + rng.index(300) as f32 * 0.01
}

pub(crate) fn turrent_spin_degrees_per_sec(rng: &mut CombatRng) -> f32 {
    240.0 - rng.index(480) as f32
}

pub(crate) fn building_piece_frame_paths_for_variants(
    piece_index: usize,
    variants: usize,
) -> Vec<String> {
    let fort_piece = variants > 2;
    let piece_index = piece_index.min(variants.saturating_sub(1));
    (0..12)
        .map(|frame| {
            if fort_piece {
                format!("buildings/death_effects/fort_piece{piece_index}_n{frame:02}.png")
            } else {
                format!("buildings/death_effects/piece{piece_index}_n{frame:02}.png")
            }
        })
        .collect()
}

pub(crate) fn bridge_turrent_spawn_points(
    bridge: BridgeFootprint,
    rng: &mut CombatRng,
) -> Vec<Vec2> {
    match bridge.building {
        BuildingType::BridgeVert => bridge_vert_ui::turrent_spawn_points(bridge, rng),
        BuildingType::BridgeHorz => bridge_horz_ui::turrent_spawn_points(bridge, rng),
        _ => Vec::new(),
    }
}

pub(crate) fn bridge_turrent_frame_paths(planet: PlanetType) -> Vec<String> {
    match planet {
        PlanetType::Desert
        | PlanetType::Volcanic
        | PlanetType::Arctic
        | PlanetType::Jungle
        | PlanetType::City => bridge_vert_ui::turrent_frame_paths(planet),
    }
}

pub(crate) fn bridge_rock_particle_frame_paths(planet: PlanetType) -> Vec<String> {
    match planet {
        PlanetType::Desert
        | PlanetType::Volcanic
        | PlanetType::Arctic
        | PlanetType::Jungle
        | PlanetType::City => bridge_vert_ui::rock_particle_frame_paths(planet),
    }
}

pub(crate) fn bridge_turrent_trajectory(
    anchor_map: Vec2,
    reversed: bool,
    rng: &mut CombatRng,
) -> EffectTrajectory {
    bridge_vert_ui::turrent_trajectory(anchor_map, reversed, rng)
}

pub(crate) fn bridge_rock_particle_trajectory(
    anchor_map: Vec2,
    rng: &mut CombatRng,
) -> EffectTrajectory {
    bridge_vert_ui::rock_particle_trajectory(anchor_map, rng)
}

pub(crate) fn bridge_end_particle_count(rng: &mut CombatRng) -> usize {
    bridge_vert_ui::end_particle_count(rng)
}

pub(crate) fn bridge_turrent_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    arc_size(rise, final_time, t)
}

pub(crate) fn bridge_rock_particle_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    arc_size(rise, final_time, t)
}

pub(crate) fn radar_overlay_should_be_visible(
    kind: RadarOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
) -> bool {
    radar_ui::overlay_should_be_visible(kind, owner, stats)
}

pub(crate) fn repair_overlay_should_be_visible(
    kind: RepairOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    repairing_unit: bool,
) -> bool {
    repair_ui::overlay_should_be_visible(kind, owner, stats, repairing_unit)
}

pub(crate) fn repair_overlay_forced_frame(
    kind: RepairOverlayKind,
    owner: TeamType,
    repairing_unit: bool,
) -> Option<usize> {
    repair_ui::overlay_forced_frame(kind, owner, repairing_unit)
}

pub(crate) fn factory_overlay_should_be_visible(
    kind: FactoryOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    production_active: bool,
    current: usize,
) -> bool {
    if robot_factory_ui::overlay_kind_is_robot(kind) {
        return robot_factory_ui::overlay_should_be_visible(
            kind,
            owner,
            stats,
            production_active,
            current,
        );
    }

    vehicle_factory_ui::overlay_should_be_visible(kind, owner, stats, production_active)
}

pub(crate) fn factory_overlay_next_frame(
    ref_id: u32,
    kind: FactoryOverlayKind,
    current: usize,
    frame_count: usize,
    rng: &mut CombatRng,
    robot_light_updates: &mut HashMap<u32, Option<[usize; 3]>>,
) -> usize {
    robot_factory_ui::overlay_next_frame(
        ref_id,
        kind,
        current,
        frame_count,
        rng,
        robot_light_updates,
    )
}

pub(crate) fn planet_asset_name(planet: PlanetType) -> &'static str {
    match planet {
        PlanetType::Desert => "desert",
        PlanetType::Volcanic => "volcanic",
        PlanetType::Arctic => "arctic",
        PlanetType::Jungle => "jungle",
        PlanetType::City => "city",
    }
}

fn arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    -(rise / final_time) * (t * t) + rise * t + 1.0
}

fn building_local_point(center: Vec2, size: Vec2, local: Vec2) -> Vec2 {
    let top_left = Vec2::new(center.x - size.x * 0.5, center.y + size.y * 0.5);
    Vec2::new(top_left.x + local.x, top_left.y - local.y)
}

fn map_point_to_world(point: Vec2) -> Vec2 {
    Vec2::new(point.x, -point.y)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
