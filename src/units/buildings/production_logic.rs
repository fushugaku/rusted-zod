use std::collections::VecDeque;

use bevy::prelude::Vec2;

#[cfg(test)]
use crate::units::unit_settings;
use crate::{
    components::{
        BuildingProduction, BuildingProductionStatus, BuildingRallyPoints, MovementWaypoint,
        ObjectStats, ProductionLevel,
    },
    original::{
        objects::{BuildingType, ObjectKind},
        types::TeamType,
    },
    settings_sync::SourceSettingsState,
    units,
};

pub(crate) const MAX_QUEUE_ITEMS: usize = 5;
pub(crate) const MAX_STORED_CANNONS: usize = 4;
pub(crate) const DEFAULT_MAX_UNITS_PER_TEAM: usize = 70;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CannonZoneSnapshot {
    pub(crate) zone: Option<usize>,
    pub(crate) count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BuildingCreateUnitOutcome {
    SpawnObjects { unit: ObjectKind, count: u8 },
    StoredCannon,
    DroppedCannon,
}

#[cfg(test)]
pub(crate) fn initial_production_for_building(
    kind: ObjectKind,
    level: impl Into<ProductionLevel>,
    owner: TeamType,
    stats: ObjectStats,
) -> Option<BuildingProduction> {
    if owner == TeamType::Null {
        return None;
    }

    let unit = default_production_unit(kind, level)?;
    let mut production = BuildingProduction {
        status: BuildingProductionStatus::Select,
        current: None,
        queue: VecDeque::new(),
        elapsed: 0.0,
        duration: 0.0,
        zone_ownage: 0.0,
        unit_limit_reached: false,
        stored_cannons: Vec::new(),
    };

    start_production(&mut production, unit, stats).then_some(production)
}

pub(crate) fn initial_production_for_building_from_source(
    kind: ObjectKind,
    level: impl Into<ProductionLevel>,
    owner: TeamType,
    stats: ObjectStats,
    settings: &SourceSettingsState,
) -> Option<BuildingProduction> {
    if owner == TeamType::Null {
        return None;
    }

    let unit = default_production_unit(kind, level)?;
    let mut production = BuildingProduction {
        status: BuildingProductionStatus::Select,
        current: None,
        queue: VecDeque::new(),
        elapsed: 0.0,
        duration: 0.0,
        zone_ownage: 0.0,
        unit_limit_reached: false,
        stored_cannons: Vec::new(),
    };
    start_production_from_source(&mut production, unit, stats, settings).then_some(production)
}

#[cfg(test)]
pub(crate) fn start_production(
    production: &mut BuildingProduction,
    unit: ObjectKind,
    stats: ObjectStats,
) -> bool {
    if production.current == Some(unit) {
        return false;
    }

    let Some(duration) =
        production_duration(unit, stats.health, stats.max_health, production.zone_ownage)
    else {
        return false;
    };

    start_production_with_duration(production, unit, duration)
}

pub(crate) fn start_production_from_source(
    production: &mut BuildingProduction,
    unit: ObjectKind,
    stats: ObjectStats,
    settings: &SourceSettingsState,
) -> bool {
    if production.current == Some(unit) {
        return false;
    }
    let Some(duration) = production_duration_from_source(
        unit,
        stats.health,
        stats.max_health,
        production.zone_ownage,
        settings,
    ) else {
        return false;
    };
    start_production_with_duration(production, unit, duration)
}

fn start_production_with_duration(
    production: &mut BuildingProduction,
    unit: ObjectKind,
    duration: f32,
) -> bool {
    production.current = Some(unit);
    production.status = BuildingProductionStatus::Building;
    production.elapsed = 0.0;
    production.duration = duration;

    if production.queue.is_empty() {
        production.queue.push_front(unit);
    }

    true
}

pub(crate) fn stop_production(production: &mut BuildingProduction, clear_queue: bool) -> bool {
    let changed = production.current.is_some()
        || production.status != BuildingProductionStatus::Select
        || (clear_queue && !production.queue.is_empty());

    production.current = None;
    production.status = BuildingProductionStatus::Select;
    production.elapsed = 0.0;
    production.duration = 0.0;
    if clear_queue {
        production.queue.clear();
    }

    changed
}

pub(crate) fn add_production_queue(
    production: &mut BuildingProduction,
    building: BuildingType,
    level: impl Into<ProductionLevel>,
    unit: ObjectKind,
    push_to_front: bool,
) -> bool {
    if production.queue.len() >= MAX_QUEUE_ITEMS
        || !unit_in_default_build_list(building, level, unit)
    {
        return false;
    }

    if push_to_front {
        production.queue.push_front(unit);
    } else {
        production.queue.push_back(unit);
    }
    true
}

pub(crate) fn cancel_production_queue_item(
    production: &mut BuildingProduction,
    index: usize,
    unit: ObjectKind,
) -> bool {
    if production.queue.get(index).copied() != Some(unit) {
        return false;
    }

    production.queue.remove(index).is_some()
}

#[cfg(test)]
pub(crate) fn advance_production(
    production: &mut BuildingProduction,
    delta_secs: f32,
    stats: ObjectStats,
) -> Vec<ObjectKind> {
    if production.status != BuildingProductionStatus::Building || production.current.is_none() {
        return Vec::new();
    }

    production.elapsed += delta_secs.max(0.0);
    if production.duration <= 0.0 || production.elapsed < production.duration {
        return Vec::new();
    }

    let Some(unit) = production.current else {
        return Vec::new();
    };
    reset_production(production, stats);
    vec![unit]
}

#[cfg(test)]
pub(crate) fn advance_production_with_unit_limit(
    production: &mut BuildingProduction,
    delta_secs: f32,
    stats: ObjectStats,
    unit_limit_reached: bool,
) -> Vec<ObjectKind> {
    production.unit_limit_reached = unit_limit_reached;

    if !matches!(
        production.status,
        BuildingProductionStatus::Building | BuildingProductionStatus::Paused
    ) || production.current.is_none()
    {
        return Vec::new();
    }

    if unit_limit_reached {
        production.elapsed += delta_secs.max(0.0);
        if production.duration > 0.0 {
            production.elapsed = production.elapsed.min(production.duration);
        }
        return Vec::new();
    }

    advance_production(production, delta_secs, stats)
}

pub(crate) fn advance_production_with_unit_limit_from_source(
    production: &mut BuildingProduction,
    delta_secs: f32,
    stats: ObjectStats,
    unit_limit_reached: bool,
    settings: &SourceSettingsState,
) -> Vec<ObjectKind> {
    production.unit_limit_reached = unit_limit_reached;
    if !matches!(
        production.status,
        BuildingProductionStatus::Building | BuildingProductionStatus::Paused
    ) || production.current.is_none()
    {
        return Vec::new();
    }
    if unit_limit_reached {
        production.elapsed += delta_secs.max(0.0);
        if production.duration > 0.0 {
            production.elapsed = production.elapsed.min(production.duration);
        }
        return Vec::new();
    }

    production.elapsed += delta_secs.max(0.0);
    if production.duration <= 0.0 || production.elapsed < production.duration {
        return Vec::new();
    }
    let Some(unit) = production.current else {
        return Vec::new();
    };
    reset_production_from_source(production, stats, settings);
    vec![unit]
}

fn reset_production_from_source(
    production: &mut BuildingProduction,
    stats: ObjectStats,
    settings: &SourceSettingsState,
) -> bool {
    if let Some(next_unit) = production.queue.pop_front() {
        stop_production(production, false);
        start_production_from_source(production, next_unit, stats, settings)
    } else {
        stop_production(production, true)
    }
}

#[cfg(test)]
pub(crate) fn reset_production(production: &mut BuildingProduction, stats: ObjectStats) -> bool {
    if let Some(next_unit) = production.queue.pop_front() {
        stop_production(production, false);
        start_production(production, next_unit, stats)
    } else {
        stop_production(production, true)
    }
}

#[cfg(test)]
pub(crate) fn reset_build_time(
    production: &mut BuildingProduction,
    stats: ObjectStats,
    zone_ownage: f32,
) -> bool {
    let zone_ownage = zone_ownage.clamp(0.0, 1.0);
    let old_duration = production.duration;
    production.zone_ownage = zone_ownage;

    if production.status == BuildingProductionStatus::Select {
        return false;
    }

    let Some(unit) = production.current else {
        return false;
    };
    let Some(duration) =
        production_duration(unit, stats.health, stats.max_health, production.zone_ownage)
    else {
        return false;
    };

    production.duration = duration;
    production.duration != old_duration
}

pub(crate) fn reset_build_time_from_source(
    production: &mut BuildingProduction,
    stats: ObjectStats,
    zone_ownage: f32,
    settings: &SourceSettingsState,
) -> bool {
    let zone_ownage = zone_ownage.clamp(0.0, 1.0);
    let old_duration = production.duration;
    production.zone_ownage = zone_ownage;
    if production.status == BuildingProductionStatus::Select {
        return false;
    }
    let Some(unit) = production.current else {
        return false;
    };
    let Some(duration) = production_duration_from_source(
        unit,
        stats.health,
        stats.max_health,
        production.zone_ownage,
        settings,
    ) else {
        return false;
    };
    production.duration = duration;
    production.duration != old_duration
}

#[cfg(test)]
pub(crate) fn set_default_production(
    production: &mut BuildingProduction,
    kind: ObjectKind,
    level: impl Into<ProductionLevel>,
    stats: ObjectStats,
) -> bool {
    if production.current.is_some() || production.status != BuildingProductionStatus::Select {
        return false;
    }

    let Some(unit) = default_production_unit(kind, level) else {
        return false;
    };

    start_production(production, unit, stats)
}

pub(crate) fn set_default_production_from_source(
    production: &mut BuildingProduction,
    kind: ObjectKind,
    level: impl Into<ProductionLevel>,
    stats: ObjectStats,
    settings: &SourceSettingsState,
) -> bool {
    if production.current.is_some() || production.status != BuildingProductionStatus::Select {
        return false;
    }
    let Some(unit) = default_production_unit(kind, level) else {
        return false;
    };
    start_production_from_source(production, unit, stats, settings)
}

pub(crate) fn is_stored_cannon(unit: ObjectKind) -> bool {
    matches!(unit, ObjectKind::Cannon(_))
}

pub(crate) fn store_built_cannon(production: &mut BuildingProduction, unit: ObjectKind) -> bool {
    if !is_stored_cannon(unit) || production.stored_cannons.len() >= MAX_STORED_CANNONS {
        return false;
    }

    production.stored_cannons.push(unit);
    production.status = BuildingProductionStatus::Place;
    true
}

pub(crate) fn can_store_cannon_in_zone(stored_or_placed_count: usize) -> bool {
    stored_or_placed_count < MAX_STORED_CANNONS
}

pub(crate) fn cannon_count_in_zone(snapshots: &[CannonZoneSnapshot], zone: Option<usize>) -> usize {
    snapshots
        .iter()
        .filter(|snapshot| snapshot.zone == zone)
        .map(|snapshot| snapshot.count)
        .sum()
}

pub(crate) fn store_built_cannon_in_zone(
    production: &mut BuildingProduction,
    unit: ObjectKind,
    zone: Option<usize>,
    snapshots: &mut Vec<CannonZoneSnapshot>,
) -> bool {
    if !is_stored_cannon(unit) || !can_store_cannon_in_zone(cannon_count_in_zone(snapshots, zone)) {
        return false;
    }

    if !store_built_cannon(production, unit) {
        return false;
    }

    snapshots.push(CannonZoneSnapshot { zone, count: 1 });
    true
}

#[cfg(test)]
pub(crate) fn building_create_unit_outcome(
    production: &mut BuildingProduction,
    unit: ObjectKind,
    zone: Option<usize>,
    snapshots: &mut Vec<CannonZoneSnapshot>,
) -> BuildingCreateUnitOutcome {
    if is_stored_cannon(unit) {
        return if store_built_cannon_in_zone(production, unit, zone, snapshots) {
            BuildingCreateUnitOutcome::StoredCannon
        } else {
            BuildingCreateUnitOutcome::DroppedCannon
        };
    }

    BuildingCreateUnitOutcome::SpawnObjects {
        unit,
        count: produced_object_count(unit),
    }
}

pub(crate) fn building_create_unit_outcome_from_source(
    production: &mut BuildingProduction,
    unit: ObjectKind,
    zone: Option<usize>,
    snapshots: &mut Vec<CannonZoneSnapshot>,
    settings: &SourceSettingsState,
) -> BuildingCreateUnitOutcome {
    if is_stored_cannon(unit) {
        return if store_built_cannon_in_zone(production, unit, zone, snapshots) {
            BuildingCreateUnitOutcome::StoredCannon
        } else {
            BuildingCreateUnitOutcome::DroppedCannon
        };
    }
    BuildingCreateUnitOutcome::SpawnObjects {
        unit,
        count: settings.produced_object_count(unit),
    }
}

pub(crate) fn apply_building_rally_point(
    rally_points: &mut BuildingRallyPoints,
    point: Vec2,
    append: bool,
) {
    if !append {
        rally_points.points.clear();
    }
    rally_points.points.push(point);
}

pub(crate) fn can_set_rallypoints(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Building(
            BuildingType::FortFront
                | BuildingType::FortBack
                | BuildingType::RobotFactory
                | BuildingType::VehicleFactory
        )
    )
}

#[cfg(test)]
pub(crate) fn production_spawn_route(
    move_target: Vec2,
    rally_points: Option<&BuildingRallyPoints>,
) -> Vec<Vec2> {
    production_spawn_waypoints(move_target, rally_points)
        .into_iter()
        .map(|waypoint| waypoint.position)
        .collect()
}

pub(crate) fn production_spawn_waypoints(
    move_target: Vec2,
    rally_points: Option<&BuildingRallyPoints>,
) -> Vec<MovementWaypoint> {
    let mut route = vec![MovementWaypoint::force_move(move_target)];
    if let Some(rally_points) = rally_points {
        route.extend(
            rally_points
                .points
                .iter()
                .copied()
                .map(MovementWaypoint::rally_move),
        );
    }
    route
}

#[cfg(test)]
pub(crate) fn production_spawn_route_for_member(
    move_target: Vec2,
    rally_points: Option<&BuildingRallyPoints>,
    is_group_leader: bool,
) -> Vec<Vec2> {
    production_spawn_waypoints_for_member(move_target, rally_points, is_group_leader)
        .into_iter()
        .map(|waypoint| waypoint.position)
        .collect()
}

pub(crate) fn production_spawn_waypoints_for_member(
    move_target: Vec2,
    rally_points: Option<&BuildingRallyPoints>,
    is_group_leader: bool,
) -> Vec<MovementWaypoint> {
    if is_group_leader {
        production_spawn_waypoints(move_target, rally_points)
    } else {
        vec![MovementWaypoint::force_move(move_target)]
    }
}

pub(crate) fn team_unit_limit_reached(unit_count: usize) -> bool {
    unit_count >= DEFAULT_MAX_UNITS_PER_TEAM
}

#[cfg(test)]
pub(crate) fn remove_stored_cannon(production: &mut BuildingProduction, unit: ObjectKind) -> bool {
    let Some(index) = production
        .stored_cannons
        .iter()
        .position(|stored| *stored == unit)
    else {
        return false;
    };

    production.stored_cannons.remove(index);
    if production.stored_cannons.is_empty() && production.status == BuildingProductionStatus::Place
    {
        production.status = if production.current.is_some() {
            BuildingProductionStatus::Building
        } else {
            BuildingProductionStatus::Select
        };
    }
    true
}

pub(crate) fn apply_stored_cannon_list(
    production: &mut BuildingProduction,
    stored_cannons: Vec<ObjectKind>,
) {
    production.stored_cannons = stored_cannons;
    if production.stored_cannons.is_empty() && production.status == BuildingProductionStatus::Place
    {
        production.status = if production.current.is_some() {
            BuildingProductionStatus::Building
        } else {
            BuildingProductionStatus::Select
        };
    }
}

pub(crate) fn default_production_unit(
    kind: ObjectKind,
    level: impl Into<ProductionLevel>,
) -> Option<ObjectKind> {
    let ObjectKind::Building(building) = kind else {
        return None;
    };

    super::default_production_unit(building, level)
}

pub(crate) fn unit_in_default_build_list(
    building: BuildingType,
    level: impl Into<ProductionLevel>,
    unit: ObjectKind,
) -> bool {
    super::unit_in_default_build_list(building, level, unit)
}

#[cfg(test)]
pub(crate) fn production_duration(
    unit: ObjectKind,
    health: f32,
    max_health: f32,
    zone_ownage: f32,
) -> Option<f32> {
    let base = unit_settings(unit)?.build_time;
    Some(production_duration_from_base(
        base,
        health,
        max_health,
        zone_ownage,
    ))
}

pub(crate) fn production_duration_from_source(
    unit: ObjectKind,
    health: f32,
    max_health: f32,
    zone_ownage: f32,
    settings: &SourceSettingsState,
) -> Option<f32> {
    let base = settings.unit_settings(unit)?.build_time;
    Some(production_duration_from_base(
        base,
        health,
        max_health,
        zone_ownage,
    ))
}

fn production_duration_from_base(base: f32, health: f32, max_health: f32, zone_ownage: f32) -> f32 {
    let health_ratio = if max_health <= 0.0 {
        0.0
    } else {
        (health / max_health).clamp(0.0, 1.0)
    };
    let zone_ownage = zone_ownage.clamp(0.0, 1.0);

    base - base * 0.5 * zone_ownage + base * (1.25 * (1.0 - health_ratio))
}

#[cfg(test)]
pub(crate) fn produced_object_count(unit: ObjectKind) -> u8 {
    units::produced_object_count(unit)
}

#[cfg(test)]
pub(crate) fn initial_spawn_count(kind: ObjectKind) -> u32 {
    match kind {
        ObjectKind::Robot(_) => u32::from(produced_object_count(kind)),
        _ => 1,
    }
}

pub(crate) fn production_world_points(
    building: BuildingType,
    tile_x: u16,
    tile_y: u16,
) -> Option<(Vec2, Vec2)> {
    super::production_world_points(building, tile_x, tile_y)
}

pub(crate) fn production_source_map_points(
    unit: ObjectKind,
    create_center_world: Vec2,
    move_target_world: Vec2,
) -> Option<(Vec2, Vec2)> {
    let object_size = units::source_mobile_dimensions(unit)?;
    Some((
        Vec2::new(create_center_world.x, -create_center_world.y) - object_size * 0.5,
        Vec2::new(move_target_world.x, -move_target_world.y),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::MovementWaypointMode;
    use crate::original::objects::{CannonType, RobotType, VehicleType};

    fn test_production() -> BuildingProduction {
        BuildingProduction {
            status: BuildingProductionStatus::Building,
            current: Some(ObjectKind::Cannon(CannonType::Gatling)),
            queue: VecDeque::new(),
            elapsed: 0.0,
            duration: 1.0,
            zone_ownage: 0.0,
            unit_limit_reached: false,
            stored_cannons: Vec::new(),
        }
    }

    #[test]
    fn cannon_zone_count_matches_original_stored_and_placed_sum() {
        let snapshots = vec![
            CannonZoneSnapshot {
                zone: Some(1),
                count: 1,
            },
            CannonZoneSnapshot {
                zone: Some(1),
                count: 2,
            },
            CannonZoneSnapshot {
                zone: Some(2),
                count: 4,
            },
        ];

        assert_eq!(cannon_count_in_zone(&snapshots, Some(1)), 3);
        assert_eq!(cannon_count_in_zone(&snapshots, Some(2)), 4);
        assert_eq!(cannon_count_in_zone(&snapshots, Some(3)), 0);
    }

    #[test]
    fn built_cannon_store_updates_zone_snapshot_like_original_relay_state() {
        let mut production = test_production();
        let mut snapshots = vec![CannonZoneSnapshot {
            zone: Some(1),
            count: MAX_STORED_CANNONS - 1,
        }];

        assert!(store_built_cannon_in_zone(
            &mut production,
            ObjectKind::Cannon(CannonType::Gatling),
            Some(2),
            &mut snapshots,
        ));
        assert_eq!(
            production.stored_cannons,
            vec![ObjectKind::Cannon(CannonType::Gatling)]
        );
        assert_eq!(production.status, BuildingProductionStatus::Place);
        assert_eq!(cannon_count_in_zone(&snapshots, Some(2)), 1);
    }

    #[test]
    fn built_cannon_is_dropped_when_zone_capacity_is_full_like_original_server_path() {
        let mut production = test_production();
        let mut snapshots = vec![CannonZoneSnapshot {
            zone: Some(1),
            count: MAX_STORED_CANNONS,
        }];

        assert!(!store_built_cannon_in_zone(
            &mut production,
            ObjectKind::Cannon(CannonType::Gun),
            Some(1),
            &mut snapshots,
        ));
        assert!(production.stored_cannons.is_empty());
        assert_eq!(production.status, BuildingProductionStatus::Building);
        assert_eq!(snapshots.len(), 1);
    }

    #[test]
    fn building_create_unit_outcome_spawns_non_cannons_like_original_server_path() {
        let mut production = test_production();
        let mut snapshots = Vec::new();

        assert_eq!(
            building_create_unit_outcome(
                &mut production,
                ObjectKind::Robot(crate::original::objects::RobotType::Grunt),
                Some(1),
                &mut snapshots,
            ),
            BuildingCreateUnitOutcome::SpawnObjects {
                unit: ObjectKind::Robot(crate::original::objects::RobotType::Grunt),
                count: 3,
            }
        );
        assert!(production.stored_cannons.is_empty());
        assert!(snapshots.is_empty());
    }

    #[test]
    fn building_create_unit_outcome_reports_stored_and_dropped_cannons() {
        let mut stored = test_production();
        let mut snapshots = Vec::new();
        assert_eq!(
            building_create_unit_outcome(
                &mut stored,
                ObjectKind::Cannon(CannonType::Gun),
                Some(1),
                &mut snapshots,
            ),
            BuildingCreateUnitOutcome::StoredCannon
        );
        assert_eq!(
            stored.stored_cannons,
            vec![ObjectKind::Cannon(CannonType::Gun)]
        );

        let mut dropped = test_production();
        let mut full_snapshots = vec![CannonZoneSnapshot {
            zone: Some(1),
            count: MAX_STORED_CANNONS,
        }];
        assert_eq!(
            building_create_unit_outcome(
                &mut dropped,
                ObjectKind::Cannon(CannonType::Howitzer),
                Some(1),
                &mut full_snapshots,
            ),
            BuildingCreateUnitOutcome::DroppedCannon
        );
        assert!(dropped.stored_cannons.is_empty());
    }

    #[test]
    fn rally_point_command_replaces_or_appends_like_original_shift_send() {
        let mut rally_points = BuildingRallyPoints::default();

        apply_building_rally_point(&mut rally_points, Vec2::new(10.0, -20.0), false);
        assert_eq!(rally_points.points, vec![Vec2::new(10.0, -20.0)]);

        apply_building_rally_point(&mut rally_points, Vec2::new(30.0, -40.0), true);
        assert_eq!(
            rally_points.points,
            vec![Vec2::new(10.0, -20.0), Vec2::new(30.0, -40.0)]
        );

        apply_building_rally_point(&mut rally_points, Vec2::new(50.0, -60.0), false);
        assert_eq!(rally_points.points, vec![Vec2::new(50.0, -60.0)]);
    }

    #[test]
    fn can_set_rallypoints_matches_source_building_overrides() {
        assert!(can_set_rallypoints(ObjectKind::Building(
            BuildingType::FortFront
        )));
        assert!(can_set_rallypoints(ObjectKind::Building(
            BuildingType::FortBack
        )));
        assert!(can_set_rallypoints(ObjectKind::Building(
            BuildingType::RobotFactory
        )));
        assert!(can_set_rallypoints(ObjectKind::Building(
            BuildingType::VehicleFactory
        )));
        assert!(!can_set_rallypoints(ObjectKind::Building(
            BuildingType::Repair
        )));
        assert!(!can_set_rallypoints(ObjectKind::Building(
            BuildingType::Radar
        )));
        assert!(!can_set_rallypoints(ObjectKind::Vehicle(
            VehicleType::Light
        )));
    }

    #[test]
    fn production_spawn_route_appends_rally_points_after_factory_exit() {
        let mut rally_points = BuildingRallyPoints::default();
        rally_points.points = vec![Vec2::new(32.0, -64.0), Vec2::new(96.0, -128.0)];

        assert_eq!(
            production_spawn_route(Vec2::new(16.0, -16.0), Some(&rally_points)),
            vec![
                Vec2::new(16.0, -16.0),
                Vec2::new(32.0, -64.0),
                Vec2::new(96.0, -128.0),
            ]
        );
        assert_eq!(
            production_spawn_route(Vec2::new(16.0, -16.0), None),
            vec![Vec2::new(16.0, -16.0)]
        );
    }

    #[test]
    fn production_spawn_route_for_minion_keeps_only_initial_clone_waypoint() {
        let mut rally_points = BuildingRallyPoints::default();
        rally_points.points = vec![Vec2::new(32.0, -64.0)];

        assert_eq!(
            production_spawn_route_for_member(Vec2::new(16.0, -16.0), Some(&rally_points), true),
            vec![Vec2::new(16.0, -16.0), Vec2::new(32.0, -64.0)]
        );
        assert_eq!(
            production_spawn_route_for_member(Vec2::new(16.0, -16.0), Some(&rally_points), false),
            vec![Vec2::new(16.0, -16.0)]
        );
    }

    #[test]
    fn production_spawn_waypoints_preserve_source_modes_and_flags() {
        let mut rally_points = BuildingRallyPoints::default();
        rally_points.points = vec![Vec2::new(32.0, -64.0), Vec2::new(96.0, -128.0)];

        let route = production_spawn_waypoints(Vec2::new(16.0, -16.0), Some(&rally_points));

        assert_eq!(route.len(), 3);
        assert_eq!(route[0].position, Vec2::new(16.0, -16.0));
        assert_eq!(route[0].mode, MovementWaypointMode::ForceMove);
        assert_eq!(route[0].ref_id, None);
        assert!(!route[0].attack_to);
        assert!(!route[0].player_given);

        assert_eq!(route[1].position, Vec2::new(32.0, -64.0));
        assert_eq!(route[1].mode, MovementWaypointMode::Move);
        assert_eq!(route[1].ref_id, None);
        assert!(route[1].attack_to);
        assert!(route[1].player_given);

        assert_eq!(route[2].position, Vec2::new(96.0, -128.0));
        assert_eq!(route[2].mode, MovementWaypointMode::Move);
        assert_eq!(route[2].ref_id, None);
        assert!(route[2].attack_to);
        assert!(route[2].player_given);
    }

    #[test]
    fn production_spawn_waypoints_for_minion_keeps_force_exit_only() {
        let mut rally_points = BuildingRallyPoints::default();
        rally_points.points = vec![Vec2::new(32.0, -64.0)];

        let route = production_spawn_waypoints_for_member(
            Vec2::new(16.0, -16.0),
            Some(&rally_points),
            false,
        );

        assert_eq!(route.len(), 1);
        assert_eq!(route[0].position, Vec2::new(16.0, -16.0));
        assert_eq!(route[0].mode, MovementWaypointMode::ForceMove);
        assert_eq!(route[0].ref_id, None);
        assert!(!route[0].attack_to);
        assert!(!route[0].player_given);
    }

    #[test]
    fn production_source_points_use_original_mobile_dimensions_and_map_axis() {
        assert_eq!(
            production_source_map_points(
                ObjectKind::Robot(RobotType::Grunt),
                Vec2::new(100.0, -200.0),
                Vec2::new(120.0, -240.0),
            ),
            Some((Vec2::new(92.0, 192.0), Vec2::new(120.0, 240.0)))
        );
        assert_eq!(
            production_source_map_points(
                ObjectKind::Vehicle(VehicleType::Light),
                Vec2::new(100.0, -200.0),
                Vec2::new(120.0, -240.0),
            ),
            Some((Vec2::new(84.0, 184.0), Vec2::new(120.0, 240.0)))
        );
        assert_eq!(
            production_source_map_points(
                ObjectKind::Building(BuildingType::Radar),
                Vec2::ZERO,
                Vec2::ZERO,
            ),
            None
        );
    }
}
