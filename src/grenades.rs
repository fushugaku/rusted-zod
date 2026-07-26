use bevy::prelude::*;

use crate::{
    components::*,
    constants::TILE_SIZE,
    local_player::LocalPlayerState,
    object_sync::{
        ObjectDestroyPacketQueue, apply_object_grenade_amount_packet,
        apply_pickup_grenade_animation_packet, relay_destroy_object, relay_object_grenade_amount,
        relay_pickup_grenade_animation,
    },
    original::objects::ObjectKind,
    units::{items::grenades as grenade_item, robots},
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
        &ObjectTeam,
        &mut GrenadeInventory,
        &PickupGrenadesTarget,
        &MobileSpriteLayer,
        Option<&RobotGroup>,
        Option<&AttackTarget>,
    )>,
    mut object_query: Query<(&GameObjectEntity, &Transform, Option<&mut GrenadeBox>)>,
    local_player: Res<LocalPlayerState>,
    mut portrait_state: ResMut<PortraitAnimationState>,
    mut portrait_sounds: ResMut<PortraitAnimationSoundQueue>,
    mut space_bar_events: ResMut<SpaceBarEventQueue>,
    mut destroy_packets: ResMut<ObjectDestroyPacketQueue>,
) {
    for (
        robot_entity,
        robot,
        robot_transform,
        team,
        mut inventory,
        pickup,
        mobile,
        maybe_group,
        attack_target,
    ) in &mut robot_query
    {
        let Some(target) = find_pickup_waypoint_target(pickup.ref_id, &mut object_query) else {
            commands
                .entity(robot_entity)
                .remove::<PickupGrenadesTarget>();
            continue;
        };

        let is_minion = maybe_group.is_some_and(|group| group.leader_ref_id != robot.ref_id);
        let action = source_pickup_waypoint_action(
            can_pickup_grenades(robot.kind, inventory.amount),
            is_grenade_box(target.kind),
            robot_transform.translation.truncate(),
            target.position,
            is_minion,
        );

        match action {
            SourcePickupWaypointAction::KeepMoving => continue,
            SourcePickupWaypointAction::KillWaypoint => {
                commands
                    .entity(robot_entity)
                    .remove::<PickupGrenadesTarget>();
                continue;
            }
            SourcePickupWaypointAction::ApplyPickup => {}
        }

        let source_box_amount = target
            .box_amount
            .as_ref()
            .map_or(0, |box_amount| box_amount.amount);
        let mut target_box_amount = GrenadeBox {
            amount: source_box_amount,
        };
        transfer_grenade_box(&mut inventory, &mut target_box_amount);
        if let Some(mut target_box) = target.box_amount {
            target_box.amount = target_box_amount.amount;
        }
        if let Some(packet) = relay_object_grenade_amount(robot.ref_id, inventory.amount) {
            apply_object_grenade_amount_packet(&packet, robot.ref_id, &mut inventory);
        }
        let pickup_animation = relay_pickup_grenade_animation(robot.ref_id).is_some_and(|packet| {
            apply_pickup_grenade_animation_packet(
                &packet,
                robot.ref_id,
                can_have_grenades(robot.kind),
                attack_target.is_some(),
            )
        });
        {
            let mut entity_commands = commands.entity(robot_entity);
            entity_commands.remove::<PickupGrenadesTarget>();
            if pickup_animation {
                entity_commands.insert(RobotGrenadePickupAnimation {
                    upward: robots::grenade_pickup_uses_upward_frames(mobile.rotation),
                    frame: robots::grenade_pickup_start_frame(),
                    elapsed: 0.0,
                });
            }
        }
        if pickup_animation {
            apply_pickup_grenade_portrait_event(
                robot.ref_id,
                team.0,
                local_player.team(),
                &mut portrait_state,
                &mut portrait_sounds,
                &mut space_bar_events,
            );
        }
        relay_grenade_box_pickup_cleanup(&mut destroy_packets, pickup.ref_id);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourcePickupWaypointAction {
    KeepMoving,
    KillWaypoint,
    ApplyPickup,
}

fn source_pickup_waypoint_action(
    can_pickup: bool,
    target_is_grenade_box: bool,
    robot_center: Vec2,
    box_center: Vec2,
    is_minion: bool,
) -> SourcePickupWaypointAction {
    if !can_pickup || !target_is_grenade_box {
        return SourcePickupWaypointAction::KillWaypoint;
    }

    if !point_in_box(robot_center, box_center) {
        return SourcePickupWaypointAction::KeepMoving;
    }

    if is_minion {
        SourcePickupWaypointAction::KillWaypoint
    } else {
        SourcePickupWaypointAction::ApplyPickup
    }
}

fn apply_pickup_grenade_portrait_event(
    ref_id: u32,
    object_team: crate::original::types::TeamType,
    local_team: crate::original::types::TeamType,
    portrait_state: &mut PortraitAnimationState,
    portrait_sounds: &mut PortraitAnimationSoundQueue,
    space_bar_events: &mut SpaceBarEventQueue,
) -> bool {
    if object_team != local_team || portrait_state.doing_anim() {
        return false;
    }
    let kind = PortraitAnimationKind::GrenadesCollected;
    portrait_state.start(PortraitAnimationEvent { ref_id, kind });
    portrait_sounds.pending.push(kind);
    space_bar_events.add(SpaceBarEvent::new(ref_id, true, false));
    true
}

struct SourcePickupWaypointTarget<'a> {
    kind: ObjectKind,
    position: Vec2,
    box_amount: Option<Mut<'a, GrenadeBox>>,
}

fn find_pickup_waypoint_target<'a>(
    ref_id: u32,
    object_query: &'a mut Query<(&GameObjectEntity, &Transform, Option<&mut GrenadeBox>)>,
) -> Option<SourcePickupWaypointTarget<'a>> {
    object_query
        .iter_mut()
        .find(|(object, _, _)| object.ref_id == ref_id)
        .map(
            |(object, transform, box_amount)| SourcePickupWaypointTarget {
                kind: object.kind,
                position: transform.translation.truncate(),
                box_amount,
            },
        )
}

fn point_in_box(point: Vec2, box_center: Vec2) -> bool {
    let half = TILE_SIZE * 0.5;
    point.x >= box_center.x - half
        && point.x <= box_center.x + half
        && point.y >= box_center.y - half
        && point.y <= box_center.y + half
}

fn relay_grenade_box_pickup_cleanup(
    destroy_packets: &mut ObjectDestroyPacketQueue,
    ref_id: u32,
) -> bool {
    relay_destroy_object(destroy_packets, ref_id, None, true, false, false)
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
    use crate::original::types::TeamType;
    use crate::units::items::grenades::GRENADES_PER_BOX;

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

    #[test]
    fn grenade_box_pickup_cleanup_relays_source_destroy_packet() {
        let mut queue = ObjectDestroyPacketQueue::default();

        assert!(relay_grenade_box_pickup_cleanup(&mut queue, 12));
        assert_eq!(queue.pending.len(), 1);
        assert_eq!(queue.pending[0].ref_id, 12);
        assert_eq!(queue.pending[0].killer_ref_id, -1);
        assert!(queue.pending[0].destroy_object);
        assert!(!queue.pending[0].do_fire_death);
        assert!(!queue.pending[0].do_missile_death);
        assert!(queue.pending[0].fire_missiles.is_empty());

        assert!(!relay_grenade_box_pickup_cleanup(
            &mut queue,
            i32::MAX as u32 + 1
        ));
        assert_eq!(queue.pending.len(), 1);
    }

    #[test]
    fn pickup_waypoint_action_matches_source_kill_and_minion_rules() {
        let center = Vec2::new(16.0, 16.0);
        let far = Vec2::new(48.0, 16.0);

        assert_eq!(
            source_pickup_waypoint_action(false, true, center, center, false),
            SourcePickupWaypointAction::KillWaypoint
        );
        assert_eq!(
            source_pickup_waypoint_action(true, false, center, center, false),
            SourcePickupWaypointAction::KillWaypoint
        );
        assert_eq!(
            source_pickup_waypoint_action(true, true, far, center, false),
            SourcePickupWaypointAction::KeepMoving
        );
        assert_eq!(
            source_pickup_waypoint_action(true, true, center, center, true),
            SourcePickupWaypointAction::KillWaypoint
        );
        assert_eq!(
            source_pickup_waypoint_action(true, true, center, center, false),
            SourcePickupWaypointAction::ApplyPickup
        );
    }

    #[test]
    fn pickup_grenade_portrait_event_uses_source_client_guards() {
        let mut portrait_state = PortraitAnimationState::default();
        let mut portrait_sounds = PortraitAnimationSoundQueue::default();
        let mut space_bar_events = SpaceBarEventQueue::default();

        assert!(apply_pickup_grenade_portrait_event(
            7,
            TeamType::Red,
            TeamType::Red,
            &mut portrait_state,
            &mut portrait_sounds,
            &mut space_bar_events,
        ));
        assert!(portrait_state.doing_anim());
        assert_eq!(
            portrait_sounds.pending,
            vec![PortraitAnimationKind::GrenadesCollected]
        );
        assert_eq!(
            space_bar_events.events[0],
            SpaceBarEvent::new(7, true, false)
        );

        assert!(!apply_pickup_grenade_portrait_event(
            8,
            TeamType::Red,
            TeamType::Red,
            &mut portrait_state,
            &mut portrait_sounds,
            &mut space_bar_events,
        ));

        let mut other_team_state = PortraitAnimationState::default();
        let mut other_team_sounds = PortraitAnimationSoundQueue::default();
        let mut other_team_events = SpaceBarEventQueue::default();
        assert!(!apply_pickup_grenade_portrait_event(
            9,
            TeamType::Blue,
            TeamType::Red,
            &mut other_team_state,
            &mut other_team_sounds,
            &mut other_team_events,
        ));
        assert!(other_team_sounds.pending.is_empty());
        assert!(other_team_events.events.is_empty());
    }
}
