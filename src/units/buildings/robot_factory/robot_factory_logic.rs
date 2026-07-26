use crate::{
    components::ProductionLevel,
    original::objects::{CannonType, ObjectKind, RobotType},
};

pub(crate) fn health_ratio() -> f32 {
    2000.0 / 240.0
}

pub(crate) fn default_production_unit() -> ObjectKind {
    ObjectKind::Robot(RobotType::Grunt)
}

pub(crate) fn default_build_list(level: ProductionLevel) -> Vec<ObjectKind> {
    use ProductionLevel::*;

    match level {
        Level0 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Cannon(CannonType::Gatling),
        ],
        Level1 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Cannon(CannonType::Gatling),
        ],
        Level2 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Cannon(CannonType::Gun),
        ],
        Level3 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Robot(RobotType::Pyro),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Cannon(CannonType::Howitzer),
        ],
        Level4 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Robot(RobotType::Pyro),
            ObjectKind::Robot(RobotType::Laser),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Cannon(CannonType::Howitzer),
        ],
        Level5 => vec![
            ObjectKind::Robot(RobotType::Grunt),
            ObjectKind::Robot(RobotType::Psycho),
            ObjectKind::Robot(RobotType::Sniper),
            ObjectKind::Robot(RobotType::Tough),
            ObjectKind::Robot(RobotType::Pyro),
            ObjectKind::Robot(RobotType::Laser),
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Cannon(CannonType::Gun),
            ObjectKind::Cannon(CannonType::Howitzer),
            ObjectKind::Cannon(CannonType::MissileCannon),
        ],
    }
}
