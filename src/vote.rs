use bevy::prelude::Resource;

use crate::components::GamePauseUpdate;
use crate::network_commands::{CommandPayload, SetLocalPlayerVoteInfoPacket, VoteInfoPacket};
use crate::original::types::TeamType;
use crate::perpetual_settings::PerpetualServerSettings;

const P_NULL_VOTE: i32 = 0;
const P_YES_VOTE: i32 = 1;
const P_NO_VOTE: i32 = 2;
const P_PASS_VOTE: i32 = 3;
const P_MAX_VOTE_CHOICES: i32 = 4;
const GAMES_PER_VOTING_POWER: i32 = 5;
const PAUSE_VOTE_VALUE: i32 = -1;
const MAX_VOTE_TIME_SECONDS: f32 = 30.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VoteType {
    PauseGame,
    ResumeGame,
    ChangeMap,
    StartBot,
    StopBot,
    ResetGame,
    ReshuffleTeams,
    ChangeGameSpeed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VoteChoice {
    Yes,
    No,
    Pass,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VoteDisplaySnapshot {
    pub(crate) description: String,
    pub(crate) have_votes: i32,
    pub(crate) needed_votes: i32,
    pub(crate) for_votes: i32,
    pub(crate) against_votes: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct VoteChoiceOutcome {
    pub(crate) pause_update: Option<GamePauseUpdate>,
    pub(crate) game_speed_percent_update: Option<i32>,
    pub(crate) non_pause_action: Option<NonPauseVoteAction>,
    pub(crate) news_message: Option<&'static str>,
    pub(crate) player_vote_infos: Vec<SetLocalPlayerVoteInfoPacket>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct VoteRequestOutcome {
    pub(crate) pause_update: Option<GamePauseUpdate>,
    pub(crate) game_speed_percent_update: Option<i32>,
    pub(crate) non_pause_action: Option<NonPauseVoteAction>,
    pub(crate) news_message: Option<String>,
    pub(crate) player_vote_infos: Vec<SetLocalPlayerVoteInfoPacket>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct VoteExpirationOutcome {
    pub(crate) expired: bool,
    pub(crate) player_vote_infos: Vec<SetLocalPlayerVoteInfoPacket>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct VoteServerAction {
    processed_vote: Option<ProcessedVote>,
    player_vote_infos: Vec<SetLocalPlayerVoteInfoPacket>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProcessedVote {
    vote_type: VoteType,
    value: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ProcessedVoteOutcome {
    pause_update: Option<GamePauseUpdate>,
    game_speed_percent_update: Option<i32>,
    non_pause_action: Option<NonPauseVoteAction>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NonPauseVoteRequest {
    ChangeMap { map_num: i32 },
    StartBot { team: i32 },
    StopBot { team: i32 },
    ResetGame,
    ReshuffleTeams,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NonPauseVoteAction {
    ChangeMap { map_num: usize },
    StartBot { team: i32 },
    StopBot { team: i32 },
    ResetGame,
    ReshuffleTeams,
}

#[derive(Default, Resource)]
pub(crate) struct NonPauseVoteRequestQueue {
    pub(crate) pending: Vec<NonPauseVoteRequest>,
}

#[derive(Default, Resource)]
pub(crate) struct NonPauseVoteActionQueue {
    pub(crate) pending: Vec<NonPauseVoteAction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub(crate) struct LocalBotTeams {
    started: [bool; 9],
    ignored: [bool; 9],
}

impl Default for LocalBotTeams {
    fn default() -> Self {
        Self::from_settings(&PerpetualServerSettings::default())
    }
}

impl LocalBotTeams {
    pub(crate) fn from_settings(settings: &PerpetualServerSettings) -> Self {
        Self {
            started: [false; 9],
            ignored: [settings.bots_start_ignored; 9],
        }
    }

    pub(crate) fn active_teams(&self) -> Vec<TeamType> {
        (1_i8..=8)
            .filter(|team| self.started[*team as usize] && !self.ignored[*team as usize])
            .filter_map(|team| TeamType::try_from(team).ok())
            .collect()
    }

    pub(crate) fn set_active(&mut self, team: i32, active: bool) -> bool {
        let Ok(team_index) = usize::try_from(team) else {
            return false;
        };
        if !(1..self.started.len()).contains(&team_index) {
            return false;
        }
        if active {
            self.started[team_index] = true;
            self.ignored[team_index] = false;
        } else {
            self.ignored[team_index] = true;
        }
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LocalVotePlayer {
    p_id: i32,
    name: &'static str,
    is_player: bool,
    vote_choice: i32,
    voting_power: i32,
    total_games: i32,
    logged_in: bool,
}

impl LocalVotePlayer {
    fn local_player() -> Self {
        Self {
            p_id: 0,
            name: "Player",
            is_player: true,
            vote_choice: P_NULL_VOTE,
            voting_power: 0,
            total_games: 0,
            logged_in: false,
        }
    }

    fn real_voting_power(self) -> i32 {
        self.voting_power + (self.total_games / GAMES_PER_VOTING_POWER)
    }
}

#[derive(Clone, Debug, Resource)]
pub(crate) struct LocalVotePlayers {
    players: Vec<LocalVotePlayer>,
}

impl Default for LocalVotePlayers {
    fn default() -> Self {
        Self {
            players: vec![LocalVotePlayer::local_player()],
        }
    }
}

#[derive(Clone, Copy, Debug, Resource)]
pub(crate) struct LocalVoteSettings {
    require_login: bool,
    allow_game_speed_change: bool,
}

impl Default for LocalVoteSettings {
    fn default() -> Self {
        Self::from_settings(&PerpetualServerSettings::default())
    }
}

impl LocalVoteSettings {
    pub(crate) fn from_settings(settings: &PerpetualServerSettings) -> Self {
        Self {
            require_login: settings.require_login,
            allow_game_speed_change: settings.allow_game_speed_change,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub(crate) struct GameVoteState {
    in_progress: bool,
    vote_type: Option<VoteType>,
    value: i32,
    elapsed_seconds: f32,
}

impl Default for GameVoteState {
    fn default() -> Self {
        Self {
            in_progress: false,
            vote_type: None,
            value: -1,
            elapsed_seconds: 0.0,
        }
    }
}

impl GameVoteState {
    fn apply_vote_info(&mut self, packet: VoteInfoPacket) {
        if !packet.in_progress {
            *self = Self::default();
            return;
        }

        self.in_progress = true;
        self.vote_type = VoteType::from_wire_id(packet.vote_type);
        self.value = packet.value;
    }
}

pub(crate) fn tick_vote_expiration(
    vote: &mut GameVoteState,
    players: &mut LocalVotePlayers,
    delta_seconds: f32,
) -> VoteExpirationOutcome {
    if !vote.in_progress {
        return VoteExpirationOutcome::default();
    }

    vote.elapsed_seconds += delta_seconds.max(0.0);
    if vote.elapsed_seconds < MAX_VOTE_TIME_SECONDS {
        return VoteExpirationOutcome::default();
    }

    let player_vote_infos = kill_vote(vote, players);
    relay_vote_info(vote);
    VoteExpirationOutcome {
        expired: true,
        player_vote_infos,
    }
}

pub(crate) fn vote_display_snapshot(
    vote: &GameVoteState,
    players: &LocalVotePlayers,
    selectable_maps: &[String],
) -> Option<VoteDisplaySnapshot> {
    if !vote.in_progress {
        return None;
    }

    Some(VoteDisplaySnapshot {
        description: source_vote_description(vote.vote_type?, vote.value, selectable_maps),
        have_votes: players
            .players
            .first()
            .map_or(0, |player| player.real_voting_power()),
        needed_votes: votes_needed(players),
        for_votes: votes_for(players),
        against_votes: votes_against(players),
    })
}

pub(crate) fn submit_vote_choice(
    current_paused: bool,
    choice: VoteChoice,
    vote: &mut GameVoteState,
    players: &mut LocalVotePlayers,
    settings: &LocalVoteSettings,
    player_index: usize,
) -> VoteChoiceOutcome {
    let news_message = vote_choice_news(choice, vote, players, settings, player_index);
    let vote_was_in_progress = vote.in_progress;
    let action = match choice {
        VoteChoice::Yes => vote_yes(vote, players, settings, player_index),
        VoteChoice::No => vote_no(vote, players, settings, player_index),
        VoteChoice::Pass => vote_pass(vote, players, settings, player_index),
    };

    if vote_was_in_progress && !vote.in_progress {
        relay_vote_info(vote);
    }

    let processed = processed_vote_outcome(current_paused, action.processed_vote);

    VoteChoiceOutcome {
        pause_update: processed.pause_update,
        game_speed_percent_update: processed.game_speed_percent_update,
        non_pause_action: processed.non_pause_action,
        news_message,
        player_vote_infos: action.player_vote_infos,
    }
}

impl VoteType {
    fn wire_id(self) -> i32 {
        match self {
            Self::PauseGame => 0,
            Self::ResumeGame => 1,
            Self::ChangeMap => 2,
            Self::StartBot => 3,
            Self::StopBot => 4,
            Self::ResetGame => 5,
            Self::ReshuffleTeams => 6,
            Self::ChangeGameSpeed => 7,
        }
    }

    fn from_wire_id(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::PauseGame),
            1 => Some(Self::ResumeGame),
            2 => Some(Self::ChangeMap),
            3 => Some(Self::StartBot),
            4 => Some(Self::StopBot),
            5 => Some(Self::ResetGame),
            6 => Some(Self::ReshuffleTeams),
            7 => Some(Self::ChangeGameSpeed),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::PauseGame => "Pause Game",
            Self::ResumeGame => "Resume Game",
            Self::ChangeMap => "Change Map",
            Self::StartBot => "Start Bot",
            Self::StopBot => "Stop Bot",
            Self::ResetGame => "Reset Game",
            Self::ReshuffleTeams => "Reshuffle Teams",
            Self::ChangeGameSpeed => "Set Game Speed",
        }
    }
}

pub(crate) fn source_vote_append_description(
    vote_type: VoteType,
    value: i32,
    selectable_maps: &[String],
) -> String {
    match vote_type {
        VoteType::ChangeMap => usize::try_from(value)
            .ok()
            .and_then(|index| {
                selectable_maps
                    .get(index)
                    .map(|map| format!("{value}. {map}"))
            })
            .unwrap_or_default(),
        VoteType::StartBot | VoteType::StopBot => source_vote_team_name(value)
            .map(str::to_string)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn source_vote_description(vote_type: VoteType, value: i32, selectable_maps: &[String]) -> String {
    let append_description = source_vote_append_description(vote_type, value, selectable_maps);
    if append_description.is_empty() {
        vote_type.label().to_string()
    } else {
        format!("{}: {append_description}", vote_type.label())
    }
}

fn source_vote_team_name(value: i32) -> Option<&'static str> {
    match value {
        0 => Some("null"),
        1 => Some("red"),
        2 => Some("blue"),
        3 => Some("green"),
        4 => Some("yellow"),
        5 => Some("purple"),
        6 => Some("teal"),
        7 => Some("white"),
        8 => Some("black"),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn pause_vote_update_for_request(
    current_paused: bool,
    requested_paused: bool,
    vote: &mut GameVoteState,
    players: &mut LocalVotePlayers,
    settings: &LocalVoteSettings,
    player_index: usize,
) -> Option<GamePauseUpdate> {
    pause_vote_outcome_for_request(
        current_paused,
        requested_paused,
        vote,
        players,
        settings,
        player_index,
    )
    .pause_update
}

pub(crate) fn pause_vote_outcome_for_request(
    current_paused: bool,
    requested_paused: bool,
    vote: &mut GameVoteState,
    players: &mut LocalVotePlayers,
    settings: &LocalVoteSettings,
    player_index: usize,
) -> VoteRequestOutcome {
    if current_paused == requested_paused {
        return VoteRequestOutcome::default();
    }

    let vote_type = if requested_paused {
        VoteType::PauseGame
    } else {
        VoteType::ResumeGame
    };
    let outcome = start_vote_outcome(
        vote_type,
        PAUSE_VOTE_VALUE,
        vote,
        players,
        settings,
        player_index,
        &[],
    );
    let processed = processed_vote_outcome(current_paused, outcome.processed_vote);

    VoteRequestOutcome {
        pause_update: processed.pause_update,
        game_speed_percent_update: processed.game_speed_percent_update,
        non_pause_action: processed.non_pause_action,
        news_message: outcome.news_message,
        player_vote_infos: outcome.player_vote_infos,
    }
}

pub(crate) fn game_speed_vote_outcome_for_request(
    speed_percent: i32,
    current_paused: bool,
    vote: &mut GameVoteState,
    players: &mut LocalVotePlayers,
    settings: &LocalVoteSettings,
    player_index: usize,
) -> VoteRequestOutcome {
    let outcome = start_vote_outcome(
        VoteType::ChangeGameSpeed,
        speed_percent,
        vote,
        players,
        settings,
        player_index,
        &[],
    );
    let processed = processed_vote_outcome(current_paused, outcome.processed_vote);

    VoteRequestOutcome {
        pause_update: processed.pause_update,
        game_speed_percent_update: processed.game_speed_percent_update,
        non_pause_action: processed.non_pause_action,
        news_message: outcome.news_message,
        player_vote_infos: outcome.player_vote_infos,
    }
}

pub(crate) fn non_pause_vote_outcome_for_request(
    request: NonPauseVoteRequest,
    current_paused: bool,
    selectable_maps: &[String],
    vote: &mut GameVoteState,
    players: &mut LocalVotePlayers,
    settings: &LocalVoteSettings,
    player_index: usize,
) -> VoteRequestOutcome {
    if matches!(
        request,
        NonPauseVoteRequest::StartBot { team } | NonPauseVoteRequest::StopBot { team }
            if !(1..=8).contains(&team)
    ) {
        return VoteRequestOutcome::default();
    }
    let (vote_type, value) = match request {
        NonPauseVoteRequest::ChangeMap { map_num } => (VoteType::ChangeMap, map_num),
        NonPauseVoteRequest::StartBot { team } => (VoteType::StartBot, team),
        NonPauseVoteRequest::StopBot { team } => (VoteType::StopBot, team),
        NonPauseVoteRequest::ResetGame => (VoteType::ResetGame, -1),
        NonPauseVoteRequest::ReshuffleTeams => (VoteType::ReshuffleTeams, -1),
    };
    let outcome = start_vote_outcome(
        vote_type,
        value,
        vote,
        players,
        settings,
        player_index,
        selectable_maps,
    );
    let processed = processed_vote_outcome(current_paused, outcome.processed_vote);

    VoteRequestOutcome {
        pause_update: processed.pause_update,
        game_speed_percent_update: processed.game_speed_percent_update,
        non_pause_action: processed.non_pause_action,
        news_message: outcome.news_message,
        player_vote_infos: outcome.player_vote_infos,
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct StartVoteOutcome {
    processed_vote: Option<ProcessedVote>,
    news_message: Option<String>,
    player_vote_infos: Vec<SetLocalPlayerVoteInfoPacket>,
}

fn start_vote_outcome(
    vote_type: VoteType,
    value: i32,
    vote: &mut GameVoteState,
    players: &mut LocalVotePlayers,
    settings: &LocalVoteSettings,
    player_index: usize,
    selectable_maps: &[String],
) -> StartVoteOutcome {
    let Some(player) = players.players.get(player_index).copied() else {
        return StartVoteOutcome::default();
    };
    if let Some(message) = start_vote_rejection_news(vote_type, value, settings, selectable_maps) {
        return StartVoteOutcome {
            processed_vote: None,
            news_message: Some(message),
            player_vote_infos: Vec::new(),
        };
    }
    if !can_vote(player, settings) {
        return StartVoteOutcome {
            processed_vote: None,
            news_message: Some("you must be logged in to start a vote, please type /help".into()),
            player_vote_infos: Vec::new(),
        };
    }

    if vote.in_progress {
        if vote.vote_type == Some(vote_type) && vote.value == value {
            let news_message =
                vote_choice_news(VoteChoice::Yes, vote, players, settings, player_index)
                    .map(str::to_string);
            let action = vote_yes(vote, players, settings, player_index);
            if !vote.in_progress {
                relay_vote_info(vote);
            }
            return StartVoteOutcome {
                processed_vote: action.processed_vote,
                news_message,
                player_vote_infos: action.player_vote_infos,
            };
        }
        return StartVoteOutcome::default();
    }

    if !vote_required(players) || player.real_voting_power() >= votes_needed(players) {
        return StartVoteOutcome {
            processed_vote: process_vote(vote_type, value),
            news_message: None,
            player_vote_infos: Vec::new(),
        };
    }

    vote.in_progress = true;
    vote.vote_type = Some(vote_type);
    vote.value = value;
    vote.elapsed_seconds = 0.0;
    let mut player_vote_infos = clear_player_votes(players);
    let append_description = source_vote_append_description(vote_type, value, selectable_maps);
    let news_message = Some(vote_started_news_message(
        player.name,
        vote_type,
        &append_description,
    ));
    let mut action = vote_yes(vote, players, settings, player_index);
    player_vote_infos.append(&mut action.player_vote_infos);
    relay_vote_info(vote);
    StartVoteOutcome {
        processed_vote: action.processed_vote,
        news_message,
        player_vote_infos,
    }
}

fn vote_yes(
    vote: &mut GameVoteState,
    players: &mut LocalVotePlayers,
    settings: &LocalVoteSettings,
    player_index: usize,
) -> VoteServerAction {
    if !vote.in_progress {
        return VoteServerAction::default();
    }

    let Some(player) = players.players.get(player_index).copied() else {
        return VoteServerAction::default();
    };
    if player.vote_choice != P_NULL_VOTE || !can_vote(player, settings) {
        return VoteServerAction::default();
    }

    players.players[player_index].vote_choice = P_YES_VOTE;
    let mut player_vote_infos = relay_player_vote_choice(players, player_index)
        .into_iter()
        .collect::<Vec<_>>();
    let mut action = check_vote(vote, players);
    player_vote_infos.append(&mut action.player_vote_infos);
    VoteServerAction {
        processed_vote: action.processed_vote,
        player_vote_infos,
    }
}

fn vote_no(
    vote: &mut GameVoteState,
    players: &mut LocalVotePlayers,
    settings: &LocalVoteSettings,
    player_index: usize,
) -> VoteServerAction {
    if !vote.in_progress {
        return VoteServerAction::default();
    }

    let Some(player) = players.players.get(player_index).copied() else {
        return VoteServerAction::default();
    };
    if player.vote_choice != P_NULL_VOTE || !can_vote(player, settings) {
        return VoteServerAction::default();
    }

    players.players[player_index].vote_choice = P_NO_VOTE;
    let mut player_vote_infos = relay_player_vote_choice(players, player_index)
        .into_iter()
        .collect::<Vec<_>>();
    let mut action = check_vote(vote, players);
    player_vote_infos.append(&mut action.player_vote_infos);
    VoteServerAction {
        processed_vote: action.processed_vote,
        player_vote_infos,
    }
}

fn vote_pass(
    vote: &mut GameVoteState,
    players: &mut LocalVotePlayers,
    settings: &LocalVoteSettings,
    player_index: usize,
) -> VoteServerAction {
    if !vote.in_progress {
        return VoteServerAction::default();
    }

    let Some(player) = players.players.get(player_index).copied() else {
        return VoteServerAction::default();
    };
    if player.vote_choice != P_NULL_VOTE || !can_vote(player, settings) {
        return VoteServerAction::default();
    }

    players.players[player_index].vote_choice = P_PASS_VOTE;
    let mut player_vote_infos = relay_player_vote_choice(players, player_index)
        .into_iter()
        .collect::<Vec<_>>();
    let mut action = check_vote(vote, players);
    player_vote_infos.append(&mut action.player_vote_infos);
    VoteServerAction {
        processed_vote: action.processed_vote,
        player_vote_infos,
    }
}

fn vote_choice_news(
    choice: VoteChoice,
    vote: &GameVoteState,
    players: &LocalVotePlayers,
    settings: &LocalVoteSettings,
    player_index: usize,
) -> Option<&'static str> {
    if !vote.in_progress {
        return None;
    }

    let player = players.players.get(player_index).copied()?;
    if player.vote_choice != P_NULL_VOTE {
        return Some("you have already voted");
    }
    if !can_vote(player, settings) {
        return Some("you must be logged in to vote, please type /help");
    }

    Some(match choice {
        VoteChoice::Yes => "you have voted yes",
        VoteChoice::No => "you have voted no",
        VoteChoice::Pass => "you have passed on voting",
    })
}

fn vote_started_news_message(
    player_name: &str,
    vote_type: VoteType,
    append_description: &str,
) -> String {
    if append_description.is_empty() {
        format!("vote started by {player_name} to {}", vote_type.label())
    } else {
        format!(
            "vote started by {player_name} to {}: {append_description}",
            vote_type.label()
        )
    }
}

fn check_vote(vote: &mut GameVoteState, players: &mut LocalVotePlayers) -> VoteServerAction {
    if !vote.in_progress {
        return VoteServerAction::default();
    }

    let votes_needed = votes_needed(players);
    if votes_for(players) >= votes_needed {
        let processed_vote = vote
            .vote_type
            .and_then(|vote_type| process_vote(vote_type, vote.value));
        VoteServerAction {
            processed_vote,
            player_vote_infos: kill_vote(vote, players),
        }
    } else if votes_against(players) >= votes_needed {
        VoteServerAction {
            processed_vote: None,
            player_vote_infos: kill_vote(vote, players),
        }
    } else {
        VoteServerAction::default()
    }
}

fn start_vote_rejection_news(
    vote_type: VoteType,
    value: i32,
    settings: &LocalVoteSettings,
    selectable_maps: &[String],
) -> Option<String> {
    match vote_type {
        VoteType::ChangeMap
            if value < 0 || usize::try_from(value).map_or(true, |i| i >= selectable_maps.len()) =>
        {
            Some("invalid map choice, please type /listmaps".to_string())
        }
        VoteType::StartBot | VoteType::StopBot if !(1..=8).contains(&value) => None,
        VoteType::ChangeGameSpeed if !settings.allow_game_speed_change => {
            Some("changing the game speed is not allowed on this server".to_string())
        }
        VoteType::ChangeGameSpeed if value <= 0 => {
            Some("new game speed must be above zero".to_string())
        }
        _ => None,
    }
}

fn process_vote(vote_type: VoteType, value: i32) -> Option<ProcessedVote> {
    match vote_type {
        VoteType::PauseGame
        | VoteType::ResumeGame
        | VoteType::ChangeMap
        | VoteType::StartBot
        | VoteType::StopBot
        | VoteType::ResetGame
        | VoteType::ReshuffleTeams => Some(ProcessedVote { vote_type, value }),
        VoteType::ChangeGameSpeed if value > 0 => Some(ProcessedVote { vote_type, value }),
        VoteType::ChangeGameSpeed => None,
    }
}

fn processed_vote_outcome(
    current_paused: bool,
    processed_vote: Option<ProcessedVote>,
) -> ProcessedVoteOutcome {
    let Some(processed_vote) = processed_vote else {
        return ProcessedVoteOutcome::default();
    };

    match processed_vote.vote_type {
        VoteType::PauseGame if !current_paused => ProcessedVoteOutcome {
            pause_update: Some(GamePauseUpdate { game_paused: true }),
            game_speed_percent_update: None,
            non_pause_action: None,
        },
        VoteType::ResumeGame if current_paused => ProcessedVoteOutcome {
            pause_update: Some(GamePauseUpdate { game_paused: false }),
            game_speed_percent_update: None,
            non_pause_action: None,
        },
        VoteType::ChangeGameSpeed => ProcessedVoteOutcome {
            pause_update: None,
            game_speed_percent_update: Some(processed_vote.value),
            non_pause_action: None,
        },
        VoteType::ChangeMap => ProcessedVoteOutcome {
            non_pause_action: usize::try_from(processed_vote.value)
                .ok()
                .map(|map_num| NonPauseVoteAction::ChangeMap { map_num }),
            ..ProcessedVoteOutcome::default()
        },
        VoteType::StartBot => ProcessedVoteOutcome {
            non_pause_action: Some(NonPauseVoteAction::StartBot {
                team: processed_vote.value,
            }),
            ..ProcessedVoteOutcome::default()
        },
        VoteType::StopBot => ProcessedVoteOutcome {
            non_pause_action: Some(NonPauseVoteAction::StopBot {
                team: processed_vote.value,
            }),
            ..ProcessedVoteOutcome::default()
        },
        VoteType::ResetGame => ProcessedVoteOutcome {
            non_pause_action: Some(NonPauseVoteAction::ResetGame),
            ..ProcessedVoteOutcome::default()
        },
        VoteType::ReshuffleTeams => ProcessedVoteOutcome {
            non_pause_action: Some(NonPauseVoteAction::ReshuffleTeams),
            ..ProcessedVoteOutcome::default()
        },
        _ => ProcessedVoteOutcome::default(),
    }
}

fn kill_vote(
    vote: &mut GameVoteState,
    players: &mut LocalVotePlayers,
) -> Vec<SetLocalPlayerVoteInfoPacket> {
    let player_vote_infos = clear_player_votes(players);
    *vote = GameVoteState::default();
    player_vote_infos
}

fn relay_player_vote_choice(
    players: &mut LocalVotePlayers,
    player_index: usize,
) -> Option<SetLocalPlayerVoteInfoPacket> {
    let Some(player) = players.players.get(player_index).copied() else {
        return None;
    };
    let packet = SetLocalPlayerVoteInfoPacket {
        player_id: player.p_id,
        vote_choice: player.vote_choice,
    };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return None;
    };
    if let Some(decoded_packet) = SetLocalPlayerVoteInfoPacket::decode_payload(payload) {
        apply_player_vote_info(players, decoded_packet);
        Some(decoded_packet)
    } else {
        None
    }
}

fn apply_player_vote_info(
    players: &mut LocalVotePlayers,
    packet: SetLocalPlayerVoteInfoPacket,
) -> bool {
    if !(P_NULL_VOTE..P_MAX_VOTE_CHOICES).contains(&packet.vote_choice) {
        return false;
    }

    for player in &mut players.players {
        if player.p_id == packet.player_id {
            player.vote_choice = packet.vote_choice;
        }
    }

    true
}

fn relay_vote_info(vote: &mut GameVoteState) {
    let packet = VoteInfoPacket {
        in_progress: vote.in_progress,
        vote_type: vote.vote_type.map_or(-1, VoteType::wire_id),
        value: vote.value,
    };
    let wire_packet = packet.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return;
    };
    if let Some(decoded_packet) = VoteInfoPacket::decode_payload(payload) {
        vote.apply_vote_info(decoded_packet);
    }
}

fn clear_player_votes(players: &mut LocalVotePlayers) -> Vec<SetLocalPlayerVoteInfoPacket> {
    let mut player_vote_infos = Vec::with_capacity(players.players.len());
    for player_index in 0..players.players.len() {
        players.players[player_index].vote_choice = P_NULL_VOTE;
        if let Some(packet) = relay_player_vote_choice(players, player_index) {
            player_vote_infos.push(packet);
        }
    }
    player_vote_infos
}

fn can_vote(player: LocalVotePlayer, settings: &LocalVoteSettings) -> bool {
    !settings.require_login || player.logged_in
}

fn vote_required(players: &LocalVotePlayers) -> bool {
    players
        .players
        .iter()
        .filter(|player| player.is_player)
        .count()
        >= 2
}

fn votes_needed(players: &LocalVotePlayers) -> i32 {
    let mut needed_power = players
        .players
        .iter()
        .filter(|player| player.vote_choice != P_PASS_VOTE)
        .map(|player| player.real_voting_power())
        .sum::<i32>();
    if needed_power % 2 != 0 {
        needed_power += 1;
    }
    needed_power / 2
}

fn votes_for(players: &LocalVotePlayers) -> i32 {
    players
        .players
        .iter()
        .filter(|player| player.vote_choice == P_YES_VOTE)
        .map(|player| player.real_voting_power())
        .sum()
}

fn votes_against(players: &LocalVotePlayers) -> i32 {
    players
        .players
        .iter()
        .filter(|player| player.vote_choice == P_NO_VOTE)
        .map(|player| player.real_voting_power())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vote_player(voting_power: i32) -> LocalVotePlayer {
        LocalVotePlayer {
            p_id: voting_power,
            name: "Alice",
            is_player: true,
            vote_choice: P_NULL_VOTE,
            voting_power,
            total_games: 0,
            logged_in: false,
        }
    }

    fn vote_info(player_id: i32, vote_choice: i32) -> SetLocalPlayerVoteInfoPacket {
        SetLocalPlayerVoteInfoPacket {
            player_id,
            vote_choice,
        }
    }

    #[test]
    fn single_local_player_pause_vote_processes_without_active_vote() {
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers::default();
        let settings = LocalVoteSettings::default();

        assert_eq!(
            pause_vote_update_for_request(true, false, &mut vote, &mut players, &settings, 0),
            Some(GamePauseUpdate { game_paused: false })
        );
        assert_eq!(vote, GameVoteState::default());
        assert_eq!(players.players[0].vote_choice, P_NULL_VOTE);
    }

    #[test]
    fn pause_vote_keeps_source_same_state_noop_guard() {
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers::default();
        let settings = LocalVoteSettings::default();

        assert_eq!(
            pause_vote_update_for_request(true, true, &mut vote, &mut players, &settings, 0),
            None
        );
        assert_eq!(
            pause_vote_update_for_request(false, false, &mut vote, &mut players, &settings, 0),
            None
        );
        assert_eq!(vote, GameVoteState::default());
    }

    #[test]
    fn multi_player_pause_vote_waits_for_majority_then_processes_same_vote_yes() {
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers {
            players: vec![vote_player(1), vote_player(5)],
        };
        let settings = LocalVoteSettings::default();

        assert_eq!(
            pause_vote_update_for_request(true, false, &mut vote, &mut players, &settings, 0),
            None
        );
        assert_eq!(vote.in_progress, true);
        assert_eq!(vote.vote_type, Some(VoteType::ResumeGame));
        assert_eq!(vote.value, PAUSE_VOTE_VALUE);
        assert_eq!(players.players[0].vote_choice, P_YES_VOTE);

        assert_eq!(
            pause_vote_update_for_request(true, false, &mut vote, &mut players, &settings, 1),
            Some(GamePauseUpdate { game_paused: false })
        );
        assert_eq!(vote, GameVoteState::default());
        assert!(
            players
                .players
                .iter()
                .all(|player| player.vote_choice == P_NULL_VOTE)
        );
    }

    #[test]
    fn vote_no_kills_vote_when_against_votes_reach_needed_power() {
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers {
            players: vec![vote_player(1), vote_player(5)],
        };
        let settings = LocalVoteSettings::default();

        assert_eq!(
            pause_vote_update_for_request(true, false, &mut vote, &mut players, &settings, 0),
            None
        );
        assert_eq!(
            submit_vote_choice(true, VoteChoice::No, &mut vote, &mut players, &settings, 1,),
            VoteChoiceOutcome {
                pause_update: None,
                game_speed_percent_update: None,
                non_pause_action: None,
                news_message: Some("you have voted no"),
                player_vote_infos: vec![
                    vote_info(5, P_NO_VOTE),
                    vote_info(1, P_NULL_VOTE),
                    vote_info(5, P_NULL_VOTE),
                ],
            }
        );
        assert_eq!(vote, GameVoteState::default());
        assert!(
            players
                .players
                .iter()
                .all(|player| player.vote_choice == P_NULL_VOTE)
        );
    }

    #[test]
    fn vote_pass_can_lower_needed_power_and_process_existing_yes_votes() {
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers {
            players: vec![vote_player(2), vote_player(5)],
        };
        let settings = LocalVoteSettings::default();

        assert_eq!(
            pause_vote_update_for_request(true, false, &mut vote, &mut players, &settings, 0),
            None
        );
        assert_eq!(
            submit_vote_choice(
                true,
                VoteChoice::Pass,
                &mut vote,
                &mut players,
                &settings,
                1,
            ),
            VoteChoiceOutcome {
                pause_update: Some(GamePauseUpdate { game_paused: false }),
                game_speed_percent_update: None,
                non_pause_action: None,
                news_message: Some("you have passed on voting"),
                player_vote_infos: vec![
                    vote_info(5, P_PASS_VOTE),
                    vote_info(2, P_NULL_VOTE),
                    vote_info(5, P_NULL_VOTE),
                ],
            }
        );
        assert_eq!(vote, GameVoteState::default());
        assert!(
            players
                .players
                .iter()
                .all(|player| player.vote_choice == P_NULL_VOTE)
        );
    }

    #[test]
    fn vote_power_matches_source_rounding_and_pass_exclusion() {
        let mut players = LocalVotePlayers {
            players: vec![vote_player(1), vote_player(2), vote_player(4)],
        };

        assert_eq!(votes_needed(&players), 4);
        players.players[2].vote_choice = P_PASS_VOTE;
        assert_eq!(votes_needed(&players), 2);
    }

    #[test]
    fn player_vote_info_apply_matches_source_value_and_player_id_rules() {
        let mut players = LocalVotePlayers {
            players: vec![vote_player(1), vote_player(2)],
        };

        assert!(apply_player_vote_info(
            &mut players,
            SetLocalPlayerVoteInfoPacket {
                player_id: 2,
                vote_choice: P_NO_VOTE,
            },
        ));
        assert_eq!(players.players[0].vote_choice, P_NULL_VOTE);
        assert_eq!(players.players[1].vote_choice, P_NO_VOTE);

        assert!(!apply_player_vote_info(
            &mut players,
            SetLocalPlayerVoteInfoPacket {
                player_id: 2,
                vote_choice: P_MAX_VOTE_CHOICES,
            },
        ));
        assert_eq!(players.players[1].vote_choice, P_NO_VOTE);

        assert!(apply_player_vote_info(
            &mut players,
            SetLocalPlayerVoteInfoPacket {
                player_id: 99,
                vote_choice: P_YES_VOTE,
            },
        ));
        assert_eq!(players.players[0].vote_choice, P_NULL_VOTE);
        assert_eq!(players.players[1].vote_choice, P_NO_VOTE);
    }

    #[test]
    fn vote_expiration_kills_active_vote_after_source_timeout() {
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers {
            players: vec![vote_player(1), vote_player(5)],
        };
        let settings = LocalVoteSettings::default();

        assert_eq!(
            pause_vote_update_for_request(true, false, &mut vote, &mut players, &settings, 0),
            None
        );
        assert!(vote.in_progress);

        assert_eq!(
            tick_vote_expiration(&mut vote, &mut players, 29.9),
            VoteExpirationOutcome::default()
        );
        assert!(vote.in_progress);

        assert_eq!(
            tick_vote_expiration(&mut vote, &mut players, 0.2),
            VoteExpirationOutcome {
                expired: true,
                player_vote_infos: vec![vote_info(1, P_NULL_VOTE), vote_info(5, P_NULL_VOTE)],
            }
        );
        assert_eq!(vote, GameVoteState::default());
        assert!(
            players
                .players
                .iter()
                .all(|player| player.vote_choice == P_NULL_VOTE)
        );
    }

    #[test]
    fn vote_display_snapshot_matches_source_setup_images_inputs() {
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers {
            players: vec![vote_player(1), vote_player(5)],
        };
        let settings = LocalVoteSettings::default();

        assert_eq!(
            pause_vote_update_for_request(true, false, &mut vote, &mut players, &settings, 0),
            None
        );

        assert_eq!(
            vote_display_snapshot(&vote, &players, &[]),
            Some(VoteDisplaySnapshot {
                description: "Resume Game".to_string(),
                have_votes: 1,
                needed_votes: 3,
                for_votes: 1,
                against_votes: 0,
            })
        );
    }

    #[test]
    fn vote_wire_ids_and_labels_cover_every_source_vote_type() {
        let expected = [
            (VoteType::PauseGame, 0, "Pause Game"),
            (VoteType::ResumeGame, 1, "Resume Game"),
            (VoteType::ChangeMap, 2, "Change Map"),
            (VoteType::StartBot, 3, "Start Bot"),
            (VoteType::StopBot, 4, "Stop Bot"),
            (VoteType::ResetGame, 5, "Reset Game"),
            (VoteType::ReshuffleTeams, 6, "Reshuffle Teams"),
            (VoteType::ChangeGameSpeed, 7, "Set Game Speed"),
        ];

        for (vote_type, wire_id, label) in expected {
            assert_eq!(vote_type.wire_id(), wire_id);
            assert_eq!(VoteType::from_wire_id(wire_id), Some(vote_type));
            assert_eq!(vote_type.label(), label);
        }
        assert_eq!(VoteType::from_wire_id(-1), None);
        assert_eq!(VoteType::from_wire_id(8), None);
    }

    #[test]
    fn vote_append_description_matches_source_map_and_team_rules() {
        let maps = vec!["alpha.map".to_string(), "beta.map".to_string()];

        assert_eq!(
            source_vote_append_description(VoteType::ChangeMap, 1, &maps),
            "1. beta.map"
        );
        assert_eq!(
            source_vote_append_description(VoteType::StartBot, 2, &maps),
            "blue"
        );
        assert_eq!(
            source_vote_append_description(VoteType::StopBot, 8, &maps),
            "black"
        );
        assert_eq!(
            source_vote_append_description(VoteType::ResetGame, -1, &maps),
            ""
        );
        assert_eq!(
            source_vote_append_description(VoteType::ChangeMap, 9, &maps),
            ""
        );
    }

    #[test]
    fn vote_info_packet_drives_non_pause_hud_description_with_append_text() {
        let maps = vec!["alpha.map".to_string(), "beta.map".to_string()];
        let players = LocalVotePlayers::default();
        let mut vote = GameVoteState::default();

        vote.apply_vote_info(VoteInfoPacket {
            in_progress: true,
            vote_type: VoteType::ChangeMap.wire_id(),
            value: 1,
        });
        assert_eq!(
            vote_display_snapshot(&vote, &players, &maps).map(|snapshot| snapshot.description),
            Some("Change Map: 1. beta.map".to_string())
        );

        vote.apply_vote_info(VoteInfoPacket {
            in_progress: true,
            vote_type: VoteType::StartBot.wire_id(),
            value: 3,
        });
        assert_eq!(
            vote_display_snapshot(&vote, &players, &maps).map(|snapshot| snapshot.description),
            Some("Start Bot: green".to_string())
        );
    }

    #[test]
    fn vote_choice_news_matches_source_vote_yes_duplicate_and_login_strings() {
        let mut vote = GameVoteState {
            in_progress: true,
            vote_type: Some(VoteType::PauseGame),
            value: PAUSE_VOTE_VALUE,
            elapsed_seconds: 0.0,
        };
        let mut players = LocalVotePlayers {
            players: vec![vote_player(1)],
        };
        let settings = LocalVoteSettings::default();

        assert_eq!(
            submit_vote_choice(
                false,
                VoteChoice::Yes,
                &mut vote,
                &mut players,
                &settings,
                0,
            )
            .news_message,
            Some("you have voted yes")
        );

        vote.in_progress = true;
        vote.vote_type = Some(VoteType::PauseGame);
        players.players[0].vote_choice = P_YES_VOTE;
        assert_eq!(
            submit_vote_choice(
                false,
                VoteChoice::Yes,
                &mut vote,
                &mut players,
                &settings,
                0,
            )
            .news_message,
            Some("you have already voted")
        );

        players.players[0].vote_choice = P_NULL_VOTE;
        let settings = LocalVoteSettings {
            require_login: true,
            ..LocalVoteSettings::default()
        };
        assert_eq!(
            submit_vote_choice(
                false,
                VoteChoice::Yes,
                &mut vote,
                &mut players,
                &settings,
                0,
            )
            .news_message,
            Some("you must be logged in to vote, please type /help")
        );
    }

    #[test]
    fn pause_vote_start_news_matches_source_broadcast_string() {
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers {
            players: vec![vote_player(1), vote_player(5)],
        };
        let settings = LocalVoteSettings::default();

        assert_eq!(
            pause_vote_outcome_for_request(true, false, &mut vote, &mut players, &settings, 0),
            VoteRequestOutcome {
                pause_update: None,
                game_speed_percent_update: None,
                non_pause_action: None,
                news_message: Some("vote started by Alice to Resume Game".into()),
                player_vote_infos: vec![
                    vote_info(1, P_NULL_VOTE),
                    vote_info(5, P_NULL_VOTE),
                    vote_info(1, P_YES_VOTE),
                ],
            }
        );
    }

    #[test]
    fn pause_vote_start_login_news_matches_source_string() {
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers {
            players: vec![vote_player(1), vote_player(5)],
        };
        let settings = LocalVoteSettings {
            require_login: true,
            ..LocalVoteSettings::default()
        };

        assert_eq!(
            pause_vote_outcome_for_request(true, false, &mut vote, &mut players, &settings, 0),
            VoteRequestOutcome {
                pause_update: None,
                game_speed_percent_update: None,
                non_pause_action: None,
                news_message: Some(
                    "you must be logged in to start a vote, please type /help".into()
                ),
                player_vote_infos: Vec::new(),
            }
        );
    }

    #[test]
    fn game_speed_vote_processes_immediately_when_vote_not_required() {
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers::default();
        let settings = LocalVoteSettings::default();

        assert_eq!(
            game_speed_vote_outcome_for_request(150, true, &mut vote, &mut players, &settings, 0),
            VoteRequestOutcome {
                pause_update: None,
                game_speed_percent_update: Some(150),
                non_pause_action: None,
                news_message: None,
                player_vote_infos: Vec::new(),
            }
        );
        assert_eq!(vote, GameVoteState::default());
    }

    #[test]
    fn game_speed_vote_rejects_non_positive_speed_like_source_start_vote() {
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers::default();
        let settings = LocalVoteSettings::default();

        assert_eq!(
            game_speed_vote_outcome_for_request(0, true, &mut vote, &mut players, &settings, 0),
            VoteRequestOutcome {
                pause_update: None,
                game_speed_percent_update: None,
                non_pause_action: None,
                news_message: Some("new game speed must be above zero".into()),
                player_vote_infos: Vec::new(),
            }
        );
        assert_eq!(vote, GameVoteState::default());
    }

    #[test]
    fn game_speed_vote_uses_source_vote_type_and_processes_on_majority() {
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers {
            players: vec![vote_player(1), vote_player(5)],
        };
        let settings = LocalVoteSettings::default();

        assert_eq!(
            game_speed_vote_outcome_for_request(200, false, &mut vote, &mut players, &settings, 0),
            VoteRequestOutcome {
                pause_update: None,
                game_speed_percent_update: None,
                non_pause_action: None,
                news_message: Some("vote started by Alice to Set Game Speed".into()),
                player_vote_infos: vec![
                    vote_info(1, P_NULL_VOTE),
                    vote_info(5, P_NULL_VOTE),
                    vote_info(1, P_YES_VOTE),
                ],
            }
        );
        assert_eq!(vote.vote_type, Some(VoteType::ChangeGameSpeed));
        assert_eq!(vote.value, 200);

        assert_eq!(
            submit_vote_choice(
                false,
                VoteChoice::Yes,
                &mut vote,
                &mut players,
                &settings,
                1
            ),
            VoteChoiceOutcome {
                pause_update: None,
                game_speed_percent_update: Some(200),
                non_pause_action: None,
                news_message: Some("you have voted yes"),
                player_vote_infos: vec![
                    vote_info(5, P_YES_VOTE),
                    vote_info(1, P_NULL_VOTE),
                    vote_info(5, P_NULL_VOTE),
                ],
            }
        );
        assert_eq!(vote, GameVoteState::default());
    }

    #[test]
    fn non_pause_vote_start_uses_source_map_append_and_returns_passed_action() {
        let maps = vec!["alpha.map".to_string(), "beta.map".to_string()];
        let settings = LocalVoteSettings::default();
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers {
            players: vec![vote_player(1), vote_player(5)],
        };

        assert_eq!(
            non_pause_vote_outcome_for_request(
                NonPauseVoteRequest::ChangeMap { map_num: 1 },
                false,
                &maps,
                &mut vote,
                &mut players,
                &settings,
                0,
            ),
            VoteRequestOutcome {
                pause_update: None,
                game_speed_percent_update: None,
                non_pause_action: None,
                news_message: Some("vote started by Alice to Change Map: 1. beta.map".to_string()),
                player_vote_infos: vec![
                    vote_info(1, P_NULL_VOTE),
                    vote_info(5, P_NULL_VOTE),
                    vote_info(1, P_YES_VOTE),
                ],
            }
        );

        assert_eq!(
            submit_vote_choice(
                false,
                VoteChoice::Yes,
                &mut vote,
                &mut players,
                &settings,
                1,
            )
            .non_pause_action,
            Some(NonPauseVoteAction::ChangeMap { map_num: 1 })
        );
    }

    #[test]
    fn non_pause_vote_rejects_invalid_map_and_team_like_source_start_vote() {
        let maps = vec!["alpha.map".to_string()];
        let settings = LocalVoteSettings::default();
        let mut vote = GameVoteState::default();
        let mut players = LocalVotePlayers::default();

        assert_eq!(
            non_pause_vote_outcome_for_request(
                NonPauseVoteRequest::ChangeMap { map_num: 3 },
                false,
                &maps,
                &mut vote,
                &mut players,
                &settings,
                0,
            )
            .news_message,
            Some("invalid map choice, please type /listmaps".to_string())
        );
        assert_eq!(
            non_pause_vote_outcome_for_request(
                NonPauseVoteRequest::StartBot { team: 0 },
                false,
                &maps,
                &mut vote,
                &mut players,
                &settings,
                0,
            ),
            VoteRequestOutcome::default()
        );
    }

    #[test]
    fn local_bot_team_state_tracks_source_team_range() {
        let mut bots = LocalBotTeams::default();

        assert!(bots.set_active(2, true));
        assert!(bots.set_active(8, true));
        assert!(!bots.set_active(0, true));
        assert!(!bots.set_active(9, true));
        assert_eq!(bots.active_teams(), vec![TeamType::Blue, TeamType::Black]);
        assert!(bots.set_active(2, false));
        assert_eq!(bots.active_teams(), vec![TeamType::Black]);
    }
}
