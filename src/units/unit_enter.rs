use crate::{
    components::ObjectStats,
    original::{
        objects::{BuildingType, ObjectKind},
        types::TeamType,
    },
};

pub(crate) fn can_be_entered_target(kind: ObjectKind, team: TeamType, stats: ObjectStats) -> bool {
    team == TeamType::Null
        && !stats.destroyed()
        && matches!(kind, ObjectKind::Vehicle(_) | ObjectKind::Cannon(_))
}

pub(crate) fn can_enter_target(kind: ObjectKind, team: TeamType, stats: ObjectStats) -> bool {
    team != TeamType::Null && !stats.destroyed() && matches!(kind, ObjectKind::Robot(_))
}

pub(crate) fn can_enter_fort_unit(kind: ObjectKind, team: TeamType, stats: ObjectStats) -> bool {
    team != TeamType::Null
        && !stats.destroyed()
        && stats.move_speed > 0.0
        && matches!(kind, ObjectKind::Robot(_) | ObjectKind::Vehicle(_))
}

pub(crate) fn can_enter_fort(
    kind: ObjectKind,
    fort_team: TeamType,
    entering_team: TeamType,
    stats: ObjectStats,
) -> bool {
    matches!(
        kind,
        ObjectKind::Building(BuildingType::FortFront | BuildingType::FortBack)
    ) && fort_team != entering_team
        && !stats.destroyed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{CannonType, RobotType, VehicleType};

    #[test]
    fn can_be_entered_matches_original_target_rules() {
        let vehicle = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);
        let destroyed_vehicle = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 0);
        let robot = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);

        assert!(can_be_entered_target(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Null,
            vehicle
        ));
        assert!(can_be_entered_target(
            ObjectKind::Cannon(CannonType::Gun),
            TeamType::Null,
            ObjectStats::from_kind(ObjectKind::Cannon(CannonType::Gun), 100)
        ));
        assert!(!can_be_entered_target(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Red,
            vehicle
        ));
        assert!(!can_be_entered_target(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Null,
            destroyed_vehicle
        ));
        assert!(!can_be_entered_target(
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Null,
            robot
        ));
    }

    #[test]
    fn can_enter_target_is_robot_only_like_enter_wp() {
        let grunt = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let jeep = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);

        assert!(can_enter_target(
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            grunt
        ));
        assert!(!can_enter_target(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Red,
            jeep
        ));
        assert!(!can_enter_target(
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Null,
            grunt
        ));
    }

    #[test]
    fn can_enter_fort_unit_accepts_mobile_live_owned_units() {
        let grunt = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let jeep = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100);
        let cannon = ObjectStats::from_kind(ObjectKind::Cannon(CannonType::Gun), 100);
        let destroyed_grunt = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 0);

        assert!(can_enter_fort_unit(
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            grunt
        ));
        assert!(can_enter_fort_unit(
            ObjectKind::Vehicle(VehicleType::Jeep),
            TeamType::Red,
            jeep
        ));
        assert!(!can_enter_fort_unit(
            ObjectKind::Cannon(CannonType::Gun),
            TeamType::Red,
            cannon
        ));
        assert!(!can_enter_fort_unit(
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Null,
            grunt
        ));
        assert!(!can_enter_fort_unit(
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            destroyed_grunt
        ));
    }

    #[test]
    fn can_enter_fort_matches_original_owner_and_destroyed_rules() {
        let fort = ObjectStats::from_kind(ObjectKind::Building(BuildingType::FortFront), 100);
        let destroyed_fort =
            ObjectStats::from_kind(ObjectKind::Building(BuildingType::FortFront), 0);

        assert!(can_enter_fort(
            ObjectKind::Building(BuildingType::FortFront),
            TeamType::Blue,
            TeamType::Red,
            fort
        ));
        assert!(!can_enter_fort(
            ObjectKind::Building(BuildingType::FortFront),
            TeamType::Red,
            TeamType::Red,
            fort
        ));
        assert!(!can_enter_fort(
            ObjectKind::Building(BuildingType::FortFront),
            TeamType::Blue,
            TeamType::Red,
            destroyed_fort
        ));
        assert!(!can_enter_fort(
            ObjectKind::Building(BuildingType::Radar),
            TeamType::Blue,
            TeamType::Red,
            ObjectStats::from_kind(ObjectKind::Building(BuildingType::Radar), 100)
        ));
    }
}
