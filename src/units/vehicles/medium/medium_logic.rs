use crate::units::{UnitAttackSound, UnitSettings, run_time};

pub(crate) const REQUIRES_ACTIVATION: bool = true;

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 0,
        move_speed: 12.0,
        attack_radius: 128.0,
        attack_damage: 80.0 / 240.0,
        attack_damage_chance: 0.0,
        attack_damage_radius: 45.0,
        attack_missile_speed: 160.0,
        attack_speed: 2.336,
        attack_snipe_chance: 0.0,
        health_ratio: 50.0 / 74.0,
        build_time: 225.0,
        max_run_time: run_time(128.0, 12.0),
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::Medium)
}
