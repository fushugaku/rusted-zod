use crate::network_commands::{CommandPayload, RequestMapCommand, StoreMapPacket};

const SOURCE_MAP_CHUNK_SIZE: usize = 1024 * 4;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct MapDownloadAccumulator {
    data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MapDownloadStatus {
    InProgress,
    Complete(Vec<u8>),
}

impl MapDownloadAccumulator {
    pub(crate) fn apply_store_map_packet(
        &mut self,
        packet: StoreMapPacket,
    ) -> Option<MapDownloadStatus> {
        if packet.packet_number == -1 {
            let data = std::mem::take(&mut self.data);
            return Some(MapDownloadStatus::Complete(data));
        }
        if packet.packet_number < 0 {
            return None;
        }
        if packet.packet_number == 0 {
            self.data.clear();
        }
        self.data.extend_from_slice(&packet.bytes);
        Some(MapDownloadStatus::InProgress)
    }
}

pub(crate) fn relay_request_map_bytes(server_map_bytes: &[u8]) -> Option<Vec<u8>> {
    let request = RequestMapCommand;
    let wire_packet = request.encode_packet();
    let payload = wire_packet.get(8..)?;
    RequestMapCommand::decode_payload(payload)?;

    let mut accumulator = MapDownloadAccumulator::default();
    for packet in store_map_packets(server_map_bytes) {
        let wire_packet = packet.encode_packet();
        let payload = wire_packet.get(8..)?;
        let decoded_packet = StoreMapPacket::decode_payload(payload)?;
        if let MapDownloadStatus::Complete(map_bytes) =
            accumulator.apply_store_map_packet(decoded_packet)?
        {
            return Some(map_bytes);
        }
    }
    None
}

fn store_map_packets(map_bytes: &[u8]) -> Vec<StoreMapPacket> {
    let mut packets = Vec::new();
    for (packet_number, chunk) in map_bytes.chunks(SOURCE_MAP_CHUNK_SIZE).enumerate() {
        packets.push(StoreMapPacket {
            packet_number: i32::try_from(packet_number).expect("map chunk count fits source int"),
            bytes: chunk.to_vec(),
        });
    }
    packets.push(StoreMapPacket {
        packet_number: -1,
        bytes: Vec::new(),
    });
    packets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_map_reassembles_source_sized_chunks() {
        let mut bytes = vec![0u8; SOURCE_MAP_CHUNK_SIZE + 3];
        bytes[SOURCE_MAP_CHUNK_SIZE - 1] = 7;
        bytes[SOURCE_MAP_CHUNK_SIZE] = 8;
        bytes[SOURCE_MAP_CHUNK_SIZE + 2] = 9;

        assert_eq!(relay_request_map_bytes(&bytes), Some(bytes));
    }

    #[test]
    fn store_map_final_packet_returns_accumulated_bytes() {
        let mut accumulator = MapDownloadAccumulator::default();

        assert_eq!(
            accumulator.apply_store_map_packet(StoreMapPacket {
                packet_number: 0,
                bytes: vec![1, 2],
            }),
            Some(MapDownloadStatus::InProgress)
        );
        assert_eq!(
            accumulator.apply_store_map_packet(StoreMapPacket {
                packet_number: 1,
                bytes: vec![3],
            }),
            Some(MapDownloadStatus::InProgress)
        );
        assert_eq!(
            accumulator.apply_store_map_packet(StoreMapPacket {
                packet_number: -1,
                bytes: Vec::new(),
            }),
            Some(MapDownloadStatus::Complete(vec![1, 2, 3]))
        );
    }

    #[test]
    fn first_store_map_packet_restarts_download_buffer() {
        let mut accumulator = MapDownloadAccumulator::default();

        accumulator.apply_store_map_packet(StoreMapPacket {
            packet_number: 0,
            bytes: vec![1, 2],
        });
        accumulator.apply_store_map_packet(StoreMapPacket {
            packet_number: 0,
            bytes: vec![3],
        });

        assert_eq!(
            accumulator.apply_store_map_packet(StoreMapPacket {
                packet_number: -1,
                bytes: Vec::new(),
            }),
            Some(MapDownloadStatus::Complete(vec![3]))
        );
    }
}
