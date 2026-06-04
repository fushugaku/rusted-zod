use super::objects::{BuildingType, ItemType, ObjectKind};

pub const MAX_UNIT_HEALTH: f32 = 10000.0;
pub const AGRO_DISTANCE: f32 = 40.0;
pub const AUTO_GRAB_VEHICLE_DISTANCE: f32 = 220.0;
pub const GRENADES_PER_BOX: u8 = 20;
pub const GRENADE_DAMAGE: f32 = 1666.0;
pub const GRENADE_DAMAGE_RADIUS: f32 = 30.0;
pub const GRENADE_MISSILE_SPEED: f32 = 40.0;
pub const GRENADE_ATTACK_SPEED: f32 = 2.254;
pub const GRENADE_SCATTER_HALF_EXTENT: f32 = 24.0;
pub const GRENADE_BOX_EXPLOSION_RADIUS: f32 = 40.0;
pub const GRENADE_BOX_SCATTER_HALF_EXTENT: f32 = 130.0;
pub const GRENADE_BOX_EXPLOSION_DELAY: f32 = 3.0;
pub const MAP_ITEM_TURRENT_DAMAGE: f32 = 2083.0;
pub const MAP_ITEM_TURRENT_RADIUS: f32 = 40.0;
pub const MAP_ITEM_TURRENT_DELAY: f32 = 3.0;
pub const MAP_ITEM_TURRENT_DELAY_RANDOM: usize = 100;
pub const PARTIALLY_DAMAGED_UNIT_SPEED: f32 = 0.9;
pub const DAMAGED_UNIT_SPEED: f32 = 0.8;
pub const RUN_UNIT_SPEED: f32 = 1.8;
pub const RUN_RECHARGE_RATE: f32 = 0.3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitSettings {
    pub group_amount: u8,
    pub move_speed: f32,
    pub attack_radius: f32,
    pub attack_damage: f32,
    pub attack_damage_chance: f32,
    pub attack_damage_radius: f32,
    pub attack_missile_speed: f32,
    pub attack_speed: f32,
    pub attack_snipe_chance: f32,
    pub health_ratio: f32,
    pub build_time: f32,
    pub max_run_time: f32,
}

impl UnitSettings {
    pub fn max_health(self) -> f32 {
        scaled_to_original_int(self.health_ratio)
    }
}

pub fn unit_settings(kind: ObjectKind) -> Option<UnitSettings> {
    crate::units::unit_settings(kind)
}

pub fn object_max_health(kind: ObjectKind) -> f32 {
    if let Some(settings) = unit_settings(kind) {
        return settings.max_health();
    }

    let ratio = match kind {
        ObjectKind::Building(BuildingType::FortFront | BuildingType::FortBack) => 10000.0 / 240.0,
        ObjectKind::Building(
            BuildingType::Radar
            | BuildingType::Repair
            | BuildingType::RobotFactory
            | BuildingType::VehicleFactory
            | BuildingType::BridgeVert
            | BuildingType::BridgeHorz,
        )
        | ObjectKind::Bridge(_) => 2000.0 / 240.0,
        ObjectKind::Rock => 30.0 / 240.0,
        ObjectKind::MapItem(id) if id == ItemType::Rock as u8 => 30.0 / 240.0,
        ObjectKind::MapItem(_) | ObjectKind::Animal(_) => 40.0 / 240.0,
        ObjectKind::Robot(_) | ObjectKind::Vehicle(_) | ObjectKind::Cannon(_) => unreachable!(),
    };

    scaled_to_original_int(ratio)
}

pub fn object_move_speed(kind: ObjectKind) -> f32 {
    unit_settings(kind).map_or(0.0, |settings| settings.move_speed)
}

pub fn object_attack_radius(kind: ObjectKind) -> f32 {
    unit_settings(kind).map_or(0.0, |settings| settings.attack_radius)
}

pub fn object_attack_damage(kind: ObjectKind) -> f32 {
    unit_settings(kind).map_or(0.0, |settings| {
        scaled_to_original_int(settings.attack_damage)
    })
}

pub fn object_damage_chance(kind: ObjectKind) -> f32 {
    unit_settings(kind).map_or(0.0, |settings| settings.attack_damage_chance)
}

pub fn object_damage_radius(kind: ObjectKind) -> f32 {
    unit_settings(kind).map_or(0.0, |settings| settings.attack_damage_radius)
}

pub fn object_missile_speed(kind: ObjectKind) -> f32 {
    unit_settings(kind).map_or(0.0, |settings| settings.attack_missile_speed)
}

pub fn object_attack_speed(kind: ObjectKind) -> f32 {
    unit_settings(kind).map_or(0.0, |settings| settings.attack_speed)
}

pub fn object_snipe_chance(kind: ObjectKind) -> f32 {
    unit_settings(kind).map_or(0.0, |settings| settings.attack_snipe_chance)
}

fn scaled_to_original_int(value: f32) -> f32 {
    (value * MAX_UNIT_HEALTH) as i32 as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{RobotType, VehicleType};

    #[test]
    fn grunt_settings_match_cpp_defaults() {
        let settings = unit_settings(ObjectKind::Robot(RobotType::Grunt)).unwrap();
        assert_eq!(settings.move_speed, 14.0);
        assert_eq!(settings.attack_radius, 120.0);
        assert_eq!(settings.max_health(), 1081.0);
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
