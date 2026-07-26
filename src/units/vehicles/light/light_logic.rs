use crate::units::{UnitAttackSound, UnitSettings, run_time};

pub(crate) const REQUIRES_ACTIVATION: bool = false;

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 0,
        move_speed: 14.0,
        attack_radius: 120.0,
        attack_damage: 50.0 / 240.0,
        attack_damage_chance: 0.0,
        attack_damage_radius: 40.0,
        attack_missile_speed: 225.0,
        attack_speed: 1.128,
        attack_snipe_chance: 0.0,
        health_ratio: 25.0 / 74.0,
        build_time: 137.0,
        max_run_time: run_time(120.0, 14.0),
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::Light)
}
