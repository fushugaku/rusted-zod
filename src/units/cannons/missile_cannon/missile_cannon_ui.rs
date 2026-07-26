use bevy::prelude::Vec2;

#[cfg(test)]
use crate::units::cannons::{CannonRenderProfile, PASSIVE_ROTATION_INTERVAL, PLACE_FRAME_TIME};
use crate::{
    components::{CombatRng, DamageCrater, DamageMissileVisual},
    original::types::TeamType,
    units::{
        DamageMissileVisualGeometry, RocketImpactProfile,
        cannons::{CannonFrameProfile, CannonFrameRole, CannonTurrentProfile, PLACE_FRAME_COUNT},
    },
};

const FOLDER: &str = "missile_cannon";
const UNIT_OFFSET: Vec2 = Vec2::new(0.0, -8.0);
const DIRECTION_OFFSETS: [Vec2; 8] = [Vec2::ZERO; 8];
#[cfg(test)]
const FIRE_FLASH_BASE_TIME: f32 = 0.05;
#[cfg(test)]
const FIRE_FLASH_RANDOM_STEPS: usize = 100;
#[cfg(test)]
const FIRE_FLASH_STEP: f32 = 0.0003;

const DEATH_TOP_LEFT_OFFSET: Vec2 = Vec2::new(-16.0, 16.0);
const DEATH_DELAY_BASE: f32 = 2.0;
const DEATH_DELAY_RANDOM_STEPS: usize = 3;
const DEATH_SPARK_BASE_COUNT: usize = 20;
const DEATH_SPARK_RANDOM_STEPS: usize = 15;

const TURRENT_TARGET_CENTER_OFFSET: Vec2 = Vec2::splat(16.0);
const TURRENT_MAX_HORIZONTAL_DISTANCE: f32 = 300.0;
const TURRENT_MAX_VERTICAL_DISTANCE: f32 = 300.0;
const TURRENT_OFFSET_TIME_BASE: f32 = 7.0;
const TURRENT_OFFSET_TIME_RANDOM_STEPS: usize = 300;
const TURRENT_OFFSET_TIME_STEP: f32 = 0.01;
const TURRENT_RISE_BASE: f32 = 1.0;
const TURRENT_RISE_RANDOM_STEPS: usize = 300;
const TURRENT_RISE_STEP: f32 = 0.01;
const TURRENT_START_JITTER_CENTER: f32 = 5.0;
const TURRENT_START_JITTER_STEPS: usize = 10;
const TURRENT_ARC_LIFT_PIXELS: f32 = 30.0;
const TURRENT_SPIN_BASE_DEGREES_PER_SEC: f32 = 240.0;
const TURRENT_SPIN_RANDOM_STEPS: usize = 480;
#[cfg(test)]
const ROCKET_MUZZLE_X: [f32; 8] = [20.0, 12.0, 0.0, -12.0, -20.0, -12.0, 0.0, 12.0];
#[cfg(test)]
const ROCKET_MUZZLE_Y: [f32; 8] = [0.0, -12.0, -20.0, -12.0, 0.0, 12.0, 20.0, 12.0];
const TURRENT_DAMAGE: i32 = 40;
const TURRENT_RADIUS: i32 = 40;

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(32.0)
}

pub(crate) fn hud_name() -> &'static str {
    "missile_cannon"
}

pub(crate) fn damage_missile_visual() -> Option<DamageMissileVisual> {
    Some(DamageMissileVisual::MissileCannon)
}

pub(crate) fn damage_missile_frame_paths() -> Vec<String> {
    vec![format!("units/cannons/{FOLDER}/bullet.png")]
}

#[cfg(test)]
pub(crate) fn render_profile() -> CannonRenderProfile {
    CannonRenderProfile {
        unit_offset: UNIT_OFFSET,
        direction_offsets: DIRECTION_OFFSETS,
        place_frames: PLACE_FRAME_COUNT,
        place_frame_time: PLACE_FRAME_TIME,
        passive_rotation_interval: PASSIVE_ROTATION_INTERVAL,
        fire_flash_base_time: Some(FIRE_FLASH_BASE_TIME),
        fire_flash_random_steps: FIRE_FLASH_RANDOM_STEPS,
        fire_flash_step: FIRE_FLASH_STEP,
    }
}

pub(crate) fn empty_frame_profile(team: TeamType, rotation: u16) -> CannonFrameProfile {
    if team == TeamType::Null {
        return crate::units::cannons::frame_profile(
            CannonFrameRole::Empty,
            TeamType::Red,
            format!("units/cannons/{FOLDER}/empty_null.png"),
        );
    }

    let team = team.atlas_team();
    let team_name = team.asset_name();
    crate::units::cannons::frame_profile(
        CannonFrameRole::Empty,
        team,
        format!("units/cannons/{FOLDER}/empty_{team_name}_r{rotation:03}.png"),
    )
}

pub(crate) fn equipped_frame_profile(team: TeamType, rotation: u16) -> Option<CannonFrameProfile> {
    if team == TeamType::Null {
        return None;
    }

    let team = team.atlas_team();
    let team_name = team.asset_name();
    Some(crate::units::cannons::frame_profile(
        CannonFrameRole::Equipped,
        team,
        format!("units/cannons/{FOLDER}/equiped_{team_name}_r{rotation:03}.png"),
    ))
}

pub(crate) fn passive_frame_profile(team: TeamType, rotation: u16) -> CannonFrameProfile {
    if let Some(mut profile) = equipped_frame_profile(team, rotation) {
        profile.role = CannonFrameRole::Passive;
        profile
    } else {
        empty_frame_profile(team, rotation)
    }
}

#[cfg(test)]
pub(crate) fn fire_frame_profile(team: TeamType, rotation: u16) -> CannonFrameProfile {
    if let Some(mut profile) = equipped_frame_profile(team, rotation) {
        profile.role = CannonFrameRole::Fire;
        profile
    } else {
        empty_frame_profile(team, rotation)
    }
}

pub(crate) fn place_frame_profile(team: TeamType, frame: usize) -> CannonFrameProfile {
    let team = team.atlas_team();
    let team_name = team.asset_name();
    crate::units::cannons::frame_profile(
        CannonFrameRole::Place,
        team,
        format!(
            "units/cannons/{FOLDER}/place_{team_name}_n{:02}.png",
            frame.min(PLACE_FRAME_COUNT - 1)
        ),
    )
}

pub(crate) fn render_offset(direction: usize) -> Vec2 {
    UNIT_OFFSET + DIRECTION_OFFSETS[direction.min(7)]
}

#[cfg(test)]
pub(crate) fn fire_flash_duration(rng: &mut CombatRng) -> Option<f32> {
    Some(FIRE_FLASH_BASE_TIME + rng.index(FIRE_FLASH_RANDOM_STEPS) as f32 * FIRE_FLASH_STEP)
}

pub(crate) fn rocket_impact_profile() -> RocketImpactProfile {
    RocketImpactProfile {
        xx_large_mushrooms: 0,
        large_mushrooms: 3,
        small_mushrooms: 1,
        unit_particle_radius: 50.0,
        unit_particle_amount: 18,
    }
}

pub(crate) fn damage_crater() -> DamageCrater {
    DamageCrater {
        is_big: false,
        chance: 1.0,
        big_chance: None,
    }
}

pub(crate) fn primary_offset(direction: Vec2) -> Vec2 {
    Vec2::new(direction.y * 4.0, -direction.x * 8.0)
}

pub(crate) fn other_offset(direction: Vec2) -> Vec2 {
    Vec2::new(-direction.y * 8.0, direction.x * 8.0)
}

pub(crate) fn visual_geometry(direction: Vec2) -> DamageMissileVisualGeometry {
    let primary_offset = primary_offset(direction);
    let other_offset = primary_offset + other_offset(direction);
    DamageMissileVisualGeometry {
        primary_offset,
        replica_offsets: vec![other_offset],
        smoke_offsets: vec![primary_offset, other_offset],
    }
}

#[cfg(test)]
pub(crate) fn rocket_muzzle_offset(direction: usize) -> Vec2 {
    let direction = direction.min(7);
    Vec2::new(
        1.0 + ROCKET_MUZZLE_X[direction],
        2.0 - ROCKET_MUZZLE_Y[direction],
    )
}

pub(crate) fn death_wreck_asset_path() -> String {
    format!("units/cannons/{FOLDER}/wasted.png")
}

#[cfg(test)]
pub(crate) fn death_wreck_frame_profile() -> CannonFrameProfile {
    crate::units::cannons::frame_profile(
        CannonFrameRole::DeathWreck,
        TeamType::Red,
        death_wreck_asset_path(),
    )
}

pub(crate) fn destroyed_asset_path(team: TeamType) -> String {
    match team.atlas_team() {
        TeamType::Blue => format!("units/cannons/{FOLDER}/wasted_blue.png"),
        TeamType::Green => format!("units/cannons/{FOLDER}/wasted_green.png"),
        TeamType::Yellow => format!("units/cannons/{FOLDER}/wasted_yellow.png"),
        _ => format!("units/cannons/{FOLDER}/wasted_red.png"),
    }
}

#[cfg(test)]
pub(crate) fn destroyed_frame_profile(team: TeamType) -> CannonFrameProfile {
    let atlas_team = team.atlas_team();
    crate::units::cannons::frame_profile(
        CannonFrameRole::Destroyed,
        atlas_team,
        destroyed_asset_path(team),
    )
}

pub(crate) fn death_top_left_world(center: Vec2) -> Vec2 {
    center + DEATH_TOP_LEFT_OFFSET
}

pub(crate) fn death_delay(rng: &mut CombatRng) -> f32 {
    DEATH_DELAY_BASE + rng.index(DEATH_DELAY_RANDOM_STEPS) as f32
}

pub(crate) fn death_spark_count(rng: &mut CombatRng) -> usize {
    DEATH_SPARK_BASE_COUNT + rng.index(DEATH_SPARK_RANDOM_STEPS)
}

pub(crate) fn death_missile_offset_time(rng: &mut CombatRng) -> f32 {
    TURRENT_OFFSET_TIME_BASE
        + rng.index(TURRENT_OFFSET_TIME_RANDOM_STEPS) as f32 * TURRENT_OFFSET_TIME_STEP
}

pub(crate) fn turrent_profile() -> CannonTurrentProfile {
    CannonTurrentProfile {
        target_center_offset: TURRENT_TARGET_CENTER_OFFSET,
        max_horizontal_distance: TURRENT_MAX_HORIZONTAL_DISTANCE,
        max_vertical_distance: TURRENT_MAX_VERTICAL_DISTANCE,
        damage: TURRENT_DAMAGE,
        radius: TURRENT_RADIUS,
        offset_time_base: TURRENT_OFFSET_TIME_BASE,
        offset_time_random_steps: TURRENT_OFFSET_TIME_RANDOM_STEPS,
        offset_time_step: TURRENT_OFFSET_TIME_STEP,
        rise_base: TURRENT_RISE_BASE,
        rise_random_steps: TURRENT_RISE_RANDOM_STEPS,
        rise_step: TURRENT_RISE_STEP,
        start_jitter_center: TURRENT_START_JITTER_CENTER,
        start_jitter_steps: TURRENT_START_JITTER_STEPS,
        arc_lift_pixels: TURRENT_ARC_LIFT_PIXELS,
        spin_base_degrees_per_sec: TURRENT_SPIN_BASE_DEGREES_PER_SEC,
        spin_random_steps: TURRENT_SPIN_RANDOM_STEPS,
    }
}

pub(crate) fn turrent_target_offset(rng: &mut CombatRng) -> Vec2 {
    Vec2::new(
        TURRENT_MAX_HORIZONTAL_DISTANCE
            - rng.index((TURRENT_MAX_HORIZONTAL_DISTANCE * 2.0) as usize) as f32,
        TURRENT_MAX_VERTICAL_DISTANCE
            - rng.index((TURRENT_MAX_VERTICAL_DISTANCE * 2.0) as usize) as f32,
    )
}

pub(crate) fn turrent_rise(rng: &mut CombatRng) -> f32 {
    TURRENT_RISE_BASE + rng.index(TURRENT_RISE_RANDOM_STEPS) as f32 * TURRENT_RISE_STEP
}

pub(crate) fn turrent_start_jitter(rng: &mut CombatRng) -> Vec2 {
    Vec2::new(
        TURRENT_START_JITTER_CENTER - rng.index(TURRENT_START_JITTER_STEPS) as f32,
        TURRENT_START_JITTER_CENTER - rng.index(TURRENT_START_JITTER_STEPS) as f32,
    )
}

pub(crate) fn turrent_spin_degrees_per_sec(rng: &mut CombatRng) -> f32 {
    TURRENT_SPIN_BASE_DEGREES_PER_SEC - rng.index(TURRENT_SPIN_RANDOM_STEPS) as f32
}

pub(crate) fn turrent_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    -(rise / final_time) * (t * t) + rise * t + 1.0
}
