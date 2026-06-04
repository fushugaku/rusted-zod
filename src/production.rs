use std::collections::VecDeque;

use bevy::prelude::Vec2;

use crate::{
    components::{BuildingProduction, BuildingProductionStatus, ObjectStats, ProductionLevel},
    constants::TILE_SIZE,
    original::{
        objects::{BuildingType, ObjectKind},
        settings::unit_settings,
        types::TeamType,
    },
    units,
};

pub(crate) const MAX_QUEUE_ITEMS: usize = 5;
pub(crate) const MAX_STORED_CANNONS: usize = 4;
pub(crate) const DEFAULT_MAX_UNITS_PER_TEAM: usize = 70;

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
        ready_units: Vec::new(),
        stored_cannons: Vec::new(),
    };

    start_production(&mut production, unit, stats).then_some(production)
}

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

#[allow(dead_code)]
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

pub(crate) fn advance_production(
    production: &mut BuildingProduction,
    delta_secs: f32,
    stats: ObjectStats,
) -> Vec<ObjectKind> {
    if production.status != BuildingProductionStatus::Building || production.current.is_none() {
        return Vec::new();
    }

    production.elapsed += delta_secs.max(0.0);
    let mut completed = Vec::new();

    while production.duration > 0.0 && production.elapsed >= production.duration {
        let Some(unit) = production.current else {
            break;
        };
        production.elapsed -= production.duration;
        if !is_stored_cannon(unit) {
            production.ready_units.push(unit);
        }
        completed.push(unit);
        reset_production(production, stats);
    }

    completed
}

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

pub(crate) fn reset_production(production: &mut BuildingProduction, stats: ObjectStats) -> bool {
    if let Some(next_unit) = production.queue.pop_front() {
        stop_production(production, false);
        start_production(production, next_unit, stats)
    } else {
        stop_production(production, true)
    }
}

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

pub(crate) fn team_unit_limit_reached(unit_count: usize) -> bool {
    unit_count >= DEFAULT_MAX_UNITS_PER_TEAM
}

#[allow(dead_code)]
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

pub(crate) fn default_production_unit(
    kind: ObjectKind,
    level: impl Into<ProductionLevel>,
) -> Option<ObjectKind> {
    let ObjectKind::Building(building) = kind else {
        return None;
    };

    units::buildings::default_production_unit(building, level)
}

pub(crate) fn unit_in_default_build_list(
    building: BuildingType,
    level: impl Into<ProductionLevel>,
    unit: ObjectKind,
) -> bool {
    units::buildings::unit_in_default_build_list(building, level, unit)
}

pub(crate) fn default_build_list(
    building: BuildingType,
    level: impl Into<ProductionLevel>,
) -> Vec<ObjectKind> {
    units::buildings::default_build_list(building, level)
}

pub(crate) fn production_duration(
    unit: ObjectKind,
    health: f32,
    max_health: f32,
    zone_ownage: f32,
) -> Option<f32> {
    let base = unit_settings(unit)?.build_time;
    let health_ratio = if max_health <= 0.0 {
        0.0
    } else {
        (health / max_health).clamp(0.0, 1.0)
    };
    let zone_ownage = zone_ownage.clamp(0.0, 1.0);

    Some(base - base * 0.5 * zone_ownage + base * (1.25 * (1.0 - health_ratio)))
}

pub(crate) fn produced_object_count(unit: ObjectKind) -> u8 {
    units::produced_object_count(unit)
}

pub(crate) fn initial_spawn_count(kind: ObjectKind) -> u32 {
    match kind {
        ObjectKind::Robot(_) => u32::from(produced_object_count(kind)),
        _ => 1,
    }
}

pub(crate) use units::buildings::ProductionPlacement;

pub(crate) fn production_placement(building: BuildingType) -> Option<ProductionPlacement> {
    units::buildings::production_placement(building)
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
