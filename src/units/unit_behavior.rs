use bevy::prelude::Vec2;

use crate::{
    components::ObjectStats,
    original::{
        objects::{BuildingType, ItemType, ObjectKind, VehicleType},
        types::TeamType,
    },
};

use super::{attack::can_attack_target_identity, buildings::fort_entry_points};
use crate::units::items::grenades::can_pickup_grenades;

pub(crate) const AGRO_DISTANCE: f32 = 40.0;
pub(crate) const AUTO_GRAB_VEHICLE_DISTANCE: f32 = 220.0;
pub(crate) const RUN_UNIT_SPEED: f32 = 1.8;
pub(crate) const RUN_RECHARGE_RATE: f32 = 0.3;

#[derive(Clone, Copy)]
pub(crate) struct PassiveCombatTargetSnapshot {
    pub(crate) ref_id: u32,
    pub(crate) kind: ObjectKind,
    pub(crate) position: Vec2,
    pub(crate) size: Vec2,
    pub(crate) team: TeamType,
    pub(crate) stats: ObjectStats,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PassiveAutoEnterRobotSnapshot {
    pub(crate) ref_id: u32,
    pub(crate) position: Vec2,
    pub(crate) has_waypoint: bool,
    pub(crate) has_attack_target: bool,
    pub(crate) has_task_target: bool,
    pub(crate) is_minion: bool,
    pub(crate) just_left_cannon: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PassiveAutoEnterTargetSnapshot {
    pub(crate) ref_id: u32,
    pub(crate) kind: ObjectKind,
    pub(crate) position: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PassiveEnterFortTargetSnapshot {
    pub(crate) ref_id: u32,
    pub(crate) position: Vec2,
    pub(crate) team: TeamType,
    pub(crate) inside_point: Vec2,
    pub(crate) exit_point: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PassiveGrenadeBoxSnapshot {
    pub(crate) ref_id: u32,
    pub(crate) position: Vec2,
}

pub(crate) fn can_passively_engage(
    kind: ObjectKind,
    team: TeamType,
    stats: ObjectStats,
    mobile: bool,
    is_moving: bool,
) -> bool {
    if team == TeamType::Null || !stats.can_attack() || stats.destroyed() {
        return false;
    }
    if !combat_unit_kind(kind) {
        return false;
    }
    if matches!(kind, ObjectKind::Robot(_)) && mobile && is_moving {
        return false;
    }

    true
}

pub(crate) fn passive_engage_target_kind(kind: ObjectKind) -> bool {
    combat_unit_kind(kind)
}

pub(crate) fn passive_attack_target_choice(
    attacker_ref_id: u32,
    attacker_position: Vec2,
    attacker_team: TeamType,
    attacker_stats: ObjectStats,
    attacker_grenade_amount: u8,
    targets: impl IntoIterator<Item = PassiveCombatTargetSnapshot>,
) -> Option<PassiveCombatTargetSnapshot> {
    targets.into_iter().find(|target| {
        target.ref_id != attacker_ref_id
            && passive_engage_target_kind(target.kind)
            && target.team != TeamType::Null
            && can_attack_target_identity(
                attacker_team,
                attacker_stats,
                attacker_grenade_amount,
                target.team,
                target.stats,
            )
            && attacker_position.distance(target.position) <= attacker_stats.attack_radius
    })
}

pub(crate) fn passive_agro_target_choice(
    attacker_ref_id: u32,
    attacker_position: Vec2,
    attacker_team: TeamType,
    attacker_stats: ObjectStats,
    attacker_grenade_amount: u8,
    targets: impl IntoIterator<Item = PassiveCombatTargetSnapshot>,
) -> Option<PassiveCombatTargetSnapshot> {
    targets
        .into_iter()
        .filter(|target| {
            target.ref_id != attacker_ref_id
                && passive_engage_target_kind(target.kind)
                && target.team != TeamType::Null
                && can_attack_target_identity(
                    attacker_team,
                    attacker_stats,
                    attacker_grenade_amount,
                    target.team,
                    target.stats,
                )
                && attacker_position.distance(target.position) > attacker_stats.attack_radius
                && attacker_position.distance(target.position)
                    <= attacker_stats.attack_radius + AGRO_DISTANCE
        })
        .min_by(|a, b| {
            attacker_position
                .distance_squared(a.position)
                .total_cmp(&attacker_position.distance_squared(b.position))
        })
}

pub(crate) fn attack_to_target_choices(
    attacker_ref_id: u32,
    attacker_position: Vec2,
    attacker_team: TeamType,
    attacker_stats: ObjectStats,
    targets: impl IntoIterator<Item = PassiveCombatTargetSnapshot>,
) -> Vec<PassiveCombatTargetSnapshot> {
    if !attacker_stats.can_attack() || attacker_team == TeamType::Null {
        return Vec::new();
    }

    targets
        .into_iter()
        .filter(|target| {
            target.ref_id != attacker_ref_id
                && passive_engage_target_kind(target.kind)
                && target.team != TeamType::Null
                && target.team != attacker_team
                && !target.stats.destroyed()
                && attacker_position.distance(target.position)
                    <= attacker_stats.attack_radius + AGRO_DISTANCE
        })
        .collect()
}

pub(crate) fn uses_robot_route_footprint(kind: ObjectKind) -> bool {
    matches!(kind, ObjectKind::Robot(_))
}

pub(crate) fn passive_grenade_pickup_target_choice(
    robot_kind: ObjectKind,
    mobile: bool,
    robot: PassiveAutoEnterRobotSnapshot,
    grenade_amount: Option<u8>,
    grenade_boxes: &[PassiveGrenadeBoxSnapshot],
) -> Option<PassiveGrenadeBoxSnapshot> {
    if !passive_grenade_pickup_unit_ready(
        robot_kind,
        mobile,
        can_pickup_grenades(robot_kind, grenade_amount.unwrap_or(0)),
        robot.has_waypoint,
        robot.has_attack_target,
        robot.has_task_target,
        robot.is_minion,
    ) {
        return None;
    }

    grenade_boxes
        .iter()
        .copied()
        .filter(|target| robot.position.distance(target.position) <= AUTO_GRAB_VEHICLE_DISTANCE)
        .min_by(|a, b| {
            robot
                .position
                .distance_squared(a.position)
                .total_cmp(&robot.position.distance_squared(b.position))
        })
}

pub(crate) fn passive_grenade_pickup_unit_ready(
    kind: ObjectKind,
    mobile: bool,
    can_pickup_grenades: bool,
    has_waypoint: bool,
    has_attack_target: bool,
    has_task_target: bool,
    is_minion: bool,
) -> bool {
    matches!(kind, ObjectKind::Robot(_))
        && mobile
        && can_pickup_grenades
        && !has_waypoint
        && !has_attack_target
        && !has_task_target
        && !is_minion
}

pub(crate) fn passive_auto_enter_target_choice(
    robot_kind: ObjectKind,
    mobile: bool,
    robot: PassiveAutoEnterRobotSnapshot,
    targets: &[PassiveAutoEnterTargetSnapshot],
) -> Option<PassiveAutoEnterTargetSnapshot> {
    if !passive_auto_enter_unit_ready(
        robot_kind,
        mobile,
        robot.has_waypoint,
        robot.has_attack_target,
        robot.has_task_target,
        robot.is_minion,
    ) {
        return None;
    }

    targets
        .iter()
        .copied()
        .filter(|target| {
            robot.ref_id != target.ref_id
                && robot.position.distance(target.position) <= AUTO_GRAB_VEHICLE_DISTANCE
                && passive_auto_enter_allows_target(target.kind, robot.just_left_cannon)
        })
        .min_by(|a, b| {
            robot
                .position
                .distance_squared(a.position)
                .total_cmp(&robot.position.distance_squared(b.position))
        })
}

pub(crate) fn passive_auto_enter_unit_ready(
    kind: ObjectKind,
    mobile: bool,
    has_waypoint: bool,
    has_attack_target: bool,
    has_task_target: bool,
    is_minion: bool,
) -> bool {
    matches!(kind, ObjectKind::Robot(_))
        && mobile
        && !has_waypoint
        && !has_attack_target
        && !has_task_target
        && !is_minion
}

pub(crate) fn passive_auto_enter_allows_target(
    target_kind: ObjectKind,
    just_left_cannon: bool,
) -> bool {
    !(just_left_cannon && matches!(target_kind, ObjectKind::Cannon(_)))
}

pub(crate) fn passive_auto_enter_allows_available_target(kind: ObjectKind) -> bool {
    !matches!(kind, ObjectKind::Vehicle(VehicleType::Apc))
}

pub(crate) fn passive_enter_fort_unit_ready(
    kind: ObjectKind,
    mobile: bool,
    has_waypoint: bool,
    has_attack_target: bool,
    has_task_target: bool,
    is_minion: bool,
) -> bool {
    matches!(kind, ObjectKind::Robot(_) | ObjectKind::Vehicle(_))
        && mobile
        && !has_waypoint
        && !has_attack_target
        && !has_task_target
        && !is_minion
}

pub(crate) fn passive_enter_fort_target_building(
    kind: ObjectKind,
    team: TeamType,
    destroyed: bool,
) -> Option<BuildingType> {
    let ObjectKind::Building(building @ (BuildingType::FortFront | BuildingType::FortBack)) = kind
    else {
        return None;
    };

    (team != TeamType::Null && !destroyed).then_some(building)
}

pub(crate) fn passive_enter_fort_target_snapshot(
    snapshot: PassiveCombatTargetSnapshot,
) -> Option<PassiveEnterFortTargetSnapshot> {
    let building = passive_enter_fort_target_building(
        snapshot.kind,
        snapshot.team,
        snapshot.stats.destroyed(),
    )?;
    let (inside_point, exit_point) = fort_entry_points(snapshot.position, snapshot.size, building)?;

    Some(PassiveEnterFortTargetSnapshot {
        ref_id: snapshot.ref_id,
        position: snapshot.position,
        team: snapshot.team,
        inside_point,
        exit_point,
    })
}

pub(crate) fn passive_enter_fort_target_choice(
    unit_kind: ObjectKind,
    mobile: bool,
    unit_team: TeamType,
    unit: PassiveAutoEnterRobotSnapshot,
    targets: &[PassiveEnterFortTargetSnapshot],
) -> Option<PassiveEnterFortTargetSnapshot> {
    if !passive_enter_fort_unit_ready(
        unit_kind,
        mobile,
        unit.has_waypoint,
        unit.has_attack_target,
        unit.has_task_target,
        unit.is_minion,
    ) {
        return None;
    }

    targets
        .iter()
        .copied()
        .filter(|target| target.team != unit_team)
        .min_by(|a, b| {
            unit.position
                .distance_squared(a.position)
                .total_cmp(&unit.position.distance_squared(b.position))
        })
}

pub(crate) fn is_flag(kind: ObjectKind) -> bool {
    matches!(kind, ObjectKind::MapItem(id) if id == ItemType::Flag as u8)
}

pub(crate) fn can_capture_zone(kind: ObjectKind, team: TeamType, destroyed: bool) -> bool {
    !destroyed
        && team != TeamType::Null
        && matches!(kind, ObjectKind::Robot(_) | ObjectKind::Vehicle(_))
}

pub(crate) fn eliminated_team_for_destroyed_fort(
    kind: ObjectKind,
    team: TeamType,
    destroyed: bool,
) -> Option<TeamType> {
    (destroyed && team != TeamType::Null && fort_kind(kind)).then_some(team)
}

pub(crate) fn is_alive_combat_unit(
    kind: ObjectKind,
    team: TeamType,
    object_team: TeamType,
    destroyed: bool,
) -> bool {
    object_team == team && !destroyed && combat_unit_kind(kind)
}

pub(crate) fn is_alive_fort(
    kind: ObjectKind,
    team: TeamType,
    object_team: TeamType,
    destroyed: bool,
) -> bool {
    object_team == team && !destroyed && fort_kind(kind)
}

pub(crate) fn object_should_be_destroyed_by_fort_elimination(
    kind: ObjectKind,
    team: TeamType,
    destroyed: bool,
    eliminated_team: TeamType,
) -> bool {
    team == eliminated_team && !destroyed && !matches!(kind, ObjectKind::MapItem(_))
}

pub(crate) fn movement_speed_multiplier(
    kind: ObjectKind,
    stats: ObjectStats,
    running: bool,
) -> f32 {
    damaged_speed_multiplier(kind, stats) * run_speed_multiplier(kind, stats, running)
}

pub(crate) fn speed_offset_percent(
    moving: bool,
    base_move_speed: f32,
    actual_move_speed: f32,
) -> f32 {
    if !moving || base_move_speed <= 0.0 {
        return 1.0;
    }

    (actual_move_speed / base_move_speed).max(0.0)
}

pub(crate) fn damaged_speed_multiplier(kind: ObjectKind, stats: ObjectStats) -> f32 {
    let ObjectKind::Vehicle(_) = kind else {
        return 1.0;
    };
    if stats.max_health <= 0.0 {
        return 1.0;
    }

    crate::units::vehicles::damaged_speed_multiplier_for_ratio(stats.health / stats.max_health)
}

pub(crate) fn run_speed_multiplier(kind: ObjectKind, stats: ObjectStats, running: bool) -> f32 {
    if let ObjectKind::Vehicle(_) = kind
        && stats.max_health > 0.0
    {
        return crate::units::vehicles::run_speed_multiplier_for_ratio(
            stats.health / stats.max_health,
            running,
        );
    }

    if running { RUN_UNIT_SPEED } else { 1.0 }
}

pub(crate) fn missile_object_particle_amount(kind: ObjectKind, amount: usize) -> usize {
    if matches!(kind, ObjectKind::Robot(_)) {
        amount / 2
    } else {
        amount
    }
}

fn combat_unit_kind(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Cannon(_) | ObjectKind::Vehicle(_) | ObjectKind::Robot(_)
    )
}

fn fort_kind(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Building(BuildingType::FortFront | BuildingType::FortBack)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{CannonType, RobotType, VehicleType};

    #[test]
    fn passive_engage_accepts_live_owned_combat_units() {
        let grunt = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let jeep = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);
        let cannon = ObjectStats::from_kind(ObjectKind::Cannon(CannonType::Gun), 100);

        assert!(can_passively_engage(
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            grunt,
            true,
            false
        ));
        assert!(can_passively_engage(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Red,
            jeep,
            true,
            true
        ));
        assert!(can_passively_engage(
            ObjectKind::Cannon(CannonType::Gun),
            TeamType::Red,
            cannon,
            false,
            false
        ));
        assert!(!can_passively_engage(
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Null,
            grunt,
            true,
            false
        ));
        assert!(!can_passively_engage(
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            grunt,
            true,
            true
        ));
    }

    #[test]
    fn passive_target_rules_keep_unit_filters() {
        assert!(passive_engage_target_kind(ObjectKind::Robot(
            RobotType::Grunt
        )));
        assert!(passive_engage_target_kind(ObjectKind::Vehicle(
            VehicleType::Jeep
        )));
        assert!(passive_engage_target_kind(ObjectKind::Cannon(
            CannonType::Gun
        )));
        assert!(!passive_engage_target_kind(ObjectKind::Rock));
        assert!(passive_auto_enter_allows_target(
            ObjectKind::Vehicle(VehicleType::Jeep),
            true
        ));
        assert!(!passive_auto_enter_allows_target(
            ObjectKind::Cannon(CannonType::Gun),
            true
        ));
        assert!(passive_auto_enter_allows_available_target(
            ObjectKind::Vehicle(VehicleType::Jeep)
        ));
        assert!(!passive_auto_enter_allows_available_target(
            ObjectKind::Vehicle(VehicleType::Apc)
        ));
    }

    #[test]
    fn attack_to_choices_match_source_agro_radius_filter() {
        let attacker = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let jeep = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);
        let building = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Radar), 100);
        let choices = attack_to_target_choices(
            1,
            Vec2::ZERO,
            TeamType::Red,
            attacker,
            [
                PassiveCombatTargetSnapshot {
                    ref_id: 2,
                    kind: ObjectKind::Vehicle(VehicleType::Jeep),
                    position: Vec2::new(attacker.attack_radius * 0.5, 0.0),
                    size: Vec2::ZERO,
                    team: TeamType::Blue,
                    stats: jeep,
                },
                PassiveCombatTargetSnapshot {
                    ref_id: 3,
                    kind: ObjectKind::Vehicle(VehicleType::Jeep),
                    position: Vec2::new(attacker.attack_radius + AGRO_DISTANCE + 1.0, 0.0),
                    size: Vec2::ZERO,
                    team: TeamType::Blue,
                    stats: jeep,
                },
                PassiveCombatTargetSnapshot {
                    ref_id: 4,
                    kind: ObjectKind::Building(BuildingType::Radar),
                    position: Vec2::new(attacker.attack_radius * 0.5, 0.0),
                    size: Vec2::ZERO,
                    team: TeamType::Blue,
                    stats: building,
                },
                PassiveCombatTargetSnapshot {
                    ref_id: 5,
                    kind: ObjectKind::Vehicle(VehicleType::Jeep),
                    position: Vec2::new(attacker.attack_radius * 0.5, 0.0),
                    size: Vec2::ZERO,
                    team: TeamType::Red,
                    stats: jeep,
                },
            ],
        );

        assert_eq!(
            choices
                .iter()
                .map(|target| target.ref_id)
                .collect::<Vec<_>>(),
            vec![2]
        );
    }

    #[test]
    fn passive_idle_unit_gates_are_unit_scoped() {
        assert!(passive_grenade_pickup_unit_ready(
            ObjectKind::Robot(RobotType::Grunt),
            true,
            true,
            false,
            false,
            false,
            false
        ));
        assert!(!passive_grenade_pickup_unit_ready(
            ObjectKind::Vehicle(VehicleType::Jeep),
            true,
            true,
            false,
            false,
            false,
            false
        ));
        assert!(passive_auto_enter_unit_ready(
            ObjectKind::Robot(RobotType::Grunt),
            true,
            false,
            false,
            false,
            false
        ));
        assert!(passive_enter_fort_unit_ready(
            ObjectKind::Vehicle(VehicleType::Jeep),
            true,
            false,
            false,
            false,
            false
        ));
        assert!(!passive_enter_fort_unit_ready(
            ObjectKind::Cannon(CannonType::Gun),
            true,
            false,
            false,
            false,
            false
        ));
    }

    #[test]
    fn fort_elimination_rules_live_with_units() {
        assert_eq!(
            eliminated_team_for_destroyed_fort(
                ObjectKind::Building(BuildingType::FortFront),
                TeamType::Blue,
                true
            ),
            Some(TeamType::Blue)
        );
        assert_eq!(
            eliminated_team_for_destroyed_fort(
                ObjectKind::Building(BuildingType::Radar),
                TeamType::Blue,
                true
            ),
            None
        );
        assert!(is_alive_combat_unit(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Red,
            TeamType::Red,
            false
        ));
        assert!(is_alive_fort(
            ObjectKind::Building(BuildingType::FortBack),
            TeamType::Red,
            TeamType::Red,
            false
        ));
        assert!(!object_should_be_destroyed_by_fort_elimination(
            ObjectKind::MapItem(ItemType::Flag as u8),
            TeamType::Red,
            false,
            TeamType::Red
        ));
    }

    #[test]
    fn speed_offset_percent_matches_original_velocity_ratio() {
        assert_eq!(speed_offset_percent(false, 4.0, 7.2), 1.0);
        assert_eq!(speed_offset_percent(true, 0.0, 7.2), 1.0);
        assert_eq!(speed_offset_percent(true, 4.0, 7.2), 1.8);
        assert_eq!(speed_offset_percent(true, 4.0, -1.0), 0.0);
    }
}
