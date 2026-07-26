use bevy::prelude::Vec2;

use crate::original::types::{PlanetType, TeamType};

use crate::units::buildings::{
    BuildingAtlasFrameSpec, BuildingDeathProfile, BuildingEffectBox, BuildingPathingSpec,
    ProductionPlacement, building_ui::planet_asset_name,
};

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(64.0)
}

pub(crate) fn pathing_spec() -> BuildingPathingSpec {
    BuildingPathingSpec {
        blocked_rects: Vec::new(),
        blocked_masks: vec![&[
            ".##....##.",
            ".###..###.",
            ".###..###.",
            "##########",
            "##########",
            "##########",
            "##########",
            ".########.",
            "..#....#..",
            "..........",
        ]],
        unblocked_tiles: Vec::new(),
    }
}

pub(crate) fn production_placement() -> ProductionPlacement {
    ProductionPlacement {
        create_offset: Vec2::new(80.0, 32.0),
        move_offset: Vec2::new(80.0, -16.0),
    }
}

#[cfg(test)]
pub(crate) fn production_label_asset_path() -> &'static str {
    crate::units::buildings::fort_front_ui::production_label_asset_path()
}

pub(crate) fn atlas_layer_specs(team: TeamType, planet: PlanetType) -> Vec<BuildingAtlasFrameSpec> {
    let team = team.atlas_team();
    let flag_frames = flag_frame_names(team);
    vec![
        BuildingAtlasFrameSpec {
            atlas_team: TeamType::Red,
            frame_name: base_atlas_frame_name(planet),
            world_offset: Vec2::ZERO,
            animation_frame_names: Vec::new(),
        },
        BuildingAtlasFrameSpec {
            atlas_team: team,
            frame_name: flag_frames[0].clone(),
            world_offset: flag_world_offset(),
            animation_frame_names: flag_frames,
        },
    ]
}

pub(crate) fn base_atlas_frame_name(planet: PlanetType) -> String {
    format!("fort_{}_back", planet_asset_name(planet))
}

pub(crate) fn flag_frame_names(team: TeamType) -> Vec<String> {
    crate::units::buildings::fort_front_ui::flag_frame_names(team)
}

pub(crate) fn flag_world_offset() -> Vec2 {
    crate::units::buildings::fort_front_ui::flag_world_offset()
}

pub(crate) fn entrance_rect() -> (f32, f32, f32, f32) {
    (64.0, 16.0, 32.0, 64.0)
}

pub(crate) fn destroyed_asset_path(planet: PlanetType) -> String {
    format!(
        "buildings/fort/fort_{}_back_destroyed.png",
        planet_asset_name(planet)
    )
}

pub(crate) fn death_profile(_planet: PlanetType) -> BuildingDeathProfile {
    BuildingDeathProfile {
        effect_box: BuildingEffectBox {
            x: 18.0,
            y: 18.0,
            width: 136,
            height: 118,
        },
        width_pix: 160.0,
        height_pix: 176.0,
        max_effects_base: 20,
        max_effects_random: 8,
        fireball_base: 12,
        fireball_random: 6,
        piece_base: 16,
        piece_random: 6,
        piece_variants: 5,
        piece_flight_base: 3.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn death_profile_matches_original_do_death_effect() {
        assert_eq!(
            death_profile(PlanetType::Desert),
            BuildingDeathProfile {
                effect_box: BuildingEffectBox {
                    x: 18.0,
                    y: 18.0,
                    width: 136,
                    height: 118
                },
                width_pix: 160.0,
                height_pix: 176.0,
                max_effects_base: 20,
                max_effects_random: 8,
                fireball_base: 12,
                fireball_random: 6,
                piece_base: 16,
                piece_random: 6,
                piece_variants: 5,
                piece_flight_base: 3.0
            }
        );
    }

    #[test]
    fn destroyed_asset_matches_original_path() {
        assert_eq!(
            destroyed_asset_path(PlanetType::City),
            "buildings/fort/fort_city_back_destroyed.png"
        );
    }

    #[test]
    fn atlas_and_flag_specs_match_original_frames() {
        assert_eq!(
            base_atlas_frame_name(PlanetType::Arctic),
            "fort_arctic_back"
        );
        assert_eq!(
            flag_frame_names(TeamType::Yellow)
                .last()
                .map(String::as_str),
            Some("fort_flag_yellow_n03")
        );
        assert_eq!(flag_world_offset(), Vec2::new(85.0, 29.0));

        let specs = atlas_layer_specs(TeamType::Yellow, PlanetType::Desert);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].atlas_team, TeamType::Red);
        assert_eq!(specs[0].frame_name, "fort_desert_back");
        assert_eq!(specs[1].atlas_team, TeamType::Yellow);
        assert_eq!(specs[1].animation_frame_names.len(), 4);
    }

    #[test]
    fn entrance_rect_matches_original_back_zone() {
        assert_eq!(entrance_rect(), (64.0, 16.0, 32.0, 64.0));
    }
}
