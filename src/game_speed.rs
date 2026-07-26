use bevy::{
    prelude::{Res, ResMut, Resource, Time},
    time::Virtual,
};

use crate::{
    components::GamePauseState,
    network_commands::{CommandPayload, TcpEventId, UpdateGameSpeedPacket, encode_packet},
};

#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub(crate) struct GameSpeedState {
    pub(crate) game_speed: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameSpeedVoteRequest {
    pub(crate) speed_percent: i32,
}

#[derive(Default, Resource)]
pub(crate) struct GameSpeedVoteRequestQueue {
    pub(crate) pending: Vec<GameSpeedVoteRequest>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GameSpeedUpdate {
    pub(crate) game_speed: f32,
}

#[derive(Default, Resource)]
pub(crate) struct GameSpeedUpdateQueue {
    pub(crate) pending: Vec<GameSpeedUpdate>,
}

#[derive(Default, Resource)]
pub(crate) struct GameSpeedInitialQueryState {
    requested: bool,
}

impl Default for GameSpeedState {
    fn default() -> Self {
        Self { game_speed: 1.0 }
    }
}

pub(crate) fn sync_source_game_time(
    pause: Res<GamePauseState>,
    speed: Res<GameSpeedState>,
    mut game_time: ResMut<Time<Virtual>>,
) {
    apply_source_game_time_control(pause.paused, speed.game_speed, &mut game_time);
}

pub(crate) fn apply_source_game_time_control(
    paused: bool,
    game_speed: f32,
    game_time: &mut Time<Virtual>,
) {
    let game_speed = if game_speed.is_finite() {
        game_speed.max(0.0)
    } else {
        0.0
    };
    if game_time.relative_speed() != game_speed {
        game_time.set_relative_speed(game_speed);
    }
    if paused {
        game_time.pause();
    } else {
        game_time.unpause();
    }
}

pub(crate) fn process_initial_game_speed_query(
    mut initial_query: ResMut<GameSpeedInitialQueryState>,
    mut game_speed: ResMut<GameSpeedState>,
) {
    if initial_query.requested {
        return;
    }

    let current_server_speed = game_speed.game_speed;
    relay_get_game_speed(current_server_speed, &mut game_speed);
    initial_query.requested = true;
}

pub(crate) fn relay_get_game_speed(
    current_server_speed: f32,
    client_state: &mut GameSpeedState,
) -> bool {
    let wire_packet = encode_packet(TcpEventId::GetGameSpeed, &[]);
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    if !payload.is_empty() {
        return false;
    }
    relay_update_game_speed(current_server_speed, client_state)
}

pub(crate) fn relay_update_game_speed(game_speed: f32, client_state: &mut GameSpeedState) -> bool {
    let packet = UpdateGameSpeedPacket { game_speed };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = UpdateGameSpeedPacket::decode_payload(payload) else {
        return false;
    };
    apply_update_game_speed(decoded_packet, client_state)
}

pub(crate) fn game_speed_from_percent(speed_percent: i32) -> f32 {
    speed_percent as f32 / 100.0
}

pub(crate) fn game_speed_percent_from_float(game_speed: f32) -> i32 {
    (game_speed * 100.0) as i32
}

pub(crate) fn game_speed_changed_news_message(game_speed: f32) -> String {
    format!(
        "game speed changed to {}%",
        game_speed_percent_from_float(game_speed)
    )
}

pub(crate) fn apply_update_game_speed(
    packet: UpdateGameSpeedPacket,
    client_state: &mut GameSpeedState,
) -> bool {
    client_state.game_speed = packet.game_speed.max(0.0);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_game_speed_matches_source_ztime_constructor() {
        assert_eq!(GameSpeedState::default().game_speed, 1.0);
    }

    #[test]
    fn get_game_speed_relays_current_server_speed_to_client_state() {
        let mut state = GameSpeedState { game_speed: 0.25 };

        assert!(relay_get_game_speed(1.5, &mut state));
        assert_eq!(state.game_speed, 1.5);
    }

    #[test]
    fn update_game_speed_apply_clamps_negative_like_source_set_game_speed() {
        let mut state = GameSpeedState { game_speed: 1.0 };

        assert!(apply_update_game_speed(
            UpdateGameSpeedPacket { game_speed: -2.0 },
            &mut state
        ));
        assert_eq!(state.game_speed, 0.0);
    }

    #[test]
    fn speed_percent_helpers_match_source_int_float_casts() {
        assert_eq!(game_speed_from_percent(150), 1.5);
        assert_eq!(game_speed_percent_from_float(1.5), 150);
        assert_eq!(game_speed_percent_from_float(0.505), 50);
        assert_eq!(
            game_speed_changed_news_message(2.0),
            "game speed changed to 200%"
        );
    }

    #[test]
    fn source_time_control_pauses_and_scales_bevy_virtual_time() {
        let mut time = Time::<Virtual>::default();

        apply_source_game_time_control(false, 1.5, &mut time);
        assert!(!time.is_paused());
        assert_eq!(time.relative_speed(), 1.5);

        apply_source_game_time_control(true, 0.5, &mut time);
        assert!(time.is_paused());
        assert_eq!(time.relative_speed(), 0.5);

        apply_source_game_time_control(false, f32::NAN, &mut time);
        assert!(!time.is_paused());
        assert_eq!(time.relative_speed(), 0.0);
    }
}
