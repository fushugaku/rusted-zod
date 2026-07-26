use bevy::ecs::system::SystemParam;
use bevy::prelude::{
    AssetServer, Commands, Component, Entity, ParamSet, Query, Res, ResMut, Resource, Time,
    Transform, Vec2, With, Without,
};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    components::{
        AcceptedEmptyWaypointCommand, AttackTarget, AttackTargetLifecycleComponents,
        BridgeFootprint, BuildingProduction, BuildingProductionStatus, BuildingRallyPoints,
        CombatRng, CraneRepairTarget, CurrentMap, DamageCauseTimers, DestroyedObject, DriverHealth,
        EnterFortTarget, EnterTarget, GameObjectEntity, GrenadeInventory, HealthPercent, HudLayout,
        JustLeftCannon, MinimapDot, MobileSpriteLayer, MovementPath, MovementVelocity,
        MovementWaypoint, MovementWaypointMode, NextObjectRefId, ObjectLayerRef, ObjectStats,
        ObjectTeam, PickupGrenadesTarget, PortraitAnimationKind, RepairBuildingAnimState,
        RobotGroup, Selectable, SelectionHealthBar, SelectionMarker, SelectionState,
        SourceLocationInterpolation, SourceObjectLocation, UnitRepairTarget, VehicleLidState,
        area_is_fort_turret_tile,
    },
    constants::TILE_SIZE,
    local_player::LocalPlayerState,
    network_commands::{
        AttackObjectPacket, BuildingQueuePacket, BuildingQueueUnit, BuildingStatePacket,
        BuiltCannonListPacket, CommandPayload, CraneAnimPacket, DeleteObjectPacket,
        DestroyObjectMissileInfo, DestroyObjectPacket, DoPortraitAnimPacket, DriverHitEffectPacket,
        EjectVehiclePacket, ObjectGrenadeAmountPacket, ObjectGroupInfoPacket, ObjectHealthPacket,
        ObjectInitPacket, ObjectLocationPacket, ObjectTeamDriverInfo, ObjectTeamPacket,
        PickupGrenadeAnimationPacket, RepairBuildingAnimPacket, RequestObjectsCommand,
        SendRallypointsPacket, SendWaypointsPacket, SetLidOpenPacket, SnipeObjectPacket,
        SourceWaypoint, SourceWaypointMode,
    },
    original::{
        map::{MapObject, MapObjectType, ZMap},
        objects::{BuildingType, CannonType, ItemType, ObjectKind, RobotType, VehicleType},
        types::TeamType,
    },
    production::initial_production_for_building_from_source,
    render::atlas::GameAtlases,
    settings_sync::SourceSettingsState,
    units::{self, buildings, vehicles::crane_ui::CraneConcoVisualTarget},
    world_objects::{RuntimeSpawnLayer, spawn_runtime_object_from_source_init},
};

#[derive(Default, Resource)]
pub(crate) struct ObjectHealthPacketQueue {
    pub(crate) pending: Vec<ObjectHealthPacket>,
}

#[derive(Default, Resource)]
pub(crate) struct ObjectLocationPacketQueue {
    pub(crate) pending: Vec<ObjectLocationPacket>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SourceObjectEvent {
    DeferredUntilEarlyDrain {
        events: Vec<SourceObjectEvent>,
    },
    AddNewObject {
        packet: ObjectInitPacket,
        just_left_cannon: bool,
    },
    ObjectGroupInfo {
        packet: ObjectGroupInfoPacket,
        requires_new_object_refs: Vec<u32>,
    },
    UpdateHealth {
        packet: ObjectHealthPacket,
        requires_new_object_ref: Option<u32>,
    },
    BuildingState {
        packet: BuildingStatePacket,
        requires_new_object_ref: Option<u32>,
    },
    RepairBuildingAnimation {
        packet: RepairBuildingAnimPacket,
        requires_new_object_ref: Option<u32>,
    },
    BuildingQueue {
        packet: BuildingQueuePacket,
        requires_new_object_ref: Option<u32>,
    },
    ObjectWaypoints {
        packet: SendWaypointsPacket,
        requires_new_object_ref: Option<u32>,
        target_team: Option<TeamType>,
    },
    ObjectTeam {
        packet: ObjectTeamPacket,
        requires_new_object_ref: Option<u32>,
    },
    ObjectGrenadeAmount {
        packet: ObjectGrenadeAmountPacket,
    },
    ObjectRallyPoints {
        packet: SendRallypointsPacket,
        requires_new_object_ref: Option<u32>,
    },
    DeleteObject {
        packet: DeleteObjectPacket,
    },
    CommitEjectedDriverBatch {
        carrier_ref_id: u32,
        requires_new_object_refs: Vec<u32>,
    },
    CommitProducedObjectBatch {
        building_ref_id: u32,
        requires_new_object_refs: Vec<u32>,
    },
    CommitRepairedObjectBatch {
        building_ref_id: u32,
        requires_new_object_refs: Vec<u32>,
    },
    SetBuiltCannonAmount {
        packet: BuiltCannonListPacket,
        requires_new_object_ref: Option<u32>,
    },
}

#[derive(Default, Resource)]
pub(crate) struct SourceObjectEventQueue {
    pub(crate) pending: Vec<SourceObjectEvent>,
}

#[derive(Clone)]
pub(crate) struct EjectDriverCleanupPackets {
    pub(crate) attack_clear: AttackObjectPacket,
    pub(crate) waypoint_clear: SendWaypointsPacket,
    pub(crate) stop_location: Option<ObjectLocationPacket>,
}

#[derive(Clone, Component)]
pub(crate) struct EjectDriverBatchPending {
    pub(crate) base_world_position: Vec2,
    pub(crate) object_team: ObjectTeamPacket,
    pub(crate) cleanup: Option<EjectDriverCleanupPackets>,
}

#[derive(Component)]
pub(crate) struct EjectDriverBatchReady;

#[derive(Clone)]
pub(crate) struct ProducedObjectMemberRoute {
    pub(crate) ref_id: u32,
    pub(crate) waypoints: Vec<MovementWaypoint>,
}

#[derive(Clone, Component)]
pub(crate) struct ProducedObjectBatchPending {
    pub(crate) unit: ObjectKind,
    pub(crate) leader_ref_id: u32,
    pub(crate) member_routes: Vec<ProducedObjectMemberRoute>,
}

#[derive(Component)]
pub(crate) struct ProducedObjectBatchReady;

#[derive(Clone, Component)]
pub(crate) struct RepairedObjectBatchPending {
    pub(crate) unit: ObjectKind,
    pub(crate) member_routes: Vec<ProducedObjectMemberRoute>,
}

#[derive(Component)]
pub(crate) struct RepairedObjectBatchReady;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SourceDeleteGroupMember {
    pub(crate) ref_id: u32,
    pub(crate) leader_ref_id: u32,
    pub(crate) member_index: u16,
    pub(crate) grenade_amount: u8,
}

#[derive(Default, Resource)]
pub(crate) struct ObjectHealthReviveQueue {
    pub(crate) pending: Vec<u32>,
}

#[derive(Default, Resource)]
pub(crate) struct ObjectHealthHitEffectQueue {
    pub(crate) pending: Vec<u32>,
}

#[derive(Default, Resource)]
pub(crate) struct ObjectDestroyPacketQueue {
    pub(crate) pending: Vec<DestroyObjectPacket>,
}

#[derive(Default, Resource)]
pub(crate) struct DriverHitEffectPacketQueue {
    pub(crate) pending: Vec<DriverHitEffectPacket>,
}

#[derive(Default, Resource)]
pub(crate) struct DriverHitEffectQueue {
    pub(crate) pending: Vec<u32>,
}

#[derive(Default, Resource)]
pub(crate) struct EjectVehiclePacketQueue {
    pub(crate) pending: Vec<EjectVehiclePacket>,
}

#[derive(Default, Resource)]
pub(crate) struct VehicleLidPacketQueue {
    pub(crate) pending: Vec<SetLidOpenPacket>,
}

#[derive(Default, Resource)]
pub(crate) struct CraneAnimPacketQueue {
    pub(crate) pending: Vec<CraneAnimPacket>,
}

#[derive(Default, Resource)]
pub(crate) struct RepairBuildingAnimPacketQueue {
    pub(crate) pending: Vec<RepairBuildingAnimPacket>,
}

pub(crate) struct ObjectTeamApply {
    pub(crate) owner: TeamType,
    pub(crate) driver: Option<DriverHealth>,
}

pub(crate) struct AttackObjectApply {
    pub(crate) target_ref_id: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ObjectInitApply {
    pub(crate) ref_id: u32,
    pub(crate) kind: ObjectKind,
    pub(crate) owner: TeamType,
    pub(crate) source_map_position: Vec2,
    pub(crate) building_level: i8,
    pub(crate) extra_links: u16,
}

struct PreparedNewObject {
    entity: Entity,
    apply: ObjectInitApply,
}

#[derive(Clone)]
pub(crate) struct DriverlessObjectCleanupSnapshot {
    pub(crate) ref_id: u32,
    pub(crate) world_position: Vec2,
    pub(crate) map_position: Vec2,
    pub(crate) attack_target_ref_id: Option<u32>,
    pub(crate) movement_path: Option<MovementPath>,
    pub(crate) is_moving: bool,
    pub(crate) has_special_waypoint: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EmptyWaypointObjectSnapshot {
    pub(crate) ref_id: u32,
    pub(crate) map_position: Vec2,
    pub(crate) group_leader_ref_id: Option<u32>,
    pub(crate) destroyed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DriverlessObjectCleanupEvent {
    pub(crate) ref_id: u32,
    pub(crate) base_world_position: Vec2,
    pub(crate) attack_clear_packet: Option<AttackObjectPacket>,
    pub(crate) waypoint_packet: Option<SendWaypointsPacket>,
    pub(crate) stop_location_packet: Option<ObjectLocationPacket>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DriverlessObjectCleanupPlan {
    pub(crate) target: DriverlessObjectCleanupEvent,
    pub(crate) dependents: Vec<DriverlessObjectCleanupEvent>,
}

pub(crate) struct PortraitAnimationApply {
    pub(crate) ref_id: u32,
    pub(crate) kind: Option<PortraitAnimationKind>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RepairBuildingAnimApply {
    pub(crate) state: Option<RepairBuildingAnimState>,
    pub(crate) play_sound: bool,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn relay_new_object(
    queue: &mut SourceObjectEventQueue,
    ref_id: u32,
    kind: ObjectKind,
    owner: TeamType,
    source_map_position: Vec2,
    building_level: i8,
    extra_links: u16,
    health_percent: i32,
    just_left_cannon: bool,
) -> bool {
    let Some((object_type, object_id)) = source_object_init_parts(kind) else {
        return false;
    };
    let Some(x) = source_location_i32(source_map_position.x) else {
        return false;
    };
    let Some(y) = source_location_i32(source_map_position.y) else {
        return false;
    };
    let Ok(ref_id) = i32::try_from(ref_id) else {
        return false;
    };
    let packet = ObjectInitPacket {
        x,
        y,
        ref_id,
        owner: team_wire(owner),
        object_type,
        object_id,
        building_level,
        extra_links,
        health: source_health_from_percent(kind, health_percent),
    };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = ObjectInitPacket::decode_payload(payload) else {
        return false;
    };
    if process_new_object_packet(decoded_packet).is_none() {
        return false;
    }
    queue.pending.push(SourceObjectEvent::AddNewObject {
        packet: decoded_packet,
        just_left_cannon,
    });
    true
}

pub(crate) fn relay_built_cannon_list(
    queue: &mut SourceObjectEventQueue,
    ref_id: u32,
    stored_cannons: &[ObjectKind],
    requires_new_object_ref: Option<u32>,
) -> bool {
    let Ok(ref_id) = i32::try_from(ref_id) else {
        return false;
    };
    let Some(cannon_ids) = stored_cannons
        .iter()
        .map(|kind| match kind {
            ObjectKind::Cannon(cannon) => Some(*cannon as u8),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let packet = BuiltCannonListPacket { ref_id, cannon_ids };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = BuiltCannonListPacket::decode_payload(payload) else {
        return false;
    };
    queue.pending.push(SourceObjectEvent::SetBuiltCannonAmount {
        packet: decoded_packet,
        requires_new_object_ref,
    });
    true
}

pub(crate) fn relay_object_group_info(
    queue: &mut SourceObjectEventQueue,
    ref_id: u32,
    leader_ref_id: Option<u32>,
    minion_refs: &[u32],
    requires_new_object_refs: &[u32],
) -> bool {
    let Ok(ref_id) = i32::try_from(ref_id) else {
        return false;
    };
    let leader_ref_id = match leader_ref_id {
        Some(leader_ref_id) => {
            let Ok(leader_ref_id) = i32::try_from(leader_ref_id) else {
                return false;
            };
            leader_ref_id
        }
        None => -1,
    };
    let Some(minion_refs) = minion_refs
        .iter()
        .map(|ref_id| i32::try_from(*ref_id).ok())
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let packet = ObjectGroupInfoPacket {
        ref_id,
        leader_ref_id,
        minion_refs,
    };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = ObjectGroupInfoPacket::decode_payload(payload) else {
        return false;
    };
    queue.pending.push(SourceObjectEvent::ObjectGroupInfo {
        packet: decoded_packet,
        requires_new_object_refs: requires_new_object_refs.to_vec(),
    });
    true
}

pub(crate) fn relay_object_health(
    queue: &mut SourceObjectEventQueue,
    ref_id: u32,
    health: i32,
    requires_new_object_ref: Option<u32>,
) -> bool {
    let Ok(ref_id) = i32::try_from(ref_id) else {
        return false;
    };
    let packet = ObjectHealthPacket { ref_id, health };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = ObjectHealthPacket::decode_payload(payload) else {
        return false;
    };
    queue.pending.push(SourceObjectEvent::UpdateHealth {
        packet: decoded_packet,
        requires_new_object_ref,
    });
    true
}

pub(crate) fn relay_ejected_driver_group(
    queue: &mut SourceObjectEventQueue,
    first_ref_id: u32,
    robot: RobotType,
    owner: TeamType,
    source_map_position: Vec2,
    driver_healths: &[i32],
    just_left_cannon: bool,
) -> Option<Vec<u32>> {
    let member_refs = ejected_driver_group_refs(first_ref_id, driver_healths.len())?;
    let minion_refs = member_refs.get(1..).unwrap_or_default();
    let checkpoint = queue.pending.len();

    for (member_index, (ref_id, health)) in member_refs
        .iter()
        .copied()
        .zip(driver_healths.iter().copied())
        .enumerate()
    {
        let relayed = relay_new_object(
            queue,
            ref_id,
            ObjectKind::Robot(robot),
            owner,
            source_map_position,
            0,
            0,
            100,
            just_left_cannon,
        ) && if member_index == 0 {
            relay_object_group_info(queue, ref_id, None, minion_refs, &member_refs)
        } else {
            relay_object_group_info(queue, ref_id, Some(first_ref_id), &[], &member_refs)
        } && relay_object_health(queue, ref_id, health, Some(ref_id));
        if !relayed {
            queue.pending.truncate(checkpoint);
            return None;
        }
    }

    Some(member_refs)
}

pub(crate) fn ejected_driver_group_refs(
    first_ref_id: u32,
    driver_count: usize,
) -> Option<Vec<u32>> {
    source_new_object_refs(first_ref_id, driver_count)
}

pub(crate) fn source_new_object_refs(first_ref_id: u32, object_count: usize) -> Option<Vec<u32>> {
    if object_count == 0 {
        return None;
    }
    (0..object_count)
        .map(|index| {
            let ref_id = first_ref_id.checked_add(u32::try_from(index).ok()?)?;
            i32::try_from(ref_id).ok()?;
            Some(ref_id)
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn relay_produced_object_batch(
    queue: &mut SourceObjectEventQueue,
    building_ref_id: u32,
    first_ref_id: u32,
    unit: ObjectKind,
    owner: TeamType,
    source_top_left_map: Vec2,
    initial_move_target_map: Vec2,
    object_count: usize,
) -> Option<Vec<u32>> {
    let member_refs = source_new_object_refs(first_ref_id, object_count)?;
    let robot = match unit {
        ObjectKind::Robot(robot) => Some(robot),
        ObjectKind::Vehicle(_) if object_count == 1 => None,
        _ => return None,
    };
    let minion_refs = member_refs.get(1..).unwrap_or_default();
    let checkpoint = queue.pending.len();

    for (member_index, ref_id) in member_refs.iter().copied().enumerate() {
        let relayed = relay_new_object(
            queue,
            ref_id,
            unit,
            owner,
            source_top_left_map,
            0,
            0,
            100,
            false,
        ) && robot.is_none_or(|_| {
            if member_index == 0 {
                relay_object_group_info(queue, ref_id, None, minion_refs, &member_refs)
            } else {
                relay_object_group_info(queue, ref_id, Some(first_ref_id), &[], &member_refs)
            }
        });
        if !relayed {
            queue.pending.truncate(checkpoint);
            return None;
        }
    }

    let Some(waypoint_x) = source_location_i32(initial_move_target_map.x) else {
        queue.pending.truncate(checkpoint);
        return None;
    };
    let Some(waypoint_y) = source_location_i32(initial_move_target_map.y) else {
        queue.pending.truncate(checkpoint);
        return None;
    };
    let waypoint = SourceWaypoint {
        mode: SourceWaypointMode::ForceMove,
        ref_id: -1,
        x: waypoint_x,
        y: waypoint_y,
        attack_to: false,
        player_given: false,
    };
    let Some(packet) = relay_object_waypoints(first_ref_id, &[waypoint]) else {
        queue.pending.truncate(checkpoint);
        return None;
    };
    queue.pending.push(SourceObjectEvent::ObjectWaypoints {
        packet,
        requires_new_object_ref: Some(first_ref_id),
        target_team: None,
    });
    queue
        .pending
        .push(SourceObjectEvent::CommitProducedObjectBatch {
            building_ref_id,
            requires_new_object_refs: member_refs.clone(),
        });

    Some(member_refs)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn relay_repaired_object_batch(
    queue: &mut SourceObjectEventQueue,
    building_ref_id: u32,
    first_ref_id: u32,
    unit: ObjectKind,
    owner: TeamType,
    source_top_left_map: Vec2,
    initial_move_target_map: Vec2,
    object_count: usize,
    driver: Option<&DriverHealth>,
    resume_waypoints: &[MovementWaypoint],
) -> Option<Vec<u32>> {
    let member_refs = source_new_object_refs(first_ref_id, object_count)?;
    let robot = match unit {
        ObjectKind::Robot(robot) => Some(robot),
        ObjectKind::Vehicle(_) if object_count == 1 => None,
        _ => return None,
    };
    let minion_refs = member_refs.get(1..).unwrap_or_default();
    let checkpoint = queue.pending.len();

    for (member_index, ref_id) in member_refs.iter().copied().enumerate() {
        let relayed = relay_new_object(
            queue,
            ref_id,
            unit,
            owner,
            source_top_left_map,
            0,
            0,
            100,
            false,
        ) && robot.is_none_or(|_| {
            if member_index == 0 {
                relay_object_group_info(queue, ref_id, None, minion_refs, &member_refs)
            } else {
                relay_object_group_info(queue, ref_id, Some(first_ref_id), &[], &member_refs)
            }
        });
        if !relayed {
            queue.pending.truncate(checkpoint);
            return None;
        }
    }

    let Some(waypoint_x) = source_location_i32(initial_move_target_map.x) else {
        queue.pending.truncate(checkpoint);
        return None;
    };
    let Some(waypoint_y) = source_location_i32(initial_move_target_map.y) else {
        queue.pending.truncate(checkpoint);
        return None;
    };
    let mut waypoints = Vec::with_capacity(1 + resume_waypoints.len());
    waypoints.push(SourceWaypoint {
        mode: SourceWaypointMode::ForceMove,
        ref_id: -1,
        x: waypoint_x,
        y: waypoint_y,
        attack_to: false,
        player_given: false,
    });
    for waypoint in resume_waypoints {
        let Some(x) = source_location_i32(waypoint.position.x) else {
            queue.pending.truncate(checkpoint);
            return None;
        };
        let Some(y) = source_location_i32(-waypoint.position.y) else {
            queue.pending.truncate(checkpoint);
            return None;
        };
        waypoints.push(SourceWaypoint {
            mode: source_waypoint_mode_for_movement(waypoint.mode),
            ref_id: waypoint
                .ref_id
                .and_then(|ref_id| i32::try_from(ref_id).ok())
                .unwrap_or(-1),
            x,
            y,
            attack_to: waypoint.attack_to,
            player_given: waypoint.player_given,
        });
    }
    let Some(packet) = relay_object_waypoints(first_ref_id, &waypoints) else {
        queue.pending.truncate(checkpoint);
        return None;
    };
    queue.pending.push(SourceObjectEvent::ObjectWaypoints {
        packet,
        requires_new_object_ref: Some(first_ref_id),
        target_team: Some(owner),
    });
    let Some(packet) = relay_object_team_update(first_ref_id, owner, driver) else {
        queue.pending.truncate(checkpoint);
        return None;
    };
    queue.pending.push(SourceObjectEvent::ObjectTeam {
        packet,
        requires_new_object_ref: Some(first_ref_id),
    });
    queue
        .pending
        .push(SourceObjectEvent::CommitRepairedObjectBatch {
            building_ref_id,
            requires_new_object_refs: member_refs.clone(),
        });

    Some(member_refs)
}

pub(crate) fn relay_deferred_delete_object(
    queue: &mut SourceObjectEventQueue,
    ref_id: u32,
    group_members: &[SourceDeleteGroupMember],
) -> bool {
    let mut deferred = SourceObjectEventQueue::default();
    if let Some(removed) = group_members.iter().find(|member| member.ref_id == ref_id) {
        let mut group: Vec<_> = group_members
            .iter()
            .copied()
            .filter(|member| {
                member.ref_id == removed.leader_ref_id
                    || member.leader_ref_id == removed.leader_ref_id
            })
            .collect();
        group.sort_by_key(|member| member.member_index);
        if group.len() > 1 {
            if removed.leader_ref_id == removed.ref_id {
                let remaining: Vec<_> = group
                    .iter()
                    .copied()
                    .filter(|member| member.ref_id != ref_id)
                    .collect();
                if let Some(new_leader) = remaining.first().copied() {
                    let minion_refs: Vec<_> = remaining
                        .iter()
                        .skip(1)
                        .map(|member| member.ref_id)
                        .collect();
                    if !relay_object_group_info(
                        &mut deferred,
                        new_leader.ref_id,
                        None,
                        &minion_refs,
                        &[],
                    ) {
                        return false;
                    }
                    for minion in remaining.iter().skip(1) {
                        if !relay_object_group_info(
                            &mut deferred,
                            minion.ref_id,
                            Some(new_leader.ref_id),
                            &[],
                            &[],
                        ) {
                            return false;
                        }
                    }
                    if removed.grenade_amount > 0 {
                        let Some(packet) =
                            relay_object_grenade_amount(new_leader.ref_id, removed.grenade_amount)
                        else {
                            return false;
                        };
                        deferred
                            .pending
                            .push(SourceObjectEvent::ObjectGrenadeAmount { packet });
                    }
                }
            } else {
                let minion_refs: Vec<_> = group
                    .iter()
                    .filter(|member| {
                        member.ref_id != removed.leader_ref_id && member.ref_id != ref_id
                    })
                    .map(|member| member.ref_id)
                    .collect();
                if !relay_object_group_info(
                    &mut deferred,
                    removed.leader_ref_id,
                    None,
                    &minion_refs,
                    &[],
                ) {
                    return false;
                }
            }
            if !relay_object_group_info(&mut deferred, ref_id, None, &[], &[]) {
                return false;
            }
        }
    }

    let Ok(ref_id) = i32::try_from(ref_id) else {
        return false;
    };
    let packet = DeleteObjectPacket { ref_id };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(packet) = DeleteObjectPacket::decode_payload(payload) else {
        return false;
    };
    deferred
        .pending
        .push(SourceObjectEvent::DeleteObject { packet });
    queue
        .pending
        .push(SourceObjectEvent::DeferredUntilEarlyDrain {
            events: deferred.pending,
        });
    true
}

pub(crate) fn ejected_driver_refs_available(
    member_refs: &[u32],
    reserved_ref_ids: &HashSet<u32>,
) -> bool {
    source_new_object_refs_available(member_refs, reserved_ref_ids)
}

pub(crate) fn source_new_object_refs_available(
    member_refs: &[u32],
    reserved_ref_ids: &HashSet<u32>,
) -> bool {
    member_refs
        .iter()
        .all(|ref_id| !reserved_ref_ids.contains(ref_id))
}

fn source_new_object_batch_ready(
    required_new_object_refs: &[u32],
    spawned_ref_ids: &HashSet<u32>,
) -> bool {
    required_new_object_refs
        .iter()
        .all(|ref_id| spawned_ref_ids.contains(ref_id))
}

#[derive(SystemParam)]
pub(crate) struct SourceObjectAuxState<'w, 's> {
    settings: Res<'w, SourceSettingsState>,
    selection: ResMut<'w, SelectionState>,
    layers: Query<'w, 's, (Entity, &'static ObjectLayerRef)>,
    attack_targets: Query<'w, 's, (Entity, &'static AttackTarget)>,
    minimap_dots: Query<'w, 's, (Entity, &'static MinimapDot)>,
    selection_markers: Query<'w, 's, (Entity, &'static SelectionMarker)>,
    selection_health_bars: Query<'w, 's, (Entity, &'static SelectionHealthBar)>,
    grenade_inventories: Query<'w, 's, &'static mut GrenadeInventory>,
    health_percents: Query<'w, 's, &'static mut HealthPercent>,
    rally_points: Query<'w, 's, &'static mut BuildingRallyPoints>,
    robot_groups: Query<'w, 's, (&'static GameObjectEntity, &'static RobotGroup)>,
}

pub(crate) fn relay_ejected_driver_batch_commit(
    queue: &mut SourceObjectEventQueue,
    carrier_ref_id: u32,
    requires_new_object_refs: &[u32],
) {
    queue
        .pending
        .push(SourceObjectEvent::CommitEjectedDriverBatch {
            carrier_ref_id,
            requires_new_object_refs: requires_new_object_refs.to_vec(),
        });
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum SourceObjectDrainPhase {
    Early,
    Late,
}

fn take_source_events_for_drain(
    queue: &mut SourceObjectEventQueue,
    drain_phase: SourceObjectDrainPhase,
) -> Vec<SourceObjectEvent> {
    let mut events = Vec::new();
    for event in std::mem::take(&mut queue.pending) {
        match event {
            SourceObjectEvent::DeferredUntilEarlyDrain { events: deferred }
                if drain_phase == SourceObjectDrainPhase::Early =>
            {
                events.extend(deferred);
            }
            event @ SourceObjectEvent::DeferredUntilEarlyDrain { .. } => {
                queue.pending.push(event);
            }
            event => events.push(event),
        }
    }
    events
}

pub(crate) fn process_source_object_event_queue(
    commands: Commands,
    queue: ResMut<SourceObjectEventQueue>,
    game_atlases: Res<GameAtlases>,
    asset_server: Res<AssetServer>,
    map: Res<CurrentMap>,
    hud_layout: Res<HudLayout>,
    local_player: Res<LocalPlayerState>,
    next_ref: ResMut<NextObjectRefId>,
    rng: ResMut<CombatRng>,
    revive_queue: ResMut<ObjectHealthReviveQueue>,
    hit_effect_queue: ResMut<ObjectHealthHitEffectQueue>,
    repair_anim_packets: ResMut<RepairBuildingAnimPacketQueue>,
    objects: Query<(Entity, &GameObjectEntity, Option<&DestroyedObject>)>,
    object_stats: Query<&mut ObjectStats>,
    productions: Query<(&GameObjectEntity, &mut BuildingProduction)>,
    aux: SourceObjectAuxState,
) {
    process_source_object_event_queue_for_phase(
        commands,
        queue,
        game_atlases,
        asset_server,
        map,
        hud_layout,
        local_player,
        next_ref,
        rng,
        revive_queue,
        hit_effect_queue,
        repair_anim_packets,
        objects,
        object_stats,
        productions,
        aux,
        SourceObjectDrainPhase::Early,
    );
}

#[allow(clippy::too_many_arguments)]
fn process_source_object_event_queue_for_phase(
    mut commands: Commands,
    mut queue: ResMut<SourceObjectEventQueue>,
    game_atlases: Res<GameAtlases>,
    asset_server: Res<AssetServer>,
    map: Res<CurrentMap>,
    hud_layout: Res<HudLayout>,
    local_player: Res<LocalPlayerState>,
    mut next_ref: ResMut<NextObjectRefId>,
    mut rng: ResMut<CombatRng>,
    mut revive_queue: ResMut<ObjectHealthReviveQueue>,
    mut hit_effect_queue: ResMut<ObjectHealthHitEffectQueue>,
    mut repair_anim_packets: ResMut<RepairBuildingAnimPacketQueue>,
    objects: Query<(Entity, &GameObjectEntity, Option<&DestroyedObject>)>,
    mut object_stats: Query<&mut ObjectStats>,
    mut productions: Query<(&GameObjectEntity, &mut BuildingProduction)>,
    mut aux: SourceObjectAuxState,
    drain_phase: SourceObjectDrainPhase,
) {
    let events = take_source_events_for_drain(&mut queue, drain_phase);
    let mut object_entities: HashMap<u32, Entity> = objects
        .iter()
        .map(|(entity, object, _)| (object.ref_id, entity))
        .collect();
    let mut object_kinds: HashMap<u32, ObjectKind> = objects
        .iter()
        .map(|(_, object, _)| (object.ref_id, object.kind))
        .collect();
    let destroyed_ref_ids: HashSet<u32> = objects
        .iter()
        .filter_map(|(_, object, destroyed)| destroyed.is_some().then_some(object.ref_id))
        .collect();
    let mut known_ref_ids: HashSet<u32> = object_entities.keys().copied().collect();
    let mut prepared_new_objects = HashMap::new();

    for event in &events {
        let SourceObjectEvent::AddNewObject { packet, .. } = event else {
            continue;
        };
        let Some(apply) = accept_new_object_packet(*packet, &mut known_ref_ids) else {
            continue;
        };
        let entity = commands.spawn_empty().id();
        object_entities.insert(apply.ref_id, entity);
        object_kinds.insert(apply.ref_id, apply.kind);
        prepared_new_objects.insert(apply.ref_id, PreparedNewObject { entity, apply });
    }

    let mut accepted_new_ref_ids: HashSet<u32> = prepared_new_objects.keys().copied().collect();
    let mut spawned_ref_ids = HashSet::new();
    let mut spawned_visual_layers: HashMap<u32, Vec<RuntimeSpawnLayer>> = HashMap::new();
    let mut robot_groups: HashMap<u32, RobotGroup> = aux
        .robot_groups
        .iter()
        .map(|(object, group)| (object.ref_id, *group))
        .collect();
    let mut pending_building_productions = HashMap::new();

    for event in events {
        match event {
            SourceObjectEvent::DeferredUntilEarlyDrain { .. } => unreachable!(),
            SourceObjectEvent::AddNewObject {
                packet,
                just_left_cannon,
            } => {
                let Ok(ref_id) = u32::try_from(packet.ref_id) else {
                    continue;
                };
                let Some(prepared) = prepared_new_objects.remove(&ref_id) else {
                    continue;
                };
                let apply = prepared.apply;

                let tile_x = packet.x.div_euclid(TILE_SIZE as i32);
                let tile_y = packet.y.div_euclid(TILE_SIZE as i32);
                let cannon_ejectable = units::cannon_ejectable_on_spawn(
                    apply.kind,
                    area_is_fort_turret_tile(&map.0, tile_x, tile_y),
                );
                let initial_rotation = if matches!(apply.kind, ObjectKind::Vehicle(_)) {
                    crate::rotation_for_direction(rng.index(8))
                } else {
                    180
                };
                let Some(spawned) = spawn_runtime_object_from_source_init(
                    &mut commands,
                    &game_atlases,
                    &asset_server,
                    map.0.basics.terrain_type,
                    &hud_layout,
                    &aux.settings,
                    apply.ref_id,
                    apply.kind,
                    apply.owner,
                    apply.source_map_position,
                    apply.building_level,
                    apply.extra_links,
                    cannon_ejectable,
                    initial_rotation,
                    just_left_cannon,
                    matches!(apply.kind, ObjectKind::Cannon(_))
                        && apply.owner == local_player.team(),
                    prepared.entity,
                ) else {
                    commands.entity(prepared.entity).despawn();
                    known_ref_ids.remove(&apply.ref_id);
                    accepted_new_ref_ids.remove(&apply.ref_id);
                    object_entities.remove(&apply.ref_id);
                    object_kinds.remove(&apply.ref_id);
                    continue;
                };
                if spawned.gameplay_entity != prepared.entity {
                    commands.entity(spawned.gameplay_entity).despawn();
                    commands.entity(prepared.entity).despawn();
                    known_ref_ids.remove(&apply.ref_id);
                    accepted_new_ref_ids.remove(&apply.ref_id);
                    object_entities.remove(&apply.ref_id);
                    object_kinds.remove(&apply.ref_id);
                    continue;
                }

                spawned_ref_ids.insert(apply.ref_id);
                spawned_visual_layers.insert(apply.ref_id, spawned.visual_layers);
                next_ref.0 = next_ref.0.max(apply.ref_id.saturating_add(1));
            }
            SourceObjectEvent::ObjectGroupInfo {
                packet,
                requires_new_object_refs,
            } => {
                if requires_new_object_refs
                    .iter()
                    .any(|ref_id| !accepted_new_ref_ids.contains(ref_id))
                {
                    continue;
                }
                let Some(assignments) = runtime_group_assignments(&packet, &object_kinds) else {
                    continue;
                };
                for (ref_id, leader_ref_id, member_index) in assignments {
                    let Some(entity) = object_entities.get(&ref_id).copied() else {
                        continue;
                    };
                    let member_index = member_index
                        .or_else(|| robot_groups.get(&ref_id).map(|group| group.member_index))
                        .unwrap_or(0);
                    let group = RobotGroup {
                        leader_ref_id,
                        member_index,
                    };
                    robot_groups.insert(ref_id, group);
                    commands.entity(entity).insert(group);
                }
            }
            SourceObjectEvent::UpdateHealth {
                packet,
                requires_new_object_ref,
            } => {
                if requires_new_object_ref.is_some_and(|ref_id| !spawned_ref_ids.contains(&ref_id))
                {
                    continue;
                }
                let Ok(ref_id) = u32::try_from(packet.ref_id) else {
                    continue;
                };
                let Some(entity) = object_entities.get(&ref_id).copied() else {
                    continue;
                };
                if spawned_ref_ids.contains(&ref_id) {
                    let Some(kind) = object_kinds.get(&ref_id).copied() else {
                        continue;
                    };
                    let mut stats = aux.settings.object_stats(kind, 100);
                    if !apply_object_health_packet(packet, ref_id, &mut stats) {
                        continue;
                    }
                    commands.entity(entity).insert((
                        stats,
                        HealthPercent(health_percent_from_source_health(
                            stats.max_health,
                            packet.health,
                        )),
                    ));
                    hit_effect_queue.pending.push(ref_id);
                } else if let Ok(mut stats) = object_stats.get_mut(entity) {
                    let was_destroyed = destroyed_ref_ids.contains(&ref_id) || stats.destroyed();
                    if !apply_object_health_packet(packet, ref_id, &mut stats) {
                        continue;
                    }
                    if let Ok(mut health_percent) = aux.health_percents.get_mut(entity) {
                        health_percent.0 =
                            health_percent_from_source_health(stats.max_health, packet.health);
                    }
                    hit_effect_queue.pending.push(ref_id);
                    if was_destroyed && !stats.destroyed() {
                        revive_queue.pending.push(ref_id);
                    }
                }
            }
            SourceObjectEvent::BuildingState {
                packet,
                requires_new_object_ref,
            } => {
                if requires_new_object_ref.is_some_and(|ref_id| !spawned_ref_ids.contains(&ref_id))
                {
                    continue;
                }
                let Ok(ref_id) = u32::try_from(packet.ref_id) else {
                    continue;
                };
                let Some(kind) = object_kinds.get(&ref_id).copied() else {
                    continue;
                };
                let existing = pending_building_productions.remove(&ref_id).or_else(|| {
                    productions.iter_mut().find_map(|(object, production)| {
                        (object.ref_id == ref_id).then(|| production.clone())
                    })
                });
                if let Some(production) =
                    apply_building_state_packet(packet, ref_id, kind, existing)
                {
                    pending_building_productions.insert(ref_id, production);
                }
            }
            SourceObjectEvent::RepairBuildingAnimation {
                packet,
                requires_new_object_ref,
            } => {
                if requires_new_object_ref.is_none_or(|ref_id| spawned_ref_ids.contains(&ref_id)) {
                    repair_anim_packets.pending.push(packet);
                }
            }
            SourceObjectEvent::BuildingQueue {
                packet,
                requires_new_object_ref,
            } => {
                if requires_new_object_ref.is_some_and(|ref_id| !spawned_ref_ids.contains(&ref_id))
                {
                    continue;
                }
                let Ok(ref_id) = u32::try_from(packet.ref_id) else {
                    continue;
                };
                let mut production = pending_building_productions.remove(&ref_id).or_else(|| {
                    productions.iter_mut().find_map(|(object, production)| {
                        (object.ref_id == ref_id).then(|| production.clone())
                    })
                });
                if let Some(production) = production.as_mut()
                    && apply_building_queue_packet(&packet, ref_id, production)
                {
                    pending_building_productions.insert(ref_id, production.clone());
                }
            }
            SourceObjectEvent::ObjectWaypoints {
                packet,
                requires_new_object_ref,
                target_team,
            } => {
                if target_team.is_some_and(|team| team != local_player.team()) {
                    continue;
                }
                if requires_new_object_ref.is_some_and(|ref_id| !spawned_ref_ids.contains(&ref_id))
                {
                    continue;
                }
                let Ok(ref_id) = u32::try_from(packet.ref_id) else {
                    continue;
                };
                let Some(entity) = object_entities.get(&ref_id).copied() else {
                    continue;
                };
                let Some(kind) = object_kinds.get(&ref_id).copied() else {
                    continue;
                };
                let Some(waypoints) = movement_waypoints_from_source_map_packet(&packet, ref_id)
                else {
                    continue;
                };
                let move_speed = aux.settings.object_stats(kind, 100).move_speed;
                if waypoints.is_empty() || move_speed <= 0.0 {
                    commands.entity(entity).remove::<MovementPath>();
                } else {
                    commands
                        .entity(entity)
                        .insert(MovementPath::from_typed(waypoints.clone(), move_speed));
                }
                for layer in spawned_visual_layers.get(&ref_id).into_iter().flatten() {
                    if waypoints.is_empty() || move_speed <= 0.0 {
                        commands.entity(layer.entity).remove::<MovementPath>();
                    } else {
                        commands
                            .entity(layer.entity)
                            .insert(MovementPath::from_typed(
                                waypoints
                                    .iter()
                                    .map(|waypoint| {
                                        waypoint
                                            .with_position(waypoint.position + layer.world_offset)
                                    })
                                    .collect(),
                                move_speed,
                            ));
                    }
                }
            }
            SourceObjectEvent::ObjectTeam {
                packet,
                requires_new_object_ref,
            } => {
                if requires_new_object_ref.is_some_and(|ref_id| !spawned_ref_ids.contains(&ref_id))
                {
                    continue;
                }
                let Ok(ref_id) = u32::try_from(packet.ref_id) else {
                    continue;
                };
                let Some(entity) = object_entities.get(&ref_id).copied() else {
                    continue;
                };
                let Some(apply) = apply_object_team_packet(&packet, ref_id) else {
                    continue;
                };
                commands.entity(entity).insert(ObjectTeam(apply.owner));
                if let Some(driver) = apply.driver {
                    commands.entity(entity).insert(driver);
                } else {
                    commands.entity(entity).remove::<DriverHealth>();
                }
            }
            SourceObjectEvent::ObjectGrenadeAmount { packet } => {
                let Ok(ref_id) = u32::try_from(packet.ref_id) else {
                    continue;
                };
                let Some(entity) = object_entities.get(&ref_id).copied() else {
                    continue;
                };
                if spawned_ref_ids.contains(&ref_id) {
                    let amount = source_grenade_amount(packet.grenade_amount);
                    commands.entity(entity).insert(GrenadeInventory { amount });
                    continue;
                }
                let Ok(mut inventory) = aux.grenade_inventories.get_mut(entity) else {
                    continue;
                };
                apply_object_grenade_amount_packet(&packet, ref_id, &mut inventory);
            }
            SourceObjectEvent::ObjectRallyPoints {
                packet,
                requires_new_object_ref,
            } => {
                if requires_new_object_ref.is_some_and(|ref_id| !spawned_ref_ids.contains(&ref_id))
                {
                    continue;
                }
                let Ok(ref_id) = u32::try_from(packet.ref_id) else {
                    continue;
                };
                let Some(kind) = object_kinds.get(&ref_id).copied() else {
                    continue;
                };
                let Some(rally_points) = rally_points_from_packet(&packet, ref_id, kind) else {
                    continue;
                };
                let Some(entity) = object_entities.get(&ref_id).copied() else {
                    continue;
                };
                if spawned_ref_ids.contains(&ref_id) {
                    commands.entity(entity).insert(rally_points);
                } else if let Ok(mut current) = aux.rally_points.get_mut(entity) {
                    *current = rally_points;
                } else {
                    commands.entity(entity).insert(rally_points);
                }
            }
            SourceObjectEvent::DeleteObject { packet } => {
                let Ok(ref_id) = u32::try_from(packet.ref_id) else {
                    continue;
                };
                let Some(root_entity) = object_entities.remove(&ref_id) else {
                    continue;
                };
                let mut despawned_root = false;
                for (entity, layer_ref) in &aux.layers {
                    if layer_ref.0 == ref_id {
                        despawned_root |= entity == root_entity;
                        commands.entity(entity).despawn();
                    }
                }
                if !despawned_root {
                    commands.entity(root_entity).despawn();
                }
                for (entity, target) in &aux.attack_targets {
                    if target.ref_id == ref_id {
                        commands
                            .entity(entity)
                            .remove::<AttackTargetLifecycleComponents>();
                    }
                }
                for (entity, dot) in &aux.minimap_dots {
                    if dot.ref_id == ref_id {
                        commands.entity(entity).despawn();
                    }
                }
                for (entity, marker) in &aux.selection_markers {
                    if marker.ref_id == ref_id {
                        commands.entity(entity).despawn();
                    }
                }
                for (entity, bar) in &aux.selection_health_bars {
                    if bar.ref_id == ref_id {
                        commands.entity(entity).despawn();
                    }
                }
                aux.selection
                    .selected_refs
                    .retain(|selected_ref| *selected_ref != ref_id);
                object_kinds.remove(&ref_id);
                known_ref_ids.remove(&ref_id);
                accepted_new_ref_ids.remove(&ref_id);
                spawned_ref_ids.remove(&ref_id);
                spawned_visual_layers.remove(&ref_id);
                if let Some(prepared) = prepared_new_objects.remove(&ref_id) {
                    commands.entity(prepared.entity).despawn();
                }
            }
            SourceObjectEvent::CommitEjectedDriverBatch {
                carrier_ref_id,
                requires_new_object_refs,
            } => {
                let Some(entity) = object_entities.get(&carrier_ref_id).copied() else {
                    continue;
                };
                if source_new_object_batch_ready(&requires_new_object_refs, &spawned_ref_ids) {
                    commands.entity(entity).insert(EjectDriverBatchReady);
                } else {
                    commands.entity(entity).remove::<EjectDriverBatchPending>();
                }
            }
            SourceObjectEvent::CommitProducedObjectBatch {
                building_ref_id,
                requires_new_object_refs,
            } => {
                let Some(entity) = object_entities.get(&building_ref_id).copied() else {
                    continue;
                };
                if source_new_object_batch_ready(&requires_new_object_refs, &spawned_ref_ids) {
                    commands.entity(entity).insert(ProducedObjectBatchReady);
                } else {
                    commands
                        .entity(entity)
                        .remove::<ProducedObjectBatchPending>();
                }
            }
            SourceObjectEvent::CommitRepairedObjectBatch {
                building_ref_id,
                requires_new_object_refs,
            } => {
                let Some(entity) = object_entities.get(&building_ref_id).copied() else {
                    continue;
                };
                if source_new_object_batch_ready(&requires_new_object_refs, &spawned_ref_ids) {
                    commands.entity(entity).insert(RepairedObjectBatchReady);
                } else {
                    commands
                        .entity(entity)
                        .remove::<RepairedObjectBatchPending>();
                }
            }
            SourceObjectEvent::SetBuiltCannonAmount {
                packet,
                requires_new_object_ref,
            } => {
                if requires_new_object_ref.is_some_and(|ref_id| !spawned_ref_ids.contains(&ref_id))
                {
                    continue;
                }
                for (object, mut production) in &mut productions {
                    if apply_built_cannon_list_packet(&packet, object.ref_id, &mut production) {
                        break;
                    }
                }
            }
        }
    }
    for (ref_id, production) in pending_building_productions {
        if let Some(entity) = object_entities.get(&ref_id).copied() {
            commands.entity(entity).insert(production);
        }
    }
}

pub(crate) fn process_late_source_object_event_queue(
    commands: Commands,
    queue: ResMut<SourceObjectEventQueue>,
    game_atlases: Res<GameAtlases>,
    asset_server: Res<AssetServer>,
    map: Res<CurrentMap>,
    hud_layout: Res<HudLayout>,
    local_player: Res<LocalPlayerState>,
    next_ref: ResMut<NextObjectRefId>,
    rng: ResMut<CombatRng>,
    revive_queue: ResMut<ObjectHealthReviveQueue>,
    hit_effect_queue: ResMut<ObjectHealthHitEffectQueue>,
    repair_anim_packets: ResMut<RepairBuildingAnimPacketQueue>,
    objects: Query<(Entity, &GameObjectEntity, Option<&DestroyedObject>)>,
    object_stats: Query<&mut ObjectStats>,
    productions: Query<(&GameObjectEntity, &mut BuildingProduction)>,
    aux: SourceObjectAuxState,
) {
    process_source_object_event_queue_for_phase(
        commands,
        queue,
        game_atlases,
        asset_server,
        map,
        hud_layout,
        local_player,
        next_ref,
        rng,
        revive_queue,
        hit_effect_queue,
        repair_anim_packets,
        objects,
        object_stats,
        productions,
        aux,
        SourceObjectDrainPhase::Late,
    );
}

fn accept_new_object_packet(
    packet: ObjectInitPacket,
    known_ref_ids: &mut HashSet<u32>,
) -> Option<ObjectInitApply> {
    let apply = apply_new_object_packet(packet)?;
    known_ref_ids.insert(apply.ref_id).then_some(apply)
}

fn runtime_group_assignments(
    packet: &ObjectGroupInfoPacket,
    object_kinds: &HashMap<u32, ObjectKind>,
) -> Option<Vec<(u32, u32, Option<u16>)>> {
    let ref_id = u32::try_from(packet.ref_id).ok()?;
    if !matches!(object_kinds.get(&ref_id), Some(ObjectKind::Robot(_))) {
        return None;
    }
    let leader_ref_id = u32::try_from(packet.leader_ref_id)
        .ok()
        .filter(|leader_ref_id| {
            matches!(object_kinds.get(leader_ref_id), Some(ObjectKind::Robot(_)))
        })
        .unwrap_or(ref_id);
    let is_leader_packet = leader_ref_id == ref_id;
    let mut assignments = vec![(ref_id, leader_ref_id, is_leader_packet.then_some(0))];
    assignments.extend(
        packet
            .minion_refs
            .iter()
            .enumerate()
            .filter_map(|(index, minion_ref)| {
                let minion_ref = u32::try_from(*minion_ref).ok()?;
                matches!(object_kinds.get(&minion_ref), Some(ObjectKind::Robot(_))).then_some((
                    minion_ref,
                    ref_id,
                    u16::try_from(index + 1).ok(),
                ))
            }),
    );
    Some(assignments)
}

fn movement_waypoints_from_source_map_packet(
    packet: &SendWaypointsPacket,
    object_ref_id: u32,
) -> Option<Vec<MovementWaypoint>> {
    apply_object_waypoints_packet(packet, object_ref_id)?
        .iter()
        .copied()
        .map(|waypoint| {
            let mut waypoint = movement_waypoint_from_source(waypoint)?;
            waypoint.position.y = -waypoint.position.y;
            Some(waypoint)
        })
        .collect()
}

fn apply_built_cannon_list_packet(
    packet: &BuiltCannonListPacket,
    object_ref_id: u32,
    production: &mut BuildingProduction,
) -> bool {
    let Ok(packet_ref_id) = u32::try_from(packet.ref_id) else {
        return false;
    };
    if packet_ref_id != object_ref_id {
        return false;
    }
    let Some(stored_cannons) = packet
        .cannon_ids
        .iter()
        .map(|cannon_id| {
            CannonType::try_from(*cannon_id)
                .ok()
                .map(ObjectKind::Cannon)
        })
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    buildings::production_logic::apply_stored_cannon_list(production, stored_cannons);
    true
}

fn apply_building_state_packet(
    packet: BuildingStatePacket,
    object_ref_id: u32,
    object_kind: ObjectKind,
    production: Option<BuildingProduction>,
) -> Option<BuildingProduction> {
    if u32::try_from(packet.ref_id).ok()? != object_ref_id
        || !buildings::can_set_rallypoints(object_kind)
        || !packet.init_offset.is_finite()
        || !packet.production_time.is_finite()
    {
        return None;
    }
    let status = match packet.state {
        0 => BuildingProductionStatus::Place,
        1 => BuildingProductionStatus::Select,
        2 => BuildingProductionStatus::Building,
        3 => BuildingProductionStatus::Paused,
        _ => return None,
    };
    let current = if packet.object_type == u8::MAX && packet.object_id == u8::MAX {
        None
    } else {
        source_object_kind_from_init_parts(packet.object_type, packet.object_id)
    };
    if matches!(
        status,
        BuildingProductionStatus::Building | BuildingProductionStatus::Paused
    ) && current.is_none()
    {
        return None;
    }
    let mut production = production.unwrap_or(BuildingProduction {
        status: BuildingProductionStatus::Select,
        current: None,
        queue: VecDeque::new(),
        elapsed: 0.0,
        duration: 0.0,
        zone_ownage: 0.0,
        unit_limit_reached: false,
        stored_cannons: Vec::new(),
    });
    production.status = status;
    production.current = current;
    production.duration = packet.production_time.max(0.0) as f32;
    production.elapsed = (-packet.init_offset).max(0.0) as f32;
    if production.duration > 0.0 {
        production.elapsed = production.elapsed.min(production.duration);
    }
    production.unit_limit_reached = status == BuildingProductionStatus::Paused;
    Some(production)
}

fn apply_building_queue_packet(
    packet: &BuildingQueuePacket,
    object_ref_id: u32,
    production: &mut BuildingProduction,
) -> bool {
    if u32::try_from(packet.ref_id).ok() != Some(object_ref_id) {
        return false;
    }
    let Some(queue) = packet
        .units
        .iter()
        .map(|unit| source_object_kind_from_init_parts(unit.object_type, unit.object_id))
        .collect::<Option<VecDeque<_>>>()
    else {
        return false;
    };
    production.queue = queue;
    true
}

fn rally_points_from_packet(
    packet: &SendRallypointsPacket,
    object_ref_id: u32,
    object_kind: ObjectKind,
) -> Option<BuildingRallyPoints> {
    if u32::try_from(packet.ref_id).ok()? != object_ref_id
        || !buildings::can_set_rallypoints(object_kind)
    {
        return None;
    }
    let points = packet
        .waypoints
        .iter()
        .filter(|waypoint| waypoint.mode == SourceWaypointMode::Move)
        .map(|waypoint| Vec2::new(waypoint.x as f32, -(waypoint.y as f32)))
        .collect();
    Some(BuildingRallyPoints { points })
}

fn health_percent_from_source_health(max_health: f32, health: i32) -> i32 {
    let max_health = max_health.max(1.0);
    ((health.max(0) as f32 * 100.0 / max_health).floor() as i32).clamp(0, 100)
}

pub(crate) fn process_object_health_packet_queue(
    mut queue: ResMut<ObjectHealthPacketQueue>,
    mut revive_queue: ResMut<ObjectHealthReviveQueue>,
    mut hit_effect_queue: ResMut<ObjectHealthHitEffectQueue>,
    mut objects: Query<(
        &GameObjectEntity,
        Option<&DestroyedObject>,
        &mut ObjectStats,
    )>,
) {
    let packets = std::mem::take(&mut queue.pending);
    for packet in packets {
        for (object, destroyed_marker, mut stats) in &mut objects {
            let was_destroyed = destroyed_marker.is_some() || stats.destroyed();
            if apply_object_health_packet(packet, object.ref_id, &mut stats) {
                hit_effect_queue.pending.push(object.ref_id);
                if was_destroyed && !stats.destroyed() {
                    revive_queue.pending.push(object.ref_id);
                }
                break;
            }
        }
    }
}

pub(crate) fn process_object_destroy_packet_queue(
    mut queue: ResMut<ObjectDestroyPacketQueue>,
    mut objects: Query<(
        &GameObjectEntity,
        &mut ObjectStats,
        Option<&mut DamageCauseTimers>,
    )>,
) {
    let packets = std::mem::take(&mut queue.pending);
    for packet in packets {
        for (object, mut stats, cause) in &mut objects {
            if apply_destroy_object_packet(&packet, object.ref_id, &mut stats, cause) {
                break;
            }
        }
    }
}

pub(crate) fn process_driver_hit_effect_packet_queue(
    mut packet_queue: ResMut<DriverHitEffectPacketQueue>,
    mut hit_effect_queue: ResMut<DriverHitEffectQueue>,
    objects: Query<&GameObjectEntity>,
) {
    let packets = std::mem::take(&mut packet_queue.pending);
    for packet in packets {
        let Some(ref_id) =
            process_driver_hit_effect_packet(packet, objects.iter().map(|object| object.ref_id))
        else {
            continue;
        };
        hit_effect_queue.pending.push(ref_id);
    }
}

pub(crate) fn process_vehicle_lid_packet_queue(
    mut packet_queue: ResMut<VehicleLidPacketQueue>,
    mut objects: Query<(&GameObjectEntity, &mut VehicleLidState)>,
) {
    let packets = std::mem::take(&mut packet_queue.pending);
    for packet in packets {
        for (object, mut lid) in &mut objects {
            if apply_vehicle_lid_packet(&packet, object.ref_id, &mut lid) {
                break;
            }
        }
    }
}

pub(crate) fn process_crane_anim_packet_queue(
    mut commands: Commands,
    mut packet_queue: ResMut<CraneAnimPacketQueue>,
    cranes: Query<(Entity, &GameObjectEntity)>,
    repair_targets: Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        Option<&BridgeFootprint>,
    )>,
) {
    let packets = std::mem::take(&mut packet_queue.pending);
    for packet in packets {
        let Some((entity, _)) = cranes.iter().find(|(_, object)| {
            matches!(object.kind, ObjectKind::Vehicle(VehicleType::Crane))
                && u32::try_from(packet.ref_id).ok() == Some(object.ref_id)
        }) else {
            continue;
        };

        let target = crane_anim_visual_target_from_packet(&packet, &repair_targets);
        match apply_crane_anim_packet(&packet, target) {
            Some(Some(target)) => {
                commands.entity(entity).insert(target);
            }
            Some(None) => {
                commands.entity(entity).remove::<CraneConcoVisualTarget>();
            }
            None => {}
        }
    }
}

pub(crate) fn relay_startup_object_events(
    map: &ZMap,
    settings: &SourceSettingsState,
) -> Option<Vec<SourceObjectEvent>> {
    let wire_packet = RequestObjectsCommand.encode_packet();
    let payload = wire_packet.get(8..)?;
    RequestObjectsCommand::decode_payload(payload)?;

    let object_inits = server_object_init_packets(map, settings);
    let group_packets = server_object_group_info_packets(map, &object_inits, settings)?;
    let mut group_packet_by_ref = HashMap::new();
    let mut group_refs_by_ref = HashMap::new();
    for packet in group_packets {
        let decoded = ObjectGroupInfoPacket::decode_payload(&packet.encode_payload())?;
        let ref_id = u32::try_from(decoded.ref_id).ok()?;
        let leader_ref_id = u32::try_from(decoded.leader_ref_id).ok().unwrap_or(ref_id);
        group_packet_by_ref.insert(ref_id, decoded.clone());
        if leader_ref_id == ref_id {
            let mut refs = vec![ref_id];
            refs.extend(
                decoded
                    .minion_refs
                    .iter()
                    .filter_map(|minion_ref| u32::try_from(*minion_ref).ok()),
            );
            for member_ref in &refs {
                group_refs_by_ref.insert(*member_ref, refs.clone());
            }
        }
    }

    let mut events = Vec::new();
    for packet in object_inits {
        let packet = ObjectInitPacket::decode_payload(&packet.encode_payload())?;
        let apply = apply_new_object_packet(packet)?;
        let ref_id = apply.ref_id;
        events.push(SourceObjectEvent::AddNewObject {
            packet,
            just_left_cannon: false,
        });
        if let Some(packet) = group_packet_by_ref.remove(&ref_id) {
            events.push(SourceObjectEvent::ObjectGroupInfo {
                packet,
                requires_new_object_refs: group_refs_by_ref
                    .get(&ref_id)
                    .cloned()
                    .unwrap_or_else(|| vec![ref_id]),
            });
        }

        let health_packet = ObjectHealthPacket {
            ref_id: packet.ref_id,
            health: packet.health,
        };
        let health_packet = ObjectHealthPacket::decode_payload(&health_packet.encode_payload())?;
        events.push(SourceObjectEvent::UpdateHealth {
            packet: health_packet,
            requires_new_object_ref: Some(ref_id),
        });

        if matches!(apply.kind, ObjectKind::Building(_) | ObjectKind::Bridge(_)) {
            let state = startup_building_state_packet(packet, apply.kind, apply.owner, settings)?;
            events.push(SourceObjectEvent::BuildingState {
                packet: BuildingStatePacket::decode_payload(&state.encode_payload())?,
                requires_new_object_ref: Some(ref_id),
            });
            if apply.kind == ObjectKind::Building(BuildingType::Repair) {
                let repair = RepairBuildingAnimPacket {
                    ref_id: packet.ref_id,
                    on: false,
                    remaining_time: 0.0,
                    play_sound: false,
                };
                events.push(SourceObjectEvent::RepairBuildingAnimation {
                    packet: RepairBuildingAnimPacket::decode_payload(&repair.encode_payload())?,
                    requires_new_object_ref: Some(ref_id),
                });
            }
            if buildings::can_set_rallypoints(apply.kind) {
                let queue =
                    startup_building_queue_packet(packet, apply.kind, apply.owner, settings)?;
                events.push(SourceObjectEvent::BuildingQueue {
                    packet: BuildingQueuePacket::decode_payload(&queue.encode_payload())?,
                    requires_new_object_ref: Some(ref_id),
                });
            }
        }

        if units::items::grenades::can_have_grenades(apply.kind) {
            let grenade = ObjectGrenadeAmountPacket {
                ref_id: packet.ref_id,
                grenade_amount: 0,
            };
            events.push(SourceObjectEvent::ObjectGrenadeAmount {
                packet: ObjectGrenadeAmountPacket::decode_payload(&grenade.encode_payload())?,
            });
        }
        if buildings::can_set_rallypoints(apply.kind) {
            let rally = SendRallypointsPacket {
                ref_id: packet.ref_id,
                waypoints: Vec::new(),
            };
            events.push(SourceObjectEvent::ObjectRallyPoints {
                packet: SendRallypointsPacket::decode_payload(&rally.encode_payload())?,
                requires_new_object_ref: Some(ref_id),
            });
        }
    }

    Some(events)
}

#[cfg(test)]
pub(crate) fn relay_request_object_inits(map: &ZMap) -> Option<Vec<ObjectInitPacket>> {
    let settings = SourceSettingsState::default();
    let events = relay_startup_object_events(map, &settings)?;
    Some(
        events
            .into_iter()
            .filter_map(|event| match event {
                SourceObjectEvent::AddNewObject { packet, .. } => Some(packet),
                _ => None,
            })
            .collect(),
    )
}

#[cfg(test)]
pub(crate) fn next_ref_id_after_object_inits(objects: &[ObjectInitPacket]) -> u32 {
    objects
        .iter()
        .filter_map(|object| u32::try_from(object.ref_id).ok())
        .max()
        .map_or(0, |ref_id| ref_id + 1)
}

#[cfg(test)]
pub(crate) fn relay_object_group_infos(
    map: &ZMap,
    object_inits: &[ObjectInitPacket],
) -> Option<Vec<ObjectGroupInfoPacket>> {
    let settings = SourceSettingsState::default();
    let server_packets = server_object_group_info_packets(map, object_inits, &settings)?;
    let mut client_packets = Vec::new();
    for packet in server_packets {
        let wire_packet = packet.encode_packet();
        let payload = wire_packet.get(8..)?;
        let decoded_packet = ObjectGroupInfoPacket::decode_payload(payload)?;
        if process_group_info_packet(&decoded_packet, object_inits).is_some() {
            client_packets.push(decoded_packet);
        }
    }
    Some(client_packets)
}

#[cfg(test)]
pub(crate) fn relay_object_health_updates(
    object_inits: &[ObjectInitPacket],
) -> Option<Vec<ObjectHealthPacket>> {
    let mut client_packets = Vec::new();
    for object in object_inits {
        let packet = ObjectHealthPacket {
            ref_id: object.ref_id,
            health: object.health,
        };
        let wire_packet = packet.encode_packet();
        let payload = wire_packet.get(8..)?;
        let decoded_packet = ObjectHealthPacket::decode_payload(payload)?;
        if process_object_health_packet(decoded_packet, object_inits).is_some() {
            client_packets.push(decoded_packet);
        }
    }
    Some(client_packets)
}

pub(crate) fn relay_destroy_object(
    queue: &mut ObjectDestroyPacketQueue,
    ref_id: u32,
    killer_ref_id: Option<u32>,
    destroy_object: bool,
    do_fire_death: bool,
    do_missile_death: bool,
) -> bool {
    let Some(decoded_packet) = relay_destroy_object_packet(
        ref_id,
        killer_ref_id,
        destroy_object,
        do_fire_death,
        do_missile_death,
    ) else {
        return false;
    };
    queue.pending.push(decoded_packet);
    true
}

pub(crate) fn relay_destroy_object_packet(
    ref_id: u32,
    killer_ref_id: Option<u32>,
    destroy_object: bool,
    do_fire_death: bool,
    do_missile_death: bool,
) -> Option<DestroyObjectPacket> {
    relay_destroy_object_packet_with_missiles(
        ref_id,
        killer_ref_id,
        destroy_object,
        do_fire_death,
        do_missile_death,
        Vec::new(),
    )
}

pub(crate) fn relay_destroy_object_packet_with_missiles(
    ref_id: u32,
    killer_ref_id: Option<u32>,
    destroy_object: bool,
    do_fire_death: bool,
    do_missile_death: bool,
    fire_missiles: Vec<DestroyObjectMissileInfo>,
) -> Option<DestroyObjectPacket> {
    let Ok(ref_id) = i32::try_from(ref_id) else {
        return None;
    };
    let killer_ref_id = match killer_ref_id {
        Some(killer_ref_id) => {
            let Ok(killer_ref_id) = i32::try_from(killer_ref_id) else {
                return None;
            };
            killer_ref_id
        }
        None => -1,
    };
    let packet = DestroyObjectPacket {
        ref_id,
        killer_ref_id,
        destroy_object,
        do_fire_death,
        do_missile_death,
        fire_missiles,
    };
    let wire_packet = packet.encode_packet();
    let payload = wire_packet.get(8..)?;
    DestroyObjectPacket::decode_payload(payload)
}

pub(crate) fn relay_object_team_update(
    ref_id: u32,
    owner: TeamType,
    driver: Option<&DriverHealth>,
) -> Option<ObjectTeamPacket> {
    let packet = object_team_packet_from_state(ref_id, owner, driver)?;
    let wire_packet = packet.encode_packet();
    let payload = wire_packet.get(8..)?;
    ObjectTeamPacket::decode_payload(payload)
}

pub(crate) fn relay_object_attack_target(
    ref_id: u32,
    attack_target_ref_id: Option<u32>,
) -> Option<AttackObjectPacket> {
    let ref_id = i32::try_from(ref_id).ok()?;
    let attack_object_ref_id = match attack_target_ref_id {
        Some(target_ref_id) => i32::try_from(target_ref_id).ok()?,
        None => -1,
    };
    let packet = AttackObjectPacket {
        ref_id,
        attack_object_ref_id,
    };
    let wire_packet = packet.encode_packet();
    let payload = wire_packet.get(8..)?;
    AttackObjectPacket::decode_payload(payload)
}

pub(crate) fn source_waypoint_mode_for_movement(mode: MovementWaypointMode) -> SourceWaypointMode {
    match mode {
        MovementWaypointMode::Move => SourceWaypointMode::Move,
        MovementWaypointMode::Attack => SourceWaypointMode::Attack,
        MovementWaypointMode::ForceMove => SourceWaypointMode::ForceMove,
    }
}

pub(crate) fn movement_waypoint_mode_from_source(
    mode: SourceWaypointMode,
) -> Option<MovementWaypointMode> {
    Some(match mode {
        SourceWaypointMode::Move => MovementWaypointMode::Move,
        SourceWaypointMode::Attack => MovementWaypointMode::Attack,
        SourceWaypointMode::ForceMove => MovementWaypointMode::ForceMove,
        SourceWaypointMode::Enter
        | SourceWaypointMode::CraneRepair
        | SourceWaypointMode::UnitRepair
        | SourceWaypointMode::Agro
        | SourceWaypointMode::EnterFort
        | SourceWaypointMode::Dodge
        | SourceWaypointMode::PickupGrenades => return None,
    })
}

pub(crate) fn source_waypoint_from_movement(waypoint: MovementWaypoint) -> SourceWaypoint {
    SourceWaypoint {
        mode: source_waypoint_mode_for_movement(waypoint.mode),
        ref_id: waypoint
            .ref_id
            .and_then(|ref_id| i32::try_from(ref_id).ok())
            .unwrap_or(-1),
        x: waypoint.position.x.round() as i32,
        y: waypoint.position.y.round() as i32,
        attack_to: waypoint.attack_to,
        player_given: waypoint.player_given,
    }
}

pub(crate) fn movement_waypoint_from_source(waypoint: SourceWaypoint) -> Option<MovementWaypoint> {
    Some(MovementWaypoint {
        position: Vec2::new(waypoint.x as f32, waypoint.y as f32),
        mode: movement_waypoint_mode_from_source(waypoint.mode)?,
        ref_id: u32::try_from(waypoint.ref_id).ok(),
        attack_to: waypoint.attack_to,
        player_given: waypoint.player_given,
    })
}

pub(crate) fn source_waypoints_from_existing_path(
    path: Option<&MovementPath>,
) -> Vec<SourceWaypoint> {
    path.map(|path| {
        path.typed_waypoints
            .iter()
            .copied()
            .map(source_waypoint_from_movement)
            .collect()
    })
    .unwrap_or_default()
}

pub(crate) fn source_waypoints_from_movement_path(
    path: &MovementPath,
    layer_offset: Vec2,
) -> Vec<SourceWaypoint> {
    path.typed_waypoints
        .iter()
        .copied()
        .map(|waypoint| {
            source_waypoint_from_movement(waypoint.with_position(waypoint.position - layer_offset))
        })
        .collect()
}

pub(crate) fn relay_object_waypoints(
    ref_id: u32,
    waypoints: &[SourceWaypoint],
) -> Option<SendWaypointsPacket> {
    let ref_id = i32::try_from(ref_id).ok()?;
    let packet = SendWaypointsPacket {
        ref_id,
        waypoints: waypoints.to_vec(),
    };
    let wire_packet = packet.encode_packet();
    let payload = wire_packet.get(8..)?;
    SendWaypointsPacket::decode_payload(payload)
}

pub(crate) fn relay_object_empty_waypoints(ref_id: u32) -> Option<SendWaypointsPacket> {
    relay_object_waypoints(ref_id, &[])
}

pub(crate) fn relay_object_location(
    ref_id: u32,
    map_position: Vec2,
    velocity: Vec2,
) -> Option<ObjectLocationPacket> {
    let ref_id = i32::try_from(ref_id).ok()?;
    let packet = ObjectLocationPacket {
        ref_id,
        x: source_location_i32(map_position.x)?,
        y: source_location_i32(map_position.y)?,
        dx: velocity.x,
        dy: velocity.y,
    };
    let wire_packet = packet.encode_packet();
    let payload = wire_packet.get(8..)?;
    ObjectLocationPacket::decode_payload(payload)
}

pub(crate) fn apply_object_world_location_packet(
    packet: &ObjectLocationPacket,
    object_ref_id: u32,
    layer_offset: Vec2,
) -> Option<(Vec2, Vec2)> {
    let (map_position, map_velocity) = apply_object_location_packet(packet, object_ref_id)?;
    Some((
        Vec2::new(map_position.x, -map_position.y) + layer_offset,
        Vec2::new(map_velocity.x, -map_velocity.y),
    ))
}

pub(crate) fn process_object_location_packet_queue(
    mut commands: Commands,
    mut packet_queue: ResMut<ObjectLocationPacketQueue>,
    destroy_queue: Res<ObjectDestroyPacketQueue>,
    mut queries: ParamSet<(
        Query<(
            &GameObjectEntity,
            &SourceObjectLocation,
            &Transform,
            Option<&DestroyedObject>,
        )>,
        Query<(
            Entity,
            &ObjectLayerRef,
            &mut Transform,
            Option<&mut MovementVelocity>,
            Option<&mut MobileSpriteLayer>,
            Option<&mut SourceObjectLocation>,
        )>,
    )>,
) {
    let packets = std::mem::take(&mut packet_queue.pending);
    if packets.is_empty() {
        return;
    }

    let object_locations: Vec<_> = queries
        .p0()
        .iter()
        .map(|(object, location, transform, destroyed)| {
            (
                object.ref_id,
                location.map_position,
                location.world_anchor,
                transform.translation.truncate(),
                destroyed.is_some(),
            )
        })
        .collect();
    let layer_offsets: Vec<_> = queries
        .p1()
        .iter_mut()
        .filter_map(|(entity, layer_ref, transform, _, _, _)| {
            let base_world_position =
                object_locations
                    .iter()
                    .find_map(|(ref_id, _, _, base_position, _)| {
                        (*ref_id == layer_ref.0).then_some(*base_position)
                    })?;
            Some((
                entity,
                layer_ref.0,
                transform.translation.truncate() - base_world_position,
            ))
        })
        .collect();
    let pending_destroy_refs: Vec<u32> = destroy_queue
        .pending
        .iter()
        .filter_map(|packet| u32::try_from(packet.ref_id).ok())
        .collect();

    for packet in packets {
        let Ok(ref_id) = u32::try_from(packet.ref_id) else {
            continue;
        };
        let Some((_, _, world_anchor, _, destroyed)) = object_locations
            .iter()
            .find(|(object_ref_id, _, _, _, _)| *object_ref_id == ref_id)
        else {
            continue;
        };
        if *destroyed || pending_destroy_refs.contains(&ref_id) {
            continue;
        }
        let Some((map_position, map_velocity)) = apply_object_location_packet(&packet, ref_id)
        else {
            continue;
        };

        for (
            entity,
            layer_ref,
            mut transform,
            mut movement_velocity,
            mut mobile,
            mut source_location,
        ) in &mut queries.p1()
        {
            if layer_ref.0 != ref_id {
                continue;
            }
            let Some(layer_offset) =
                layer_offsets
                    .iter()
                    .find_map(|(layer_entity, layer_ref_id, offset)| {
                        (*layer_entity == entity && *layer_ref_id == ref_id).then_some(*offset)
                    })
            else {
                continue;
            };
            let packet_world_offset = *world_anchor + layer_offset;
            let Some((world_position, world_velocity)) =
                apply_object_world_location_packet(&packet, ref_id, packet_world_offset)
            else {
                continue;
            };
            let velocity_changed = movement_velocity
                .as_ref()
                .is_none_or(|velocity| velocity.0 != world_velocity);

            transform.translation.x = world_position.x;
            transform.translation.y = world_position.y;
            if let Some(velocity) = &mut movement_velocity {
                velocity.0 = world_velocity;
            }
            if let Some(location) = &mut source_location {
                location.map_position = map_position;
                location.map_velocity = map_velocity;
            }
            if velocity_changed
                && MovementVelocity(world_velocity).is_moving()
                && let Some(direction) = crate::direction_index_from_delta(world_velocity)
                && let Some(mobile) = &mut mobile
            {
                mobile.rotation = crate::rotation_for_direction(direction);
            }
            commands.entity(entity).insert(SourceLocationInterpolation {
                last_map_position: map_position,
                layer_map_offset: Vec2::new(packet_world_offset.x, -packet_world_offset.y),
                map_velocity,
                elapsed: 0.0,
                just_set: true,
            });
        }
    }
}

pub(crate) fn smooth_object_locations(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &mut SourceLocationInterpolation,
            Option<&mut SourceObjectLocation>,
            Option<&mut MovementVelocity>,
        ),
        Without<DestroyedObject>,
    >,
) {
    for (mut transform, mut interpolation, mut source_location, mut movement_velocity) in &mut query
    {
        if interpolation.just_set {
            interpolation.just_set = false;
            continue;
        }

        interpolation.elapsed += time.delta_secs();
        let map_delta = Vec2::new(
            (interpolation.map_velocity.x * interpolation.elapsed).floor(),
            (interpolation.map_velocity.y * interpolation.elapsed).floor(),
        );
        let object_map_position = interpolation.last_map_position + map_delta;
        let layer_map_position = object_map_position + interpolation.layer_map_offset;
        transform.translation.x = layer_map_position.x;
        transform.translation.y = -layer_map_position.y;
        if let Some(location) = &mut source_location {
            location.map_position = object_map_position;
            location.map_velocity = interpolation.map_velocity;
        }
        if let Some(velocity) = &mut movement_velocity {
            velocity.0 = Vec2::new(interpolation.map_velocity.x, -interpolation.map_velocity.y);
        }
    }
}

pub(crate) fn accepted_empty_waypoint_location_packets(
    leader_ref_id: u32,
    snapshots: &[EmptyWaypointObjectSnapshot],
) -> Option<Vec<ObjectLocationPacket>> {
    let leader = snapshots
        .iter()
        .find(|snapshot| snapshot.ref_id == leader_ref_id && !snapshot.destroyed)?;
    if leader
        .group_leader_ref_id
        .is_some_and(|group_ref_id| group_ref_id != leader_ref_id)
    {
        return None;
    }

    let mut members = vec![*leader];
    let mut minions: Vec<_> = snapshots
        .iter()
        .copied()
        .filter(|snapshot| {
            snapshot.ref_id != leader_ref_id
                && !snapshot.destroyed
                && snapshot.group_leader_ref_id == Some(leader_ref_id)
        })
        .collect();
    minions.sort_by_key(|snapshot| snapshot.ref_id);
    members.extend(minions);

    members
        .into_iter()
        .map(|snapshot| relay_object_location(snapshot.ref_id, snapshot.map_position, Vec2::ZERO))
        .collect()
}

pub(crate) fn process_accepted_empty_waypoint_commands(
    mut commands: Commands,
    mut location_packets: ResMut<ObjectLocationPacketQueue>,
    mut queries: ParamSet<(
        Query<
            (
                &GameObjectEntity,
                Option<&MovementPath>,
                Option<&PickupGrenadesTarget>,
                Option<&EnterTarget>,
                Option<&EnterFortTarget>,
                Option<&CraneRepairTarget>,
                Option<&UnitRepairTarget>,
            ),
            With<AcceptedEmptyWaypointCommand>,
        >,
        Query<(
            &GameObjectEntity,
            &SourceObjectLocation,
            Option<&RobotGroup>,
            &ObjectStats,
        )>,
        Query<(Entity, &ObjectLayerRef)>,
    )>,
) {
    let mut pending_refs = Vec::new();
    let mut stale_refs = Vec::new();
    for (object, path, pickup, enter, enter_fort, crane_repair, unit_repair) in &queries.p0() {
        if path.is_some()
            || pickup.is_some()
            || enter.is_some()
            || enter_fort.is_some()
            || crane_repair.is_some()
            || unit_repair.is_some()
        {
            stale_refs.push(object.ref_id);
        } else {
            pending_refs.push(object.ref_id);
        }
    }
    if !stale_refs.is_empty() {
        for (entity, layer_ref) in &queries.p2() {
            if stale_refs.contains(&layer_ref.0) {
                commands
                    .entity(entity)
                    .remove::<AcceptedEmptyWaypointCommand>();
            }
        }
    }
    if pending_refs.is_empty() {
        return;
    }
    for (entity, layer_ref) in &queries.p2() {
        if pending_refs.contains(&layer_ref.0) {
            commands
                .entity(entity)
                .remove::<AcceptedEmptyWaypointCommand>();
        }
    }

    let snapshots: Vec<_> = queries
        .p1()
        .iter()
        .map(
            |(object, location, group, stats)| EmptyWaypointObjectSnapshot {
                ref_id: object.ref_id,
                map_position: location.map_position,
                group_leader_ref_id: group.map(|group| group.leader_ref_id),
                destroyed: stats.destroyed(),
            },
        )
        .collect();
    let mut leader_refs: Vec<u32> = pending_refs
        .iter()
        .filter_map(|ref_id| {
            snapshots
                .iter()
                .find(|snapshot| snapshot.ref_id == *ref_id)
                .map(|snapshot| snapshot.group_leader_ref_id.unwrap_or(snapshot.ref_id))
        })
        .collect();
    leader_refs.sort_unstable();
    leader_refs.dedup();

    for leader_ref_id in leader_refs {
        let Some(packets) = accepted_empty_waypoint_location_packets(leader_ref_id, &snapshots)
        else {
            continue;
        };
        for packet in packets {
            let Ok(ref_id) = u32::try_from(packet.ref_id) else {
                continue;
            };
            location_packets.pending.push(packet);
            for (entity, layer_ref) in &queries.p2() {
                if layer_ref.0 != ref_id {
                    continue;
                }
                commands
                    .entity(entity)
                    .remove::<AcceptedEmptyWaypointCommand>()
                    .remove::<MovementPath>()
                    .remove::<PickupGrenadesTarget>()
                    .remove::<EnterTarget>()
                    .remove::<EnterFortTarget>()
                    .remove::<CraneRepairTarget>()
                    .remove::<UnitRepairTarget>()
                    .remove::<JustLeftCannon>();
            }
        }
    }
}

pub(crate) fn driverless_object_cleanup_plan(
    target_ref_id: u32,
    snapshots: &[DriverlessObjectCleanupSnapshot],
) -> Option<DriverlessObjectCleanupPlan> {
    let target = snapshots
        .iter()
        .find(|snapshot| snapshot.ref_id == target_ref_id)?;
    let target_has_waypoints = target
        .movement_path
        .as_ref()
        .is_some_and(|path| !path.is_empty())
        || target.has_special_waypoint;
    let target_attack_clear_packet = if target.attack_target_ref_id.is_some() {
        Some(relay_object_attack_target(target.ref_id, None)?)
    } else {
        None
    };
    let target_waypoint_packet = if target_has_waypoints {
        Some(relay_object_empty_waypoints(target.ref_id)?)
    } else {
        None
    };
    let target_stop_location_packet = if target_has_waypoints && target.is_moving {
        Some(relay_object_location(
            target.ref_id,
            target.map_position,
            Vec2::ZERO,
        )?)
    } else {
        None
    };
    let target_event = DriverlessObjectCleanupEvent {
        ref_id: target.ref_id,
        base_world_position: target.world_position,
        attack_clear_packet: target_attack_clear_packet,
        waypoint_packet: target_waypoint_packet,
        stop_location_packet: target_stop_location_packet,
    };

    let mut ordered_snapshots: Vec<_> = snapshots
        .iter()
        .filter(|snapshot| snapshot.ref_id != target_ref_id)
        .collect();
    ordered_snapshots.sort_by_key(|snapshot| snapshot.ref_id);

    let mut dependents = Vec::new();
    for snapshot in ordered_snapshots {
        let clears_attack = snapshot.attack_target_ref_id == Some(target_ref_id);
        let removes_front_waypoint = snapshot
            .movement_path
            .as_ref()
            .and_then(|path| path.typed_waypoints.first())
            .is_some_and(|waypoint| {
                waypoint.mode == MovementWaypointMode::Attack
                    && waypoint.ref_id == Some(target_ref_id)
            });
        if !clears_attack && !removes_front_waypoint {
            continue;
        }

        let waypoint_packet = if removes_front_waypoint {
            let mut remaining_path = snapshot.movement_path.clone()?;
            remaining_path.pop_front_waypoint();
            Some(relay_object_waypoints(
                snapshot.ref_id,
                &source_waypoints_from_existing_path(Some(&remaining_path)),
            )?)
        } else {
            None
        };
        let stop_location_packet = if removes_front_waypoint && snapshot.is_moving {
            Some(relay_object_location(
                snapshot.ref_id,
                snapshot.map_position,
                Vec2::ZERO,
            )?)
        } else {
            None
        };
        let attack_clear_packet = if clears_attack {
            Some(relay_object_attack_target(snapshot.ref_id, None)?)
        } else {
            None
        };
        dependents.push(DriverlessObjectCleanupEvent {
            ref_id: snapshot.ref_id,
            base_world_position: snapshot.world_position,
            attack_clear_packet,
            waypoint_packet,
            stop_location_packet,
        });
    }

    Some(DriverlessObjectCleanupPlan {
        target: target_event,
        dependents,
    })
}

pub(crate) fn relay_object_grenade_amount(
    ref_id: u32,
    grenade_amount: u8,
) -> Option<ObjectGrenadeAmountPacket> {
    let ref_id = i32::try_from(ref_id).ok()?;
    let packet = ObjectGrenadeAmountPacket {
        ref_id,
        grenade_amount: i32::from(grenade_amount),
    };
    let wire_packet = packet.encode_packet();
    let payload = wire_packet.get(8..)?;
    ObjectGrenadeAmountPacket::decode_payload(payload)
}

pub(crate) fn relay_pickup_grenade_animation(ref_id: u32) -> Option<PickupGrenadeAnimationPacket> {
    let ref_id = i32::try_from(ref_id).ok()?;
    let packet = PickupGrenadeAnimationPacket { ref_id };
    let wire_packet = packet.encode_packet();
    let payload = wire_packet.get(8..)?;
    PickupGrenadeAnimationPacket::decode_payload(payload)
}

pub(crate) fn relay_driver_hit_effect(queue: &mut DriverHitEffectPacketQueue, ref_id: u32) -> bool {
    let Ok(ref_id) = i32::try_from(ref_id) else {
        return false;
    };
    let packet = DriverHitEffectPacket { ref_id };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = DriverHitEffectPacket::decode_payload(payload) else {
        return false;
    };
    queue.pending.push(decoded_packet);
    true
}

pub(crate) fn relay_snipe_object(ref_id: u32) -> Option<SnipeObjectPacket> {
    let ref_id = i32::try_from(ref_id).ok()?;
    let packet = SnipeObjectPacket { ref_id };
    let wire_packet = packet.encode_packet();
    let payload = wire_packet.get(8..)?;
    SnipeObjectPacket::decode_payload(payload)
}

pub(crate) fn relay_vehicle_lid_state(
    queue: &mut VehicleLidPacketQueue,
    ref_id: u32,
    lid_open: bool,
) -> bool {
    let Ok(ref_id) = i32::try_from(ref_id) else {
        return false;
    };
    let packet = SetLidOpenPacket { ref_id, lid_open };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = SetLidOpenPacket::decode_payload(payload) else {
        return false;
    };
    queue.pending.push(decoded_packet);
    true
}

pub(crate) fn relay_eject_vehicle_command(
    queue: &mut EjectVehiclePacketQueue,
    ref_id: u32,
) -> bool {
    let Ok(ref_id) = i32::try_from(ref_id) else {
        return false;
    };
    let packet = EjectVehiclePacket { ref_id };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = EjectVehiclePacket::decode_payload(payload) else {
        return false;
    };
    queue.pending.push(decoded_packet);
    true
}

pub(crate) fn relay_crane_anim_state(
    queue: &mut CraneAnimPacketQueue,
    ref_id: u32,
    repair_ref_id: u32,
    on: bool,
) -> bool {
    let Ok(ref_id) = i32::try_from(ref_id) else {
        return false;
    };
    let Ok(repair_ref_id) = i32::try_from(repair_ref_id) else {
        return false;
    };
    let packet = CraneAnimPacket {
        ref_id,
        repair_ref_id,
        on,
    };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = CraneAnimPacket::decode_payload(payload) else {
        return false;
    };
    queue.pending.push(decoded_packet);
    true
}

pub(crate) fn relay_repair_building_anim_state(
    queue: &mut RepairBuildingAnimPacketQueue,
    ref_id: u32,
    on: bool,
    remaining_time: f32,
    play_sound: bool,
) -> bool {
    let Ok(ref_id) = i32::try_from(ref_id) else {
        return false;
    };
    let packet = RepairBuildingAnimPacket {
        ref_id,
        on,
        remaining_time: f64::from(remaining_time.max(0.0)),
        play_sound,
    };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = RepairBuildingAnimPacket::decode_payload(payload) else {
        return false;
    };
    queue.pending.push(decoded_packet);
    true
}

pub(crate) fn relay_portrait_anim(
    ref_id: u32,
    kind: PortraitAnimationKind,
) -> Option<DoPortraitAnimPacket> {
    let ref_id = i32::try_from(ref_id).ok()?;
    let packet = DoPortraitAnimPacket {
        ref_id,
        anim_id: kind.wire_id(),
    };
    let wire_packet = packet.encode_packet();
    let payload = wire_packet.get(8..)?;
    DoPortraitAnimPacket::decode_payload(payload)
}

pub(crate) fn apply_vehicle_lid_packet(
    packet: &SetLidOpenPacket,
    object_ref_id: u32,
    lid: &mut VehicleLidState,
) -> bool {
    let Ok(packet_ref_id) = u32::try_from(packet.ref_id) else {
        return false;
    };
    if packet_ref_id != object_ref_id {
        return false;
    }
    units::vehicles::set_lid_state(lid, packet.lid_open);
    true
}

pub(crate) fn apply_eject_vehicle_packet(
    packet: &EjectVehiclePacket,
    object_ref_id: u32,
    object_owner: TeamType,
    player_team: TeamType,
    object_kind: ObjectKind,
    stats: ObjectStats,
) -> bool {
    let Ok(packet_ref_id) = u32::try_from(packet.ref_id) else {
        return false;
    };
    packet_ref_id == object_ref_id
        && object_owner != TeamType::Null
        && object_owner == player_team
        && units::can_eject_drivers(object_kind, stats)
}

pub(crate) fn apply_crane_anim_packet(
    packet: &CraneAnimPacket,
    target: Option<CraneConcoVisualTarget>,
) -> Option<Option<CraneConcoVisualTarget>> {
    if packet.ref_id < 0 {
        return None;
    }
    if !packet.on {
        return Some(None);
    }
    target.map(Some)
}

pub(crate) fn apply_repair_building_anim_packet(
    packet: &RepairBuildingAnimPacket,
    object_ref_id: u32,
    object_kind: ObjectKind,
) -> Option<RepairBuildingAnimApply> {
    let packet_ref_id = u32::try_from(packet.ref_id).ok()?;
    if packet_ref_id != object_ref_id {
        return None;
    }
    if object_kind != ObjectKind::Building(BuildingType::Repair) {
        return None;
    }
    let state = packet.on.then(|| RepairBuildingAnimState {
        remaining_time: source_remaining_time_to_seconds(packet.remaining_time),
    });
    Some(RepairBuildingAnimApply {
        state,
        play_sound: packet.play_sound,
    })
}

fn source_remaining_time_to_seconds(remaining_time: f64) -> f32 {
    if !remaining_time.is_finite() {
        return 0.0;
    }
    remaining_time.clamp(0.0, f64::from(f32::MAX)) as f32
}

pub(crate) fn apply_object_attack_packet(
    packet: &AttackObjectPacket,
    object_ref_id: u32,
    object_ref_ids: impl IntoIterator<Item = u32>,
) -> Option<AttackObjectApply> {
    let packet_ref_id = u32::try_from(packet.ref_id).ok()?;
    if packet_ref_id != object_ref_id {
        return None;
    }
    if packet.attack_object_ref_id < 0 {
        return Some(AttackObjectApply {
            target_ref_id: None,
        });
    }
    let attack_object_ref_id = u32::try_from(packet.attack_object_ref_id).ok()?;
    let target_ref_id = object_ref_ids
        .into_iter()
        .any(|ref_id| ref_id == attack_object_ref_id)
        .then_some(attack_object_ref_id);
    Some(AttackObjectApply { target_ref_id })
}

pub(crate) fn apply_object_waypoints_packet<'a>(
    packet: &'a SendWaypointsPacket,
    object_ref_id: u32,
) -> Option<&'a [SourceWaypoint]> {
    let packet_ref_id = u32::try_from(packet.ref_id).ok()?;
    (packet_ref_id == object_ref_id).then_some(packet.waypoints.as_slice())
}

pub(crate) fn apply_empty_object_waypoints_packet(
    packet: &SendWaypointsPacket,
    object_ref_id: u32,
) -> bool {
    apply_object_waypoints_packet(packet, object_ref_id)
        .is_some_and(|waypoints| waypoints.is_empty())
}

pub(crate) fn movement_path_from_object_waypoints_packet(
    packet: &SendWaypointsPacket,
    object_ref_id: u32,
    current_path: Option<&MovementPath>,
    layer_offset: Vec2,
) -> Option<Option<MovementPath>> {
    let source_waypoints = apply_object_waypoints_packet(packet, object_ref_id)?;
    if source_waypoints.is_empty() {
        return Some(None);
    }

    let current_path = current_path?;
    let typed_waypoints = source_waypoints
        .iter()
        .copied()
        .map(|waypoint| {
            movement_waypoint_from_source(waypoint)
                .map(|waypoint| waypoint.with_position(waypoint.position + layer_offset))
        })
        .collect::<Option<Vec<_>>>()?;
    let mut path = MovementPath::from_typed(typed_waypoints, current_path.speed);
    path.attempt_run = current_path.attempt_run;
    Some(Some(path))
}

pub(crate) fn apply_object_location_packet(
    packet: &ObjectLocationPacket,
    object_ref_id: u32,
) -> Option<(Vec2, Vec2)> {
    let packet_ref_id = u32::try_from(packet.ref_id).ok()?;
    if packet_ref_id != object_ref_id {
        return None;
    }
    Some((
        Vec2::new(packet.x as f32, packet.y as f32),
        Vec2::new(packet.dx, packet.dy),
    ))
}

pub(crate) fn apply_object_grenade_amount_packet(
    packet: &ObjectGrenadeAmountPacket,
    object_ref_id: u32,
    inventory: &mut GrenadeInventory,
) -> bool {
    let Ok(packet_ref_id) = u32::try_from(packet.ref_id) else {
        return false;
    };
    if packet_ref_id != object_ref_id {
        return false;
    }
    inventory.amount = source_grenade_amount(packet.grenade_amount);
    true
}

pub(crate) fn apply_pickup_grenade_animation_packet(
    packet: &PickupGrenadeAnimationPacket,
    object_ref_id: u32,
    can_have_grenades: bool,
    is_attacking: bool,
) -> bool {
    let Ok(packet_ref_id) = u32::try_from(packet.ref_id) else {
        return false;
    };
    packet_ref_id == object_ref_id && can_have_grenades && !is_attacking
}

pub(crate) fn apply_object_team_packet(
    packet: &ObjectTeamPacket,
    ref_id: u32,
) -> Option<ObjectTeamApply> {
    let packet_ref_id = u32::try_from(packet.ref_id).ok()?;
    if packet_ref_id != ref_id {
        return None;
    }
    let owner = TeamType::try_from(packet.owner).ok()?;
    let driver_kind = RobotType::try_from(u8::try_from(packet.driver_type).ok()?).ok()?;
    let driver_healths: Vec<f32> = packet
        .drivers
        .iter()
        .map(|driver| driver.driver_health as f32)
        .collect();
    let next_attack_cooldowns: Vec<f32> = packet
        .drivers
        .iter()
        .map(|driver| driver.next_attack_time as f32)
        .collect();
    let driver = (!driver_healths.is_empty()).then(|| {
        DriverHealth::with_driver_states(driver_kind, driver_healths, next_attack_cooldowns)
    });
    Some(ObjectTeamApply { owner, driver })
}

pub(crate) fn apply_portrait_anim_packet(
    packet: &DoPortraitAnimPacket,
    object_ref_id: u32,
    object_team: TeamType,
    local_team: TeamType,
    portrait_busy: bool,
) -> Option<PortraitAnimationApply> {
    let packet_ref_id = u32::try_from(packet.ref_id).ok()?;
    if packet_ref_id != object_ref_id || object_team != local_team || portrait_busy {
        return None;
    }
    Some(PortraitAnimationApply {
        ref_id: packet_ref_id,
        kind: PortraitAnimationKind::from_wire_id(packet.anim_id),
    })
}

fn crane_anim_visual_target_from_packet(
    packet: &CraneAnimPacket,
    repair_targets: &Query<(
        &GameObjectEntity,
        &Transform,
        &Selectable,
        Option<&BridgeFootprint>,
    )>,
) -> Option<CraneConcoVisualTarget> {
    if !packet.on || packet.repair_ref_id < 0 {
        return None;
    }
    let repair_ref_id = u32::try_from(packet.repair_ref_id).ok()?;
    repair_targets
        .iter()
        .find_map(|(object, transform, selectable, bridge)| {
            (object.ref_id == repair_ref_id
                && matches!(object.kind, ObjectKind::Building(_) | ObjectKind::Bridge(_)))
            .then(|| {
                let (top_left_map, size) = buildings::repair_target_map_bounds(
                    transform.translation.truncate(),
                    selectable.selection_size,
                    bridge.copied(),
                );
                CraneConcoVisualTarget {
                    ref_id: object.ref_id,
                    top_left_map,
                    size,
                    is_bridge: bridge.is_some(),
                }
            })
        })
}

fn startup_building_state_packet(
    object: ObjectInitPacket,
    kind: ObjectKind,
    owner: TeamType,
    settings: &SourceSettingsState,
) -> Option<BuildingStatePacket> {
    let mut stats = settings.object_stats(kind, 100);
    stats.health = object.health as f32;
    let production = initial_production_for_building_from_source(
        kind,
        object.building_level,
        owner,
        stats,
        settings,
    );
    let (state, init_offset, production_time, object_type, object_id) = production
        .as_ref()
        .map(|production| {
            let current_parts = production
                .current
                .and_then(units::object_kind_to_map_parts)
                .map(|(object_type, object_id)| (object_type as u8, object_id));
            (
                building_production_status_wire(production.status),
                -(production.elapsed as f64),
                production.duration as f64,
                current_parts.map_or(u8::MAX, |parts| parts.0),
                current_parts.map_or(u8::MAX, |parts| parts.1),
            )
        })
        .unwrap_or((1, 0.0, 0.0, u8::MAX, u8::MAX));
    Some(BuildingStatePacket {
        ref_id: object.ref_id,
        state,
        init_offset,
        production_time,
        object_type,
        object_id,
    })
}

fn startup_building_queue_packet(
    object: ObjectInitPacket,
    kind: ObjectKind,
    owner: TeamType,
    settings: &SourceSettingsState,
) -> Option<BuildingQueuePacket> {
    let mut stats = settings.object_stats(kind, 100);
    stats.health = object.health as f32;
    let units = initial_production_for_building_from_source(
        kind,
        object.building_level,
        owner,
        stats,
        settings,
    )
    .map(|production| {
        production
            .queue
            .into_iter()
            .filter_map(units::object_kind_to_map_parts)
            .map(|(object_type, object_id)| BuildingQueueUnit {
                object_type: object_type as u8,
                object_id,
            })
            .collect()
    })
    .unwrap_or_default();
    Some(BuildingQueuePacket {
        ref_id: object.ref_id,
        units,
    })
}

fn building_production_status_wire(status: BuildingProductionStatus) -> i32 {
    match status {
        BuildingProductionStatus::Place => 0,
        BuildingProductionStatus::Select => 1,
        BuildingProductionStatus::Building => 2,
        BuildingProductionStatus::Paused => 3,
    }
}

fn server_object_init_packets(map: &ZMap, settings: &SourceSettingsState) -> Vec<ObjectInitPacket> {
    let mut packets = Vec::new();
    let mut next_ref_id = 0_i32;

    for object in &map.objects {
        let kind = ObjectKind::from_map_parts(object.object_type, object.object_id)
            .unwrap_or(ObjectKind::MapItem(object.object_id));
        let spawn_count = settings.initial_spawn_count(kind) as usize;

        for _ in 0..spawn_count {
            packets.push(object_init_packet_from_map_object(object, next_ref_id));
            next_ref_id += 1;
        }
    }

    packets
}

fn server_object_group_info_packets(
    map: &ZMap,
    object_inits: &[ObjectInitPacket],
    settings: &SourceSettingsState,
) -> Option<Vec<ObjectGroupInfoPacket>> {
    let mut packets = Vec::new();
    let mut init_index = 0;

    for object in &map.objects {
        let kind = ObjectKind::from_map_parts(object.object_type, object.object_id)
            .unwrap_or(ObjectKind::MapItem(object.object_id));
        let spawn_count = settings.initial_spawn_count(kind) as usize;
        if spawn_count == 0 {
            continue;
        }

        let group = object_inits.get(init_index..init_index + spawn_count)?;
        init_index += spawn_count;
        if !matches!(kind, ObjectKind::Robot(_)) {
            continue;
        }

        let leader_ref_id = group.first()?.ref_id;
        let minion_refs = group.iter().skip(1).map(|packet| packet.ref_id).collect();
        packets.push(ObjectGroupInfoPacket {
            ref_id: leader_ref_id,
            leader_ref_id: -1,
            minion_refs,
        });
        for minion in group.iter().skip(1) {
            packets.push(ObjectGroupInfoPacket {
                ref_id: minion.ref_id,
                leader_ref_id,
                minion_refs: Vec::new(),
            });
        }
    }

    Some(packets)
}

fn object_init_packet_from_map_object(object: &MapObject, ref_id: i32) -> ObjectInitPacket {
    let kind = ObjectKind::from_map_parts(object.object_type, object.object_id)
        .unwrap_or(ObjectKind::MapItem(object.object_id));
    ObjectInitPacket {
        x: i32::from(object.x) * TILE_SIZE as i32,
        y: i32::from(object.y) * TILE_SIZE as i32,
        ref_id,
        owner: team_wire(units::items::object_display_owner(kind, object.owner)),
        object_type: object.object_type as u8,
        object_id: object.object_id,
        building_level: object.building_level,
        extra_links: object.extra_links,
        health: source_health_from_percent(kind, object.health_percent),
    }
}

fn apply_new_object_packet(packet: ObjectInitPacket) -> Option<ObjectInitApply> {
    let packet = process_new_object_packet(packet)?;
    let ref_id = u32::try_from(packet.ref_id).ok()?;
    let owner = TeamType::try_from(packet.owner).ok()?;
    let kind = source_object_kind_from_init_parts(packet.object_type, packet.object_id)?;
    Some(ObjectInitApply {
        ref_id,
        kind,
        owner,
        source_map_position: Vec2::new(packet.x as f32, packet.y as f32),
        building_level: packet.building_level,
        extra_links: packet.extra_links,
    })
}

fn source_object_init_parts(kind: ObjectKind) -> Option<(u8, u8)> {
    let (object_type, object_id) = match kind {
        ObjectKind::Bridge(building) | ObjectKind::Building(building) => {
            (MapObjectType::Building, building as u8)
        }
        ObjectKind::Cannon(cannon) => (MapObjectType::Cannon, cannon as u8),
        ObjectKind::Vehicle(vehicle) => (MapObjectType::Vehicle, vehicle as u8),
        ObjectKind::Robot(robot) => (MapObjectType::Robot, robot as u8),
        ObjectKind::Rock => (MapObjectType::MapItem, ItemType::Rock as u8),
        ObjectKind::MapItem(item_id) if item_id <= ItemType::MapObjectStart as u8 + 21 => {
            (MapObjectType::MapItem, item_id)
        }
        ObjectKind::Animal(animal_id) => (MapObjectType::Animal, animal_id),
        ObjectKind::MapItem(_) => return None,
    };
    Some((object_type as u8, object_id))
}

fn source_object_kind_from_init_parts(object_type: u8, object_id: u8) -> Option<ObjectKind> {
    match MapObjectType::try_from(object_type).ok()? {
        MapObjectType::Building => {
            let building = BuildingType::try_from(object_id).ok()?;
            if matches!(
                building,
                BuildingType::BridgeVert | BuildingType::BridgeHorz
            ) {
                Some(ObjectKind::Bridge(building))
            } else {
                Some(ObjectKind::Building(building))
            }
        }
        MapObjectType::Cannon => Some(ObjectKind::Cannon(CannonType::try_from(object_id).ok()?)),
        MapObjectType::Vehicle => Some(ObjectKind::Vehicle(VehicleType::try_from(object_id).ok()?)),
        MapObjectType::Robot => Some(ObjectKind::Robot(RobotType::try_from(object_id).ok()?)),
        MapObjectType::Animal => Some(ObjectKind::Animal(object_id)),
        MapObjectType::MapItem if object_id == ItemType::Rock as u8 => Some(ObjectKind::Rock),
        MapObjectType::MapItem if object_id <= ItemType::MapObjectStart as u8 + 21 => {
            Some(ObjectKind::MapItem(object_id))
        }
        MapObjectType::Rock | MapObjectType::Bridge | MapObjectType::MapItem => None,
    }
}

fn process_new_object_packet(packet: ObjectInitPacket) -> Option<ObjectInitPacket> {
    if packet.ref_id < 0 {
        return None;
    }
    TeamType::try_from(packet.owner).ok()?;
    crate::original::map::MapObjectType::try_from(packet.object_type).ok()?;
    Some(packet)
}

#[cfg(test)]
fn process_group_info_packet<'a>(
    packet: &'a ObjectGroupInfoPacket,
    object_inits: &[ObjectInitPacket],
) -> Option<&'a ObjectGroupInfoPacket> {
    object_inits
        .iter()
        .any(|object| object.ref_id == packet.ref_id)
        .then_some(())?;
    if packet.leader_ref_id != -1
        && !object_inits
            .iter()
            .any(|object| object.ref_id == packet.leader_ref_id)
    {
        return None;
    }
    if packet.minion_refs.iter().any(|minion_ref| {
        !object_inits
            .iter()
            .any(|object| object.ref_id == *minion_ref)
    }) {
        return None;
    }
    Some(packet)
}

#[cfg(test)]
fn process_object_health_packet(
    packet: ObjectHealthPacket,
    object_inits: &[ObjectInitPacket],
) -> Option<ObjectHealthPacket> {
    object_inits
        .iter()
        .any(|object| object.ref_id == packet.ref_id)
        .then_some(packet)
}

fn process_driver_hit_effect_packet(
    packet: DriverHitEffectPacket,
    object_ref_ids: impl Iterator<Item = u32>,
) -> Option<u32> {
    let ref_id = u32::try_from(packet.ref_id).ok()?;
    object_ref_ids
        .into_iter()
        .any(|object_ref_id| object_ref_id == ref_id)
        .then_some(ref_id)
}

fn object_team_packet_from_state(
    ref_id: u32,
    owner: TeamType,
    driver: Option<&DriverHealth>,
) -> Option<ObjectTeamPacket> {
    let driver_type = driver.map_or(RobotType::Grunt, |driver| driver.driver_kind) as i8;
    let drivers = driver
        .map(|driver| {
            driver
                .driver_healths
                .iter()
                .enumerate()
                .map(|(index, health)| ObjectTeamDriverInfo {
                    driver_health: *health as i32,
                    next_attack_time: driver
                        .next_attack_cooldowns
                        .get(index)
                        .copied()
                        .unwrap_or(0.0) as f64,
                })
                .collect()
        })
        .unwrap_or_default();
    Some(ObjectTeamPacket {
        ref_id: i32::try_from(ref_id).ok()?,
        owner: owner as i8,
        driver_type,
        drivers,
    })
}

pub(crate) fn apply_object_health_packet(
    packet: ObjectHealthPacket,
    ref_id: u32,
    stats: &mut ObjectStats,
) -> bool {
    let Ok(packet_ref_id) = u32::try_from(packet.ref_id) else {
        return false;
    };
    if packet_ref_id != ref_id {
        return false;
    }

    stats.health = source_set_health(packet.health, stats.max_health);
    true
}

pub(crate) fn apply_destroy_object_packet(
    packet: &DestroyObjectPacket,
    ref_id: u32,
    stats: &mut ObjectStats,
    cause: Option<bevy::prelude::Mut<DamageCauseTimers>>,
) -> bool {
    let Ok(packet_ref_id) = u32::try_from(packet.ref_id) else {
        return false;
    };
    if packet_ref_id != ref_id {
        return false;
    }

    stats.health = source_set_health(0, stats.max_health);
    if let Some(mut cause) = cause {
        if packet.do_fire_death {
            cause.fire = 1.0;
        }
        if packet.do_missile_death {
            cause.missile = 1.0;
        }
        cause.killer_ref_id = u32::try_from(packet.killer_ref_id).ok();
        cause.killer = cause.killer_ref_id.map_or(0.0, |_| 1.0);
    }
    true
}

fn source_health_from_percent(kind: ObjectKind, health_percent: i32) -> i32 {
    let health_percent = health_percent.clamp(0, 100);
    let max_health = units::object_max_health(kind) as i32;
    health_percent * max_health / 100
}

fn source_set_health(new_health: i32, max_health: f32) -> f32 {
    let max_health = max_health.max(0.0) as i32;
    new_health.clamp(0, max_health) as f32
}

fn source_grenade_amount(grenade_amount: i32) -> u8 {
    if (0..=99).contains(&grenade_amount) {
        grenade_amount as u8
    } else {
        0
    }
}

fn source_location_i32(value: f32) -> Option<i32> {
    if !value.is_finite() || value < i32::MIN as f32 || value > i32::MAX as f32 {
        return None;
    }
    Some(value.round() as i32)
}

fn team_wire(team: TeamType) -> i8 {
    team as i8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::AttackTarget;
    use crate::network_commands::{SourceWaypoint, SourceWaypointMode};
    use crate::original::{
        map::{MapBasics, MapObjectType, MapTile, MapZone},
        objects::{BuildingType, ItemType, RobotType, VehicleType},
        types::PlanetType,
    };
    use crate::render::atlas::MobileSpriteRole;
    use bevy::prelude::{App, IntoScheduleConfigs, Update};

    fn test_map() -> ZMap {
        ZMap {
            basics: MapBasics {
                width: 20,
                height: 20,
                map_name: "objects".to_string(),
                player_count: 2,
                object_count: 2,
                terrain_type: PlanetType::Desert,
                zone_count: 1,
            },
            zones: vec![MapZone {
                x: 0,
                y: 0,
                w: 20,
                h: 20,
            }],
            objects: vec![
                MapObject {
                    x: 2,
                    y: 3,
                    owner: TeamType::Red,
                    object_type: MapObjectType::Building,
                    object_id: BuildingType::FortFront as u8,
                    building_level: 2,
                    extra_links: 0,
                    health_percent: 80,
                },
                MapObject {
                    x: 4,
                    y: 5,
                    owner: TeamType::Blue,
                    object_type: MapObjectType::Robot,
                    object_id: RobotType::Grunt as u8,
                    building_level: 0,
                    extra_links: 0,
                    health_percent: 100,
                },
            ],
            tiles: vec![MapTile { tile: 0 }; 400],
        }
    }

    #[test]
    fn request_objects_relays_source_object_init_packets() {
        let objects = relay_request_object_inits(&test_map()).unwrap();

        assert_eq!(objects.len(), 4);
        assert_eq!(
            objects[0],
            ObjectInitPacket {
                x: 32,
                y: 48,
                ref_id: 0,
                owner: team_wire(TeamType::Red),
                object_type: MapObjectType::Building as u8,
                object_id: BuildingType::FortFront as u8,
                building_level: 2,
                extra_links: 0,
                health: 333332,
            }
        );
        assert_eq!(objects[1].ref_id, 1);
        assert_eq!(objects[2].ref_id, 2);
        assert_eq!(objects[3].ref_id, 3);
        assert_eq!(next_ref_id_after_object_inits(&objects), 4);
    }

    #[test]
    fn object_group_info_relays_robot_group_metadata() {
        let map = test_map();
        let objects = relay_request_object_inits(&map).unwrap();
        let groups = relay_object_group_infos(&map, &objects).unwrap();

        assert_eq!(
            groups,
            vec![
                ObjectGroupInfoPacket {
                    ref_id: 1,
                    leader_ref_id: -1,
                    minion_refs: vec![2, 3],
                },
                ObjectGroupInfoPacket {
                    ref_id: 2,
                    leader_ref_id: 1,
                    minion_refs: Vec::new(),
                },
                ObjectGroupInfoPacket {
                    ref_id: 3,
                    leader_ref_id: 1,
                    minion_refs: Vec::new(),
                },
            ]
        );
    }

    #[test]
    fn startup_object_events_follow_source_per_object_order() {
        let events =
            relay_startup_object_events(&test_map(), &SourceSettingsState::default()).unwrap();
        let labels: Vec<_> = events
            .iter()
            .map(|event| match event {
                SourceObjectEvent::AddNewObject { .. } => "add",
                SourceObjectEvent::ObjectGroupInfo { .. } => "group",
                SourceObjectEvent::UpdateHealth { .. } => "health",
                SourceObjectEvent::BuildingState { .. } => "building",
                SourceObjectEvent::BuildingQueue { .. } => "queue",
                SourceObjectEvent::ObjectGrenadeAmount { .. } => "grenade",
                SourceObjectEvent::ObjectRallyPoints { .. } => "rally",
                _ => "other",
            })
            .collect();

        assert_eq!(
            labels,
            vec![
                "add", "health", "building", "queue", "rally", // fort
                "add", "group", "health", "grenade", // robot leader
                "add", "group", "health", "grenade", // minion 1
                "add", "group", "health", "grenade", // minion 2
            ]
        );
        let SourceObjectEvent::BuildingState { packet, .. } = events[2] else {
            panic!("fort state should follow health");
        };
        assert_eq!(packet.ref_id, 0);
        assert_eq!(packet.state, 2);
        assert_eq!(packet.object_type, MapObjectType::Robot as u8);
        assert_eq!(packet.object_id, RobotType::Grunt as u8);
        let SourceObjectEvent::BuildingQueue { ref packet, .. } = events[3] else {
            panic!("fort queue should follow state");
        };
        assert_eq!(
            packet.units,
            vec![BuildingQueueUnit {
                object_type: MapObjectType::Robot as u8,
                object_id: RobotType::Grunt as u8,
            }]
        );
    }

    #[test]
    fn startup_repair_events_include_state_then_repair_animation() {
        let mut map = test_map();
        map.objects = vec![MapObject {
            x: 2,
            y: 3,
            owner: TeamType::Red,
            object_type: MapObjectType::Building,
            object_id: BuildingType::Repair as u8,
            building_level: 0,
            extra_links: 0,
            health_percent: 100,
        }];
        let events = relay_startup_object_events(&map, &SourceSettingsState::default()).unwrap();

        assert!(matches!(events[0], SourceObjectEvent::AddNewObject { .. }));
        assert!(matches!(events[1], SourceObjectEvent::UpdateHealth { .. }));
        assert!(matches!(events[2], SourceObjectEvent::BuildingState { .. }));
        assert!(matches!(
            events[3],
            SourceObjectEvent::RepairBuildingAnimation { .. }
        ));
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn inbound_building_state_queue_and_rally_replace_client_state() {
        let kind = ObjectKind::Building(BuildingType::RobotFactory);
        let state = BuildingStatePacket {
            ref_id: 7,
            state: 2,
            init_offset: -9.0,
            production_time: 36.0,
            object_type: MapObjectType::Robot as u8,
            object_id: RobotType::Grunt as u8,
        };
        let mut production = apply_building_state_packet(state, 7, kind, None).unwrap();
        assert_eq!(production.status, BuildingProductionStatus::Building);
        assert_eq!(
            production.current,
            Some(ObjectKind::Robot(RobotType::Grunt))
        );
        assert_eq!(production.elapsed, 9.0);
        assert_eq!(production.duration, 36.0);

        assert!(apply_building_queue_packet(
            &BuildingQueuePacket {
                ref_id: 7,
                units: vec![BuildingQueueUnit {
                    object_type: MapObjectType::Robot as u8,
                    object_id: RobotType::Psycho as u8,
                }],
            },
            7,
            &mut production,
        ));
        assert_eq!(
            production.queue,
            VecDeque::from([ObjectKind::Robot(RobotType::Psycho)])
        );

        let rally = rally_points_from_packet(
            &SendRallypointsPacket {
                ref_id: 7,
                waypoints: vec![SourceWaypoint {
                    mode: SourceWaypointMode::Move,
                    ref_id: -1,
                    x: 64,
                    y: 80,
                    attack_to: true,
                    player_given: true,
                }],
            },
            7,
            kind,
        )
        .unwrap();
        assert_eq!(rally.points, vec![Vec2::new(64.0, -80.0)]);
    }

    #[test]
    fn process_group_info_packet_rejects_unknown_refs() {
        let objects = relay_request_object_inits(&test_map()).unwrap();

        assert!(
            process_group_info_packet(
                &ObjectGroupInfoPacket {
                    ref_id: 1,
                    leader_ref_id: -1,
                    minion_refs: vec![2, 3],
                },
                &objects,
            )
            .is_some()
        );
        assert_eq!(
            process_group_info_packet(
                &ObjectGroupInfoPacket {
                    ref_id: 99,
                    leader_ref_id: -1,
                    minion_refs: Vec::new(),
                },
                &objects,
            ),
            None
        );
        assert_eq!(
            process_group_info_packet(
                &ObjectGroupInfoPacket {
                    ref_id: 1,
                    leader_ref_id: 99,
                    minion_refs: Vec::new(),
                },
                &objects,
            ),
            None
        );
        assert_eq!(
            process_group_info_packet(
                &ObjectGroupInfoPacket {
                    ref_id: 1,
                    leader_ref_id: -1,
                    minion_refs: vec![99],
                },
                &objects,
            ),
            None
        );
    }

    #[test]
    fn object_health_updates_relay_source_actual_health() {
        let objects = relay_request_object_inits(&test_map()).unwrap();
        let health_updates = relay_object_health_updates(&objects).unwrap();

        assert_eq!(health_updates.len(), objects.len());
        assert_eq!(
            health_updates[0],
            ObjectHealthPacket {
                ref_id: 0,
                health: 333332,
            }
        );
        assert_eq!(
            health_updates[1],
            ObjectHealthPacket {
                ref_id: 1,
                health: 1081,
            }
        );
    }

    #[test]
    fn process_object_health_packet_rejects_unknown_ref() {
        let objects = relay_request_object_inits(&test_map()).unwrap();

        assert_eq!(
            process_object_health_packet(
                ObjectHealthPacket {
                    ref_id: 1,
                    health: 0,
                },
                &objects,
            ),
            Some(ObjectHealthPacket {
                ref_id: 1,
                health: 0,
            })
        );
        assert_eq!(
            process_object_health_packet(
                ObjectHealthPacket {
                    ref_id: 99,
                    health: 0,
                },
                &objects,
            ),
            None
        );
    }

    #[test]
    fn relay_object_team_update_round_trips_source_packet() {
        let driver = DriverHealth::with_driver_states(
            RobotType::Sniper,
            vec![1081.0, 500.0],
            vec![1.5, 0.0],
        );

        let packet = relay_object_team_update(7, TeamType::Blue, Some(&driver)).unwrap();

        assert_eq!(
            packet,
            ObjectTeamPacket {
                ref_id: 7,
                owner: TeamType::Blue as i8,
                driver_type: RobotType::Sniper as i8,
                drivers: vec![
                    ObjectTeamDriverInfo {
                        driver_health: 1081,
                        next_attack_time: 1.5,
                    },
                    ObjectTeamDriverInfo {
                        driver_health: 500,
                        next_attack_time: 0.0,
                    },
                ],
            }
        );
        assert_eq!(
            relay_object_team_update(8, TeamType::Null, None),
            Some(ObjectTeamPacket {
                ref_id: 8,
                owner: TeamType::Null as i8,
                driver_type: RobotType::Grunt as i8,
                drivers: Vec::new(),
            })
        );
        assert_eq!(
            relay_object_team_update(u32::MAX, TeamType::Red, None),
            None
        );
    }

    #[test]
    fn apply_object_team_packet_validates_ref_owner_and_driver_type() {
        let packet = ObjectTeamPacket {
            ref_id: 7,
            owner: TeamType::Green as i8,
            driver_type: RobotType::Laser as i8,
            drivers: vec![ObjectTeamDriverInfo {
                driver_health: 250,
                next_attack_time: 0.25,
            }],
        };

        let applied = apply_object_team_packet(&packet, 7).unwrap();
        assert_eq!(applied.owner, TeamType::Green);
        let driver = applied.driver.unwrap();
        assert_eq!(driver.driver_kind, RobotType::Laser);
        assert_eq!(driver.driver_healths, vec![250.0]);
        assert_eq!(driver.next_attack_cooldowns, vec![0.25]);

        assert!(apply_object_team_packet(&packet, 8).is_none());
        assert!(
            apply_object_team_packet(
                &ObjectTeamPacket {
                    owner: -1,
                    ..packet.clone()
                },
                7,
            )
            .is_none()
        );
        assert!(
            apply_object_team_packet(
                &ObjectTeamPacket {
                    driver_type: -1,
                    ..packet
                },
                7,
            )
            .is_none()
        );
    }

    #[test]
    fn relay_object_attack_target_round_trips_source_packet() {
        assert_eq!(
            relay_object_attack_target(7, Some(9)),
            Some(AttackObjectPacket {
                ref_id: 7,
                attack_object_ref_id: 9,
            })
        );
        assert_eq!(
            relay_object_attack_target(7, None),
            Some(AttackObjectPacket {
                ref_id: 7,
                attack_object_ref_id: -1,
            })
        );
        assert_eq!(relay_object_attack_target(u32::MAX, Some(9)), None);
        assert_eq!(relay_object_attack_target(7, Some(u32::MAX)), None);
    }

    #[test]
    fn relay_object_empty_waypoints_round_trips_source_packet() {
        assert_eq!(
            relay_object_empty_waypoints(7),
            Some(SendWaypointsPacket {
                ref_id: 7,
                waypoints: Vec::new(),
            })
        );
        assert_eq!(relay_object_empty_waypoints(u32::MAX), None);
    }

    #[test]
    fn relay_object_waypoints_round_trips_remaining_source_route() {
        let waypoints = vec![SourceWaypoint {
            mode: SourceWaypointMode::Move,
            ref_id: -1,
            x: 96,
            y: -112,
            attack_to: true,
            player_given: true,
        }];

        assert_eq!(
            relay_object_waypoints(7, &waypoints),
            Some(SendWaypointsPacket {
                ref_id: 7,
                waypoints,
            })
        );
        assert_eq!(relay_object_waypoints(u32::MAX, &[]), None);
    }

    #[test]
    fn driverless_cleanup_plan_matches_source_target_and_dependent_branches() {
        let target_path = MovementPath::from_typed(
            vec![MovementWaypoint::move_to(Vec2::new(64.0, -80.0))],
            40.0,
        );
        let attacker_path = MovementPath::from_typed(
            vec![
                MovementWaypoint::attack_target(7, Vec2::new(32.0, -48.0), false),
                MovementWaypoint::move_to(Vec2::new(96.0, -112.0)),
            ],
            50.0,
        );
        let waypoint_only_path = MovementPath::from_typed(
            vec![MovementWaypoint::attack_target(
                7,
                Vec2::new(32.0, -48.0),
                false,
            )],
            30.0,
        );
        let unrelated_path = MovementPath::from_typed(
            vec![MovementWaypoint::move_to(Vec2::new(12.0, -18.0))],
            25.0,
        );
        let snapshots = vec![
            DriverlessObjectCleanupSnapshot {
                ref_id: 7,
                world_position: Vec2::new(32.0, -48.0),
                map_position: Vec2::new(32.0, 48.0),
                attack_target_ref_id: Some(99),
                movement_path: Some(target_path),
                is_moving: true,
                has_special_waypoint: false,
            },
            DriverlessObjectCleanupSnapshot {
                ref_id: 4,
                world_position: Vec2::new(16.0, -24.0),
                map_position: Vec2::new(16.0, 24.0),
                attack_target_ref_id: Some(7),
                movement_path: Some(unrelated_path),
                is_moving: false,
                has_special_waypoint: false,
            },
            DriverlessObjectCleanupSnapshot {
                ref_id: 2,
                world_position: Vec2::new(8.0, -10.0),
                map_position: Vec2::new(8.0, 10.0),
                attack_target_ref_id: Some(7),
                movement_path: Some(attacker_path),
                is_moving: true,
                has_special_waypoint: false,
            },
            DriverlessObjectCleanupSnapshot {
                ref_id: 3,
                world_position: Vec2::new(14.0, -20.0),
                map_position: Vec2::new(14.0, 20.0),
                attack_target_ref_id: None,
                movement_path: Some(waypoint_only_path),
                is_moving: true,
                has_special_waypoint: false,
            },
        ];

        let plan = driverless_object_cleanup_plan(7, &snapshots).unwrap();
        assert_eq!(
            plan.target.attack_clear_packet,
            Some(AttackObjectPacket {
                ref_id: 7,
                attack_object_ref_id: -1,
            })
        );
        assert_eq!(
            plan.target.waypoint_packet,
            Some(SendWaypointsPacket {
                ref_id: 7,
                waypoints: Vec::new(),
            })
        );
        assert_eq!(
            plan.target.stop_location_packet,
            Some(ObjectLocationPacket {
                ref_id: 7,
                x: 32,
                y: 48,
                dx: 0.0,
                dy: 0.0,
            })
        );

        assert_eq!(
            plan.dependents
                .iter()
                .map(|event| event.ref_id)
                .collect::<Vec<_>>(),
            vec![2, 3, 4]
        );
        assert!(plan.dependents[0].attack_clear_packet.is_some());
        assert_eq!(
            plan.dependents[0]
                .waypoint_packet
                .as_ref()
                .unwrap()
                .waypoints,
            vec![SourceWaypoint {
                mode: SourceWaypointMode::Move,
                ref_id: -1,
                x: 96,
                y: -112,
                attack_to: false,
                player_given: false,
            }]
        );
        assert!(plan.dependents[0].stop_location_packet.is_some());
        assert!(plan.dependents[1].attack_clear_packet.is_none());
        assert!(
            plan.dependents[1]
                .waypoint_packet
                .as_ref()
                .unwrap()
                .waypoints
                .is_empty()
        );
        assert!(plan.dependents[1].stop_location_packet.is_some());
        assert!(plan.dependents[2].attack_clear_packet.is_some());
        assert!(plan.dependents[2].waypoint_packet.is_none());
        assert!(plan.dependents[2].stop_location_packet.is_none());
    }

    #[test]
    fn driverless_cleanup_does_not_relay_location_for_stationary_attack_waypoints() {
        let attack_path = MovementPath::from_typed(
            vec![MovementWaypoint::attack_target(
                7,
                Vec2::new(32.0, -48.0),
                false,
            )],
            40.0,
        );
        let snapshots = vec![
            DriverlessObjectCleanupSnapshot {
                ref_id: 7,
                world_position: Vec2::new(32.0, -48.0),
                map_position: Vec2::new(32.0, 48.0),
                attack_target_ref_id: None,
                movement_path: Some(MovementPath::from_typed(
                    vec![MovementWaypoint::move_to(Vec2::new(32.0, -48.0))],
                    40.0,
                )),
                is_moving: false,
                has_special_waypoint: false,
            },
            DriverlessObjectCleanupSnapshot {
                ref_id: 2,
                world_position: Vec2::new(24.0, -40.0),
                map_position: Vec2::new(24.0, 40.0),
                attack_target_ref_id: Some(7),
                movement_path: Some(attack_path),
                is_moving: false,
                has_special_waypoint: false,
            },
        ];

        let plan = driverless_object_cleanup_plan(7, &snapshots).unwrap();
        assert!(plan.target.waypoint_packet.is_some());
        assert!(plan.target.stop_location_packet.is_none());
        assert_eq!(plan.dependents.len(), 1);
        assert!(plan.dependents[0].waypoint_packet.is_some());
        assert!(plan.dependents[0].stop_location_packet.is_none());
    }

    #[test]
    fn driverless_cleanup_only_stops_target_inside_source_waypoint_clear_branch() {
        let snapshots = vec![DriverlessObjectCleanupSnapshot {
            ref_id: 7,
            world_position: Vec2::new(32.0, -48.0),
            map_position: Vec2::new(32.0, 48.0),
            attack_target_ref_id: None,
            movement_path: None,
            is_moving: true,
            has_special_waypoint: false,
        }];

        let plan = driverless_object_cleanup_plan(7, &snapshots).unwrap();

        assert!(plan.target.waypoint_packet.is_none());
        assert!(plan.target.stop_location_packet.is_none());
    }

    #[test]
    fn relay_object_location_round_trips_source_packet() {
        assert_eq!(
            relay_object_location(7, Vec2::new(32.4, 47.6), Vec2::ZERO),
            Some(ObjectLocationPacket {
                ref_id: 7,
                x: 32,
                y: 48,
                dx: 0.0,
                dy: 0.0,
            })
        );
        assert_eq!(
            relay_object_location(u32::MAX, Vec2::new(32.0, 48.0), Vec2::ZERO),
            None
        );
        assert_eq!(
            relay_object_location(7, Vec2::new(f32::NAN, 48.0), Vec2::ZERO),
            None
        );
    }

    #[test]
    fn world_location_packet_preserves_source_axis_conversion_and_layer_offset() {
        let packet = relay_object_location(7, Vec2::new(32.0, 48.0), Vec2::new(2.0, 3.0)).unwrap();

        assert_eq!(
            packet,
            ObjectLocationPacket {
                ref_id: 7,
                x: 32,
                y: 48,
                dx: 2.0,
                dy: 3.0,
            }
        );
        assert_eq!(
            apply_object_world_location_packet(&packet, 7, Vec2::new(4.0, -6.0)),
            Some((Vec2::new(36.0, -54.0), Vec2::new(2.0, -3.0)))
        );
    }

    #[test]
    fn object_location_packet_queue_applies_position_velocity_direction_and_offsets() {
        let mut app = App::new();
        app.insert_resource(ObjectLocationPacketQueue {
            pending: vec![
                ObjectLocationPacket {
                    ref_id: -1,
                    x: 999,
                    y: 999,
                    dx: 8.0,
                    dy: 8.0,
                },
                ObjectLocationPacket {
                    ref_id: 7,
                    x: 40,
                    y: 60,
                    dx: 0.0,
                    dy: 2.0,
                },
                ObjectLocationPacket {
                    ref_id: 8,
                    x: 50,
                    y: 70,
                    dx: 0.0,
                    dy: 2.0,
                },
            ],
        })
        .init_resource::<ObjectDestroyPacketQueue>()
        .init_resource::<Time>()
        .add_systems(
            Update,
            (
                process_object_location_packet_queue,
                smooth_object_locations,
            )
                .chain(),
        );

        let root = app
            .world_mut()
            .spawn((
                GameObjectEntity {
                    ref_id: 7,
                    kind: ObjectKind::Robot(RobotType::Grunt),
                },
                ObjectLayerRef(7),
                Transform::from_xyz(100.0, -200.0, 5.0),
                SourceObjectLocation {
                    map_position: Vec2::new(80.0, 180.0),
                    map_velocity: Vec2::new(1.0, 0.0),
                    map_remainder: Vec2::ZERO,
                    world_anchor: Vec2::new(20.0, -20.0),
                },
                MovementVelocity(Vec2::new(1.0, 0.0)),
                MobileSpriteLayer {
                    kind: ObjectKind::Robot(RobotType::Grunt),
                    team: TeamType::Red,
                    role: MobileSpriteRole::Robot,
                    rotation: 0,
                    frame: 0,
                    elapsed: 0.0,
                },
            ))
            .id();
        let overlay = app
            .world_mut()
            .spawn((
                ObjectLayerRef(7),
                Transform::from_xyz(104.0, -206.0, 5.1),
                MovementVelocity(Vec2::new(1.0, 0.0)),
            ))
            .id();
        let same_velocity = app
            .world_mut()
            .spawn((
                GameObjectEntity {
                    ref_id: 8,
                    kind: ObjectKind::Robot(RobotType::Grunt),
                },
                ObjectLayerRef(8),
                Transform::from_xyz(10.0, -20.0, 5.0),
                SourceObjectLocation {
                    map_position: Vec2::new(0.0, 10.0),
                    map_velocity: Vec2::new(0.0, 2.0),
                    map_remainder: Vec2::ZERO,
                    world_anchor: Vec2::new(10.0, -10.0),
                },
                MovementVelocity(Vec2::new(0.0, -2.0)),
                MobileSpriteLayer {
                    kind: ObjectKind::Robot(RobotType::Grunt),
                    team: TeamType::Red,
                    role: MobileSpriteRole::Robot,
                    rotation: 135,
                    frame: 0,
                    elapsed: 0.0,
                },
            ))
            .id();

        app.update();

        {
            let world = app.world();
            assert_eq!(
                world.get::<Transform>(root).unwrap().translation.truncate(),
                Vec2::new(60.0, -80.0)
            );
            assert_eq!(
                world
                    .get::<Transform>(overlay)
                    .unwrap()
                    .translation
                    .truncate(),
                Vec2::new(64.0, -86.0)
            );
            assert_eq!(
                world.get::<MovementVelocity>(root).unwrap().0,
                Vec2::new(0.0, -2.0)
            );
            assert_eq!(
                world.get::<MovementVelocity>(overlay).unwrap().0,
                Vec2::new(0.0, -2.0)
            );
            assert_eq!(
                world.get::<MobileSpriteLayer>(root).unwrap().rotation,
                crate::rotation_for_direction(
                    crate::direction_index_from_delta(Vec2::new(0.0, -2.0)).unwrap()
                )
            );
            assert_eq!(
                world
                    .get::<Transform>(same_velocity)
                    .unwrap()
                    .translation
                    .truncate(),
                Vec2::new(60.0, -80.0)
            );
            assert_eq!(
                world
                    .get::<MobileSpriteLayer>(same_velocity)
                    .unwrap()
                    .rotation,
                135
            );
            assert!(
                world
                    .resource::<ObjectLocationPacketQueue>()
                    .pending
                    .is_empty()
            );
            assert_eq!(
                world.get::<SourceObjectLocation>(root).unwrap(),
                &SourceObjectLocation {
                    map_position: Vec2::new(40.0, 60.0),
                    map_velocity: Vec2::new(0.0, 2.0),
                    map_remainder: Vec2::ZERO,
                    world_anchor: Vec2::new(20.0, -20.0),
                }
            );
            assert!(world.get::<SourceLocationInterpolation>(root).is_some());
        }

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(0.75));
        app.update();

        let world = app.world();
        assert_eq!(
            world.get::<Transform>(root).unwrap().translation.truncate(),
            Vec2::new(60.0, -81.0)
        );
        assert_eq!(
            world
                .get::<Transform>(overlay)
                .unwrap()
                .translation
                .truncate(),
            Vec2::new(64.0, -87.0)
        );
        assert_eq!(
            world
                .get::<SourceObjectLocation>(root)
                .unwrap()
                .map_position,
            Vec2::new(40.0, 61.0)
        );
    }

    #[test]
    fn object_location_packet_queue_rejects_ref_with_pending_destroy() {
        let mut app = App::new();
        app.insert_resource(ObjectLocationPacketQueue {
            pending: vec![ObjectLocationPacket {
                ref_id: 9,
                x: 40,
                y: 60,
                dx: 2.0,
                dy: 3.0,
            }],
        })
        .insert_resource(ObjectDestroyPacketQueue {
            pending: vec![DestroyObjectPacket {
                ref_id: 9,
                killer_ref_id: -1,
                destroy_object: true,
                do_fire_death: false,
                do_missile_death: false,
                fire_missiles: Vec::new(),
            }],
        })
        .add_systems(Update, process_object_location_packet_queue);
        let entity = app
            .world_mut()
            .spawn((
                GameObjectEntity {
                    ref_id: 9,
                    kind: ObjectKind::Robot(RobotType::Grunt),
                },
                ObjectLayerRef(9),
                Transform::from_xyz(100.0, -200.0, 5.0),
                SourceObjectLocation {
                    map_position: Vec2::new(92.0, 192.0),
                    map_velocity: Vec2::ZERO,
                    map_remainder: Vec2::ZERO,
                    world_anchor: Vec2::new(8.0, -8.0),
                },
                MovementVelocity::default(),
            ))
            .id();

        app.update();

        let world = app.world();
        assert_eq!(
            world
                .get::<Transform>(entity)
                .unwrap()
                .translation
                .truncate(),
            Vec2::new(100.0, -200.0)
        );
        assert_eq!(
            world
                .get::<SourceObjectLocation>(entity)
                .unwrap()
                .map_position,
            Vec2::new(92.0, 192.0)
        );
        assert!(world.get::<SourceLocationInterpolation>(entity).is_none());
    }

    #[test]
    fn accepted_empty_waypoint_relay_stops_leader_and_live_minions_in_ref_order() {
        let snapshots = vec![
            EmptyWaypointObjectSnapshot {
                ref_id: 10,
                map_position: Vec2::new(100.0, 200.0),
                group_leader_ref_id: Some(10),
                destroyed: false,
            },
            EmptyWaypointObjectSnapshot {
                ref_id: 12,
                map_position: Vec2::new(90.0, 180.0),
                group_leader_ref_id: Some(10),
                destroyed: false,
            },
            EmptyWaypointObjectSnapshot {
                ref_id: 11,
                map_position: Vec2::new(80.0, 160.0),
                group_leader_ref_id: Some(10),
                destroyed: false,
            },
            EmptyWaypointObjectSnapshot {
                ref_id: 13,
                map_position: Vec2::new(70.0, 140.0),
                group_leader_ref_id: Some(10),
                destroyed: true,
            },
            EmptyWaypointObjectSnapshot {
                ref_id: 20,
                map_position: Vec2::new(60.0, 120.0),
                group_leader_ref_id: Some(20),
                destroyed: false,
            },
        ];

        assert_eq!(
            accepted_empty_waypoint_location_packets(10, &snapshots).unwrap(),
            vec![
                ObjectLocationPacket {
                    ref_id: 10,
                    x: 100,
                    y: 200,
                    dx: 0.0,
                    dy: 0.0,
                },
                ObjectLocationPacket {
                    ref_id: 11,
                    x: 80,
                    y: 160,
                    dx: 0.0,
                    dy: 0.0,
                },
                ObjectLocationPacket {
                    ref_id: 12,
                    x: 90,
                    y: 180,
                    dx: 0.0,
                    dy: 0.0,
                },
            ]
        );
        assert!(accepted_empty_waypoint_location_packets(11, &snapshots).is_none());
    }

    #[test]
    fn accepted_empty_waypoint_runtime_stops_group_without_collapsing_layers() {
        let mut app = App::new();
        app.init_resource::<ObjectLocationPacketQueue>()
            .init_resource::<ObjectDestroyPacketQueue>()
            .init_resource::<Time>()
            .add_systems(
                Update,
                (
                    process_accepted_empty_waypoint_commands,
                    process_object_location_packet_queue,
                    smooth_object_locations,
                )
                    .chain(),
            );

        let leader = app
            .world_mut()
            .spawn((
                GameObjectEntity {
                    ref_id: 10,
                    kind: ObjectKind::Robot(RobotType::Grunt),
                },
                ObjectLayerRef(10),
                Transform::from_xyz(100.0, -200.0, 5.0),
                SourceObjectLocation {
                    map_position: Vec2::new(92.0, 192.0),
                    map_velocity: Vec2::new(4.0, 3.0),
                    map_remainder: Vec2::ZERO,
                    world_anchor: Vec2::new(8.0, -8.0),
                },
                RobotGroup {
                    leader_ref_id: 10,
                    member_index: 0,
                },
                ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100),
                MovementVelocity(Vec2::new(4.0, -3.0)),
                AttackTarget {
                    ref_id: 99,
                    cooldown: 0.25,
                    player_given: true,
                },
                JustLeftCannon,
                AcceptedEmptyWaypointCommand,
            ))
            .id();
        let leader_overlay = app
            .world_mut()
            .spawn((
                ObjectLayerRef(10),
                Transform::from_xyz(104.0, -206.0, 5.1),
                MovementVelocity(Vec2::new(4.0, -3.0)),
                AttackTarget {
                    ref_id: 99,
                    cooldown: 0.25,
                    player_given: true,
                },
                JustLeftCannon,
            ))
            .id();

        let minion = app
            .world_mut()
            .spawn((
                GameObjectEntity {
                    ref_id: 11,
                    kind: ObjectKind::Robot(RobotType::Grunt),
                },
                ObjectLayerRef(11),
                Transform::from_xyz(80.0, -160.0, 5.0),
                SourceObjectLocation {
                    map_position: Vec2::new(72.0, 152.0),
                    map_velocity: Vec2::new(-2.0, -1.0),
                    map_remainder: Vec2::ZERO,
                    world_anchor: Vec2::new(8.0, -8.0),
                },
                RobotGroup {
                    leader_ref_id: 10,
                    member_index: 0,
                },
                ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100),
                MovementVelocity(Vec2::new(-2.0, 1.0)),
                AttackTarget {
                    ref_id: 99,
                    cooldown: 0.5,
                    player_given: false,
                },
                JustLeftCannon,
                AcceptedEmptyWaypointCommand,
            ))
            .id();
        let minion_overlay = app
            .world_mut()
            .spawn((
                ObjectLayerRef(11),
                Transform::from_xyz(77.0, -155.0, 5.1),
                MovementVelocity(Vec2::new(-2.0, 1.0)),
                AttackTarget {
                    ref_id: 99,
                    cooldown: 0.5,
                    player_given: false,
                },
                JustLeftCannon,
            ))
            .id();

        app.update();

        for (entity, expected_position) in [
            (leader, Vec2::new(100.0, -200.0)),
            (leader_overlay, Vec2::new(104.0, -206.0)),
            (minion, Vec2::new(80.0, -160.0)),
            (minion_overlay, Vec2::new(77.0, -155.0)),
        ] {
            let world = app.world();
            assert_eq!(
                world
                    .get::<Transform>(entity)
                    .unwrap()
                    .translation
                    .truncate(),
                expected_position
            );
            assert_eq!(world.get::<MovementVelocity>(entity).unwrap().0, Vec2::ZERO);
            assert!(world.get::<MovementPath>(entity).is_none());
            assert!(world.get::<AttackTarget>(entity).is_some());
            assert!(world.get::<JustLeftCannon>(entity).is_none());
            assert!(world.get::<AcceptedEmptyWaypointCommand>(entity).is_none());
        }
    }

    #[test]
    fn stale_empty_waypoint_marker_does_not_override_later_special_command() {
        let mut app = App::new();
        app.init_resource::<ObjectLocationPacketQueue>()
            .add_systems(Update, process_accepted_empty_waypoint_commands);

        let path = MovementPath::new(vec![Vec2::new(140.0, -220.0)], 40.0);
        let root = app
            .world_mut()
            .spawn((
                GameObjectEntity {
                    ref_id: 10,
                    kind: ObjectKind::Robot(RobotType::Grunt),
                },
                ObjectLayerRef(10),
                Transform::from_xyz(100.0, -200.0, 5.0),
                SourceObjectLocation {
                    map_position: Vec2::new(92.0, 192.0),
                    map_velocity: Vec2::new(4.0, 3.0),
                    map_remainder: Vec2::ZERO,
                    world_anchor: Vec2::new(8.0, -8.0),
                },
                RobotGroup {
                    leader_ref_id: 10,
                    member_index: 0,
                },
                ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100),
                MovementVelocity(Vec2::new(4.0, -3.0)),
                path.clone(),
                PickupGrenadesTarget { ref_id: 99 },
                AcceptedEmptyWaypointCommand,
            ))
            .id();
        let overlay = app
            .world_mut()
            .spawn((
                ObjectLayerRef(10),
                Transform::from_xyz(104.0, -206.0, 5.1),
                MovementVelocity(Vec2::new(4.0, -3.0)),
                path,
                PickupGrenadesTarget { ref_id: 99 },
                AcceptedEmptyWaypointCommand,
            ))
            .id();

        app.update();

        for entity in [root, overlay] {
            let world = app.world();
            assert_eq!(
                world.get::<MovementVelocity>(entity).unwrap().0,
                Vec2::new(4.0, -3.0)
            );
            assert!(world.get::<MovementPath>(entity).is_some());
            assert!(world.get::<PickupGrenadesTarget>(entity).is_some());
            assert!(world.get::<AcceptedEmptyWaypointCommand>(entity).is_none());
        }
    }

    #[test]
    fn accepted_empty_waypoint_consumes_marker_for_destroyed_leader() {
        let mut app = App::new();
        app.init_resource::<ObjectLocationPacketQueue>()
            .add_systems(Update, process_accepted_empty_waypoint_commands);
        let mut stats = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        stats.health = 0.0;
        let root = app
            .world_mut()
            .spawn((
                GameObjectEntity {
                    ref_id: 10,
                    kind: ObjectKind::Robot(RobotType::Grunt),
                },
                ObjectLayerRef(10),
                Transform::from_xyz(100.0, -200.0, 5.0),
                SourceObjectLocation {
                    map_position: Vec2::new(92.0, 192.0),
                    map_velocity: Vec2::ZERO,
                    map_remainder: Vec2::ZERO,
                    world_anchor: Vec2::new(8.0, -8.0),
                },
                RobotGroup {
                    leader_ref_id: 10,
                    member_index: 0,
                },
                stats,
                AcceptedEmptyWaypointCommand,
            ))
            .id();
        let overlay = app
            .world_mut()
            .spawn((ObjectLayerRef(10), AcceptedEmptyWaypointCommand))
            .id();

        app.update();

        let world = app.world();
        assert!(world.get::<AcceptedEmptyWaypointCommand>(root).is_none());
        assert!(world.get::<AcceptedEmptyWaypointCommand>(overlay).is_none());
        assert!(
            world
                .resource::<ObjectLocationPacketQueue>()
                .pending
                .is_empty()
        );
    }

    #[test]
    fn apply_object_attack_packet_validates_object_and_target_refs() {
        let packet = AttackObjectPacket {
            ref_id: 7,
            attack_object_ref_id: 9,
        };

        assert_eq!(
            apply_object_attack_packet(&packet, 7, [7_u32, 9_u32])
                .unwrap()
                .target_ref_id,
            Some(9)
        );
        assert_eq!(
            apply_object_attack_packet(
                &AttackObjectPacket {
                    ref_id: 7,
                    attack_object_ref_id: -1,
                },
                7,
                [7_u32, 9_u32],
            )
            .unwrap()
            .target_ref_id,
            None
        );
        assert_eq!(
            apply_object_attack_packet(
                &AttackObjectPacket {
                    ref_id: 7,
                    attack_object_ref_id: 99,
                },
                7,
                [7_u32, 9_u32],
            )
            .unwrap()
            .target_ref_id,
            None
        );
        assert!(apply_object_attack_packet(&packet, 8, [7_u32, 9_u32]).is_none());
        assert!(
            apply_object_attack_packet(
                &AttackObjectPacket {
                    ref_id: -1,
                    attack_object_ref_id: 9,
                },
                7,
                [7_u32, 9_u32],
            )
            .is_none()
        );
    }

    #[test]
    fn apply_empty_object_waypoints_packet_validates_ref_and_empty_list() {
        let packet = SendWaypointsPacket {
            ref_id: 7,
            waypoints: Vec::new(),
        };
        assert!(apply_empty_object_waypoints_packet(&packet, 7));
        assert!(!apply_empty_object_waypoints_packet(&packet, 8));
        assert!(!apply_empty_object_waypoints_packet(
            &SendWaypointsPacket {
                waypoints: vec![SourceWaypoint {
                    mode: SourceWaypointMode::Move,
                    ref_id: -1,
                    x: 1,
                    y: 2,
                    attack_to: true,
                    player_given: true,
                }],
                ..packet
            },
            7,
        ));
        assert!(!apply_empty_object_waypoints_packet(
            &SendWaypointsPacket {
                ref_id: -1,
                waypoints: Vec::new(),
            },
            7,
        ));
    }

    #[test]
    fn movement_path_from_waypoint_packet_preserves_runtime_state_and_layer_offset() {
        let current = MovementPath::from_typed(
            vec![MovementWaypoint::attack_target(
                7,
                Vec2::new(32.0, -48.0),
                false,
            )],
            45.0,
        )
        .with_run_attempt();
        let packet = SendWaypointsPacket {
            ref_id: 2,
            waypoints: vec![SourceWaypoint {
                mode: SourceWaypointMode::Move,
                ref_id: -1,
                x: 96,
                y: -112,
                attack_to: true,
                player_given: true,
            }],
        };

        let next = movement_path_from_object_waypoints_packet(
            &packet,
            2,
            Some(&current),
            Vec2::new(3.0, 4.0),
        )
        .unwrap()
        .unwrap();
        assert_eq!(next.speed, 45.0);
        assert!(next.attempt_run);
        assert_eq!(next.waypoints, vec![Vec2::new(99.0, -108.0)]);
        assert_eq!(next.typed_waypoints[0].mode, MovementWaypointMode::Move);
        assert!(next.typed_waypoints[0].attack_to);
        assert!(next.typed_waypoints[0].player_given);

        assert!(
            movement_path_from_object_waypoints_packet(
                &SendWaypointsPacket {
                    ref_id: 2,
                    waypoints: Vec::new(),
                },
                2,
                None,
                Vec2::ZERO,
            )
            .unwrap()
            .is_none()
        );
        assert!(
            movement_path_from_object_waypoints_packet(&packet, 3, Some(&current), Vec2::ZERO,)
                .is_none()
        );
    }

    #[test]
    fn apply_object_location_packet_validates_ref_and_returns_position_velocity() {
        let packet = ObjectLocationPacket {
            ref_id: 7,
            x: 32,
            y: 48,
            dx: 0.0,
            dy: 0.0,
        };

        assert_eq!(
            apply_object_location_packet(&packet, 7),
            Some((Vec2::new(32.0, 48.0), Vec2::ZERO))
        );
        assert_eq!(apply_object_location_packet(&packet, 8), None);
        assert_eq!(
            apply_object_location_packet(
                &ObjectLocationPacket {
                    ref_id: -1,
                    ..packet
                },
                7,
            ),
            None
        );
    }

    #[test]
    fn relay_object_grenade_amount_round_trips_source_packet() {
        assert_eq!(
            relay_object_grenade_amount(7, 3),
            Some(ObjectGrenadeAmountPacket {
                ref_id: 7,
                grenade_amount: 3,
            })
        );
        assert_eq!(relay_object_grenade_amount(u32::MAX, 3), None);
    }

    #[test]
    fn apply_object_grenade_amount_packet_validates_ref_and_clamps_like_robot() {
        let mut inventory = GrenadeInventory { amount: 4 };

        assert!(apply_object_grenade_amount_packet(
            &ObjectGrenadeAmountPacket {
                ref_id: 7,
                grenade_amount: 8,
            },
            7,
            &mut inventory,
        ));
        assert_eq!(inventory.amount, 8);

        assert!(apply_object_grenade_amount_packet(
            &ObjectGrenadeAmountPacket {
                ref_id: 7,
                grenade_amount: -1,
            },
            7,
            &mut inventory,
        ));
        assert_eq!(inventory.amount, 0);

        inventory.amount = 4;
        assert!(apply_object_grenade_amount_packet(
            &ObjectGrenadeAmountPacket {
                ref_id: 7,
                grenade_amount: 100,
            },
            7,
            &mut inventory,
        ));
        assert_eq!(inventory.amount, 0);

        inventory.amount = 4;
        assert!(!apply_object_grenade_amount_packet(
            &ObjectGrenadeAmountPacket {
                ref_id: 8,
                grenade_amount: 1,
            },
            7,
            &mut inventory,
        ));
        assert_eq!(inventory.amount, 4);
    }

    #[test]
    fn relay_pickup_grenade_animation_round_trips_source_packet() {
        assert_eq!(
            relay_pickup_grenade_animation(7),
            Some(PickupGrenadeAnimationPacket { ref_id: 7 })
        );
        assert_eq!(relay_pickup_grenade_animation(u32::MAX), None);
    }

    #[test]
    fn apply_pickup_grenade_animation_packet_matches_source_guards() {
        let packet = PickupGrenadeAnimationPacket { ref_id: 7 };

        assert!(apply_pickup_grenade_animation_packet(
            &packet, 7, true, false
        ));
        assert!(!apply_pickup_grenade_animation_packet(
            &packet, 8, true, false
        ));
        assert!(!apply_pickup_grenade_animation_packet(
            &PickupGrenadeAnimationPacket { ref_id: -1 },
            7,
            true,
            false,
        ));
        assert!(!apply_pickup_grenade_animation_packet(
            &packet, 7, false, false
        ));
        assert!(!apply_pickup_grenade_animation_packet(
            &packet, 7, true, true
        ));
    }

    #[test]
    fn relay_driver_hit_effect_round_trips_source_packet() {
        let mut queue = DriverHitEffectPacketQueue::default();

        assert!(relay_driver_hit_effect(&mut queue, 7));
        assert_eq!(queue.pending, vec![DriverHitEffectPacket { ref_id: 7 }]);

        assert!(!relay_driver_hit_effect(&mut queue, u32::MAX));
        assert_eq!(queue.pending, vec![DriverHitEffectPacket { ref_id: 7 }]);
    }

    #[test]
    fn relay_vehicle_lid_state_round_trips_source_packet() {
        let mut queue = VehicleLidPacketQueue::default();

        assert!(relay_vehicle_lid_state(&mut queue, 7, true));
        assert_eq!(
            queue.pending,
            vec![SetLidOpenPacket {
                ref_id: 7,
                lid_open: true,
            }]
        );

        assert!(relay_vehicle_lid_state(&mut queue, 7, false));
        assert_eq!(
            queue.pending,
            vec![
                SetLidOpenPacket {
                    ref_id: 7,
                    lid_open: true,
                },
                SetLidOpenPacket {
                    ref_id: 7,
                    lid_open: false,
                },
            ]
        );

        assert!(!relay_vehicle_lid_state(&mut queue, u32::MAX, true));
        assert_eq!(queue.pending.len(), 2);
    }

    #[test]
    fn relay_eject_vehicle_command_round_trips_source_packet() {
        let mut queue = EjectVehiclePacketQueue::default();

        assert!(relay_eject_vehicle_command(&mut queue, 7));
        assert_eq!(queue.pending, vec![EjectVehiclePacket { ref_id: 7 }]);

        assert!(!relay_eject_vehicle_command(&mut queue, u32::MAX));
        assert_eq!(queue.pending.len(), 1);
    }

    #[test]
    fn relay_crane_anim_state_round_trips_source_packet() {
        let mut queue = CraneAnimPacketQueue::default();

        assert!(relay_crane_anim_state(&mut queue, 7, 9, true));
        assert_eq!(
            queue.pending,
            vec![CraneAnimPacket {
                ref_id: 7,
                repair_ref_id: 9,
                on: true,
            }]
        );

        assert!(relay_crane_anim_state(&mut queue, 7, 9, false));
        assert_eq!(
            queue.pending.last().copied(),
            Some(CraneAnimPacket {
                ref_id: 7,
                repair_ref_id: 9,
                on: false,
            })
        );

        assert!(!relay_crane_anim_state(&mut queue, u32::MAX, 9, true));
        assert_eq!(queue.pending.len(), 2);
    }

    #[test]
    fn relay_repair_building_anim_state_round_trips_source_packet() {
        let mut queue = RepairBuildingAnimPacketQueue::default();

        assert!(relay_repair_building_anim_state(
            &mut queue, 7, true, 2.5, true
        ));
        assert_eq!(
            queue.pending,
            vec![RepairBuildingAnimPacket {
                ref_id: 7,
                on: true,
                remaining_time: 2.5,
                play_sound: true,
            }]
        );

        assert!(relay_repair_building_anim_state(
            &mut queue, 7, false, -1.0, true
        ));
        assert_eq!(
            queue.pending.last().copied(),
            Some(RepairBuildingAnimPacket {
                ref_id: 7,
                on: false,
                remaining_time: 0.0,
                play_sound: true,
            })
        );

        assert!(!relay_repair_building_anim_state(
            &mut queue,
            u32::MAX,
            true,
            2.5,
            true
        ));
        assert_eq!(queue.pending.len(), 2);
    }

    #[test]
    fn apply_vehicle_lid_packet_validates_ref_and_sets_lid_state() {
        let mut lid = VehicleLidState::closed();
        lid.closing = true;
        lid.close_delay = 0.4;

        assert!(apply_vehicle_lid_packet(
            &SetLidOpenPacket {
                ref_id: 7,
                lid_open: true,
            },
            7,
            &mut lid,
        ));
        assert!(lid.open);
        assert!(!lid.closing);
        assert_eq!(lid.close_delay, 0.0);

        assert!(apply_vehicle_lid_packet(
            &SetLidOpenPacket {
                ref_id: 7,
                lid_open: false,
            },
            7,
            &mut lid,
        ));
        assert!(!lid.open);

        let unchanged = lid;
        assert!(!apply_vehicle_lid_packet(
            &SetLidOpenPacket {
                ref_id: 8,
                lid_open: true,
            },
            7,
            &mut lid,
        ));
        assert_eq!(lid, unchanged);
        assert!(!apply_vehicle_lid_packet(
            &SetLidOpenPacket {
                ref_id: -1,
                lid_open: true,
            },
            7,
            &mut lid,
        ));
        assert_eq!(lid, unchanged);
    }

    #[test]
    fn apply_eject_vehicle_packet_matches_source_server_gates() {
        let live_apc = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Apc), 100);
        let destroyed_apc = ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Apc), 0);
        let packet = EjectVehiclePacket { ref_id: 7 };

        assert!(apply_eject_vehicle_packet(
            &packet,
            7,
            TeamType::Red,
            TeamType::Red,
            ObjectKind::Vehicle(VehicleType::Apc),
            live_apc,
        ));
        assert!(!apply_eject_vehicle_packet(
            &packet,
            8,
            TeamType::Red,
            TeamType::Red,
            ObjectKind::Vehicle(VehicleType::Apc),
            live_apc,
        ));
        assert!(!apply_eject_vehicle_packet(
            &packet,
            7,
            TeamType::Null,
            TeamType::Red,
            ObjectKind::Vehicle(VehicleType::Apc),
            live_apc,
        ));
        assert!(!apply_eject_vehicle_packet(
            &packet,
            7,
            TeamType::Blue,
            TeamType::Red,
            ObjectKind::Vehicle(VehicleType::Apc),
            live_apc,
        ));
        assert!(!apply_eject_vehicle_packet(
            &packet,
            7,
            TeamType::Red,
            TeamType::Red,
            ObjectKind::Vehicle(VehicleType::Jeep),
            ObjectStats::from_kind(ObjectKind::Vehicle(VehicleType::Jeep), 100),
        ));
        assert!(!apply_eject_vehicle_packet(
            &packet,
            7,
            TeamType::Red,
            TeamType::Red,
            ObjectKind::Vehicle(VehicleType::Apc),
            destroyed_apc,
        ));
    }

    #[test]
    fn apply_crane_anim_packet_matches_source_visual_event_shape() {
        let target = CraneConcoVisualTarget {
            ref_id: 9,
            top_left_map: bevy::prelude::Vec2::new(64.0, 96.0),
            size: bevy::prelude::Vec2::new(32.0, 32.0),
            is_bridge: false,
        };

        assert_eq!(
            apply_crane_anim_packet(
                &CraneAnimPacket {
                    ref_id: 7,
                    repair_ref_id: 9,
                    on: true,
                },
                Some(target),
            ),
            Some(Some(target))
        );
        assert_eq!(
            apply_crane_anim_packet(
                &CraneAnimPacket {
                    ref_id: 7,
                    repair_ref_id: 9,
                    on: false,
                },
                Some(target),
            ),
            Some(None)
        );
        assert_eq!(
            apply_crane_anim_packet(
                &CraneAnimPacket {
                    ref_id: 7,
                    repair_ref_id: 9,
                    on: true,
                },
                None,
            ),
            None
        );
        assert_eq!(
            apply_crane_anim_packet(
                &CraneAnimPacket {
                    ref_id: -1,
                    repair_ref_id: 9,
                    on: true,
                },
                Some(target),
            ),
            None
        );
    }

    #[test]
    fn apply_repair_building_anim_packet_matches_source_event_shape() {
        let start = RepairBuildingAnimPacket {
            ref_id: 7,
            on: true,
            remaining_time: 2.5,
            play_sound: true,
        };
        assert_eq!(
            apply_repair_building_anim_packet(
                &start,
                7,
                ObjectKind::Building(BuildingType::Repair)
            ),
            Some(RepairBuildingAnimApply {
                state: Some(RepairBuildingAnimState {
                    remaining_time: 2.5,
                }),
                play_sound: true,
            })
        );

        let stop = RepairBuildingAnimPacket {
            ref_id: 7,
            on: false,
            remaining_time: 0.0,
            play_sound: true,
        };
        assert_eq!(
            apply_repair_building_anim_packet(&stop, 7, ObjectKind::Building(BuildingType::Repair)),
            Some(RepairBuildingAnimApply {
                state: None,
                play_sound: true,
            })
        );
        assert_eq!(
            apply_repair_building_anim_packet(
                &start,
                8,
                ObjectKind::Building(BuildingType::Repair)
            ),
            None
        );
        assert_eq!(
            apply_repair_building_anim_packet(
                &start,
                7,
                ObjectKind::Building(BuildingType::RobotFactory)
            ),
            None
        );
        assert_eq!(
            apply_repair_building_anim_packet(
                &RepairBuildingAnimPacket {
                    remaining_time: -2.0,
                    ..start
                },
                7,
                ObjectKind::Building(BuildingType::Repair)
            ),
            Some(RepairBuildingAnimApply {
                state: Some(RepairBuildingAnimState {
                    remaining_time: 0.0,
                }),
                play_sound: true,
            })
        );
    }

    #[test]
    fn relay_snipe_object_round_trips_source_packet() {
        assert_eq!(relay_snipe_object(7), Some(SnipeObjectPacket { ref_id: 7 }));
        assert_eq!(relay_snipe_object(u32::MAX), None);
    }

    #[test]
    fn relay_portrait_anim_round_trips_source_packet() {
        assert_eq!(
            relay_portrait_anim(7, PortraitAnimationKind::VehicleCaptured),
            Some(DoPortraitAnimPacket {
                ref_id: 7,
                anim_id: PortraitAnimationKind::VehicleCaptured.wire_id(),
            })
        );
        assert_eq!(
            relay_portrait_anim(u32::MAX, PortraitAnimationKind::GunCaptured),
            None
        );
    }

    #[test]
    fn apply_portrait_anim_packet_validates_source_client_guards() {
        let packet = DoPortraitAnimPacket {
            ref_id: 7,
            anim_id: PortraitAnimationKind::GunCaptured.wire_id(),
        };

        let applied =
            apply_portrait_anim_packet(&packet, 7, TeamType::Red, TeamType::Red, false).unwrap();
        assert_eq!(applied.ref_id, 7);
        assert_eq!(applied.kind, Some(PortraitAnimationKind::GunCaptured));

        assert!(
            apply_portrait_anim_packet(&packet, 8, TeamType::Red, TeamType::Red, false).is_none()
        );
        assert!(
            apply_portrait_anim_packet(&packet, 7, TeamType::Blue, TeamType::Red, false).is_none()
        );
        assert!(
            apply_portrait_anim_packet(&packet, 7, TeamType::Red, TeamType::Red, true).is_none()
        );
        assert!(
            apply_portrait_anim_packet(
                &DoPortraitAnimPacket {
                    ref_id: -1,
                    anim_id: PortraitAnimationKind::GunCaptured.wire_id(),
                },
                7,
                TeamType::Red,
                TeamType::Red,
                false,
            )
            .is_none()
        );

        let unknown = apply_portrait_anim_packet(
            &DoPortraitAnimPacket {
                ref_id: 7,
                anim_id: 999,
            },
            7,
            TeamType::Red,
            TeamType::Red,
            false,
        )
        .unwrap();
        assert_eq!(unknown.kind, None);
    }

    #[test]
    fn process_driver_hit_effect_packet_rejects_unknown_or_negative_ref() {
        assert_eq!(
            process_driver_hit_effect_packet(
                DriverHitEffectPacket { ref_id: 7 },
                [2_u32, 7_u32].into_iter(),
            ),
            Some(7)
        );
        assert_eq!(
            process_driver_hit_effect_packet(
                DriverHitEffectPacket { ref_id: 99 },
                [2_u32, 7_u32].into_iter(),
            ),
            None
        );
        assert_eq!(
            process_driver_hit_effect_packet(
                DriverHitEffectPacket { ref_id: -1 },
                [2_u32, 7_u32].into_iter(),
            ),
            None
        );
    }

    #[test]
    fn source_health_percent_clamps_like_source_set_health_percent() {
        assert_eq!(
            source_health_from_percent(ObjectKind::Robot(RobotType::Grunt), 150),
            1081
        );
        assert_eq!(
            source_health_from_percent(ObjectKind::Robot(RobotType::Grunt), -1),
            0
        );
    }

    #[test]
    fn apply_object_health_packet_updates_live_stats_like_source_set_health() {
        let mut stats = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);

        assert!(apply_object_health_packet(
            ObjectHealthPacket {
                ref_id: 7,
                health: 500,
            },
            7,
            &mut stats,
        ));
        assert_eq!(stats.health, 500.0);

        assert!(apply_object_health_packet(
            ObjectHealthPacket {
                ref_id: 7,
                health: 5000,
            },
            7,
            &mut stats,
        ));
        assert_eq!(stats.health, stats.max_health);

        assert!(apply_object_health_packet(
            ObjectHealthPacket {
                ref_id: 7,
                health: -5,
            },
            7,
            &mut stats,
        ));
        assert_eq!(stats.health, 0.0);
    }

    #[test]
    fn apply_object_health_packet_rejects_unknown_ref() {
        let mut stats = ObjectStats::from_kind(ObjectKind::Robot(RobotType::Grunt), 100);
        let original_health = stats.health;

        assert!(!apply_object_health_packet(
            ObjectHealthPacket {
                ref_id: 99,
                health: 0,
            },
            7,
            &mut stats,
        ));
        assert_eq!(stats.health, original_health);

        assert!(!apply_object_health_packet(
            ObjectHealthPacket {
                ref_id: -1,
                health: 0,
            },
            7,
            &mut stats,
        ));
        assert_eq!(stats.health, original_health);
    }

    #[test]
    fn relay_destroy_object_round_trips_source_packet() {
        let mut queue = ObjectDestroyPacketQueue::default();

        assert_eq!(
            relay_destroy_object_packet(8, Some(3), false, true, false),
            Some(DestroyObjectPacket {
                ref_id: 8,
                killer_ref_id: 3,
                destroy_object: false,
                do_fire_death: true,
                do_missile_death: false,
                fire_missiles: Vec::new(),
            })
        );
        let missile = DestroyObjectMissileInfo {
            missile_offset_time: 3.25,
            missile_x: 44,
            missile_y: -12,
        };
        assert_eq!(
            relay_destroy_object_packet_with_missiles(9, None, true, false, false, vec![missile]),
            Some(DestroyObjectPacket {
                ref_id: 9,
                killer_ref_id: -1,
                destroy_object: true,
                do_fire_death: false,
                do_missile_death: false,
                fire_missiles: vec![missile],
            })
        );

        assert!(relay_destroy_object(
            &mut queue, 7, None, true, false, false
        ));
        assert_eq!(
            queue.pending,
            vec![DestroyObjectPacket {
                ref_id: 7,
                killer_ref_id: -1,
                destroy_object: true,
                do_fire_death: false,
                do_missile_death: false,
                fire_missiles: Vec::new(),
            }]
        );
        assert!(!relay_destroy_object(
            &mut queue,
            i32::MAX as u32 + 1,
            None,
            true,
            false,
            false
        ));
        assert_eq!(queue.pending.len(), 1);
    }

    #[test]
    fn apply_destroy_object_packet_sets_source_health_zero() {
        let mut stats = ObjectStats::from_kind(ObjectKind::MapItem(ItemType::Grenades as u8), 100);
        let packet = DestroyObjectPacket {
            ref_id: 12,
            killer_ref_id: -1,
            destroy_object: true,
            do_fire_death: false,
            do_missile_death: false,
            fire_missiles: Vec::new(),
        };

        assert!(apply_destroy_object_packet(&packet, 12, &mut stats, None));
        assert_eq!(stats.health, 0.0);

        let mut other_stats =
            ObjectStats::from_kind(ObjectKind::MapItem(ItemType::Grenades as u8), 100);
        assert!(!apply_destroy_object_packet(
            &packet,
            99,
            &mut other_stats,
            None
        ));
        assert_eq!(other_stats.health, other_stats.max_health);
    }

    #[test]
    fn process_new_object_packet_rejects_invalid_wire_values() {
        let mut packet = object_init_packet_from_map_object(&test_map().objects[0], 1);

        assert_eq!(process_new_object_packet(packet), Some(packet));
        packet.x = -16;
        packet.y = -32;
        assert_eq!(process_new_object_packet(packet), Some(packet));
        packet.owner = -1;
        assert_eq!(process_new_object_packet(packet), None);
        packet.owner = team_wire(TeamType::Red);
        packet.object_type = 99;
        assert_eq!(process_new_object_packet(packet), None);
        packet.object_type = MapObjectType::Building as u8;
        packet.ref_id = -1;
        assert_eq!(process_new_object_packet(packet), None);
    }

    #[test]
    fn relay_new_object_round_trips_source_packet_and_apply_contract() {
        let mut queue = SourceObjectEventQueue::default();
        let kind = ObjectKind::Cannon(CannonType::Gun);

        assert!(relay_new_object(
            &mut queue,
            41,
            kind,
            TeamType::Blue,
            Vec2::new(48.0, 64.0),
            3,
            0x1234,
            75,
            false,
        ));
        assert_eq!(queue.pending.len(), 1);
        let SourceObjectEvent::AddNewObject {
            packet,
            just_left_cannon,
        } = &queue.pending[0]
        else {
            panic!("relay should enqueue ADD_NEW_OBJECT");
        };
        assert!(!just_left_cannon);
        assert_eq!(
            *packet,
            ObjectInitPacket {
                x: 48,
                y: 64,
                ref_id: 41,
                owner: team_wire(TeamType::Blue),
                object_type: MapObjectType::Cannon as u8,
                object_id: CannonType::Gun as u8,
                building_level: 3,
                extra_links: 0x1234,
                health: source_health_from_percent(kind, 75),
            }
        );
        assert_eq!(
            apply_new_object_packet(*packet),
            Some(ObjectInitApply {
                ref_id: 41,
                kind,
                owner: TeamType::Blue,
                source_map_position: Vec2::new(48.0, 64.0),
                building_level: 3,
                extra_links: 0x1234,
            })
        );
    }

    #[test]
    fn built_cannon_list_round_trips_after_add_event_and_replaces_client_list() {
        let mut queue = SourceObjectEventQueue::default();
        assert!(relay_new_object(
            &mut queue,
            41,
            ObjectKind::Cannon(CannonType::Gun),
            TeamType::Blue,
            Vec2::new(48.0, 64.0),
            0,
            0,
            100,
            false,
        ));
        let stored = vec![
            ObjectKind::Cannon(CannonType::Gatling),
            ObjectKind::Cannon(CannonType::MissileCannon),
        ];
        assert!(relay_built_cannon_list(&mut queue, 7, &stored, Some(41)));
        assert!(matches!(
            queue.pending[0],
            SourceObjectEvent::AddNewObject { .. }
        ));
        let SourceObjectEvent::SetBuiltCannonAmount {
            packet,
            requires_new_object_ref,
        } = &queue.pending[1]
        else {
            panic!("second source event should update the parent cannon list");
        };
        assert_eq!(*requires_new_object_ref, Some(41));
        assert_eq!(packet.ref_id, 7);
        assert_eq!(
            packet.cannon_ids,
            vec![CannonType::Gatling as u8, CannonType::MissileCannon as u8]
        );

        let mut production = BuildingProduction {
            status: crate::components::BuildingProductionStatus::Select,
            current: None,
            queue: std::collections::VecDeque::new(),
            elapsed: 0.0,
            duration: 0.0,
            zone_ownage: 0.0,
            unit_limit_reached: false,
            stored_cannons: vec![ObjectKind::Cannon(CannonType::Howitzer)],
        };
        assert!(!apply_built_cannon_list_packet(packet, 8, &mut production));
        assert_eq!(
            production.stored_cannons,
            vec![ObjectKind::Cannon(CannonType::Howitzer)]
        );
        assert!(apply_built_cannon_list_packet(packet, 7, &mut production));
        assert_eq!(production.stored_cannons, stored);

        let invalid = BuiltCannonListPacket {
            ref_id: 7,
            cannon_ids: vec![u8::MAX],
        };
        assert!(!apply_built_cannon_list_packet(
            &invalid,
            7,
            &mut production
        ));
        assert_eq!(production.stored_cannons, stored);
    }

    #[test]
    fn ejected_driver_group_relays_each_member_in_source_fifo_order() {
        let mut queue = SourceObjectEventQueue::default();

        let refs = relay_ejected_driver_group(
            &mut queue,
            100,
            RobotType::Tough,
            TeamType::Green,
            Vec2::new(48.0, 80.0),
            &[91, 42, 7],
            true,
        )
        .unwrap();

        assert_eq!(refs, vec![100, 101, 102]);
        assert_eq!(queue.pending.len(), 9);
        for (member_index, chunk) in queue.pending.chunks_exact(3).enumerate() {
            let expected_ref_id = 100 + member_index as i32;
            let SourceObjectEvent::AddNewObject {
                packet,
                just_left_cannon,
            } = &chunk[0]
            else {
                panic!("member event 0 should be ADD_NEW_OBJECT");
            };
            assert_eq!(packet.ref_id, expected_ref_id);
            assert_eq!(packet.x, 48);
            assert_eq!(packet.y, 80);
            assert_eq!(packet.owner, team_wire(TeamType::Green));
            assert_eq!(packet.object_type, MapObjectType::Robot as u8);
            assert_eq!(packet.object_id, RobotType::Tough as u8);
            assert!(*just_left_cannon);

            let SourceObjectEvent::ObjectGroupInfo {
                packet: group,
                requires_new_object_refs,
            } = &chunk[1]
            else {
                panic!("member event 1 should be OBJECT_GROUP_INFO");
            };
            assert_eq!(requires_new_object_refs, &refs);
            assert_eq!(group.ref_id, expected_ref_id);
            if member_index == 0 {
                assert_eq!(group.leader_ref_id, -1);
                assert_eq!(group.minion_refs, vec![101, 102]);
            } else {
                assert_eq!(group.leader_ref_id, 100);
                assert!(group.minion_refs.is_empty());
            }

            let SourceObjectEvent::UpdateHealth {
                packet: health,
                requires_new_object_ref,
            } = &chunk[2]
            else {
                panic!("member event 2 should be UPDATE_HEALTH");
            };
            assert_eq!(*requires_new_object_ref, Some(refs[member_index]));
            assert_eq!(health.ref_id, expected_ref_id);
            assert_eq!(health.health, [91, 42, 7][member_index]);
        }

        relay_ejected_driver_batch_commit(&mut queue, 7, &refs);
        let SourceObjectEvent::CommitEjectedDriverBatch {
            carrier_ref_id,
            requires_new_object_refs,
        } = &queue.pending[9]
        else {
            panic!("batch commit should follow every wire event");
        };
        assert_eq!(*carrier_ref_id, 7);
        assert_eq!(requires_new_object_refs, &refs);
    }

    #[test]
    fn ejected_driver_batch_preflight_rejects_collisions_and_wire_overflow() {
        let refs = ejected_driver_group_refs(100, 3).unwrap();
        assert_eq!(refs, vec![100, 101, 102]);
        assert!(ejected_driver_refs_available(&refs, &HashSet::new()));
        assert!(!ejected_driver_refs_available(&refs, &HashSet::from([101])));
        assert!(source_new_object_batch_ready(
            &refs,
            &HashSet::from([100, 101, 102])
        ));
        assert!(!source_new_object_batch_ready(
            &refs,
            &HashSet::from([100, 102])
        ));
        assert!(source_new_object_batch_ready(&[], &HashSet::new()));
        assert!(ejected_driver_group_refs(i32::MAX as u32, 2).is_none());
        assert!(ejected_driver_group_refs(100, 0).is_none());

        let mut queue = SourceObjectEventQueue::default();
        relay_ejected_driver_batch_commit(&mut queue, 7, &[]);
        assert_eq!(
            queue.pending,
            vec![SourceObjectEvent::CommitEjectedDriverBatch {
                carrier_ref_id: 7,
                requires_new_object_refs: Vec::new(),
            }]
        );
    }

    #[test]
    fn ejected_driver_group_rolls_back_the_whole_batch_on_relay_failure() {
        let mut queue = SourceObjectEventQueue::default();
        assert!(relay_object_health(&mut queue, 7, 55, None));
        let checkpoint = queue.pending.clone();

        assert!(
            relay_ejected_driver_group(
                &mut queue,
                i32::MAX as u32,
                RobotType::Grunt,
                TeamType::Red,
                Vec2::ZERO,
                &[10, 20],
                false,
            )
            .is_none()
        );
        assert_eq!(queue.pending, checkpoint);

        assert!(
            relay_ejected_driver_group(
                &mut queue,
                8,
                RobotType::Grunt,
                TeamType::Red,
                Vec2::ZERO,
                &[],
                false,
            )
            .is_none()
        );
        assert_eq!(queue.pending, checkpoint);
    }

    #[test]
    fn produced_robot_group_relays_source_add_group_then_leader_waypoints() {
        let mut queue = SourceObjectEventQueue::default();

        let refs = relay_produced_object_batch(
            &mut queue,
            7,
            100,
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            Vec2::new(92.0, 192.0),
            Vec2::new(120.0, 240.0),
            3,
        )
        .unwrap();

        assert_eq!(refs, vec![100, 101, 102]);
        assert_eq!(queue.pending.len(), 8);
        for (member_index, chunk) in queue.pending[..6].chunks_exact(2).enumerate() {
            let expected_ref_id = 100 + member_index as i32;
            let SourceObjectEvent::AddNewObject { packet, .. } = &chunk[0] else {
                panic!("member event 0 should be ADD_NEW_OBJECT");
            };
            assert_eq!(packet.ref_id, expected_ref_id);
            assert_eq!((packet.x, packet.y), (92, 192));
            let SourceObjectEvent::ObjectGroupInfo {
                packet: group,
                requires_new_object_refs,
            } = &chunk[1]
            else {
                panic!("member event 1 should be OBJECT_GROUP_INFO");
            };
            assert_eq!(requires_new_object_refs, &refs);
            assert_eq!(group.ref_id, expected_ref_id);
            if member_index == 0 {
                assert_eq!(group.leader_ref_id, -1);
                assert_eq!(group.minion_refs, vec![101, 102]);
            } else {
                assert_eq!(group.leader_ref_id, 100);
                assert!(group.minion_refs.is_empty());
            }
        }
        let SourceObjectEvent::ObjectWaypoints {
            packet,
            requires_new_object_ref,
            target_team,
        } = &queue.pending[6]
        else {
            panic!("group creation should relay leader SEND_WAYPOINTS");
        };
        assert_eq!(*requires_new_object_ref, Some(100));
        assert_eq!(*target_team, None);
        assert_eq!(packet.ref_id, 100);
        assert_eq!(
            packet.waypoints,
            vec![SourceWaypoint {
                mode: SourceWaypointMode::ForceMove,
                ref_id: -1,
                x: 120,
                y: 240,
                attack_to: false,
                player_given: false,
            }]
        );
        assert_eq!(
            movement_waypoints_from_source_map_packet(packet, 100).unwrap()[0].position,
            Vec2::new(120.0, -240.0)
        );
        assert!(matches!(
            queue.pending[7],
            SourceObjectEvent::CommitProducedObjectBatch {
                building_ref_id: 7,
                ref requires_new_object_refs,
            } if requires_new_object_refs == &refs
        ));
        assert!(
            !queue
                .pending
                .iter()
                .any(|event| matches!(event, SourceObjectEvent::UpdateHealth { .. }))
        );
    }

    #[test]
    fn produced_vehicle_batch_has_no_robot_group_packet_and_rolls_back_invalid_units() {
        let mut queue = SourceObjectEventQueue::default();
        let refs = relay_produced_object_batch(
            &mut queue,
            7,
            40,
            ObjectKind::Vehicle(VehicleType::Light),
            TeamType::Blue,
            Vec2::new(84.0, 184.0),
            Vec2::new(120.0, 240.0),
            1,
        )
        .unwrap();
        assert_eq!(refs, vec![40]);
        assert_eq!(queue.pending.len(), 3);
        assert!(matches!(
            queue.pending[0],
            SourceObjectEvent::AddNewObject { .. }
        ));
        assert!(matches!(
            queue.pending[1],
            SourceObjectEvent::ObjectWaypoints { .. }
        ));
        assert!(matches!(
            queue.pending[2],
            SourceObjectEvent::CommitProducedObjectBatch { .. }
        ));

        let checkpoint = queue.pending.clone();
        assert!(
            relay_produced_object_batch(
                &mut queue,
                7,
                41,
                ObjectKind::Cannon(CannonType::Gun),
                TeamType::Blue,
                Vec2::ZERO,
                Vec2::ZERO,
                1,
            )
            .is_none()
        );
        assert_eq!(queue.pending, checkpoint);
        assert!(
            relay_produced_object_batch(
                &mut queue,
                7,
                41,
                ObjectKind::Vehicle(VehicleType::Light),
                TeamType::Blue,
                Vec2::ZERO,
                Vec2::new(f32::NAN, 0.0),
                1,
            )
            .is_none()
        );
        assert_eq!(queue.pending, checkpoint);
    }

    #[test]
    fn repaired_vehicle_relays_add_owner_waypoints_team_then_commit() {
        let mut queue = SourceObjectEventQueue::default();
        let driver = DriverHealth::with_driver_states(RobotType::Sniper, vec![42.0], vec![1.25]);
        let resume = [
            MovementWaypoint::player_move_to(Vec2::new(180.0, -260.0), true),
            MovementWaypoint::player_attack_target(91, Vec2::new(220.0, -280.0)),
        ];

        assert_eq!(
            relay_repaired_object_batch(
                &mut queue,
                7,
                40,
                ObjectKind::Vehicle(VehicleType::Light),
                TeamType::Blue,
                Vec2::new(84.0, 184.0),
                Vec2::new(120.0, 240.0),
                1,
                Some(&driver),
                &resume,
            ),
            Some(vec![40])
        );
        assert_eq!(queue.pending.len(), 4);
        assert!(matches!(
            queue.pending[0],
            SourceObjectEvent::AddNewObject { .. }
        ));
        let SourceObjectEvent::ObjectWaypoints {
            packet,
            requires_new_object_ref,
            target_team,
        } = &queue.pending[1]
        else {
            panic!("repaired leader should receive SEND_WAYPOINTS");
        };
        assert_eq!(*requires_new_object_ref, Some(40));
        assert_eq!(*target_team, Some(TeamType::Blue));
        assert_eq!(packet.ref_id, 40);
        assert_eq!(packet.waypoints.len(), 3);
        assert_eq!(packet.waypoints[0].mode, SourceWaypointMode::ForceMove);
        assert_eq!((packet.waypoints[0].x, packet.waypoints[0].y), (120, 240));
        assert_eq!(packet.waypoints[1].mode, SourceWaypointMode::Move);
        assert_eq!((packet.waypoints[1].x, packet.waypoints[1].y), (180, 260));
        assert!(packet.waypoints[1].attack_to);
        assert!(packet.waypoints[1].player_given);
        assert_eq!(packet.waypoints[2].mode, SourceWaypointMode::Attack);
        assert_eq!(packet.waypoints[2].ref_id, 91);
        assert_eq!((packet.waypoints[2].x, packet.waypoints[2].y), (220, 280));

        let SourceObjectEvent::ObjectTeam {
            packet,
            requires_new_object_ref,
        } = &queue.pending[2]
        else {
            panic!("repaired leader should receive SET_OBJECT_TEAM");
        };
        assert_eq!(*requires_new_object_ref, Some(40));
        assert_eq!(packet.ref_id, 40);
        assert_eq!(packet.owner, TeamType::Blue as i8);
        assert_eq!(packet.driver_type, RobotType::Sniper as i8);
        assert_eq!(packet.drivers.len(), 1);
        assert_eq!(packet.drivers[0].driver_health, 42);
        assert_eq!(packet.drivers[0].next_attack_time, 1.25);
        assert!(matches!(
            queue.pending[3],
            SourceObjectEvent::CommitRepairedObjectBatch {
                building_ref_id: 7,
                ..
            }
        ));
        assert!(!queue.pending.iter().any(|event| matches!(
            event,
            SourceObjectEvent::ObjectGroupInfo { .. } | SourceObjectEvent::UpdateHealth { .. }
        )));
    }

    #[test]
    fn repaired_robot_group_uses_source_member_order_and_one_team_packet() {
        let mut queue = SourceObjectEventQueue::default();
        let refs = relay_repaired_object_batch(
            &mut queue,
            7,
            100,
            ObjectKind::Robot(RobotType::Grunt),
            TeamType::Red,
            Vec2::new(92.0, 192.0),
            Vec2::new(120.0, 240.0),
            3,
            None,
            &[],
        )
        .unwrap();

        assert_eq!(refs, vec![100, 101, 102]);
        assert_eq!(queue.pending.len(), 9);
        for (index, chunk) in queue.pending[..6].chunks_exact(2).enumerate() {
            assert!(matches!(chunk[0], SourceObjectEvent::AddNewObject { .. }));
            let SourceObjectEvent::ObjectGroupInfo { packet, .. } = &chunk[1] else {
                panic!("robot member should relay OBJECT_GROUP_INFO");
            };
            assert_eq!(packet.ref_id, 100 + index as i32);
        }
        assert!(matches!(
            queue.pending[6],
            SourceObjectEvent::ObjectWaypoints { .. }
        ));
        assert!(matches!(
            queue.pending[7],
            SourceObjectEvent::ObjectTeam { .. }
        ));
        assert!(matches!(
            queue.pending[8],
            SourceObjectEvent::CommitRepairedObjectBatch { .. }
        ));
    }

    #[test]
    fn repair_entry_defers_source_group_promotion_then_delete_to_early_drain() {
        let mut queue = SourceObjectEventQueue::default();
        let members = [
            SourceDeleteGroupMember {
                ref_id: 10,
                leader_ref_id: 10,
                member_index: 0,
                grenade_amount: 7,
            },
            SourceDeleteGroupMember {
                ref_id: 12,
                leader_ref_id: 10,
                member_index: 1,
                grenade_amount: 0,
            },
            SourceDeleteGroupMember {
                ref_id: 11,
                leader_ref_id: 10,
                member_index: 2,
                grenade_amount: 0,
            },
        ];

        assert!(relay_deferred_delete_object(&mut queue, 10, &members));
        let [SourceObjectEvent::DeferredUntilEarlyDrain { events }] = queue.pending.as_slice()
        else {
            panic!("repair entry should defer deletion until the next early drain");
        };
        assert_eq!(events.len(), 5);
        let SourceObjectEvent::ObjectGroupInfo { packet, .. } = &events[0] else {
            panic!("new leader group packet should be first");
        };
        assert_eq!(packet.ref_id, 12);
        assert_eq!(packet.leader_ref_id, -1);
        assert_eq!(packet.minion_refs, vec![11]);
        let SourceObjectEvent::ObjectGroupInfo { packet, .. } = &events[1] else {
            panic!("remaining minion packet should follow");
        };
        assert_eq!((packet.ref_id, packet.leader_ref_id), (11, 12));
        assert!(matches!(
            events[2],
            SourceObjectEvent::ObjectGrenadeAmount { .. }
        ));
        let SourceObjectEvent::ObjectGroupInfo { packet, .. } = &events[3] else {
            panic!("removed leader clear packet should precede delete");
        };
        assert_eq!(packet.ref_id, 10);
        assert_eq!(packet.leader_ref_id, -1);
        assert!(packet.minion_refs.is_empty());
        assert_eq!(
            events[4],
            SourceObjectEvent::DeleteObject {
                packet: DeleteObjectPacket { ref_id: 10 },
            }
        );
    }

    #[test]
    fn deferred_repair_delete_is_ignored_by_late_drain_and_unwrapped_by_early_drain() {
        let mut queue = SourceObjectEventQueue::default();
        assert!(relay_deferred_delete_object(&mut queue, 7, &[]));

        assert!(take_source_events_for_drain(&mut queue, SourceObjectDrainPhase::Late,).is_empty());
        assert_eq!(queue.pending.len(), 1);

        assert_eq!(
            take_source_events_for_drain(&mut queue, SourceObjectDrainPhase::Early),
            vec![SourceObjectEvent::DeleteObject {
                packet: DeleteObjectPacket { ref_id: 7 },
            }]
        );
        assert!(queue.pending.is_empty());
    }

    #[test]
    fn runtime_group_assignments_use_the_corrected_source_group_intent() {
        let object_kinds = HashMap::from([
            (100, ObjectKind::Robot(RobotType::Tough)),
            (101, ObjectKind::Robot(RobotType::Tough)),
            (102, ObjectKind::Robot(RobotType::Tough)),
            (200, ObjectKind::Vehicle(VehicleType::Apc)),
        ]);
        let leader_packet = ObjectGroupInfoPacket {
            ref_id: 100,
            leader_ref_id: -1,
            minion_refs: vec![101, 102, 999],
        };
        assert_eq!(
            runtime_group_assignments(&leader_packet, &object_kinds),
            Some(vec![
                (100, 100, Some(0)),
                (101, 100, Some(1)),
                (102, 100, Some(2)),
            ])
        );

        let minion_packet = ObjectGroupInfoPacket {
            ref_id: 101,
            leader_ref_id: 100,
            minion_refs: Vec::new(),
        };
        assert_eq!(
            runtime_group_assignments(&minion_packet, &object_kinds),
            Some(vec![(101, 100, None)])
        );

        let missing_leader = ObjectGroupInfoPacket {
            ref_id: 102,
            leader_ref_id: 999,
            minion_refs: Vec::new(),
        };
        assert_eq!(
            runtime_group_assignments(&missing_leader, &object_kinds),
            Some(vec![(102, 102, Some(0))])
        );

        let non_robot = ObjectGroupInfoPacket {
            ref_id: 200,
            leader_ref_id: -1,
            minion_refs: Vec::new(),
        };
        assert_eq!(runtime_group_assignments(&non_robot, &object_kinds), None);
    }

    #[test]
    fn new_object_apply_uses_source_constructible_types_and_fifo_ref_identity() {
        let mut known_ref_ids = HashSet::new();
        let bridge_packet = ObjectInitPacket {
            x: -16,
            y: -32,
            ref_id: 9,
            owner: team_wire(TeamType::Red),
            object_type: MapObjectType::Building as u8,
            object_id: BuildingType::BridgeVert as u8,
            building_level: 2,
            extra_links: 7,
            health: 1,
        };
        let accepted = accept_new_object_packet(bridge_packet, &mut known_ref_ids).unwrap();
        assert_eq!(accepted.kind, ObjectKind::Bridge(BuildingType::BridgeVert));
        assert_eq!(accepted.source_map_position, Vec2::new(-16.0, -32.0));
        assert!(accept_new_object_packet(bridge_packet, &mut known_ref_ids).is_none());

        let mut invalid = bridge_packet;
        invalid.ref_id = 10;
        invalid.object_type = MapObjectType::Animal as u8;
        invalid.object_id = 4;
        assert_eq!(
            apply_new_object_packet(invalid).map(|apply| apply.kind),
            Some(ObjectKind::Animal(4))
        );
        invalid.object_type = MapObjectType::Vehicle as u8;
        invalid.object_id = u8::MAX;
        assert!(apply_new_object_packet(invalid).is_none());
    }
}
