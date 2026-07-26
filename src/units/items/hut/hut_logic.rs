use crate::{
    components::HutAnimalSpawner,
    original::objects::{ItemType, ObjectKind},
};

pub(crate) fn health_ratio() -> f32 {
    40.0 / 240.0
}

pub(crate) fn spawns_animals(kind: ObjectKind) -> bool {
    matches!(kind, ObjectKind::MapItem(id) if id == ItemType::Hut as u8)
}

pub(crate) fn initial_animal_spawner(kind: ObjectKind) -> Option<HutAnimalSpawner> {
    spawns_animals(kind).then_some(HutAnimalSpawner {
        max_animals: 0,
        animal_timer: 0.0,
        max_timer: 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hut_spawn_policy_initializes_empty_animal_spawner() {
        let spawner = initial_animal_spawner(ObjectKind::MapItem(ItemType::Hut as u8))
            .expect("hut should spawn animals");
        assert_eq!(spawner.max_animals, 0);
        assert_eq!(spawner.animal_timer, 0.0);
        assert_eq!(spawner.max_timer, 0.0);

        assert!(!spawns_animals(ObjectKind::MapItem(
            ItemType::Grenades as u8
        )));
        assert!(initial_animal_spawner(ObjectKind::MapItem(ItemType::Grenades as u8)).is_none());
    }
}
