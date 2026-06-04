use bevy::prelude::*;

use crate::components::{
    CombatRng, CurrentMap, CurrentTileInfo, PlanetAtlas, TerrainEffectPools, TerrainEffectTile,
};
use crate::constants::TILE_SIZE;
use crate::original::map::ZMap;
use crate::original::tileinfo::PaletteTileInfo;
use crate::original::types::PlanetType;

pub(crate) fn spawn_terrain(
    commands: &mut Commands,
    map: &ZMap,
    tile_info: &[PaletteTileInfo],
    atlas: &PlanetAtlas,
    rng: &mut CombatRng,
) {
    let mut road_tiles = 0;
    let mut water_tiles = 0;
    let mut blocked_tiles = 0;
    let pools = terrain_effect_pools(tile_info);

    let image = match map.basics.terrain_type {
        PlanetType::Desert => atlas.desert.clone(),
        PlanetType::Volcanic => atlas.volcanic.clone(),
        PlanetType::Arctic => atlas.arctic.clone(),
        PlanetType::Jungle => atlas.jungle.clone(),
        PlanetType::City => atlas.city.clone(),
    };

    for (i, tile) in map.tiles.iter().enumerate() {
        let (current_tile, effect_state) = initial_terrain_effect_state(
            map.basics.terrain_type,
            tile.tile,
            tile_info,
            &pools,
            rng,
        );
        let x = (i % map.basics.width as usize) as f32 * TILE_SIZE + TILE_SIZE * 0.5;
        let y = -((i / map.basics.width as usize) as f32 * TILE_SIZE + TILE_SIZE * 0.5);
        if let Some(info) = tile_info.get(current_tile as usize) {
            if info.is_road {
                road_tiles += 1;
            }
            if info.is_water {
                water_tiles += 1;
            }
            if info.walk_speed() == 0.0 {
                blocked_tiles += 1;
            }
        }

        let mut entity = commands.spawn((
            Sprite {
                image: image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas.layout.clone(),
                    index: current_tile.min(479) as usize,
                }),
                ..default()
            },
            Transform::from_xyz(x, y, 0.0),
        ));
        if let Some(effect_state) = effect_state {
            entity.insert(effect_state);
        }
    }
    commands.insert_resource(pools);

    println!(
        "Terrain metadata in use: {road_tiles} road tiles, {water_tiles} water tiles, {blocked_tiles} blocked tiles"
    );
}

pub(crate) fn animate_terrain_effects(
    time: Res<Time>,
    map: Res<CurrentMap>,
    tile_info: Res<CurrentTileInfo>,
    pools: Res<TerrainEffectPools>,
    mut rng: ResMut<CombatRng>,
    mut query: Query<(&mut Sprite, &mut TerrainEffectTile)>,
) {
    let now = time.elapsed_secs();
    for (mut sprite, mut tile) in &mut query {
        let Some(next_tile) = advance_terrain_effect_tile(
            &tile_info.0,
            &pools,
            map.0.basics.terrain_type,
            now,
            &mut rng,
            &mut tile,
        ) else {
            continue;
        };
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = next_tile.min(479) as usize;
        }
    }
}

pub(crate) fn terrain_effect_min_interval(planet: PlanetType) -> f32 {
    match planet {
        PlanetType::Volcanic => 0.5,
        PlanetType::Arctic => 0.4,
        PlanetType::Desert | PlanetType::Jungle | PlanetType::City => 0.2,
    }
}

fn terrain_effect_next_time(planet: PlanetType, now: f32, rng: &mut CombatRng) -> f32 {
    now + terrain_effect_min_interval(planet) + rng.index(4) as f32 * 0.033
}

pub(crate) fn terrain_effect_pools(tile_info: &[PaletteTileInfo]) -> TerrainEffectPools {
    TerrainEffectPools {
        water_tiles: tile_info
            .iter()
            .enumerate()
            .filter_map(|(index, info)| {
                (info.is_usable && info.is_water && !info.is_effect).then_some(index as u16)
            })
            .collect(),
        water_effect_tiles: tile_info
            .iter()
            .enumerate()
            .filter_map(|(index, info)| {
                (info.is_usable && info.is_water_effect).then_some(index as u16)
            })
            .collect(),
    }
}

fn initial_terrain_effect_state(
    planet: PlanetType,
    tile: u16,
    tile_info: &[PaletteTileInfo],
    pools: &TerrainEffectPools,
    rng: &mut CombatRng,
) -> (u16, Option<TerrainEffectTile>) {
    let Some(info) = tile_info
        .get(tile as usize)
        .copied()
        .filter(|info| info.is_usable)
    else {
        return (tile, None);
    };

    if planet == PlanetType::Desert
        && info.is_water
        && info.is_effect
        && !pools.water_tiles.is_empty()
    {
        let current_tile = pools.water_tiles[rng.index(pools.water_tiles.len())];
        return (
            current_tile,
            Some(TerrainEffectTile {
                current_tile,
                next_effect_time: 0.0,
                effect_active: false,
                water_listed: true,
            }),
        );
    }

    if info.is_effect {
        return (
            tile,
            Some(TerrainEffectTile {
                current_tile: tile,
                next_effect_time: 0.0,
                effect_active: true,
                water_listed: false,
            }),
        );
    }

    if info.is_water {
        return (
            tile,
            Some(TerrainEffectTile {
                current_tile: tile,
                next_effect_time: 0.0,
                effect_active: false,
                water_listed: true,
            }),
        );
    }

    (tile, None)
}

fn advance_terrain_effect_tile(
    tile_info: &[PaletteTileInfo],
    pools: &TerrainEffectPools,
    planet: PlanetType,
    now: f32,
    rng: &mut CombatRng,
    tile: &mut TerrainEffectTile,
) -> Option<u16> {
    if now < tile.next_effect_time {
        return Some(tile.current_tile);
    }

    tile.next_effect_time = terrain_effect_next_time(planet, now, rng);

    if tile.effect_active {
        let info = tile_info.get(tile.current_tile as usize)?;
        tile.current_tile = info.next_tile_in_effect;
        if tile_info
            .get(tile.current_tile as usize)
            .is_some_and(|info| info.is_water_effect)
        {
            if let Some(water_tile) = random_pool_tile(&pools.water_tiles, rng) {
                tile.current_tile = water_tile;
            }
            tile.effect_active = false;
            tile.water_listed = true;
        }
        return Some(tile.current_tile);
    }

    if tile.water_listed
        && !pools.water_effect_tiles.is_empty()
        && tile_info
            .get(tile.current_tile as usize)
            .is_some_and(|info| !info.is_effect)
        && rng.index(40) == 0
    {
        tile.current_tile = random_pool_tile(&pools.water_effect_tiles, rng)?;
        tile.effect_active = true;
        return Some(tile.current_tile);
    }

    None
}

fn random_pool_tile(pool: &[u16], rng: &mut CombatRng) -> Option<u16> {
    (!pool.is_empty()).then(|| pool[rng.index(pool.len())])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(
        is_water: bool,
        is_effect: bool,
        is_water_effect: bool,
        next_tile_in_effect: u16,
    ) -> PaletteTileInfo {
        PaletteTileInfo {
            is_water,
            is_passable: true,
            is_usable: true,
            is_road: false,
            is_effect,
            is_water_effect,
            next_tile_in_effect,
            takes_tank_tracks: false,
            crater_type: 0,
            is_starter_tile: false,
        }
    }

    #[test]
    fn terrain_effect_intervals_match_original_planets() {
        assert_eq!(terrain_effect_min_interval(PlanetType::Desert), 0.2);
        assert_eq!(terrain_effect_min_interval(PlanetType::Jungle), 0.2);
        assert_eq!(terrain_effect_min_interval(PlanetType::City), 0.2);
        assert_eq!(terrain_effect_min_interval(PlanetType::Arctic), 0.4);
        assert_eq!(terrain_effect_min_interval(PlanetType::Volcanic), 0.5);
    }

    #[test]
    fn terrain_effect_pools_match_original_classification() {
        let tile_info = vec![
            info(true, false, false, 0),
            info(true, true, true, 2),
            info(false, true, false, 2),
            info(false, false, false, 0),
        ];
        let pools = terrain_effect_pools(&tile_info);

        assert_eq!(pools.water_tiles, vec![0]);
        assert_eq!(pools.water_effect_tiles, vec![1]);
    }

    #[test]
    fn terrain_effect_advances_immediately_through_next_tile() {
        let tile_info = vec![
            info(false, false, false, 0),
            info(false, true, false, 2),
            info(false, true, false, 3),
            info(false, false, false, 0),
        ];
        let pools = terrain_effect_pools(&tile_info);
        let mut tile = TerrainEffectTile {
            current_tile: 1,
            next_effect_time: 0.0,
            effect_active: true,
            water_listed: false,
        };
        let mut rng = CombatRng::default();

        assert_eq!(
            advance_terrain_effect_tile(
                &tile_info,
                &pools,
                PlanetType::Desert,
                0.01,
                &mut rng,
                &mut tile,
            ),
            Some(2)
        );
        assert_eq!(tile.current_tile, 2);
        assert!(tile.effect_active);
        assert!(tile.next_effect_time >= 0.2);
    }

    #[test]
    fn terrain_water_effect_returns_to_random_water_and_deactivates() {
        let tile_info = vec![
            info(true, false, false, 0),
            info(true, true, false, 2),
            info(true, true, true, 0),
        ];
        let pools = terrain_effect_pools(&tile_info);
        let mut tile = TerrainEffectTile {
            current_tile: 1,
            next_effect_time: 0.0,
            effect_active: true,
            water_listed: false,
        };
        let mut rng = CombatRng::default();

        assert_eq!(
            advance_terrain_effect_tile(
                &tile_info,
                &pools,
                PlanetType::Desert,
                0.01,
                &mut rng,
                &mut tile,
            ),
            Some(0)
        );
        assert_eq!(tile.current_tile, 0);
        assert!(!tile.effect_active);
        assert!(tile.water_listed);
    }

    #[test]
    fn terrain_water_effect_start_uses_one_in_forty_gate() {
        let tile_info = vec![
            info(true, false, false, 0),
            info(true, true, true, 2),
            info(true, true, false, 1),
        ];
        let pools = terrain_effect_pools(&tile_info);

        for seed in 0..10_000 {
            let mut rng = CombatRng(seed);
            let mut tile = TerrainEffectTile {
                current_tile: 0,
                next_effect_time: 0.0,
                effect_active: false,
                water_listed: true,
            };
            if advance_terrain_effect_tile(
                &tile_info,
                &pools,
                PlanetType::Desert,
                0.01,
                &mut rng,
                &mut tile,
            )
            .is_some()
            {
                assert_eq!(tile.current_tile, 1);
                assert!(tile.effect_active);
                return;
            }
        }

        panic!("expected at least one deterministic seed to pass the 1/40 water effect gate");
    }
}

pub(crate) fn spawn_zone_overlays(commands: &mut Commands, map: &ZMap) {
    for zone in &map.zones {
        let x = zone.x as f32 * TILE_SIZE + zone.w as f32 * TILE_SIZE * 0.5;
        let y = -(zone.y as f32 * TILE_SIZE + zone.h as f32 * TILE_SIZE * 0.5);
        let size = Vec2::new(
            (zone.w.max(1)) as f32 * TILE_SIZE,
            (zone.h.max(1)) as f32 * TILE_SIZE,
        );

        commands.spawn((
            Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.05), size),
            Transform::from_xyz(x, y, 1.0),
        ));
    }
}
