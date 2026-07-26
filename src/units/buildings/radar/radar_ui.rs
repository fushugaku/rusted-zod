use bevy::prelude::Vec2;

use crate::{
    components::ObjectStats,
    original::types::{PlanetType, TeamType},
    render::atlas::RadarOverlayKind,
};

use crate::units::buildings::{
    BuildingAtlasFrameSpec, BuildingDeathProfile, BuildingEffectBox, BuildingOverlayFrameSpec,
    BuildingPathingRect, BuildingPathingSpec, building_ui::planet_asset_name,
};

pub(crate) const OVERLAY_FRAME_TIME: f32 = 0.25;
pub(crate) const OVERLAY_KINDS: [RadarOverlayKind; 4] = [
    RadarOverlayKind::FrontLight,
    RadarOverlayKind::SideLight,
    RadarOverlayKind::BoxSpinner,
    RadarOverlayKind::Dish,
];

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(64.0)
}

pub(crate) fn pathing_spec() -> BuildingPathingSpec {
    BuildingPathingSpec {
        blocked_rects: vec![BuildingPathingRect {
            dx: 0,
            dy: 0,
            width: 4,
            height: 3,
        }],
        blocked_masks: Vec::new(),
        unblocked_tiles: vec![(3, 2)],
    }
}

pub(crate) fn crane_repair_local_points(size: Vec2) -> (Vec2, Vec2) {
    (Vec2::new(28.0, size.y + 32.0), Vec2::new(28.0, 24.0))
}

pub(crate) fn atlas_layer_specs(team: TeamType, planet: PlanetType) -> Vec<BuildingAtlasFrameSpec> {
    let team = team.atlas_team();
    vec![
        BuildingAtlasFrameSpec {
            atlas_team: TeamType::Red,
            frame_name: base_atlas_frame_name(planet),
            world_offset: Vec2::ZERO,
            animation_frame_names: Vec::new(),
        },
        BuildingAtlasFrameSpec {
            atlas_team: team,
            frame_name: team_overlay_frame_name(team),
            world_offset: team_overlay_world_offset(),
            animation_frame_names: Vec::new(),
        },
    ]
}

pub(crate) fn base_atlas_frame_name(planet: PlanetType) -> String {
    format!("building_radar_base_{}", planet_asset_name(planet))
}

pub(crate) fn team_overlay_frame_name(team: TeamType) -> String {
    format!("building_radar_{}", team.atlas_team().asset_name())
}

pub(crate) fn team_overlay_world_offset() -> Vec2 {
    Vec2::new(0.0, 32.0)
}

pub(crate) fn destroyed_asset_path(planet: PlanetType) -> String {
    format!(
        "buildings/radar/base_destroyed_{}.png",
        planet_asset_name(planet)
    )
}

pub(crate) fn overlay_frame_specs(kind: RadarOverlayKind) -> Vec<BuildingOverlayFrameSpec> {
    overlay_frame_names(kind)
        .into_iter()
        .zip(overlay_offsets(kind))
        .map(|(frame_name, world_offset)| BuildingOverlayFrameSpec {
            frame_name,
            world_offset,
        })
        .collect()
}

pub(crate) fn overlay_frame_names(kind: RadarOverlayKind) -> Vec<String> {
    let (stem, count) = match kind {
        RadarOverlayKind::FrontLight => ("front_light", 2),
        RadarOverlayKind::SideLight => ("side_light", 2),
        RadarOverlayKind::BoxSpinner => ("box_spinner", 12),
        RadarOverlayKind::Dish => ("dish", 8),
    };

    (0..count)
        .map(|frame| format!("building_radar_{stem}_{frame}"))
        .collect()
}

pub(crate) fn overlay_offsets(kind: RadarOverlayKind) -> Vec<Vec2> {
    match kind {
        RadarOverlayKind::FrontLight => vec![Vec2::new(16.0, 22.0); 2],
        RadarOverlayKind::SideLight => vec![Vec2::new(41.0, 0.0); 2],
        RadarOverlayKind::BoxSpinner => vec![Vec2::new(18.0, 13.0); 12],
        RadarOverlayKind::Dish => [-5.0, -6.0, -10.0, -13.0, -15.0, -13.0, -10.0, -6.0]
            .into_iter()
            .map(|y| Vec2::new(15.0, y))
            .collect(),
    }
}

pub(crate) fn overlay_should_be_visible(
    kind: RadarOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
) -> bool {
    !stats.destroyed() && (owner != TeamType::Null || matches!(kind, RadarOverlayKind::FrontLight))
}

pub(crate) fn death_profile() -> BuildingDeathProfile {
    BuildingDeathProfile {
        effect_box: BuildingEffectBox {
            x: 1.0,
            y: 6.0,
            width: 44,
            height: 30,
        },
        width_pix: 64.0,
        height_pix: 48.0,
        max_effects_base: 6,
        max_effects_random: 3,
        fireball_base: 4,
        fireball_random: 3,
        piece_base: 3,
        piece_random: 3,
        piece_variants: 2,
        piece_flight_base: 1.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{BuildingType, ObjectKind};

    #[test]
    fn death_profile_matches_original_do_death_effect() {
        assert_eq!(
            death_profile(),
            BuildingDeathProfile {
                effect_box: BuildingEffectBox {
                    x: 1.0,
                    y: 6.0,
                    width: 44,
                    height: 30
                },
                width_pix: 64.0,
                height_pix: 48.0,
                max_effects_base: 6,
                max_effects_random: 3,
                fireball_base: 4,
                fireball_random: 3,
                piece_base: 3,
                piece_random: 3,
                piece_variants: 2,
                piece_flight_base: 1.5
            }
        );
    }

    #[test]
    fn destroyed_asset_matches_original_path() {
        assert_eq!(
            destroyed_asset_path(PlanetType::Arctic),
            "buildings/radar/base_destroyed_arctic.png"
        );
    }

    #[test]
    fn atlas_layer_specs_match_original_base_and_team_overlay() {
        let specs = atlas_layer_specs(TeamType::Purple, PlanetType::City);

        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].atlas_team, TeamType::Red);
        assert_eq!(specs[0].frame_name, "building_radar_base_city");
        assert_eq!(specs[0].world_offset, Vec2::ZERO);
        assert_eq!(specs[1].atlas_team, TeamType::Red);
        assert_eq!(specs[1].frame_name, "building_radar_red");
        assert_eq!(specs[1].world_offset, Vec2::new(0.0, 32.0));
    }

    #[test]
    fn overlay_frame_specs_match_original_assets_and_offsets() {
        assert_eq!(OVERLAY_KINDS.len(), 4);
        assert_eq!(OVERLAY_FRAME_TIME, 0.25);
        assert_eq!(
            overlay_frame_names(RadarOverlayKind::FrontLight),
            vec![
                "building_radar_front_light_0".to_string(),
                "building_radar_front_light_1".to_string()
            ]
        );
        assert_eq!(
            overlay_offsets(RadarOverlayKind::Dish),
            vec![
                Vec2::new(15.0, -5.0),
                Vec2::new(15.0, -6.0),
                Vec2::new(15.0, -10.0),
                Vec2::new(15.0, -13.0),
                Vec2::new(15.0, -15.0),
                Vec2::new(15.0, -13.0),
                Vec2::new(15.0, -10.0),
                Vec2::new(15.0, -6.0),
            ]
        );

        let specs = overlay_frame_specs(RadarOverlayKind::BoxSpinner);
        assert_eq!(specs.len(), 12);
        assert_eq!(specs[0].frame_name, "building_radar_box_spinner_0");
        assert_eq!(specs[0].world_offset, Vec2::new(18.0, 13.0));
    }

    #[test]
    fn overlay_visibility_matches_original_owner_and_destroyed_rules() {
        let live = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Radar), 100);
        let destroyed = ObjectStats::from_kind(ObjectKind::Building(BuildingType::Radar), 0);

        assert!(overlay_should_be_visible(
            RadarOverlayKind::FrontLight,
            TeamType::Null,
            live
        ));
        assert!(!overlay_should_be_visible(
            RadarOverlayKind::Dish,
            TeamType::Null,
            live
        ));
        assert!(overlay_should_be_visible(
            RadarOverlayKind::BoxSpinner,
            TeamType::Red,
            live
        ));
        assert!(!overlay_should_be_visible(
            RadarOverlayKind::FrontLight,
            TeamType::Red,
            destroyed
        ));
    }
}
