use bevy::prelude::Vec2;

use crate::{
    components::{CombatRng, DamageMissileVisual, VehicleLidState},
    original::{
        objects::VehicleType,
        types::{PlanetType, TeamType},
    },
    render::atlas::MobileSpriteRole,
    units::{RocketImpactProfile, UnitAttackSound, UnitSettings},
};

pub(crate) const PARTIALLY_DAMAGED_UNIT_SPEED: f32 = 0.9;
pub(crate) const DAMAGED_UNIT_SPEED: f32 = 0.8;
pub(crate) const LID_FRAME_TIME: f32 = 0.2;
pub(crate) const LID_MAX_FRAME: usize = 2;

pub(crate) mod vehicle_ui;

#[path = "apc/apc_mod.rs"]
pub(crate) mod apc;
#[path = "crane/crane_mod.rs"]
pub(crate) mod crane;
#[path = "heavy/heavy_mod.rs"]
pub(crate) mod heavy;
#[path = "jeep/jeep_mod.rs"]
pub(crate) mod jeep;
#[path = "light/light_mod.rs"]
pub(crate) mod light;
#[path = "medium/medium_mod.rs"]
pub(crate) mod medium;
#[path = "missile_launcher/missile_launcher_mod.rs"]
pub(crate) mod missile_launcher;

pub(crate) mod apc_ui {
    pub(crate) use super::apc::apc_ui::*;
}

pub(crate) mod crane_ui {
    pub(crate) use super::crane::crane_ui::*;
}

pub(crate) mod heavy_ui {
    pub(crate) use super::heavy::heavy_ui::*;
}

pub(crate) mod jeep_ui {
    pub(crate) use super::jeep::jeep_ui::*;
}

pub(crate) mod light_ui {
    pub(crate) use super::light::light_ui::*;
}

pub(crate) mod medium_ui {
    pub(crate) use super::medium::medium_ui::*;
}

pub(crate) mod missile_launcher_ui {
    pub(crate) use super::missile_launcher::missile_launcher_ui::*;
}

#[allow(unused_imports)]
pub(crate) use vehicle_ui::EFFECT_DROP_INTERVAL;
pub(crate) use vehicle_ui::{
    DEATH_SPARK_FRAME_TIME, LID_FRAME_COUNT, LID_FRAME_SIZE, TANK_DRIVER_FRAME_COUNT,
    TANK_DUST_FRAME_TIME, TANK_SPARK_FRAME_TIME, VEHICLE_BASE_FRAME_TIME,
    VEHICLE_DEATH_STANDARD_FRAME_TIME, VehicleAtlasFrameSpec, VehicleDamageEffectPolicy,
    VehicleDeathSparkTrajectory, VehicleDeathStandardEffectProfile, VehicleDeathWreckVisual,
    VehicleLidRenderOrder, VehicleLidVisualPlacement, VehicleSelectionProfile,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VehicleTrackKind {
    Jeep,
    Tank,
}

pub(crate) fn source_dimensions(_vehicle: VehicleType) -> Vec2 {
    vehicle_ui::source_dimensions()
}

pub(crate) fn requires_activation(vehicle: VehicleType) -> bool {
    match vehicle {
        VehicleType::Jeep => jeep::REQUIRES_ACTIVATION,
        VehicleType::Light => light::REQUIRES_ACTIVATION,
        VehicleType::Medium => medium::REQUIRES_ACTIVATION,
        VehicleType::Heavy => heavy::REQUIRES_ACTIVATION,
        VehicleType::Apc => apc::REQUIRES_ACTIVATION,
        VehicleType::MissileLauncher => missile_launcher::REQUIRES_ACTIVATION,
        VehicleType::Crane => crane::REQUIRES_ACTIVATION,
    }
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
    pub(crate) damage: i32,
    pub(crate) radius: i32,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VehicleLidSignal {
    None,
    Open,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VehicleDamageProfile {
    pub(crate) missile_visual: Option<DamageMissileVisual>,
    pub(crate) missile_frames: Option<VehicleMissileFrameProfile>,
    pub(crate) impact: Option<RocketImpactProfile>,
}

pub(crate) fn has_lid(vehicle: VehicleType) -> bool {
    matches!(
        vehicle,
        VehicleType::Light | VehicleType::Medium | VehicleType::Heavy
    )
}

pub(crate) fn lid_open_signal_applies(roll: usize) -> bool {
    roll % 5 != 0
}

pub(crate) fn lid_close_delay(roll: usize) -> f32 {
    0.1 * (roll % 15) as f32
}

pub(crate) fn lid_signal_for_attack_target(
    previous_attack_target_ref: Option<u32>,
    current_attack_target_ref: Option<u32>,
    current_target_can_snipe: bool,
) -> VehicleLidSignal {
    if previous_attack_target_ref == current_attack_target_ref {
        return VehicleLidSignal::None;
    }

    match current_attack_target_ref {
        Some(_) if current_target_can_snipe => VehicleLidSignal::Open,
        None if previous_attack_target_ref.is_some() => VehicleLidSignal::Close,
        _ => VehicleLidSignal::None,
    }
}

pub(crate) fn signal_lid_should_open(state: &mut VehicleLidState, roll: usize) -> bool {
    if !lid_open_signal_applies(roll) {
        return false;
    }
    state.open = true;
    state.closing = false;
    state.close_delay = 0.0;
    true
}

pub(crate) fn signal_lid_should_close(state: &mut VehicleLidState, roll: usize) {
    if state.open && !state.closing {
        state.closing = true;
        state.close_delay = lid_close_delay(roll);
    }
}

pub(crate) fn process_server_lid(state: &mut VehicleLidState, delta_secs: f32) -> Option<bool> {
    if !state.closing {
        return None;
    }

    state.close_delay -= delta_secs.max(0.0);
    if state.close_delay <= f32::EPSILON {
        state.open = false;
        state.closing = false;
        state.close_delay = 0.0;
        return Some(state.open);
    }
    None
}

pub(crate) fn set_lid_state(state: &mut VehicleLidState, open: bool) {
    state.open = open;
    if open {
        state.closing = false;
        state.close_delay = 0.0;
    }
}

pub(crate) fn process_lid_animation(state: &mut VehicleLidState, delta_secs: f32) {
    state.elapsed += delta_secs.max(0.0);
    while state.elapsed >= LID_FRAME_TIME {
        state.elapsed -= LID_FRAME_TIME;
        if state.open {
            if state.frame >= LID_MAX_FRAME {
                state.show_driver = true;
            } else {
                state.frame += 1;
            }
        } else {
            state.show_driver = false;
            if state.frame > 0 {
                state.frame -= 1;
            }
        }
    }
}

pub(crate) fn default_selection_size(vehicle: VehicleType) -> Vec2 {
    vehicle_ui::default_selection_size(vehicle)
}

pub(crate) fn hud_name(vehicle: VehicleType) -> &'static str {
    vehicle_ui::hud_name(vehicle)
}

pub(crate) fn selection_profile(selection_size: Vec2) -> VehicleSelectionProfile {
    vehicle_ui::selection_profile(selection_size)
}

pub(crate) fn fallback_marker_size() -> Vec2 {
    vehicle_ui::fallback_marker_size()
}

pub(crate) fn mobile_sprite_role(layer_index: usize) -> Option<MobileSpriteRole> {
    vehicle_ui::mobile_sprite_role(layer_index)
}

pub(crate) fn mobile_frame_count(vehicle: VehicleType, role: MobileSpriteRole) -> Option<usize> {
    vehicle_ui::mobile_frame_count(vehicle, role)
}

pub(crate) fn spawn_atlas_frame_specs(
    vehicle: VehicleType,
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> Vec<VehicleAtlasFrameSpec> {
    vehicle_ui::spawn_atlas_frame_specs(vehicle, team, rotation, frame)
}

pub(crate) fn direction_index_for_rotation(rotation: u16) -> usize {
    vehicle_ui::direction_index_for_rotation(rotation)
}

pub(crate) fn lid_frame_paths() -> Vec<String> {
    vehicle_ui::lid_frame_paths()
}

pub(crate) fn tank_driver_frame_paths(team: TeamType) -> Vec<String> {
    vehicle_ui::tank_driver_frame_paths(team)
}

pub(crate) fn tank_driver_frame_size(direction: usize) -> Vec2 {
    vehicle_ui::tank_driver_frame_size(direction)
}

pub(crate) fn lid_visual_placement(
    vehicle: VehicleType,
    body_direction: usize,
    turrent_direction: usize,
) -> Option<VehicleLidVisualPlacement> {
    vehicle_ui::lid_visual_placement(vehicle, body_direction, turrent_direction)
}

pub(crate) fn mobile_atlas_frame_spec(
    vehicle: VehicleType,
    team: TeamType,
    role: MobileSpriteRole,
    rotation: u16,
    frame: usize,
    moving: bool,
) -> Option<VehicleAtlasFrameSpec> {
    vehicle_ui::mobile_atlas_frame_spec(vehicle, team, role, rotation, frame, moving)
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

pub(crate) fn enter_removes_group_member(
    target_kind: crate::original::objects::ObjectKind,
    entrant_ref_id: u32,
    member_ref_id: u32,
    member_leader_ref_id: u32,
    member_destroyed: bool,
    member_health: f32,
) -> bool {
    apc::enter_removes_group_member(
        target_kind,
        entrant_ref_id,
        member_ref_id,
        member_leader_ref_id,
        member_destroyed,
        member_health,
    )
}

pub(crate) fn apply_apc_driver_attack_stats(
    stats: &mut crate::components::ObjectStats,
    robot_kind: crate::original::objects::RobotType,
) {
    apc::apply_driver_attack_stats(stats, robot_kind)
}

pub(crate) fn damage_profile(vehicle: VehicleType) -> VehicleDamageProfile {
    vehicle_ui::damage_profile(vehicle)
}

pub(crate) fn initial_effect_drop_timer_elapsed(vehicle: VehicleType) -> Option<f32> {
    vehicle_ui::initial_effect_drop_timer_elapsed(vehicle)
}

pub(crate) fn damage_missile_visual(vehicle: VehicleType) -> Option<DamageMissileVisual> {
    damage_profile(vehicle).missile_visual
}

pub(crate) fn rocket_muzzle_offset(direction: usize) -> Vec2 {
    vehicle_ui::rocket_muzzle_offset(direction)
}

#[cfg(test)]
pub(crate) fn death_profile(vehicle: VehicleType) -> Option<VehicleDeathProfile> {
    vehicle_ui::death_profile(vehicle)
}

#[cfg(test)]
pub(crate) fn death_effect_bounds(vehicle: VehicleType) -> Option<VehicleDeathEffectBounds> {
    death_profile(vehicle).map(|profile| profile.effect_bounds)
}

pub(crate) fn death_standard_effect_shape(
    kind: VehicleDeathStandardKind,
    anchor_map: Vec2,
) -> Option<(Vec2, Vec<String>)> {
    vehicle_ui::death_standard_effect_shape(kind, anchor_map)
}

pub(crate) fn standard_effect_kind(rng: &mut CombatRng) -> VehicleDeathStandardKind {
    vehicle_ui::standard_effect_kind(rng)
}

#[cfg(test)]
pub(crate) fn death_wreck_asset_path(
    vehicle: VehicleType,
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> Option<String> {
    vehicle_ui::death_wreck_asset_path(vehicle, team, rotation, frame)
}

pub(crate) fn death_wreck_visual(
    vehicle: VehicleType,
    team: TeamType,
    center: Vec2,
    rotation: u16,
    frame: usize,
    rng: &mut CombatRng,
) -> Option<VehicleDeathWreckVisual> {
    vehicle_ui::death_wreck_visual(vehicle, team, center, rotation, frame, rng)
}

pub(crate) fn destroyed_asset_path(vehicle: VehicleType, team: TeamType) -> Option<String> {
    vehicle_ui::destroyed_asset_path(vehicle, team)
}

pub(crate) fn death_damaged_frame(rotation: u16, frame: usize) -> (u16, usize) {
    vehicle_ui::death_damaged_frame(rotation, frame)
}

#[cfg(test)]
pub(crate) fn death_top_left_world(center: Vec2) -> Vec2 {
    vehicle_ui::death_top_left_world(center)
}

pub(crate) fn death_top_left_map(center: Vec2) -> Vec2 {
    vehicle_ui::death_top_left_map(center)
}

#[cfg(test)]
pub(crate) fn death_lifetime(rng: &mut CombatRng) -> f32 {
    vehicle_ui::death_lifetime(rng)
}

pub(crate) fn death_spark_count(rng: &mut CombatRng) -> usize {
    vehicle_ui::death_spark_count(rng)
}

pub(crate) fn death_spark_center(top_left_map: Vec2) -> Vec2 {
    vehicle_ui::death_spark_center(top_left_map)
}

pub(crate) fn death_spark_trajectory(
    map_center: Vec2,
    rng: &mut CombatRng,
) -> VehicleDeathSparkTrajectory {
    vehicle_ui::death_spark_trajectory(map_center, rng)
}

pub(crate) fn turrent_frame_paths(vehicle: VehicleType, team: TeamType) -> Option<Vec<String>> {
    vehicle_ui::turrent_frame_paths(vehicle, team)
}

pub(crate) fn turrent_profile(vehicle: VehicleType) -> Option<VehicleTurrentProfile> {
    vehicle_ui::turrent_profile(vehicle)
}

pub(crate) fn turrent_start(top_left_map: Vec2) -> Vec2 {
    vehicle_ui::turrent_start(top_left_map)
}

pub(crate) fn turrent_target(center_map: Vec2, rng: &mut CombatRng) -> Vec2 {
    vehicle_ui::turrent_target(center_map, rng)
}

pub(crate) fn turrent_flight_time(rng: &mut CombatRng) -> f32 {
    vehicle_ui::turrent_flight_time(rng)
}

pub(crate) fn turrent_rise(rng: &mut CombatRng) -> f32 {
    vehicle_ui::turrent_rise(rng)
}

pub(crate) fn death_standard_effects(
    vehicle: VehicleType,
    top_left_map: Vec2,
    rng: &mut CombatRng,
) -> Vec<VehicleDeathStandardEffectProfile> {
    vehicle_ui::death_standard_effects(vehicle, top_left_map, rng)
}

pub(crate) fn show_partially_damaged_effects(health_ratio: f32) -> bool {
    vehicle_ui::show_partially_damaged_effects(health_ratio)
}

pub(crate) fn show_heavily_damaged_effects(health_ratio: f32) -> bool {
    vehicle_ui::show_heavily_damaged_effects(health_ratio)
}

pub(crate) fn damage_effect_policy(health_ratio: f32) -> VehicleDamageEffectPolicy {
    vehicle_ui::damage_effect_policy(health_ratio)
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
        crate::units::unit_behavior::RUN_UNIT_SPEED
    } else {
        1.0
    }
}

#[cfg(test)]
pub(crate) fn movement_speed_multiplier_for_ratio(health_ratio: f32, running: bool) -> f32 {
    damaged_speed_multiplier_for_ratio(health_ratio)
        * run_speed_multiplier_for_ratio(health_ratio, running)
}

#[cfg(test)]
pub(crate) fn tank_dirt_shape(planet: PlanetType) -> Option<(usize, usize)> {
    vehicle_ui::tank_dirt_shape(planet)
}

pub(crate) fn effect_drop_timer_ready(elapsed: &mut f32, delta_secs: f32) -> bool {
    vehicle_ui::effect_drop_timer_ready(elapsed, delta_secs)
}

pub(crate) fn track_points(center: Vec2, direction: usize, rng: &mut CombatRng) -> [Vec2; 2] {
    vehicle_ui::track_points(center, direction, rng)
}

pub(crate) fn should_lay_track(point_is_near_road: bool, roll: f32) -> bool {
    vehicle_ui::should_lay_track(point_is_near_road, roll)
}

pub(crate) fn track_road_check_points(point: Vec2) -> [Vec2; 5] {
    vehicle_ui::track_road_check_points(point)
}

#[cfg(test)]
pub(crate) fn track_offsets(direction: usize) -> [Vec2; 2] {
    vehicle_ui::track_offsets(direction)
}

pub(crate) fn track_start_delay(rng: &mut CombatRng) -> f32 {
    vehicle_ui::track_start_delay(rng)
}

pub(crate) fn track_frame_for_delta(delta: f32) -> Option<usize> {
    vehicle_ui::track_frame_for_delta(delta)
}

pub(crate) fn track_effect_frame_paths(
    vehicle: VehicleType,
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    vehicle_ui::track_effect_frame_paths(vehicle, planet, direction)
}

pub(crate) fn jeep_track_effect_frame_paths(
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    vehicle_ui::jeep_track_effect_frame_paths(planet, direction)
}

pub(crate) fn tank_track_effect_frame_paths(
    planet: PlanetType,
    direction: usize,
) -> Option<Vec<String>> {
    vehicle_ui::tank_track_effect_frame_paths(planet, direction)
}

pub(crate) fn tank_dirt_frame_paths(
    planet: PlanetType,
    rng: &mut CombatRng,
) -> Option<Vec<String>> {
    vehicle_ui::tank_dirt_frame_paths(planet, rng)
}

pub(crate) fn tank_smoke_frame_paths(direction: usize, spark_first: bool) -> Vec<String> {
    vehicle_ui::tank_smoke_frame_paths(direction, spark_first)
}

pub(crate) fn tank_smoke_top_left(center: Vec2, direction: usize, rng: &mut CombatRng) -> Vec2 {
    vehicle_ui::tank_smoke_top_left(center, direction, rng)
}

pub(crate) fn tank_oil_frame_paths(rng: &mut CombatRng) -> Vec<String> {
    vehicle_ui::tank_oil_frame_paths(rng)
}

pub(crate) fn tank_oil_frame_time(rng: &mut CombatRng) -> f32 {
    vehicle_ui::tank_oil_frame_time(rng)
}

pub(crate) fn tank_oil_offset(direction: usize, rng: &mut CombatRng) -> Vec2 {
    vehicle_ui::tank_oil_offset(direction, rng)
}

pub(crate) fn tank_spark_frame_paths() -> Vec<String> {
    vehicle_ui::tank_spark_frame_paths()
}

pub(crate) fn tank_spark_lifetime_frames(rng: &mut CombatRng) -> usize {
    vehicle_ui::tank_spark_lifetime_frames(rng)
}

pub(crate) fn tank_spark_offset(direction: usize, rng: &mut CombatRng) -> Vec2 {
    vehicle_ui::tank_spark_offset(direction, rng)
}

pub(crate) fn death_spark_frame_paths() -> Vec<String> {
    vehicle_ui::death_spark_frame_paths()
}

pub(crate) fn death_spark_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    vehicle_ui::death_spark_arc_size(rise, final_time, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vehicle_movement_speed_thresholds_are_exposed_for_main_wiring() {
        assert_eq!(damaged_speed_multiplier_for_ratio(0.7), 1.0);
        assert_eq!(
            damaged_speed_multiplier_for_ratio(0.69),
            PARTIALLY_DAMAGED_UNIT_SPEED
        );
        assert_eq!(damaged_speed_multiplier_for_ratio(0.4), 1.0);
        assert_eq!(damaged_speed_multiplier_for_ratio(0.39), DAMAGED_UNIT_SPEED);

        assert_eq!(
            run_speed_multiplier_for_ratio(0.69, true),
            crate::units::unit_behavior::RUN_UNIT_SPEED
        );
        assert_eq!(run_speed_multiplier_for_ratio(0.39, true), 1.0);
        assert_eq!(
            movement_speed_multiplier_for_ratio(0.69, true),
            PARTIALLY_DAMAGED_UNIT_SPEED * crate::units::unit_behavior::RUN_UNIT_SPEED
        );
    }

    #[test]
    fn lid_state_matches_original_signal_and_animation_timing() {
        assert!(has_lid(VehicleType::Light));
        assert!(has_lid(VehicleType::Medium));
        assert!(has_lid(VehicleType::Heavy));
        assert!(!has_lid(VehicleType::Jeep));
        assert!(!lid_open_signal_applies(0));
        assert!(lid_open_signal_applies(1));
        assert_eq!(lid_close_delay(14), 1.4);
        assert_eq!(
            lid_signal_for_attack_target(None, Some(3), true),
            VehicleLidSignal::Open
        );
        assert_eq!(
            lid_signal_for_attack_target(Some(3), Some(3), true),
            VehicleLidSignal::None
        );
        assert_eq!(
            lid_signal_for_attack_target(Some(3), Some(4), false),
            VehicleLidSignal::None
        );
        assert_eq!(
            lid_signal_for_attack_target(Some(4), None, false),
            VehicleLidSignal::Close
        );

        let mut state = VehicleLidState::closed();
        assert!(!signal_lid_should_open(&mut state, 0));
        assert!(!state.open);
        assert!(signal_lid_should_open(&mut state, 1));
        assert!(state.open);

        process_lid_animation(&mut state, 0.2);
        assert_eq!(state.frame, 1);
        process_lid_animation(&mut state, 0.2);
        assert_eq!(state.frame, 2);
        assert!(!state.show_driver);
        process_lid_animation(&mut state, 0.2);
        assert!(state.show_driver);

        signal_lid_should_close(&mut state, 3);
        assert!(state.closing);
        assert_eq!(process_server_lid(&mut state, 0.2), None);
        assert!(state.open);
        assert_eq!(process_server_lid(&mut state, 0.1), Some(false));
        assert!(!state.open);
        process_lid_animation(&mut state, 0.2);
        assert_eq!(state.frame, 1);
        assert!(!state.show_driver);
        set_lid_state(&mut state, true);
        assert!(state.open);
        assert!(!state.closing);
    }
}
