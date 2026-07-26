use bevy::prelude::{
    AssetServer, Commands, Component, Entity, Name, Query, Res, ResMut, Sprite, Time, Transform,
    Vec2, Vec3, With, Without,
};

use super::animal_logic::{self as animal, HutAnimal, HutAnimalKind, HutAnimalState};
use crate::{
    components::{
        CombatRng, CurrentMap, DestroyedObject, GameObjectEntity, HutAnimalSpawner,
        MapGridPosition, ObjectStats, PassabilityGrid,
    },
    map_top_left_to_world, rotation_for_direction,
};

pub(crate) const HUT_ANIMAL_WALK_FRAME_TIME: f32 = 0.2;
pub(crate) const HUT_ANIMAL_LOOK_FRAME_TIME: f32 = 0.35;

#[derive(Component, Default)]
pub(crate) struct HutAnimalAnimation {
    walk_frame: usize,
    look_frame: usize,
    walk_elapsed: f32,
    look_elapsed: f32,
}

pub(crate) fn default_selection_size() -> Vec2 {
    Vec2::splat(16.0)
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
    mut animals: Query<(&mut HutAnimal, &mut HutAnimalAnimation)>,
) {
    let delta_secs = time.delta_secs();

    for (hut, grid, stats, mut spawner) in &mut huts {
        if stats.destroyed() {
            continue;
        }

        spawner.max_timer -= delta_secs;
        if spawner.max_animals == 0 || spawner.max_timer <= 0.0 {
            spawner.max_animals = animal::hut_animal_max_animals(&mut rng);
            spawner.max_timer = animal::HUT_ANIMAL_MAX_INTERVAL;
        }

        spawner.animal_timer -= delta_secs;
        if spawner.animal_timer > 0.0 {
            continue;
        }
        spawner.animal_timer = animal::HUT_ANIMAL_POPULATION_INTERVAL;

        let current_animals = animals
            .iter()
            .filter(|(animal, _)| animal.hut_ref_id == hut.ref_id)
            .count();
        let displacement = spawner.max_animals as isize - current_animals as isize;

        if displacement > 0 {
            let spawn_amount = rng.index(displacement as usize + 1);
            for _ in 0..spawn_amount {
                if let Some(animal) =
                    animal::build_hut_animal(&map.0, &passability, &mut rng, hut.ref_id, *grid)
                {
                    spawn_hut_animal_sprite(&mut commands, &asset_server, animal);
                }
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
    mut animals: Query<(
        Entity,
        &mut HutAnimal,
        &mut HutAnimalAnimation,
        &mut Transform,
        &mut Sprite,
    )>,
) {
    let active_huts: Vec<u32> = huts
        .iter()
        .filter_map(|(hut, stats)| (!stats.destroyed()).then_some(hut.ref_id))
        .collect();
    let delta_secs = time.delta_secs();

    for (entity, mut animal, mut animation, mut transform, mut sprite) in &mut animals {
        if !active_huts.contains(&animal.hut_ref_id) {
            commands.entity(entity).despawn();
            continue;
        }

        let mut despawn = false;
        match animal.state {
            HutAnimalState::Walking => {
                advance_hut_animal_walk_frame(animal.kind, &mut animation, delta_secs);
                if animal::advance_hut_animal_position(&mut animal, delta_secs) {
                    if animal.going_home
                        && animal.position_map.distance_squared(animal.home_map) < 0.01
                    {
                        despawn = true;
                    } else if rng.index(3) != 0 {
                        if !animal::goto_random_hut_animal_tile(&mut animal, &passability, &mut rng)
                        {
                            despawn = true;
                        }
                    } else {
                        set_hut_animal_idle(&mut animal, &mut animation, &mut rng);
                    }
                }
            }
            HutAnimalState::Idle => {
                animal.idle_timer -= delta_secs;
                if animal.idle_timer <= 0.0 {
                    if hut_animal_look_frame_count(animal.kind) == 0 || rng.index(5) != 0 {
                        if !animal::goto_random_hut_animal_tile(&mut animal, &passability, &mut rng)
                        {
                            despawn = true;
                        }
                    } else {
                        animal.state = HutAnimalState::Looking;
                        animation.look_frame = 0;
                        animation.look_elapsed = 0.0;
                    }
                }
            }
            HutAnimalState::Looking => {
                animation.look_elapsed += delta_secs;
                if animation.look_elapsed >= HUT_ANIMAL_LOOK_FRAME_TIME {
                    animation.look_elapsed %= HUT_ANIMAL_LOOK_FRAME_TIME;
                    animation.look_frame += 1;
                    if animation.look_frame >= hut_animal_look_frame_count(animal.kind) {
                        if rng.index(5) != 0 {
                            set_hut_animal_idle(&mut animal, &mut animation, &mut rng);
                        } else {
                            animation.look_frame = 0;
                        }
                    }
                }
            }
        }

        if despawn {
            commands.entity(entity).despawn();
            continue;
        }

        sync_hut_animal_sprite(
            &asset_server,
            &animal,
            &animation,
            &mut transform,
            &mut sprite,
        );
    }
}

fn spawn_hut_animal_sprite(commands: &mut Commands, asset_server: &AssetServer, animal: HutAnimal) {
    let world = map_top_left_to_world(animal.position_map);
    let animation = HutAnimalAnimation::default();
    commands.spawn((
        Sprite::from_image(asset_server.load(hut_animal_frame_path_for(&animal, &animation))),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(world.x, world.y, hut_animal_z(animal.position_map)),
        animation,
        animal,
        Name::new("hut_animal"),
    ));
}

fn sync_hut_animal_sprite(
    asset_server: &AssetServer,
    animal: &HutAnimal,
    animation: &HutAnimalAnimation,
    transform: &mut Transform,
    sprite: &mut Sprite,
) {
    let world = map_top_left_to_world(animal.position_map);
    transform.translation = Vec3::new(world.x, world.y, hut_animal_z(animal.position_map));
    sprite.image = asset_server.load(hut_animal_frame_path_for(animal, animation));
}

fn hut_animal_frame_path_for(animal: &HutAnimal, animation: &HutAnimalAnimation) -> String {
    hut_animal_frame_path(
        animal.kind,
        animal.state,
        animal.direction,
        animation.walk_frame,
        animation.look_frame,
    )
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

fn advance_hut_animal_walk_frame(
    kind: HutAnimalKind,
    animation: &mut HutAnimalAnimation,
    delta_secs: f32,
) {
    animation.walk_elapsed += delta_secs;
    if animation.walk_elapsed < HUT_ANIMAL_WALK_FRAME_TIME {
        return;
    }
    animation.walk_elapsed %= HUT_ANIMAL_WALK_FRAME_TIME;
    animation.walk_frame = (animation.walk_frame + 1) % hut_animal_walk_frame_count(kind);
}

fn set_hut_animal_idle(
    animal: &mut HutAnimal,
    animation: &mut HutAnimalAnimation,
    rng: &mut CombatRng,
) {
    animal.state = HutAnimalState::Idle;
    animal.velocity_map = Vec2::ZERO;
    animal.idle_timer = animal::hut_animal_idle_time(rng);
    reset_hut_animal_idle_visual(animal.kind, animation);
}

fn reset_hut_animal_idle_visual(kind: HutAnimalKind, animation: &mut HutAnimalAnimation) {
    animation.look_frame = 0;
    animation.look_elapsed = 0.0;
    if hut_animal_walk_to_zero(kind) {
        animation.walk_frame = 0;
    }
}

fn send_hut_animals_home(
    animals: &mut Query<(&mut HutAnimal, &mut HutAnimalAnimation)>,
    rng: &mut CombatRng,
    hut_ref_id: u32,
    amount: usize,
) {
    let going_home = animals
        .iter()
        .filter(|(animal, _)| animal.hut_ref_id == hut_ref_id && animal.going_home)
        .count();
    let mut remaining = amount.saturating_sub(going_home);
    if remaining == 0 {
        return;
    }

    let mut candidates: Vec<usize> = animals
        .iter()
        .enumerate()
        .filter_map(|(index, (animal, _))| {
            (animal.hut_ref_id == hut_ref_id && !animal.going_home).then_some(index)
        })
        .collect();

    while remaining > 0 && !candidates.is_empty() {
        let candidate_index = rng.index(candidates.len());
        let animal_index = candidates.swap_remove(candidate_index);
        if let Some((mut animal, mut animation)) = animals.iter_mut().nth(animal_index) {
            animal.going_home = true;
            animal::goto_hut_animal_home(&mut animal);
            reset_hut_animal_idle_visual(animal.kind, &mut animation);
            remaining -= 1;
        }
    }
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

#[cfg(test)]
pub(crate) fn hut_animal_dead_frame_path(kind: HutAnimalKind, facing_up: bool) -> String {
    let name = hut_animal_asset_name(kind);
    let direction = if facing_up { "up" } else { "down" };
    format!("other/hut_animals/{name}_dead_{direction}.png")
}

fn hut_animal_z(position_map: Vec2) -> f32 {
    4.8 + position_map.y * 0.00001
}

fn hut_animal_look_rotation(direction: usize) -> u16 {
    match direction % 8 {
        0 | 1 => 0,
        2 | 3 => 90,
        4 | 5 => 180,
        _ => 270,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
