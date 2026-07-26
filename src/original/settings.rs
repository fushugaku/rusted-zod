#![allow(dead_code)]

use super::objects::ObjectKind;
pub use crate::units::UnitSettings;

pub const MAX_UNIT_HEALTH: f32 = crate::units::unit_stats::MAX_UNIT_HEALTH;
pub const AGRO_DISTANCE: f32 = crate::units::unit_behavior::AGRO_DISTANCE;
pub const AUTO_GRAB_VEHICLE_DISTANCE: f32 = crate::units::unit_behavior::AUTO_GRAB_VEHICLE_DISTANCE;
pub const GRENADES_PER_BOX: u8 = crate::units::items::grenades::GRENADES_PER_BOX;
pub const GRENADE_DAMAGE: f32 = crate::units::items::grenades::GRENADE_DAMAGE;
pub const GRENADE_DAMAGE_RADIUS: f32 = crate::units::items::grenades::GRENADE_DAMAGE_RADIUS;
pub const GRENADE_MISSILE_SPEED: f32 = crate::units::items::grenades::GRENADE_MISSILE_SPEED;
pub const GRENADE_ATTACK_SPEED: f32 = crate::units::items::grenades::GRENADE_ATTACK_SPEED;
pub const GRENADE_SCATTER_HALF_EXTENT: f32 =
    crate::units::items::grenades::GRENADE_SCATTER_HALF_EXTENT;
pub const GRENADE_BOX_EXPLOSION_RADIUS: f32 =
    crate::units::items::grenades::GRENADE_BOX_EXPLOSION_RADIUS;
pub const GRENADE_BOX_SCATTER_HALF_EXTENT: f32 =
    crate::units::items::grenades::GRENADE_BOX_SCATTER_HALF_EXTENT;
pub const GRENADE_BOX_EXPLOSION_DELAY: f32 =
    crate::units::items::grenades::GRENADE_BOX_EXPLOSION_DELAY;
pub const MAP_ITEM_TURRENT_DAMAGE: f32 = crate::units::items::map_object::TURRENT_DAMAGE;
pub const MAP_ITEM_TURRENT_RADIUS: f32 = crate::units::items::map_object::TURRENT_RADIUS;
pub const MAP_ITEM_TURRENT_DELAY: f32 = crate::units::items::map_object_ui::TURRENT_DELAY;
pub const MAP_ITEM_TURRENT_DELAY_RANDOM: usize =
    crate::units::items::map_object_ui::TURRENT_DELAY_RANDOM;
pub const PARTIALLY_DAMAGED_UNIT_SPEED: f32 = crate::units::vehicles::PARTIALLY_DAMAGED_UNIT_SPEED;
pub const DAMAGED_UNIT_SPEED: f32 = crate::units::vehicles::DAMAGED_UNIT_SPEED;
pub const RUN_UNIT_SPEED: f32 = crate::units::unit_behavior::RUN_UNIT_SPEED;
pub const RUN_RECHARGE_RATE: f32 = crate::units::unit_behavior::RUN_RECHARGE_RATE;

pub fn unit_settings(kind: ObjectKind) -> Option<UnitSettings> {
    crate::units::unit_settings(kind)
}

pub fn object_max_health(kind: ObjectKind) -> f32 {
    crate::units::object_max_health(kind)
}

pub fn object_move_speed(kind: ObjectKind) -> f32 {
    crate::units::object_move_speed(kind)
}

pub fn object_attack_radius(kind: ObjectKind) -> f32 {
    crate::units::object_attack_radius(kind)
}

pub fn object_attack_damage(kind: ObjectKind) -> f32 {
    crate::units::object_attack_damage(kind)
}

pub fn object_damage_chance(kind: ObjectKind) -> f32 {
    crate::units::object_damage_chance(kind)
}

pub fn object_damage_radius(kind: ObjectKind) -> f32 {
    crate::units::object_damage_radius(kind)
}

pub fn object_missile_speed(kind: ObjectKind) -> f32 {
    crate::units::object_missile_speed(kind)
}

pub fn object_attack_speed(kind: ObjectKind) -> f32 {
    crate::units::object_attack_speed(kind)
}

pub fn object_snipe_chance(kind: ObjectKind) -> f32 {
    crate::units::object_snipe_chance(kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{BuildingType, RobotType, VehicleType};

    #[test]
    fn grunt_settings_match_cpp_defaults() {
        let settings = unit_settings(ObjectKind::Robot(RobotType::Grunt)).unwrap();
        assert_eq!(settings.move_speed, 14.0);
        assert_eq!(settings.attack_radius, 120.0);
        assert_eq!(
            object_max_health(ObjectKind::Robot(RobotType::Grunt)),
            1081.0
        );
    }

    #[test]
    fn building_health_matches_cpp_defaults() {
        assert_eq!(
            object_max_health(ObjectKind::Building(BuildingType::FortFront)),
            416666.0
        );
        assert_eq!(
            object_max_health(ObjectKind::Building(BuildingType::Radar)),
            83333.0
        );
    }

    #[test]
    fn vehicle_speeds_are_type_specific() {
        assert_eq!(
            object_move_speed(ObjectKind::Vehicle(VehicleType::Jeep)),
            17.0
        );
        assert_eq!(
            object_move_speed(ObjectKind::Vehicle(VehicleType::Heavy)),
            9.0
        );
        assert_eq!(
            object_move_speed(ObjectKind::Vehicle(VehicleType::MissileLauncher)),
            6.0
        );
    }

    #[test]
    fn attack_damage_uses_original_int_scale() {
        assert_eq!(
            object_attack_damage(ObjectKind::Robot(RobotType::Grunt)),
            11.0
        );
        assert_eq!(
            object_attack_damage(ObjectKind::Vehicle(VehicleType::Heavy)),
            5000.0
        );
    }
}
