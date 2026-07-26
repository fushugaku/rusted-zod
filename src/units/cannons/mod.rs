use bevy::prelude::Vec2;

use crate::{
    components::{CombatRng, DamageMissileVisual},
    original::{objects::CannonType, types::TeamType},
    units::{UnitAttackSound, UnitSettings},
};

pub(crate) mod cannon_ui;
#[path = "gatling/gatling_mod.rs"]
pub(crate) mod gatling;
#[path = "gun/gun_mod.rs"]
pub(crate) mod gun;
#[path = "howitzer/howitzer_mod.rs"]
pub(crate) mod howitzer;
#[path = "missile_cannon/missile_cannon_mod.rs"]
pub(crate) mod missile_cannon;

pub(crate) mod gatling_ui {
    pub(crate) use super::gatling::gatling_ui::*;
}

pub(crate) mod gun_ui {
    pub(crate) use super::gun::gun_ui::*;
}

pub(crate) mod howitzer_ui {
    pub(crate) use super::howitzer::howitzer_ui::*;
}

pub(crate) mod missile_cannon_ui {
    pub(crate) use super::missile_cannon::missile_cannon_ui::*;
}

pub(crate) use cannon_ui::{
    CannonDeathVisualPolicy, CannonFrameProfile, CannonFrameRole, CannonPlacementAnimation,
    CannonTurrentProfile, INIT_PLACE_FRAME_COUNT, PLACE_FRAME_COUNT,
};
#[cfg(test)]
pub(crate) use cannon_ui::{CannonRenderProfile, PASSIVE_ROTATION_INTERVAL, PLACE_FRAME_TIME};

const PATHING_BLOCK_OFFSETS: &[(u16, u16)] = &[(0, 0)];

pub(crate) fn frame_profile(
    role: CannonFrameRole,
    atlas_team: TeamType,
    asset_path: String,
) -> CannonFrameProfile {
    cannon_ui::frame_profile(role, atlas_team, asset_path)
}

#[cfg(test)]
pub(crate) fn init_place_frame_path(frame: usize) -> String {
    cannon_ui::init_place_frame_path(frame)
}

#[cfg(test)]
pub(crate) fn init_place_frame_profile(frame: usize) -> CannonFrameProfile {
    cannon_ui::init_place_frame_profile(frame)
}

pub(crate) fn placement_frame_profile(
    cannon: CannonType,
    team: TeamType,
    frame: usize,
) -> Option<CannonFrameProfile> {
    cannon_ui::placement_frame_profile(cannon, team, frame)
}

#[cfg(test)]
pub(crate) fn render_profile(cannon: CannonType) -> CannonRenderProfile {
    cannon_ui::render_profile(cannon)
}

pub(crate) fn hud_name(cannon: CannonType) -> &'static str {
    cannon_ui::hud_name(cannon)
}

pub(crate) fn render_offset(cannon: CannonType, direction: usize) -> Vec2 {
    cannon_ui::render_offset(cannon, direction)
}

#[cfg(test)]
pub(crate) fn empty_frame_profile(
    cannon: CannonType,
    team: TeamType,
    rotation: u16,
) -> CannonFrameProfile {
    cannon_ui::empty_frame_profile(cannon, team, rotation)
}

#[cfg(test)]
pub(crate) fn passive_frame_profile(
    cannon: CannonType,
    team: TeamType,
    rotation: u16,
) -> CannonFrameProfile {
    cannon_ui::passive_frame_profile(cannon, team, rotation)
}

pub(crate) fn spawn_frame_profile(
    cannon: CannonType,
    team: TeamType,
    rotation: u16,
) -> CannonFrameProfile {
    cannon_ui::spawn_frame_profile(cannon, team, rotation)
}

pub(crate) fn captured_frame_profile(
    cannon: CannonType,
    team: TeamType,
    rotation: u16,
) -> CannonFrameProfile {
    cannon_ui::captured_frame_profile(cannon, team, rotation)
}

#[cfg(test)]
pub(crate) fn fire_frame_profile(
    cannon: CannonType,
    team: TeamType,
    rotation: u16,
) -> CannonFrameProfile {
    cannon_ui::fire_frame_profile(cannon, team, rotation)
}

#[cfg(test)]
pub(crate) fn equipped_frame_profile(
    cannon: CannonType,
    team: TeamType,
    rotation: u16,
) -> Option<CannonFrameProfile> {
    cannon_ui::equipped_frame_profile(cannon, team, rotation)
}

#[cfg(test)]
pub(crate) fn place_frame_profile(
    cannon: CannonType,
    team: TeamType,
    frame: usize,
) -> CannonFrameProfile {
    cannon_ui::place_frame_profile(cannon, team, frame)
}

#[cfg(test)]
pub(crate) fn death_wreck_frame_profile(cannon: CannonType) -> CannonFrameProfile {
    cannon_ui::death_wreck_frame_profile(cannon)
}

#[cfg(test)]
pub(crate) fn destroyed_frame_profile(cannon: CannonType, team: TeamType) -> CannonFrameProfile {
    cannon_ui::destroyed_frame_profile(cannon, team)
}

#[cfg(test)]
pub(crate) fn atlas_team_for_frame(
    cannon: CannonType,
    role: CannonFrameRole,
    team: TeamType,
) -> Option<TeamType> {
    cannon_ui::atlas_team_for_frame(cannon, role, team)
}

pub(crate) fn default_selection_size(cannon: CannonType) -> Vec2 {
    cannon_ui::default_selection_size(cannon)
}

pub(crate) fn fallback_collision_size() -> Vec2 {
    cannon_ui::fallback_collision_size()
}

pub(crate) fn pathing_block_offsets() -> &'static [(u16, u16)] {
    PATHING_BLOCK_OFFSETS
}

pub(crate) fn settings(cannon: CannonType) -> UnitSettings {
    match cannon {
        CannonType::Gatling => gatling::settings(),
        CannonType::Gun => gun::settings(),
        CannonType::Howitzer => howitzer::settings(),
        CannonType::MissileCannon => missile_cannon::settings(),
    }
}

pub(crate) fn attack_sound(cannon: CannonType) -> Option<UnitAttackSound> {
    match cannon {
        CannonType::Gatling => gatling::attack_sound(),
        CannonType::Gun => gun::attack_sound(),
        CannonType::Howitzer => howitzer::attack_sound(),
        CannonType::MissileCannon => missile_cannon::attack_sound(),
    }
}

pub(crate) fn damage_missile_visual(cannon: CannonType) -> Option<DamageMissileVisual> {
    cannon_ui::damage_missile_visual(cannon)
}

#[cfg(test)]
pub(crate) fn death_wreck_asset_path(cannon: CannonType) -> Option<String> {
    cannon_ui::death_wreck_asset_path(cannon)
}

pub(crate) fn death_visual_policy(
    cannon: CannonType,
    team: TeamType,
    center: Vec2,
    end_map: Vec2,
    missile_offset_time: f32,
    rng: &mut CombatRng,
) -> Option<CannonDeathVisualPolicy> {
    cannon_ui::death_visual_policy(cannon, team, center, end_map, missile_offset_time, rng)
}

pub(crate) fn destroyed_asset_path(cannon: CannonType, team: TeamType) -> Option<String> {
    cannon_ui::destroyed_asset_path(cannon, team)
}

#[cfg(test)]
pub(crate) fn death_top_left_world_for(cannon: CannonType, center: Vec2) -> Vec2 {
    cannon_ui::death_top_left_world_for(cannon, center)
}

#[cfg(test)]
pub(crate) fn death_top_left_world(center: Vec2) -> Vec2 {
    death_top_left_world_for(CannonType::Gatling, center)
}

#[cfg(test)]
pub(crate) fn death_delay_for(cannon: CannonType, rng: &mut CombatRng) -> f32 {
    cannon_ui::death_delay_for(cannon, rng)
}

pub(crate) fn death_spark_count_for(cannon: CannonType, rng: &mut CombatRng) -> usize {
    cannon_ui::death_spark_count_for(cannon, rng)
}

pub(crate) fn death_missile_offset_time_for(cannon: CannonType, rng: &mut CombatRng) -> f32 {
    cannon_ui::death_missile_offset_time_for(cannon, rng)
}

pub(crate) fn turrent_profile(cannon: CannonType) -> CannonTurrentProfile {
    cannon_ui::turrent_profile(cannon)
}

pub(crate) fn turrent_target_offset_for(cannon: CannonType, rng: &mut CombatRng) -> Vec2 {
    cannon_ui::turrent_target_offset_for(cannon, rng)
}

#[cfg(test)]
pub(crate) fn turrent_rise(cannon: CannonType, rng: &mut CombatRng) -> f32 {
    cannon_ui::turrent_rise(cannon, rng)
}

pub(crate) fn turrent_start_jitter(cannon: CannonType, rng: &mut CombatRng) -> Vec2 {
    cannon_ui::turrent_start_jitter(cannon, rng)
}

pub(crate) fn turrent_spin_degrees_per_sec(cannon: CannonType, rng: &mut CombatRng) -> f32 {
    cannon_ui::turrent_spin_degrees_per_sec(cannon, rng)
}

pub(crate) fn turrent_arc_lift_pixels(cannon: CannonType) -> f32 {
    turrent_profile(cannon).arc_lift_pixels
}

pub(crate) fn turrent_arc_size_for(cannon: CannonType, rise: f32, final_time: f32, t: f32) -> f32 {
    cannon_ui::turrent_arc_size_for(cannon, rise, final_time, t)
}

pub(crate) fn turrent_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    turrent_arc_size_for(CannonType::Gatling, rise, final_time, t)
}

#[cfg(test)]
pub(crate) fn rocket_muzzle_offset(cannon: CannonType, direction: usize) -> Option<Vec2> {
    cannon_ui::rocket_muzzle_offset(cannon, direction)
}

#[cfg(test)]
pub(crate) fn direct_fire_muzzle_offset(cannon: CannonType, direction: usize) -> Option<Vec2> {
    cannon_ui::direct_fire_muzzle_offset(cannon, direction)
}

#[cfg(test)]
pub(crate) fn fire_flash_duration(cannon: CannonType, rng: &mut CombatRng) -> Option<f32> {
    cannon_ui::fire_flash_duration(cannon, rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_CANNONS: [CannonType; 4] = [
        CannonType::Gatling,
        CannonType::Gun,
        CannonType::Howitzer,
        CannonType::MissileCannon,
    ];

    #[test]
    fn cannon_frame_profiles_match_original_asset_families() {
        assert_eq!(init_place_frame_path(2), "units/cannons/init-place_n02.png");
        let init_place = init_place_frame_profile(99);
        assert_eq!(init_place.role, CannonFrameRole::Place);
        assert_eq!(init_place.atlas_team, TeamType::Red);
        assert_eq!(init_place.asset_path, "units/cannons/init-place_n02.png");
        assert_eq!(init_place.atlas_frame_name, "cannon_init_place_n02");

        let gatling_passive = passive_frame_profile(CannonType::Gatling, TeamType::Blue, 180);
        assert_eq!(gatling_passive.role, CannonFrameRole::Passive);
        assert_eq!(gatling_passive.atlas_team, TeamType::Blue);
        assert_eq!(
            gatling_passive.asset_path,
            "units/cannons/gatling/fire_blue_r180_n00.png"
        );
        assert_eq!(
            gatling_passive.atlas_frame_name,
            "cannon_gatling_fire_blue_r180_n00"
        );
        assert_eq!(
            fire_frame_profile(CannonType::Gatling, TeamType::Yellow, 45).asset_path,
            "units/cannons/gatling/fire_yellow_r045_n01.png"
        );
        assert_eq!(
            spawn_frame_profile(CannonType::Gatling, TeamType::Blue, 180),
            empty_frame_profile(CannonType::Gatling, TeamType::Blue, 180)
        );
        assert_eq!(
            captured_frame_profile(CannonType::Gatling, TeamType::Blue, 180).atlas_team,
            TeamType::Blue
        );

        let gun_equipped = equipped_frame_profile(CannonType::Gun, TeamType::Green, 225).unwrap();
        assert_eq!(gun_equipped.role, CannonFrameRole::Equipped);
        assert_eq!(
            gun_equipped.asset_path,
            "units/cannons/gun/equiped_green_r225.png"
        );
        assert_eq!(
            spawn_frame_profile(CannonType::Gun, TeamType::Green, 225),
            gun_equipped
        );
        let captured_gun = captured_frame_profile(CannonType::Gun, TeamType::Green, 225);
        assert_eq!(captured_gun.role, CannonFrameRole::Passive);
        assert_eq!(captured_gun.asset_path, gun_equipped.asset_path);
        assert_eq!(
            spawn_frame_profile(CannonType::Gun, TeamType::Null, 225).asset_path,
            "units/cannons/gun/empty.png"
        );
        assert_eq!(
            passive_frame_profile(CannonType::Gun, TeamType::Null, 225).asset_path,
            "units/cannons/gun/empty.png"
        );

        assert_eq!(
            passive_frame_profile(CannonType::Howitzer, TeamType::Red, 315).asset_path,
            "units/cannons/howitzer/fire_red_r315_n00.png"
        );
        assert_eq!(
            fire_frame_profile(CannonType::Howitzer, TeamType::Red, 315).asset_path,
            "units/cannons/howitzer/fire_red_r315_n01.png"
        );

        assert_eq!(
            empty_frame_profile(CannonType::MissileCannon, TeamType::Null, 90).asset_path,
            "units/cannons/missile_cannon/empty_null.png"
        );
        assert_eq!(
            empty_frame_profile(CannonType::MissileCannon, TeamType::Blue, 90).asset_path,
            "units/cannons/missile_cannon/empty_blue_r090.png"
        );
        assert_eq!(
            equipped_frame_profile(CannonType::MissileCannon, TeamType::Blue, 90)
                .unwrap()
                .asset_path,
            "units/cannons/missile_cannon/equiped_blue_r090.png"
        );
        assert_eq!(
            captured_frame_profile(CannonType::MissileCannon, TeamType::Null, 90).asset_path,
            "units/cannons/missile_cannon/empty_null.png"
        );
        assert_eq!(
            place_frame_profile(CannonType::Howitzer, TeamType::Green, 99).asset_path,
            "units/cannons/howitzer/place_green_n03.png"
        );
    }

    #[test]
    fn cannon_placement_sequence_uses_three_shared_and_four_unit_frames() {
        for cannon in ALL_CANNONS {
            assert_eq!(
                placement_frame_profile(cannon, TeamType::Blue, 0)
                    .unwrap()
                    .asset_path,
                "units/cannons/init-place_n00.png"
            );
            assert_eq!(
                placement_frame_profile(cannon, TeamType::Blue, 2)
                    .unwrap()
                    .asset_path,
                "units/cannons/init-place_n02.png"
            );
            assert!(
                placement_frame_profile(cannon, TeamType::Blue, 3)
                    .unwrap()
                    .asset_path
                    .ends_with("_blue_n00.png")
            );
            assert!(
                placement_frame_profile(cannon, TeamType::Blue, 6)
                    .unwrap()
                    .asset_path
                    .ends_with("_blue_n03.png")
            );
            assert!(placement_frame_profile(cannon, TeamType::Blue, 7).is_none());
        }

        let mut animation = CannonPlacementAnimation::new(
            CannonType::Howitzer,
            TeamType::Blue,
            Vec2::new(32.0, 48.0),
        );
        assert!(!animation.process(1.0));
        assert_eq!(animation.frame, 0);
        assert_eq!(animation.elapsed, 0.0);
        assert!(!animation.process(0.09));
        assert_eq!(animation.frame, 0);
        assert!(!animation.process(0.02));
        assert_eq!(animation.frame, 1);
        assert_eq!(animation.elapsed, 0.0);
        for _ in 0..5 {
            assert!(!animation.process(1.0));
        }
        assert!(animation.process(1.0));
        assert_eq!(animation.frame, 7);
    }

    #[test]
    fn cannon_atlas_team_choice_is_owned_by_cannon_ui_profiles() {
        assert_eq!(
            atlas_team_for_frame(CannonType::Gatling, CannonFrameRole::Empty, TeamType::Blue),
            Some(TeamType::Red)
        );
        assert_eq!(
            atlas_team_for_frame(
                CannonType::Gatling,
                CannonFrameRole::Passive,
                TeamType::Blue
            ),
            Some(TeamType::Blue)
        );
        assert_eq!(
            atlas_team_for_frame(CannonType::Gun, CannonFrameRole::Equipped, TeamType::White),
            Some(TeamType::Red)
        );
        assert_eq!(
            atlas_team_for_frame(CannonType::Gun, CannonFrameRole::Equipped, TeamType::Null),
            None
        );
        assert_eq!(
            atlas_team_for_frame(
                CannonType::MissileCannon,
                CannonFrameRole::Empty,
                TeamType::Blue
            ),
            Some(TeamType::Blue)
        );
        assert_eq!(
            atlas_team_for_frame(
                CannonType::MissileCannon,
                CannonFrameRole::Empty,
                TeamType::Null
            ),
            Some(TeamType::Red)
        );
        assert_eq!(
            atlas_team_for_frame(
                CannonType::MissileCannon,
                CannonFrameRole::Destroyed,
                TeamType::Yellow
            ),
            Some(TeamType::Yellow)
        );
    }

    #[test]
    fn cannon_render_profiles_keep_original_offsets_and_timing() {
        assert_eq!(
            cannon_ui::CANNON_ROTATIONS,
            [0, 45, 90, 135, 180, 225, 270, 315]
        );
        assert_eq!(cannon_ui::INIT_PLACE_FRAME_COUNT, 3);
        assert_eq!(PLACE_FRAME_COUNT, 4);
        assert_eq!(PLACE_FRAME_TIME, 0.1);
        assert_eq!(PASSIVE_ROTATION_INTERVAL, 1.0);

        assert_eq!(render_offset(CannonType::Gatling, 0), Vec2::new(1.0, -7.0));
        assert_eq!(render_offset(CannonType::Howitzer, 5), Vec2::new(0.0, -9.0));
        assert_eq!(
            render_offset(CannonType::MissileCannon, 3),
            Vec2::new(0.0, -8.0)
        );

        let gatling = render_profile(CannonType::Gatling);
        assert_eq!(gatling.place_frames, PLACE_FRAME_COUNT);
        assert_eq!(gatling.place_frame_time, PLACE_FRAME_TIME);
        assert_eq!(gatling.passive_rotation_interval, PASSIVE_ROTATION_INTERVAL);
        assert_eq!(gatling.fire_flash_base_time, Some(0.07));
        assert_eq!(render_profile(CannonType::Gun).fire_flash_base_time, None);
        assert_eq!(
            render_profile(CannonType::Howitzer).fire_flash_base_time,
            Some(0.05)
        );
    }

    #[test]
    fn cannon_death_assets_and_top_left_match_original_effect() {
        assert_eq!(
            death_wreck_asset_path(CannonType::Gatling).unwrap(),
            "units/cannons/gatling/wasted.png"
        );
        assert_eq!(
            death_wreck_asset_path(CannonType::Gun).unwrap(),
            "units/cannons/gun/wasted.png"
        );
        assert_eq!(
            death_wreck_asset_path(CannonType::Howitzer).unwrap(),
            "units/cannons/howitzer/wasted.png"
        );
        assert_eq!(
            death_wreck_asset_path(CannonType::MissileCannon).unwrap(),
            "units/cannons/missile_cannon/wasted.png"
        );
        assert_eq!(
            destroyed_asset_path(CannonType::MissileCannon, TeamType::Blue).unwrap(),
            "units/cannons/missile_cannon/wasted_blue.png"
        );
        assert_eq!(
            destroyed_frame_profile(CannonType::MissileCannon, TeamType::Yellow).atlas_team,
            TeamType::Yellow
        );
        assert_eq!(
            death_wreck_frame_profile(CannonType::Gun).role,
            CannonFrameRole::DeathWreck
        );

        for cannon in ALL_CANNONS {
            assert_eq!(
                death_top_left_world_for(cannon, Vec2::new(100.0, -50.0)),
                Vec2::new(84.0, -34.0)
            );
        }
        assert_eq!(
            death_top_left_world(Vec2::new(100.0, -50.0)),
            Vec2::new(84.0, -34.0)
        );
    }

    #[test]
    fn cannon_death_timing_and_sparks_match_original_ranges() {
        let mut rng = CombatRng::default();
        for cannon in ALL_CANNONS {
            let profile = turrent_profile(cannon);
            assert_eq!(profile.target_center_offset, Vec2::splat(16.0));
            assert_eq!(profile.max_horizontal_distance, 300.0);
            assert_eq!(profile.max_vertical_distance, 300.0);
            assert_eq!(profile.damage, 40);
            assert_eq!(profile.radius, 40);
            assert_eq!(profile.arc_lift_pixels, 30.0);

            for _ in 0..16 {
                let delay = death_delay_for(cannon, &mut rng);
                assert!((2.0..=4.0).contains(&delay));

                let count = death_spark_count_for(cannon, &mut rng);
                assert!((20..=34).contains(&count));

                let offset_time = death_missile_offset_time_for(cannon, &mut rng);
                assert!((7.0..10.0).contains(&offset_time));

                let target = turrent_target_offset_for(cannon, &mut rng);
                assert!(target.x > -profile.max_horizontal_distance);
                assert!(target.x <= profile.max_horizontal_distance);
                assert!(target.y > -profile.max_vertical_distance);
                assert!(target.y <= profile.max_vertical_distance);

                let rise = turrent_rise(cannon, &mut rng);
                assert!((1.0..4.0).contains(&rise));

                let jitter = turrent_start_jitter(cannon, &mut rng);
                assert!(jitter.x > -5.0 && jitter.x <= 5.0);
                assert!(jitter.y > -5.0 && jitter.y <= 5.0);

                let spin = turrent_spin_degrees_per_sec(cannon, &mut rng);
                assert!((-240.0..=240.0).contains(&spin));
            }
        }
    }

    #[test]
    fn cannon_turrent_arc_matches_original_parabola() {
        assert_eq!(turrent_arc_size(2.0, 4.0, 0.0), 1.0);
        for cannon in ALL_CANNONS {
            assert_eq!(turrent_arc_size_for(cannon, 2.0, 4.0, 0.0), 1.0);
            assert!(turrent_arc_size_for(cannon, 2.0, 4.0, 1.0) > 1.0);
            assert_eq!(turrent_arc_lift_pixels(cannon), 30.0);
        }
    }

    #[test]
    fn cannon_projectile_muzzle_offsets_live_on_cannon_ui_modules() {
        assert_eq!(
            rocket_muzzle_offset(CannonType::Gun, 0),
            Some(Vec2::new(21.0, 2.0))
        );
        assert_eq!(
            rocket_muzzle_offset(CannonType::Howitzer, 2),
            Some(Vec2::new(1.0, 22.0))
        );
        assert_eq!(
            rocket_muzzle_offset(CannonType::MissileCannon, 4),
            Some(Vec2::new(-19.0, 2.0))
        );
        assert_eq!(rocket_muzzle_offset(CannonType::Gatling, 0), None);

        assert_eq!(
            direct_fire_muzzle_offset(CannonType::Gatling, 0),
            Some(Vec2::new(18.0, 10.0))
        );
        assert_eq!(
            direct_fire_muzzle_offset(CannonType::Gatling, 6),
            Some(Vec2::new(-1.0, -6.0))
        );
        assert_eq!(direct_fire_muzzle_offset(CannonType::Gun, 0), None);

        let mut rng = CombatRng::default();
        assert!((0.07..0.1).contains(&fire_flash_duration(CannonType::Gatling, &mut rng).unwrap()));
        assert!(fire_flash_duration(CannonType::Gun, &mut rng).is_none());
        assert!(
            (0.05..0.08).contains(&fire_flash_duration(CannonType::Howitzer, &mut rng).unwrap())
        );
    }
}
