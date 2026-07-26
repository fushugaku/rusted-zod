use crate::units::{UnitAttackSound, UnitSettings, run_time};

pub(crate) const REQUIRES_ACTIVATION: bool = true;

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: 4,
        move_speed: 12.0,
        attack_radius: 120.0,
        attack_damage: 0.010486,
        attack_damage_chance: 0.7,
        attack_damage_radius: 0.0,
        attack_missile_speed: 0.0,
        attack_speed: 0.1,
        attack_snipe_chance: 0.0,
        health_ratio: 20.0 / 74.0,
        build_time: 161.0,
        max_run_time: run_time(120.0, 12.0),
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::Pyro)
}
