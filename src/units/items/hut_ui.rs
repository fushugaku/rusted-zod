use bevy::prelude::Vec2;

use crate::original::types::PlanetType;

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(16.0)
}

pub(crate) fn asset_path(planet: PlanetType) -> String {
    format!("other/map_items/hut_{}.png", planet_asset_name(planet))
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
