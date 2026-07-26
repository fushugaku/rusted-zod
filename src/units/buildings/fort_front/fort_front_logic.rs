use crate::{
    components::ProductionLevel,
    original::objects::{CannonType, ObjectKind, RobotType, VehicleType},
};

pub(crate) fn health_ratio() -> f32 {
    10000.0 / 240.0
}

pub(crate) fn default_production_unit() -> ObjectKind {
    ObjectKind::Robot(RobotType::Grunt)
}

pub(crate) fn default_build_list(level: ProductionLevel) -> Vec<ObjectKind> {
    use ProductionLevel::*;

    match level {
        Level0 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Vehicle(VehicleType::Crane),
            ObjectKind::Cannon(CannonType::Gatling),
        ],
        Level1 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Vehicle(VehicleType::Crane),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
        ],
        Level2 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Vehicle(VehicleType::Crane),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Cannon(CannonType::Howitzer),
        ],
        Level3 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Vehicle(VehicleType::Crane),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Cannon(CannonType::Howitzer),
            ObjectKind::Robot(RobotType::Pyro),
            ObjectKind::Vehicle(VehicleType::Apc),
        ],
        Level4 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Vehicle(VehicleType::Crane),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Cannon(CannonType::Howitzer),
            ObjectKind::Robot(RobotType::Pyro),
            ObjectKind::Vehicle(VehicleType::Apc),
            ObjectKind::Robot(RobotType::Laser),
            ObjectKind::Vehicle(VehicleType::Heavy),
            ObjectKind::Cannon(CannonType::MissileCannon),
        ],
        Level5 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Vehicle(VehicleType::Crane),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Cannon(CannonType::Howitzer),
            ObjectKind::Robot(RobotType::Pyro),
            ObjectKind::Vehicle(VehicleType::Apc),
            ObjectKind::Robot(RobotType::Laser),
            ObjectKind::Vehicle(VehicleType::Heavy),
            ObjectKind::Cannon(CannonType::MissileCannon),
            ObjectKind::Vehicle(VehicleType::MissileLauncher),
        ],
    }
}
