use bevy::prelude::{ResMut, Resource};

use crate::{
    network_commands::{
        AddLocalPlayerPacket, ClearPlayerListPacket, CommandPayload, DeleteLocalPlayerPacket,
        PlayerIdPacket, RequestPlayerIdCommand, RequestPlayerListCommand,
        SetLocalPlayerIgnoredPacket, SetLocalPlayerLogInfoPacket, SetLocalPlayerModePacket,
        SetLocalPlayerNamePacket, SetLocalPlayerTeamPacket, SetLocalPlayerVoteInfoPacket,
        SetNameCommand, SetPlayerModeCommand, SetTeamCommand,
    },
    news::NewsLog,
    original::types::TeamType,
};

const LOCAL_PLAYER_ID: i32 = 0;
const GAMES_PER_VOTING_POWER: i32 = 5;
const P_NULL_VOTE: i32 = 0;
const P_MAX_VOTE_CHOICES: i32 = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum PlayerMode {
    Nobody = 0,
    Player = 1,
    Bot = 2,
    Spectator = 3,
    Tray = 4,
}

#[derive(Clone, Debug, Resource)]
pub(crate) struct LocalPlayerState {
    player_id: i32,
    player_name: String,
    desired_team: TeamType,
    our_mode: PlayerMode,
    name: String,
    team: TeamType,
    mode: PlayerMode,
    ignored: bool,
    db_id: i32,
    logged_in: bool,
    activated: bool,
    voting_power: i32,
    total_games: i32,
    bot_logged_in: bool,
    vote_choice: i32,
    players: Vec<LocalPlayerInfo>,
}

#[derive(Default, Resource)]
pub(crate) struct LocalPlayerInfoInitialSendState {
    sent: bool,
}

#[derive(Default, Resource)]
pub(crate) struct LocalPlayerListInitialRequestState {
    requested: bool,
}

#[derive(Default, Resource)]
pub(crate) struct LocalPlayerPacketQueue {
    pub(crate) delete_player_ids: Vec<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalPlayerInfo {
    player_id: i32,
    name: String,
    team: TeamType,
    mode: PlayerMode,
    ignored: bool,
    db_id: i32,
    logged_in: bool,
    activated: bool,
    bot_logged_in: bool,
    voting_power: i32,
    total_games: i32,
    vote_choice: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReshuffleTeamsResult {
    NoPlayers,
    NoAvailableTeams,
    Changed { player_count: usize },
}

impl LocalPlayerInfo {
    fn new(player_id: i32) -> Self {
        Self {
            player_id,
            name: String::new(),
            team: TeamType::Null,
            mode: PlayerMode::Nobody,
            ignored: false,
            db_id: -1,
            logged_in: false,
            activated: false,
            bot_logged_in: false,
            voting_power: 0,
            total_games: 0,
            vote_choice: P_NULL_VOTE,
        }
    }
}

impl Default for LocalPlayerState {
    fn default() -> Self {
        Self {
            player_id: LOCAL_PLAYER_ID,
            player_name: "Player".to_string(),
            desired_team: TeamType::Red,
            our_mode: PlayerMode::Player,
            name: String::new(),
            team: TeamType::Null,
            mode: PlayerMode::Nobody,
            ignored: false,
            db_id: -1,
            logged_in: false,
            activated: false,
            voting_power: 0,
            total_games: 0,
            bot_logged_in: false,
            vote_choice: P_NULL_VOTE,
            players: Vec::new(),
        }
    }
}

impl LocalPlayerState {
    pub(crate) fn name(&self) -> &str {
        self.our_player_info()
            .and_then(|player| (!player.name.is_empty()).then_some(player.name.as_str()))
            .or_else(|| (!self.name.is_empty()).then_some(self.name.as_str()))
            .unwrap_or(&self.player_name)
    }

    pub(crate) fn team(&self) -> TeamType {
        self.our_player_info()
            .map(|player| player.team)
            .filter(|team| *team != TeamType::Null)
            .or_else(|| (self.team != TeamType::Null).then_some(self.team))
            .unwrap_or(self.desired_team)
    }

    pub(crate) fn logged_in(&self) -> bool {
        self.our_player_info()
            .map(|player| player.logged_in)
            .unwrap_or(self.logged_in)
    }

    pub(crate) fn activated(&self) -> bool {
        self.our_player_info()
            .map(|player| player.activated)
            .unwrap_or(self.activated)
    }

    pub(crate) fn bot_logged_in(&self) -> bool {
        self.our_player_info()
            .map(|player| player.bot_logged_in)
            .unwrap_or(self.bot_logged_in)
    }

    pub(crate) fn voting_power(&self) -> i32 {
        self.our_player_info()
            .map(|player| player.voting_power)
            .unwrap_or(self.voting_power)
    }

    pub(crate) fn db_id(&self) -> i32 {
        self.our_player_info()
            .map(|player| player.db_id)
            .unwrap_or(self.db_id)
    }

    pub(crate) fn real_voting_power(&self) -> i32 {
        self.our_player_info()
            .map(LocalPlayerInfo::real_voting_power)
            .unwrap_or_else(|| self.voting_power + (self.total_games / GAMES_PER_VOTING_POWER))
    }

    fn our_player_info(&self) -> Option<&LocalPlayerInfo> {
        self.players
            .iter()
            .find(|player| player.player_id == self.player_id)
    }

    fn player_info_mut(&mut self, player_id: i32) -> Option<&mut LocalPlayerInfo> {
        self.players
            .iter_mut()
            .find(|player| player.player_id == player_id)
    }
}

impl LocalPlayerInfo {
    fn real_voting_power(&self) -> i32 {
        self.voting_power + (self.total_games / GAMES_PER_VOTING_POWER)
    }
}

impl PlayerMode {
    fn from_wire_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Nobody),
            1 => Some(Self::Player),
            2 => Some(Self::Bot),
            3 => Some(Self::Spectator),
            4 => Some(Self::Tray),
            _ => None,
        }
    }

    fn from_wire_char(value: i8) -> Option<Self> {
        Self::from_wire_i32(i32::from(value))
    }

    fn wire_i32(self) -> i32 {
        self as i32
    }

    fn wire_char(self) -> i8 {
        self as i8
    }
}

pub(crate) fn process_initial_player_info_send(
    mut initial_send: ResMut<LocalPlayerInfoInitialSendState>,
    mut local_player: ResMut<LocalPlayerState>,
    mut news_log: ResMut<NewsLog>,
) {
    if initial_send.sent {
        return;
    }

    relay_send_player_info(&mut local_player, &mut news_log);
    initial_send.sent = true;
}

pub(crate) fn process_initial_player_list_request(
    mut initial_request: ResMut<LocalPlayerListInitialRequestState>,
    mut local_player: ResMut<LocalPlayerState>,
) {
    if initial_request.requested {
        return;
    }

    relay_request_player_list(&mut local_player);
    initial_request.requested = true;
}

pub(crate) fn process_local_player_packet_queue(
    mut local_player: ResMut<LocalPlayerState>,
    mut packet_queue: ResMut<LocalPlayerPacketQueue>,
) {
    let delete_player_ids = std::mem::take(&mut packet_queue.delete_player_ids);
    for player_id in delete_player_ids {
        relay_delete_local_player(&mut local_player, player_id);
    }
}

pub(crate) fn relay_send_player_info(
    local_player: &mut LocalPlayerState,
    news_log: &mut NewsLog,
) -> bool {
    let name = local_player.player_name.clone();
    let name_ok = SetNameCommand::new(name)
        .is_some_and(|command| relay_set_player_name(local_player, news_log, command));
    let team_ok = relay_set_player_team(
        local_player,
        SetTeamCommand {
            team: team_wire(local_player.desired_team),
        },
    );
    let mode_ok = relay_set_player_mode(
        local_player,
        SetPlayerModeCommand {
            mode: local_player.our_mode.wire_char(),
        },
    );
    name_ok && team_ok && mode_ok
}

pub(crate) fn relay_request_player_list(local_player: &mut LocalPlayerState) -> bool {
    let player_id_ok = RequestPlayerIdCommand
        .encode_packet()
        .get(8..)
        .is_some_and(|payload| payload.is_empty())
        && relay_give_player_id(local_player, local_player.player_id);
    let list_ok = RequestPlayerListCommand
        .encode_packet()
        .get(8..)
        .is_some_and(|payload| payload.is_empty())
        && relay_player_list_contents(local_player);

    player_id_ok && list_ok
}

fn relay_give_player_id(local_player: &mut LocalPlayerState, player_id: i32) -> bool {
    let packet = PlayerIdPacket { player_id };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = PlayerIdPacket::decode_payload(payload) else {
        return false;
    };
    apply_player_id(local_player, decoded_packet)
}

fn relay_player_list_contents(local_player: &mut LocalPlayerState) -> bool {
    let clear_packet = ClearPlayerListPacket;
    let wire_packet = clear_packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_clear) = ClearPlayerListPacket::decode_payload(payload) else {
        return false;
    };
    if !apply_clear_player_list(local_player, decoded_clear) {
        return false;
    }

    relay_add_local_player(local_player, local_player.player_id)
        && relay_set_local_player_name(local_player)
        && relay_set_local_player_team(local_player)
        && relay_set_local_player_mode(local_player)
        && relay_set_local_player_ignored(local_player)
        && relay_set_local_player_loginfo(local_player)
}

fn relay_add_local_player(local_player: &mut LocalPlayerState, player_id: i32) -> bool {
    let packet = AddLocalPlayerPacket { player_id };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = AddLocalPlayerPacket::decode_payload(payload) else {
        return false;
    };
    apply_add_local_player(local_player, decoded_packet)
}

fn relay_delete_local_player(local_player: &mut LocalPlayerState, player_id: i32) -> bool {
    let packet = DeleteLocalPlayerPacket { player_id };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = DeleteLocalPlayerPacket::decode_payload(payload) else {
        return false;
    };
    apply_delete_local_player(local_player, decoded_packet)
}

fn relay_set_player_name(
    local_player: &mut LocalPlayerState,
    news_log: &mut NewsLog,
    command: SetNameCommand,
) -> bool {
    let wire_packet = command.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_command) = SetNameCommand::decode_payload(payload) else {
        return false;
    };

    let old_name = local_player.name.clone();
    local_player.name = decoded_command.name;
    news_log.relay_source_news(
        format!(
            "player '{}' set their name to '{}'",
            old_name, local_player.name
        ),
        0,
        0,
        0,
    );

    relay_set_local_player_name(local_player)
}

fn relay_set_player_team(local_player: &mut LocalPlayerState, command: SetTeamCommand) -> bool {
    let wire_packet = command.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_command) = SetTeamCommand::decode_payload(payload) else {
        return false;
    };
    let Some(team) = team_from_wire(decoded_command.team) else {
        return false;
    };

    local_player.team = team;
    relay_set_local_player_team(local_player)
}

pub(crate) fn relay_change_player_team(
    local_player: &mut LocalPlayerState,
    news_log: &mut NewsLog,
    team: TeamType,
) -> bool {
    let previous_team = local_player.team();
    let player_name = local_player.name().to_string();
    if !relay_set_player_team(
        local_player,
        SetTeamCommand {
            team: team_wire(team),
        },
    ) {
        return false;
    }

    news_log.relay_source_news(
        format!("you have been set to the {} team", team.asset_name()),
        0,
        0,
        0,
    );
    news_log.relay_source_news(
        format!(
            "{player_name} has changed from the {} team to the {} team",
            previous_team.asset_name(),
            team.asset_name()
        ),
        0,
        0,
        0,
    );
    true
}

pub(crate) fn relay_reshuffle_player_teams(
    local_player: &mut LocalPlayerState,
    news_log: &mut NewsLog,
    available_teams: &[TeamType],
    mut choose_index: impl FnMut(usize) -> usize,
) -> ReshuffleTeamsResult {
    let mut logged_in_players = Vec::new();
    let mut other_players = Vec::new();
    for player in &local_player.players {
        if player.mode != PlayerMode::Player {
            continue;
        }
        if player.logged_in {
            logged_in_players.push(player.player_id);
        } else {
            other_players.push(player.player_id);
        }
    }

    if logged_in_players.is_empty() && other_players.is_empty() {
        return ReshuffleTeamsResult::NoPlayers;
    }
    if available_teams.is_empty() {
        return ReshuffleTeamsResult::NoAvailableTeams;
    }

    let original_teams = available_teams.to_vec();
    let mut remaining_teams = original_teams.clone();
    let player_count = logged_in_players.len() + other_players.len();
    for player_id in logged_in_players.into_iter().chain(other_players) {
        if let Some(player) = local_player.player_info_mut(player_id) {
            player.team = TeamType::Null;
        }
        if player_id == local_player.player_id {
            local_player.team = TeamType::Null;
        }

        if remaining_teams.is_empty() {
            remaining_teams.clone_from(&original_teams);
        }
        let team_index = choose_index(remaining_teams.len()) % remaining_teams.len();
        let team = remaining_teams.remove(team_index);
        relay_change_roster_player_team(local_player, news_log, player_id, team);
    }

    ReshuffleTeamsResult::Changed { player_count }
}

fn relay_change_roster_player_team(
    local_player: &mut LocalPlayerState,
    news_log: &mut NewsLog,
    player_id: i32,
    team: TeamType,
) -> bool {
    let packet = SetLocalPlayerTeamPacket {
        player_id,
        team: team_wire(team),
    };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = SetLocalPlayerTeamPacket::decode_payload(payload) else {
        return false;
    };
    if !apply_set_local_player_team(local_player, decoded_packet) {
        return false;
    }

    if player_id != local_player.player_id {
        return true;
    }
    let command = SetTeamCommand {
        team: team_wire(team),
    };
    let wire_packet = command.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_command) = SetTeamCommand::decode_payload(payload) else {
        return false;
    };
    let Some(decoded_team) = team_from_wire(decoded_command.team) else {
        return false;
    };
    local_player.team = decoded_team;
    news_log.relay_source_news(
        format!(
            "you have been set to the {} team",
            decoded_team.asset_name()
        ),
        0,
        0,
        0,
    );
    true
}

pub(crate) fn relay_account_login(
    local_player: &mut LocalPlayerState,
    news_log: &mut NewsLog,
    user_name: &str,
    db_id: i32,
    activated: bool,
    voting_power: i32,
    total_games: i32,
) -> bool {
    if local_player.logged_in() {
        return false;
    }

    local_player.name = user_name.to_string();
    local_player.db_id = db_id;
    local_player.activated = activated;
    local_player.voting_power = voting_power;
    local_player.total_games = total_games;
    local_player.logged_in = true;
    news_log.relay_source_news(format!("{user_name} logged in"), 0, 0, 0);

    relay_set_local_player_name(local_player)
        && relay_set_local_player_loginfo(local_player)
        && relay_set_local_player_vote_choice(local_player)
}

pub(crate) fn relay_account_logout(
    local_player: &mut LocalPlayerState,
    news_log: &mut NewsLog,
) -> bool {
    if !local_player.logged_in() {
        return true;
    }

    let user_name = local_player.name().to_string();
    news_log.relay_source_news(format!("{user_name} logged out"), 0, 0, 0);
    local_player.name.clear();
    local_player.logged_in = false;
    local_player.db_id = -1;
    local_player.activated = false;
    local_player.voting_power = 0;
    local_player.total_games = 0;
    local_player.bot_logged_in = false;

    relay_set_local_player_name(local_player)
        && relay_set_local_player_loginfo(local_player)
        && relay_set_local_player_vote_choice(local_player)
}

pub(crate) fn relay_spend_voting_power(local_player: &mut LocalPlayerState, amount: i32) -> bool {
    if amount < 0 || local_player.voting_power() < amount {
        return false;
    }
    local_player.voting_power -= amount;
    relay_set_local_player_loginfo(local_player)
}

fn relay_set_player_mode(
    local_player: &mut LocalPlayerState,
    command: SetPlayerModeCommand,
) -> bool {
    let wire_packet = command.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_command) = SetPlayerModeCommand::decode_payload(payload) else {
        return false;
    };
    let Some(mode) = PlayerMode::from_wire_char(decoded_command.mode) else {
        return false;
    };

    local_player.mode = mode;
    relay_set_local_player_mode(local_player)
}

fn relay_set_local_player_name(local_player: &mut LocalPlayerState) -> bool {
    let packet = SetLocalPlayerNamePacket {
        player_id: local_player.player_id,
        name: local_player.name.clone(),
    };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = SetLocalPlayerNamePacket::decode_payload(payload) else {
        return false;
    };
    apply_set_local_player_name(local_player, decoded_packet)
}

fn relay_set_local_player_team(local_player: &mut LocalPlayerState) -> bool {
    let packet = SetLocalPlayerTeamPacket {
        player_id: local_player.player_id,
        team: team_wire(local_player.team),
    };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = SetLocalPlayerTeamPacket::decode_payload(payload) else {
        return false;
    };
    apply_set_local_player_team(local_player, decoded_packet)
}

fn relay_set_local_player_mode(local_player: &mut LocalPlayerState) -> bool {
    let packet = SetLocalPlayerModePacket {
        player_id: local_player.player_id,
        mode: local_player.mode.wire_i32(),
    };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = SetLocalPlayerModePacket::decode_payload(payload) else {
        return false;
    };
    apply_set_local_player_mode(local_player, decoded_packet)
}

fn relay_set_local_player_ignored(local_player: &mut LocalPlayerState) -> bool {
    let packet = SetLocalPlayerIgnoredPacket {
        player_id: local_player.player_id,
        ignored: i32::from(local_player.ignored),
    };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = SetLocalPlayerIgnoredPacket::decode_payload(payload) else {
        return false;
    };
    apply_set_local_player_ignored(local_player, decoded_packet)
}

fn relay_set_local_player_loginfo(local_player: &mut LocalPlayerState) -> bool {
    let packet = SetLocalPlayerLogInfoPacket {
        player_id: local_player.player_id,
        db_id: local_player.db_id,
        voting_power: local_player.voting_power,
        total_games: local_player.total_games,
        activated: local_player.activated,
        logged_in: local_player.logged_in,
        bot_logged_in: local_player.bot_logged_in,
    };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = SetLocalPlayerLogInfoPacket::decode_payload(payload) else {
        return false;
    };
    apply_set_local_player_loginfo(local_player, decoded_packet)
}

fn relay_set_local_player_vote_choice(local_player: &mut LocalPlayerState) -> bool {
    let packet = SetLocalPlayerVoteInfoPacket {
        player_id: local_player.player_id,
        vote_choice: local_player.vote_choice,
    };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return false;
    };
    let Some(decoded_packet) = SetLocalPlayerVoteInfoPacket::decode_payload(payload) else {
        return false;
    };
    apply_set_local_player_voteinfo(local_player, decoded_packet)
}

fn apply_set_local_player_name(
    local_player: &mut LocalPlayerState,
    packet: SetLocalPlayerNamePacket,
) -> bool {
    if packet.player_id == local_player.player_id {
        local_player.name = packet.name.clone();
    }
    if let Some(player) = local_player.player_info_mut(packet.player_id) {
        player.name = packet.name;
    }
    true
}

fn apply_set_local_player_team(
    local_player: &mut LocalPlayerState,
    packet: SetLocalPlayerTeamPacket,
) -> bool {
    let Some(team) = team_from_wire(packet.team) else {
        return false;
    };
    if packet.player_id == local_player.player_id {
        local_player.team = team;
    }
    if let Some(player) = local_player.player_info_mut(packet.player_id) {
        player.team = team;
    }
    true
}

fn apply_set_local_player_mode(
    local_player: &mut LocalPlayerState,
    packet: SetLocalPlayerModePacket,
) -> bool {
    let Some(mode) = PlayerMode::from_wire_i32(packet.mode) else {
        return false;
    };
    if packet.player_id == local_player.player_id {
        local_player.mode = mode;
    }
    if let Some(player) = local_player.player_info_mut(packet.player_id) {
        player.mode = mode;
    }
    true
}

fn apply_set_local_player_ignored(
    local_player: &mut LocalPlayerState,
    packet: SetLocalPlayerIgnoredPacket,
) -> bool {
    if packet.ignored < 0 || packet.ignored >= 5 {
        return false;
    }
    let ignored = packet.ignored != 0;
    if packet.player_id == local_player.player_id {
        local_player.ignored = ignored;
    }
    if let Some(player) = local_player.player_info_mut(packet.player_id) {
        player.ignored = ignored;
    }
    true
}

fn apply_set_local_player_loginfo(
    local_player: &mut LocalPlayerState,
    packet: SetLocalPlayerLogInfoPacket,
) -> bool {
    if packet.player_id == local_player.player_id {
        local_player.db_id = packet.db_id;
        local_player.voting_power = packet.voting_power;
        local_player.total_games = packet.total_games;
        local_player.activated = packet.activated;
        local_player.logged_in = packet.logged_in;
        local_player.bot_logged_in = packet.bot_logged_in;
    }
    if let Some(player) = local_player.player_info_mut(packet.player_id) {
        player.db_id = packet.db_id;
        player.voting_power = packet.voting_power;
        player.total_games = packet.total_games;
        player.activated = packet.activated;
        player.logged_in = packet.logged_in;
        player.bot_logged_in = packet.bot_logged_in;
    }
    true
}

pub(crate) fn apply_set_local_player_voteinfo(
    local_player: &mut LocalPlayerState,
    packet: SetLocalPlayerVoteInfoPacket,
) -> bool {
    if !(P_NULL_VOTE..P_MAX_VOTE_CHOICES).contains(&packet.vote_choice) {
        return false;
    }
    if packet.player_id == local_player.player_id && local_player.vote_choice != packet.vote_choice
    {
        local_player.vote_choice = packet.vote_choice;
    }
    if let Some(player) = local_player.player_info_mut(packet.player_id) {
        if player.vote_choice != packet.vote_choice {
            player.vote_choice = packet.vote_choice;
        }
    }
    true
}

fn apply_player_id(local_player: &mut LocalPlayerState, packet: PlayerIdPacket) -> bool {
    local_player.player_id = packet.player_id;
    true
}

fn apply_clear_player_list(
    local_player: &mut LocalPlayerState,
    _packet: ClearPlayerListPacket,
) -> bool {
    local_player.players.clear();
    true
}

fn apply_add_local_player(
    local_player: &mut LocalPlayerState,
    packet: AddLocalPlayerPacket,
) -> bool {
    local_player
        .players
        .push(LocalPlayerInfo::new(packet.player_id));
    true
}

fn apply_delete_local_player(
    local_player: &mut LocalPlayerState,
    packet: DeleteLocalPlayerPacket,
) -> bool {
    local_player
        .players
        .retain(|player| player.player_id != packet.player_id);
    true
}

fn team_wire(team: TeamType) -> i32 {
    team as i8 as i32
}

fn team_from_wire(value: i32) -> Option<TeamType> {
    match value {
        0 => Some(TeamType::Null),
        1 => Some(TeamType::Red),
        2 => Some(TeamType::Blue),
        3 => Some(TeamType::Green),
        4 => Some(TeamType::Yellow),
        5 => Some(TeamType::Purple),
        6 => Some(TeamType::Teal),
        7 => Some(TeamType::White),
        8 => Some(TeamType::Black),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_local_player_matches_zplayer_constructor_shape() {
        let player = LocalPlayerState::default();

        assert_eq!(player.name(), "Player");
        assert_eq!(player.team(), TeamType::Red);
        assert_eq!(player.our_mode, PlayerMode::Player);
        assert_eq!(player.mode, PlayerMode::Nobody);
    }

    #[test]
    fn send_player_info_relays_name_team_and_mode_to_local_player() {
        let mut player = LocalPlayerState::default();
        let mut news_log = NewsLog::default();

        assert!(relay_send_player_info(&mut player, &mut news_log));

        assert_eq!(player.name(), "Player");
        assert_eq!(player.team(), TeamType::Red);
        assert_eq!(player.mode, PlayerMode::Player);
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("player '' set their name to 'Player'")
        );
    }

    #[test]
    fn reshuffle_assigns_logged_players_first_and_relays_local_team() {
        let mut player = LocalPlayerState::default();
        let mut news_log = NewsLog::default();
        assert!(relay_send_player_info(&mut player, &mut news_log));
        assert!(relay_request_player_list(&mut player));
        assert!(apply_set_local_player_loginfo(
            &mut player,
            SetLocalPlayerLogInfoPacket {
                player_id: LOCAL_PLAYER_ID,
                db_id: 1,
                voting_power: 1,
                total_games: 0,
                activated: true,
                logged_in: true,
                bot_logged_in: false,
            },
        ));

        let result = relay_reshuffle_player_teams(
            &mut player,
            &mut news_log,
            &[TeamType::Blue, TeamType::Green],
            |_| 1,
        );

        assert_eq!(result, ReshuffleTeamsResult::Changed { player_count: 1 });
        assert_eq!(player.team(), TeamType::Green);
        assert_eq!(player.players[0].team, TeamType::Green);
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("you have been set to the green team")
        );
    }

    #[test]
    fn reshuffle_reports_source_empty_player_and_team_errors() {
        let mut player = LocalPlayerState::default();
        let mut news_log = NewsLog::default();
        assert_eq!(
            relay_reshuffle_player_teams(&mut player, &mut news_log, &[TeamType::Red], |_| 0,),
            ReshuffleTeamsResult::NoPlayers
        );

        assert!(relay_send_player_info(&mut player, &mut news_log));
        assert!(relay_request_player_list(&mut player));
        assert_eq!(
            relay_reshuffle_player_teams(&mut player, &mut news_log, &[], |_| 0),
            ReshuffleTeamsResult::NoAvailableTeams
        );
    }

    #[test]
    fn request_player_list_relays_player_id_clear_add_and_current_info() {
        let mut player = LocalPlayerState::default();
        let mut news_log = NewsLog::default();
        assert!(relay_send_player_info(&mut player, &mut news_log));

        assert!(relay_request_player_list(&mut player));

        assert_eq!(player.player_id, LOCAL_PLAYER_ID);
        assert_eq!(player.players.len(), 1);
        assert_eq!(
            player.players[0],
            LocalPlayerInfo {
                player_id: LOCAL_PLAYER_ID,
                name: "Player".to_string(),
                team: TeamType::Red,
                mode: PlayerMode::Player,
                ignored: false,
                db_id: -1,
                logged_in: false,
                activated: false,
                bot_logged_in: false,
                voting_power: 0,
                total_games: 0,
                vote_choice: P_NULL_VOTE,
            }
        );
        assert_eq!(player.name(), "Player");
        assert_eq!(player.team(), TeamType::Red);
    }

    #[test]
    fn clear_and_add_player_list_match_source_client_vector_shape() {
        let mut player = LocalPlayerState::default();
        assert!(apply_add_local_player(
            &mut player,
            AddLocalPlayerPacket { player_id: 1 },
        ));
        assert!(apply_add_local_player(
            &mut player,
            AddLocalPlayerPacket { player_id: 2 },
        ));
        assert_eq!(player.players.len(), 2);

        assert!(apply_clear_player_list(&mut player, ClearPlayerListPacket));
        assert!(player.players.is_empty());

        assert!(apply_add_local_player(
            &mut player,
            AddLocalPlayerPacket { player_id: 2 },
        ));
        assert_eq!(player.players, vec![LocalPlayerInfo::new(2)]);
    }

    #[test]
    fn delete_player_removes_all_matching_roster_entries_like_source_loop() {
        let mut player = LocalPlayerState::default();
        assert!(apply_add_local_player(
            &mut player,
            AddLocalPlayerPacket { player_id: 2 },
        ));
        assert!(apply_add_local_player(
            &mut player,
            AddLocalPlayerPacket { player_id: 2 },
        ));
        assert!(apply_add_local_player(
            &mut player,
            AddLocalPlayerPacket { player_id: 3 },
        ));

        assert!(apply_delete_local_player(
            &mut player,
            DeleteLocalPlayerPacket { player_id: 2 },
        ));

        assert_eq!(player.players, vec![LocalPlayerInfo::new(3)]);
    }

    #[test]
    fn ignored_packet_accepts_source_int_range_and_updates_roster() {
        let mut player = LocalPlayerState::default();
        assert!(apply_add_local_player(
            &mut player,
            AddLocalPlayerPacket {
                player_id: LOCAL_PLAYER_ID,
            },
        ));

        assert!(apply_set_local_player_ignored(
            &mut player,
            SetLocalPlayerIgnoredPacket {
                player_id: LOCAL_PLAYER_ID,
                ignored: 4,
            },
        ));
        assert!(player.ignored);
        assert!(player.players[0].ignored);

        assert!(!apply_set_local_player_ignored(
            &mut player,
            SetLocalPlayerIgnoredPacket {
                player_id: LOCAL_PLAYER_ID,
                ignored: 5,
            },
        ));
        assert!(player.ignored);
    }

    #[test]
    fn loginfo_packet_updates_playerinfo_getters_from_roster() {
        let mut player = LocalPlayerState::default();
        assert!(relay_send_player_info(&mut player, &mut NewsLog::default()));
        assert!(relay_request_player_list(&mut player));

        assert!(apply_set_local_player_loginfo(
            &mut player,
            SetLocalPlayerLogInfoPacket {
                player_id: LOCAL_PLAYER_ID,
                db_id: 10,
                voting_power: 7,
                total_games: 15,
                activated: true,
                logged_in: true,
                bot_logged_in: false,
            },
        ));

        assert!(player.logged_in());
        assert!(player.activated());
        assert_eq!(player.voting_power(), 7);
        assert_eq!(player.real_voting_power(), 10);
        assert_eq!(player.players[0].db_id, 10);
    }

    #[test]
    fn voteinfo_packet_updates_playerinfo_vote_choice_like_source_client() {
        let mut player = LocalPlayerState::default();
        assert!(relay_send_player_info(&mut player, &mut NewsLog::default()));
        assert!(relay_request_player_list(&mut player));

        assert!(apply_set_local_player_voteinfo(
            &mut player,
            SetLocalPlayerVoteInfoPacket {
                player_id: LOCAL_PLAYER_ID,
                vote_choice: 1,
            },
        ));
        assert_eq!(player.vote_choice, 1);
        assert_eq!(player.players[0].vote_choice, 1);

        assert!(!apply_set_local_player_voteinfo(
            &mut player,
            SetLocalPlayerVoteInfoPacket {
                player_id: LOCAL_PLAYER_ID,
                vote_choice: P_MAX_VOTE_CHOICES,
            },
        ));
        assert_eq!(player.vote_choice, 1);
        assert_eq!(player.players[0].vote_choice, 1);

        assert!(apply_set_local_player_voteinfo(
            &mut player,
            SetLocalPlayerVoteInfoPacket {
                player_id: 99,
                vote_choice: 2,
            },
        ));
        assert_eq!(player.vote_choice, 1);
        assert_eq!(player.players[0].vote_choice, 1);
    }

    #[test]
    fn player_id_packet_updates_our_player_id() {
        let mut player = LocalPlayerState::default();

        assert!(apply_player_id(
            &mut player,
            PlayerIdPacket { player_id: 3 }
        ));

        assert_eq!(player.player_id, 3);
    }

    #[test]
    fn set_name_command_truncates_at_source_max_name_size() {
        let long_name = "abcdefghijklmnopqrstuvwxyz1234567890";
        let command = SetNameCommand::new(long_name).unwrap();

        let decoded = SetNameCommand::decode_payload(&command.encode_payload()).unwrap();

        assert_eq!(decoded.name, "abcdefghijklmnopqrstuvwxyz1234");
    }

    #[test]
    fn invalid_team_or_mode_packets_do_not_mutate_local_player() {
        let mut player = LocalPlayerState::default();

        assert!(!apply_set_local_player_team(
            &mut player,
            SetLocalPlayerTeamPacket {
                player_id: LOCAL_PLAYER_ID,
                team: 99,
            },
        ));
        assert_eq!(player.team(), TeamType::Red);

        assert!(!apply_set_local_player_mode(
            &mut player,
            SetLocalPlayerModePacket {
                player_id: LOCAL_PLAYER_ID,
                mode: 99,
            },
        ));
        assert_eq!(player.mode, PlayerMode::Nobody);
    }

    #[test]
    fn change_player_team_round_trips_request_roster_and_source_news_order() {
        let mut player = LocalPlayerState::default();
        assert!(relay_send_player_info(&mut player, &mut NewsLog::default()));
        assert!(relay_request_player_list(&mut player));
        let mut news = NewsLog::default();

        assert!(relay_change_player_team(
            &mut player,
            &mut news,
            TeamType::Blue
        ));

        assert_eq!(player.team(), TeamType::Blue);
        assert_eq!(player.players[0].team, TeamType::Blue);
        assert_eq!(
            news.display_entry(1).map(|entry| entry.message),
            Some("you have been set to the blue team")
        );
        assert_eq!(
            news.display_entry(0).map(|entry| entry.message),
            Some("Player has changed from the red team to the blue team")
        );
    }
}
