use bevy::prelude::Vec2;

use crate::{
    components::CombatRng,
    original::{
        objects::VehicleType,
        types::{PlanetType, TeamType},
    },
    render::atlas::MobileSpriteRole,
    rotation_for_direction,
    units::vehicles::{
        VehicleDamageProfile, VehicleDeathEffectBounds, VehicleDeathProfile,
        VehicleDeathStandardKind, VehicleMovementProfile, VehicleTrackKind, VehicleTurrentProfile,
        apc_ui, crane_ui, heavy_ui, jeep_ui, light_ui, medium_ui, missile_launcher_ui,
    },
};

pub(crate) const TURRENT_MAX_DISTANCE: f32 = 300.0;
const TURRENT_RISE_BASE: f32 = 1.0;
const TURRENT_RISE_RANDOM_STEPS: usize = 300;
const TURRENT_RISE_STEP: f32 = 0.01;
pub(crate) const EFFECT_DROP_INTERVAL: f32 = 0.2;
pub(crate) const TANK_DUST_FRAME_TIME: f32 = 0.15;
pub(crate) const TANK_SPARK_FRAME_TIME: f32 = 0.1;
pub(crate) const DEATH_SPARK_FRAME_TIME: f32 = 0.1;
pub(crate) const VEHICLE_DEATH_STANDARD_FRAME_TIME: f32 = 0.15;
pub(crate) const VEHICLE_BASE_FRAME_TIME: f32 = 0.1;
pub(crate) const VEHICLE_SELECTION_RADIUS: f32 = 16.0;
pub(crate) const VEHICLE_FALLBACK_MARKER_SIZE: f32 = 10.0;
const TRACK_FADE_FRAME_1_TIME: f32 = 3.3;
const TRACK_FADE_FRAME_2_TIME: f32 = 3.6;
const TRACK_FADE_END_TIME: f32 = 3.9;
const TANK_OIL_FRAME_TIME: f32 = 3.0;
const DAMAGE_SMOKE_CHANCE: f32 = 1.0 / 3.0;
const DAMAGE_OIL_CHANCE: f32 = 1.0 / 16.0;
const DAMAGE_SPARK_CHANCE: f32 = 1.0 / 48.0;
pub(crate) const LID_FRAME_COUNT: usize = 3;
pub(crate) const TANK_DRIVER_FRAME_COUNT: usize = 2;
pub(crate) const LID_FRAME_SIZE: Vec2 = Vec2::new(8.0, 8.0);

pub(crate) fn source_dimensions() -> Vec2 {
    Vec2::splat(32.0)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VehicleAtlasFrameSpec {
    pub(crate) atlas_team: TeamType,
    pub(crate) frame_name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VehicleLidRenderOrder {
    LidBehindDriver,
    DriverBehindLid,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VehicleLidVisualPlacement {
    pub(crate) lid_top_left_offset: Vec2,
    pub(crate) driver_top_left_offset: Vec2,
    pub(crate) order: VehicleLidRenderOrder,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VehicleSelectionProfile {
    pub(crate) radius: f32,
    pub(crate) selection_size: Vec2,
    pub(crate) mobile: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VehicleDamageEffectPolicy {
    pub(crate) smoke_chance: f32,
    pub(crate) smoke_starts_with_spark: bool,
    pub(crate) oil_chance: f32,
    pub(crate) spark_chance: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VehicleDeathWreckVisual {
    pub(crate) wreck_path: String,
    pub(crate) top_left_world: Vec2,
    pub(crate) top_left_map: Vec2,
    pub(crate) lifetime: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VehicleDeathStandardEffectProfile {
    pub(crate) kind: VehicleDeathStandardKind,
    pub(crate) anchor_map: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VehicleDeathSparkTrajectory {
    pub(crate) start_map: Vec2,
    pub(crate) velocity_map: Vec2,
    pub(crate) lifetime: f32,
    pub(crate) rise: f32,
}

pub(crate) fn default_selection_size(vehicle: VehicleType) -> Vec2 {
    match vehicle {
        VehicleType::Jeep => jeep_ui::default_selection_size(),
        VehicleType::Light => light_ui::default_selection_size(),
        VehicleType::Medium => medium_ui::default_selection_size(),
        VehicleType::Heavy => heavy_ui::default_selection_size(),
        VehicleType::Apc => apc_ui::default_selection_size(),
        VehicleType::MissileLauncher => missile_launcher_ui::default_selection_size(),
        VehicleType::Crane => crane_ui::default_selection_size(),
    }
}

pub(crate) fn hud_name(vehicle: VehicleType) -> &'static str {
    match vehicle {
        VehicleType::Jeep => jeep_ui::hud_name(),
        VehicleType::Light => light_ui::hud_name(),
        VehicleType::Medium => medium_ui::hud_name(),
        VehicleType::Heavy => heavy_ui::hud_name(),
        VehicleType::Apc => apc_ui::hud_name(),
        VehicleType::MissileLauncher => missile_launcher_ui::hud_name(),
        VehicleType::Crane => crane_ui::hud_name(),
    }
}

pub(crate) fn selection_profile(selection_size: Vec2) -> VehicleSelectionProfile {
    VehicleSelectionProfile {
        radius: VEHICLE_SELECTION_RADIUS,
        selection_size,
        mobile: true,
    }
}

pub(crate) fn fallback_marker_size() -> Vec2 {
    Vec2::splat(VEHICLE_FALLBACK_MARKER_SIZE)
}

pub(crate) fn mobile_sprite_role(layer_index: usize) -> Option<MobileSpriteRole> {
    match layer_index {
        0 => Some(MobileSpriteRole::VehicleBase),
        1 => Some(MobileSpriteRole::VehicleTop),
        _ => None,
    }
}

pub(crate) fn direction_index_for_rotation(rotation: u16) -> usize {
    match rotation {
        0 => 0,
        45 => 1,
        90 => 2,
        135 => 3,
        180 => 4,
        225 => 5,
        270 => 6,
        315 => 7,
        _ => 0,
    }
}

pub(crate) fn lid_frame_path(rotation: u16, frame: usize) -> String {
    format!(
        "units/vehicles/tank_lid_r{rotation:03}_n{:02}.png",
        frame.min(LID_FRAME_COUNT - 1)
    )
}

pub(crate) fn lid_frame_paths() -> Vec<String> {
    (0..8)
        .flat_map(|direction| {
            let rotation = rotation_for_direction(direction);
            (0..LID_FRAME_COUNT).map(move |frame| lid_frame_path(rotation, frame))
        })
        .collect()
}

pub(crate) fn tank_driver_frame_path(team: TeamType, rotation: u16, frame: usize) -> String {
    let team_name = team.atlas_team().asset_name();
    format!(
        "units/robots/tank_fire_{team_name}_r{rotation:03}_n{:02}.png",
        frame.min(TANK_DRIVER_FRAME_COUNT - 1)
    )
}

pub(crate) fn tank_driver_frame_paths(team: TeamType) -> Vec<String> {
    (0..8)
        .flat_map(|direction| {
            let rotation = rotation_for_direction(direction);
            (0..TANK_DRIVER_FRAME_COUNT)
                .map(move |frame| tank_driver_frame_path(team, rotation, frame))
        })
        .collect()
}

pub(crate) fn tank_driver_frame_size(direction: usize) -> Vec2 {
    const HEIGHTS: [f32; 8] = [9.0, 12.0, 14.0, 12.0, 9.0, 11.0, 12.0, 11.0];
    Vec2::new(16.0, HEIGHTS[direction.min(7)])
}

pub(crate) fn lid_render_order(direction: usize) -> VehicleLidRenderOrder {
    if direction == 0 || direction > 3 {
        VehicleLidRenderOrder::LidBehindDriver
    } else {
        VehicleLidRenderOrder::DriverBehindLid
    }
}

pub(crate) fn tank_driver_offset_from_lid(direction: usize) -> Vec2 {
    const X: [f32; 8] = [3.0, -1.0, -3.0, -7.0, -10.0, -7.0, -4.0, 0.0];
    const Y: [f32; 8] = [0.0, -4.0, -6.0, -4.0, 0.0, 1.0, 1.0, 1.0];
    let direction = direction.min(7);
    Vec2::new(X[direction], Y[direction])
}

pub(crate) fn lid_visual_placement(
    vehicle: VehicleType,
    body_direction: usize,
    turrent_direction: usize,
) -> Option<VehicleLidVisualPlacement> {
    let lid_top_left_offset = match vehicle {
        VehicleType::Light => light_ui::lid_top_left_offset(body_direction, turrent_direction),
        VehicleType::Medium => medium_ui::lid_top_left_offset(body_direction, turrent_direction),
        VehicleType::Heavy => heavy_ui::lid_top_left_offset(body_direction, turrent_direction),
        VehicleType::Jeep
        | VehicleType::Apc
        | VehicleType::MissileLauncher
        | VehicleType::Crane => return None,
    };
    Some(VehicleLidVisualPlacement {
        lid_top_left_offset,
        driver_top_left_offset: lid_top_left_offset
            + tank_driver_offset_from_lid(turrent_direction),
        order: lid_render_order(turrent_direction),
    })
}

pub(crate) fn base_atlas_frame_spec(
    vehicle: VehicleType,
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> VehicleAtlasFrameSpec {
    match vehicle {
        VehicleType::Jeep => jeep_ui::base_atlas_frame_spec(team, rotation, frame),
        VehicleType::Light => light_ui::base_atlas_frame_spec(team, rotation, frame),
        VehicleType::Medium => medium_ui::base_atlas_frame_spec(team, rotation, frame),
        VehicleType::Heavy => heavy_ui::base_atlas_frame_spec(team, rotation, frame),
        VehicleType::Apc => apc_ui::base_atlas_frame_spec(team, rotation, frame),
        VehicleType::MissileLauncher => {
            missile_launcher_ui::base_atlas_frame_spec(team, rotation, frame)
        }
        VehicleType::Crane => crane_ui::base_atlas_frame_spec(team, rotation, frame),
    }
}

pub(crate) fn base_atlas_frame_spec_for_folder(
    folder: &str,
    base_frame_count: usize,
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> VehicleAtlasFrameSpec {
    let atlas_team = team.atlas_team();
    let team_name = atlas_team.asset_name();
    VehicleAtlasFrameSpec {
        atlas_team,
        frame_name: format!(
            "vehicle_{folder}_base_{team_name}_r{rotation:03}_n{:02}",
            frame % base_frame_count
        ),
    }
}

pub(crate) fn red_top_atlas_frame_spec(folder: &str, rotation: u16) -> VehicleAtlasFrameSpec {
    VehicleAtlasFrameSpec {
        atlas_team: TeamType::Red,
        frame_name: format!("vehicle_{folder}_top_r{rotation:03}"),
    }
}

pub(crate) fn team_top_atlas_frame_spec(
    folder: &str,
    team: TeamType,
    rotation: u16,
) -> VehicleAtlasFrameSpec {
    let atlas_team = team.atlas_team();
    let team_name = atlas_team.asset_name();
    VehicleAtlasFrameSpec {
        atlas_team,
        frame_name: format!("vehicle_{folder}_top_{team_name}_r{rotation:03}"),
    }
}

pub(crate) fn top_atlas_frame_spec(
    vehicle: VehicleType,
    team: TeamType,
    rotation: u16,
) -> Option<VehicleAtlasFrameSpec> {
    match vehicle {
        VehicleType::Jeep => jeep_ui::top_atlas_frame_spec(team, rotation),
        VehicleType::Light => light_ui::top_atlas_frame_spec(team, rotation),
        VehicleType::Medium => medium_ui::top_atlas_frame_spec(team, rotation),
        VehicleType::Heavy => heavy_ui::top_atlas_frame_spec(team, rotation),
        VehicleType::Apc => apc_ui::top_atlas_frame_spec(team, rotation),
        VehicleType::MissileLauncher => missile_launcher_ui::top_atlas_frame_spec(team, rotation),
        VehicleType::Crane => crane_ui::top_atlas_frame_spec(team, rotation),
    }
}

pub(crate) fn spawn_atlas_frame_specs(
    vehicle: VehicleType,
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> Vec<VehicleAtlasFrameSpec> {
    let mut specs = vec![base_atlas_frame_spec(vehicle, team, rotation, frame)];
    if let Some(top) = top_atlas_frame_spec(vehicle, team, rotation) {
        specs.push(top);
    }
    specs
}

pub(crate) fn mobile_atlas_frame_spec(
    vehicle: VehicleType,
    team: TeamType,
    role: MobileSpriteRole,
    rotation: u16,
    frame: usize,
    _moving: bool,
) -> Option<VehicleAtlasFrameSpec> {
    match role {
        MobileSpriteRole::VehicleBase => {
            Some(base_atlas_frame_spec(vehicle, team, rotation, frame))
        }
        MobileSpriteRole::VehicleTop => top_atlas_frame_spec(vehicle, team, rotation),
        MobileSpriteRole::Robot => None,
    }
}

pub(crate) fn movement_profile(vehicle: VehicleType) -> VehicleMovementProfile {
    match vehicle {
        VehicleType::Jeep => jeep_ui::movement_profile(),
        VehicleType::Light => light_ui::movement_profile(),
        VehicleType::Medium => medium_ui::movement_profile(),
        VehicleType::Heavy => heavy_ui::movement_profile(),
        VehicleType::Apc => apc_ui::movement_profile(),
        VehicleType::MissileLauncher => missile_launcher_ui::movement_profile(),
        VehicleType::Crane => crane_ui::movement_profile(),
    }
}

#[cfg(test)]
pub(crate) fn base_frame_time(vehicle: VehicleType) -> f32 {
    movement_profile(vehicle).base_frame_time
}

pub(crate) fn mobile_frame_count(vehicle: VehicleType, role: MobileSpriteRole) -> Option<usize> {
    match role {
        MobileSpriteRole::VehicleBase => Some(movement_profile(vehicle).base_frame_count),
        MobileSpriteRole::VehicleTop => Some(1),
        MobileSpriteRole::Robot => None,
    }
}

#[cfg(test)]
pub(crate) fn mobile_frame_time(vehicle: VehicleType, role: MobileSpriteRole) -> Option<f32> {
    match role {
        MobileSpriteRole::VehicleBase => Some(base_frame_time(vehicle)),
        MobileSpriteRole::VehicleTop => Some(0.0),
        MobileSpriteRole::Robot => None,
    }
}

pub(crate) fn drops_damage_effects(vehicle: VehicleType) -> bool {
    movement_profile(vehicle).drops_damage_effects
}

pub(crate) fn initial_effect_drop_timer_elapsed(vehicle: VehicleType) -> Option<f32> {
    drops_damage_effects(vehicle).then_some(EFFECT_DROP_INTERVAL)
}

pub(crate) fn damage_profile(vehicle: VehicleType) -> VehicleDamageProfile {
    match vehicle {
        VehicleType::Jeep => jeep_ui::damage_profile(),
        VehicleType::Light => light_ui::damage_profile(),
        VehicleType::Medium => medium_ui::damage_profile(),
        VehicleType::Heavy => heavy_ui::damage_profile(),
        VehicleType::Apc => apc_ui::damage_profile(),
        VehicleType::MissileLauncher => missile_launcher_ui::damage_profile(),
        VehicleType::Crane => crane_ui::damage_profile(),
    }
}

#[cfg(test)]
pub(crate) fn damage_missile_frame_paths(vehicle: VehicleType) -> Option<Vec<String>> {
    match vehicle {
        VehicleType::Light => Some(light_ui::damage_missile_frame_paths()),
        VehicleType::Medium => Some(medium_ui::damage_missile_frame_paths()),
        VehicleType::Heavy => Some(heavy_ui::damage_missile_frame_paths()),
        VehicleType::MissileLauncher => Some(missile_launcher_ui::damage_missile_frame_paths()),
        VehicleType::Jeep | VehicleType::Apc | VehicleType::Crane => None,
    }
}

pub(crate) fn rocket_muzzle_offset(direction: usize) -> Vec2 {
    const SX: [f32; 8] = [20.0, 12.0, 0.0, -12.0, -20.0, -12.0, 0.0, 12.0];
    const SY: [f32; 8] = [0.0, -12.0, -20.0, -12.0, 0.0, 12.0, 20.0, 12.0];
    let direction = direction.min(7);
    Vec2::new(1.0 + SX[direction], 2.0 - SY[direction])
}

pub(crate) fn death_profile(vehicle: VehicleType) -> Option<VehicleDeathProfile> {
    Some(match vehicle {
        VehicleType::Jeep => jeep_ui::death_profile(),
        VehicleType::Light => light_ui::death_profile(),
        VehicleType::Medium => medium_ui::death_profile(),
        VehicleType::Heavy => heavy_ui::death_profile(),
        VehicleType::Apc => apc_ui::death_profile(),
        VehicleType::MissileLauncher => missile_launcher_ui::death_profile(),
        VehicleType::Crane => crane_ui::death_profile(),
    })
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
        VehicleType::Jeep => Some(jeep_ui::death_wreck_asset_path()),
        VehicleType::Light => light_ui::death_wreck_asset_path(team, rotation, frame),
        VehicleType::Medium => medium_ui::death_wreck_asset_path(team, rotation, frame),
        VehicleType::Heavy => heavy_ui::death_wreck_asset_path(team, rotation, frame),
        VehicleType::Apc => Some(apc_ui::death_wreck_asset_path()),
        VehicleType::MissileLauncher => Some(missile_launcher_ui::death_wreck_asset_path()),
        VehicleType::Crane => Some(crane_ui::death_wreck_asset_path()),
    }
}

pub(crate) fn death_wreck_visual(
    vehicle: VehicleType,
    team: TeamType,
    center: Vec2,
    rotation: u16,
    frame: usize,
    rng: &mut CombatRng,
) -> Option<VehicleDeathWreckVisual> {
    let wreck_path = death_wreck_asset_path(vehicle, team, rotation, frame)?;
    let top_left_world = death_top_left_world(center);
    let top_left_map = death_top_left_map(center);
    Some(VehicleDeathWreckVisual {
        wreck_path,
        top_left_world,
        top_left_map,
        lifetime: death_lifetime(rng),
    })
}

pub(crate) fn destroyed_asset_path(vehicle: VehicleType, team: TeamType) -> Option<String> {
    match vehicle {
        VehicleType::Jeep => jeep_ui::destroyed_asset_path(team),
        VehicleType::Light => light_ui::destroyed_asset_path(team),
        VehicleType::Medium => medium_ui::destroyed_asset_path(team),
        VehicleType::Heavy => heavy_ui::destroyed_asset_path(team),
        VehicleType::Apc => apc_ui::destroyed_asset_path(team),
        VehicleType::MissileLauncher => missile_launcher_ui::destroyed_asset_path(team),
        VehicleType::Crane => crane_ui::destroyed_asset_path(team),
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

pub(crate) fn death_top_left_world(center: Vec2) -> Vec2 {
    Vec2::new(center.x - 16.0, center.y + 16.0)
}

pub(crate) fn death_top_left_map(center: Vec2) -> Vec2 {
    let top_left_world = death_top_left_world(center);
    Vec2::new(top_left_world.x, -top_left_world.y)
}

pub(crate) fn death_lifetime(rng: &mut CombatRng) -> f32 {
    5.0 + rng.index(3) as f32
}

pub(crate) fn death_spark_count(rng: &mut CombatRng) -> usize {
    40 + rng.index(30)
}

pub(crate) fn death_spark_center(top_left_map: Vec2) -> Vec2 {
    top_left_map + Vec2::splat(16.0)
}

pub(crate) fn death_spark_trajectory(
    map_center: Vec2,
    rng: &mut CombatRng,
) -> VehicleDeathSparkTrajectory {
    let lifetime = 1.5 + rng.index(3) as f32 * 0.1;
    let spark = map_center + Vec2::new(2.0 - rng.index(5) as f32, 2.0 - rng.index(5) as f32);
    let start_map = spark - Vec2::new(8.0, 5.0);
    let end_map = spark + Vec2::new(180.0 - rng.index(360) as f32, 150.0 - rng.index(220) as f32);
    VehicleDeathSparkTrajectory {
        start_map,
        velocity_map: (end_map - start_map) / lifetime,
        lifetime,
        rise: 3.0 + rng.index(300) as f32 * 0.01,
    }
}

pub(crate) fn turrent_frame_paths(vehicle: VehicleType, team: TeamType) -> Option<Vec<String>> {
    match vehicle {
        VehicleType::Light => light_ui::turrent_frame_paths(team),
        VehicleType::Medium => medium_ui::turrent_frame_paths(team),
        VehicleType::Heavy => heavy_ui::turrent_frame_paths(team),
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

pub(crate) fn turrent_rise(rng: &mut CombatRng) -> f32 {
    TURRENT_RISE_BASE + rng.index(TURRENT_RISE_RANDOM_STEPS) as f32 * TURRENT_RISE_STEP
}

pub(crate) fn death_standard_effects(
    vehicle: VehicleType,
    top_left_map: Vec2,
    rng: &mut CombatRng,
) -> Vec<VehicleDeathStandardEffectProfile> {
    let Some(bounds) = death_profile(vehicle).map(|profile| profile.effect_bounds) else {
        return Vec::new();
    };

    let mut effects = Vec::new();
    for _ in 0..(3 + rng.index(3)) {
        effects.push(VehicleDeathStandardEffectProfile {
            kind: VehicleDeathStandardKind::LittleFire,
            anchor_map: top_left_map + random_death_point(bounds, rng),
        });
    }
    for _ in 0..(1 + rng.index(2)) {
        effects.push(VehicleDeathStandardEffectProfile {
            kind: VehicleDeathStandardKind::BigSmoke,
            anchor_map: top_left_map + random_death_point(bounds, rng),
        });
    }
    for _ in 0..rng.index(2) {
        effects.push(VehicleDeathStandardEffectProfile {
            kind: VehicleDeathStandardKind::SmallFireSmoke,
            anchor_map: top_left_map + random_death_point(bounds, rng),
        });
    }
    effects
}

pub(crate) fn show_partially_damaged_effects(health_ratio: f32) -> bool {
    health_ratio < 0.7 && health_ratio > 0.4
}

pub(crate) fn show_heavily_damaged_effects(health_ratio: f32) -> bool {
    health_ratio < 0.4
}

pub(crate) fn damage_effect_policy(health_ratio: f32) -> VehicleDamageEffectPolicy {
    if show_heavily_damaged_effects(health_ratio) {
        VehicleDamageEffectPolicy {
            smoke_chance: DAMAGE_SMOKE_CHANCE,
            smoke_starts_with_spark: true,
            oil_chance: DAMAGE_OIL_CHANCE,
            spark_chance: DAMAGE_SPARK_CHANCE,
        }
    } else if show_partially_damaged_effects(health_ratio) {
        VehicleDamageEffectPolicy {
            smoke_chance: DAMAGE_SMOKE_CHANCE,
            smoke_starts_with_spark: false,
            oil_chance: 0.0,
            spark_chance: 0.0,
        }
    } else {
        VehicleDamageEffectPolicy {
            smoke_chance: 0.0,
            smoke_starts_with_spark: false,
            oil_chance: 0.0,
            spark_chance: 0.0,
        }
    }
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

pub(crate) fn should_lay_track(point_is_near_road: bool, roll: f32) -> bool {
    !point_is_near_road && roll < 0.8
}

pub(crate) fn track_road_check_points(point: Vec2) -> [Vec2; 5] {
    [
        point,
        point + Vec2::new(0.0, -16.0),
        point + Vec2::new(0.0, 16.0),
        point + Vec2::new(-16.0, 0.0),
        point + Vec2::new(16.0, 0.0),
    ]
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
        VehicleType::Jeep => jeep_ui::track_effect_frame_paths(planet, direction),
        VehicleType::Light => light_ui::track_effect_frame_paths(planet, direction),
        VehicleType::Medium => medium_ui::track_effect_frame_paths(planet, direction),
        VehicleType::Heavy => heavy_ui::track_effect_frame_paths(planet, direction),
        VehicleType::Apc => apc_ui::track_effect_frame_paths(planet, direction),
        VehicleType::MissileLauncher => {
            missile_launcher_ui::track_effect_frame_paths(planet, direction)
        }
        VehicleType::Crane => crane_ui::track_effect_frame_paths(planet, direction),
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
    use crate::{
        components::DamageMissileVisual,
        units::vehicles::{
            VEHICLE_BASE_FRAME_TIME, VehicleDeathWreckProfile, VehicleDestroyedAssetProfile,
            VehicleMissileFrameProfile, VehicleTrackKind, VehicleTurrentProfile,
        },
    };

    #[test]
    fn atlas_frame_specs_are_exposed_per_vehicle_for_spawn_and_mobile_rendering() {
        assert_eq!(
            base_atlas_frame_spec(VehicleType::Jeep, TeamType::Blue, 90, 3),
            VehicleAtlasFrameSpec {
                atlas_team: TeamType::Blue,
                frame_name: "vehicle_jeep_base_blue_r090_n01".to_string(),
            }
        );
        assert_eq!(
            top_atlas_frame_spec(VehicleType::Light, TeamType::Blue, 180),
            Some(VehicleAtlasFrameSpec {
                atlas_team: TeamType::Red,
                frame_name: "vehicle_light_top_r180".to_string(),
            })
        );
        assert_eq!(
            top_atlas_frame_spec(VehicleType::Heavy, TeamType::Green, 45),
            Some(VehicleAtlasFrameSpec {
                atlas_team: TeamType::Green,
                frame_name: "vehicle_heavy_top_green_r045".to_string(),
            })
        );
        assert_eq!(
            top_atlas_frame_spec(VehicleType::Apc, TeamType::Yellow, 315),
            Some(VehicleAtlasFrameSpec {
                atlas_team: TeamType::Red,
                frame_name: "vehicle_apc_top_r315".to_string(),
            })
        );
        assert!(top_atlas_frame_spec(VehicleType::Crane, TeamType::Red, 180).is_none());

        assert_eq!(
            spawn_atlas_frame_specs(VehicleType::MissileLauncher, TeamType::Null, 0, 4),
            vec![
                VehicleAtlasFrameSpec {
                    atlas_team: TeamType::Red,
                    frame_name: "vehicle_missile_launcher_base_red_r000_n01".to_string(),
                },
                VehicleAtlasFrameSpec {
                    atlas_team: TeamType::Red,
                    frame_name: "vehicle_missile_launcher_top_red_r000".to_string(),
                },
            ]
        );
        assert_eq!(
            mobile_atlas_frame_spec(
                VehicleType::Heavy,
                TeamType::Blue,
                MobileSpriteRole::VehicleTop,
                270,
                0,
                false,
            ),
            Some(VehicleAtlasFrameSpec {
                atlas_team: TeamType::Blue,
                frame_name: "vehicle_heavy_top_blue_r270".to_string(),
            })
        );
    }

    #[test]
    fn selection_and_mobile_render_constants_are_vehicle_side_policy() {
        let selection_size = Vec2::new(32.0, 24.0);
        assert_eq!(
            selection_profile(selection_size),
            VehicleSelectionProfile {
                radius: VEHICLE_SELECTION_RADIUS,
                selection_size,
                mobile: true,
            }
        );
        assert_eq!(
            fallback_marker_size(),
            Vec2::splat(VEHICLE_FALLBACK_MARKER_SIZE)
        );

        assert_eq!(mobile_sprite_role(0), Some(MobileSpriteRole::VehicleBase));
        assert_eq!(mobile_sprite_role(1), Some(MobileSpriteRole::VehicleTop));
        assert_eq!(mobile_sprite_role(2), None);
        assert_eq!(
            mobile_frame_count(VehicleType::Jeep, MobileSpriteRole::VehicleBase),
            Some(2)
        );
        assert_eq!(
            mobile_frame_count(VehicleType::Apc, MobileSpriteRole::VehicleTop),
            Some(1)
        );
        assert_eq!(
            mobile_frame_time(VehicleType::MissileLauncher, MobileSpriteRole::VehicleBase),
            Some(VEHICLE_BASE_FRAME_TIME)
        );
        assert_eq!(
            mobile_frame_time(VehicleType::Light, MobileSpriteRole::VehicleTop),
            Some(0.0)
        );
        assert_eq!(
            mobile_frame_time(VehicleType::Light, MobileSpriteRole::Robot),
            None
        );
    }

    #[test]
    fn movement_profiles_keep_vehicle_specific_animation_and_track_rules() {
        assert_eq!(movement_profile(VehicleType::Jeep).base_frame_count, 2);
        assert_eq!(movement_profile(VehicleType::Light).base_frame_count, 3);
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
        assert_eq!(base_frame_time(VehicleType::Heavy), VEHICLE_BASE_FRAME_TIME);
        assert!(drops_damage_effects(VehicleType::Crane));
        assert_eq!(
            initial_effect_drop_timer_elapsed(VehicleType::Jeep),
            Some(EFFECT_DROP_INTERVAL)
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
    fn track_policy_matches_original_road_and_roll_gate() {
        assert!(!should_lay_track(true, 0.0));
        assert!(should_lay_track(false, 0.79));
        assert!(!should_lay_track(false, 0.8));
        assert_eq!(
            track_road_check_points(Vec2::new(32.0, 48.0)),
            [
                Vec2::new(32.0, 48.0),
                Vec2::new(32.0, 32.0),
                Vec2::new(32.0, 64.0),
                Vec2::new(16.0, 48.0),
                Vec2::new(48.0, 48.0),
            ]
        );
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
            damage_profile(VehicleType::Medium)
                .impact
                .unwrap()
                .large_mushrooms,
            1
        );
        assert_eq!(
            damage_profile(VehicleType::Heavy)
                .impact
                .unwrap()
                .xx_large_mushrooms,
            1
        );
        assert_eq!(
            damage_profile(VehicleType::MissileLauncher)
                .impact
                .unwrap()
                .unit_particle_amount,
            23
        );
        assert!(damage_profile(VehicleType::Apc).impact.is_none());
        assert!(damage_profile(VehicleType::Crane).missile_visual.is_none());
    }

    #[test]
    fn damage_effect_policy_keeps_strict_original_thresholds_and_probabilities() {
        assert_eq!(
            damage_effect_policy(0.7),
            VehicleDamageEffectPolicy {
                smoke_chance: 0.0,
                smoke_starts_with_spark: false,
                oil_chance: 0.0,
                spark_chance: 0.0,
            }
        );
        assert_eq!(
            damage_effect_policy(0.69),
            VehicleDamageEffectPolicy {
                smoke_chance: 1.0 / 3.0,
                smoke_starts_with_spark: false,
                oil_chance: 0.0,
                spark_chance: 0.0,
            }
        );
        assert_eq!(
            damage_effect_policy(0.4),
            VehicleDamageEffectPolicy {
                smoke_chance: 0.0,
                smoke_starts_with_spark: false,
                oil_chance: 0.0,
                spark_chance: 0.0,
            }
        );
        assert_eq!(
            damage_effect_policy(0.39),
            VehicleDamageEffectPolicy {
                smoke_chance: 1.0 / 3.0,
                smoke_starts_with_spark: true,
                oil_chance: 1.0 / 16.0,
                spark_chance: 1.0 / 48.0,
            }
        );
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
            death_profile(VehicleType::Crane).unwrap().destroyed_asset,
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
                damage: 40,
                radius: 40,
            }
        );
        assert_eq!(
            turrent_profile(VehicleType::Heavy).unwrap(),
            VehicleTurrentProfile {
                frame_count: 8,
                team_colored: true,
                damage: 40,
                radius: 40,
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
    fn vehicle_lid_visual_assets_match_original_shared_files() {
        let lid_paths = lid_frame_paths();
        assert_eq!(lid_paths.len(), 8 * LID_FRAME_COUNT);
        assert_eq!(
            lid_paths.first().unwrap(),
            "units/vehicles/tank_lid_r000_n00.png"
        );
        assert_eq!(
            lid_paths.last().unwrap(),
            "units/vehicles/tank_lid_r315_n02.png"
        );

        let driver_paths = tank_driver_frame_paths(TeamType::Blue);
        assert_eq!(driver_paths.len(), 8 * TANK_DRIVER_FRAME_COUNT);
        assert_eq!(
            driver_paths.first().unwrap(),
            "units/robots/tank_fire_blue_r000_n00.png"
        );
        assert_eq!(
            driver_paths.last().unwrap(),
            "units/robots/tank_fire_blue_r315_n01.png"
        );
    }

    #[test]
    fn vehicle_lid_offsets_match_original_render_tables() {
        assert_eq!(
            lid_visual_placement(VehicleType::Light, 0, 0)
                .unwrap()
                .lid_top_left_offset,
            Vec2::new(13.0, 3.0)
        );
        assert_eq!(
            lid_visual_placement(VehicleType::Light, 2, 3)
                .unwrap()
                .lid_top_left_offset,
            Vec2::new(10.0, 4.0)
        );
        assert_eq!(
            lid_visual_placement(VehicleType::Medium, 2, 7)
                .unwrap()
                .lid_top_left_offset,
            Vec2::new(11.0, 0.0)
        );
        assert_eq!(
            lid_visual_placement(VehicleType::Heavy, 4, 5)
                .unwrap()
                .lid_top_left_offset,
            Vec2::new(15.0, 2.0)
        );
        assert!(lid_visual_placement(VehicleType::Jeep, 0, 0).is_none());
    }

    #[test]
    fn vehicle_lid_driver_order_matches_original_direction_rule() {
        assert_eq!(lid_render_order(0), VehicleLidRenderOrder::LidBehindDriver);
        assert_eq!(lid_render_order(1), VehicleLidRenderOrder::DriverBehindLid);
        assert_eq!(lid_render_order(3), VehicleLidRenderOrder::DriverBehindLid);
        assert_eq!(lid_render_order(4), VehicleLidRenderOrder::LidBehindDriver);
        assert_eq!(tank_driver_offset_from_lid(2), Vec2::new(-3.0, -6.0));
        assert_eq!(tank_driver_frame_size(6), Vec2::new(16.0, 12.0));
    }
}
