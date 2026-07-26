use crate::units::{UnitAttackSound, UnitSettings, run_time};

pub(crate) const REQUIRES_ACTIVATION: bool = true;

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 0,
        move_speed: 6.0,
        attack_radius: 160.0,
        attack_damage: 62.0 / 74.0,
        attack_damage_chance: 0.0,
        attack_damage_radius: 80.0,
        attack_missile_speed: 70.0,
        attack_speed: 4.454,
        attack_snipe_chance: 0.0,
        health_ratio: 50.0 / 74.0,
        build_time: 373.0,
        max_run_time: run_time(160.0, 6.0) * 0.5,
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::MobileMissile)
}
