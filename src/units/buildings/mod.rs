use bevy::prelude::Vec2;

use crate::{
    components::{
        BridgeFootprint, CombatRng, MapGridPosition, ObjectStats, PassabilityGrid, ProductionLevel,
        ProductionWindowKind,
    },
    constants::TILE_SIZE,
    original::objects::{BuildingType, ObjectKind},
    original::types::{PlanetType, TeamType},
};

#[path = "bridge_horz/bridge_horz_mod.rs"]
pub(crate) mod bridge_horz;
#[path = "bridge_vert/bridge_vert_mod.rs"]
pub(crate) mod bridge_vert;
pub(crate) mod building_ui;
#[path = "fort_back/fort_back_mod.rs"]
pub(crate) mod fort_back;
#[path = "fort_front/fort_front_mod.rs"]
pub(crate) mod fort_front;
pub(crate) mod production_logic;
#[path = "radar/radar_mod.rs"]
pub(crate) mod radar;
#[path = "repair/repair_mod.rs"]
pub(crate) mod repair;
#[path = "robot_factory/robot_factory_mod.rs"]
pub(crate) mod robot_factory;
#[path = "vehicle_factory/vehicle_factory_mod.rs"]
pub(crate) mod vehicle_factory;

pub(crate) mod bridge_horz_ui {
    pub(crate) use super::bridge_horz::bridge_horz_ui::*;
}

pub(crate) mod bridge_vert_ui {
    pub(crate) use super::bridge_vert::bridge_vert_ui::*;
}

pub(crate) mod fort_back_ui {
    pub(crate) use super::fort_back::fort_back_ui::*;
}

pub(crate) mod fort_front_ui {
    pub(crate) use super::fort_front::fort_front_ui::*;
}

pub(crate) mod radar_ui {
    pub(crate) use super::radar::radar_ui::*;
}

pub(crate) mod repair_ui {
    pub(crate) use super::repair::repair_ui::*;
}

pub(crate) mod robot_factory_ui {
    pub(crate) use super::robot_factory::robot_factory_ui::*;
}

pub(crate) mod vehicle_factory_ui {
    pub(crate) use super::vehicle_factory::vehicle_factory_ui::*;
}

const AUTO_REPAIR_TIME: f32 = 10.0 * 60.0;
const AUTO_REPAIR_RANDOM_ADDITIONAL_TIME: usize = 61;

#[cfg(test)]
pub(crate) use building_ui::ProductionSelectorGeometrySpec;
pub(crate) use building_ui::{
    BRIDGE_REVIVE_RERENDER_DELAY, BRIDGE_ROCK_PARTICLE_FRAME_TIME, BRIDGE_TURRENT_FRAME_TIME,
    BRIDGE_TURRENT_MAX_DISTANCE, BUILDING_TURRENT_FRAME_TIME, BridgeVisualState,
    BridgeVisualTileSpec, BuildingAtlasFrameSpec, BuildingOverlayFrameSpec, ProductionPlacement,
};
pub(crate) use repair::{
    REPAIR_BUILDING_SECONDS, repaired_unit_source_points, repaired_unit_waypoints,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BuildingEffectBox {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BuildingDeathProfile {
    pub(crate) effect_box: BuildingEffectBox,
    pub(crate) width_pix: f32,
    pub(crate) height_pix: f32,
    pub(crate) max_effects_base: usize,
    pub(crate) max_effects_random: usize,
    pub(crate) fireball_base: usize,
    pub(crate) fireball_random: usize,
    pub(crate) piece_base: usize,
    pub(crate) piece_random: usize,
    pub(crate) piece_variants: usize,
    pub(crate) piece_flight_base: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EffectTrajectory {
    pub(crate) start: Vec2,
    pub(crate) end: Vec2,
    pub(crate) final_time: f32,
    pub(crate) rise: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WorldRect {
    pub(crate) min_x: f32,
    pub(crate) max_x: f32,
    pub(crate) min_y: f32,
    pub(crate) max_y: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BuildingPathingRect {
    pub(crate) dx: u16,
    pub(crate) dy: u16,
    pub(crate) width: u16,
    pub(crate) height: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BuildingPathingSpec {
    pub(crate) blocked_rects: Vec<BuildingPathingRect>,
    pub(crate) blocked_masks: Vec<&'static [&'static str]>,
    pub(crate) unblocked_tiles: Vec<(u16, u16)>,
}

pub(crate) fn default_selection_size(building: BuildingType) -> Vec2 {
    building_ui::default_selection_size(building)
}

pub(crate) fn health_ratio(building: BuildingType) -> f32 {
    match building {
        BuildingType::FortFront => fort_front::health_ratio(),
        BuildingType::FortBack => fort_back::health_ratio(),
        BuildingType::Radar => radar::health_ratio(),
        BuildingType::Repair => repair::health_ratio(),
        BuildingType::RobotFactory => robot_factory::health_ratio(),
        BuildingType::VehicleFactory => vehicle_factory::health_ratio(),
        BuildingType::BridgeVert => bridge_vert::health_ratio(),
        BuildingType::BridgeHorz => bridge_horz::health_ratio(),
    }
}

pub(crate) fn default_production_unit(
    building: BuildingType,
    level: impl Into<ProductionLevel>,
) -> Option<ObjectKind> {
    let level = level.into();
    let unit = match building {
        BuildingType::FortFront => fort_front::default_production_unit(),
        BuildingType::FortBack => fort_back::default_production_unit(),
        BuildingType::RobotFactory => robot_factory::default_production_unit(),
        BuildingType::VehicleFactory => vehicle_factory::default_production_unit(),
        BuildingType::Radar
        | BuildingType::Repair
        | BuildingType::BridgeVert
        | BuildingType::BridgeHorz => return None,
    };

    unit_in_default_build_list(building, level, unit).then_some(unit)
}

pub(crate) fn unit_in_default_build_list(
    building: BuildingType,
    level: impl Into<ProductionLevel>,
    unit: ObjectKind,
) -> bool {
    default_build_list(building, level).contains(&unit)
}

pub(crate) fn default_build_list(
    building: BuildingType,
    level: impl Into<ProductionLevel>,
) -> Vec<ObjectKind> {
    let level = level.into();
    match building {
        BuildingType::FortFront => fort_front::default_build_list(level),
        BuildingType::FortBack => fort_back::default_build_list(level),
        BuildingType::RobotFactory => robot_factory::default_build_list(level),
        BuildingType::VehicleFactory => vehicle_factory::default_build_list(level),
        BuildingType::Radar
        | BuildingType::Repair
        | BuildingType::BridgeVert
        | BuildingType::BridgeHorz => Vec::new(),
    }
}

pub(crate) fn production_world_points(
    building: BuildingType,
    tile_x: u16,
    tile_y: u16,
) -> Option<(Vec2, Vec2)> {
    building_ui::production_world_points(building, tile_x, tile_y)
}

#[cfg(test)]
pub(crate) fn production_selector_geometry() -> ProductionSelectorGeometrySpec {
    building_ui::production_selector_geometry()
}

pub(crate) fn production_label_asset_path(kind: ProductionWindowKind) -> &'static str {
    building_ui::production_label_asset_path(kind)
}

#[cfg(test)]
pub(crate) fn production_label_asset_path_for_building(
    building: BuildingType,
) -> Option<&'static str> {
    building_ui::production_label_asset_path_for_building(building)
}

#[cfg(test)]
pub(crate) fn fort_entrance_rect(building: BuildingType) -> Option<(f32, f32, f32, f32)> {
    building_ui::fort_entrance_rect(building)
}

pub(crate) fn point_in_fort_entrance_rect(
    point: Vec2,
    center: Vec2,
    size: Vec2,
    building: BuildingType,
) -> bool {
    building_ui::point_in_fort_entrance_rect(point, center, size, building)
}

pub(crate) fn fallback_collision_size(building: BuildingType) -> Vec2 {
    building_ui::fallback_collision_size(building)
}

pub(crate) fn map_object_has_fort_turret_tile(
    object_type: crate::original::map::MapObjectType,
    object_id: u8,
    object_tile_x: i32,
    object_tile_y: i32,
    tile_x: i32,
    tile_y: i32,
) -> bool {
    building_ui::map_object_has_fort_turret_tile(
        object_type,
        object_id,
        object_tile_x,
        object_tile_y,
        tile_x,
        tile_y,
    )
}

pub(crate) fn fort_turret_slot_allows(
    kind: ObjectKind,
    center: Vec2,
    size: Vec2,
    map_left: f32,
    map_top: f32,
) -> bool {
    building_ui::fort_turret_slot_allows(kind, center, size, map_left, map_top)
}

pub(crate) fn fort_entry_points(
    center: Vec2,
    size: Vec2,
    building: BuildingType,
) -> Option<(Vec2, Vec2)> {
    building_ui::fort_entry_points(center, size, building)
}

pub(crate) fn building_layer_specs(
    building: BuildingType,
    team: TeamType,
    planet: PlanetType,
) -> Vec<BuildingAtlasFrameSpec> {
    building_ui::building_layer_specs(building, team, planet)
}

pub(crate) fn bridge_visual_state(health_percent: i32) -> BridgeVisualState {
    building_ui::bridge_visual_state(health_percent)
}

pub(crate) fn bridge_visual_tile_specs(
    building: BuildingType,
    extra_links: u16,
    state: BridgeVisualState,
) -> Vec<BridgeVisualTileSpec> {
    building_ui::bridge_visual_tile_specs(building, extra_links, state)
}

pub(crate) fn bridge_pathing_dimensions(
    building: BuildingType,
    extra_links: u16,
) -> Option<(u16, u16)> {
    building_ui::bridge_pathing_dimensions(building, extra_links)
}

pub(crate) fn bridge_edge_block_offsets(
    building: BuildingType,
    extra_links: u16,
) -> Vec<(u16, u16)> {
    building_ui::bridge_edge_block_offsets(building, extra_links)
}

pub(crate) fn bridge_center_offsets(building: BuildingType, extra_links: u16) -> Vec<(u16, u16)> {
    building_ui::bridge_center_offsets(building, extra_links)
}

pub(crate) fn building_pathing_spec(building: BuildingType) -> BuildingPathingSpec {
    building_ui::building_pathing_spec(building)
}

#[cfg(test)]
pub(crate) fn bridge_vert_fill_index(state: BridgeVisualState, row: usize) -> usize {
    building_ui::bridge_vert_fill_index(state, row)
}

#[cfg(test)]
pub(crate) fn bridge_horz_fill_index(state: BridgeVisualState, col: usize) -> usize {
    building_ui::bridge_horz_fill_index(state, col)
}

#[cfg(test)]
pub(crate) fn radar_overlay_frame_names(
    kind: crate::render::atlas::RadarOverlayKind,
) -> Vec<String> {
    building_ui::radar_overlay_frame_names(kind)
}

#[cfg(test)]
pub(crate) fn radar_overlay_offsets(kind: crate::render::atlas::RadarOverlayKind) -> Vec<Vec2> {
    building_ui::radar_overlay_offsets(kind)
}

pub(crate) fn radar_overlay_frame_specs(
    kind: crate::render::atlas::RadarOverlayKind,
) -> Vec<BuildingOverlayFrameSpec> {
    building_ui::radar_overlay_frame_specs(kind)
}

#[cfg(test)]
pub(crate) fn radar_overlay_kinds() -> &'static [crate::render::atlas::RadarOverlayKind] {
    building_ui::radar_overlay_kinds()
}

pub(crate) fn radar_overlay_kinds_for_object(
    kind: ObjectKind,
) -> Option<&'static [crate::render::atlas::RadarOverlayKind]> {
    building_ui::radar_overlay_kinds_for_object(kind)
}

pub(crate) fn radar_overlay_frame_time() -> f32 {
    building_ui::radar_overlay_frame_time()
}

#[cfg(test)]
pub(crate) fn repair_overlay_frame_names(
    kind: crate::render::atlas::RepairOverlayKind,
) -> Vec<String> {
    building_ui::repair_overlay_frame_names(kind)
}

#[cfg(test)]
pub(crate) fn repair_overlay_offsets(kind: crate::render::atlas::RepairOverlayKind) -> Vec<Vec2> {
    building_ui::repair_overlay_offsets(kind)
}

pub(crate) fn repair_overlay_frame_specs(
    kind: crate::render::atlas::RepairOverlayKind,
) -> Vec<BuildingOverlayFrameSpec> {
    building_ui::repair_overlay_frame_specs(kind)
}

#[cfg(test)]
pub(crate) fn repair_overlay_kinds() -> &'static [crate::render::atlas::RepairOverlayKind] {
    building_ui::repair_overlay_kinds()
}

pub(crate) fn repair_overlay_kinds_for_object(
    kind: ObjectKind,
) -> Option<&'static [crate::render::atlas::RepairOverlayKind]> {
    building_ui::repair_overlay_kinds_for_object(kind)
}

pub(crate) fn repair_overlay_frame_time() -> f32 {
    building_ui::repair_overlay_frame_time()
}

pub(crate) fn repair_overlay_initial_frame(
    kind: crate::render::atlas::RepairOverlayKind,
    owner: TeamType,
    repairing_unit: bool,
) -> usize {
    building_ui::repair_overlay_initial_frame(kind, owner, repairing_unit)
}

#[cfg(test)]
pub(crate) fn factory_overlay_frame_names(
    kind: crate::render::atlas::FactoryOverlayKind,
) -> Vec<String> {
    building_ui::factory_overlay_frame_names(kind)
}

#[cfg(test)]
pub(crate) fn factory_overlay_offsets(kind: crate::render::atlas::FactoryOverlayKind) -> Vec<Vec2> {
    building_ui::factory_overlay_offsets(kind)
}

pub(crate) fn factory_overlay_frame_specs(
    kind: crate::render::atlas::FactoryOverlayKind,
) -> Vec<BuildingOverlayFrameSpec> {
    building_ui::factory_overlay_frame_specs(kind)
}

#[cfg(test)]
pub(crate) fn factory_overlay_kinds(
    building: BuildingType,
) -> Option<&'static [crate::render::atlas::FactoryOverlayKind]> {
    building_ui::factory_overlay_kinds(building)
}

pub(crate) fn factory_overlay_kinds_for_object(
    kind: ObjectKind,
) -> Option<&'static [crate::render::atlas::FactoryOverlayKind]> {
    building_ui::factory_overlay_kinds_for_object(kind)
}

pub(crate) fn factory_overlay_frame_time() -> f32 {
    building_ui::factory_overlay_frame_time()
}

pub(crate) fn factory_overlay_process_tick(
    elapsed: f32,
    delta_secs: f32,
    frame_time: f32,
) -> (bool, f32) {
    building_ui::factory_overlay_process_tick(elapsed, delta_secs, frame_time)
}

pub(crate) fn factory_overlay_initial_frame_visible(
    kind: crate::render::atlas::FactoryOverlayKind,
) -> bool {
    building_ui::factory_overlay_initial_frame_visible(kind)
}

pub(crate) fn production_window_kind(kind: ObjectKind) -> Option<ProductionWindowKind> {
    building_ui::production_window_kind(kind)
}

pub(crate) fn selected_production_unit(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    selected_index: usize,
) -> Option<ObjectKind> {
    building_ui::selected_production_unit(kind, level, selected_index)
}

pub(crate) fn cycle_production_selection(
    kind: ObjectKind,
    level: impl Into<ProductionLevel> + Copy,
    selected_index: usize,
    direction: i32,
) -> usize {
    building_ui::cycle_production_selection(kind, level, selected_index, direction)
}

pub(crate) fn production_selector_unit_index(
    kind: ObjectKind,
    level: impl Into<ProductionLevel>,
    unit: ObjectKind,
) -> Option<usize> {
    building_ui::production_selector_unit_index(kind, level, unit)
}

pub(crate) fn production_selector_units(
    kind: ObjectKind,
    level: impl Into<ProductionLevel>,
) -> Vec<(ObjectKind, Vec2)> {
    building_ui::production_selector_units(kind, level)
}

pub(crate) fn can_set_rallypoints(kind: ObjectKind) -> bool {
    production_logic::can_set_rallypoints(kind)
}

pub(crate) fn crane_repair_points(
    center: Vec2,
    size: Vec2,
    kind: ObjectKind,
    bridge: Option<BridgeFootprint>,
    repairer_positions: &[Vec2],
) -> Option<(Vec2, Vec2)> {
    building_ui::crane_repair_points(center, size, kind, bridge, repairer_positions)
}

pub(crate) fn repair_building_points(
    center: Vec2,
    size: Vec2,
    kind: ObjectKind,
) -> Option<(Vec2, Vec2)> {
    building_ui::repair_building_points(center, size, kind)
}

pub(crate) fn can_repair_unit(
    kind: ObjectKind,
    building_team: TeamType,
    unit_team: TeamType,
    stats: ObjectStats,
) -> bool {
    repair::can_repair_unit(kind, building_team, unit_team, stats)
}

pub(crate) fn can_repair_target_unit(kind: ObjectKind, team: TeamType, stats: ObjectStats) -> bool {
    repair::can_repair_target_unit(kind, team, stats)
}

pub(crate) fn death_profile(kind: ObjectKind, planet: PlanetType) -> Option<BuildingDeathProfile> {
    building_ui::death_profile(kind, planet)
}

pub(crate) fn destroyed_asset_path(building: BuildingType, planet: PlanetType) -> Option<String> {
    building_ui::destroyed_asset_path(building, planet)
}

#[cfg(test)]
pub(crate) fn destroyed_asset_name(kind: ObjectKind, planet: PlanetType) -> Option<String> {
    building_ui::destroyed_asset_name(kind, planet)
}

pub(crate) fn standard_max_effects(profile: BuildingDeathProfile, rng: &mut CombatRng) -> usize {
    building_ui::standard_max_effects(profile, rng)
}

pub(crate) fn death_fireball_count(profile: BuildingDeathProfile, rng: &mut CombatRng) -> usize {
    building_ui::death_fireball_count(profile, rng)
}

pub(crate) fn death_piece_count(profile: BuildingDeathProfile, rng: &mut CombatRng) -> usize {
    building_ui::death_piece_count(profile, rng)
}

pub(crate) fn death_effect_point(
    top_left: Vec2,
    effect_box: BuildingEffectBox,
    rng: &mut CombatRng,
) -> Vec2 {
    building_ui::death_effect_point(top_left, effect_box, rng)
}

pub(crate) fn death_piece_target(
    top_left: Vec2,
    profile: BuildingDeathProfile,
    rng: &mut CombatRng,
) -> Vec2 {
    building_ui::death_piece_target(top_left, profile, rng)
}

pub(crate) fn piece_flight_time(profile: BuildingDeathProfile, rng: &mut CombatRng) -> f32 {
    building_ui::piece_flight_time(profile, rng)
}

pub(crate) fn building_top_left_from_grid(grid: MapGridPosition) -> Vec2 {
    building_ui::building_top_left_from_grid(grid)
}

pub(crate) fn building_turrent_rise(rng: &mut CombatRng) -> f32 {
    building_ui::building_turrent_rise(rng)
}

pub(crate) fn turrent_spin_degrees_per_sec(rng: &mut CombatRng) -> f32 {
    building_ui::turrent_spin_degrees_per_sec(rng)
}

pub(crate) fn building_piece_frame_paths_for_variants(
    piece_index: usize,
    variants: usize,
) -> Vec<String> {
    building_ui::building_piece_frame_paths_for_variants(piece_index, variants)
}

pub(crate) fn auto_repairable_after_destroy(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Building(
            BuildingType::Radar
                | BuildingType::Repair
                | BuildingType::RobotFactory
                | BuildingType::VehicleFactory
        ) | ObjectKind::Bridge(BuildingType::BridgeVert | BuildingType::BridgeHorz)
    )
}

pub(crate) fn auto_repair_delay(rng: &mut CombatRng) -> f32 {
    AUTO_REPAIR_TIME + rng.index(AUTO_REPAIR_RANDOM_ADDITIONAL_TIME) as f32
}

pub(crate) fn auto_repair_blocking_fort(kind: ObjectKind) -> bool {
    matches!(kind, ObjectKind::Building(BuildingType::FortFront))
}

pub(crate) fn bridge_turrent_spawn_points(
    bridge: BridgeFootprint,
    rng: &mut CombatRng,
) -> Vec<Vec2> {
    building_ui::bridge_turrent_spawn_points(bridge, rng)
}

pub(crate) fn bridge_turrent_frame_paths(planet: PlanetType) -> Vec<String> {
    building_ui::bridge_turrent_frame_paths(planet)
}

pub(crate) fn bridge_rock_particle_frame_paths(planet: PlanetType) -> Vec<String> {
    building_ui::bridge_rock_particle_frame_paths(planet)
}

pub(crate) fn bridge_turrent_trajectory(
    anchor_map: Vec2,
    reversed: bool,
    rng: &mut CombatRng,
) -> EffectTrajectory {
    building_ui::bridge_turrent_trajectory(anchor_map, reversed, rng)
}

pub(crate) fn bridge_rock_particle_trajectory(
    anchor_map: Vec2,
    rng: &mut CombatRng,
) -> EffectTrajectory {
    building_ui::bridge_rock_particle_trajectory(anchor_map, rng)
}

pub(crate) fn bridge_end_particle_count(rng: &mut CombatRng) -> usize {
    building_ui::bridge_end_particle_count(rng)
}

pub(crate) fn bridge_turrent_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    building_ui::bridge_turrent_arc_size(rise, final_time, t)
}

pub(crate) fn bridge_rock_particle_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    building_ui::bridge_rock_particle_arc_size(rise, final_time, t)
}

pub(crate) fn bridge_world_rect(bridge: BridgeFootprint) -> Option<WorldRect> {
    let (x, y, width, height) =
        PassabilityGrid::bridge_bounds(bridge.x, bridge.y, bridge.building, bridge.extra_links)?;
    Some(WorldRect {
        min_x: x as f32 * TILE_SIZE,
        max_x: x.saturating_add(width) as f32 * TILE_SIZE,
        min_y: -(y.saturating_add(height) as f32 * TILE_SIZE),
        max_y: -(y as f32 * TILE_SIZE),
    })
}

pub(crate) fn bridge_pixel_bounds(bridge: BridgeFootprint) -> Option<(Vec2, f32, f32)> {
    let (x, y, width, height) =
        PassabilityGrid::bridge_bounds(bridge.x, bridge.y, bridge.building, bridge.extra_links)?;
    Some((
        Vec2::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE),
        width as f32 * TILE_SIZE,
        height as f32 * TILE_SIZE,
    ))
}

pub(crate) fn repair_target_map_bounds(
    center: Vec2,
    size: Vec2,
    bridge: Option<BridgeFootprint>,
) -> (Vec2, Vec2) {
    if let Some((top_left, width, height)) = bridge.and_then(bridge_pixel_bounds) {
        return (top_left, Vec2::new(width, height));
    }

    (object_top_left_map(center, size), size)
}

pub(crate) fn object_top_left_map(center: Vec2, size: Vec2) -> Vec2 {
    Vec2::new(center.x - size.x * 0.5, -center.y - size.y * 0.5)
}

pub(crate) fn bridge_destroy_kills_unit(
    kind: ObjectKind,
    stats: ObjectStats,
    position: Vec2,
    selection_size: Vec2,
    bridge: BridgeFootprint,
) -> bool {
    matches!(kind, ObjectKind::Robot(_) | ObjectKind::Vehicle(_))
        && !stats.destroyed()
        && bridge_world_rect(bridge).is_some_and(|bridge_rect| {
            rects_intersect(object_world_rect(position, selection_size), bridge_rect)
        })
}

pub(crate) fn object_world_rect(position: Vec2, size: Vec2) -> WorldRect {
    let half_size = size * 0.5;
    WorldRect {
        min_x: position.x - half_size.x,
        max_x: position.x + half_size.x,
        min_y: position.y - half_size.y,
        max_y: position.y + half_size.y,
    }
}

pub(crate) fn rects_intersect(a: WorldRect, b: WorldRect) -> bool {
    a.min_x < b.max_x && a.max_x > b.min_x && a.min_y < b.max_y && a.max_y > b.min_y
}

pub(crate) fn radar_overlay_should_be_visible(
    kind: crate::render::atlas::RadarOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
) -> bool {
    building_ui::radar_overlay_should_be_visible(kind, owner, stats)
}

pub(crate) fn repair_overlay_should_be_visible(
    kind: crate::render::atlas::RepairOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    repairing_unit: bool,
) -> bool {
    building_ui::repair_overlay_should_be_visible(kind, owner, stats, repairing_unit)
}

pub(crate) fn repair_overlay_forced_frame(
    kind: crate::render::atlas::RepairOverlayKind,
    owner: TeamType,
    repairing_unit: bool,
) -> Option<usize> {
    building_ui::repair_overlay_forced_frame(kind, owner, repairing_unit)
}

pub(crate) fn factory_overlay_should_be_visible(
    kind: crate::render::atlas::FactoryOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    production_active: bool,
    current: usize,
) -> bool {
    building_ui::factory_overlay_should_be_visible(kind, owner, stats, production_active, current)
}

pub(crate) fn factory_overlay_next_frame(
    ref_id: u32,
    kind: crate::render::atlas::FactoryOverlayKind,
    current: usize,
    frame_count: usize,
    rng: &mut CombatRng,
    robot_light_updates: &mut std::collections::HashMap<u32, Option<[usize; 3]>>,
) -> usize {
    building_ui::factory_overlay_next_frame(
        ref_id,
        kind,
        current,
        frame_count,
        rng,
        robot_light_updates,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::ProductionWindowKind,
        original::objects::{CannonType, RobotType, VehicleType},
        render::atlas::{FactoryOverlayKind, RadarOverlayKind, RepairOverlayKind},
    };

    #[test]
    fn building_piece_assets_and_timing_match_original_turrent_missiles() {
        assert_eq!(building_piece_frame_paths_for_variants(0, 2).len(), 12);
        assert_eq!(
            building_piece_frame_paths_for_variants(0, 2)
                .first()
                .unwrap(),
            "buildings/death_effects/piece0_n00.png"
        );
        assert_eq!(
            building_piece_frame_paths_for_variants(1, 2)
                .last()
                .unwrap(),
            "buildings/death_effects/piece1_n11.png"
        );
        assert_eq!(
            building_piece_frame_paths_for_variants(4, 5)
                .last()
                .unwrap(),
            "buildings/death_effects/fort_piece4_n11.png"
        );
        assert_eq!(
            building_top_left_from_grid(MapGridPosition { x: 3, y: 4 }),
            Vec2::new(48.0, 64.0)
        );
        assert_eq!(BUILDING_TURRENT_FRAME_TIME, 0.1);
    }

    #[test]
    fn destroyed_building_assets_match_original_paths() {
        assert_eq!(
            destroyed_asset_name(
                ObjectKind::Building(BuildingType::Radar),
                PlanetType::Arctic
            ),
            Some("buildings/radar/base_destroyed_arctic.png".to_string())
        );
        assert_eq!(
            destroyed_asset_name(
                ObjectKind::Building(BuildingType::FortFront),
                PlanetType::Desert
            ),
            Some("buildings/fort/fort_desert_front_destroyed.png".to_string())
        );
        assert_eq!(
            destroyed_asset_name(
                ObjectKind::Building(BuildingType::BridgeHorz),
                PlanetType::Desert
            ),
            None
        );
        assert_eq!(
            destroyed_asset_name(ObjectKind::Cannon(CannonType::Gun), PlanetType::Desert),
            None
        );
    }

    #[test]
    fn production_window_kind_matches_producing_buildings_only() {
        assert_eq!(
            production_window_kind(ObjectKind::Building(BuildingType::RobotFactory)),
            Some(ProductionWindowKind::Robot)
        );
        assert_eq!(
            production_window_kind(ObjectKind::Building(BuildingType::VehicleFactory)),
            Some(ProductionWindowKind::Vehicle)
        );
        assert_eq!(
            production_window_kind(ObjectKind::Building(BuildingType::FortBack)),
            Some(ProductionWindowKind::Fort)
        );
        assert_eq!(
            production_window_kind(ObjectKind::Building(BuildingType::Radar)),
            None
        );
        assert_eq!(
            production_window_kind(ObjectKind::Robot(RobotType::Grunt)),
            None
        );
    }

    #[test]
    fn production_selector_groups_build_list_into_original_ui_rows() {
        let robot_factory = ObjectKind::Building(BuildingType::RobotFactory);

        assert_eq!(
            production_selector_units(robot_factory, 0),
            vec![
                (ObjectKind::Robot(RobotType::Grunt), Vec2::new(6.0, 22.0)),
                (
                    ObjectKind::Cannon(CannonType::Gatling),
                    Vec2::new(6.0, 75.0)
                ),
            ]
        );
        assert_eq!(
            production_selector_unit_index(
                robot_factory,
                0,
                ObjectKind::Cannon(CannonType::Gatling)
            ),
            Some(1)
        );
        assert_eq!(
            selected_production_unit(robot_factory, 0, usize::MAX),
            Some(ObjectKind::Robot(RobotType::Grunt))
        );
        assert_eq!(cycle_production_selection(robot_factory, 0, 1, 1), 0);
    }

    #[test]
    fn production_geometry_and_labels_match_building_ui_specs() {
        assert_eq!(
            production_selector_geometry(),
            ProductionSelectorGeometrySpec {
                object_offset: Vec2::new(6.0, 22.0),
                object_step: Vec2::new(47.0, 53.0)
            }
        );
        assert_eq!(
            production_world_points(BuildingType::RobotFactory, 10, 20),
            Some((Vec2::new(203.0, -373.0), Vec2::new(203.0, -416.0)))
        );
        assert_eq!(
            production_label_asset_path(ProductionWindowKind::Fort),
            "other/production_gui/fort_factory_label.png"
        );
        assert_eq!(
            production_label_asset_path_for_building(BuildingType::VehicleFactory),
            Some("other/production_gui/vehicle_factory_label.png")
        );
        assert_eq!(
            production_label_asset_path_for_building(BuildingType::Radar),
            None
        );
    }

    #[test]
    fn building_atlas_layer_specs_route_to_per_unit_ui_modules() {
        assert_eq!(
            building_layer_specs(BuildingType::Radar, TeamType::Blue, PlanetType::Desert),
            vec![
                BuildingAtlasFrameSpec {
                    atlas_team: TeamType::Red,
                    frame_name: "building_radar_base_desert".to_string(),
                    world_offset: Vec2::ZERO,
                    animation_frame_names: Vec::new()
                },
                BuildingAtlasFrameSpec {
                    atlas_team: TeamType::Blue,
                    frame_name: "building_radar_blue".to_string(),
                    world_offset: Vec2::new(0.0, 32.0),
                    animation_frame_names: Vec::new()
                }
            ]
        );

        let fort_specs =
            building_layer_specs(BuildingType::FortFront, TeamType::Yellow, PlanetType::City);
        assert_eq!(fort_specs[0].frame_name, "fort_city_front");
        assert_eq!(
            fort_specs[1]
                .animation_frame_names
                .last()
                .map(String::as_str),
            Some("fort_flag_yellow_n03")
        );
        assert_eq!(fort_specs[1].world_offset, Vec2::new(85.0, 29.0));
    }

    #[test]
    fn overlay_specs_are_available_from_building_facade() {
        assert_eq!(radar_overlay_kinds().len(), 4);
        assert_eq!(repair_overlay_kinds().len(), 5);
        assert_eq!(
            factory_overlay_kinds(BuildingType::RobotFactory)
                .unwrap()
                .len(),
            8
        );
        assert_eq!(radar_overlay_frame_time(), 0.25);
        assert_eq!(repair_overlay_frame_time(), 0.35);
        assert_eq!(factory_overlay_frame_time(), 0.25);
        assert_eq!(factory_overlay_process_tick(0.1, 0.1, 0.25), (false, 0.2));
        assert_eq!(factory_overlay_process_tick(0.2, 0.1, 0.25), (true, 0.0));
        assert_eq!(factory_overlay_process_tick(0.0, 0.8, 0.25), (true, 0.0));

        assert_eq!(
            radar_overlay_frame_specs(RadarOverlayKind::FrontLight)[0],
            BuildingOverlayFrameSpec {
                frame_name: "building_radar_front_light_0".to_string(),
                world_offset: Vec2::new(16.0, 22.0)
            }
        );
        assert_eq!(
            repair_overlay_initial_frame(RepairOverlayKind::SideLight, TeamType::Red, false),
            1
        );
        assert_eq!(
            repair_overlay_frame_specs(RepairOverlayKind::SmokeStack)[0],
            BuildingOverlayFrameSpec {
                frame_name: "building_repair_smoke_stack_0".to_string(),
                world_offset: Vec2::new(61.0, 0.0)
            }
        );
        assert_eq!(
            factory_overlay_frame_specs(FactoryOverlayKind::VehicleLight1)[0],
            BuildingOverlayFrameSpec {
                frame_name: "building_vehicle_lights_1".to_string(),
                world_offset: Vec2::new(42.0, 47.0)
            }
        );
        assert!(!factory_overlay_initial_frame_visible(
            FactoryOverlayKind::RobotSingleLight0
        ));
    }

    #[test]
    fn bridge_visual_tile_specs_match_original_bridge_atlas_policy() {
        assert_eq!(bridge_visual_state(50), BridgeVisualState::Live);
        assert_eq!(bridge_visual_state(49), BridgeVisualState::Damaged);
        assert_eq!(bridge_visual_state(0), BridgeVisualState::Destroyed);

        let vert =
            bridge_visual_tile_specs(BuildingType::BridgeVert, 2, BridgeVisualState::Damaged);
        assert_eq!(vert.len(), 7);
        assert_eq!(
            vert[2].index,
            bridge_vert_fill_index(BridgeVisualState::Damaged, 2)
        );
        assert_eq!(vert[2].frame_size, Vec2::new(64.0, 16.0));

        let horz =
            bridge_visual_tile_specs(BuildingType::BridgeHorz, 2, BridgeVisualState::Destroyed);
        assert_eq!(horz.len(), 7);
        assert_eq!(
            horz[2].index,
            bridge_horz_fill_index(BridgeVisualState::Destroyed, 2)
        );
        assert_eq!(horz[2].world_offset, Vec2::new(32.0, 0.0));
        assert!(
            bridge_visual_tile_specs(BuildingType::Radar, 0, BridgeVisualState::Live).is_empty()
        );
    }

    #[test]
    fn fort_enter_geometry_matches_original_rects_and_points() {
        let center = Vec2::new(80.0, -96.0);
        let front_size = Vec2::new(160.0, 192.0);
        assert_eq!(
            fort_entrance_rect(BuildingType::FortFront),
            Some((64.0, 32.0, 32.0, 96.0))
        );
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
        assert_eq!(
            fort_entry_points(center, front_size, BuildingType::FortFront),
            Some((Vec2::new(80.0, -128.0), Vec2::new(80.0, -208.0)))
        );
        assert_eq!(fort_entrance_rect(BuildingType::Radar), None);
    }

    #[test]
    fn bridge_world_rect_uses_original_tile_footprint() {
        let bridge = BridgeFootprint {
            x: 2,
            y: 3,
            building: BuildingType::BridgeHorz,
            extra_links: 2,
        };

        let rect = bridge_world_rect(bridge).expect("horizontal bridge footprint");

        assert_eq!(rect.min_x, 2.0 * TILE_SIZE);
        assert_eq!(rect.max_x, 9.0 * TILE_SIZE);
        assert_eq!(rect.min_y, -7.0 * TILE_SIZE);
        assert_eq!(rect.max_y, -3.0 * TILE_SIZE);
    }

    #[test]
    fn destroyed_bridge_kills_intersecting_robots_and_vehicles_only() {
        let bridge = BridgeFootprint {
            x: 1,
            y: 1,
            building: BuildingType::BridgeVert,
            extra_links: 0,
        };
        let inside_bridge = Vec2::new(3.0 * TILE_SIZE, -3.0 * TILE_SIZE);
        let outside_bridge = Vec2::new(7.0 * TILE_SIZE, -3.0 * TILE_SIZE);
        let size = Vec2::splat(12.0);
        let live_robot = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let live_vehicle = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);
        let live_cannon = ObjectStats::from_kind(ObjectKind::Cannon(CannonType::Gun), 100);
        let destroyed_robot = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 0);

        assert!(bridge_destroy_kills_unit(
            ObjectKind::Robot(RobotType::Grunt),
            live_robot,
            inside_bridge,
            size,
            bridge
        ));
        assert!(bridge_destroy_kills_unit(
            ObjectKind::Vehicle(VehicleType::Jeep),
            live_vehicle,
            inside_bridge,
            size,
            bridge
        ));
        assert!(!bridge_destroy_kills_unit(
            ObjectKind::Cannon(CannonType::Gun),
            live_cannon,
            inside_bridge,
            size,
            bridge
        ));
        assert!(!bridge_destroy_kills_unit(
            ObjectKind::Robot(RobotType::Grunt),
            live_robot,
            outside_bridge,
            size,
            bridge
        ));
        assert!(!bridge_destroy_kills_unit(
            ObjectKind::Robot(RobotType::Grunt),
            destroyed_robot,
            inside_bridge,
            size,
            bridge
        ));
    }

    #[test]
    fn auto_repair_policy_matches_original_non_fort_buildings() {
        assert!(auto_repairable_after_destroy(ObjectKind::Building(
            BuildingType::Radar
        )));
        assert!(auto_repairable_after_destroy(ObjectKind::Building(
            BuildingType::Repair
        )));
        assert!(auto_repairable_after_destroy(ObjectKind::Bridge(
            BuildingType::BridgeVert
        )));
        assert!(!auto_repairable_after_destroy(ObjectKind::Building(
            BuildingType::FortFront
        )));
        assert!(!auto_repairable_after_destroy(ObjectKind::Cannon(
            CannonType::Gun
        )));
    }

    #[test]
    fn auto_repair_delay_matches_original_default_range() {
        let mut rng = CombatRng::default();
        for _ in 0..32 {
            let delay = auto_repair_delay(&mut rng);
            assert!((600.0..=660.0).contains(&delay));
        }
    }

    #[test]
    fn auto_repair_zone_blocker_matches_original_fort_front_check() {
        assert!(auto_repair_blocking_fort(ObjectKind::Building(
            BuildingType::FortFront
        )));
        assert!(!auto_repair_blocking_fort(ObjectKind::Building(
            BuildingType::FortBack
        )));
        assert!(!auto_repair_blocking_fort(ObjectKind::Building(
            BuildingType::Radar
        )));
    }
}
