use bevy::prelude::{ResMut, Resource};

use crate::{
    network_commands::{
        CommandPayload, GiveVersionPacket, SOURCE_GAME_VERSION, TcpEventId, encode_packet,
    },
    news::NewsLog,
};

#[derive(Default, Resource)]
pub(crate) struct VersionInitialQueryState {
    requested: bool,
}

pub(crate) fn process_initial_version_query(
    mut initial_query: ResMut<VersionInitialQueryState>,
    mut news_log: ResMut<NewsLog>,
) {
    if initial_query.requested {
        return;
    }

    relay_request_version(&mut news_log);
    initial_query.requested = true;
}

pub(crate) fn relay_request_version(news_log: &mut NewsLog) -> bool {
    let wire_packet = encode_packet(TcpEventId::RequestVersion, &[]);
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    if !payload.is_empty() {
        return false;
    }
    relay_version_packet(news_log)
}

pub(crate) fn relay_version_packet(news_log: &mut NewsLog) -> bool {
    let packet = GiveVersionPacket::source_current();
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = GiveVersionPacket::decode_payload(payload) else {
        return false;
    };
    apply_version_packet(decoded_packet, news_log)
}

pub(crate) fn apply_version_packet(packet: GiveVersionPacket, news_log: &mut NewsLog) -> bool {
    let message = if packet.version == SOURCE_GAME_VERSION {
        format!("the server version is {}", packet.version)
    } else {
        format!(
            "the server version is {}, which mismatches our version {}",
            packet.version, SOURCE_GAME_VERSION
        )
    };
    news_log.push_source_news(message, 255, 255, 255);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_version_relays_source_version_packet_to_news() {
        let mut news_log = NewsLog::default();

        assert!(relay_request_version(&mut news_log));
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("the server version is 2011-09-06")
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.color),
            Some(crate::news::NewsColor::from_source_rgb(255, 255, 255))
        );
    }

    #[test]
    fn version_packet_mismatch_message_matches_source_client_branch() {
        let mut news_log = NewsLog::default();

        assert!(apply_version_packet(
            GiveVersionPacket::new("other").unwrap(),
            &mut news_log
        ));
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("the server version is other, which mismatches our version 2011-09-06")
        );
    }
}
