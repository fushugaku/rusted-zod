use bevy::prelude::Resource;

use crate::{
    account::AccountCommand,
    components::GamePauseRequest,
    game_speed::GameSpeedVoteRequest,
    local_player::LocalPlayerState,
    network_commands::{CommandPayload, SendChatCommand},
    news::NewsLog,
    original::types::TeamType,
    version::relay_version_packet,
    vote::NonPauseVoteRequest,
};

#[derive(Clone, Debug, Resource)]
pub(crate) struct ChatInputState {
    collecting: bool,
    message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChatCommandContext {
    player_name: String,
    player_team: TeamType,
    current_map_name: String,
    selectable_maps: Vec<String>,
    logged_in: bool,
    activated: bool,
    voting_power: i32,
    real_voting_power: i32,
    active_bot_teams: Vec<TeamType>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ChatSubmitOutcome {
    pub(crate) pause_request: Option<GamePauseRequest>,
    pub(crate) game_speed_request: Option<GameSpeedVoteRequest>,
    pub(crate) non_pause_vote_request: Option<NonPauseVoteRequest>,
    pub(crate) team_change_request: Option<TeamType>,
    pub(crate) account_command: Option<AccountCommand>,
    pub(crate) registration_request: bool,
}

impl Default for ChatInputState {
    fn default() -> Self {
        Self {
            collecting: false,
            message: String::new(),
        }
    }
}

impl Default for ChatCommandContext {
    fn default() -> Self {
        Self {
            player_name: "Player".to_string(),
            player_team: TeamType::Red,
            current_map_name: String::new(),
            selectable_maps: Vec::new(),
            logged_in: false,
            activated: false,
            voting_power: 0,
            real_voting_power: 0,
            active_bot_teams: Vec::new(),
        }
    }
}

impl ChatInputState {
    pub(crate) fn collecting(&self) -> bool {
        self.collecting
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn toggle_collecting(&mut self) -> Option<String> {
        if self.collecting {
            self.collecting = false;
            let submitted = std::mem::take(&mut self.message);
            Some(submitted)
        } else {
            self.collecting = true;
            self.message.clear();
            None
        }
    }

    pub(crate) fn start_command(&mut self) {
        self.collecting = true;
        self.message.clear();
        self.message.push('/');
    }

    pub(crate) fn push_text(&mut self, text: &str) {
        if !self.collecting {
            return;
        }
        for character in text.chars().filter(|character| !character.is_control()) {
            self.message.push(character);
        }
    }

    pub(crate) fn backspace(&mut self) {
        if self.collecting {
            self.message.pop();
        }
    }
}

impl ChatCommandContext {
    #[cfg(test)]
    pub(crate) fn from_local_player(
        local_player: &LocalPlayerState,
        current_map_name: impl Into<String>,
    ) -> Self {
        Self::from_runtime(local_player, current_map_name, &[])
    }

    pub(crate) fn from_runtime(
        local_player: &LocalPlayerState,
        current_map_name: impl Into<String>,
        selectable_maps: &[String],
    ) -> Self {
        Self {
            player_name: local_player.name().to_string(),
            player_team: local_player.team(),
            current_map_name: current_map_name.into(),
            selectable_maps: selectable_maps.to_vec(),
            logged_in: local_player.logged_in(),
            activated: local_player.activated(),
            voting_power: local_player.voting_power(),
            real_voting_power: local_player.real_voting_power(),
            active_bot_teams: Vec::new(),
        }
    }

    pub(crate) fn with_active_bot_teams(
        mut self,
        teams: impl IntoIterator<Item = TeamType>,
    ) -> Self {
        self.active_bot_teams = teams.into_iter().collect();
        self
    }
}

pub(crate) fn submit_local_chat_message(
    context: &ChatCommandContext,
    news_log: &mut NewsLog,
    message: impl Into<String>,
) -> ChatSubmitOutcome {
    let Some(command) = SendChatCommand::new(message) else {
        return ChatSubmitOutcome::default();
    };
    let wire_packet = command.encode_packet();
    let Some(payload) = wire_packet.get(8..) else {
        return ChatSubmitOutcome::default();
    };
    let Some(decoded_command) = SendChatCommand::decode_payload(payload) else {
        return ChatSubmitOutcome::default();
    };
    relay_chat_command(context, news_log, decoded_command)
}

fn relay_chat_command(
    context: &ChatCommandContext,
    news_log: &mut NewsLog,
    command: SendChatCommand,
) -> ChatSubmitOutcome {
    if let Some(command_name) = command.message.strip_prefix('/') {
        return process_player_command(command_name, context, news_log);
    }

    let (r, g, b) = chat_team_color(context.player_team);
    news_log.relay_source_news(
        format!("{}:: {}", context.player_name, command.message),
        r,
        g,
        b,
    );
    ChatSubmitOutcome::default()
}

fn process_player_command(
    command: &str,
    context: &ChatCommandContext,
    news_log: &mut NewsLog,
) -> ChatSubmitOutcome {
    let (command_name, contents) = player_command_name_and_contents(command);
    match command_name {
        "help" => {
            send_help_command_news(contents, news_log);
            ChatSubmitOutcome::default()
        }
        "listcommands" => {
            send_listcommands_news(news_log);
            ChatSubmitOutcome::default()
        }
        "playerinfo" => {
            send_playerinfo_news(context, news_log);
            ChatSubmitOutcome::default()
        }
        "currentmap" => {
            send_command_news(
                news_log,
                format!("current map: {}", context.current_map_name),
            );
            ChatSubmitOutcome::default()
        }
        "listmaps" => {
            send_listmaps_news(&context.selectable_maps, news_log);
            ChatSubmitOutcome::default()
        }
        "version" => {
            relay_version_packet(news_log);
            ChatSubmitOutcome::default()
        }
        "login" => player_command_login(contents, news_log),
        "logout" => ChatSubmitOutcome {
            account_command: Some(AccountCommand::Logout),
            ..ChatSubmitOutcome::default()
        },
        "createuser" => player_command_create_user(contents, news_log),
        "buyregistration" => ChatSubmitOutcome {
            registration_request: true,
            ..ChatSubmitOutcome::default()
        },
        "pause" => ChatSubmitOutcome {
            pause_request: Some(GamePauseRequest { game_paused: true }),
            game_speed_request: None,
            non_pause_vote_request: None,
            team_change_request: None,
            account_command: None,
            registration_request: false,
        },
        "resume" => ChatSubmitOutcome {
            pause_request: Some(GamePauseRequest { game_paused: false }),
            game_speed_request: None,
            non_pause_vote_request: None,
            team_change_request: None,
            account_command: None,
            registration_request: false,
        },
        "changemap" => player_command_change_map(contents, news_log),
        "startbot" => player_command_start_bot(contents, news_log),
        "stopbot" => player_command_stop_bot(contents, context, news_log),
        "resetgame" => ChatSubmitOutcome {
            non_pause_vote_request: Some(NonPauseVoteRequest::ResetGame),
            ..ChatSubmitOutcome::default()
        },
        "reshuffleteams" => ChatSubmitOutcome {
            non_pause_vote_request: Some(NonPauseVoteRequest::ReshuffleTeams),
            ..ChatSubmitOutcome::default()
        },
        "changeteam" => player_command_change_team(contents, context, news_log),
        "changespeed" => player_command_change_speed(contents, news_log),
        _ => {
            news_log.relay_source_news(
                "command not found, please type /help or /listcommands",
                0,
                0,
                0,
            );
            ChatSubmitOutcome::default()
        }
    }
}

fn player_command_change_speed(contents: &str, news_log: &mut NewsLog) -> ChatSubmitOutcome {
    let Some(value) = first_command_value(contents) else {
        send_command_news(news_log, "command error: invalid input(s)");
        return ChatSubmitOutcome::default();
    };

    ChatSubmitOutcome {
        pause_request: None,
        game_speed_request: Some(GameSpeedVoteRequest {
            speed_percent: source_atoi(value),
        }),
        non_pause_vote_request: None,
        team_change_request: None,
        account_command: None,
        registration_request: false,
    }
}

fn player_command_login(contents: &str, news_log: &mut NewsLog) -> ChatSubmitOutcome {
    let Some(values) = source_command_values(contents, 2) else {
        send_command_news(news_log, "command error: invalid input(s)");
        return ChatSubmitOutcome::default();
    };

    ChatSubmitOutcome {
        account_command: Some(AccountCommand::Login {
            login_name: values[0].clone(),
            password: values[1].clone(),
        }),
        ..ChatSubmitOutcome::default()
    }
}

fn player_command_create_user(contents: &str, news_log: &mut NewsLog) -> ChatSubmitOutcome {
    let Some(values) = source_command_values(contents, 4) else {
        send_command_news(news_log, "command error: invalid input(s)");
        return ChatSubmitOutcome::default();
    };

    ChatSubmitOutcome {
        account_command: Some(AccountCommand::CreateUser {
            user_name: values[0].clone(),
            login_name: values[1].clone(),
            password: values[2].clone(),
            email: values[3].clone(),
        }),
        ..ChatSubmitOutcome::default()
    }
}

fn source_command_values(contents: &str, count: usize) -> Option<Vec<String>> {
    let mut values = contents
        .split(',')
        .take(count)
        .map(|value| value.trim_start_matches(' ').to_string())
        .collect::<Vec<_>>();
    values.resize(count, String::new());
    values
        .iter()
        .all(|value| !value.is_empty())
        .then_some(values)
}

fn player_command_change_team(
    contents: &str,
    context: &ChatCommandContext,
    news_log: &mut NewsLog,
) -> ChatSubmitOutcome {
    let Some(value) = first_command_value(contents) else {
        send_command_news(news_log, "command error: invalid input(s)");
        return ChatSubmitOutcome::default();
    };
    let Some(team) = source_player_team_by_name(value) else {
        send_command_news(
            news_log,
            "change team error: invalid team, example command usage: /changeteam red ... or /changeteam blue",
        );
        return ChatSubmitOutcome::default();
    };
    if team == context.player_team {
        send_command_news(news_log, "change team error: you are already on that team");
        return ChatSubmitOutcome::default();
    }

    ChatSubmitOutcome {
        team_change_request: Some(team),
        ..ChatSubmitOutcome::default()
    }
}

fn player_command_change_map(contents: &str, news_log: &mut NewsLog) -> ChatSubmitOutcome {
    let Some(value) = first_command_value(contents) else {
        send_command_news(news_log, "command error: invalid input(s)");
        return ChatSubmitOutcome::default();
    };

    ChatSubmitOutcome {
        non_pause_vote_request: Some(NonPauseVoteRequest::ChangeMap {
            map_num: source_atoi(value),
        }),
        ..ChatSubmitOutcome::default()
    }
}

fn player_command_start_bot(contents: &str, news_log: &mut NewsLog) -> ChatSubmitOutcome {
    let Some(value) = first_command_value(contents) else {
        send_command_news(news_log, "command error: invalid input(s)");
        return ChatSubmitOutcome::default();
    };
    let Some(team) = source_team_by_name(value) else {
        send_command_news(
            news_log,
            format!(
                "start bot error: invalid team, available teams: {}",
                source_non_null_team_names().join(", ")
            ),
        );
        return ChatSubmitOutcome::default();
    };

    ChatSubmitOutcome {
        non_pause_vote_request: Some(NonPauseVoteRequest::StartBot { team: team as i32 }),
        ..ChatSubmitOutcome::default()
    }
}

fn player_command_stop_bot(
    contents: &str,
    context: &ChatCommandContext,
    news_log: &mut NewsLog,
) -> ChatSubmitOutcome {
    let Some(value) = first_command_value(contents) else {
        send_command_news(news_log, "command error: invalid input(s)");
        return ChatSubmitOutcome::default();
    };
    let team = source_team_by_name(value);
    if team.is_none_or(|team| !context.active_bot_teams.contains(&team)) {
        let available = context
            .active_bot_teams
            .iter()
            .map(|team| team.asset_name())
            .collect::<Vec<_>>()
            .join(", ");
        send_command_news(
            news_log,
            format!("start bot error: invalid team, available teams: {available}"),
        );
        return ChatSubmitOutcome::default();
    }
    let team = team.expect("validated active source team");

    ChatSubmitOutcome {
        non_pause_vote_request: Some(NonPauseVoteRequest::StopBot { team: team as i32 }),
        ..ChatSubmitOutcome::default()
    }
}

fn source_non_null_team_names() -> [&'static str; 8] {
    [
        "red", "blue", "green", "yellow", "purple", "teal", "white", "black",
    ]
}

fn source_team_by_name(value: &str) -> Option<TeamType> {
    match value {
        "red" => Some(TeamType::Red),
        "blue" => Some(TeamType::Blue),
        "green" => Some(TeamType::Green),
        "yellow" => Some(TeamType::Yellow),
        "purple" => Some(TeamType::Purple),
        "teal" => Some(TeamType::Teal),
        "white" => Some(TeamType::White),
        "black" => Some(TeamType::Black),
        _ => None,
    }
}

fn source_player_team_by_name(value: &str) -> Option<TeamType> {
    if value == "null" {
        Some(TeamType::Null)
    } else {
        source_team_by_name(value)
    }
}

fn player_command_name_and_contents(full_command: &str) -> (&str, &str) {
    full_command
        .split_once(' ')
        .map_or((full_command, ""), |(command, contents)| {
            (command, contents)
        })
}

fn first_command_value(contents: &str) -> Option<&str> {
    let value = contents
        .split_once(',')
        .map_or(contents, |(value, _)| value)
        .trim_start_matches(' ');
    (!value.is_empty()).then_some(value)
}

fn source_atoi(value: &str) -> i32 {
    let value = value.trim_start_matches(|character| character == ' ' || character == '\t');
    let mut chars = value.chars().peekable();
    let sign = match chars.peek().copied() {
        Some('-') => {
            chars.next();
            -1
        }
        Some('+') => {
            chars.next();
            1
        }
        _ => 1,
    };
    let mut parsed = 0_i32;
    let mut saw_digit = false;
    while let Some(character) = chars.peek().copied() {
        let Some(digit) = character.to_digit(10) else {
            break;
        };
        saw_digit = true;
        chars.next();
        parsed = parsed.saturating_mul(10).saturating_add(digit as i32);
    }

    if saw_digit { parsed * sign } else { 0 }
}

fn send_help_command_news(contents: &str, news_log: &mut NewsLog) {
    match contents {
        "" | "help" => {
            send_command_news(news_log, "help usage: /help command");
            send_command_news(
                news_log,
                "help purpose: explain how to use a command and what it is used for",
            );
            send_listcommands_news(news_log);
        }
        "listcommands" => {
            send_command_news(news_log, "listcommands usage: /listcommands");
            send_command_news(
                news_log,
                "listcommands purpose: list all of the available commands",
            );
        }
        "login" => {
            send_command_news(news_log, "login usage: /login username, password");
            send_command_news(news_log, "login purpose: log into your username");
        }
        "logout" => {
            send_command_news(news_log, "logout usage: /logout");
            send_command_news(news_log, "logout purpose: log out of your username");
        }
        "createuser" => {
            send_command_news(
                news_log,
                "createuser usage: /createuser username, loginname, password, email",
            );
            send_command_news(news_log, "createuser purpose: create a new user");
        }
        "pause" => {
            send_command_news(news_log, "pause usage: /pause");
            send_command_news(news_log, "pause purpose: pauses the game");
        }
        "resume" => {
            send_command_news(news_log, "resume usage: /resume");
            send_command_news(news_log, "resume purpose: resumes the game");
        }
        "listmaps" => {
            send_command_news(news_log, "listmaps usage: /listmaps");
            send_command_news(
                news_log,
                "listmaps purpose: lists available maps to be used with /changemap",
            );
        }
        "changemap" => {
            send_command_news(news_log, "changemap usage: /changemap map_number");
            send_command_news(
                news_log,
                "changemap purpose: reset game to desired map, use /listmaps to get the map_number",
            );
        }
        "startbot" => {
            send_command_news(news_log, "startbot usage: /startbot team_color");
            send_command_news(news_log, "startbot purpose: start a bot");
        }
        "stopbot" => {
            send_command_news(news_log, "stopbot usage: /stopbot team_color");
            send_command_news(news_log, "stopbot purpose: stop a bot");
        }
        "playerinfo" => {
            send_command_news(news_log, "playerinfo usage: /playerinfo");
            send_command_news(
                news_log,
                "playerinfo purpose: gives details on your logged in user",
            );
        }
        "currentmap" => {
            send_command_news(news_log, "currentmap usage: /currentmap");
            send_command_news(
                news_log,
                "currentmap purpose: gives the name of the current map",
            );
        }
        "resetgame" => {
            send_command_news(news_log, "resetgame usage: /resetgame");
            send_command_news(news_log, "resetgame purpose: resets the current game");
        }
        "changeteam" => {
            send_command_news(news_log, "changeteam usage: /changeteam team_color");
            send_command_news(news_log, "changeteam purpose: change your team");
        }
        "reshuffleteams" => {
            send_command_news(news_log, "reshuffleteams usage: /reshuffleteams");
            send_command_news(
                news_log,
                "reshuffleteams purpose: randomly places players on new teams and preserves balance",
            );
        }
        "buyregistration" => {
            send_command_news(news_log, "buyregistration usage: /buyregistration");
            send_command_news(
                news_log,
                "buyregistration purpose: downloads an offline registration key from the server for a cost in voting power",
            );
        }
        "changespeed" => {
            send_command_news(
                news_log,
                "changespeed usage: /changespeed multiplier_number",
            );
            send_command_news(
                news_log,
                "changespeed purpose: changes the game speed. half speed is 50, double speed is 200",
            );
        }
        "version" => {
            send_command_news(news_log, "version usage: /version");
            send_command_news(
                news_log,
                "version purpose: returns the version of the server",
            );
        }
        _ => {}
    }
}

fn send_listcommands_news(news_log: &mut NewsLog) {
    send_command_news(
        news_log,
        "command list: help, listcommands, login, logout, createuser, pause, resume, listmaps, changemap, startbot, stopbot",
    );
    send_command_news(
        news_log,
        "command list: playerinfo, currentmap, resetgame, changeteam, reshuffleteams, buyregistration, changespeed, version",
    );
}

fn send_listmaps_news(maps: &[String], news_log: &mut NewsLog) {
    let mut i = 0;
    while i < maps.len() {
        let mut send_str = String::new();
        for _ in 0..4 {
            if i >= maps.len() {
                break;
            }
            if !send_str.is_empty() {
                send_str.push_str(", ");
            }
            send_str.push_str(&format!("{}. {}", i, maps[i]));
            i += 1;
        }
        send_str.insert_str(0, "map list: ");
        send_command_news(news_log, send_str);
    }
}

fn send_command_news(news_log: &mut NewsLog, message: impl Into<String>) {
    news_log.relay_source_news(message, 0, 0, 0);
}

fn send_playerinfo_news(context: &ChatCommandContext, news_log: &mut NewsLog) {
    send_command_news(
        news_log,
        format!("player info: name: '{}'", context.player_name),
    );
    send_command_news(
        news_log,
        format!("player info: team: {}", context.player_team.asset_name()),
    );

    if context.logged_in {
        send_command_news(news_log, "player info: logged in: yes");
        send_command_news(news_log, "player info: activated: yes");
        send_command_news(
            news_log,
            format!("player info: voting power: {}", context.voting_power),
        );
        send_command_news(
            news_log,
            format!(
                "player info: real voting power: {}",
                context.real_voting_power
            ),
        );
    } else {
        send_command_news(news_log, "player info: logged in: no");
    }
}

fn chat_team_color(team: TeamType) -> (u8, u8, u8) {
    let (r, g, b) = match team {
        TeamType::Null => (115, 115, 115),
        TeamType::Red => (223, 0, 0),
        TeamType::Blue => (19, 55, 251),
        TeamType::Green => (23, 143, 19),
        TeamType::Yellow => (203, 99, 47),
        TeamType::Purple | TeamType::Teal | TeamType::White | TeamType::Black => (115, 115, 115),
    };
    (
        (r as f32 * 0.3) as u8,
        (g as f32 * 0.3) as u8,
        (b as f32 * 0.3) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_input_toggle_and_editing_match_source_shape() {
        let mut state = ChatInputState::default();

        assert_eq!(state.toggle_collecting(), None);
        assert!(state.collecting());
        assert_eq!(state.message(), "");

        state.push_text("hello");
        state.backspace();
        assert_eq!(state.message(), "hell");

        assert_eq!(state.toggle_collecting(), Some("hell".to_string()));
        assert!(!state.collecting());
        assert_eq!(state.message(), "");
    }

    #[test]
    fn slash_key_starts_command_draft_like_source() {
        let mut state = ChatInputState::default();

        state.start_command();

        assert!(state.collecting());
        assert_eq!(state.message(), "/");
    }

    #[test]
    fn chat_message_relay_broadcasts_source_player_prefix() {
        let local_player = LocalPlayerState::default();
        let context = ChatCommandContext::from_local_player(&local_player, "ignored");
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "hello"),
            ChatSubmitOutcome::default()
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("Player:: hello")
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.color),
            Some(crate::news::NewsColor::from_source_rgb(66, 0, 0))
        );
    }

    #[test]
    fn slash_pause_resume_commands_emit_pause_requests() {
        let context = ChatCommandContext::default();
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/pause").pause_request,
            Some(GamePauseRequest { game_paused: true })
        );
        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/resume").pause_request,
            Some(GamePauseRequest { game_paused: false })
        );
        assert_eq!(news_log.display_entry(0), None);
    }

    #[test]
    fn changespeed_command_emits_source_speed_vote_request() {
        let context = ChatCommandContext::default();
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/changespeed 200"),
            ChatSubmitOutcome {
                pause_request: None,
                game_speed_request: Some(GameSpeedVoteRequest { speed_percent: 200 }),
                non_pause_vote_request: None,
                team_change_request: None,
                account_command: None,
                registration_request: false,
            }
        );
        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/changespeed 50, 100"),
            ChatSubmitOutcome {
                pause_request: None,
                game_speed_request: Some(GameSpeedVoteRequest { speed_percent: 50 }),
                non_pause_vote_request: None,
                team_change_request: None,
                account_command: None,
                registration_request: false,
            }
        );
        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/changespeed nope"),
            ChatSubmitOutcome {
                pause_request: None,
                game_speed_request: Some(GameSpeedVoteRequest { speed_percent: 0 }),
                non_pause_vote_request: None,
                team_change_request: None,
                account_command: None,
                registration_request: false,
            }
        );
        assert_eq!(news_log.display_entry(0), None);
    }

    #[test]
    fn changespeed_missing_input_matches_source_command_error() {
        let context = ChatCommandContext::default();
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/changespeed"),
            ChatSubmitOutcome::default()
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("command error: invalid input(s)")
        );
    }

    #[test]
    fn listcommands_matches_source_news_lines() {
        let context = ChatCommandContext::default();
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/listcommands"),
            ChatSubmitOutcome::default()
        );

        assert_eq!(
            news_log.display_entry(1).map(|entry| entry.message),
            Some(
                "command list: help, listcommands, login, logout, createuser, pause, resume, listmaps, changemap, startbot, stopbot"
            )
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some(
                "command list: playerinfo, currentmap, resetgame, changeteam, reshuffleteams, buyregistration, changespeed, version"
            )
        );
    }

    #[test]
    fn help_command_matches_source_news_lines() {
        let context = ChatCommandContext::default();
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/help"),
            ChatSubmitOutcome::default()
        );

        assert_eq!(
            news_log.display_entry(3).map(|entry| entry.message),
            Some("help usage: /help command")
        );
        assert_eq!(
            news_log.display_entry(2).map(|entry| entry.message),
            Some("help purpose: explain how to use a command and what it is used for")
        );
        assert_eq!(
            news_log.display_entry(1).map(|entry| entry.message),
            Some(
                "command list: help, listcommands, login, logout, createuser, pause, resume, listmaps, changemap, startbot, stopbot"
            )
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some(
                "command list: playerinfo, currentmap, resetgame, changeteam, reshuffleteams, buyregistration, changespeed, version"
            )
        );
    }

    #[test]
    fn help_specific_command_matches_source_news_lines() {
        let context = ChatCommandContext::default();
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/help pause"),
            ChatSubmitOutcome::default()
        );
        assert_eq!(
            news_log.display_entry(1).map(|entry| entry.message),
            Some("pause usage: /pause")
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("pause purpose: pauses the game")
        );
    }

    #[test]
    fn help_unknown_contents_matches_source_noop() {
        let context = ChatCommandContext::default();
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/help wat"),
            ChatSubmitOutcome::default()
        );
        assert_eq!(news_log.display_entry(0), None);
    }

    #[test]
    fn playerinfo_not_logged_in_matches_source_news_lines() {
        let mut context = ChatCommandContext::default();
        context.player_name = "Alice".to_string();
        context.player_team = TeamType::Blue;
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/playerinfo"),
            ChatSubmitOutcome::default()
        );

        assert_eq!(
            news_log.display_entry(2).map(|entry| entry.message),
            Some("player info: name: 'Alice'")
        );
        assert_eq!(
            news_log.display_entry(1).map(|entry| entry.message),
            Some("player info: team: blue")
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("player info: logged in: no")
        );
    }

    #[test]
    fn playerinfo_logged_in_matches_source_news_lines() {
        let context = ChatCommandContext {
            player_name: "Alice".to_string(),
            player_team: TeamType::Green,
            logged_in: true,
            activated: false,
            voting_power: 7,
            real_voting_power: 3,
            ..Default::default()
        };
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/playerinfo"),
            ChatSubmitOutcome::default()
        );

        assert_eq!(
            news_log.display_entry(5).map(|entry| entry.message),
            Some("player info: name: 'Alice'")
        );
        assert_eq!(
            news_log.display_entry(4).map(|entry| entry.message),
            Some("player info: team: green")
        );
        assert_eq!(
            news_log.display_entry(3).map(|entry| entry.message),
            Some("player info: logged in: yes")
        );
        assert_eq!(
            news_log.display_entry(2).map(|entry| entry.message),
            Some("player info: activated: yes")
        );
        assert_eq!(
            news_log.display_entry(1).map(|entry| entry.message),
            Some("player info: voting power: 7")
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("player info: real voting power: 3")
        );
    }

    #[test]
    fn currentmap_matches_source_news_line() {
        let context = ChatCommandContext {
            current_map_name: "zod_map".to_string(),
            ..Default::default()
        };
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/currentmap"),
            ChatSubmitOutcome::default()
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("current map: zod_map")
        );
    }

    #[test]
    fn listmaps_matches_source_grouped_news_lines() {
        let context = ChatCommandContext {
            selectable_maps: vec![
                "a.map".to_string(),
                "b.map".to_string(),
                "c.map".to_string(),
                "d.map".to_string(),
                "e.map".to_string(),
            ],
            ..Default::default()
        };
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/listmaps"),
            ChatSubmitOutcome::default()
        );

        assert_eq!(
            news_log.display_entry(1).map(|entry| entry.message),
            Some("map list: 0. a.map, 1. b.map, 2. c.map, 3. d.map")
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("map list: 4. e.map")
        );
    }

    #[test]
    fn empty_listmaps_matches_source_noop() {
        let context = ChatCommandContext::default();
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/listmaps"),
            ChatSubmitOutcome::default()
        );
        assert_eq!(news_log.display_entry(0), None);
    }

    #[test]
    fn version_command_relays_source_version_packet_to_news() {
        let context = ChatCommandContext::default();
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/version"),
            ChatSubmitOutcome::default()
        );
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
    fn non_pause_vote_commands_emit_source_requests() {
        let context =
            ChatCommandContext::default().with_active_bot_teams([TeamType::Blue, TeamType::Yellow]);
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/changemap 12")
                .non_pause_vote_request,
            Some(NonPauseVoteRequest::ChangeMap { map_num: 12 })
        );
        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/startbot green")
                .non_pause_vote_request,
            Some(NonPauseVoteRequest::StartBot { team: 3 })
        );
        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/stopbot blue")
                .non_pause_vote_request,
            Some(NonPauseVoteRequest::StopBot { team: 2 })
        );
        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/resetgame").non_pause_vote_request,
            Some(NonPauseVoteRequest::ResetGame)
        );
        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/reshuffleteams")
                .non_pause_vote_request,
            Some(NonPauseVoteRequest::ReshuffleTeams)
        );
    }

    #[test]
    fn bot_command_errors_match_source_strings_and_active_bot_filter() {
        let context = ChatCommandContext::default().with_active_bot_teams([TeamType::Blue]);
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/startbot orange"),
            ChatSubmitOutcome::default()
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some(
                "start bot error: invalid team, available teams: red, blue, green, yellow, purple, teal, white, black"
            )
        );

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/stopbot red"),
            ChatSubmitOutcome::default()
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("start bot error: invalid team, available teams: blue")
        );
    }

    #[test]
    fn changeteam_command_matches_source_validation_and_request() {
        let context = ChatCommandContext::default();
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/changeteam blue")
                .team_change_request,
            Some(TeamType::Blue)
        );
        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/changeteam null")
                .team_change_request,
            Some(TeamType::Null)
        );

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/changeteam red"),
            ChatSubmitOutcome::default()
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("change team error: you are already on that team")
        );

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/changeteam orange"),
            ChatSubmitOutcome::default()
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some(
                "change team error: invalid team, example command usage: /changeteam red ... or /changeteam blue"
            )
        );
    }

    #[test]
    fn account_commands_preserve_source_csv_parsing() {
        let context = ChatCommandContext::default();
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/login alice, secret")
                .account_command,
            Some(AccountCommand::Login {
                login_name: "alice".to_string(),
                password: "secret".to_string(),
            })
        );
        assert_eq!(
            submit_local_chat_message(
                &context,
                &mut news_log,
                "/createuser Alice, alice, secret, alice@example.test"
            )
            .account_command,
            Some(AccountCommand::CreateUser {
                user_name: "Alice".to_string(),
                login_name: "alice".to_string(),
                password: "secret".to_string(),
                email: "alice@example.test".to_string(),
            })
        );
        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/logout").account_command,
            Some(AccountCommand::Logout)
        );

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/login alice"),
            ChatSubmitOutcome::default()
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("command error: invalid input(s)")
        );
    }

    #[test]
    fn buyregistration_command_emits_source_poll_request() {
        let context = ChatCommandContext::default();
        let mut news_log = NewsLog::default();

        let outcome = submit_local_chat_message(&context, &mut news_log, "/buyregistration");

        assert!(outcome.registration_request);
        assert_eq!(outcome.account_command, None);
        assert_eq!(news_log.display_entry(0), None);
    }

    #[test]
    fn unknown_slash_command_matches_source_not_found_news() {
        let context = ChatCommandContext::default();
        let mut news_log = NewsLog::default();

        assert_eq!(
            submit_local_chat_message(&context, &mut news_log, "/wat"),
            ChatSubmitOutcome::default()
        );
        assert_eq!(
            news_log.display_entry(0).map(|entry| entry.message),
            Some("command not found, please type /help or /listcommands")
        );
    }
}
