use bevy::prelude::Vec2;

use crate::{
    components::{MovementWaypoint, ObjectStats},
    original::{
        objects::{BuildingType, ObjectKind},
        types::TeamType,
    },
};

pub(crate) const REPAIR_BUILDING_SECONDS: f32 = 5.0;

pub(crate) fn health_ratio() -> f32 {
    2000.0 / 240.0
}

pub(crate) fn can_repair_unit(
    kind: ObjectKind,
    building_team: TeamType,
    unit_team: TeamType,
    stats: ObjectStats,
) -> bool {
    matches!(kind, ObjectKind::Building(BuildingType::Repair))
        && building_team != TeamType::Null
        && building_team == unit_team
        && !stats.destroyed()
}

pub(crate) fn can_repair_target_unit(kind: ObjectKind, team: TeamType, stats: ObjectStats) -> bool {
    matches!(kind, ObjectKind::Vehicle(_))
        && team != TeamType::Null
        && !stats.destroyed()
        && stats.health < stats.max_health
}

pub(crate) fn repaired_unit_source_points(
    unit: ObjectKind,
    repair_center_world: Vec2,
    repair_entrance_world: Vec2,
) -> Option<(Vec2, Vec2)> {
    let object_size = crate::units::source_mobile_dimensions(unit)?;
    Some((
        Vec2::new(repair_center_world.x, -repair_center_world.y) - object_size * 0.5,
        Vec2::new(repair_entrance_world.x, -repair_entrance_world.y),
    ))
}

pub(crate) fn repaired_unit_waypoints(
    repair_entrance_world: Vec2,
    resume_waypoints: &[MovementWaypoint],
) -> Vec<MovementWaypoint> {
    let mut waypoints = Vec::with_capacity(1 + resume_waypoints.len());
    waypoints.push(MovementWaypoint::force_move(repair_entrance_world));
    waypoints.extend_from_slice(resume_waypoints);
    waypoints
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{RobotType, VehicleType};

    #[test]
    fn repair_building_accepts_same_team_damaged_vehicle() {
        let building_stats =
            ObjectStats::from_kind(ObjectKind::Building(BuildingType::Repair), 100);
        let vehicle_stats = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Light), 50);

        assert!(can_repair_unit(
            ObjectKind::Building(BuildingType::Repair),
            TeamType::Red,
            TeamType::Red,
            building_stats
        ));
        assert!(can_repair_target_unit(
            ObjectKind::Vehicle(VehicleType::Light),
            TeamType::Red,
            vehicle_stats
        ));
        assert!(!can_repair_unit(
            ObjectKind::Building(BuildingType::Repair),
            TeamType::Blue,
            TeamType::Red,
            building_stats
        ));
    }

    #[test]
    fn repaired_unit_points_and_route_match_building_repair_unit() {
        assert_eq!(
            repaired_unit_source_points(
                ObjectKind::Vehicle(VehicleType::Light),
                Vec2::new(100.0, -200.0),
                Vec2::new(120.0, -240.0),
            ),
            Some((Vec2::new(84.0, 184.0), Vec2::new(120.0, 240.0)))
        );
        assert_eq!(
            repaired_unit_source_points(
                ObjectKind::Robot(RobotType::Grunt),
                Vec2::new(100.0, -200.0),
                Vec2::new(120.0, -240.0),
            ),
            Some((Vec2::new(92.0, 192.0), Vec2::new(120.0, 240.0)))
        );

        let tail = MovementWaypoint::player_move_to(Vec2::new(180.0, -260.0), true);
        let route = repaired_unit_waypoints(Vec2::new(120.0, -240.0), &[tail]);
        assert_eq!(route.len(), 2);
        assert_eq!(
            route[0],
            MovementWaypoint::force_move(Vec2::new(120.0, -240.0))
        );
        assert_eq!(route[1], tail);
    }
}
