use crate::units::{UnitAttackSound, UnitSettings};

const GROUP_AMOUNT: u8 = 0;
const MOVE_SPEED: f32 = 0.0;
const ATTACK_RADIUS: f32 = 144.0;
const ATTACK_DAMAGE: f32 = 200.0 / 240.0;
const ATTACK_DAMAGE_CHANCE: f32 = 0.0;
const ATTACK_DAMAGE_RADIUS: f32 = 50.0;
const ATTACK_MISSILE_SPEED: f32 = 128.0;
const ATTACK_SPEED: f32 = 1.124;
const ATTACK_SNIPE_CHANCE: f32 = 0.0;
const HEALTH_RATIO: f32 = 25.0 / 74.0;
const BUILD_TIME: f32 = 182.0;
const MAX_RUN_TIME: f32 = 0.0;

pub(crate) fn settings() -> UnitSettings {
    UnitSettings {
        group_amount: GROUP_AMOUNT,
        move_speed: MOVE_SPEED,
        attack_radius: ATTACK_RADIUS,
        attack_damage: ATTACK_DAMAGE,
        attack_damage_chance: ATTACK_DAMAGE_CHANCE,
        attack_damage_radius: ATTACK_DAMAGE_RADIUS,
        attack_missile_speed: ATTACK_MISSILE_SPEED,
        attack_speed: ATTACK_SPEED,
        attack_snipe_chance: ATTACK_SNIPE_CHANCE,
        health_ratio: HEALTH_RATIO,
        build_time: BUILD_TIME,
        max_run_time: MAX_RUN_TIME,
    }
}

pub(crate) fn attack_sound() -> Option<UnitAttackSound> {
    Some(UnitAttackSound::MobileMissile)
}
