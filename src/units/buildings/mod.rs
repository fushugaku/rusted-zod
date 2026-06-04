use bevy::prelude::Vec2;

use crate::{
    components::{
        BridgeFootprint, CombatRng, MapGridPosition, ObjectStats, PassabilityGrid, ProductionLevel,
    },
    constants::TILE_SIZE,
    original::objects::{BuildingType, ObjectKind},
    original::types::{PlanetType, TeamType},
};

pub(crate) mod bridge_horz;
pub(crate) mod bridge_horz_ui;
pub(crate) mod bridge_vert;
pub(crate) mod bridge_vert_ui;
pub(crate) mod fort_back;
pub(crate) mod fort_back_ui;
pub(crate) mod fort_front;
pub(crate) mod fort_front_ui;
pub(crate) mod radar;
pub(crate) mod radar_ui;
pub(crate) mod repair;
pub(crate) mod repair_ui;
pub(crate) mod robot_factory;
pub(crate) mod robot_factory_ui;
pub(crate) mod vehicle_factory;
pub(crate) mod vehicle_factory_ui;

const AUTO_REPAIR_TIME: f32 = 10.0 * 60.0;
const AUTO_REPAIR_RANDOM_ADDITIONAL_TIME: usize = 61;

pub(crate) const BUILDING_TURRENT_FRAME_TIME: f32 = 0.1;
pub(crate) const BRIDGE_TURRENT_FRAME_TIME: f32 = 0.07;
pub(crate) const BRIDGE_TURRENT_MAX_DISTANCE: f32 = 140.0;
pub(crate) const BRIDGE_ROCK_PARTICLE_FRAME_TIME: f32 = 0.07;
pub(crate) const BRIDGE_REVIVE_RERENDER_DELAY: f32 = 2.25;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ProductionPlacement {
    pub(crate) create_offset: Vec2,
    pub(crate) move_offset: Vec2,
}

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

pub(crate) fn default_selection_size(building: BuildingType) -> Vec2 {
    match building {
        BuildingType::FortFront => fort_front::default_selection_size(),
        BuildingType::FortBack => fort_back::default_selection_size(),
        BuildingType::Radar => radar::default_selection_size(),
        BuildingType::Repair => repair::default_selection_size(),
        BuildingType::RobotFactory => robot_factory::default_selection_size(),
        BuildingType::VehicleFactory => vehicle_factory::default_selection_size(),
        BuildingType::BridgeVert => bridge_vert::default_selection_size(),
        BuildingType::BridgeHorz => bridge_horz::default_selection_size(),
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

pub(crate) fn production_placement(building: BuildingType) -> Option<ProductionPlacement> {
    match building {
        BuildingType::FortFront => Some(fort_front::production_placement()),
        BuildingType::FortBack => Some(fort_back::production_placement()),
        BuildingType::RobotFactory => Some(robot_factory::production_placement()),
        BuildingType::VehicleFactory => Some(vehicle_factory::production_placement()),
        BuildingType::Radar
        | BuildingType::Repair
        | BuildingType::BridgeVert
        | BuildingType::BridgeHorz => None,
    }
}

pub(crate) fn death_profile(kind: ObjectKind, planet: PlanetType) -> Option<BuildingDeathProfile> {
    let ObjectKind::Building(building) = kind else {
        return None;
    };

    match building {
        BuildingType::FortFront => Some(fort_front::death_profile(planet)),
        BuildingType::FortBack => Some(fort_back::death_profile(planet)),
        BuildingType::Radar => Some(radar::death_profile()),
        BuildingType::Repair => Some(repair::death_profile()),
        BuildingType::RobotFactory => Some(robot_factory::death_profile()),
        BuildingType::VehicleFactory => Some(vehicle_factory::death_profile()),
        BuildingType::BridgeVert | BuildingType::BridgeHorz => None,
    }
}

pub(crate) fn destroyed_asset_path(building: BuildingType, planet: PlanetType) -> Option<String> {
    match building {
        BuildingType::FortFront => Some(fort_front::destroyed_asset_path(planet)),
        BuildingType::FortBack => Some(fort_back::destroyed_asset_path(planet)),
        BuildingType::Radar => Some(radar::destroyed_asset_path(planet)),
        BuildingType::Repair => Some(repair::destroyed_asset_path(planet)),
        BuildingType::RobotFactory => Some(robot_factory::destroyed_asset_path(planet)),
        BuildingType::VehicleFactory => Some(vehicle_factory::destroyed_asset_path(planet)),
        BuildingType::BridgeVert | BuildingType::BridgeHorz => None,
    }
}

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
    match bridge.building {
        BuildingType::BridgeVert => bridge_vert::turrent_spawn_points(bridge, rng),
        BuildingType::BridgeHorz => bridge_horz::turrent_spawn_points(bridge, rng),
        _ => Vec::new(),
    }
}

pub(crate) fn bridge_turrent_frame_paths(planet: PlanetType) -> Vec<String> {
    match planet {
        PlanetType::Desert
        | PlanetType::Volcanic
        | PlanetType::Arctic
        | PlanetType::Jungle
        | PlanetType::City => bridge_vert::turrent_frame_paths(planet),
    }
}

pub(crate) fn bridge_rock_particle_frame_paths(planet: PlanetType) -> Vec<String> {
    match planet {
        PlanetType::Desert
        | PlanetType::Volcanic
        | PlanetType::Arctic
        | PlanetType::Jungle
        | PlanetType::City => bridge_vert::rock_particle_frame_paths(planet),
    }
}

pub(crate) fn bridge_turrent_trajectory(
    anchor_map: Vec2,
    reversed: bool,
    rng: &mut CombatRng,
) -> EffectTrajectory {
    bridge_vert::turrent_trajectory(anchor_map, reversed, rng)
}

pub(crate) fn bridge_rock_particle_trajectory(
    anchor_map: Vec2,
    rng: &mut CombatRng,
) -> EffectTrajectory {
    bridge_vert::rock_particle_trajectory(anchor_map, rng)
}

pub(crate) fn bridge_end_particle_count(rng: &mut CombatRng) -> usize {
    bridge_vert::end_particle_count(rng)
}

pub(crate) fn bridge_turrent_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    arc_size(rise, final_time, t)
}

pub(crate) fn bridge_rock_particle_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    arc_size(rise, final_time, t)
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
    radar::overlay_should_be_visible(kind, owner, stats)
}

pub(crate) fn repair_overlay_should_be_visible(
    kind: crate::render::atlas::RepairOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    repairing_unit: bool,
) -> bool {
    repair::overlay_should_be_visible(kind, owner, stats, repairing_unit)
}

pub(crate) fn repair_overlay_forced_frame(
    kind: crate::render::atlas::RepairOverlayKind,
    owner: TeamType,
    repairing_unit: bool,
) -> Option<usize> {
    repair::overlay_forced_frame(kind, owner, repairing_unit)
}

pub(crate) fn factory_overlay_should_be_visible(
    kind: crate::render::atlas::FactoryOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    production_active: bool,
    current: usize,
) -> bool {
    if robot_factory::overlay_kind_is_robot(kind) {
        return robot_factory::overlay_should_be_visible(
            kind,
            owner,
            stats,
            production_active,
            current,
        );
    }

    vehicle_factory::overlay_should_be_visible(kind, owner, stats, production_active)
}

pub(crate) fn factory_overlay_next_frame(
    ref_id: u32,
    kind: crate::render::atlas::FactoryOverlayKind,
    current: usize,
    frame_count: usize,
    rng: &mut CombatRng,
    robot_light_updates: &mut std::collections::HashMap<u32, Option<[usize; 3]>>,
) -> usize {
    robot_factory::overlay_next_frame(ref_id, kind, current, frame_count, rng, robot_light_updates)
}

pub(crate) fn factory_overlay_is_robot_single_light(
    kind: crate::render::atlas::FactoryOverlayKind,
) -> bool {
    robot_factory::overlay_is_single_light(kind)
}

pub(crate) fn factory_overlay_robot_single_light_index(
    kind: crate::render::atlas::FactoryOverlayKind,
) -> Option<usize> {
    robot_factory::overlay_single_light_index(kind)
}

pub(crate) fn particle_trajectory(
    anchor_map: Vec2,
    max_horz: f32,
    max_vert: f32,
    lifetime_base: f32,
    lifetime_random: usize,
    rise_base: f32,
    rise_random: usize,
    rng: &mut CombatRng,
) -> EffectTrajectory {
    let particle = anchor_map + Vec2::new(rng.index(8) as f32, rng.index(24) as f32);
    let start = particle - Vec2::new(8.0, 5.0);
    let end = particle
        + Vec2::new(
            max_horz - rng.index((max_horz * 2.0) as usize) as f32,
            max_vert - rng.index((max_vert * 2.0) as usize) as f32,
        );

    EffectTrajectory {
        start,
        end,
        final_time: lifetime_base + rng.index(lifetime_random) as f32 * 0.1,
        rise: rise_base + rng.index(rise_random) as f32 * 0.01,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{CannonType, RobotType, VehicleType};

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
