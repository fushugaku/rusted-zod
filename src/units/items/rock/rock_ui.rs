#[cfg(test)]
use bevy::prelude::UVec2;
use bevy::prelude::Vec2;

use crate::{
    components::{CombatRng, MapGridPosition},
    constants::TILE_SIZE,
    original::types::PlanetType,
};

pub(crate) const ROCK_PARTICLE_FRAME_TIME: f32 = 0.07;

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RockRenderPieceSpec {
    pub(crate) tile_offset: UVec2,
    pub(crate) atlas_index: usize,
    pub(crate) z: f32,
    pub(crate) name: &'static str,
}

#[cfg(test)]
#[derive(Clone, Copy)]
struct RockRender {
    top: [Option<usize>; 3],
    shadow: [Option<usize>; 3],
}

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(16.0)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RockParticleKind {
    Small,
    Mid,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EffectTrajectory {
    pub(crate) start: Vec2,
    pub(crate) end: Vec2,
    pub(crate) final_time: f32,
    pub(crate) rise: f32,
}

#[cfg(test)]
pub(crate) fn atlas_path(planet: PlanetType) -> String {
    format!("planets/rocks_{}.png", planet_asset_name(planet))
}

#[cfg(test)]
pub(crate) fn render_piece_specs(
    rock_list: &[Vec<bool>],
    tx: usize,
    ty: usize,
    map_w: usize,
    map_h: usize,
) -> Vec<RockRenderPieceSpec> {
    let render = rock_render_for(rock_list, tx, ty, map_w, map_h);
    let mut pieces = Vec::new();

    for (j, maybe_index) in render.shadow.into_iter().enumerate() {
        if let Some(atlas_index) = maybe_index {
            pieces.push(RockRenderPieceSpec {
                tile_offset: UVec2::new(1, j as u32),
                atlas_index,
                z: 3.0,
                name: "rock_shadow",
            });
        }
    }

    for (j, maybe_index) in render.top.into_iter().enumerate() {
        if let Some(atlas_index) = maybe_index {
            pieces.push(RockRenderPieceSpec {
                tile_offset: UVec2::new(0, j as u32),
                atlas_index,
                z: 4.0 + j as f32 * 0.01,
                name: "rock",
            });
        }
    }

    pieces
}

pub(crate) fn destroyed_rubble_indices() -> [usize; 6] {
    [33, 34, 35, 17, 23, 29]
}

pub(crate) fn destroyed_rubble_index(rng: &mut CombatRng) -> usize {
    let indices = destroyed_rubble_indices();
    indices[rng.index(indices.len())]
}

pub(crate) fn destroyed_rubble_world_position(grid: MapGridPosition) -> Vec2 {
    Vec2::new(
        grid.x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        -(grid.y.saturating_add(2) as f32 * TILE_SIZE + TILE_SIZE * 0.5),
    )
}

pub(crate) fn particle_frame_paths(
    planet: PlanetType,
    kind: RockParticleKind,
    rng: &mut CombatRng,
) -> Vec<String> {
    let planet = planet_asset_name(planet);
    let (prefix, frames) = match kind {
        RockParticleKind::Small => ("debri_small".to_string(), 16),
        RockParticleKind::Mid => (format!("debri_mid{}", rng.index(2)), 8),
    };
    (0..frames)
        .map(|frame| format!("planets/rock_effects/{prefix}_{planet}_n{frame:02}.png"))
        .collect()
}

pub(crate) fn turrent_frame_paths(planet: PlanetType, rng: &mut CombatRng) -> Vec<String> {
    let planet_name = planet_asset_name(planet);
    let variant = match planet {
        PlanetType::City | PlanetType::Desert => 0,
        _ => rng.index(2),
    };
    (0..12)
        .map(|frame| {
            format!("planets/rock_effects/debri_large{variant}_{planet_name}_n{frame:02}.png")
        })
        .collect()
}

pub(crate) fn small_particle_count(rng: &mut CombatRng) -> usize {
    12 + rng.index(6)
}

pub(crate) fn mid_particle_count(rng: &mut CombatRng) -> usize {
    4 + rng.index(3)
}

pub(crate) fn large_turrent_count(rng: &mut CombatRng) -> usize {
    rng.index(2)
}

pub(crate) fn turrent_end_small_particle_count(rng: &mut CombatRng) -> usize {
    12 + rng.index(6)
}

pub(crate) fn particle_trajectory(
    anchor_map: Vec2,
    max_horz: f32,
    max_vert: f32,
    rng: &mut CombatRng,
) -> EffectTrajectory {
    original_particle_trajectory(anchor_map, max_horz, max_vert, 1.1, 10, 1.1, 30, rng)
}

pub(crate) fn turrent_trajectory(
    anchor_map: Vec2,
    max_horz: f32,
    max_vert: f32,
    rng: &mut CombatRng,
) -> EffectTrajectory {
    original_particle_trajectory(anchor_map, max_horz, max_vert, 1.5, 10, 1.1, 200, rng)
}

pub(crate) fn turrent_spin_degrees_per_sec(rng: &mut CombatRng) -> f32 {
    240.0 - rng.index(480) as f32
}

pub(crate) fn arc_size(rise: f32, final_time: f32, t: f32) -> f32 {
    if final_time <= 0.0 {
        1.0
    } else {
        -(rise / final_time) * (t * t) + rise * t + 1.0
    }
}

#[cfg(test)]
fn rock_render_for(
    rock_list: &[Vec<bool>],
    tx: usize,
    ty: usize,
    map_w: usize,
    map_h: usize,
) -> RockRender {
    let l = tx == 0 || rock_list[tx - 1][ty];
    let up = ty == 0 || rock_list[tx][ty - 1];
    let r = tx == map_w - 1 || rock_list[tx + 1][ty];
    let dn = ty == map_h - 1 || rock_list[tx][ty + 1];

    let dl = tx > 0 && ty < map_h - 1 && rock_list[tx - 1][ty + 1];
    let ddn = ty >= map_h - 2 || rock_list[tx][ty + 2];
    let uup = ty < 2 || rock_list[tx][ty - 2];
    let uur = ty >= 2 && tx < map_w - 1 && rock_list[tx + 1][ty - 2];
    let ur = ty >= 1 && tx < map_w - 1 && rock_list[tx + 1][ty - 1];
    let dr = ty < map_h - 1 && tx < map_w - 1 && rock_list[tx + 1][ty + 1];
    let ddr = ty < map_h - 2 && tx < map_w - 1 && rock_list[tx + 1][ty + 2];

    let mut top = [None, None, Some(rock_index(3, 4))];
    let mut shadow = [None, None, None];

    top[0] = Some(if r && l && up && dn {
        rock_index(1, 1)
    } else if r && !l && !up && dn {
        rock_index(0, 0)
    } else if !r && l && !up && dn {
        rock_index(2, 0)
    } else if !r && l && up && !dn {
        rock_index(2, 2)
    } else if r && !l && up && !dn {
        rock_index(0, 2)
    } else if r && l && !up && dn {
        rock_index(1, 0)
    } else if r && l && up && !dn {
        rock_index(1, 2)
    } else if !r && l && up && dn {
        rock_index(2, 1)
    } else if r && !l && up && dn {
        rock_index(0, 1)
    } else if !r && !l && !up && dn {
        rock_index(3, 0)
    } else if !r && !l && up && dn {
        rock_index(3, 1)
    } else if !r && !l && up && !dn {
        rock_index(3, 2)
    } else if r && !l && !up && !dn {
        rock_index(0, 5)
    } else if r && l && !up && !dn {
        rock_index(1, 5)
    } else if !r && l && !up && !dn {
        rock_index(2, 5)
    } else {
        rock_index(3, 2)
    });

    if ty + 1 < map_h {
        top[1] = if dn {
            None
        } else if dl {
            Some(if r {
                rock_index(5, 0)
            } else {
                rock_index(4, 0)
            })
        } else {
            Some(if r && l {
                rock_index(1, 3)
            } else if r && !l {
                rock_index(0, 3)
            } else if !r && l {
                rock_index(2, 3)
            } else {
                rock_index(3, 3)
            })
        };
    }

    if ty + 2 < map_h {
        top[2] = if ddn || dn {
            None
        } else if dl {
            Some(if r {
                rock_index(5, 1)
            } else {
                rock_index(4, 1)
            })
        } else {
            Some(if r && l {
                rock_index(1, 4)
            } else if r && !l {
                rock_index(0, 4)
            } else if !r && l {
                rock_index(2, 4)
            } else {
                rock_index(3, 4)
            })
        };
    }

    if tx < map_w - 1 {
        if !(uur || ur || r) {
            if up && !uup {
                shadow[0] = Some(rock_index(4, 2));
            } else if up || uup {
                shadow[0] = Some(rock_index(4, 3));
            }
        }

        if !dn && !(ur || r || dr) {
            if !up {
                shadow[1] = Some(rock_index(4, 2));
            } else {
                shadow[1] = Some(rock_index(4, 3));
            }
        }

        if !dn && !ddn && !(r || dr || ddr) {
            shadow[2] = Some(rock_index(4, 4));
        }
    }

    RockRender { top, shadow }
}

#[cfg(test)]
fn rock_index(x: usize, y: usize) -> usize {
    y * 6 + x
}

fn original_particle_trajectory(
    anchor_map: Vec2,
    max_horz: f32,
    max_vert: f32,
    lifetime_base: f32,
    lifetime_random: usize,
    rise_base: f32,
    rise_random: usize,
    rng: &mut CombatRng,
) -> EffectTrajectory {
    let particle = anchor_map + Vec2::new(rng.index(8) as f32, rng.index(24) as f32);
    let start = particle - Vec2::new(8.0, 5.0);
    let end = particle
        + Vec2::new(
            max_horz - rng.index((max_horz * 2.0) as usize) as f32,
            max_vert - rng.index((max_vert * 2.0) as usize) as f32,
        );

    EffectTrajectory {
        start,
        end,
        final_time: lifetime_base + rng.index(lifetime_random) as f32 * 0.1,
        rise: rise_base + rng.index(rise_random) as f32 * 0.01,
    }
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
    fn rock_assets_match_original_paths_and_atlas_indices() {
        assert_eq!(atlas_path(PlanetType::Desert), "planets/rocks_desert.png");
        assert_eq!(destroyed_rubble_indices(), [33, 34, 35, 17, 23, 29]);
        assert_eq!(
            destroyed_rubble_world_position(MapGridPosition { x: 3, y: 4 }),
            Vec2::new(56.0, -104.0)
        );

        let mut rng = CombatRng::default();
        let small = particle_frame_paths(PlanetType::Desert, RockParticleKind::Small, &mut rng);
        assert_eq!(small.len(), 16);
        assert_eq!(
            small.last().unwrap(),
            "planets/rock_effects/debri_small_desert_n15.png"
        );

        let mid = particle_frame_paths(PlanetType::Arctic, RockParticleKind::Mid, &mut rng);
        assert_eq!(mid.len(), 8);
        assert!(mid[0].starts_with("planets/rock_effects/debri_mid"));
        assert!(mid[0].ends_with("_arctic_n00.png"));

        let large_desert = turrent_frame_paths(PlanetType::Desert, &mut rng);
        assert_eq!(
            large_desert[0],
            "planets/rock_effects/debri_large0_desert_n00.png"
        );
        assert_eq!(large_desert.len(), 12);
        assert_eq!(ROCK_PARTICLE_FRAME_TIME, 0.07);
    }

    #[test]
    fn rock_render_piece_specs_match_original_top_and_shadow_indices() {
        let mut isolated = vec![vec![false; 5]; 5];
        isolated[2][2] = true;
        assert_eq!(
            render_piece_specs(&isolated, 2, 2, 5, 5),
            vec![
                RockRenderPieceSpec {
                    tile_offset: UVec2::new(1, 1),
                    atlas_index: 16,
                    z: 3.0,
                    name: "rock_shadow",
                },
                RockRenderPieceSpec {
                    tile_offset: UVec2::new(1, 2),
                    atlas_index: 28,
                    z: 3.0,
                    name: "rock_shadow",
                },
                RockRenderPieceSpec {
                    tile_offset: UVec2::new(0, 0),
                    atlas_index: 15,
                    z: 4.0,
                    name: "rock",
                },
                RockRenderPieceSpec {
                    tile_offset: UVec2::new(0, 1),
                    atlas_index: 21,
                    z: 4.01,
                    name: "rock",
                },
                RockRenderPieceSpec {
                    tile_offset: UVec2::new(0, 2),
                    atlas_index: 27,
                    z: 4.02,
                    name: "rock",
                },
            ]
        );

        let surrounded = vec![vec![true; 5]; 5];
        assert_eq!(
            render_piece_specs(&surrounded, 2, 2, 5, 5),
            vec![RockRenderPieceSpec {
                tile_offset: UVec2::ZERO,
                atlas_index: 7,
                z: 4.0,
                name: "rock",
            }]
        );
    }

    #[test]
    fn rock_death_counts_and_motion_match_original_ranges() {
        let mut rng = CombatRng::default();
        for _ in 0..32 {
            assert!((12..=17).contains(&small_particle_count(&mut rng)));
            assert!((4..=6).contains(&mid_particle_count(&mut rng)));
            assert!((0..=1).contains(&large_turrent_count(&mut rng)));
            assert!((12..=17).contains(&turrent_end_small_particle_count(&mut rng)));
            assert!((-239.0..=240.0).contains(&turrent_spin_degrees_per_sec(&mut rng)));
            assert!(destroyed_rubble_indices().contains(&destroyed_rubble_index(&mut rng)));
        }

        let small = particle_trajectory(Vec2::ZERO, 80.0, 60.0, &mut rng);
        assert!((1.1..=2.0).contains(&small.final_time));
        assert!((1.1..=1.39).contains(&small.rise));

        let large = turrent_trajectory(Vec2::ZERO, 140.0, 140.0, &mut rng);
        assert!((1.5..=2.4).contains(&large.final_time));
        assert!((1.1..=3.09).contains(&large.rise));
        assert_eq!(arc_size(1.1, 1.1, 1.1), 1.0);
        assert!(arc_size(1.1, 1.1, 0.5) > 1.0);
    }
}
