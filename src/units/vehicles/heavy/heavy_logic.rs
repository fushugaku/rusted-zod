use crate::units::{UnitAttackSound, UnitSettings, run_time};

pub(crate) const REQUIRES_ACTIVATION: bool = true;

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 0,
        move_speed: 9.0,
        attack_radius: 144.0,
        attack_damage: 120.0 / 240.0,
        attack_damage_chance: 0.0,
        attack_damage_radius: 50.0,
        attack_missile_speed: 135.0,
        attack_speed: 4.088,
        attack_snipe_chance: 0.0,
        health_ratio: 62.0 / 74.0,
        build_time: 309.0,
        max_run_time: run_time(144.0, 9.0) * 0.7,
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::Heavy)
}
