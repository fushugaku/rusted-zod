use bevy::prelude::{Component, Vec2};

use crate::{
    components::{CombatRng, DamageMissileVisual},
    constants::TILE_SIZE,
    original::{objects::CannonType, types::TeamType},
    units::cannons::{gatling_ui, gun_ui, howitzer_ui, missile_cannon_ui},
};

#[cfg(test)]
pub(crate) const CANNON_ROTATIONS: [u16; 8] = [0, 45, 90, 135, 180, 225, 270, 315];
pub(crate) const INIT_PLACE_FRAME_COUNT: usize = 3;
pub(crate) const PLACE_FRAME_COUNT: usize = 4;
pub(crate) const PLACE_FRAME_TIME: f32 = 0.1;
#[cfg(test)]
pub(crate) const PASSIVE_ROTATION_INTERVAL: f32 = 1.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CannonFrameRole {
    Empty,
    Passive,
    #[cfg(test)]
    Fire,
    Equipped,
    Place,
    #[cfg(test)]
    DeathWreck,
    #[cfg(test)]
    Destroyed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CannonFrameProfile {
    pub(crate) role: CannonFrameRole,
    pub(crate) atlas_team: TeamType,
    pub(crate) atlas_frame_name: String,
    pub(crate) asset_path: String,
}

#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub(crate) struct CannonPlacementAnimation {
    pub(crate) cannon: CannonType,
    pub(crate) team: TeamType,
    pub(crate) source_top_left_map: Vec2,
    pub(crate) frame: usize,
    pub(crate) elapsed: f32,
    pub(crate) just_started: bool,
}

impl CannonPlacementAnimation {
    pub(crate) fn new(cannon: CannonType, team: TeamType, source_top_left_map: Vec2) -> Self {
        Self {
            cannon,
            team,
            source_top_left_map,
            frame: 0,
            elapsed: 0.0,
            just_started: true,
        }
    }

    pub(crate) fn process(&mut self, delta_secs: f32) -> bool {
        if self.just_started {
            self.just_started = false;
            return false;
        }
        self.elapsed += delta_secs.max(0.0);
        if self.elapsed >= PLACE_FRAME_TIME
            && self.frame < INIT_PLACE_FRAME_COUNT + PLACE_FRAME_COUNT
        {
            self.elapsed = 0.0;
            self.frame += 1;
        }
        self.frame >= INIT_PLACE_FRAME_COUNT + PLACE_FRAME_COUNT
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CannonRenderProfile {
    pub(crate) unit_offset: Vec2,
    pub(crate) direction_offsets: [Vec2; 8],
    pub(crate) place_frames: usize,
    pub(crate) place_frame_time: f32,
    pub(crate) passive_rotation_interval: f32,
    pub(crate) fire_flash_base_time: Option<f32>,
    pub(crate) fire_flash_random_steps: usize,
    pub(crate) fire_flash_step: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CannonTurrentProfile {
    pub(crate) target_center_offset: Vec2,
    pub(crate) max_horizontal_distance: f32,
    pub(crate) max_vertical_distance: f32,
    pub(crate) damage: i32,
    pub(crate) radius: i32,
    pub(crate) offset_time_base: f32,
    pub(crate) offset_time_random_steps: usize,
    pub(crate) offset_time_step: f32,
    pub(crate) rise_base: f32,
    pub(crate) rise_random_steps: usize,
    pub(crate) rise_step: f32,
    pub(crate) start_jitter_center: f32,
    pub(crate) start_jitter_steps: usize,
    pub(crate) arc_lift_pixels: f32,
    pub(crate) spin_base_degrees_per_sec: f32,
    pub(crate) spin_random_steps: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CannonDeathVisualPolicy {
    pub(crate) wreck_path: String,
    pub(crate) top_left_world: Vec2,
    pub(crate) top_left_map: Vec2,
    pub(crate) end_map: Vec2,
    pub(crate) delay: f32,
    pub(crate) missile_time: f32,
    pub(crate) rise: f32,
}

pub(crate) fn default_selection_size(cannon: CannonType) -> Vec2 {
    match cannon {
        CannonType::Gatling => gatling_ui::default_selection_size(),
        CannonType::Gun => gun_ui::default_selection_size(),
        CannonType::Howitzer => howitzer_ui::default_selection_size(),
        CannonType::MissileCannon => missile_cannon_ui::default_selection_size(),
    }
}

pub(crate) fn fallback_collision_size() -> Vec2 {
    Vec2::splat(TILE_SIZE * 2.0)
}

pub(crate) fn hud_name(cannon: CannonType) -> &'static str {
    match cannon {
        CannonType::Gatling => gatling_ui::hud_name(),
        CannonType::Gun => gun_ui::hud_name(),
        CannonType::Howitzer => howitzer_ui::hud_name(),
        CannonType::MissileCannon => missile_cannon_ui::hud_name(),
    }
}

pub(crate) fn frame_profile(
    role: CannonFrameRole,
    atlas_team: TeamType,
    asset_path: String,
) -> CannonFrameProfile {
    let atlas_frame_name = asset_path
        .strip_suffix(".png")
        .unwrap_or(&asset_path)
        .strip_prefix("units/cannons/")
        .unwrap_or(&asset_path)
        .replace('/', "_")
        .replace('-', "_");
    CannonFrameProfile {
        role,
        atlas_team,
        atlas_frame_name: format!("cannon_{atlas_frame_name}"),
        asset_path,
    }
}

pub(crate) fn init_place_frame_path(frame: usize) -> String {
    format!(
        "units/cannons/init-place_n{:02}.png",
        frame.min(INIT_PLACE_FRAME_COUNT - 1)
    )
}

pub(crate) fn init_place_frame_profile(frame: usize) -> CannonFrameProfile {
    frame_profile(
        CannonFrameRole::Place,
        TeamType::Red,
        init_place_frame_path(frame),
    )
}

pub(crate) fn placement_frame_profile(
    cannon: CannonType,
    team: TeamType,
    frame: usize,
) -> Option<CannonFrameProfile> {
    if frame < INIT_PLACE_FRAME_COUNT {
        return Some(init_place_frame_profile(frame));
    }
    let unit_frame = frame.checked_sub(INIT_PLACE_FRAME_COUNT)?;
    if unit_frame >= PLACE_FRAME_COUNT {
        return None;
    }
    Some(match cannon {
        CannonType::Gatling => gatling_ui::place_frame_profile(team, unit_frame),
        CannonType::Gun => gun_ui::place_frame_profile(team, unit_frame),
        CannonType::Howitzer => howitzer_ui::place_frame_profile(team, unit_frame),
        CannonType::MissileCannon => missile_cannon_ui::place_frame_profile(team, unit_frame),
    })
}

#[cfg(test)]
pub(crate) fn render_profile(cannon: CannonType) -> CannonRenderProfile {
    match cannon {
        CannonType::Gatling => gatling_ui::render_profile(),
        CannonType::Gun => gun_ui::render_profile(),
        CannonType::Howitzer => howitzer_ui::render_profile(),
        CannonType::MissileCannon => missile_cannon_ui::render_profile(),
    }
}

pub(crate) fn render_offset(cannon: CannonType, direction: usize) -> Vec2 {
    match cannon {
        CannonType::Gatling => gatling_ui::render_offset(direction),
        CannonType::Gun => gun_ui::render_offset(direction),
        CannonType::Howitzer => howitzer_ui::render_offset(direction),
        CannonType::MissileCannon => missile_cannon_ui::render_offset(direction),
    }
}

pub(crate) fn empty_frame_profile(
    cannon: CannonType,
    team: TeamType,
    rotation: u16,
) -> CannonFrameProfile {
    match cannon {
        CannonType::Gatling => gatling_ui::empty_frame_profile(rotation),
        CannonType::Gun => gun_ui::empty_frame_profile(),
        CannonType::Howitzer => howitzer_ui::empty_frame_profile(rotation),
        CannonType::MissileCannon => missile_cannon_ui::empty_frame_profile(team, rotation),
    }
}

pub(crate) fn passive_frame_profile(
    cannon: CannonType,
    team: TeamType,
    rotation: u16,
) -> CannonFrameProfile {
    match cannon {
        CannonType::Gatling => gatling_ui::passive_frame_profile(team, rotation),
        CannonType::Gun => gun_ui::passive_frame_profile(team, rotation),
        CannonType::Howitzer => howitzer_ui::passive_frame_profile(team, rotation),
        CannonType::MissileCannon => missile_cannon_ui::passive_frame_profile(team, rotation),
    }
}

pub(crate) fn spawn_frame_profile(
    cannon: CannonType,
    team: TeamType,
    rotation: u16,
) -> CannonFrameProfile {
    match cannon {
        CannonType::Gatling | CannonType::Howitzer => empty_frame_profile(cannon, team, rotation),
        CannonType::Gun | CannonType::MissileCannon => {
            equipped_frame_profile(cannon, team, rotation)
                .unwrap_or_else(|| empty_frame_profile(cannon, team, rotation))
        }
    }
}

pub(crate) fn captured_frame_profile(
    cannon: CannonType,
    team: TeamType,
    rotation: u16,
) -> CannonFrameProfile {
    passive_frame_profile(cannon, team, rotation)
}

#[cfg(test)]
pub(crate) fn fire_frame_profile(
    cannon: CannonType,
    team: TeamType,
    rotation: u16,
) -> CannonFrameProfile {
    match cannon {
        CannonType::Gatling => gatling_ui::fire_frame_profile(team, rotation),
        CannonType::Gun => gun_ui::fire_frame_profile(team, rotation),
        CannonType::Howitzer => howitzer_ui::fire_frame_profile(team, rotation),
        CannonType::MissileCannon => missile_cannon_ui::fire_frame_profile(team, rotation),
    }
}

pub(crate) fn equipped_frame_profile(
    cannon: CannonType,
    team: TeamType,
    rotation: u16,
) -> Option<CannonFrameProfile> {
    match cannon {
        CannonType::Gatling => gatling_ui::equipped_frame_profile(team, rotation),
        CannonType::Gun => gun_ui::equipped_frame_profile(team, rotation),
        CannonType::Howitzer => howitzer_ui::equipped_frame_profile(team, rotation),
        CannonType::MissileCannon => missile_cannon_ui::equipped_frame_profile(team, rotation),
    }
}

#[cfg(test)]
pub(crate) fn place_frame_profile(
    cannon: CannonType,
    team: TeamType,
    frame: usize,
) -> CannonFrameProfile {
    match cannon {
        CannonType::Gatling => gatling_ui::place_frame_profile(team, frame),
        CannonType::Gun => gun_ui::place_frame_profile(team, frame),
        CannonType::Howitzer => howitzer_ui::place_frame_profile(team, frame),
        CannonType::MissileCannon => missile_cannon_ui::place_frame_profile(team, frame),
    }
}

#[cfg(test)]
pub(crate) fn death_wreck_frame_profile(cannon: CannonType) -> CannonFrameProfile {
    match cannon {
        CannonType::Gatling => gatling_ui::death_wreck_frame_profile(),
        CannonType::Gun => gun_ui::death_wreck_frame_profile(),
        CannonType::Howitzer => howitzer_ui::death_wreck_frame_profile(),
        CannonType::MissileCannon => missile_cannon_ui::death_wreck_frame_profile(),
    }
}

#[cfg(test)]
pub(crate) fn destroyed_frame_profile(cannon: CannonType, team: TeamType) -> CannonFrameProfile {
    match cannon {
        CannonType::Gatling => gatling_ui::destroyed_frame_profile(),
        CannonType::Gun => gun_ui::destroyed_frame_profile(),
        CannonType::Howitzer => howitzer_ui::destroyed_frame_profile(),
        CannonType::MissileCannon => missile_cannon_ui::destroyed_frame_profile(team),
    }
}

#[cfg(test)]
pub(crate) fn atlas_team_for_frame(
    cannon: CannonType,
    role: CannonFrameRole,
    team: TeamType,
) -> Option<TeamType> {
    match role {
        CannonFrameRole::Empty => Some(empty_frame_profile(cannon, team, 180).atlas_team),
        CannonFrameRole::Passive => Some(passive_frame_profile(cannon, team, 180).atlas_team),
        CannonFrameRole::Fire => Some(fire_frame_profile(cannon, team, 180).atlas_team),
        CannonFrameRole::Equipped => {
            equipped_frame_profile(cannon, team, 180).map(|profile| profile.atlas_team)
        }
        CannonFrameRole::Place => Some(place_frame_profile(cannon, team, 0).atlas_team),
        CannonFrameRole::DeathWreck => Some(death_wreck_frame_profile(cannon).atlas_team),
        CannonFrameRole::Destroyed => Some(destroyed_frame_profile(cannon, team).atlas_team),
    }
}

pub(crate) fn damage_missile_visual(cannon: CannonType) -> Option<DamageMissileVisual> {
    match cannon {
        CannonType::Gun => gun_ui::damage_missile_visual(),
        CannonType::Howitzer => howitzer_ui::damage_missile_visual(),
        CannonType::MissileCannon => missile_cannon_ui::damage_missile_visual(),
        CannonType::Gatling => None,
    }
}

pub(crate) fn death_wreck_asset_path(cannon: CannonType) -> Option<String> {
    Some(match cannon {
        CannonType::Gatling => gatling_ui::death_wreck_asset_path(),
        CannonType::Gun => gun_ui::death_wreck_asset_path(),
        CannonType::Howitzer => howitzer_ui::death_wreck_asset_path(),
        CannonType::MissileCannon => missile_cannon_ui::death_wreck_asset_path(),
    })
}

pub(crate) fn death_visual_policy(
    cannon: CannonType,
    team: TeamType,
    center: Vec2,
    end_map: Vec2,
    missile_offset_time: f32,
    rng: &mut CombatRng,
) -> Option<CannonDeathVisualPolicy> {
    if team == TeamType::Null {
        return None;
    }

    let wreck_path = death_wreck_asset_path(cannon)?;
    let top_left_world = death_top_left_world_for(cannon, center);
    let top_left_map = Vec2::new(top_left_world.x, -top_left_world.y);
    let delay = death_delay_for(cannon, rng);
    Some(CannonDeathVisualPolicy {
        wreck_path,
        top_left_world,
        top_left_map,
        end_map,
        delay,
        missile_time: (missile_offset_time - delay).max(0.1),
        rise: turrent_rise(cannon, rng),
    })
}

pub(crate) fn destroyed_asset_path(cannon: CannonType, team: TeamType) -> Option<String> {
    Some(match cannon {
        CannonType::Gatling => gatling_ui::destroyed_asset_path(),
        CannonType::Gun => gun_ui::destroyed_asset_path(),
        CannonType::Howitzer => howitzer_ui::destroyed_asset_path(),
        CannonType::MissileCannon => missile_cannon_ui::destroyed_asset_path(team),
    })
}

pub(crate) fn death_top_left_world_for(cannon: CannonType, center: Vec2) -> Vec2 {
    match cannon {
        CannonType::Gatling => gatling_ui::death_top_left_world(center),
        CannonType::Gun => gun_ui::death_top_left_world(center),
        CannonType::Howitzer => howitzer_ui::death_top_left_world(center),
        CannonType::MissileCannon => missile_cannon_ui::death_top_left_world(center),
    }
}

pub(crate) fn death_delay_for(cannon: CannonType, rng: &mut CombatRng) -> f32 {
    match cannon {
        CannonType::Gatling => gatling_ui::death_delay(rng),
        CannonType::Gun => gun_ui::death_delay(rng),
        CannonType::Howitzer => howitzer_ui::death_delay(rng),
        CannonType::MissileCannon => missile_cannon_ui::death_delay(rng),
    }
}

pub(crate) fn death_spark_count_for(cannon: CannonType, rng: &mut CombatRng) -> usize {
    match cannon {
        CannonType::Gatling => gatling_ui::death_spark_count(rng),
        CannonType::Gun => gun_ui::death_spark_count(rng),
        CannonType::Howitzer => howitzer_ui::death_spark_count(rng),
        CannonType::MissileCannon => missile_cannon_ui::death_spark_count(rng),
    }
}

pub(crate) fn death_missile_offset_time_for(cannon: CannonType, rng: &mut CombatRng) -> f32 {
    match cannon {
        CannonType::Gatling => gatling_ui::death_missile_offset_time(rng),
        CannonType::Gun => gun_ui::death_missile_offset_time(rng),
        CannonType::Howitzer => howitzer_ui::death_missile_offset_time(rng),
        CannonType::MissileCannon => missile_cannon_ui::death_missile_offset_time(rng),
    }
}

pub(crate) fn turrent_profile(cannon: CannonType) -> CannonTurrentProfile {
    match cannon {
        CannonType::Gatling => gatling_ui::turrent_profile(),
        CannonType::Gun => gun_ui::turrent_profile(),
        CannonType::Howitzer => howitzer_ui::turrent_profile(),
        CannonType::MissileCannon => missile_cannon_ui::turrent_profile(),
    }
}

pub(crate) fn turrent_target_offset_for(cannon: CannonType, rng: &mut CombatRng) -> Vec2 {
    match cannon {
        CannonType::Gatling => gatling_ui::turrent_target_offset(rng),
        CannonType::Gun => gun_ui::turrent_target_offset(rng),
        CannonType::Howitzer => howitzer_ui::turrent_target_offset(rng),
        CannonType::MissileCannon => missile_cannon_ui::turrent_target_offset(rng),
    }
}

pub(crate) fn turrent_rise(cannon: CannonType, rng: &mut CombatRng) -> f32 {
    match cannon {
        CannonType::Gatling => gatling_ui::turrent_rise(rng),
        CannonType::Gun => gun_ui::turrent_rise(rng),
        CannonType::Howitzer => howitzer_ui::turrent_rise(rng),
        CannonType::MissileCannon => missile_cannon_ui::turrent_rise(rng),
    }
}

pub(crate) fn turrent_start_jitter(cannon: CannonType, rng: &mut CombatRng) -> Vec2 {
    match cannon {
        CannonType::Gatling => gatling_ui::turrent_start_jitter(rng),
        CannonType::Gun => gun_ui::turrent_start_jitter(rng),
        CannonType::Howitzer => howitzer_ui::turrent_start_jitter(rng),
        CannonType::MissileCannon => missile_cannon_ui::turrent_start_jitter(rng),
    }
}

pub(crate) fn turrent_spin_degrees_per_sec(cannon: CannonType, rng: &mut CombatRng) -> f32 {
    match cannon {
        CannonType::Gatling => gatling_ui::turrent_spin_degrees_per_sec(rng),
        CannonType::Gun => gun_ui::turrent_spin_degrees_per_sec(rng),
        CannonType::Howitzer => howitzer_ui::turrent_spin_degrees_per_sec(rng),
        CannonType::MissileCannon => missile_cannon_ui::turrent_spin_degrees_per_sec(rng),
    }
}

pub(crate) fn turrent_arc_size_for(cannon: CannonType, rise: f32, final_time: f32, t: f32) -> f32 {
    match cannon {
        CannonType::Gatling => gatling_ui::turrent_arc_size(rise, final_time, t),
        CannonType::Gun => gun_ui::turrent_arc_size(rise, final_time, t),
        CannonType::Howitzer => howitzer_ui::turrent_arc_size(rise, final_time, t),
        CannonType::MissileCannon => missile_cannon_ui::turrent_arc_size(rise, final_time, t),
    }
}

#[cfg(test)]
pub(crate) fn rocket_muzzle_offset(cannon: CannonType, direction: usize) -> Option<Vec2> {
    match cannon {
        CannonType::Gatling => None,
        CannonType::Gun => Some(gun_ui::rocket_muzzle_offset(direction)),
        CannonType::Howitzer => Some(howitzer_ui::rocket_muzzle_offset(direction)),
        CannonType::MissileCannon => Some(missile_cannon_ui::rocket_muzzle_offset(direction)),
    }
}

#[cfg(test)]
pub(crate) fn direct_fire_muzzle_offset(cannon: CannonType, direction: usize) -> Option<Vec2> {
    match cannon {
        CannonType::Gatling => Some(gatling_ui::direct_fire_muzzle_offset(direction)),
        CannonType::Gun | CannonType::Howitzer | CannonType::MissileCannon => None,
    }
}

#[cfg(test)]
pub(crate) fn fire_flash_duration(cannon: CannonType, rng: &mut CombatRng) -> Option<f32> {
    match cannon {
        CannonType::Gatling => gatling_ui::fire_flash_duration(rng),
        CannonType::Gun => gun_ui::fire_flash_duration(rng),
        CannonType::Howitzer => howitzer_ui::fire_flash_duration(rng),
        CannonType::MissileCannon => missile_cannon_ui::fire_flash_duration(rng),
    }
}
