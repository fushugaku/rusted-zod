use bevy::prelude::*;

use crate::{
    components::*, constants::TILE_SIZE, original::objects::ObjectKind,
    units::items::grenades as grenade_item,
};

pub(crate) fn can_pickup_grenades(kind: ObjectKind, current_amount: u8) -> bool {
    grenade_item::can_pickup_grenades(kind, current_amount)
}

pub(crate) fn can_have_grenades(kind: ObjectKind) -> bool {
    grenade_item::can_have_grenades(kind)
}

pub(crate) fn is_grenade_box(kind: ObjectKind) -> bool {
    grenade_item::is_grenade_box(kind)
}

pub(crate) fn transfer_grenade_box(inventory: &mut GrenadeInventory, box_amount: &mut GrenadeBox) {
    (inventory.amount, box_amount.amount) =
        grenade_item::transfer_amount(inventory.amount, box_amount.amount);
}

pub(crate) fn process_grenade_pickups(
    mut commands: Commands,
    mut robot_query: Query<(
        Entity,
        &GameObjectEntity,
        &Transform,
        &mut GrenadeInventory,
        &PickupGrenadesTarget,
    )>,
    mut box_query: Query<(&GameObjectEntity, &Transform, &ObjectStats, &mut GrenadeBox)>,
    layer_query: Query<(Entity, &ObjectLayerRef)>,
    minimap_query: Query<(Entity, &MinimapDot)>,
) {
    for (robot_entity, robot, robot_transform, mut inventory, pickup) in &mut robot_query {
        if !can_pickup_grenades(robot.kind, inventory.amount) {
            commands
                .entity(robot_entity)
                .remove::<PickupGrenadesTarget>();
            continue;
        }

        let Some((box_transform, mut box_amount)) = find_grenade_box(pickup.ref_id, &mut box_query)
        else {
            commands
                .entity(robot_entity)
                .remove::<PickupGrenadesTarget>();
            continue;
        };

        if !point_in_box(
            robot_transform.translation.truncate(),
            box_transform.translation.truncate(),
        ) {
            continue;
        }

        transfer_grenade_box(&mut inventory, &mut box_amount);
        commands
            .entity(robot_entity)
            .remove::<PickupGrenadesTarget>();
        despawn_object_layers(&mut commands, pickup.ref_id, &layer_query);
        despawn_minimap_dot(&mut commands, pickup.ref_id, &minimap_query);
    }
}

fn find_grenade_box<'a>(
    ref_id: u32,
    box_query: &'a mut Query<(&GameObjectEntity, &Transform, &ObjectStats, &mut GrenadeBox)>,
) -> Option<(&'a Transform, Mut<'a, GrenadeBox>)> {
    box_query
        .iter_mut()
        .find(|(object, _, stats, box_amount)| {
            object.ref_id == ref_id
                && is_grenade_box(object.kind)
                && !stats.destroyed()
                && box_amount.amount > 0
        })
        .map(|(_, transform, _, box_amount)| (transform, box_amount))
}

fn point_in_box(point: Vec2, box_center: Vec2) -> bool {
    let half = TILE_SIZE * 0.5;
    point.x >= box_center.x - half
        && point.x <= box_center.x + half
        && point.y >= box_center.y - half
        && point.y <= box_center.y + half
}

fn despawn_object_layers(
    commands: &mut Commands,
    ref_id: u32,
    layer_query: &Query<(Entity, &ObjectLayerRef)>,
) {
    for (entity, layer_ref) in layer_query {
        if layer_ref.0 == ref_id {
            commands.entity(entity).despawn();
        }
    }
}

fn despawn_minimap_dot(
    commands: &mut Commands,
    ref_id: u32,
    minimap_query: &Query<(Entity, &MinimapDot)>,
) {
    for (entity, dot) in minimap_query {
        if dot.ref_id == ref_id {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn pickup_target_for_right_click(
    world_pos: Vec2,
    selected_refs: &[u32],
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &ObjectTeam,
        &ObjectStats,
        Option<&GrenadeInventory>,
    )>,
) -> Option<PickupCommand> {
    let target = grenade_box_at_position(world_pos, object_query)?;
    let robots: Vec<PickupRobot> = object_query
        .iter()
        .filter_map(|(object, transform, team, stats, inventory)| {
            if !selected_refs.contains(&object.ref_id)
                || team.0 == crate::original::types::TeamType::Null
                || stats.destroyed()
            {
                return None;
            }
            let inventory = inventory?;
            can_pickup_grenades(object.kind, inventory.amount).then_some(PickupRobot {
                ref_id: object.ref_id,
                position: transform.translation.truncate(),
                move_speed: stats.move_speed,
            })
        })
        .collect();

    (!robots.is_empty()).then_some(PickupCommand { target, robots })
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PickupCommand {
    pub(crate) target: PickupTarget,
    pub(crate) robots: Vec<PickupRobot>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PickupTarget {
    pub(crate) ref_id: u32,
    pub(crate) position: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PickupRobot {
    pub(crate) ref_id: u32,
    pub(crate) position: Vec2,
    pub(crate) move_speed: f32,
}

fn grenade_box_at_position(
    world_pos: Vec2,
    object_query: &Query<(
        &GameObjectEntity,
        &Transform,
        &ObjectTeam,
        &ObjectStats,
        Option<&GrenadeInventory>,
    )>,
) -> Option<PickupTarget> {
    object_query
        .iter()
        .filter_map(|(object, transform, _, stats, _)| {
            if stats.destroyed() || !is_grenade_box(object.kind) {
                return None;
            }
            let position = transform.translation.truncate();
            point_in_box(world_pos, position).then_some(PickupTarget {
                ref_id: object.ref_id,
                position,
            })
        })
        .min_by(|a, b| {
            a.position
                .distance_squared(world_pos)
                .total_cmp(&b.position.distance_squared(world_pos))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{BuildingType, ItemType, RobotType};
    use crate::original::settings::GRENADES_PER_BOX;

    #[test]
    fn can_pickup_grenades_matches_original_robot_rules() {
        assert!(can_pickup_grenades(ObjectKind::Robot(RobotType::Grunt), 0));
        assert!(!can_pickup_grenades(ObjectKind::Robot(RobotType::Grunt), 1));
        assert!(!can_pickup_grenades(ObjectKind::Robot(RobotType::Tough), 0));
        assert!(!can_pickup_grenades(
            ObjectKind::Building(BuildingType::FortBack),
            0
        ));
    }

    #[test]
    fn grenade_box_kind_matches_original_map_item() {
        assert!(is_grenade_box(ObjectKind::MapItem(
            ItemType::Grenades as u8
        )));
        assert!(!is_grenade_box(ObjectKind::MapItem(ItemType::Flag as u8)));
        assert!(!is_grenade_box(ObjectKind::MapItem(ItemType::Rock as u8)));
    }

    #[test]
    fn grenade_transfer_moves_box_amount_and_empties_box() {
        let mut inventory = GrenadeInventory { amount: 0 };
        let mut box_amount = GrenadeBox {
            amount: GRENADES_PER_BOX,
        };

        transfer_grenade_box(&mut inventory, &mut box_amount);

        assert_eq!(inventory.amount, GRENADES_PER_BOX);
        assert_eq!(box_amount.amount, 0);
    }

    #[test]
    fn grenade_transfer_clamps_like_original_robot_setter() {
        let mut inventory = GrenadeInventory { amount: 95 };
        let mut box_amount = GrenadeBox { amount: 20 };

        transfer_grenade_box(&mut inventory, &mut box_amount);

        assert_eq!(inventory.amount, 99);
        assert_eq!(box_amount.amount, 0);
    }
}
