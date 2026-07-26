use crate::{
    components::{CombatRng, MapGridPosition, PassabilityGrid},
    constants::TILE_SIZE,
    direction_index_from_delta, map_point_to_world,
    original::{map::ZMap, types::PlanetType},
};
use bevy::prelude::{Component, IVec2, Vec2};

pub(crate) const HUT_ANIMAL_MIN: usize = 3;
pub(crate) const HUT_ANIMAL_MAX: usize = 5;
const HUT_ANIMAL_ROAM_DISTANCE: i32 = 7 * TILE_SIZE as i32;
const HUT_ANIMAL_MOVE_SPEED: f32 = 15.0;
pub(super) const HUT_ANIMAL_POPULATION_INTERVAL: f32 = 1.0;
pub(super) const HUT_ANIMAL_MAX_INTERVAL: f32 = 10.0;

pub(crate) fn health_ratio() -> f32 {
    40.0 / 240.0
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HutAnimalKind {
    GreenSnake,
    GreenLizard,
    DesertRabit,
    Raptor,
    MiniRaptor,
    PigDino,
    YellowWorm,
    ArcticRabit,
    Penguin,
    WhiteWolf,
    Ostrich,
    Rat,
    Turtle,
    RedWorm,
    GreenEyedFox,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HutAnimalState {
    Idle,
    Looking,
    Walking,
}

#[derive(Component)]
pub(crate) struct HutAnimal {
    pub(super) hut_ref_id: u32,
    pub(super) kind: HutAnimalKind,
    pub(super) state: HutAnimalState,
    pub(super) home_map: Vec2,
    pub(super) position_map: Vec2,
    pub(super) target_map: Vec2,
    pub(super) velocity_map: Vec2,
    pub(super) direction: usize,
    pub(super) idle_timer: f32,
    pub(super) going_home: bool,
}

pub(super) fn build_hut_animal(
    map: &ZMap,
    passability: &PassabilityGrid,
    rng: &mut CombatRng,
    hut_ref_id: u32,
    grid: MapGridPosition,
) -> Option<HutAnimal> {
    let exit_tile = hut_exit_tile(passability, grid, rng)?;
    let home_map = hut_home_map(grid);
    let kind = random_hut_animal_kind(map.basics.terrain_type, rng);
    let mut animal = HutAnimal {
        hut_ref_id,
        kind,
        state: HutAnimalState::Idle,
        home_map,
        position_map: home_map,
        target_map: home_map,
        velocity_map: Vec2::ZERO,
        direction: 0,
        idle_timer: 0.0,
        going_home: false,
    };
    goto_hut_animal_tile(&mut animal, exit_tile);

    Some(animal)
}

pub(super) fn hut_animal_max_animals(rng: &mut CombatRng) -> usize {
    let diff = HUT_ANIMAL_MAX.saturating_sub(HUT_ANIMAL_MIN);
    if diff == 0 {
        HUT_ANIMAL_MIN
    } else {
        HUT_ANIMAL_MIN + rng.index(diff)
    }
}

fn random_hut_animal_kind(planet: PlanetType, rng: &mut CombatRng) -> HutAnimalKind {
    let palette = hut_animal_palette(planet);
    palette[rng.index(palette.len())]
}

fn hut_animal_palette(planet: PlanetType) -> &'static [HutAnimalKind] {
    use HutAnimalKind::*;
    match planet {
        PlanetType::Desert => &[GreenSnake, GreenLizard, DesertRabit],
        PlanetType::Volcanic => &[Raptor, MiniRaptor, PigDino, YellowWorm],
        PlanetType::Arctic => &[ArcticRabit, Penguin, WhiteWolf],
        PlanetType::Jungle => &[Ostrich, Rat, Turtle],
        PlanetType::City => &[RedWorm, Rat, GreenEyedFox],
    }
}

fn hut_home_map(grid: MapGridPosition) -> Vec2 {
    Vec2::new(
        grid.x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        grid.y as f32 * TILE_SIZE + TILE_SIZE * 0.5,
    )
}

fn hut_exit_tile(
    passability: &PassabilityGrid,
    grid: MapGridPosition,
    rng: &mut CombatRng,
) -> Option<IVec2> {
    let default = IVec2::new(grid.x as i32, grid.y as i32 + 1);
    if passability.is_walkable(default) {
        return Some(default);
    }

    let mut candidates = Vec::new();
    for y in grid.y as i32 - 1..=grid.y as i32 + 1 {
        for x in grid.x as i32 - 1..=grid.x as i32 + 1 {
            let tile = IVec2::new(x, y);
            if tile != IVec2::new(grid.x as i32, grid.y as i32) && passability.is_walkable(tile) {
                candidates.push(tile);
            }
        }
    }

    (!candidates.is_empty()).then(|| candidates[rng.index(candidates.len())])
}

pub(super) fn goto_random_hut_animal_tile(
    animal: &mut HutAnimal,
    passability: &PassabilityGrid,
    rng: &mut CombatRng,
) -> bool {
    let current_tile = IVec2::new(
        (animal.position_map.x / TILE_SIZE).floor() as i32,
        (animal.position_map.y / TILE_SIZE).floor() as i32,
    );
    let mut possible = Vec::new();
    let mut preferred = Vec::new();

    for y in current_tile.y - 1..=current_tile.y + 1 {
        for x in current_tile.x - 1..=current_tile.x + 1 {
            let tile = IVec2::new(x, y);
            if tile == current_tile || !passability.is_walkable(tile) {
                continue;
            }
            let target = hut_animal_tile_center_map(tile);
            if !points_within_original_distance(
                animal.home_map.x as i32,
                animal.home_map.y as i32,
                target.x as i32,
                target.y as i32,
                HUT_ANIMAL_ROAM_DISTANCE,
            ) {
                continue;
            }
            let direction =
                direction_index_from_delta(map_point_to_world(target - animal.position_map));
            possible.push(tile);
            if direction.is_some_and(|direction| {
                hut_animal_preferred_direction(animal.direction, direction)
            }) {
                preferred.push(tile);
            }
        }
    }

    if possible.is_empty() {
        return false;
    }

    let tile = if !preferred.is_empty() && rng.index(5) != 0 {
        preferred[rng.index(preferred.len())]
    } else {
        possible[rng.index(possible.len())]
    };
    goto_hut_animal_tile(animal, tile);
    true
}

fn goto_hut_animal_tile(animal: &mut HutAnimal, tile: IVec2) {
    animal.target_map = hut_animal_tile_center_map(tile);
    let delta_map = animal.target_map - animal.position_map;
    animal.direction =
        direction_index_from_delta(map_point_to_world(delta_map)).unwrap_or(animal.direction);
    animal.velocity_map = delta_map.try_normalize().unwrap_or(Vec2::ZERO) * HUT_ANIMAL_MOVE_SPEED;
    animal.state = HutAnimalState::Walking;
}

pub(super) fn goto_hut_animal_home(animal: &mut HutAnimal) {
    let home_tile = IVec2::new(
        (animal.home_map.x / TILE_SIZE).floor() as i32,
        (animal.home_map.y / TILE_SIZE).floor() as i32,
    );
    goto_hut_animal_tile(animal, home_tile);
}

fn hut_animal_tile_center_map(tile: IVec2) -> Vec2 {
    Vec2::new(
        tile.x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        tile.y as f32 * TILE_SIZE + TILE_SIZE * 0.5,
    )
}

pub(super) fn advance_hut_animal_position(animal: &mut HutAnimal, delta_secs: f32) -> bool {
    let remaining = animal.target_map - animal.position_map;
    let travel = animal.velocity_map * delta_secs;
    if travel.length_squared() >= remaining.length_squared() {
        animal.position_map = animal.target_map;
        animal.velocity_map = Vec2::ZERO;
        return true;
    }

    animal.position_map += travel;
    false
}

pub(super) fn hut_animal_idle_time(rng: &mut CombatRng) -> f32 {
    0.1 + rng.index(10) as f32 * 0.1
}

fn hut_animal_preferred_direction(current: usize, next: usize) -> bool {
    let diff = current.abs_diff(next);
    !(3..=5).contains(&diff)
}

fn points_within_original_distance(x1: i32, y1: i32, x2: i32, y2: i32, distance: i32) -> bool {
    if x2 < x1 - distance || x2 > x1 + distance || y2 < y1 - distance || y2 > y1 + distance {
        return false;
    }

    let sh_dist = (distance as f32 * 0.707106781) as i32;
    let dx = (x1 - x2).abs();
    let dy = (y1 - y2).abs();
    if dx < sh_dist && dy < sh_dist {
        return true;
    }

    ((dx * dx + dy * dy) as f32).sqrt() <= distance as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hut_animal_palettes_match_original_planets() {
        use HutAnimalKind::*;

        assert_eq!(
            hut_animal_palette(PlanetType::Desert),
            &[GreenSnake, GreenLizard, DesertRabit]
        );
        assert_eq!(
            hut_animal_palette(PlanetType::Volcanic),
            &[Raptor, MiniRaptor, PigDino, YellowWorm]
        );
        assert_eq!(
            hut_animal_palette(PlanetType::Arctic),
            &[ArcticRabit, Penguin, WhiteWolf]
        );
        assert_eq!(
            hut_animal_palette(PlanetType::Jungle),
            &[Ostrich, Rat, Turtle]
        );
        assert_eq!(
            hut_animal_palette(PlanetType::City),
            &[RedWorm, Rat, GreenEyedFox]
        );
    }

    #[test]
    fn hut_animal_original_ranges_are_preserved() {
        let mut rng = CombatRng::default();
        let mut saw_three = false;
        let mut saw_four = false;

        for _ in 0..200 {
            let max = hut_animal_max_animals(&mut rng);
            assert!((HUT_ANIMAL_MIN..HUT_ANIMAL_MAX).contains(&max));
            saw_three |= max == 3;
            saw_four |= max == 4;

            let idle = hut_animal_idle_time(&mut rng);
            assert!((0.1..=1.0).contains(&idle));
            assert!(((idle * 10.0).round() - idle * 10.0).abs() < 0.001);
        }

        assert!(saw_three);
        assert!(saw_four);
    }

    #[test]
    fn hut_animal_direction_preference_and_roam_distance_match_original_helpers() {
        assert!(hut_animal_preferred_direction(0, 0));
        assert!(hut_animal_preferred_direction(0, 2));
        assert!(!hut_animal_preferred_direction(0, 3));
        assert!(!hut_animal_preferred_direction(0, 4));
        assert!(!hut_animal_preferred_direction(0, 5));
        assert!(hut_animal_preferred_direction(0, 6));

        assert!(points_within_original_distance(0, 0, 79, 79, 112));
        assert!(points_within_original_distance(0, 0, 112, 0, 112));
        assert!(!points_within_original_distance(0, 0, 113, 0, 112));
        assert!(!points_within_original_distance(0, 0, 80, 80, 112));
    }
}
