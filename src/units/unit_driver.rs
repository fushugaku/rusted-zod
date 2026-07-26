use crate::{
    components::{DriverHealth, ObjectStats},
    original::{
        objects::{ObjectKind, RobotType, VehicleType},
        types::TeamType,
    },
    units::object_max_health,
};

pub(crate) fn attacked_only_by_explosives(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Rock | ObjectKind::Bridge(_) | ObjectKind::Building(_) | ObjectKind::MapItem(_)
    )
}

pub(crate) fn can_have_driver(kind: ObjectKind) -> bool {
    matches!(kind, ObjectKind::Vehicle(_) | ObjectKind::Cannon(_))
}

pub(crate) fn initial_driver_health(kind: ObjectKind, team: TeamType) -> Option<DriverHealth> {
    (team != TeamType::Null && can_have_driver(kind)).then_some(grunt_driver_health())
}

pub(crate) fn grunt_driver_health() -> DriverHealth {
    let max_health = object_max_health(ObjectKind::Robot(RobotType::Grunt));
    DriverHealth::new(RobotType::Grunt, max_health)
}

pub(crate) fn can_eject_drivers(kind: ObjectKind, stats: ObjectStats) -> bool {
    !stats.destroyed()
        && match kind {
            ObjectKind::Vehicle(VehicleType::Apc) => true,
            ObjectKind::Cannon(_) => stats.cannon_ejectable,
            _ => false,
        }
}

pub(crate) fn cannon_ejectable_on_spawn(kind: ObjectKind, fort_turret_tile: bool) -> bool {
    !(matches!(kind, ObjectKind::Cannon(_)) && fort_turret_tile)
}

pub(crate) fn cannon_ejectable_for_runtime_spawn(
    kind: ObjectKind,
    requested_ejectable: bool,
) -> bool {
    !matches!(kind, ObjectKind::Cannon(_)) || requested_ejectable
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::CannonType;

    #[test]
    fn driver_eligibility_lives_with_units() {
        assert!(can_have_driver(ObjectKind::Vehicle(VehicleType::Apc)));
        assert!(can_have_driver(ObjectKind::Cannon(CannonType::Gun)));
        assert!(!can_have_driver(ObjectKind::Robot(RobotType::Grunt)));
        assert!(
            initial_driver_health(ObjectKind::Cannon(CannonType::Gun), TeamType::Red).is_some()
        );
        assert!(
            initial_driver_health(ObjectKind::Cannon(CannonType::Gun), TeamType::Null).is_none()
        );
    }

    #[test]
    fn eject_rules_keep_apc_and_cannon_special_cases() {
        let apc = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Apc), 100);
        let mut cannon = ObjectStats::from_kind(ObjectKind::Cannon(CannonType::Gun), 100);
        let destroyed_apc = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Apc), 0);

        assert!(can_eject_drivers(
            ObjectKind::Vehicle(VehicleType::Apc),
            apc
        ));
        assert!(can_eject_drivers(
            ObjectKind::Cannon(CannonType::Gun),
            cannon
        ));
        cannon.cannon_ejectable = false;
        assert!(!can_eject_drivers(
            ObjectKind::Cannon(CannonType::Gun),
            cannon
        ));
        assert!(!can_eject_drivers(
            ObjectKind::Vehicle(VehicleType::Apc),
            destroyed_apc
        ));
    }

    #[test]
    fn static_map_objects_are_explosive_only_targets() {
        assert!(attacked_only_by_explosives(ObjectKind::Rock));
        assert!(attacked_only_by_explosives(ObjectKind::Building(
            crate::original::objects::BuildingType::Radar
        )));
        assert!(!attacked_only_by_explosives(ObjectKind::Robot(
            RobotType::Grunt
        )));
    }
}
