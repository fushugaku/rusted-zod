use bevy::prelude::Vec2;

use crate::original::types::{PlanetType, TeamType};

use crate::units::buildings::{
    BuildingAtlasFrameSpec, BuildingDeathProfile, BuildingEffectBox, BuildingPathingSpec,
    ProductionPlacement, building_ui::planet_asset_name,
};

#[cfg(test)]
pub(crate) const DESTROYED_OVERLAY_PATH: &str = "buildings/fort/destroyed_overlay.png";
#[cfg(test)]
pub(crate) const DESTROYED_OVERLAY_ALPHA_MIN: u8 = 1;
#[cfg(test)]
pub(crate) const DESTROYED_OVERLAY_ALPHA_MAX: u8 = 254;

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(64.0)
}

pub(crate) fn pathing_spec() -> BuildingPathingSpec {
    BuildingPathingSpec {
        blocked_rects: Vec::new(),
        blocked_masks: vec![&[
            ".##....##.",
            ".########.",
            ".########.",
            "##########",
            "##########",
            "##########",
            "####..####",
            ".###..###.",
            "..##..##..",
            "...#..#...",
        ]],
        unblocked_tiles: Vec::new(),
    }
}

pub(crate) fn production_placement() -> ProductionPlacement {
    ProductionPlacement {
        create_offset: Vec2::new(80.0, 128.0),
        move_offset: Vec2::new(80.0, 208.0),
    }
}

pub(crate) fn production_label_asset_path() -> &'static str {
    "other/production_gui/fort_factory_label.png"
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
    format!("fort_{}_front", planet_asset_name(planet))
}

pub(crate) fn flag_frame_names(team: TeamType) -> Vec<String> {
    let team_name = team.atlas_team().asset_name();
    (0..4)
        .map(|frame| format!("fort_flag_{team_name}_n{frame:02}"))
        .collect()
}

pub(crate) fn flag_world_offset() -> Vec2 {
    Vec2::new(85.0, 29.0)
}

pub(crate) fn entrance_rect() -> (f32, f32, f32, f32) {
    (64.0, 32.0, 32.0, 96.0)
}

pub(crate) fn destroyed_asset_path(planet: PlanetType) -> String {
    format!(
        "buildings/fort/fort_{}_front_destroyed.png",
        planet_asset_name(planet)
    )
}

pub(crate) fn death_profile(planet: PlanetType) -> BuildingDeathProfile {
    BuildingDeathProfile {
        effect_box: BuildingEffectBox {
            x: 18.0,
            y: 18.0,
            width: 136,
            height: 118,
        },
        width_pix: 160.0,
        height_pix: match planet {
            PlanetType::Jungle => 176.0,
            PlanetType::Desert | PlanetType::Volcanic | PlanetType::Arctic | PlanetType::City => {
                192.0
            }
        },
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
                height_pix: 192.0,
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
        assert_eq!(death_profile(PlanetType::Jungle).height_pix, 176.0);
    }

    #[test]
    fn destroyed_assets_and_overlay_match_original_paths() {
        assert_eq!(
            destroyed_asset_path(PlanetType::Desert),
            "buildings/fort/fort_desert_front_destroyed.png"
        );
        assert_eq!(
            DESTROYED_OVERLAY_PATH,
            "buildings/fort/destroyed_overlay.png"
        );
        assert_eq!(DESTROYED_OVERLAY_ALPHA_MIN, 1);
        assert_eq!(DESTROYED_OVERLAY_ALPHA_MAX, 254);
    }

    #[test]
    fn atlas_and_flag_specs_match_original_frames() {
        assert_eq!(
            base_atlas_frame_name(PlanetType::Volcanic),
            "fort_volcanic_front"
        );
        assert_eq!(
            flag_frame_names(TeamType::Blue),
            vec![
                "fort_flag_blue_n00".to_string(),
                "fort_flag_blue_n01".to_string(),
                "fort_flag_blue_n02".to_string(),
                "fort_flag_blue_n03".to_string()
            ]
        );
        assert_eq!(flag_world_offset(), Vec2::new(85.0, 29.0));

        let specs = atlas_layer_specs(TeamType::Black, PlanetType::City);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].atlas_team, TeamType::Red);
        assert_eq!(specs[0].frame_name, "fort_city_front");
        assert_eq!(specs[1].atlas_team, TeamType::Red);
        assert_eq!(specs[1].animation_frame_names.len(), 4);
    }

    #[test]
    fn entrance_rect_matches_original_front_zone() {
        assert_eq!(entrance_rect(), (64.0, 32.0, 32.0, 96.0));
    }
}
