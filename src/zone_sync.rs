use crate::{
    components::ZoneOwnership,
    network_commands::{CommandPayload, RequestZonesCommand, SetZoneInfoPacket},
    original::{map::ZMap, types::TeamType},
    settings_sync::SourceSettingsState,
};

pub(crate) fn relay_request_zone_ownership(
    map: &ZMap,
    settings: &SourceSettingsState,
) -> Option<ZoneOwnership> {
    let wire_packet = RequestZonesCommand.encode_packet();
    let payload = wire_packet.get(8..)?;
    RequestZonesCommand::decode_payload(payload)?;

    let server_zones = ZoneOwnership::from_map_with_settings(map, settings);
    let mut client_zones = ZoneOwnership::from_map_with_settings(map, settings);
    client_zones.owners.fill(TeamType::Null);

    for (zone_number, owner) in server_zones.owners.iter().copied().enumerate() {
        let packet = SetZoneInfoPacket {
            zone_number: i32::try_from(zone_number).ok()?,
            owner: team_wire(owner),
        };
        let wire_packet = packet.encode_packet();
        let payload = wire_packet.get(8..)?;
        let decoded_packet = SetZoneInfoPacket::decode_payload(payload)?;
        apply_set_zone_info(&mut client_zones, decoded_packet)?;
    }

    Some(client_zones)
}

pub(crate) fn relay_set_zone_info(
    zones: &mut ZoneOwnership,
    zone_number: usize,
    owner: TeamType,
) -> Option<()> {
    let packet = SetZoneInfoPacket {
        zone_number: i32::try_from(zone_number).ok()?,
        owner: team_wire(owner),
    };
    let wire_packet = packet.encode_packet();
    let payload = wire_packet.get(8..)?;
    let decoded_packet = SetZoneInfoPacket::decode_payload(payload)?;
    apply_set_zone_info(zones, decoded_packet)
}

pub(crate) fn apply_set_zone_info(
    zones: &mut ZoneOwnership,
    packet: SetZoneInfoPacket,
) -> Option<()> {
    if packet.zone_number < 0 || packet.owner < 0 {
        return None;
    }
    let zone_number = usize::try_from(packet.zone_number).ok()?;
    if zone_number >= zones.owners.len() {
        return None;
    }
    let owner = TeamType::try_from(packet.owner).ok()?;
    zones.owners[zone_number] = owner;
    Some(())
}

fn team_wire(team: TeamType) -> i8 {
    team as i8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        components::{ZoneLink, ZoneOwnership},
        original::{
            map::{MapBasics, MapObject, MapObjectType, MapTile, MapZone, ZMap},
            objects::ItemType,
            types::PlanetType,
        },
    };

    fn test_map() -> ZMap {
        ZMap {
            basics: MapBasics {
                width: 20,
                height: 20,
                map_name: "zones".to_string(),
                player_count: 2,
                object_count: 2,
                terrain_type: PlanetType::Desert,
                zone_count: 2,
            },
            tiles: vec![MapTile { tile: 0 }; 400],
            zones: vec![
                MapZone {
                    x: 1,
                    y: 1,
                    w: 5,
                    h: 5,
                },
                MapZone {
                    x: 10,
                    y: 10,
                    w: 5,
                    h: 5,
                },
            ],
            objects: vec![
                MapObject {
                    x: 2,
                    y: 2,
                    owner: TeamType::Red,
                    object_type: MapObjectType::MapItem,
                    object_id: ItemType::Flag as u8,
                    building_level: 0,
                    extra_links: 0,
                    health_percent: 100,
                },
                MapObject {
                    x: 11,
                    y: 11,
                    owner: TeamType::Blue,
                    object_type: MapObjectType::MapItem,
                    object_id: ItemType::Flag as u8,
                    building_level: 0,
                    extra_links: 0,
                    health_percent: 100,
                },
            ],
        }
    }

    #[test]
    fn zone_request_round_trips_map_zone_owners() {
        let map = test_map();

        let zones = relay_request_zone_ownership(&map, &SourceSettingsState::default()).unwrap();

        assert_eq!(zones.owners, vec![TeamType::Red, TeamType::Blue]);
        assert_eq!(zones.links.len(), 2);
    }

    #[test]
    fn set_zone_info_applies_valid_owner_and_rejects_invalid_values() {
        let mut zones = ZoneOwnership {
            owners: vec![TeamType::Null, TeamType::Null],
            links: vec![ZoneLink {
                zone_index: 0,
                flag_ref_id: 1,
                building_refs: Vec::new(),
            }],
        };

        assert_eq!(
            apply_set_zone_info(
                &mut zones,
                SetZoneInfoPacket {
                    zone_number: 1,
                    owner: team_wire(TeamType::Green),
                },
            ),
            Some(())
        );
        assert_eq!(zones.owners[1], TeamType::Green);
        assert_eq!(
            apply_set_zone_info(
                &mut zones,
                SetZoneInfoPacket {
                    zone_number: -1,
                    owner: team_wire(TeamType::Red),
                },
            ),
            None
        );
        assert_eq!(
            apply_set_zone_info(
                &mut zones,
                SetZoneInfoPacket {
                    zone_number: 9,
                    owner: team_wire(TeamType::Red),
                },
            ),
            None
        );
        assert_eq!(
            apply_set_zone_info(
                &mut zones,
                SetZoneInfoPacket {
                    zone_number: 0,
                    owner: -1,
                },
            ),
            None
        );
        assert_eq!(
            apply_set_zone_info(
                &mut zones,
                SetZoneInfoPacket {
                    zone_number: 0,
                    owner: 99_i8,
                },
            ),
            None
        );
    }

    #[test]
    fn relay_set_zone_info_round_trips_runtime_zone_update() {
        let mut zones = ZoneOwnership {
            owners: vec![TeamType::Null, TeamType::Null],
            links: vec![ZoneLink {
                zone_index: 0,
                flag_ref_id: 1,
                building_refs: Vec::new(),
            }],
        };

        assert_eq!(relay_set_zone_info(&mut zones, 1, TeamType::Blue), Some(()));
        assert_eq!(zones.owners[1], TeamType::Blue);
        assert_eq!(
            relay_set_zone_info(&mut zones, usize::MAX, TeamType::Red),
            None
        );
    }
}
