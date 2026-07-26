use crate::{
    original::objects::ObjectKind,
    units::{buildings, items},
};

pub(crate) const MAX_UNIT_HEALTH: f32 = 10000.0;

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

pub(crate) fn object_max_health(kind: ObjectKind) -> f32 {
    if let Some(settings) = crate::units::unit_settings(kind) {
        return unit_settings_max_health(settings);
    }

    let ratio = match kind {
        ObjectKind::Building(building) | ObjectKind::Bridge(building) => {
            buildings::health_ratio(building)
        }
        ObjectKind::Rock | ObjectKind::Animal(_) | ObjectKind::MapItem(_) => {
            items::item_object_health_ratio(kind).expect("item health ratio exists")
        }
        ObjectKind::Robot(_) | ObjectKind::Vehicle(_) | ObjectKind::Cannon(_) => {
            unreachable!("mobile combat units use UnitSettings")
        }
    };

    scaled_to_original_int(ratio)
}

pub(crate) fn object_move_speed(kind: ObjectKind) -> f32 {
    crate::units::unit_settings(kind).map_or(0.0, |settings| settings.move_speed)
}

pub(crate) fn object_attack_radius(kind: ObjectKind) -> f32 {
    crate::units::unit_settings(kind).map_or(0.0, |settings| settings.attack_radius)
}

pub(crate) fn object_attack_damage(kind: ObjectKind) -> f32 {
    crate::units::unit_settings(kind).map_or(0.0, |settings| {
        scaled_to_original_int(settings.attack_damage)
    })
}

pub(crate) fn object_damage_chance(kind: ObjectKind) -> f32 {
    crate::units::unit_settings(kind).map_or(0.0, |settings| settings.attack_damage_chance)
}

pub(crate) fn object_damage_radius(kind: ObjectKind) -> f32 {
    crate::units::unit_settings(kind).map_or(0.0, |settings| settings.attack_damage_radius)
}

pub(crate) fn object_missile_speed(kind: ObjectKind) -> f32 {
    crate::units::unit_settings(kind).map_or(0.0, |settings| settings.attack_missile_speed)
}

pub(crate) fn object_attack_speed(kind: ObjectKind) -> f32 {
    crate::units::unit_settings(kind).map_or(0.0, |settings| settings.attack_speed)
}

pub(crate) fn object_snipe_chance(kind: ObjectKind) -> f32 {
    crate::units::unit_settings(kind).map_or(0.0, |settings| settings.attack_snipe_chance)
}

pub(crate) fn unit_settings_max_health(settings: UnitSettings) -> f32 {
    scaled_to_original_int(settings.health_ratio)
}

fn scaled_to_original_int(value: f32) -> f32 {
    (value * MAX_UNIT_HEALTH) as i32 as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{BuildingType, ItemType, RobotType, VehicleType};

    #[test]
    fn object_health_ratios_live_in_unit_files() {
        assert_eq!(
            object_max_health(ObjectKind::Building(BuildingType::FortFront)),
            416666.0
        );
        assert_eq!(
            object_max_health(ObjectKind::Building(BuildingType::Radar)),
            83333.0
        );
        assert_eq!(object_max_health(ObjectKind::Rock), 1250.0);
        assert_eq!(
            object_max_health(ObjectKind::MapItem(ItemType::Rock as u8)),
            1250.0
        );
        assert_eq!(
            object_max_health(ObjectKind::MapItem(ItemType::Flag as u8)),
            1666.0
        );
    }

    #[test]
    fn combat_stats_delegate_to_unit_settings() {
        assert_eq!(
            object_move_speed(ObjectKind::Vehicle(VehicleType::Jeep)),
            17.0
        );
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
