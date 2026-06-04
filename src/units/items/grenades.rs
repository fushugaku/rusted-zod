use bevy::prelude::Vec2;

use crate::{
    components::{CombatRng, DamageCrater},
    original::{
        objects::{ItemType, ObjectKind, RobotType},
        settings::{
            GRENADE_BOX_EXPLOSION_DELAY, GRENADE_BOX_EXPLOSION_RADIUS,
            GRENADE_BOX_SCATTER_HALF_EXTENT, GRENADE_DAMAGE, GRENADES_PER_BOX,
        },
    },
};

pub(crate) use super::grenades_ui::{asset_path, default_selection_size, projectile_frame_paths};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DestroyMissileRule {
    pub(crate) target: Vec2,
    pub(crate) delay: f32,
}

pub(crate) fn is_grenade_box(kind: ObjectKind) -> bool {
    matches!(kind, ObjectKind::MapItem(id) if id == ItemType::Grenades as u8)
}

pub(crate) fn can_pickup_grenades(kind: ObjectKind, current_amount: u8) -> bool {
    can_have_grenades(kind) && current_amount == 0
}

pub(crate) fn can_have_grenades(kind: ObjectKind) -> bool {
    matches!(kind, ObjectKind::Robot(robot) if robot_can_have_grenades(robot))
}

pub(crate) fn transfer_amount(current_amount: u8, box_amount: u8) -> (u8, u8) {
    ((current_amount as u16 + box_amount as u16).min(99) as u8, 0)
}

pub(crate) fn default_box_amount() -> u8 {
    GRENADES_PER_BOX
}

pub(crate) fn destroy_missile_rules(
    position: Vec2,
    grenade_amount: u8,
    rng: &mut CombatRng,
) -> Vec<DestroyMissileRule> {
    (0..grenade_amount)
        .map(|_| {
            let target = position
                + Vec2::new(
                    rng.scatter(GRENADE_BOX_SCATTER_HALF_EXTENT),
                    rng.scatter(GRENADE_BOX_SCATTER_HALF_EXTENT),
                );
            let delay = GRENADE_BOX_EXPLOSION_DELAY + rng.next_roll();
            DestroyMissileRule { target, delay }
        })
        .collect()
}

pub(crate) fn destroy_missile_damage() -> f32 {
    GRENADE_DAMAGE
}

pub(crate) fn destroy_missile_radius() -> f32 {
    GRENADE_BOX_EXPLOSION_RADIUS
}

pub(crate) fn destroy_missile_crater() -> DamageCrater {
    DamageCrater {
        is_big: false,
        chance: 0.35,
        big_chance: None,
    }
}

fn robot_can_have_grenades(robot: RobotType) -> bool {
    !matches!(robot, RobotType::Tough)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::BuildingType;

    #[test]
    fn grenade_box_assets_and_defaults_match_original() {
        assert_eq!(asset_path(), "other/map_items/grenades.png");
        assert_eq!(default_box_amount(), 20);
    }

    #[test]
    fn grenade_box_kind_and_pickup_rules_match_original() {
        assert!(is_grenade_box(ObjectKind::MapItem(
            ItemType::Grenades as u8
        )));
        assert!(!is_grenade_box(ObjectKind::MapItem(ItemType::Flag as u8)));
        assert!(!is_grenade_box(ObjectKind::MapItem(ItemType::Rock as u8)));

        assert!(can_pickup_grenades(ObjectKind::Robot(RobotType::Grunt), 0));
        assert!(!can_pickup_grenades(ObjectKind::Robot(RobotType::Grunt), 1));
        assert!(!can_pickup_grenades(ObjectKind::Robot(RobotType::Tough), 0));
        assert!(!can_pickup_grenades(
            ObjectKind::Building(BuildingType::FortBack),
            0
        ));
        assert_eq!(transfer_amount(95, 20), (99, 0));
    }

    #[test]
    fn grenade_box_destroy_spawns_one_delayed_missile_per_grenade() {
        let mut rng = CombatRng::default();
        let missiles = destroy_missile_rules(Vec2::new(10.0, 20.0), 20, &mut rng);

        assert_eq!(missiles.len(), 20);
        assert!(missiles.iter().all(|missile| {
            missile.target.x >= 10.0 - GRENADE_BOX_SCATTER_HALF_EXTENT
                && missile.target.x <= 10.0 + GRENADE_BOX_SCATTER_HALF_EXTENT
                && missile.target.y >= 20.0 - GRENADE_BOX_SCATTER_HALF_EXTENT
                && missile.target.y <= 20.0 + GRENADE_BOX_SCATTER_HALF_EXTENT
                && missile.delay >= GRENADE_BOX_EXPLOSION_DELAY
                && missile.delay < GRENADE_BOX_EXPLOSION_DELAY + 1.0
        }));
        assert_eq!(destroy_missile_damage(), GRENADE_DAMAGE);
        assert_eq!(destroy_missile_radius(), GRENADE_BOX_EXPLOSION_RADIUS);
        assert_eq!(destroy_missile_crater().chance, 0.35);
    }
}
