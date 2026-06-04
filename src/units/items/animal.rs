use bevy::prelude::*;

use crate::{
    components::{
        CombatRng, CurrentMap, DestroyedObject, GameObjectEntity, HutAnimalSpawner,
        MapGridPosition, ObjectStats, PassabilityGrid,
    },
    constants::TILE_SIZE,
    direction_index_from_delta, map_point_to_world, map_top_left_to_world,
    original::{map::ZMap, types::PlanetType},
    rotation_for_direction,
};

pub(crate) const HUT_ANIMAL_MIN: usize = 3;
pub(crate) const HUT_ANIMAL_MAX: usize = 5;
const HUT_ANIMAL_ROAM_DISTANCE: i32 = 7 * TILE_SIZE as i32;
const HUT_ANIMAL_MOVE_SPEED: f32 = 15.0;
const HUT_ANIMAL_WALK_FRAME_TIME: f32 = 0.2;
const HUT_ANIMAL_LOOK_FRAME_TIME: f32 = 0.35;
const HUT_ANIMAL_POPULATION_INTERVAL: f32 = 1.0;
const HUT_ANIMAL_MAX_INTERVAL: f32 = 10.0;

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(16.0)
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
    hut_ref_id: u32,
    kind: HutAnimalKind,
    state: HutAnimalState,
    home_map: Vec2,
    position_map: Vec2,
    target_map: Vec2,
    velocity_map: Vec2,
    direction: usize,
    walk_frame: usize,
    look_frame: usize,
    idle_timer: f32,
    walk_elapsed: f32,
    look_elapsed: f32,
    going_home: bool,
}

pub(crate) fn process_hut_animal_spawners(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    map: Res<CurrentMap>,
    passability: Res<PassabilityGrid>,
    mut rng: ResMut<CombatRng>,
    mut huts: Query<
        (
            &GameObjectEntity,
            &MapGridPosition,
            &ObjectStats,
            &mut HutAnimalSpawner,
        ),
        (Without<DestroyedObject>, Without<HutAnimal>),
    >,
    mut animals: Query<&mut HutAnimal>,
) {
    let delta_secs = time.delta_secs();

    for (hut, grid, stats, mut spawner) in &mut huts {
        if stats.destroyed() {
            continue;
        }

        spawner.max_timer -= delta_secs;
        if spawner.max_animals == 0 || spawner.max_timer <= 0.0 {
            spawner.max_animals = hut_animal_max_animals(&mut rng);
            spawner.max_timer = HUT_ANIMAL_MAX_INTERVAL;
        }

        spawner.animal_timer -= delta_secs;
        if spawner.animal_timer > 0.0 {
            continue;
        }
        spawner.animal_timer = HUT_ANIMAL_POPULATION_INTERVAL;

        let current_animals = animals
            .iter()
            .filter(|animal| animal.hut_ref_id == hut.ref_id)
            .count();
        let displacement = spawner.max_animals as isize - current_animals as isize;

        if displacement > 0 {
            let spawn_amount = rng.index(displacement as usize + 1);
            for _ in 0..spawn_amount {
                spawn_hut_animal(
                    &mut commands,
                    &asset_server,
                    &map.0,
                    &passability,
                    &mut rng,
                    hut.ref_id,
                    *grid,
                );
            }
        } else if displacement < 0 {
            let send_amount = rng.index((-displacement) as usize + 1);
            send_hut_animals_home(&mut animals, &mut rng, hut.ref_id, send_amount);
        }
    }
}

pub(crate) fn process_hut_animal_movement(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    passability: Res<PassabilityGrid>,
    mut rng: ResMut<CombatRng>,
    huts: Query<(&GameObjectEntity, &ObjectStats), (With<HutAnimalSpawner>, Without<HutAnimal>)>,
    mut animals: Query<(Entity, &mut HutAnimal, &mut Transform, &mut Sprite)>,
) {
    let active_huts: Vec<u32> = huts
        .iter()
        .filter_map(|(hut, stats)| (!stats.destroyed()).then_some(hut.ref_id))
        .collect();
    let delta_secs = time.delta_secs();

    for (entity, mut animal, mut transform, mut sprite) in &mut animals {
        if !active_huts.contains(&animal.hut_ref_id) {
            commands.entity(entity).despawn();
            continue;
        }

        let mut despawn = false;
        match animal.state {
            HutAnimalState::Walking => {
                advance_hut_animal_walk_frame(&mut animal, delta_secs);
                if advance_hut_animal_position(&mut animal, delta_secs) {
                    if animal.going_home
                        && animal.position_map.distance_squared(animal.home_map) < 0.01
                    {
                        despawn = true;
                    } else if rng.index(3) != 0 {
                        if !goto_random_hut_animal_tile(&mut animal, &passability, &mut rng) {
                            despawn = true;
                        }
                    } else {
                        set_hut_animal_idle(&mut animal, &mut rng);
                    }
                }
            }
            HutAnimalState::Idle => {
                animal.idle_timer -= delta_secs;
                if animal.idle_timer <= 0.0 {
                    if hut_animal_look_frame_count(animal.kind) == 0 || rng.index(5) != 0 {
                        if !goto_random_hut_animal_tile(&mut animal, &passability, &mut rng) {
                            despawn = true;
                        }
                    } else {
                        animal.state = HutAnimalState::Looking;
                        animal.look_frame = 0;
                        animal.look_elapsed = 0.0;
                    }
                }
            }
            HutAnimalState::Looking => {
                animal.look_elapsed += delta_secs;
                if animal.look_elapsed >= HUT_ANIMAL_LOOK_FRAME_TIME {
                    animal.look_elapsed %= HUT_ANIMAL_LOOK_FRAME_TIME;
                    animal.look_frame += 1;
                    if animal.look_frame >= hut_animal_look_frame_count(animal.kind) {
                        if rng.index(5) != 0 {
                            set_hut_animal_idle(&mut animal, &mut rng);
                        } else {
                            animal.look_frame = 0;
                        }
                    }
                }
            }
        }

        if despawn {
            commands.entity(entity).despawn();
            continue;
        }

        let world = map_top_left_to_world(animal.position_map);
        transform.translation = Vec3::new(world.x, world.y, hut_animal_z(animal.position_map));
        sprite.image = asset_server.load(hut_animal_frame_path(
            animal.kind,
            animal.state,
            animal.direction,
            animal.walk_frame,
            animal.look_frame,
        ));
    }
}

fn spawn_hut_animal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    map: &ZMap,
    passability: &PassabilityGrid,
    rng: &mut CombatRng,
    hut_ref_id: u32,
    grid: MapGridPosition,
) {
    let Some(exit_tile) = hut_exit_tile(passability, grid, rng) else {
        return;
    };
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
        walk_frame: 0,
        look_frame: 0,
        idle_timer: 0.0,
        walk_elapsed: 0.0,
        look_elapsed: 0.0,
        going_home: false,
    };
    goto_hut_animal_tile(&mut animal, exit_tile);

    let world = map_top_left_to_world(animal.position_map);
    commands.spawn((
        Sprite::from_image(asset_server.load(hut_animal_frame_path(
            animal.kind,
            animal.state,
            animal.direction,
            animal.walk_frame,
            animal.look_frame,
        ))),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(world.x, world.y, hut_animal_z(animal.position_map)),
        animal,
        Name::new("hut_animal"),
    ));
}

fn send_hut_animals_home(
    animals: &mut Query<&mut HutAnimal>,
    rng: &mut CombatRng,
    hut_ref_id: u32,
    amount: usize,
) {
    let going_home = animals
        .iter()
        .filter(|animal| animal.hut_ref_id == hut_ref_id && animal.going_home)
        .count();
    let mut remaining = amount.saturating_sub(going_home);
    if remaining == 0 {
        return;
    }

    let mut candidates: Vec<usize> = animals
        .iter()
        .enumerate()
        .filter_map(|(index, animal)| {
            (animal.hut_ref_id == hut_ref_id && !animal.going_home).then_some(index)
        })
        .collect();

    while remaining > 0 && !candidates.is_empty() {
        let candidate_index = rng.index(candidates.len());
        let animal_index = candidates.swap_remove(candidate_index);
        if let Some(mut animal) = animals.iter_mut().nth(animal_index) {
            animal.going_home = true;
            let home_tile = IVec2::new(
                (animal.home_map.x / TILE_SIZE).floor() as i32,
                (animal.home_map.y / TILE_SIZE).floor() as i32,
            );
            goto_hut_animal_tile(&mut animal, home_tile);
            remaining -= 1;
        }
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

fn goto_random_hut_animal_tile(
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
    animal.look_frame = 0;
    animal.look_elapsed = 0.0;
}

fn hut_animal_tile_center_map(tile: IVec2) -> Vec2 {
    Vec2::new(
        tile.x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        tile.y as f32 * TILE_SIZE + TILE_SIZE * 0.5,
    )
}

fn advance_hut_animal_position(animal: &mut HutAnimal, delta_secs: f32) -> bool {
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

fn advance_hut_animal_walk_frame(animal: &mut HutAnimal, delta_secs: f32) {
    animal.walk_elapsed += delta_secs;
    if animal.walk_elapsed < HUT_ANIMAL_WALK_FRAME_TIME {
        return;
    }
    animal.walk_elapsed %= HUT_ANIMAL_WALK_FRAME_TIME;
    animal.walk_frame = (animal.walk_frame + 1) % hut_animal_walk_frame_count(animal.kind);
}

fn set_hut_animal_idle(animal: &mut HutAnimal, rng: &mut CombatRng) {
    animal.state = HutAnimalState::Idle;
    animal.velocity_map = Vec2::ZERO;
    animal.idle_timer = hut_animal_idle_time(rng);
    animal.look_frame = 0;
    animal.look_elapsed = 0.0;
    if hut_animal_walk_to_zero(animal.kind) {
        animal.walk_frame = 0;
    }
}

fn hut_animal_idle_time(rng: &mut CombatRng) -> f32 {
    0.1 + rng.index(10) as f32 * 0.1
}

fn hut_animal_max_animals(rng: &mut CombatRng) -> usize {
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

fn hut_animal_asset_name(kind: HutAnimalKind) -> &'static str {
    match kind {
        HutAnimalKind::GreenSnake => "green_snake",
        HutAnimalKind::GreenLizard => "green_lizard",
        HutAnimalKind::DesertRabit => "desert_rabit",
        HutAnimalKind::Raptor => "raptor",
        HutAnimalKind::MiniRaptor => "mini_raptor",
        HutAnimalKind::PigDino => "pig_dino",
        HutAnimalKind::YellowWorm => "yellow_worm",
        HutAnimalKind::ArcticRabit => "arctic_rabit",
        HutAnimalKind::Penguin => "penguin",
        HutAnimalKind::WhiteWolf => "white_wolf",
        HutAnimalKind::Ostrich => "ostrich",
        HutAnimalKind::Rat => "rat",
        HutAnimalKind::Turtle => "turtle",
        HutAnimalKind::RedWorm => "red_worm",
        HutAnimalKind::GreenEyedFox => "green_eyed_fox",
    }
}

fn hut_animal_walk_frame_count(kind: HutAnimalKind) -> usize {
    if kind == HutAnimalKind::GreenSnake {
        8
    } else {
        4
    }
}

fn hut_animal_look_frame_count(kind: HutAnimalKind) -> usize {
    match kind {
        HutAnimalKind::GreenSnake | HutAnimalKind::YellowWorm | HutAnimalKind::RedWorm => 0,
        _ => 4,
    }
}

fn hut_animal_walk_to_zero(kind: HutAnimalKind) -> bool {
    matches!(
        kind,
        HutAnimalKind::DesertRabit | HutAnimalKind::PigDino | HutAnimalKind::ArcticRabit
    )
}

fn hut_animal_frame_path(
    kind: HutAnimalKind,
    state: HutAnimalState,
    direction: usize,
    walk_frame: usize,
    look_frame: usize,
) -> String {
    let name = hut_animal_asset_name(kind);
    match state {
        HutAnimalState::Looking if hut_animal_look_frame_count(kind) > 0 => {
            let rotation = hut_animal_look_rotation(direction);
            let frame = look_frame.min(hut_animal_look_frame_count(kind) - 1);
            format!("other/hut_animals/{name}_look_r{rotation:03}_n{frame:02}.png")
        }
        _ => {
            let rotation = rotation_for_direction(direction);
            let frame = walk_frame % hut_animal_walk_frame_count(kind);
            format!("other/hut_animals/{name}_walk_r{rotation:03}_n{frame:02}.png")
        }
    }
}

pub(crate) fn hut_animal_dead_frame_path(kind: HutAnimalKind, facing_up: bool) -> String {
    let name = hut_animal_asset_name(kind);
    let direction = if facing_up { "up" } else { "down" };
    format!("other/hut_animals/{name}_dead_{direction}.png")
}

fn hut_animal_look_rotation(direction: usize) -> u16 {
    match direction % 8 {
        0 | 1 => 0,
        2 | 3 => 90,
        4 | 5 => 180,
        _ => 270,
    }
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

fn hut_animal_z(position_map: Vec2) -> f32 {
    4.8 + position_map.y * 0.00001
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
    fn hut_animal_assets_and_frame_counts_match_original_names() {
        assert_eq!(hut_animal_walk_frame_count(HutAnimalKind::GreenSnake), 8);
        assert_eq!(hut_animal_walk_frame_count(HutAnimalKind::Rat), 4);
        assert_eq!(hut_animal_look_frame_count(HutAnimalKind::GreenSnake), 0);
        assert_eq!(hut_animal_look_frame_count(HutAnimalKind::YellowWorm), 0);
        assert_eq!(hut_animal_look_frame_count(HutAnimalKind::RedWorm), 0);
        assert_eq!(hut_animal_look_frame_count(HutAnimalKind::Penguin), 4);
        assert!(hut_animal_walk_to_zero(HutAnimalKind::DesertRabit));
        assert!(hut_animal_walk_to_zero(HutAnimalKind::PigDino));
        assert!(hut_animal_walk_to_zero(HutAnimalKind::ArcticRabit));
        assert!(!hut_animal_walk_to_zero(HutAnimalKind::Penguin));

        assert_eq!(
            hut_animal_frame_path(HutAnimalKind::GreenSnake, HutAnimalState::Walking, 6, 7, 0),
            "other/hut_animals/green_snake_walk_r270_n07.png"
        );
        assert_eq!(
            hut_animal_frame_path(HutAnimalKind::Penguin, HutAnimalState::Looking, 3, 2, 1),
            "other/hut_animals/penguin_look_r090_n01.png"
        );
        assert_eq!(
            hut_animal_frame_path(HutAnimalKind::RedWorm, HutAnimalState::Looking, 1, 3, 0),
            "other/hut_animals/red_worm_walk_r045_n03.png"
        );
        assert_eq!(
            hut_animal_dead_frame_path(HutAnimalKind::Raptor, true),
            "other/hut_animals/raptor_dead_up.png"
        );
        assert_eq!(
            hut_animal_dead_frame_path(HutAnimalKind::Raptor, false),
            "other/hut_animals/raptor_dead_down.png"
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
