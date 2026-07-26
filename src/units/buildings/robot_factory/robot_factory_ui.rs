use std::collections::HashMap;

use bevy::prelude::Vec2;

use crate::{
    components::{CombatRng, ObjectStats},
    original::types::{PlanetType, TeamType},
    render::atlas::FactoryOverlayKind,
};

use crate::units::buildings::{
    BuildingAtlasFrameSpec, BuildingDeathProfile, BuildingEffectBox, BuildingOverlayFrameSpec,
    BuildingPathingRect, BuildingPathingSpec, ProductionPlacement, building_ui::planet_asset_name,
};

pub(crate) const OVERLAY_FRAME_TIME: f32 = 0.25;
pub(crate) const OVERLAY_KINDS: [FactoryOverlayKind; 8] = [
    FactoryOverlayKind::RobotExhaust,
    FactoryOverlayKind::RobotGreenBox,
    FactoryOverlayKind::RobotSingleLight0,
    FactoryOverlayKind::RobotSingleLight1,
    FactoryOverlayKind::RobotSingleLight2,
    FactoryOverlayKind::RobotDoubleLight,
    FactoryOverlayKind::RobotBody,
    FactoryOverlayKind::RobotSpin,
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
            height: 5,
        }],
        blocked_masks: Vec::new(),
        unblocked_tiles: Vec::new(),
    }
}

pub(crate) fn crane_repair_local_points(size: Vec2) -> (Vec2, Vec2) {
    (Vec2::new(35.0, size.y + 32.0), Vec2::new(35.0, 32.0))
}

pub(crate) fn production_placement() -> ProductionPlacement {
    ProductionPlacement {
        create_offset: Vec2::new(43.0, 53.0),
        move_offset: Vec2::new(43.0, 96.0),
    }
}

pub(crate) fn production_label_asset_path() -> &'static str {
    "other/production_gui/robot_factory_label.png"
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
    format!("building_robot_base_{}", planet_asset_name(planet))
}

pub(crate) fn team_overlay_frame_name(team: TeamType) -> String {
    format!("building_robot_{}", team.atlas_team().asset_name())
}

pub(crate) fn team_overlay_world_offset() -> Vec2 {
    Vec2::new(16.0, 64.0)
}

pub(crate) fn destroyed_asset_path(planet: PlanetType) -> String {
    format!(
        "buildings/robot/base_destroyed_{}.png",
        planet_asset_name(planet)
    )
}

pub(crate) fn overlay_kind_is_robot(kind: FactoryOverlayKind) -> bool {
    matches!(
        kind,
        FactoryOverlayKind::RobotExhaust
            | FactoryOverlayKind::RobotGreenBox
            | FactoryOverlayKind::RobotSingleLight0
            | FactoryOverlayKind::RobotSingleLight1
            | FactoryOverlayKind::RobotSingleLight2
            | FactoryOverlayKind::RobotDoubleLight
            | FactoryOverlayKind::RobotBody
            | FactoryOverlayKind::RobotSpin
    )
}

pub(crate) fn overlay_frame_specs(kind: FactoryOverlayKind) -> Vec<BuildingOverlayFrameSpec> {
    overlay_frame_names(kind)
        .into_iter()
        .zip(overlay_offsets(kind))
        .map(|(frame_name, world_offset)| BuildingOverlayFrameSpec {
            frame_name,
            world_offset,
        })
        .collect()
}

pub(crate) fn overlay_frame_names(kind: FactoryOverlayKind) -> Vec<String> {
    match kind {
        FactoryOverlayKind::RobotExhaust => {
            (0..13).map(|frame| format!("exhaust_{frame}")).collect()
        }
        FactoryOverlayKind::RobotGreenBox => (0..6)
            .map(|frame| format!("building_robot_green_box_{frame}"))
            .collect(),
        FactoryOverlayKind::RobotSingleLight0
        | FactoryOverlayKind::RobotSingleLight1
        | FactoryOverlayKind::RobotSingleLight2 => (0..2)
            .map(|frame| format!("building_robot_light_{frame}"))
            .collect(),
        FactoryOverlayKind::RobotDoubleLight => vec!["building_robot_double_light_1".to_string()],
        FactoryOverlayKind::RobotBody => (0..2)
            .map(|frame| format!("building_robot_robot_{frame}"))
            .collect(),
        FactoryOverlayKind::RobotSpin => (0..8)
            .map(|frame| format!("building_robot_spin_{frame}"))
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn overlay_offsets(kind: FactoryOverlayKind) -> Vec<Vec2> {
    match kind {
        FactoryOverlayKind::RobotExhaust => (0..13)
            .map(|frame| Vec2::new(28.0, -24.0 - frame as f32 * 2.0))
            .collect(),
        FactoryOverlayKind::RobotGreenBox => vec![Vec2::new(38.0, 39.0); 6],
        FactoryOverlayKind::RobotSingleLight0 => vec![Vec2::new(13.0, 68.0); 2],
        FactoryOverlayKind::RobotSingleLight1 => vec![Vec2::new(16.0, 68.0); 2],
        FactoryOverlayKind::RobotSingleLight2 => vec![Vec2::new(19.0, 68.0); 2],
        FactoryOverlayKind::RobotDoubleLight => vec![Vec2::new(16.0, 32.0)],
        FactoryOverlayKind::RobotBody => vec![Vec2::new(16.0, 48.0); 2],
        FactoryOverlayKind::RobotSpin => vec![Vec2::new(9.0, -2.0); 8],
        _ => Vec::new(),
    }
}

pub(crate) fn overlay_should_be_visible(
    kind: FactoryOverlayKind,
    owner: TeamType,
    stats: ObjectStats,
    production_active: bool,
    current: usize,
) -> bool {
    if stats.destroyed() || !overlay_kind_is_robot(kind) {
        return false;
    }

    if owner == TeamType::Null || !production_active {
        return matches!(kind, FactoryOverlayKind::RobotBody);
    }

    if overlay_is_single_light(kind) && current == 0 {
        return false;
    }

    true
}

pub(crate) fn overlay_next_frame(
    ref_id: u32,
    kind: FactoryOverlayKind,
    current: usize,
    frame_count: usize,
    rng: &mut CombatRng,
    robot_light_updates: &mut HashMap<u32, Option<[usize; 3]>>,
) -> usize {
    if let Some(light_index) = overlay_single_light_index(kind) {
        let update = *robot_light_updates
            .entry(ref_id)
            .or_insert_with(|| robot_single_light_update(rng));
        return update.map(|states| states[light_index]).unwrap_or(current);
    }

    (current + 1) % frame_count
}

pub(crate) fn robot_single_light_update(rng: &mut CombatRng) -> Option<[usize; 3]> {
    (rng.index(3) == 0).then(|| [rng.index(2), rng.index(2), rng.index(2)])
}

pub(crate) fn overlay_is_single_light(kind: FactoryOverlayKind) -> bool {
    overlay_single_light_index(kind).is_some()
}

pub(crate) fn overlay_initial_frame_visible(kind: FactoryOverlayKind) -> bool {
    !overlay_is_single_light(kind)
}

pub(crate) fn overlay_single_light_index(kind: FactoryOverlayKind) -> Option<usize> {
    match kind {
        FactoryOverlayKind::RobotSingleLight0 => Some(0),
        FactoryOverlayKind::RobotSingleLight1 => Some(1),
        FactoryOverlayKind::RobotSingleLight2 => Some(2),
        _ => None,
    }
}

pub(crate) fn death_profile() -> BuildingDeathProfile {
    BuildingDeathProfile {
        effect_box: BuildingEffectBox {
            x: 8.0,
            y: 8.0,
            width: 40,
            height: 56,
        },
        width_pix: 64.0,
        height_pix: 80.0,
        max_effects_base: 8,
        max_effects_random: 4,
        fireball_base: 8,
        fireball_random: 3,
        piece_base: 6,
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
                    x: 8.0,
                    y: 8.0,
                    width: 40,
                    height: 56
                },
                width_pix: 64.0,
                height_pix: 80.0,
                max_effects_base: 8,
                max_effects_random: 4,
                fireball_base: 8,
                fireball_random: 3,
                piece_base: 6,
                piece_random: 3,
                piece_variants: 2,
                piece_flight_base: 1.5
            }
        );
    }

    #[test]
    fn destroyed_asset_matches_original_path() {
        assert_eq!(
            destroyed_asset_path(PlanetType::Jungle),
            "buildings/robot/base_destroyed_jungle.png"
        );
    }

    #[test]
    fn atlas_layer_and_production_label_specs_match_original_assets() {
        assert_eq!(
            production_label_asset_path(),
            "other/production_gui/robot_factory_label.png"
        );

        let specs = atlas_layer_specs(TeamType::Blue, PlanetType::Desert);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].atlas_team, TeamType::Red);
        assert_eq!(specs[0].frame_name, "building_robot_base_desert");
        assert_eq!(specs[1].atlas_team, TeamType::Blue);
        assert_eq!(specs[1].frame_name, "building_robot_blue");
        assert_eq!(specs[1].world_offset, Vec2::new(16.0, 64.0));
    }

    #[test]
    fn overlay_frame_specs_match_original_assets_and_offsets() {
        assert_eq!(OVERLAY_KINDS.len(), 8);
        assert_eq!(OVERLAY_FRAME_TIME, 0.25);
        assert_eq!(
            overlay_frame_names(FactoryOverlayKind::RobotGreenBox)
                .last()
                .map(String::as_str),
            Some("building_robot_green_box_5")
        );
        assert_eq!(
            overlay_frame_names(FactoryOverlayKind::RobotSingleLight0),
            vec![
                "building_robot_light_0".to_string(),
                "building_robot_light_1".to_string()
            ]
        );
        assert_eq!(
            overlay_offsets(FactoryOverlayKind::RobotExhaust)
                .last()
                .copied(),
            Some(Vec2::new(28.0, -48.0))
        );
        assert!(!overlay_initial_frame_visible(
            FactoryOverlayKind::RobotSingleLight0
        ));
        assert!(overlay_initial_frame_visible(FactoryOverlayKind::RobotBody));

        let specs = overlay_frame_specs(FactoryOverlayKind::RobotSpin);
        assert_eq!(specs.len(), 8);
        assert_eq!(specs[0].frame_name, "building_robot_spin_0");
        assert_eq!(specs[0].world_offset, Vec2::new(9.0, -2.0));
    }

    #[test]
    fn overlay_visibility_matches_original_owner_building_and_destroyed_rules() {
        let live = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 100);
        let destroyed = ObjectStats::from_kind(ObjectKind::Building(BuildingType::RobotFactory), 0);

        assert!(overlay_should_be_visible(
            FactoryOverlayKind::RobotBody,
            TeamType::Null,
            live,
            false,
            0
        ));
        assert!(!overlay_should_be_visible(
            FactoryOverlayKind::RobotSpin,
            TeamType::Null,
            live,
            false,
            0
        ));
        assert!(overlay_should_be_visible(
            FactoryOverlayKind::RobotSpin,
            TeamType::Red,
            live,
            true,
            0
        ));
        assert!(!overlay_should_be_visible(
            FactoryOverlayKind::RobotBody,
            TeamType::Red,
            destroyed,
            true,
            0
        ));
        assert!(!overlay_should_be_visible(
            FactoryOverlayKind::RobotSingleLight0,
            TeamType::Red,
            live,
            true,
            0
        ));
        assert!(overlay_should_be_visible(
            FactoryOverlayKind::RobotSingleLight0,
            TeamType::Red,
            live,
            true,
            1
        ));
    }

    #[test]
    fn robot_single_lights_use_original_random_on_skip_behavior() {
        let mut no_reroll = CombatRng::default();
        let mut no_updates = HashMap::new();
        assert_eq!(
            overlay_next_frame(
                10,
                FactoryOverlayKind::RobotSingleLight0,
                0,
                2,
                &mut no_reroll,
                &mut no_updates,
            ),
            0
        );
        assert_eq!(
            overlay_next_frame(
                10,
                FactoryOverlayKind::RobotSingleLight1,
                1,
                2,
                &mut no_reroll,
                &mut no_updates,
            ),
            1
        );

        let mut reroll_on = CombatRng(3);
        let mut light_updates = HashMap::new();
        assert_eq!(
            overlay_next_frame(
                20,
                FactoryOverlayKind::RobotSingleLight0,
                0,
                2,
                &mut reroll_on,
                &mut light_updates,
            ),
            1
        );
        assert_eq!(
            overlay_next_frame(
                20,
                FactoryOverlayKind::RobotSingleLight1,
                0,
                2,
                &mut reroll_on,
                &mut light_updates,
            ),
            1
        );
        assert_eq!(
            overlay_next_frame(
                20,
                FactoryOverlayKind::RobotSingleLight2,
                0,
                2,
                &mut reroll_on,
                &mut light_updates,
            ),
            1
        );

        let mut non_light = CombatRng::default();
        let mut non_light_updates = HashMap::new();
        assert_eq!(
            overlay_next_frame(
                30,
                FactoryOverlayKind::RobotBody,
                1,
                2,
                &mut non_light,
                &mut non_light_updates,
            ),
            0
        );
    }
}
