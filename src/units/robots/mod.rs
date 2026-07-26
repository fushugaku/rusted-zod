use bevy::prelude::Vec2;

use crate::{
    components::{CombatRng, DamageMissileVisual, PortraitAnimationKind},
    original::{
        objects::{ObjectKind, RobotType},
        types::TeamType,
    },
    units::{UnitAttackSound, UnitSettings},
};

pub(crate) mod robot_state;
pub(crate) mod robot_ui;

#[path = "grunt/grunt_mod.rs"]
pub(crate) mod grunt;
#[path = "laser/laser_mod.rs"]
pub(crate) mod laser;
#[path = "psycho/psycho_mod.rs"]
pub(crate) mod psycho;
#[path = "pyro/pyro_mod.rs"]
pub(crate) mod pyro;
#[path = "sniper/sniper_mod.rs"]
pub(crate) mod sniper;
#[path = "tough/tough_mod.rs"]
pub(crate) mod tough;

pub(crate) mod grunt_ui {
    pub(crate) use super::grunt::grunt_ui::*;
}

pub(crate) mod laser_ui {
    pub(crate) use super::laser::laser_ui::*;
}

pub(crate) mod psycho_ui {
    pub(crate) use super::psycho::psycho_ui::*;
}

pub(crate) mod pyro_ui {
    pub(crate) use super::pyro::pyro_ui::*;
}

pub(crate) mod sniper_ui {
    pub(crate) use super::sniper::sniper_ui::*;
}

pub(crate) mod tough_ui {
    pub(crate) use super::tough::tough_ui::*;
}

#[cfg(test)]
pub(crate) use grunt_ui::RobotDeathAnimationProfile;
#[allow(unused_imports)]
pub(crate) use grunt_ui::{
    DEATH_FRAME_TIME, FLIP_AIR_FRAME_TIME, FLIP_LAND_FRAME_TIME, MELT_DEATH_FRAME_COUNT,
    NORMAL_DEATH_FRAME_COUNTS, TURRENT_DEATH_FRAME_COUNT, TurrentDeathTrajectory,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SpecialProjectileKind {
    Laser,
    Flame,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeathEffectChoice {
    None,
    Normal,
    Melt,
    Turrent,
}

pub(crate) use robot_state::RobotFireAnimationReset;
pub(crate) use robot_state::{RobotIdleActionKind, RobotIdleProcessChoice};

pub(crate) fn default_selection_size(robot: RobotType) -> Vec2 {
    match robot {
        RobotType::Grunt => grunt_ui::default_selection_size(),
        RobotType::Psycho => psycho_ui::default_selection_size(),
        RobotType::Sniper => sniper_ui::default_selection_size(),
        RobotType::Tough => tough_ui::default_selection_size(),
        RobotType::Pyro => pyro_ui::default_selection_size(),
        RobotType::Laser => laser_ui::default_selection_size(),
    }
}

pub(crate) fn source_dimensions(_robot: RobotType) -> Vec2 {
    robot_ui::source_dimensions()
}

pub(crate) fn requires_activation(robot: RobotType) -> bool {
    match robot {
        RobotType::Grunt => grunt::REQUIRES_ACTIVATION,
        RobotType::Psycho => psycho::REQUIRES_ACTIVATION,
        RobotType::Sniper => sniper::REQUIRES_ACTIVATION,
        RobotType::Tough => tough::REQUIRES_ACTIVATION,
        RobotType::Pyro => pyro::REQUIRES_ACTIVATION,
        RobotType::Laser => laser::REQUIRES_ACTIVATION,
    }
}

pub(crate) fn hud_name(robot: RobotType) -> &'static str {
    match robot {
        RobotType::Grunt => grunt_ui::hud_name(),
        RobotType::Psycho => psycho_ui::hud_name(),
        RobotType::Sniper => sniper_ui::hud_name(),
        RobotType::Tough => tough_ui::hud_name(),
        RobotType::Pyro => pyro_ui::hud_name(),
        RobotType::Laser => laser_ui::hud_name(),
    }
}

pub(crate) fn portrait_frame_path(
    robot: RobotType,
    team: TeamType,
    frame: usize,
) -> Option<String> {
    match robot {
        RobotType::Grunt => grunt_ui::portrait_frame_path(team, frame),
        RobotType::Psycho => psycho_ui::portrait_frame_path(team, frame),
        RobotType::Sniper => sniper_ui::portrait_frame_path(team, frame),
        RobotType::Tough => tough_ui::portrait_frame_path(team, frame),
        RobotType::Pyro => pyro_ui::portrait_frame_path(team, frame),
        RobotType::Laser => laser_ui::portrait_frame_path(team, frame),
    }
}

pub(crate) fn portrait_shoulders_height(robot: RobotType) -> f32 {
    match robot {
        RobotType::Grunt => grunt_ui::PORTRAIT_SHOULDERS_HEIGHT,
        RobotType::Psycho => psycho_ui::PORTRAIT_SHOULDERS_HEIGHT,
        RobotType::Sniper => sniper_ui::PORTRAIT_SHOULDERS_HEIGHT,
        RobotType::Tough => tough_ui::PORTRAIT_SHOULDERS_HEIGHT,
        RobotType::Pyro => pyro_ui::PORTRAIT_SHOULDERS_HEIGHT,
        RobotType::Laser => laser_ui::PORTRAIT_SHOULDERS_HEIGHT,
    }
}

#[cfg(test)]
pub(crate) fn stand_atlas_frame_name(team: TeamType, rotation: u16) -> String {
    robot_ui::stand_atlas_frame_name(team, rotation)
}

pub(crate) fn stand_atlas_frame_spec(
    team: TeamType,
    rotation: u16,
) -> robot_ui::RobotAtlasFrameSpec {
    robot_ui::stand_atlas_frame_spec(team, rotation)
}

#[cfg(test)]
pub(crate) fn walk_atlas_frame_name(team: TeamType, rotation: u16, frame: usize) -> String {
    robot_ui::walk_atlas_frame_name(team, rotation, frame)
}

#[cfg(test)]
pub(crate) fn walk_atlas_frame_spec(
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> robot_ui::RobotAtlasFrameSpec {
    robot_ui::walk_atlas_frame_spec(team, rotation, frame)
}

#[cfg(test)]
pub(crate) fn mobile_atlas_frame_name(
    team: TeamType,
    rotation: u16,
    frame: usize,
    moving: bool,
) -> String {
    robot_ui::mobile_atlas_frame_name(team, rotation, frame, moving)
}

pub(crate) fn mobile_atlas_frame_spec(
    team: TeamType,
    rotation: u16,
    frame: usize,
    moving: bool,
) -> robot_ui::RobotAtlasFrameSpec {
    robot_ui::mobile_atlas_frame_spec(team, rotation, frame, moving)
}

#[cfg(test)]
pub(crate) fn throw_atlas_frame_name(team: TeamType, rotation: u16, frame: usize) -> String {
    robot_ui::throw_atlas_frame_name(team, rotation, frame)
}

pub(crate) fn throw_atlas_frame_spec(
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> robot_ui::RobotAtlasFrameSpec {
    robot_ui::throw_atlas_frame_spec(team, rotation, frame)
}

pub(crate) fn fire_atlas_frame_spec(
    robot: RobotType,
    team: TeamType,
    rotation: u16,
    frame: usize,
) -> robot_ui::RobotAtlasFrameSpec {
    let atlas_team = team.atlas_team();
    let frame_name = match robot {
        RobotType::Grunt => grunt_ui::fire_atlas_frame_name(atlas_team, rotation, frame),
        RobotType::Psycho => psycho_ui::fire_atlas_frame_name(atlas_team, rotation, frame),
        RobotType::Sniper => sniper_ui::fire_atlas_frame_name(atlas_team, rotation, frame),
        RobotType::Tough => tough_ui::fire_atlas_frame_name(atlas_team, rotation, frame),
        RobotType::Pyro => pyro_ui::fire_atlas_frame_name(atlas_team, rotation, frame),
        RobotType::Laser => laser_ui::fire_atlas_frame_name(atlas_team, rotation, frame),
    };
    robot_ui::RobotAtlasFrameSpec {
        atlas_team,
        frame_name,
    }
}

pub(crate) fn grenade_throw_start_frame() -> usize {
    robot_state::grenade_throw_start_frame()
}

pub(crate) fn grenade_pickup_start_frame() -> usize {
    robot_state::grenade_pickup_start_frame()
}

pub(crate) fn grenade_pickup_uses_upward_frames(rotation: u16) -> bool {
    robot_state::grenade_pickup_uses_upward_frames(rotation)
}

#[cfg(test)]
pub(crate) fn grenade_pickup_atlas_frame_name(
    team: TeamType,
    upward: bool,
    frame: usize,
) -> String {
    robot_ui::grenade_pickup_atlas_frame_name(team, upward, frame)
}

pub(crate) fn grenade_pickup_atlas_frame_spec(
    team: TeamType,
    upward: bool,
    frame: usize,
) -> robot_ui::RobotAtlasFrameSpec {
    robot_ui::grenade_pickup_atlas_frame_spec(team, upward, frame)
}

#[cfg(test)]
pub(crate) fn idle_action_atlas_frame_name(
    team: TeamType,
    kind: RobotIdleActionKind,
    frame: usize,
) -> String {
    robot_ui::idle_action_atlas_frame_name(team, kind, frame)
}

pub(crate) fn idle_action_atlas_frame_spec(
    team: TeamType,
    kind: RobotIdleActionKind,
    frame: usize,
) -> robot_ui::RobotAtlasFrameSpec {
    robot_ui::idle_action_atlas_frame_spec(team, kind, frame)
}

pub(crate) fn idle_action_start_frame() -> usize {
    robot_state::idle_action_start_frame()
}

pub(crate) fn idle_action_default_direction() -> usize {
    robot_state::idle_action_default_direction()
}

pub(crate) fn idle_process_choice(
    activity_roll: usize,
    turn_roll: usize,
    direction_roll: usize,
    action_roll: usize,
) -> RobotIdleProcessChoice {
    robot_state::idle_process_choice(activity_roll, turn_roll, direction_roll, action_roll)
}

pub(crate) fn common_process_delta_seconds(delta_secs: f32, speed_offset_percent: f32) -> f32 {
    robot_state::common_process_delta_seconds(delta_secs, speed_offset_percent)
}

pub(crate) fn fire_animation_start_frame() -> usize {
    robot_state::fire_animation_start_frame()
}

pub(crate) fn fire_animation_reset_for_attack_assignment(
    robot: RobotType,
) -> RobotFireAnimationReset {
    robot_state::fire_animation_reset_for_attack_assignment(robot)
}

pub(crate) fn fire_animation_projectile_start_frame(robot: RobotType) -> usize {
    robot_state::fire_animation_projectile_start_frame(robot)
}

pub(crate) fn fire_animation_projectile_frame(robot: RobotType) -> usize {
    robot_state::fire_animation_projectile_frame(robot)
}

pub(crate) fn fire_animation_delay_after_frame(
    robot: RobotType,
    frame: usize,
    rng: &mut CombatRng,
) -> f32 {
    robot_state::fire_animation_delay_after_frame(robot, frame, rng)
}

pub(crate) fn advance_fire_animation(
    robot: RobotType,
    frame: &mut usize,
    elapsed: &mut f32,
    delay: &mut f32,
    delta_secs: f32,
    rng: &mut CombatRng,
) {
    robot_state::advance_fire_animation(robot, frame, elapsed, delay, delta_secs, rng)
}

pub(crate) fn grenade_ready_attack_pose_active(
    kind: ObjectKind,
    has_throwable_grenade: bool,
    target_attacked_only_by_explosives: bool,
) -> bool {
    robot_state::grenade_ready_attack_pose_active(
        kind,
        has_throwable_grenade,
        target_attacked_only_by_explosives,
    )
}

pub(crate) fn grenade_ready_attack_pose_frame() -> usize {
    robot_state::grenade_ready_attack_pose_frame()
}

pub(crate) fn advance_grenade_throw_animation(
    frame: &mut usize,
    elapsed: &mut f32,
    delta_secs: f32,
) -> bool {
    robot_state::advance_grenade_throw_animation(frame, elapsed, delta_secs)
}

pub(crate) fn advance_grenade_pickup_animation(
    frame: &mut usize,
    elapsed: &mut f32,
    delta_secs: f32,
) -> bool {
    robot_state::advance_grenade_pickup_animation(frame, elapsed, delta_secs)
}

pub(crate) fn advance_idle_action_animation(
    kind: RobotIdleActionKind,
    frame: &mut usize,
    elapsed: &mut f32,
    delta_secs: f32,
) -> bool {
    robot_state::advance_idle_action_animation(kind, frame, elapsed, delta_secs)
}

#[cfg(test)]
pub(crate) fn movement_profile() -> robot_ui::RobotMovementProfile {
    robot_ui::movement_profile()
}

pub(crate) fn mobile_frame_count() -> usize {
    robot_ui::mobile_frame_count()
}

pub(crate) fn mobile_frame_time() -> f32 {
    robot_ui::mobile_frame_time()
}

pub(crate) fn mobile_sprite_role(
    layer_index: usize,
) -> Option<crate::render::atlas::MobileSpriteRole> {
    robot_ui::mobile_sprite_role(layer_index)
}

#[cfg(test)]
pub(crate) fn group_member_count(robot: RobotType) -> u8 {
    settings(robot).group_amount
}

#[cfg(test)]
pub(crate) fn group_selection_profile() -> robot_ui::RobotGroupSelectionProfile {
    robot_ui::group_selection_profile()
}

pub(crate) fn group_leader_ref_id(ref_id: u32, leader_ref_id: Option<u32>) -> u32 {
    robot_ui::group_leader_ref_id(ref_id, leader_ref_id)
}

#[cfg(test)]
pub(crate) fn group_member_is_leader(ref_id: u32, leader_ref_id: Option<u32>) -> bool {
    robot_ui::group_member_is_leader(ref_id, leader_ref_id)
}

#[cfg(test)]
pub(crate) fn group_member_visible_in_ordered_selection(
    ref_id: u32,
    leader_ref_id: Option<u32>,
) -> bool {
    robot_ui::group_member_visible_in_ordered_selection(ref_id, leader_ref_id)
}

#[cfg(test)]
pub(crate) fn selected_refs_include_group_member(
    selected_refs: &[u32],
    ref_id: u32,
    leader_ref_id: Option<u32>,
) -> bool {
    robot_ui::selected_refs_include_group_member(selected_refs, ref_id, leader_ref_id)
}

pub(crate) fn settings(robot: RobotType) -> UnitSettings {
    match robot {
        RobotType::Grunt => grunt::settings(),
        RobotType::Psycho => psycho::settings(),
        RobotType::Sniper => sniper::settings(),
        RobotType::Tough => tough::settings(),
        RobotType::Pyro => pyro::settings(),
        RobotType::Laser => laser::settings(),
    }
}

pub(crate) fn attack_sound(robot: RobotType) -> Option<UnitAttackSound> {
    match robot {
        RobotType::Grunt => grunt::attack_sound(),
        RobotType::Psycho => psycho::attack_sound(),
        RobotType::Sniper => sniper::attack_sound(),
        RobotType::Tough => tough::attack_sound(),
        RobotType::Pyro => pyro::attack_sound(),
        RobotType::Laser => laser::attack_sound(),
    }
}

pub(crate) fn selected_portrait_animation(
    robot: RobotType,
    rng: &mut CombatRng,
) -> PortraitAnimationKind {
    if rng.index(2) == 0 {
        PortraitAnimationKind::SelectedRobotReporting(robot)
    } else {
        super::unit_sound::selected_common_portrait_animation(rng)
    }
}

pub(crate) fn selected_reporting_voice_asset_path(robot: RobotType) -> &'static str {
    match robot {
        RobotType::Grunt => grunt_ui::selected_reporting_voice_asset_path(),
        RobotType::Psycho => psycho_ui::selected_reporting_voice_asset_path(),
        RobotType::Sniper => sniper_ui::selected_reporting_voice_asset_path(),
        RobotType::Tough => tough_ui::selected_reporting_voice_asset_path(),
        RobotType::Pyro => pyro_ui::selected_reporting_voice_asset_path(),
        RobotType::Laser => laser_ui::selected_reporting_voice_asset_path(),
    }
}

#[cfg(test)]
pub(crate) fn death_animation_profile() -> RobotDeathAnimationProfile {
    grunt_ui::death_animation_profile()
}

pub(crate) fn special_projectile_kind(robot: RobotType) -> Option<SpecialProjectileKind> {
    match robot {
        RobotType::Pyro => Some(pyro_ui::special_projectile_kind()),
        RobotType::Laser => Some(laser_ui::special_projectile_kind()),
        RobotType::Grunt | RobotType::Psycho | RobotType::Sniper | RobotType::Tough => None,
    }
}

pub(crate) fn special_projectile_frame_paths(kind: SpecialProjectileKind) -> Vec<String> {
    match kind {
        SpecialProjectileKind::Laser => laser_ui::special_projectile_frame_paths(),
        SpecialProjectileKind::Flame => pyro_ui::special_projectile_frame_paths(),
    }
}

pub(crate) fn special_projectile_duration(
    kind: SpecialProjectileKind,
    start: Vec2,
    target: Vec2,
) -> f32 {
    match kind {
        SpecialProjectileKind::Laser => laser_ui::special_projectile_duration(start, target),
        SpecialProjectileKind::Flame => pyro_ui::special_projectile_duration(start, target),
    }
}

pub(crate) fn special_projectile_frame_time(kind: SpecialProjectileKind) -> f32 {
    match kind {
        SpecialProjectileKind::Laser => laser_ui::special_projectile_frame_time(),
        SpecialProjectileKind::Flame => pyro_ui::special_projectile_frame_time(),
    }
}

pub(crate) fn special_projectile_entity_name(kind: SpecialProjectileKind) -> &'static str {
    match kind {
        SpecialProjectileKind::Laser => laser_ui::special_projectile_entity_name(),
        SpecialProjectileKind::Flame => pyro_ui::special_projectile_entity_name(),
    }
}

pub(crate) fn special_projectile_next_frame(
    kind: SpecialProjectileKind,
    current: usize,
    frame_count: usize,
    rng: &mut CombatRng,
) -> usize {
    match kind {
        SpecialProjectileKind::Laser => laser_ui::special_projectile_next_frame(current, rng),
        SpecialProjectileKind::Flame => pyro_ui::special_projectile_next_frame(frame_count, rng),
    }
}

pub(crate) fn special_projectile_spawns_fire_impact(kind: SpecialProjectileKind) -> bool {
    match kind {
        SpecialProjectileKind::Laser => laser_ui::special_projectile_spawns_fire_impact(),
        SpecialProjectileKind::Flame => pyro_ui::special_projectile_spawns_fire_impact(),
    }
}

pub(crate) fn attack_marks_fire_damage(kind: ObjectKind) -> bool {
    matches!(kind, ObjectKind::Robot(RobotType::Pyro)) && pyro_ui::attack_marks_fire_damage()
}

pub(crate) fn uses_direct_fire_bullet(kind: ObjectKind) -> bool {
    match kind {
        ObjectKind::Robot(robot) => special_projectile_kind(robot).is_none(),
        _ => true,
    }
}

pub(crate) fn pyro_fire_impact_profile() -> pyro_ui::FireImpactProfile {
    pyro_ui::fire_impact_profile()
}

#[cfg(test)]
pub(crate) fn pyro_fire_impact_family_count() -> usize {
    pyro_ui::fire_impact_family_count()
}

pub(crate) fn pyro_fire_impact_frame_paths(family: usize) -> Vec<String> {
    pyro_ui::fire_impact_frame_paths(family)
}

pub(crate) fn damage_missile_visual(robot: RobotType) -> Option<DamageMissileVisual> {
    match robot {
        RobotType::Tough => tough_ui::damage_missile_visual(),
        RobotType::Grunt
        | RobotType::Psycho
        | RobotType::Sniper
        | RobotType::Pyro
        | RobotType::Laser => None,
    }
}

pub(crate) fn tough_mushroom_effect_profile() -> tough_ui::MushroomEffectProfile {
    tough_ui::mushroom_effect_profile()
}

pub(crate) fn tough_mushroom_frame_paths() -> Vec<String> {
    tough_ui::mushroom_frame_paths()
}

pub(crate) fn tough_mushroom_base_top_left(map_center: Vec2, scale: f32) -> Vec2 {
    tough_ui::mushroom_base_top_left(map_center, scale)
}

pub(crate) fn tough_mushroom_frame_offsets(scale: f32) -> Vec<Vec2> {
    tough_ui::mushroom_frame_offsets(scale)
}

pub(crate) fn tough_smoke_effect_profile() -> tough_ui::SmokeEffectProfile {
    tough_ui::smoke_effect_profile()
}

pub(crate) fn tough_smoke_frame_paths() -> Vec<String> {
    tough_ui::smoke_frame_paths()
}

pub(crate) fn tough_rocket_smoke_positions(
    start: Vec2,
    target: Vec2,
    total_time: f32,
    smoke_time_cursor: &mut f32,
    elapsed: f32,
    offsets: &[Vec2],
) -> Vec<Vec2> {
    tough_ui::rocket_smoke_positions(
        start,
        target,
        total_time,
        smoke_time_cursor,
        elapsed,
        offsets,
    )
}

pub(crate) fn death_effect_choice(
    kind: ObjectKind,
    team: TeamType,
    do_fire_death: bool,
    do_missile_death: bool,
) -> DeathEffectChoice {
    if !matches!(kind, ObjectKind::Robot(_)) || team == TeamType::Null {
        return DeathEffectChoice::None;
    }
    if do_missile_death {
        DeathEffectChoice::Turrent
    } else if do_fire_death {
        DeathEffectChoice::Melt
    } else {
        DeathEffectChoice::Normal
    }
}

pub(crate) fn normal_death_frame_paths(team: TeamType, variant: usize) -> Option<Vec<String>> {
    grunt_ui::normal_death_frame_paths(team, variant)
}

pub(crate) fn melt_death_frame_paths(team: TeamType) -> Option<Vec<String>> {
    grunt_ui::melt_death_frame_paths(team)
}

pub(crate) fn turrent_death_frame_paths(team: TeamType) -> Option<Vec<String>> {
    grunt_ui::turrent_death_frame_paths(team)
}

pub(crate) fn death_top_left_world(center: Vec2) -> Vec2 {
    grunt_ui::death_top_left_world(center)
}

pub(crate) fn turrent_death_trajectory(
    center_map: Vec2,
    rng: &mut CombatRng,
) -> TurrentDeathTrajectory {
    grunt_ui::turrent_death_trajectory(center_map, rng)
}

pub(crate) fn turrent_death_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    grunt_ui::turrent_death_arc_size(rise, final_time, t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::VehicleType;
    use crate::render::atlas::MobileSpriteRole;

    #[test]
    fn robot_mobile_atlas_frame_api_matches_original_names() {
        assert_eq!(
            stand_atlas_frame_name(TeamType::Blue, 180),
            "robot_stand_blue_r180"
        );
        assert_eq!(
            walk_atlas_frame_name(TeamType::Green, 45, 5),
            "robot_walk_green_r045_n01"
        );
        assert_eq!(
            mobile_atlas_frame_name(TeamType::Yellow, 90, 3, true),
            "robot_walk_yellow_r090_n03"
        );
        assert_eq!(
            mobile_atlas_frame_name(TeamType::Null, 270, 7, false),
            "robot_stand_red_r270"
        );
        assert_eq!(
            throw_atlas_frame_name(TeamType::Blue, 180, 0),
            "robot_throw_blue_r180_n00"
        );
        assert_eq!(
            throw_atlas_frame_name(TeamType::White, 45, 5),
            "robot_throw_red_r045_n01"
        );
        assert_eq!(
            grenade_pickup_atlas_frame_name(TeamType::Blue, true, 0),
            "robot_pickup-up_blue_n00"
        );
        assert_eq!(
            grenade_pickup_atlas_frame_name(TeamType::White, false, 5),
            "robot_pickup-down_red_n01"
        );
        assert_eq!(
            idle_action_atlas_frame_name(TeamType::Yellow, RobotIdleActionKind::Beer, 12),
            "robot_beer_yellow_n02"
        );
        assert_eq!(
            idle_action_atlas_frame_name(TeamType::White, RobotIdleActionKind::FullScan, 11),
            "robot_full_area_scan_red_n11"
        );

        assert_eq!(
            stand_atlas_frame_spec(TeamType::Purple, 180),
            robot_ui::RobotAtlasFrameSpec {
                atlas_team: TeamType::Red,
                frame_name: "robot_stand_red_r180".to_string(),
            }
        );
        assert_eq!(
            walk_atlas_frame_spec(TeamType::Green, 45, 5),
            robot_ui::RobotAtlasFrameSpec {
                atlas_team: TeamType::Green,
                frame_name: "robot_walk_green_r045_n01".to_string(),
            }
        );
        assert_eq!(
            mobile_atlas_frame_spec(TeamType::Blue, 315, 7, true),
            robot_ui::RobotAtlasFrameSpec {
                atlas_team: TeamType::Blue,
                frame_name: "robot_walk_blue_r315_n03".to_string(),
            }
        );
        assert_eq!(
            throw_atlas_frame_spec(TeamType::Yellow, 315, 3),
            robot_ui::RobotAtlasFrameSpec {
                atlas_team: TeamType::Yellow,
                frame_name: "robot_throw_yellow_r315_n03".to_string(),
            }
        );
        assert_eq!(
            grenade_pickup_atlas_frame_spec(TeamType::Green, true, 2),
            robot_ui::RobotAtlasFrameSpec {
                atlas_team: TeamType::Green,
                frame_name: "robot_pickup-up_green_n02".to_string(),
            }
        );
        assert_eq!(
            idle_action_atlas_frame_spec(TeamType::Blue, RobotIdleActionKind::HeadStretch, 10),
            robot_ui::RobotAtlasFrameSpec {
                atlas_team: TeamType::Blue,
                frame_name: "robot_head_stretch_blue_n10".to_string(),
            }
        );
        assert_eq!(
            fire_atlas_frame_spec(RobotType::Grunt, TeamType::Blue, 180, 4),
            robot_ui::RobotAtlasFrameSpec {
                atlas_team: TeamType::Blue,
                frame_name: "robot_grunt_fire_blue_r180_n04".to_string(),
            }
        );
        assert_eq!(
            fire_atlas_frame_spec(RobotType::Psycho, TeamType::Green, 45, 3),
            robot_ui::RobotAtlasFrameSpec {
                atlas_team: TeamType::Green,
                frame_name: "robot_psycho_fire_green_r045_n01".to_string(),
            }
        );
        assert_eq!(
            fire_atlas_frame_spec(RobotType::Tough, TeamType::White, 315, 2),
            robot_ui::RobotAtlasFrameSpec {
                atlas_team: TeamType::Red,
                frame_name: "robot_tough_fire_red_r315_n02".to_string(),
            }
        );
    }

    #[test]
    fn robot_mobile_animation_profile_lives_on_robot_ui() {
        assert_eq!(
            movement_profile(),
            robot_ui::RobotMovementProfile {
                frame_count: 4,
                frame_time: 0.3,
            }
        );
        assert_eq!(mobile_frame_count(), 4);
        assert_eq!(mobile_frame_time(), 0.3);
        assert_eq!(mobile_sprite_role(0), Some(MobileSpriteRole::Robot));
        assert_eq!(mobile_sprite_role(1), None);
    }

    #[test]
    fn robot_group_selection_metadata_lives_on_robot_ui() {
        assert_eq!(group_member_count(RobotType::Grunt), 3);
        assert_eq!(group_member_count(RobotType::Tough), 2);
        assert_eq!(group_member_count(RobotType::Laser), 4);
        assert_eq!(
            group_selection_profile(),
            robot_ui::RobotGroupSelectionProfile {
                normalize_member_selection_to_leader: true,
                ordered_selection_uses_leaders_only: true,
                selected_leader_controls_members: true,
            }
        );

        assert_eq!(group_leader_ref_id(10, None), 10);
        assert_eq!(group_leader_ref_id(11, Some(10)), 10);
        assert!(group_member_is_leader(10, Some(10)));
        assert!(!group_member_is_leader(11, Some(10)));
        assert!(group_member_visible_in_ordered_selection(10, Some(10)));
        assert!(!group_member_visible_in_ordered_selection(11, Some(10)));
        assert!(selected_refs_include_group_member(&[10], 11, Some(10)));
        assert!(!selected_refs_include_group_member(&[11], 11, Some(10)));
    }

    #[test]
    fn robot_death_effect_choice_matches_original_priority() {
        let robot = ObjectKind::Robot(RobotType::Grunt);

        assert_eq!(
            death_effect_choice(robot, TeamType::Red, true, true),
            DeathEffectChoice::Turrent
        );
        assert_eq!(
            death_effect_choice(robot, TeamType::Red, true, false),
            DeathEffectChoice::Melt
        );
        assert_eq!(
            death_effect_choice(robot, TeamType::Red, false, false),
            DeathEffectChoice::Normal
        );
        assert_eq!(
            death_effect_choice(robot, TeamType::Null, true, true),
            DeathEffectChoice::None
        );
        assert_eq!(
            death_effect_choice(
                ObjectKind::Vehicle(VehicleType::Jeep),
                TeamType::Red,
                true,
                true
            ),
            DeathEffectChoice::None
        );
    }

    #[test]
    fn robot_death_effect_frame_paths_match_original_assets() {
        let die1 = normal_death_frame_paths(TeamType::Red, 0).unwrap();
        let die4 = normal_death_frame_paths(TeamType::Red, 3).unwrap();
        let die_clamped = normal_death_frame_paths(TeamType::Red, 99).unwrap();
        let melt = melt_death_frame_paths(TeamType::Blue).unwrap();
        let turrent = turrent_death_frame_paths(TeamType::Green).unwrap();

        assert_eq!(die1.len(), 10);
        assert_eq!(die1.first().unwrap(), "units/robots/die1_red_n00.png");
        assert_eq!(die4.len(), 8);
        assert_eq!(die4.last().unwrap(), "units/robots/die4_red_n07.png");
        assert_eq!(die_clamped, die4);
        assert_eq!(melt.len(), 17);
        assert_eq!(melt.last().unwrap(), "units/robots/melt_blue_n16.png");
        assert_eq!(turrent.len(), 33);
        assert_eq!(turrent.last().unwrap(), "units/robots/die5_green_n32.png");
        assert!(normal_death_frame_paths(TeamType::Null, 0).is_none());
        assert!(melt_death_frame_paths(TeamType::Null).is_none());
        assert!(turrent_death_frame_paths(TeamType::Null).is_none());
    }

    #[test]
    fn robot_death_top_left_converts_from_center_like_original_robot_loc() {
        assert_eq!(
            death_top_left_world(Vec2::new(100.0, -50.0)),
            Vec2::new(92.0, -42.0)
        );
    }

    #[test]
    fn robot_death_animation_profile_matches_original_ranges() {
        assert_eq!(
            death_animation_profile(),
            RobotDeathAnimationProfile {
                death_frame_time: 0.16,
                death_z: 34.0,
                normal_frame_counts: [10, 10, 10, 8],
                melt_frame_count: 17,
                turrent_frame_count: 33,
                turrent_air_frame_count: 8,
                turrent_land_start_frame: 8,
                turrent_last_frame: 32,
                flip_air_frame_time: 0.05,
                flip_land_frame_time: 0.15,
            }
        );
        assert_eq!(turrent_death_arc_size(2.0, 4.0, 0.0), 1.0);
        assert!(turrent_death_arc_size(2.0, 4.0, 1.0) > 1.0);

        let mut rng = CombatRng::default();
        for _ in 0..32 {
            let trajectory = turrent_death_trajectory(Vec2::new(10.0, 20.0), &mut rng);
            assert!((3.5..=4.4).contains(&trajectory.final_time));
            assert!((1.3..=3.29).contains(&trajectory.rise));
            assert!(
                (trajectory.start_map.x - 10.0) >= -2.0 && (trajectory.start_map.x - 10.0) <= 2.0
            );
            assert!(
                (trajectory.end_map.x - trajectory.start_map.x) > -100.0
                    && (trajectory.end_map.x - trajectory.start_map.x) <= 100.0
            );
        }
    }

    #[test]
    fn robot_effect_wrappers_expose_unit_side_assets_to_main() {
        assert_eq!(
            pyro_fire_impact_profile(),
            pyro_ui::FireImpactProfile {
                family_count: 5,
                frame_counts: [4, 4, 4, 6, 6],
                frame_time: 0.06,
                z: 34.4,
            }
        );
        assert_eq!(pyro_fire_impact_family_count(), 5);
        assert_eq!(
            pyro_fire_impact_frame_paths(4).last().unwrap(),
            "other/fire/fire4_n05.png"
        );

        assert_eq!(tough_mushroom_effect_profile().frame_time, 0.08);
        assert_eq!(tough_smoke_effect_profile().frame_time, 0.12);
        assert_eq!(
            tough_mushroom_frame_paths().last().unwrap(),
            "units/robots/tough/mushroom_n11.png"
        );
        assert_eq!(
            tough_smoke_frame_paths().last().unwrap(),
            "units/robots/tough/smoke_n07.png"
        );
        assert_eq!(
            tough_mushroom_base_top_left(Vec2::new(100.0, 50.0), 1.5),
            Vec2::new(76.0, 2.0)
        );
        assert_eq!(tough_mushroom_frame_offsets(1.5)[0], Vec2::new(0.0, 21.0));

        let mut cursor = 0.0;
        let smoke = tough_rocket_smoke_positions(
            Vec2::ZERO,
            Vec2::new(150.0, 0.0),
            1.0,
            &mut cursor,
            0.06,
            &[Vec2::ZERO],
        );
        assert_eq!(smoke.len(), 1);
        assert!((smoke[0].x + 6.0).abs() < 0.0001);
    }

    #[test]
    fn robot_selected_reporting_voice_assets_are_sourced_from_ui_modules() {
        assert_eq!(
            selected_reporting_voice_asset_path(RobotType::Grunt),
            grunt_ui::selected_reporting_voice_asset_path()
        );
        assert_eq!(
            selected_reporting_voice_asset_path(RobotType::Psycho),
            psycho_ui::selected_reporting_voice_asset_path()
        );
        assert_eq!(
            selected_reporting_voice_asset_path(RobotType::Sniper),
            sniper_ui::selected_reporting_voice_asset_path()
        );
        assert_eq!(
            selected_reporting_voice_asset_path(RobotType::Tough),
            tough_ui::selected_reporting_voice_asset_path()
        );
        assert_eq!(
            selected_reporting_voice_asset_path(RobotType::Laser),
            laser_ui::selected_reporting_voice_asset_path()
        );
        assert_eq!(
            selected_reporting_voice_asset_path(RobotType::Pyro),
            pyro_ui::selected_reporting_voice_asset_path()
        );
    }

    #[test]
    fn portrait_frame_paths_keep_each_source_robot_face_id_in_its_ui_file() {
        for (robot, face_id) in [
            (RobotType::Grunt, 2),
            (RobotType::Psycho, 3),
            (RobotType::Sniper, 4),
            (RobotType::Tough, 0),
            (RobotType::Pyro, 1),
            (RobotType::Laser, 1),
        ] {
            let path = portrait_frame_path(robot, TeamType::Blue, 39).unwrap();
            assert!(path.contains(&format!("SHEADBI{face_id}_0039.png")));
            assert!(path.contains(&format!("{}_blue", hud_name(robot))));
        }
        assert!(portrait_frame_path(RobotType::Grunt, TeamType::Null, 0).is_none());
    }

    #[test]
    fn special_projectile_runtime_profile_stays_on_robot_units() {
        let mut rng = CombatRng::default();
        assert_eq!(
            special_projectile_entity_name(SpecialProjectileKind::Laser),
            "laser_projectile"
        );
        assert_eq!(
            special_projectile_entity_name(SpecialProjectileKind::Flame),
            "pyro_flame_projectile"
        );
        assert_eq!(
            special_projectile_duration(
                SpecialProjectileKind::Laser,
                Vec2::ZERO,
                Vec2::new(300.0, 0.0),
            ),
            1.0
        );
        assert_eq!(
            special_projectile_frame_time(SpecialProjectileKind::Flame),
            0.05
        );
        assert_eq!(
            special_projectile_next_frame(SpecialProjectileKind::Laser, 1, 2, &mut rng),
            1
        );
        assert!(special_projectile_next_frame(SpecialProjectileKind::Flame, 0, 4, &mut rng) < 4);
        assert!(special_projectile_spawns_fire_impact(
            SpecialProjectileKind::Flame
        ));
        assert!(!special_projectile_spawns_fire_impact(
            SpecialProjectileKind::Laser
        ));
        assert!(attack_marks_fire_damage(ObjectKind::Robot(RobotType::Pyro)));
        assert!(!attack_marks_fire_damage(ObjectKind::Robot(
            RobotType::Laser
        )));
        assert!(!uses_direct_fire_bullet(ObjectKind::Robot(
            RobotType::Laser
        )));
        assert!(!uses_direct_fire_bullet(ObjectKind::Robot(RobotType::Pyro)));
        assert!(uses_direct_fire_bullet(ObjectKind::Robot(RobotType::Grunt)));
        assert!(uses_direct_fire_bullet(ObjectKind::Vehicle(
            VehicleType::Jeep
        )));
    }
}
