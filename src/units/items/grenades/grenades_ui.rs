use bevy::prelude::Vec2;

use crate::components::DamageCrater;
#[cfg(test)]
use crate::components::DamageMissileVisual;

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GrenadeBoxVisualSpec {
    pub(crate) asset_path: &'static str,
    pub(crate) atlas_frame_name: &'static str,
    pub(crate) selection_size: Vec2,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GrenadeBoxDestroyEffectSpec {
    pub(crate) missile_visual: DamageMissileVisual,
    pub(crate) frame_paths: Vec<String>,
    pub(crate) crater: DamageCrater,
    pub(crate) visual_offset: Vec2,
    pub(crate) visual_rise: f32,
    pub(crate) angle_degrees_per_sec: f32,
}

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(16.0)
}

#[cfg(test)]
pub(crate) fn visual_spec() -> GrenadeBoxVisualSpec {
    GrenadeBoxVisualSpec {
        asset_path: asset_path(),
        atlas_frame_name: atlas_frame_name(),
        selection_size: default_selection_size(),
    }
}

#[cfg(test)]
pub(crate) fn asset_path() -> &'static str {
    "other/map_items/grenades.png"
}

pub(crate) fn atlas_frame_name() -> &'static str {
    "item_grenades"
}

pub(crate) fn projectile_frame_paths() -> Vec<String> {
    (0..4)
        .map(|frame| format!("other/grenades/grenade_n{frame:02}.png"))
        .collect()
}

#[cfg(test)]
pub(crate) fn destroy_effect_spec() -> GrenadeBoxDestroyEffectSpec {
    GrenadeBoxDestroyEffectSpec {
        missile_visual: DamageMissileVisual::Grenade,
        frame_paths: projectile_frame_paths(),
        crater: destroy_missile_crater(),
        visual_offset: Vec2::ZERO,
        visual_rise: 0.0,
        angle_degrees_per_sec: 0.0,
    }
}

pub(crate) fn destroy_missile_crater() -> DamageCrater {
    DamageCrater {
        is_big: false,
        chance: 0.35,
        big_chance: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grenade_box_assets_match_original() {
        assert_eq!(
            visual_spec(),
            GrenadeBoxVisualSpec {
                asset_path: "other/map_items/grenades.png",
                atlas_frame_name: "item_grenades",
                selection_size: Vec2::splat(16.0),
            }
        );
        assert_eq!(projectile_frame_paths().len(), 4);
        assert_eq!(
            projectile_frame_paths(),
            vec![
                "other/grenades/grenade_n00.png",
                "other/grenades/grenade_n01.png",
                "other/grenades/grenade_n02.png",
                "other/grenades/grenade_n03.png",
            ]
        );

        let effect = destroy_effect_spec();
        assert_eq!(effect.missile_visual, DamageMissileVisual::Grenade);
        assert_eq!(effect.frame_paths, projectile_frame_paths());
        assert_eq!(effect.crater, destroy_missile_crater());
        assert_eq!(effect.visual_offset, Vec2::ZERO);
        assert_eq!(effect.visual_rise, 0.0);
        assert_eq!(effect.angle_degrees_per_sec, 0.0);
    }
}
