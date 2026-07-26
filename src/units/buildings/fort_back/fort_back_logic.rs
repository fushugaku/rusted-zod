use crate::{
    components::ProductionLevel,
    original::objects::{ObjectKind, RobotType},
};

pub(crate) fn health_ratio() -> f32 {
    crate::units::buildings::fort_front::health_ratio()
}

pub(crate) fn default_production_unit() -> ObjectKind {
    ObjectKind::Robot(RobotType::Grunt)
}

pub(crate) fn default_build_list(level: ProductionLevel) -> Vec<ObjectKind> {
    crate::units::buildings::fort_front::default_build_list(level)
}
