use bevy::prelude::Vec2;

use crate::{
    components::{CombatRng, DamageMissileVisual},
    original::{objects::CannonType, settings::UnitSettings, types::TeamType},
    units::UnitAttackSound,
};

pub(crate) mod gatling;
pub(crate) mod gatling_ui;
pub(crate) mod gun;
pub(crate) mod gun_ui;
pub(crate) mod howitzer;
pub(crate) mod howitzer_ui;
pub(crate) mod missile_cannon;
pub(crate) mod missile_cannon_ui;

pub(crate) const CANNON_ROTATIONS: [u16; 8] = [0, 45, 90, 135, 180, 225, 270, 315];
pub(crate) const INIT_PLACE_FRAME_COUNT: usize = 3;
pub(crate) const PLACE_FRAME_COUNT: usize = 4;
pub(crate) const PLACE_FRAME_TIME: f32 = 0.1;
pub(crate) const PASSIVE_ROTATION_INTERVAL: f32 = 1.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CannonFrameRole {
    Empty,
    Passive,
    Fire,
    Equipped,
    Place,
    DeathWreck,
    Destroyed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CannonFrameProfile {
    pub(crate) role: CannonFrameRole,
    pub(crate) atlas_team: TeamType,
    pub(crate) atlas_frame_name: String,
    pub(crate) asset_path: String,
}

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

fn frame_profile(
    role: CannonFrameRole,
    atlas_team: TeamType,
    asset_path: String,
) -> CannonFrameProfile {
    let atlas_frame_name = asset_path
        .strip_suffix(".png")
        .unwrap_or(&asset_path)
        .strip_prefix("units/cannons/")
        .unwrap_or(&asset_path)
        .replace('/', "_");
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

pub(crate) fn render_profile(cannon: CannonType) -> CannonRenderProfile {
    match cannon {
        CannonType::Gatling => gatling_ui::render_profile(),
        CannonType::Gun => gun_ui::render_profile(),
        CannonType::Howitzer => howitzer_ui::render_profile(),
        CannonType::MissileCannon => missile_cannon_ui::render_profile(),
    }
}

pub(crate) fn render_offset(cannon: CannonType, direction: usize) -> Vec2 {
    let profile = render_profile(cannon);
    profile.unit_offset + profile.direction_offsets[direction.min(7)]
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

pub(crate) fn death_wreck_frame_profile(cannon: CannonType) -> CannonFrameProfile {
    match cannon {
        CannonType::Gatling => gatling_ui::death_wreck_frame_profile(),
        CannonType::Gun => gun_ui::death_wreck_frame_profile(),
        CannonType::Howitzer => howitzer_ui::death_wreck_frame_profile(),
        CannonType::MissileCannon => missile_cannon_ui::death_wreck_frame_profile(),
    }
}

pub(crate) fn destroyed_frame_profile(cannon: CannonType, team: TeamType) -> CannonFrameProfile {
    match cannon {
        CannonType::Gatling => gatling_ui::destroyed_frame_profile(),
        CannonType::Gun => gun_ui::destroyed_frame_profile(),
        CannonType::Howitzer => howitzer_ui::destroyed_frame_profile(),
        CannonType::MissileCannon => missile_cannon_ui::destroyed_frame_profile(team),
    }
}

pub(crate) fn default_selection_size(cannon: CannonType) -> Vec2 {
    match cannon {
        CannonType::Gatling => gatling::default_selection_size(),
        CannonType::Gun => gun::default_selection_size(),
        CannonType::Howitzer => howitzer::default_selection_size(),
        CannonType::MissileCannon => missile_cannon::default_selection_size(),
    }
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
    match cannon {
        CannonType::Gun => gun::damage_missile_visual(),
        CannonType::Howitzer => howitzer::damage_missile_visual(),
        CannonType::MissileCannon => missile_cannon::damage_missile_visual(),
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

pub(crate) fn death_top_left_world(center: Vec2) -> Vec2 {
    death_top_left_world_for(CannonType::Gatling, center)
}

pub(crate) fn death_delay_for(cannon: CannonType, rng: &mut CombatRng) -> f32 {
    match cannon {
        CannonType::Gatling => gatling_ui::death_delay(rng),
        CannonType::Gun => gun_ui::death_delay(rng),
        CannonType::Howitzer => howitzer_ui::death_delay(rng),
        CannonType::MissileCannon => missile_cannon_ui::death_delay(rng),
    }
}

pub(crate) fn death_delay(rng: &mut CombatRng) -> f32 {
    death_delay_for(CannonType::Gatling, rng)
}

pub(crate) fn death_spark_count_for(cannon: CannonType, rng: &mut CombatRng) -> usize {
    match cannon {
        CannonType::Gatling => gatling_ui::death_spark_count(rng),
        CannonType::Gun => gun_ui::death_spark_count(rng),
        CannonType::Howitzer => howitzer_ui::death_spark_count(rng),
        CannonType::MissileCannon => missile_cannon_ui::death_spark_count(rng),
    }
}

pub(crate) fn death_spark_count(rng: &mut CombatRng) -> usize {
    death_spark_count_for(CannonType::Gatling, rng)
}

pub(crate) fn death_missile_offset_time_for(cannon: CannonType, rng: &mut CombatRng) -> f32 {
    match cannon {
        CannonType::Gatling => gatling_ui::death_missile_offset_time(rng),
        CannonType::Gun => gun_ui::death_missile_offset_time(rng),
        CannonType::Howitzer => howitzer_ui::death_missile_offset_time(rng),
        CannonType::MissileCannon => missile_cannon_ui::death_missile_offset_time(rng),
    }
}

pub(crate) fn death_missile_offset_time(rng: &mut CombatRng) -> f32 {
    death_missile_offset_time_for(CannonType::Gatling, rng)
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

pub(crate) fn turrent_target_offset(rng: &mut CombatRng) -> Vec2 {
    turrent_target_offset_for(CannonType::Gatling, rng)
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

pub(crate) fn turrent_arc_lift_pixels(cannon: CannonType) -> f32 {
    turrent_profile(cannon).arc_lift_pixels
}

pub(crate) fn turrent_arc_size_for(cannon: CannonType, rise: f32, final_time: f32, t: f32) -> f32 {
    match cannon {
        CannonType::Gatling => gatling_ui::turrent_arc_size(rise, final_time, t),
        CannonType::Gun => gun_ui::turrent_arc_size(rise, final_time, t),
        CannonType::Howitzer => howitzer_ui::turrent_arc_size(rise, final_time, t),
        CannonType::MissileCannon => missile_cannon_ui::turrent_arc_size(rise, final_time, t),
    }
}

pub(crate) fn turrent_arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    turrent_arc_size_for(CannonType::Gatling, rise, final_time, t)
}

pub(crate) fn rocket_muzzle_offset(cannon: CannonType, direction: usize) -> Option<Vec2> {
    match cannon {
        CannonType::Gatling => None,
        CannonType::Gun => Some(gun::rocket_muzzle_offset(direction)),
        CannonType::Howitzer => Some(howitzer::rocket_muzzle_offset(direction)),
        CannonType::MissileCannon => Some(missile_cannon::rocket_muzzle_offset(direction)),
    }
}

pub(crate) fn direct_fire_muzzle_offset(cannon: CannonType, direction: usize) -> Option<Vec2> {
    match cannon {
        CannonType::Gatling => Some(gatling::direct_fire_muzzle_offset(direction)),
        CannonType::Gun | CannonType::Howitzer | CannonType::MissileCannon => None,
    }
}

pub(crate) fn fire_flash_duration(cannon: CannonType, rng: &mut CombatRng) -> Option<f32> {
    match cannon {
        CannonType::Gatling => gatling_ui::fire_flash_duration(rng),
        CannonType::Gun => gun_ui::fire_flash_duration(rng),
        CannonType::Howitzer => howitzer_ui::fire_flash_duration(rng),
        CannonType::MissileCannon => missile_cannon_ui::fire_flash_duration(rng),
    }
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

        let gun_equipped = equipped_frame_profile(CannonType::Gun, TeamType::Green, 225).unwrap();
        assert_eq!(gun_equipped.role, CannonFrameRole::Equipped);
        assert_eq!(
            gun_equipped.asset_path,
            "units/cannons/gun/equiped_green_r225.png"
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
            place_frame_profile(CannonType::Howitzer, TeamType::Green, 3).asset_path,
            "units/cannons/howitzer/place_green_n03.png"
        );
    }

    #[test]
    fn cannon_render_profiles_keep_original_offsets_and_timing() {
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
    fn cannon_projectile_muzzle_offsets_live_on_cannon_modules() {
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
