use crate::{
    components::ProductionLevel,
    original::objects::{CannonType, ObjectKind, VehicleType},
};

pub(crate) fn health_ratio() -> f32 {
    2000.0 / 240.0
}

pub(crate) fn default_production_unit() -> ObjectKind {
    ObjectKind::Vehicle(VehicleType::Jeep)
}

pub(crate) fn default_build_list(level: ProductionLevel) -> Vec<ObjectKind> {
    use ProductionLevel::*;

    match level {
        Level0 => vec![
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Cannon(CannonType::Gatling),
        ],
        Level1 => vec![
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
        ],
        Level2 => vec![
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Vehicle(VehicleType::Medium),
        ],
        Level3 => vec![
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Vehicle(VehicleType::Apc),
            ObjectKind::Cannon(CannonType::Howitzer),
        ],
        Level4 => vec![
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Vehicle(VehicleType::Apc),
            ObjectKind::Cannon(CannonType::Howitzer),
            ObjectKind::Vehicle(VehicleType::Heavy),
        ],
        Level5 => vec![
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Vehicle(VehicleType::Light),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Vehicle(VehicleType::Medium),
            ObjectKind::Vehicle(VehicleType::Apc),
            ObjectKind::Cannon(CannonType::Howitzer),
            ObjectKind::Vehicle(VehicleType::Heavy),
            ObjectKind::Vehicle(VehicleType::MissileLauncher),
            ObjectKind::Cannon(CannonType::MissileCannon),
        ],
    }
}
