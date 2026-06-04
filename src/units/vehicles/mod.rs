use bevy::prelude::Vec2;

use crate::{
    components::{CombatRng, DamageMissileVisual},
    original::{
        objects::VehicleType,
        settings::{
            DAMAGED_UNIT_SPEED, PARTIALLY_DAMAGED_UNIT_SPEED, RUN_UNIT_SPEED, UnitSettings,
        },
        types::{PlanetType, TeamType},
    },
    rotation_for_direction,
    units::{RocketImpactProfile, UnitAttackSound},
};

pub(crate) mod apc;
pub(crate) mod crane;
pub(crate) mod heavy;
pub(crate) mod jeep;
pub(crate) mod light;
pub(crate) mod medium;
pub(crate) mod missile_launcher;

const TURRENT_MAX_DISTANCE: f32 = 300.0;
pub(crate) const EFFECT_DROP_INTERVAL: f32 = 0.2;
pub(crate) const TANK_DUST_FRAME_TIME: f32 = 0.15;
pub(crate) const TANK_SPARK_FRAME_TIME: f32 = 0.1;
pub(crate) const DEATH_SPARK_FRAME_TIME: f32 = 0.1;
pub(crate) const VEHICLE_BASE_FRAME_TIME: f32 = 0.1;
const TRACK_FADE_FRAME_1_TIME: f32 = 3.3;
const TRACK_FADE_FRAME_2_TIME: f32 = 3.6;
const TRACK_FADE_END_TIME: f32 = 3.9;
const TANK_OIL_FRAME_TIME: f32 = 3.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VehicleTrackKind {
    Jeep,
    Tank,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VehicleMovementProfile {
    pub(crate) base_frame_count: usize,
    pub(crate) base_frame_time: f32,
    pub(crate) track_kind: VehicleTrackKind,
    pub(crate) drops_damage_effects: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VehicleDeathStandardKind {
    BigSmoke,
    LittleFire,
    SmallFireSmoke,
    Fire,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VehicleDeathEffectBounds {
    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VehicleDeathWreckProfile {
    Static,
    DamagedFrames,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VehicleDestroyedAssetProfile {
    None,
    Static,
    TeamStatic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VehicleTurrentProfile {
    pub(crate) frame_count: usize,
    pub(crate) team_colored: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VehicleDeathProfile {
    pub(crate) effect_bounds: VehicleDeathEffectBounds,
    pub(crate) wreck: VehicleDeathWreckProfile,
    pub(crate) destroyed_asset: VehicleDestroyedAssetProfile,
    pub(crate) turrent: Option<VehicleTurrentProfile>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VehicleMissileFrameProfile {
    LightRocketBullet,
    MissileLauncherBullet,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VehicleDamageProfile {
    pub(crate) missile_visual: Option<DamageMissileVisual>,
    pub(crate) missile_frames: Option<VehicleMissileFrameProfile>,
    pub(crate) impact: Option<RocketImpactProfile>,
}

pub(crate) fn default_selection_size(vehicle: VehicleType) -> Vec2 {
    match vehicle {
        VehicleType::Jeep => jeep::default_selection_size(),
        VehicleType::Light => light::default_selection_size(),
        VehicleType::Medium => medium::default_selection_size(),
        VehicleType::Heavy => heavy::default_selection_size(),
        VehicleType::Apc => apc::default_selection_size(),
        VehicleType::MissileLauncher => missile_launcher::default_selection_size(),
        VehicleType::Crane => crane::default_selection_size(),
    }
}

pub(crate) fn movement_profile(vehicle: VehicleType) -> VehicleMovementProfile {
    match vehicle {
        VehicleType::Jeep => jeep::movement_profile(),
        VehicleType::Light => light::movement_profile(),
        VehicleType::Medium => medium::movement_profile(),
        VehicleType::Heavy => heavy::movement_profile(),
        VehicleType::Apc => apc::movement_profile(),
        VehicleType::MissileLauncher => missile_launcher::movement_profile(),
        VehicleType::Crane => VehicleMovementProfile {
            base_frame_count: 3,
            base_frame_time: VEHICLE_BASE_FRAME_TIME,
            track_kind: VehicleTrackKind::Tank,
            drops_damage_effects: true,
        },
    }
}

pub(crate) fn base_frame_count(vehicle: VehicleType) -> usize {
    movement_profile(vehicle).base_frame_count
}

pub(crate) fn settings(vehicle: VehicleType) -> UnitSettings {
    match vehicle {
        VehicleType::Jeep => jeep::settings(),
        VehicleType::Light => light::settings(),
        VehicleType::Medium => medium::settings(),
        VehicleType::Heavy => heavy::settings(),
        VehicleType::Apc => apc::settings(),
        VehicleType::MissileLauncher => missile_launcher::settings(),
        VehicleType::Crane => crane::settings(),
    }
}

pub(crate) fn attack_sound(vehicle: VehicleType) -> Option<UnitAttackSound> {
    match vehicle {
        VehicleType::Jeep => jeep::attack_sound(),
        VehicleType::Light => light::attack_sound(),
        VehicleType::Medium => medium::attack_sound(),
        VehicleType::Heavy => heavy::attack_sound(),
        VehicleType::Apc => apc::attack_sound(),
        VehicleType::MissileLauncher => missile_launcher::attack_sound(),
        VehicleType::Crane => crane::attack_sound(),
    }
}

pub(crate) fn damage_profile(vehicle: VehicleType) -> VehicleDamageProfile {
    match vehicle {
        VehicleType::Jeep => jeep::damage_profile(),
        VehicleType::Light => light::damage_profile(),
        VehicleType::Medium => medium::damage_profile(),
        VehicleType::Heavy => heavy::damage_profile(),
        VehicleType::Apc => apc::damage_profile(),
        VehicleType::MissileLauncher => missile_launcher::damage_profile(),
        VehicleType::Crane => VehicleDamageProfile {
            missile_visual: None,
            missile_frames: None,
            impact: None,
        },
    }
}

pub(crate) fn damage_missile_visual(vehicle: VehicleType) -> Option<DamageMissileVisual> {
    damage_profile(vehicle).missile_visual
}

pub(crate) fn damage_missile_frame_paths(vehicle: VehicleType) -> Option<Vec<String>> {
    match vehicle {
        VehicleType::Light => Some(light::damage_missile_frame_paths()),
        VehicleType::Medium => Some(medium::damage_missile_frame_paths()),
        VehicleType::Heavy => Some(heavy::damage_missile_frame_paths()),
        VehicleType::MissileLauncher => Some(missile_launcher::damage_missile_frame_paths()),
        VehicleType::Jeep | VehicleType::Apc | VehicleType::Crane => None,
    }
}

pub(crate) fn rocket_impact_profile(vehicle: VehicleType) -> Option<RocketImpactProfile> {
    damage_profile(vehicle).impact
}

pub(crate) fn rocket_muzzle_offset(direction: usize) -> Vec2 {
    const SX: [f32; 8] = [20.0, 12.0, 0.0, -12.0, -20.0, -12.0, 0.0, 12.0];
    const SY: [f32; 8] = [0.0, -12.0, -20.0, -12.0, 0.0, 12.0, 20.0, 12.0];
    let direction = direction.min(7);
    Vec2::new(1.0 + SX[direction], 2.0 - SY[direction])
}

pub(crate) fn death_profile(vehicle: VehicleType) -> Option<VehicleDeathProfile> {
    Some(match vehicle {
        VehicleType::Jeep => jeep::death_profile(),
        VehicleType::Light => light::death_profile(),
        VehicleType::Medium => medium::death_profile(),
        VehicleType::Heavy => heavy::death_profile(),
        VehicleType::Apc => apc::death_profile(),
        VehicleType::MissileLauncher => missile_launcher::death_profile(),
        VehicleType::Crane => VehicleDeathProfile {
            effect_bounds: crane::death_effect_bounds(),
            wreck: VehicleDeathWreckProfile::Static,
            destroyed_asset: VehicleDestroyedAssetProfile::TeamStatic,
            turrent: None,
        },
    })
}

pub(crate) fn death_effect_bounds(vehicle: VehicleType) -> Option<VehicleDeathEffectBounds> {
    death_profile(vehicle).map(|profile| profile.effect_bounds)
}

pub(crate) fn random_death_point(bounds: VehicleDeathEffectBounds, rng: &mut CombatRng) -> Vec2 {
    Vec2::new(
        (bounds.x + rng.index(bounds.width)) as f32,
        (bounds.y + rng.index(bounds.height)) as f32,
    )
}

pub(crate) fn death_standard_effect_shape(
    kind: VehicleDeathStandardKind,
    anchor_map: Vec2,
) -> Option<(Vec2, Vec<String>)> {
    let (prefix, offset) = match kind {
        VehicleDeathStandardKind::BigSmoke => ("big_smoke", Vec2::new(-16.0, -32.0)),
        VehicleDeathStandardKind::LittleFire => ("little_fire", Vec2::new(-4.0, -8.0)),
        VehicleDeathStandardKind::SmallFireSmoke => ("small_fire_smoke", Vec2::new(-8.0, -16.0)),
        VehicleDeathStandardKind::Fire => ("fire", Vec2::new(-4.0, -8.0)),
    };
    Some((
        anchor_map + offset,
        (0..4)
            .map(|frame| format!("units/vehicles/death_effects/{prefix}_n{frame:02}.png"))
            .collect(),
    ))
}

pub(crate) fn standard_effect_kind(rng: &mut CombatRng) -> VehicleDeathStandardKind {
    match rng.index(100) {
        0..=9 => VehicleDeathStandardKind::BigSmoke,
        10..=19 => VehicleDeathStandardKind::SmallFireSmoke,
        20..=49 => VehicleDeathStandardKind::Fire,
        _ => VehicleDeathStandardKind::LittleFire,
    }
}

pub(crate) fn death_wreck_asset_path(
    vehicle: VehicleType,
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> Option<String> {
    match vehicle {
        VehicleType::Jeep => Some(jeep::death_wreck_asset_path()),
        VehicleType::Light => light::death_wreck_asset_path(team, rotation, frame),
        VehicleType::Medium => medium::death_wreck_asset_path(team, rotation, frame),
        VehicleType::Heavy => heavy::death_wreck_asset_path(team, rotation, frame),
        VehicleType::Apc => Some(apc::death_wreck_asset_path()),
        VehicleType::MissileLauncher => Some(missile_launcher::death_wreck_asset_path()),
        VehicleType::Crane => Some(crane::death_wreck_asset_path()),
    }
}

pub(crate) fn destroyed_asset_path(vehicle: VehicleType, team: TeamType) -> Option<String> {
    match vehicle {
        VehicleType::Jeep => jeep::destroyed_asset_path(team),
        VehicleType::Light => light::destroyed_asset_path(team),
        VehicleType::Medium => medium::destroyed_asset_path(team),
        VehicleType::Heavy => heavy::destroyed_asset_path(team),
        VehicleType::Apc => apc::destroyed_asset_path(team),
        VehicleType::MissileLauncher => missile_launcher::destroyed_asset_path(team),
        VehicleType::Crane => crane::destroyed_asset_path(team),
    }
}

pub(crate) fn death_damaged_frame(rotation: u16, frame: usize) -> (u16, usize) {
    let frame = frame % 3;
    match rotation {
        0 => (0, frame),
        45 => (45, frame),
        90 => (90, frame),
        135 => (315, frame),
        180 => (0, 2 - frame),
        225 => (45, 2 - frame),
        270 => (90, 2 - frame),
        315 => (315, frame),
        _ => (0, frame),
    }
}

pub(crate) fn damaged_wreck_asset_path(
    vehicle: VehicleType,
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> Option<String> {
    if team == TeamType::Null {
        return None;
    }
    let folder = vehicle.folder();
    let team_name = team.atlas_team().asset_name();
    let (rotation, frame) = death_damaged_frame(rotation, frame);
    Some(format!(
        "units/vehicles/{folder}/base_damaged_{team_name}_r{rotation:03}_n{frame:02}.png"
    ))
}

pub(crate) fn death_top_left_world(center: Vec2) -> Vec2 {
    Vec2::new(center.x - 16.0, center.y + 16.0)
}

pub(crate) fn death_lifetime(rng: &mut CombatRng) -> f32 {
    5.0 + rng.index(3) as f32
}

pub(crate) fn death_spark_count(rng: &mut CombatRng) -> usize {
    40 + rng.index(30)
}

pub(crate) fn turrent_frame_paths(vehicle: VehicleType, team: TeamType) -> Option<Vec<String>> {
    match vehicle {
        VehicleType::Light => light::turrent_frame_paths(team),
        VehicleType::Medium => medium::turrent_frame_paths(team),
        VehicleType::Heavy => heavy::turrent_frame_paths(team),
        VehicleType::Jeep
        | VehicleType::Apc
        | VehicleType::MissileLauncher
        | VehicleType::Crane => None,
    }
}

pub(crate) fn turrent_profile(vehicle: VehicleType) -> Option<VehicleTurrentProfile> {
    death_profile(vehicle).and_then(|profile| profile.turrent)
}

pub(crate) fn turrent_start(top_left_map: Vec2) -> Vec2 {
    top_left_map + Vec2::splat(8.0)
}

pub(crate) fn turrent_target(center_map: Vec2, rng: &mut CombatRng) -> Vec2 {
    center_map
        + Vec2::new(
            TURRENT_MAX_DISTANCE - rng.index((TURRENT_MAX_DISTANCE * 2.0) as usize) as f32,
            TURRENT_MAX_DISTANCE - rng.index((TURRENT_MAX_DISTANCE * 2.0) as usize) as f32,
        )
}

pub(crate) fn turrent_flight_time(rng: &mut CombatRng) -> f32 {
    3.0 + rng.index(100) as f32 * 0.01
}

pub(crate) fn show_partially_damaged_effects(health_ratio: f32) -> bool {
    health_ratio < 0.7 && health_ratio > 0.4
}

pub(crate) fn show_heavily_damaged_effects(health_ratio: f32) -> bool {
    health_ratio < 0.4
}

pub(crate) fn damaged_speed_multiplier_for_ratio(health_ratio: f32) -> f32 {
    if show_heavily_damaged_effects(health_ratio) {
        DAMAGED_UNIT_SPEED
    } else if show_partially_damaged_effects(health_ratio) {
        PARTIALLY_DAMAGED_UNIT_SPEED
    } else {
        1.0
    }
}

pub(crate) fn run_speed_multiplier_for_ratio(health_ratio: f32, running: bool) -> f32 {
    if running && !show_heavily_damaged_effects(health_ratio) {
        RUN_UNIT_SPEED
    } else {
        1.0
    }
}

pub(crate) fn movement_speed_multiplier_for_ratio(health_ratio: f32, running: bool) -> f32 {
    damaged_speed_multiplier_for_ratio(health_ratio)
        * run_speed_multiplier_for_ratio(health_ratio, running)
}

pub(crate) fn tank_dirt_shape(planet: PlanetType) -> Option<(usize, usize)> {
    match planet {
        PlanetType::Desert | PlanetType::Volcanic | PlanetType::Arctic => Some((2, 5)),
        PlanetType::Jungle => Some((1, 6)),
        PlanetType::City => None,
    }
}

pub(crate) fn effect_drop_timer_ready(elapsed: &mut f32, delta_secs: f32) -> bool {
    *elapsed += delta_secs.max(0.0);
    if *elapsed < EFFECT_DROP_INTERVAL {
        return false;
    }

    *elapsed = 0.0;
    true
}

pub(crate) fn track_points(center: Vec2, direction: usize, rng: &mut CombatRng) -> [Vec2; 2] {
    let offsets = track_offsets(direction);
    let jitter = Vec2::new(rng.index(2) as f32, rng.index(2) as f32);
    [center + offsets[0] + jitter, center + offsets[1] + jitter]
}

pub(crate) fn track_offsets(direction: usize) -> [Vec2; 2] {
    match direction.min(7) {
        0 => [Vec2::new(-15.0, -2.0), Vec2::new(-15.0, 10.0)],
        1 => [Vec2::new(-14.0, 7.0), Vec2::new(-4.0, 17.0)],
        2 => [Vec2::new(-8.0, 15.0), Vec2::new(8.0, 15.0)],
        3 => [Vec2::new(15.0, 8.0), Vec2::new(5.0, 18.0)],
        4 => [Vec2::new(17.0, -2.0), Vec2::new(17.0, 10.0)],
        5 => [Vec2::new(13.0, 0.0), Vec2::new(3.0, -10.0)],
        6 => [Vec2::new(-8.0, -11.0), Vec2::new(8.0, -11.0)],
        7 => [Vec2::new(-14.0, -1.0), Vec2::new(-4.0, -11.0)],
        _ => unreachable!(),
    }
}

pub(crate) fn track_start_delay(rng: &mut CombatRng) -> f32 {
    rng.index(10) as f32 * 0.1
}

pub(crate) fn track_frame_for_delta(delta: f32) -> Option<usize> {
    if delta >= TRACK_FADE_END_TIME {
        None
    } else if delta >= TRACK_FADE_FRAME_2_TIME {
        Some(2)
    } else if delta >= TRACK_FADE_FRAME_1_TIME {
        Some(1)
    } else {
        Some(0)
    }
}

pub(crate) fn track_effect_frame_paths(
    vehicle: VehicleType,
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    match vehicle {
        VehicleType::Jeep => jeep::track_effect_frame_paths(planet, direction),
        VehicleType::Light => light::track_effect_frame_paths(planet, direction),
        VehicleType::Medium => medium::track_effect_frame_paths(planet, direction),
        VehicleType::Heavy => heavy::track_effect_frame_paths(planet, direction),
        VehicleType::Apc => apc::track_effect_frame_paths(planet, direction),
        VehicleType::MissileLauncher => {
            missile_launcher::track_effect_frame_paths(planet, direction)
        }
        VehicleType::Crane => tank_track_effect_frame_paths(planet, direction),
    }
}

pub(crate) fn jeep_track_effect_frame_paths(
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    if planet != PlanetType::Desert {
        return None;
    }

    track_effect_frame_paths_for_kind(VehicleTrackKind::Jeep, planet, direction)
}

pub(crate) fn tank_track_effect_frame_paths(
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    if planet == PlanetType::City {
        return None;
    }

    track_effect_frame_paths_for_kind(VehicleTrackKind::Tank, planet, direction)
}

fn track_effect_frame_paths_for_kind(
    track_kind: VehicleTrackKind,
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    if planet == PlanetType::City {
        return None;
    }

    let track_type = match track_kind {
        VehicleTrackKind::Jeep => "jeep",
        VehicleTrackKind::Tank => "tank",
    };
    let planet = planet_asset_name(planet);
    let rotation = rotation_for_direction(direction % 4);
    Some(
        (0..3)
            .map(|frame| {
                format!(
                    "units/vehicles/track_effects/{track_type}_track_{planet}_r{rotation:03}_n{frame:02}.png"
                )
            })
            .collect(),
    )
}

pub(crate) fn tank_dirt_frame_paths(
    planet: PlanetType,
    rng: &mut CombatRng,
) -> Option<Vec<String>> {
    let (variants, frame_count) = tank_dirt_shape(planet)?;
    let variant = rng.index(variants);
    let planet_name = planet_asset_name(planet);
    Some(
        (0..frame_count)
            .map(|frame| {
                format!(
                    "units/vehicles/tank_dirt/tank_dirt_{variant}_{planet_name}_n{frame:02}.png"
                )
            })
            .collect(),
    )
}

pub(crate) fn tank_smoke_frame_paths(direction: usize, spark_first: bool) -> Vec<String> {
    let rotation = rotation_for_direction(direction);
    (0..7)
        .map(|frame| {
            let prefix = if spark_first && frame < 4 {
                "track_spark"
            } else {
                "track_dust"
            };
            format!("units/vehicles/{prefix}_r{rotation:03}_n{frame:02}.png")
        })
        .collect()
}

pub(crate) fn tank_smoke_top_left(center: Vec2, direction: usize, rng: &mut CombatRng) -> Vec2 {
    const W: f32 = 16.0;
    const H: f32 = 12.0;
    match direction.min(7) {
        0 => center + Vec2::new(-15.0 - W, rng.index(8) as f32 - H),
        1 => {
            let shift = rng.index(7) as f32;
            center + Vec2::new(-13.0 + shift - W, 7.0 + shift)
        }
        2 => center + Vec2::new(-12.0 + rng.index(9) as f32, 15.0),
        3 => {
            let shift = rng.index(7) as f32;
            center + Vec2::new(13.0 - shift, 7.0 + shift)
        }
        4 => center + Vec2::new(15.0, rng.index(8) as f32 - H),
        5 => {
            let shift = rng.index(7) as f32;
            center + Vec2::new(6.0 + shift, -12.0 + shift - H)
        }
        6 => center + Vec2::new(-12.0 + rng.index(9) as f32, -15.0 - H),
        7 => {
            let shift = rng.index(7) as f32;
            center + Vec2::new(-6.0 - shift - W, -12.0 + shift - H)
        }
        _ => unreachable!(),
    }
}

pub(crate) fn tank_oil_frame_paths(rng: &mut CombatRng) -> Vec<String> {
    let variant = rng.index(3);
    (0..3)
        .map(|frame| format!("units/vehicles/tank_oil_{variant}_n{frame:02}.png"))
        .collect()
}

pub(crate) fn tank_oil_frame_time(rng: &mut CombatRng) -> f32 {
    TANK_OIL_FRAME_TIME + rng.index(10) as f32 * 0.1
}

pub(crate) fn tank_oil_offset(direction: usize, rng: &mut CombatRng) -> Vec2 {
    let base = match direction.min(7) {
        0 => Vec2::new(-8.0, 2.0),
        1 => Vec2::new(-7.0, 6.0),
        2 => Vec2::new(-3.0, 7.0),
        3 => Vec2::new(1.0, 6.0),
        4 => Vec2::new(2.0, 2.0),
        5 => Vec2::new(1.0, -2.0),
        6 => Vec2::new(-3.0, -3.0),
        7 => Vec2::new(-7.0, -2.0),
        _ => unreachable!(),
    };
    base + Vec2::splat(rng.index(7) as f32)
}

pub(crate) fn tank_spark_frame_paths() -> Vec<String> {
    (0..6)
        .map(|frame| format!("units/vehicles/ground_spark_n{frame:02}.png"))
        .collect()
}

pub(crate) fn tank_spark_lifetime_frames(rng: &mut CombatRng) -> usize {
    36 + rng.index(25)
}

pub(crate) fn tank_spark_offset(direction: usize, rng: &mut CombatRng) -> Vec2 {
    let base = match direction.min(7) {
        0 => Vec2::new(-10.0, -2.0),
        1 => Vec2::new(-9.0, 2.0),
        2 => Vec2::new(-5.0, 3.0),
        3 => Vec2::new(-1.0, 2.0),
        4 => Vec2::new(0.0, -2.0),
        5 => Vec2::new(-1.0, -6.0),
        6 => Vec2::new(-5.0, -7.0),
        7 => Vec2::new(-9.0, -6.0),
        _ => unreachable!(),
    };
    base + Vec2::splat(rng.index(11) as f32)
}

pub(crate) fn death_spark_frame_paths() -> Vec<String> {
    (0..6)
        .map(|frame| format!("units/vehicles/death_effects/spark_n{frame:02}.png"))
        .collect()
}

pub(crate) fn death_spark_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    -(rise / final_time) * (t * t) + rise * t
}

fn planet_asset_name(planet: PlanetType) -> &'static str {
    match planet {
        PlanetType::Desert => "desert",
        PlanetType::Volcanic => "volcanic",
        PlanetType::Arctic => "arctic",
        PlanetType::Jungle => "jungle",
        PlanetType::City => "city",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::DamageMissileVisual;

    #[test]
    fn movement_profiles_keep_vehicle_specific_animation_and_track_rules() {
        assert_eq!(base_frame_count(VehicleType::Jeep), 2);
        assert_eq!(base_frame_count(VehicleType::Light), 3);
        assert_eq!(
            movement_profile(VehicleType::Jeep).track_kind,
            VehicleTrackKind::Jeep
        );
        assert_eq!(
            movement_profile(VehicleType::Apc).track_kind,
            VehicleTrackKind::Tank
        );
        assert_eq!(
            movement_profile(VehicleType::MissileLauncher).base_frame_time,
            VEHICLE_BASE_FRAME_TIME
        );

        assert_eq!(
            track_effect_frame_paths(VehicleType::Jeep, PlanetType::Desert, 5)
                .unwrap()
                .last()
                .unwrap(),
            "units/vehicles/track_effects/jeep_track_desert_r045_n02.png"
        );
        assert!(track_effect_frame_paths(VehicleType::Jeep, PlanetType::Jungle, 0).is_none());
        assert_eq!(
            track_effect_frame_paths(VehicleType::MissileLauncher, PlanetType::Arctic, 6)
                .unwrap()
                .first()
                .unwrap(),
            "units/vehicles/track_effects/tank_track_arctic_r090_n00.png"
        );
        assert!(track_effect_frame_paths(VehicleType::Heavy, PlanetType::City, 0).is_none());
    }

    #[test]
    fn damage_profiles_expose_projectile_assets_and_impact_parameters() {
        assert_eq!(
            damage_profile(VehicleType::Light).missile_visual,
            Some(DamageMissileVisual::LightRocket {
                extra_small: 0,
                extra_large: 0,
                xx_large: 0,
            })
        );
        assert_eq!(
            damage_profile(VehicleType::Heavy).missile_visual,
            Some(DamageMissileVisual::LightRocket {
                extra_small: 0,
                extra_large: 1,
                xx_large: 1,
            })
        );
        assert_eq!(
            damage_profile(VehicleType::MissileLauncher).missile_frames,
            Some(VehicleMissileFrameProfile::MissileLauncherBullet)
        );
        assert_eq!(
            damage_missile_frame_paths(VehicleType::Heavy).unwrap(),
            vec!["units/vehicles/light/bullet.png".to_string()]
        );
        assert_eq!(
            damage_missile_frame_paths(VehicleType::MissileLauncher)
                .unwrap()
                .first()
                .unwrap(),
            "units/vehicles/missile_launcher/bullet.png"
        );

        assert_eq!(
            rocket_impact_profile(VehicleType::Medium)
                .unwrap()
                .large_mushrooms,
            1
        );
        assert_eq!(
            rocket_impact_profile(VehicleType::Heavy)
                .unwrap()
                .xx_large_mushrooms,
            1
        );
        assert_eq!(
            rocket_impact_profile(VehicleType::MissileLauncher)
                .unwrap()
                .unit_particle_amount,
            23
        );
        assert!(rocket_impact_profile(VehicleType::Apc).is_none());
    }

    #[test]
    fn death_profiles_keep_static_damaged_and_destroyed_assets_by_vehicle() {
        assert_eq!(
            death_profile(VehicleType::Jeep).unwrap().wreck,
            VehicleDeathWreckProfile::Static
        );
        assert_eq!(
            death_profile(VehicleType::Light).unwrap().wreck,
            VehicleDeathWreckProfile::DamagedFrames
        );
        assert_eq!(
            death_profile(VehicleType::Apc).unwrap().destroyed_asset,
            VehicleDestroyedAssetProfile::TeamStatic
        );
        assert_eq!(
            death_wreck_asset_path(VehicleType::Apc, TeamType::Blue, 180, 0).unwrap(),
            "units/vehicles/apc/wasted.png"
        );
        assert_eq!(
            death_wreck_asset_path(VehicleType::Heavy, TeamType::Green, 135, 4).unwrap(),
            "units/vehicles/heavy/base_damaged_green_r315_n01.png"
        );
        assert_eq!(
            destroyed_asset_path(VehicleType::Jeep, TeamType::Yellow).unwrap(),
            "units/vehicles/jeep/wasted.png"
        );
        assert_eq!(
            destroyed_asset_path(VehicleType::Apc, TeamType::Blue).unwrap(),
            "units/vehicles/apc/wasted_blue.png"
        );
        assert_eq!(
            destroyed_asset_path(VehicleType::MissileLauncher, TeamType::Null).unwrap(),
            "units/vehicles/missile_launcher/wasted_red.png"
        );
        assert!(destroyed_asset_path(VehicleType::Medium, TeamType::Red).is_none());
    }

    #[test]
    fn turrent_profiles_stay_with_tank_death_assets() {
        assert_eq!(
            turrent_profile(VehicleType::Light).unwrap(),
            VehicleTurrentProfile {
                frame_count: 8,
                team_colored: false,
            }
        );
        assert_eq!(
            turrent_profile(VehicleType::Heavy).unwrap(),
            VehicleTurrentProfile {
                frame_count: 8,
                team_colored: true,
            }
        );
        assert_eq!(
            turrent_start(Vec2::new(100.0, 50.0)),
            Vec2::new(108.0, 58.0)
        );
        assert_eq!(
            turrent_frame_paths(VehicleType::Medium, TeamType::Red)
                .unwrap()
                .last()
                .unwrap(),
            "units/vehicles/medium/top_pop_n07.png"
        );
        assert!(turrent_frame_paths(VehicleType::Heavy, TeamType::Null).is_none());
        assert!(turrent_profile(VehicleType::MissileLauncher).is_none());
    }

    #[test]
    fn vehicle_movement_speed_thresholds_are_exposed_for_main_wiring() {
        assert_eq!(damaged_speed_multiplier_for_ratio(0.7), 1.0);
        assert_eq!(
            damaged_speed_multiplier_for_ratio(0.69),
            PARTIALLY_DAMAGED_UNIT_SPEED
        );
        assert_eq!(damaged_speed_multiplier_for_ratio(0.4), 1.0);
        assert_eq!(damaged_speed_multiplier_for_ratio(0.39), DAMAGED_UNIT_SPEED);

        assert_eq!(run_speed_multiplier_for_ratio(0.69, true), RUN_UNIT_SPEED);
        assert_eq!(run_speed_multiplier_for_ratio(0.39, true), 1.0);
        assert_eq!(
            movement_speed_multiplier_for_ratio(0.69, true),
            PARTIALLY_DAMAGED_UNIT_SPEED * RUN_UNIT_SPEED
        );
    }
}
