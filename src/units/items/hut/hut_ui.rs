use bevy::prelude::Vec2;

use crate::original::types::PlanetType;

pub(crate) use super::hut_logic::initial_animal_spawner;

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HutVisualSpec {
    pub(crate) asset_path: String,
    pub(crate) atlas_frame_name: String,
    pub(crate) selection_size: Vec2,
}

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(16.0)
}

#[cfg(test)]
pub(crate) fn visual_spec(planet: PlanetType) -> HutVisualSpec {
    HutVisualSpec {
        asset_path: asset_path(planet),
        atlas_frame_name: atlas_frame_name(planet),
        selection_size: default_selection_size(),
    }
}

#[cfg(test)]
pub(crate) fn asset_path(planet: PlanetType) -> String {
    format!("other/map_items/hut_{}.png", planet_asset_name(planet))
}

pub(crate) fn atlas_frame_name(planet: PlanetType) -> String {
    format!("hut_{}", planet_asset_name(planet))
}

fn planet_asset_name(planet: PlanetType) -> &'static str {
    match planet {
        PlanetType::Desert => "desert",
        PlanetType::Volcanic => "volcanic",
        PlanetType::Arctic => "arctic",
        PlanetType::Jungle => "jungle",
        PlanetType::City => "city",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hut_assets_match_original_planet_specific_names() {
        assert_eq!(
            visual_spec(PlanetType::Desert),
            HutVisualSpec {
                asset_path: "other/map_items/hut_desert.png".to_string(),
                atlas_frame_name: "hut_desert".to_string(),
                selection_size: Vec2::splat(16.0),
            }
        );
        assert_eq!(
            asset_path(PlanetType::Desert),
            "other/map_items/hut_desert.png"
        );
        assert_eq!(
            asset_path(PlanetType::Volcanic),
            "other/map_items/hut_volcanic.png"
        );
        assert_eq!(
            asset_path(PlanetType::Arctic),
            "other/map_items/hut_arctic.png"
        );
        assert_eq!(
            asset_path(PlanetType::Jungle),
            "other/map_items/hut_jungle.png"
        );
        assert_eq!(asset_path(PlanetType::City), "other/map_items/hut_city.png");
    }
}
