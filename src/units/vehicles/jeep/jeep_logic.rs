use crate::units::{UnitAttackSound, UnitSettings, run_time};

pub(crate) const REQUIRES_ACTIVATION: bool = false;

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 0,
        move_speed: 17.0,
        attack_radius: 120.0,
        attack_damage: 0.0027067,
        attack_damage_chance: 0.65,
        attack_damage_radius: 0.0,
        attack_missile_speed: 0.0,
        attack_speed: 0.1,
        attack_snipe_chance: 0.4,
        health_ratio: 13.0 / 74.0,
        build_time: 81.0,
        max_run_time: run_time(120.0, 17.0),
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::Jeep)
}
