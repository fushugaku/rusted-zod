#[cfg(test)]
use crate::original::{
    map::MapObjectType,
    objects::{ItemType, ObjectKind},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum TcpEventId {
    RequestMap = 1,
    StoreMap = 3,
    RequestObjects = 4,
    RequestZones = 5,
    AddNewObject = 6,
    SetZoneInfo = 7,
    SetName = 8,
    SetTeam = 9,
    NewsEvent = 10,
    SendWaypoints = 11,
    SendRallypoints = 12,
    SendLoc = 13,
    SetObjectTeam = 14,
    SetAttackObject = 15,
    DeleteObject = 16,
    UpdateHealth = 17,
    EndGame = 18,
    ResetGame = 19,
    DestroyObject = 21,
    #[cfg(test)]
    StartBuilding = 22,
    #[cfg(test)]
    StopBuilding = 23,
    SetBuildingState = 24,
    SetBuiltCannonAmount = 25,
    #[cfg(test)]
    PlaceCannon = 26,
    SendChat = 27,
    ComputerMessage = 28,
    ObjectGroupInfo = 29,
    EjectVehicle = 30,
    DoCraneAnim = 31,
    SetRepairAnim = 32,
    RequestSettings = 33,
    SetSettings = 34,
    SetLidOpen = 35,
    SnipeObject = 36,
    DriverHitEffect = 37,
    SetPlayerMode = 38,
    RequestPlayerList = 39,
    ClearPlayerList = 40,
    AddLocalPlayer = 41,
    DeleteLocalPlayer = 42,
    SetLocalPlayerName = 43,
    SetLocalPlayerTeam = 44,
    SetLocalPlayerMode = 45,
    SetLocalPlayerIgnored = 46,
    SetLocalPlayerLogInfo = 47,
    SetLocalPlayerVoteInfo = 48,
    UpdateGamePaused = 50,
    GetGamePaused = 51,
    SetGamePaused = 52,
    #[cfg(test)]
    StartVote = 53,
    VoteYes = 54,
    VoteNo = 55,
    VotePass = 56,
    VoteInfo = 57,
    GivePlayerId = 58,
    RequestPlayerId = 59,
    RequestSelectableMapList = 60,
    GiveSelectableMapList = 61,
    SendLogin = 62,
    RequestLoginOff = 63,
    GiveLoginOff = 64,
    CreateUser = 65,
    SetGrenadeAmount = 66,
    PickupGrenadeAnimation = 67,
    DoPortraitAnim = 68,
    TeamEnded = 69,
    PollBuyRegistrationKey = 70,
    BuyRegistrationKey = 71,
    ReturnRegistrationKey = 72,
    #[cfg(test)]
    AddBuildingQueue = 76,
    SetBuildingQueueList = 77,
    #[cfg(test)]
    CancelBuildingQueue = 78,
    GetGameSpeed = 73,
    SetGameSpeed = 74,
    UpdateGameSpeed = 75,
    ReshuffleTeams = 79,
    StartBot = 80,
    StopBot = 81,
    SelectMap = 82,
    ResetMap = 83,
    RequestVersion = 84,
    GiveVersion = 85,
}

impl TcpEventId {
    pub(crate) const fn wire_id(self) -> i32 {
        self as i32
    }
}

pub(crate) trait CommandPayload {
    const EVENT_ID: TcpEventId;

    fn encode_payload(&self) -> Vec<u8>;

    fn encode_packet(&self) -> Vec<u8> {
        encode_packet(Self::EVENT_ID, &self.encode_payload())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ResetGamePacket;

impl CommandPayload for ResetGamePacket {
    const EVENT_ID: TcpEventId = TcpEventId::ResetGame;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EndGamePacket;

impl CommandPayload for EndGamePacket {
    const EVENT_ID: TcpEventId = TcpEventId::EndGame;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl EndGamePacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        payload.is_empty().then_some(Self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TeamEndedPacket {
    pub(crate) team: i32,
    pub(crate) won: bool,
}

impl CommandPayload for TeamEndedPacket {
    const EVENT_ID: TcpEventId = TcpEventId::TeamEnded;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&self.team.to_le_bytes());
        payload.push(u8::from(self.won));
        payload.extend_from_slice(&[0; 3]);
        payload
    }
}

impl TeamEndedPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 8 {
            return None;
        }
        Some(Self {
            team: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            won: match payload[4] {
                0 => false,
                1 => true,
                _ => return None,
            },
        })
    }
}

impl ResetGamePacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        payload.is_empty().then_some(Self)
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StartBuildingCommand {
    pub ref_id: i32,
    pub object_type: u8,
    pub object_id: u8,
}

#[cfg(test)]
impl StartBuildingCommand {
    pub fn new(ref_id: i32, kind: ObjectKind) -> Option<Self> {
        let (object_type, object_id) = object_kind_wire_parts(kind)?;
        Some(Self {
            ref_id,
            object_type,
            object_id,
        })
    }
}

#[cfg(test)]
impl CommandPayload for StartBuildingCommand {
    const EVENT_ID: TcpEventId = TcpEventId::StartBuilding;

    fn encode_payload(&self) -> Vec<u8> {
        encode_start_like_payload(self.ref_id, self.object_type, self.object_id)
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StopBuildingCommand {
    pub ref_id: i32,
}

#[cfg(test)]
impl CommandPayload for StopBuildingCommand {
    const EVENT_ID: TcpEventId = TcpEventId::StopBuilding;

    fn encode_payload(&self) -> Vec<u8> {
        self.ref_id.to_le_bytes().to_vec()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SetGamePausedCommand {
    pub(crate) game_paused: bool,
}

impl CommandPayload for SetGamePausedCommand {
    const EVENT_ID: TcpEventId = TcpEventId::SetGamePaused;

    fn encode_payload(&self) -> Vec<u8> {
        encode_game_paused_payload(self.game_paused)
    }
}

impl SetGamePausedCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        decode_game_paused_payload(payload).map(|game_paused| Self { game_paused })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReshuffleTeamsCommand;

impl CommandPayload for ReshuffleTeamsCommand {
    const EVENT_ID: TcpEventId = TcpEventId::ReshuffleTeams;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl ReshuffleTeamsCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        payload.is_empty().then_some(Self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StartBotCommand {
    pub(crate) team: i32,
}

impl CommandPayload for StartBotCommand {
    const EVENT_ID: TcpEventId = TcpEventId::StartBot;

    fn encode_payload(&self) -> Vec<u8> {
        self.team.to_le_bytes().to_vec()
    }
}

impl StartBotCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        decode_int_command(payload).map(|team| Self { team })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StopBotCommand {
    pub(crate) team: i32,
}

impl CommandPayload for StopBotCommand {
    const EVENT_ID: TcpEventId = TcpEventId::StopBot;

    fn encode_payload(&self) -> Vec<u8> {
        self.team.to_le_bytes().to_vec()
    }
}

impl StopBotCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        decode_int_command(payload).map(|team| Self { team })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SelectMapCommand {
    pub(crate) map_num: i32,
}

impl CommandPayload for SelectMapCommand {
    const EVENT_ID: TcpEventId = TcpEventId::SelectMap;

    fn encode_payload(&self) -> Vec<u8> {
        self.map_num.to_le_bytes().to_vec()
    }
}

impl SelectMapCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        decode_int_command(payload).map(|map_num| Self { map_num })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ResetMapCommand;

impl CommandPayload for ResetMapCommand {
    const EVENT_ID: TcpEventId = TcpEventId::ResetMap;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl ResetMapCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        payload.is_empty().then_some(Self)
    }
}

fn decode_int_command(payload: &[u8]) -> Option<i32> {
    if payload.len() != 4 {
        return None;
    }
    Some(i32::from_le_bytes(payload.try_into().ok()?))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UpdateGamePausedPacket {
    pub(crate) game_paused: bool,
}

impl UpdateGamePausedPacket {
    pub(crate) fn encode_payload(&self) -> Vec<u8> {
        encode_game_paused_payload(self.game_paused)
    }

    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        decode_game_paused_payload(payload).map(|game_paused| Self { game_paused })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VoteInfoPacket {
    pub(crate) in_progress: bool,
    pub(crate) vote_type: i32,
    pub(crate) value: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RequestMapCommand;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StoreMapPacket {
    pub(crate) packet_number: i32,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RequestSettingsCommand;

pub(crate) const SOURCE_ZSETTINGS_PACKET_SIZE: usize = 1420;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SetSettingsPacket {
    pub(crate) bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RequestZonesCommand;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SetZoneInfoPacket {
    pub(crate) zone_number: i32,
    pub(crate) owner: i8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RequestObjectsCommand;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ObjectInitPacket {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) ref_id: i32,
    pub(crate) owner: i8,
    pub(crate) object_type: u8,
    pub(crate) object_id: u8,
    pub(crate) building_level: i8,
    pub(crate) extra_links: u16,
    pub(crate) health: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BuiltCannonListPacket {
    pub(crate) ref_id: i32,
    pub(crate) cannon_ids: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ObjectTeamDriverInfo {
    pub(crate) driver_health: i32,
    pub(crate) next_attack_time: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ObjectTeamPacket {
    pub(crate) ref_id: i32,
    pub(crate) owner: i8,
    pub(crate) driver_type: i8,
    pub(crate) drivers: Vec<ObjectTeamDriverInfo>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AttackObjectPacket {
    pub(crate) ref_id: i32,
    pub(crate) attack_object_ref_id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ObjectLocationPacket {
    pub(crate) ref_id: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) dx: f32,
    pub(crate) dy: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i8)]
pub(crate) enum SourceWaypointMode {
    Move = 0,
    Enter = 1,
    Attack = 2,
    ForceMove = 3,
    CraneRepair = 4,
    UnitRepair = 5,
    Agro = 6,
    EnterFort = 7,
    Dodge = 8,
    PickupGrenades = 9,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SourceWaypoint {
    pub(crate) mode: SourceWaypointMode,
    pub(crate) ref_id: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) attack_to: bool,
    pub(crate) player_given: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SendWaypointsPacket {
    pub(crate) ref_id: i32,
    pub(crate) waypoints: Vec<SourceWaypoint>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SendRallypointsPacket {
    pub(crate) ref_id: i32,
    pub(crate) waypoints: Vec<SourceWaypoint>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObjectGroupInfoPacket {
    pub(crate) ref_id: i32,
    pub(crate) leader_ref_id: i32,
    pub(crate) minion_refs: Vec<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DeleteObjectPacket {
    pub(crate) ref_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ObjectHealthPacket {
    pub(crate) ref_id: i32,
    pub(crate) health: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BuildingStatePacket {
    pub(crate) ref_id: i32,
    pub(crate) state: i32,
    pub(crate) init_offset: f64,
    pub(crate) production_time: f64,
    pub(crate) object_type: u8,
    pub(crate) object_id: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BuildingQueueUnit {
    pub(crate) object_type: u8,
    pub(crate) object_id: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BuildingQueuePacket {
    pub(crate) ref_id: i32,
    pub(crate) units: Vec<BuildingQueueUnit>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DestroyObjectMissileInfo {
    pub(crate) missile_offset_time: f64,
    pub(crate) missile_x: i32,
    pub(crate) missile_y: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DestroyObjectPacket {
    pub(crate) ref_id: i32,
    pub(crate) killer_ref_id: i32,
    pub(crate) destroy_object: bool,
    pub(crate) do_fire_death: bool,
    pub(crate) do_missile_death: bool,
    pub(crate) fire_missiles: Vec<DestroyObjectMissileInfo>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ObjectGrenadeAmountPacket {
    pub(crate) ref_id: i32,
    pub(crate) grenade_amount: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PickupGrenadeAnimationPacket {
    pub(crate) ref_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SetLidOpenPacket {
    pub(crate) ref_id: i32,
    pub(crate) lid_open: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EjectVehiclePacket {
    pub(crate) ref_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CraneAnimPacket {
    pub(crate) ref_id: i32,
    pub(crate) repair_ref_id: i32,
    pub(crate) on: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RepairBuildingAnimPacket {
    pub(crate) ref_id: i32,
    pub(crate) on: bool,
    pub(crate) remaining_time: f64,
    pub(crate) play_sound: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SnipeObjectPacket {
    pub(crate) ref_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DriverHitEffectPacket {
    pub(crate) ref_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ComputerMessagePacket {
    pub(crate) ref_id: i32,
    pub(crate) sound: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DoPortraitAnimPacket {
    pub(crate) ref_id: i32,
    pub(crate) anim_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SetLocalPlayerVoteInfoPacket {
    pub(crate) player_id: i32,
    pub(crate) vote_choice: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RequestPlayerIdCommand;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RequestPlayerListCommand;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RequestSelectableMapListCommand;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GiveSelectableMapListPacket {
    pub(crate) maps: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SendLoginCommand {
    pub(crate) login_name: String,
    pub(crate) password: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RequestLoginOffCommand;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GiveLoginOffPacket {
    pub(crate) show_login: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CreateUserCommand {
    pub(crate) user_name: String,
    pub(crate) login_name: String,
    pub(crate) password: String,
    pub(crate) email: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PollBuyRegistrationKeyPacket;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BuyRegistrationKeyCommand {
    pub(crate) device_id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReturnRegistrationKeyPacket {
    pub(crate) encrypted_key: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ClearPlayerListPacket;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerIdPacket {
    pub(crate) player_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AddLocalPlayerPacket {
    pub(crate) player_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DeleteLocalPlayerPacket {
    pub(crate) player_id: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SetNameCommand {
    pub(crate) name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SetTeamCommand {
    pub(crate) team: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SetPlayerModeCommand {
    pub(crate) mode: i8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SetLocalPlayerNamePacket {
    pub(crate) player_id: i32,
    pub(crate) name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SetLocalPlayerTeamPacket {
    pub(crate) player_id: i32,
    pub(crate) team: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SetLocalPlayerModePacket {
    pub(crate) player_id: i32,
    pub(crate) mode: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SetLocalPlayerIgnoredPacket {
    pub(crate) player_id: i32,
    pub(crate) ignored: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SetLocalPlayerLogInfoPacket {
    pub(crate) player_id: i32,
    pub(crate) db_id: i32,
    pub(crate) voting_power: i32,
    pub(crate) total_games: i32,
    pub(crate) activated: bool,
    pub(crate) logged_in: bool,
    pub(crate) bot_logged_in: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NewsEventPacket {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
    pub(crate) message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SendChatCommand {
    pub(crate) message: String,
}

pub(crate) const SOURCE_GAME_VERSION: &str = "2011-09-06";
const MAX_PLAYER_NAME_SIZE: usize = 30;
const MAX_VERSION_PACKET_CHARS: usize = 50;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GiveVersionPacket {
    pub(crate) version: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UpdateGameSpeedPacket {
    pub(crate) game_speed: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SetGameSpeedCommand {
    pub(crate) game_speed: f32,
}

impl SendChatCommand {
    pub(crate) fn new(message: impl Into<String>) -> Option<Self> {
        let message = message.into();
        if message.is_empty() || message.as_bytes().contains(&0) {
            return None;
        }
        Some(Self { message })
    }

    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() <= 1 || payload.last().copied()? != 0 {
            return None;
        }
        let message = &payload[..payload.len() - 1];
        if message.is_empty() || message.contains(&0) {
            return None;
        }
        Some(Self {
            message: std::str::from_utf8(message).ok()?.to_owned(),
        })
    }
}

impl SendLoginCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        let values = decode_nul_csv(payload, 2)?;
        Some(Self {
            login_name: values[0].clone(),
            password: values[1].clone(),
        })
    }
}

impl CommandPayload for SendLoginCommand {
    const EVENT_ID: TcpEventId = TcpEventId::SendLogin;

    fn encode_payload(&self) -> Vec<u8> {
        encode_nul_csv([self.login_name.as_str(), self.password.as_str()])
    }
}

impl RequestLoginOffCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        payload.is_empty().then_some(Self)
    }
}

impl CommandPayload for RequestLoginOffCommand {
    const EVENT_ID: TcpEventId = TcpEventId::RequestLoginOff;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl GiveLoginOffPacket {
    pub(crate) fn encode_packet(&self) -> Vec<u8> {
        encode_packet(TcpEventId::GiveLoginOff, &self.encode_payload())
    }

    pub(crate) fn encode_payload(&self) -> Vec<u8> {
        vec![u8::from(self.show_login)]
    }

    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        decode_game_paused_payload(payload).map(|show_login| Self { show_login })
    }
}

impl CreateUserCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        let values = decode_nul_csv(payload, 4)?;
        Some(Self {
            user_name: values[0].clone(),
            login_name: values[1].clone(),
            password: values[2].clone(),
            email: values[3].clone(),
        })
    }
}

impl PollBuyRegistrationKeyPacket {
    pub(crate) fn encode_packet(&self) -> Vec<u8> {
        encode_packet(TcpEventId::PollBuyRegistrationKey, &[])
    }

    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        payload.is_empty().then_some(Self)
    }
}

impl CommandPayload for BuyRegistrationKeyCommand {
    const EVENT_ID: TcpEventId = TcpEventId::BuyRegistrationKey;

    fn encode_payload(&self) -> Vec<u8> {
        self.device_id.to_vec()
    }
}

impl BuyRegistrationKeyCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        Some(Self {
            device_id: payload.try_into().ok()?,
        })
    }
}

impl ReturnRegistrationKeyPacket {
    pub(crate) fn encode_packet(&self) -> Vec<u8> {
        encode_packet(TcpEventId::ReturnRegistrationKey, &self.encrypted_key)
    }

    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        Some(Self {
            encrypted_key: payload.try_into().ok()?,
        })
    }
}

impl CommandPayload for CreateUserCommand {
    const EVENT_ID: TcpEventId = TcpEventId::CreateUser;

    fn encode_payload(&self) -> Vec<u8> {
        encode_nul_csv([
            self.user_name.as_str(),
            self.login_name.as_str(),
            self.password.as_str(),
            self.email.as_str(),
        ])
    }
}

fn encode_nul_csv<'a>(values: impl IntoIterator<Item = &'a str>) -> Vec<u8> {
    let mut payload = values
        .into_iter()
        .collect::<Vec<_>>()
        .join(",")
        .into_bytes();
    payload.push(0);
    payload
}

fn decode_nul_csv(payload: &[u8], count: usize) -> Option<Vec<String>> {
    if payload.len() <= 1 || payload.last().copied()? != 0 {
        return None;
    }
    let text = std::str::from_utf8(&payload[..payload.len() - 1]).ok()?;
    let values = text.split(',').map(str::to_string).collect::<Vec<_>>();
    (values.len() == count && values.iter().all(|value| !value.is_empty())).then_some(values)
}

impl SetNameCommand {
    pub(crate) fn new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        if name.as_bytes().contains(&0) {
            return None;
        }
        Some(Self { name })
    }

    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.is_empty() {
            return None;
        }
        let nul_index = payload.iter().position(|byte| *byte == 0)?;
        let name_len = nul_index.min(MAX_PLAYER_NAME_SIZE);
        Some(Self {
            name: std::str::from_utf8(&payload[..name_len]).ok()?.to_owned(),
        })
    }
}

impl CommandPayload for SetNameCommand {
    const EVENT_ID: TcpEventId = TcpEventId::SetName;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(self.name.len() + 1);
        payload.extend_from_slice(self.name.as_bytes());
        payload.push(0);
        payload
    }
}

impl SetTeamCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 4 {
            return None;
        }
        Some(Self {
            team: i32::from_le_bytes(payload.try_into().ok()?),
        })
    }
}

impl CommandPayload for SetTeamCommand {
    const EVENT_ID: TcpEventId = TcpEventId::SetTeam;

    fn encode_payload(&self) -> Vec<u8> {
        self.team.to_le_bytes().to_vec()
    }
}

impl SetPlayerModeCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 1 {
            return None;
        }
        Some(Self {
            mode: i8::from_le_bytes([payload[0]]),
        })
    }
}

impl CommandPayload for SetPlayerModeCommand {
    const EVENT_ID: TcpEventId = TcpEventId::SetPlayerMode;

    fn encode_payload(&self) -> Vec<u8> {
        self.mode.to_le_bytes().to_vec()
    }
}

impl GiveVersionPacket {
    pub(crate) fn new(version: impl Into<String>) -> Option<Self> {
        let version = version.into();
        if version.len() + 1 >= MAX_VERSION_PACKET_CHARS || version.as_bytes().contains(&0) {
            return None;
        }
        Some(Self { version })
    }

    pub(crate) fn source_current() -> Self {
        Self::new(SOURCE_GAME_VERSION).expect("source GAME_VERSION fits version_packet")
    }

    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != MAX_VERSION_PACKET_CHARS {
            return None;
        }
        let version_end = payload
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(MAX_VERSION_PACKET_CHARS - 1);
        Some(Self {
            version: std::str::from_utf8(&payload[..version_end])
                .ok()?
                .to_owned(),
        })
    }
}

impl CommandPayload for GiveVersionPacket {
    const EVENT_ID: TcpEventId = TcpEventId::GiveVersion;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = vec![0; MAX_VERSION_PACKET_CHARS];
        payload[..self.version.len()].copy_from_slice(self.version.as_bytes());
        payload
    }
}

impl UpdateGameSpeedPacket {
    pub(crate) fn encode_payload(&self) -> Vec<u8> {
        self.game_speed.to_le_bytes().to_vec()
    }

    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 4 {
            return None;
        }
        Some(Self {
            game_speed: f32::from_le_bytes(payload.try_into().ok()?),
        })
    }
}

impl CommandPayload for UpdateGameSpeedPacket {
    const EVENT_ID: TcpEventId = TcpEventId::UpdateGameSpeed;

    fn encode_payload(&self) -> Vec<u8> {
        Self::encode_payload(self)
    }
}

impl CommandPayload for SetGameSpeedCommand {
    const EVENT_ID: TcpEventId = TcpEventId::SetGameSpeed;

    fn encode_payload(&self) -> Vec<u8> {
        self.game_speed.to_le_bytes().to_vec()
    }
}

impl SetGameSpeedCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 4 {
            return None;
        }
        Some(Self {
            game_speed: f32::from_le_bytes(payload.try_into().ok()?),
        })
    }
}

impl CommandPayload for SendChatCommand {
    const EVENT_ID: TcpEventId = TcpEventId::SendChat;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(self.message.len() + 1);
        payload.extend_from_slice(self.message.as_bytes());
        payload.push(0);
        payload
    }
}

impl NewsEventPacket {
    pub(crate) fn new(message: impl Into<String>, r: u8, g: u8, b: u8) -> Option<Self> {
        let message = message.into();
        if message.is_empty() || message.as_bytes().contains(&0) {
            return None;
        }
        Some(Self { r, g, b, message })
    }

    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() <= 5 {
            return None;
        }
        let (message_with_nul, colors) = payload[3..].split_last()?;
        if *message_with_nul != 0 || colors.is_empty() || colors.contains(&0) {
            return None;
        }
        Some(Self {
            r: payload[0],
            g: payload[1],
            b: payload[2],
            message: std::str::from_utf8(colors).ok()?.to_owned(),
        })
    }
}

impl CommandPayload for NewsEventPacket {
    const EVENT_ID: TcpEventId = TcpEventId::NewsEvent;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(self.message.len() + 4);
        payload.push(self.r);
        payload.push(self.g);
        payload.push(self.b);
        payload.extend_from_slice(self.message.as_bytes());
        payload.push(0);
        payload
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VoteYesCommand;

impl CommandPayload for VoteYesCommand {
    const EVENT_ID: TcpEventId = TcpEventId::VoteYes;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VoteNoCommand;

impl CommandPayload for VoteNoCommand {
    const EVENT_ID: TcpEventId = TcpEventId::VoteNo;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VotePassCommand;

impl CommandPayload for VotePassCommand {
    const EVENT_ID: TcpEventId = TcpEventId::VotePass;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl CommandPayload for RequestPlayerIdCommand {
    const EVENT_ID: TcpEventId = TcpEventId::RequestPlayerId;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl CommandPayload for RequestPlayerListCommand {
    const EVENT_ID: TcpEventId = TcpEventId::RequestPlayerList;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl CommandPayload for RequestMapCommand {
    const EVENT_ID: TcpEventId = TcpEventId::RequestMap;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl RequestMapCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        payload.is_empty().then_some(Self)
    }
}

impl CommandPayload for StoreMapPacket {
    const EVENT_ID: TcpEventId = TcpEventId::StoreMap;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(4 + self.bytes.len());
        payload.extend_from_slice(&self.packet_number.to_le_bytes());
        payload.extend_from_slice(&self.bytes);
        payload
    }
}

impl StoreMapPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() < 4 {
            return None;
        }
        Some(Self {
            packet_number: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            bytes: payload[4..].to_vec(),
        })
    }
}

impl CommandPayload for RequestSettingsCommand {
    const EVENT_ID: TcpEventId = TcpEventId::RequestSettings;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl RequestSettingsCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        payload.is_empty().then_some(Self)
    }
}

impl CommandPayload for SetSettingsPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetSettings;

    fn encode_payload(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

impl SetSettingsPacket {
    pub(crate) fn new(bytes: Vec<u8>) -> Option<Self> {
        (bytes.len() == SOURCE_ZSETTINGS_PACKET_SIZE).then_some(Self { bytes })
    }

    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != SOURCE_ZSETTINGS_PACKET_SIZE {
            return None;
        }
        Some(Self {
            bytes: payload.to_vec(),
        })
    }
}

impl CommandPayload for RequestZonesCommand {
    const EVENT_ID: TcpEventId = TcpEventId::RequestZones;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl RequestZonesCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        payload.is_empty().then_some(Self)
    }
}

impl CommandPayload for SetZoneInfoPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetZoneInfo;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(5);
        payload.extend_from_slice(&self.zone_number.to_le_bytes());
        payload.extend_from_slice(&self.owner.to_le_bytes());
        payload
    }
}

impl SetZoneInfoPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 5 {
            return None;
        }
        Some(Self {
            zone_number: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            owner: i8::from_le_bytes([payload[4]]),
        })
    }
}

impl CommandPayload for RequestObjectsCommand {
    const EVENT_ID: TcpEventId = TcpEventId::RequestObjects;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl RequestObjectsCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        payload.is_empty().then_some(Self)
    }
}

impl CommandPayload for ObjectInitPacket {
    const EVENT_ID: TcpEventId = TcpEventId::AddNewObject;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(22);
        payload.extend_from_slice(&self.x.to_le_bytes());
        payload.extend_from_slice(&self.y.to_le_bytes());
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&self.owner.to_le_bytes());
        payload.push(self.object_type);
        payload.push(self.object_id);
        payload.extend_from_slice(&self.building_level.to_le_bytes());
        payload.extend_from_slice(&self.extra_links.to_le_bytes());
        payload.extend_from_slice(&self.health.to_le_bytes());
        payload
    }
}

impl ObjectInitPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 22 {
            return None;
        }
        Some(Self {
            x: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            y: i32::from_le_bytes(payload[4..8].try_into().ok()?),
            ref_id: i32::from_le_bytes(payload[8..12].try_into().ok()?),
            owner: i8::from_le_bytes([payload[12]]),
            object_type: payload[13],
            object_id: payload[14],
            building_level: i8::from_le_bytes([payload[15]]),
            extra_links: u16::from_le_bytes(payload[16..18].try_into().ok()?),
            health: i32::from_le_bytes(payload[18..22].try_into().ok()?),
        })
    }
}

impl CommandPayload for BuiltCannonListPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetBuiltCannonAmount;

    fn encode_payload(&self) -> Vec<u8> {
        let cannon_amount =
            i32::try_from(self.cannon_ids.len()).expect("source built cannon list length fits i32");
        let mut payload = Vec::with_capacity(8 + self.cannon_ids.len());
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&cannon_amount.to_le_bytes());
        payload.extend_from_slice(&self.cannon_ids);
        payload
    }
}

impl BuiltCannonListPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() < 8 {
            return None;
        }
        let ref_id = i32::from_le_bytes(payload[0..4].try_into().ok()?);
        let cannon_amount = i32::from_le_bytes(payload[4..8].try_into().ok()?);
        let cannon_amount = usize::try_from(cannon_amount).ok()?;
        if payload.len().checked_sub(8)? != cannon_amount {
            return None;
        }
        Some(Self {
            ref_id,
            cannon_ids: payload[8..].to_vec(),
        })
    }
}

impl CommandPayload for ObjectTeamPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetObjectTeam;

    fn encode_payload(&self) -> Vec<u8> {
        let driver_amount =
            i8::try_from(self.drivers.len()).expect("source driver_amount fits signed char");
        let mut payload = Vec::with_capacity(7 + self.drivers.len() * 12);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&self.owner.to_le_bytes());
        payload.extend_from_slice(&self.driver_type.to_le_bytes());
        payload.extend_from_slice(&driver_amount.to_le_bytes());
        for driver in &self.drivers {
            payload.extend_from_slice(&driver.driver_health.to_le_bytes());
            payload.extend_from_slice(&driver.next_attack_time.to_le_bytes());
        }
        payload
    }
}

impl ObjectTeamPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() < 7 {
            return None;
        }
        let driver_amount = i8::from_le_bytes([payload[6]]);
        if driver_amount < 0 {
            return None;
        }
        let driver_amount = usize::try_from(driver_amount).ok()?;
        if payload.len() != 7 + driver_amount * 12 {
            return None;
        }
        let mut drivers = Vec::with_capacity(driver_amount);
        for i in 0..driver_amount {
            let start = 7 + i * 12;
            drivers.push(ObjectTeamDriverInfo {
                driver_health: i32::from_le_bytes(payload[start..start + 4].try_into().ok()?),
                next_attack_time: f64::from_le_bytes(
                    payload[start + 4..start + 12].try_into().ok()?,
                ),
            });
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            owner: i8::from_le_bytes([payload[4]]),
            driver_type: i8::from_le_bytes([payload[5]]),
            drivers,
        })
    }
}

impl CommandPayload for AttackObjectPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetAttackObject;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&self.attack_object_ref_id.to_le_bytes());
        payload
    }
}

impl AttackObjectPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 8 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            attack_object_ref_id: i32::from_le_bytes(payload[4..8].try_into().ok()?),
        })
    }
}

impl CommandPayload for ObjectLocationPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SendLoc;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(20);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&self.x.to_le_bytes());
        payload.extend_from_slice(&self.y.to_le_bytes());
        payload.extend_from_slice(&self.dx.to_le_bytes());
        payload.extend_from_slice(&self.dy.to_le_bytes());
        payload
    }
}

impl ObjectLocationPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 20 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            x: i32::from_le_bytes(payload[4..8].try_into().ok()?),
            y: i32::from_le_bytes(payload[8..12].try_into().ok()?),
            dx: f32::from_le_bytes(payload[12..16].try_into().ok()?),
            dy: f32::from_le_bytes(payload[16..20].try_into().ok()?),
        })
    }
}

pub(crate) const SOURCE_WAYPOINT_PACKET_SIZE: usize = 15;

impl SourceWaypointMode {
    fn wire_id(self) -> i8 {
        self as i8
    }

    fn from_wire(wire_id: i8) -> Option<Self> {
        Some(match wire_id {
            0 => Self::Move,
            1 => Self::Enter,
            2 => Self::Attack,
            3 => Self::ForceMove,
            4 => Self::CraneRepair,
            5 => Self::UnitRepair,
            6 => Self::Agro,
            7 => Self::EnterFort,
            8 => Self::Dodge,
            9 => Self::PickupGrenades,
            _ => return None,
        })
    }
}

impl SourceWaypoint {
    fn encode_wire(self) -> [u8; SOURCE_WAYPOINT_PACKET_SIZE] {
        let mut wire = [0; SOURCE_WAYPOINT_PACKET_SIZE];
        wire[0] = self.mode.wire_id() as u8;
        wire[1..5].copy_from_slice(&self.ref_id.to_le_bytes());
        wire[5..9].copy_from_slice(&self.x.to_le_bytes());
        wire[9..13].copy_from_slice(&self.y.to_le_bytes());
        wire[13] = u8::from(self.attack_to);
        wire[14] = u8::from(self.player_given);
        wire
    }

    fn decode_wire(wire: &[u8]) -> Option<Self> {
        if wire.len() != SOURCE_WAYPOINT_PACKET_SIZE {
            return None;
        }
        let attack_to = decode_bool_byte(wire[13])?;
        let player_given = decode_bool_byte(wire[14])?;
        Some(Self {
            mode: SourceWaypointMode::from_wire(i8::from_le_bytes([wire[0]]))?,
            ref_id: i32::from_le_bytes(wire[1..5].try_into().ok()?),
            x: i32::from_le_bytes(wire[5..9].try_into().ok()?),
            y: i32::from_le_bytes(wire[9..13].try_into().ok()?),
            attack_to,
            player_given,
        })
    }
}

impl CommandPayload for SendWaypointsPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SendWaypoints;

    fn encode_payload(&self) -> Vec<u8> {
        encode_source_waypoint_list_payload(self.ref_id, &self.waypoints)
    }
}

impl SendWaypointsPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        let (ref_id, waypoints) = decode_source_waypoint_list_payload(payload)?;
        Some(Self { ref_id, waypoints })
    }
}

impl CommandPayload for SendRallypointsPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SendRallypoints;

    fn encode_payload(&self) -> Vec<u8> {
        encode_source_waypoint_list_payload(self.ref_id, &self.waypoints)
    }
}

impl SendRallypointsPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        let (ref_id, waypoints) = decode_source_waypoint_list_payload(payload)?;
        Some(Self { ref_id, waypoints })
    }
}

fn encode_source_waypoint_list_payload(ref_id: i32, waypoints: &[SourceWaypoint]) -> Vec<u8> {
    let waypoint_amount = i32::try_from(waypoints.len()).expect("waypoint count exceeds C int");
    let mut payload = Vec::with_capacity(8 + waypoints.len() * SOURCE_WAYPOINT_PACKET_SIZE);
    payload.extend_from_slice(&ref_id.to_le_bytes());
    payload.extend_from_slice(&waypoint_amount.to_le_bytes());
    for waypoint in waypoints {
        payload.extend_from_slice(&waypoint.encode_wire());
    }
    payload
}

fn decode_source_waypoint_list_payload(payload: &[u8]) -> Option<(i32, Vec<SourceWaypoint>)> {
    if payload.len() < 8 {
        return None;
    }

    let ref_id = i32::from_le_bytes(payload[0..4].try_into().ok()?);
    let waypoint_amount = i32::from_le_bytes(payload[4..8].try_into().ok()?);
    if waypoint_amount < 0 {
        return None;
    }
    let waypoint_amount = usize::try_from(waypoint_amount).ok()?;
    let expected_size = 8 + waypoint_amount.checked_mul(SOURCE_WAYPOINT_PACKET_SIZE)?;
    if payload.len() != expected_size {
        return None;
    }

    let mut waypoints = Vec::with_capacity(waypoint_amount);
    for wire in payload[8..].chunks_exact(SOURCE_WAYPOINT_PACKET_SIZE) {
        waypoints.push(SourceWaypoint::decode_wire(wire)?);
    }
    Some((ref_id, waypoints))
}

impl CommandPayload for ObjectGroupInfoPacket {
    const EVENT_ID: TcpEventId = TcpEventId::ObjectGroupInfo;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(12 + self.minion_refs.len() * 4);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&self.leader_ref_id.to_le_bytes());
        payload.extend_from_slice(&(self.minion_refs.len() as i32).to_le_bytes());
        for minion_ref in &self.minion_refs {
            payload.extend_from_slice(&minion_ref.to_le_bytes());
        }
        payload
    }
}

impl ObjectGroupInfoPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() < 12 {
            return None;
        }
        let ref_id = i32::from_le_bytes(payload[0..4].try_into().ok()?);
        let leader_ref_id = i32::from_le_bytes(payload[4..8].try_into().ok()?);
        let minions = i32::from_le_bytes(payload[8..12].try_into().ok()?);
        if minions < 0 {
            return None;
        }
        let minions = usize::try_from(minions).ok()?;
        if payload.len() != 12 + minions * 4 {
            return None;
        }
        let mut minion_refs = Vec::with_capacity(minions);
        for i in 0..minions {
            let start = 12 + i * 4;
            minion_refs.push(i32::from_le_bytes(
                payload[start..start + 4].try_into().ok()?,
            ));
        }
        Some(Self {
            ref_id,
            leader_ref_id,
            minion_refs,
        })
    }
}

impl CommandPayload for DeleteObjectPacket {
    const EVENT_ID: TcpEventId = TcpEventId::DeleteObject;

    fn encode_payload(&self) -> Vec<u8> {
        self.ref_id.to_le_bytes().to_vec()
    }
}

impl DeleteObjectPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 4 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload.try_into().ok()?),
        })
    }
}

impl CommandPayload for ObjectHealthPacket {
    const EVENT_ID: TcpEventId = TcpEventId::UpdateHealth;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&self.health.to_le_bytes());
        payload
    }
}

impl ObjectHealthPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 8 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            health: i32::from_le_bytes(payload[4..8].try_into().ok()?),
        })
    }
}

impl CommandPayload for BuildingStatePacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetBuildingState;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(26);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&self.state.to_le_bytes());
        payload.extend_from_slice(&self.init_offset.to_le_bytes());
        payload.extend_from_slice(&self.production_time.to_le_bytes());
        payload.push(self.object_type);
        payload.push(self.object_id);
        payload
    }
}

impl BuildingStatePacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 26 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            state: i32::from_le_bytes(payload[4..8].try_into().ok()?),
            init_offset: f64::from_le_bytes(payload[8..16].try_into().ok()?),
            production_time: f64::from_le_bytes(payload[16..24].try_into().ok()?),
            object_type: payload[24],
            object_id: payload[25],
        })
    }
}

impl CommandPayload for BuildingQueuePacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetBuildingQueueList;

    fn encode_payload(&self) -> Vec<u8> {
        let count = i32::try_from(self.units.len()).expect("building queue exceeds C int");
        let mut payload = Vec::with_capacity(8 + self.units.len() * 2);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&count.to_le_bytes());
        for unit in &self.units {
            payload.push(unit.object_type);
            payload.push(unit.object_id);
        }
        payload
    }
}

impl BuildingQueuePacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() < 8 {
            return None;
        }
        let count = i32::from_le_bytes(payload[4..8].try_into().ok()?);
        let count = usize::try_from(count).ok()?;
        if payload.len() != 8 + count.checked_mul(2)? {
            return None;
        }
        let units = payload[8..]
            .chunks_exact(2)
            .map(|unit| BuildingQueueUnit {
                object_type: unit[0],
                object_id: unit[1],
            })
            .collect();
        Some(Self {
            ref_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            units,
        })
    }
}

impl CommandPayload for DestroyObjectPacket {
    const EVENT_ID: TcpEventId = TcpEventId::DestroyObject;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(15 + self.fire_missiles.len() * 16);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        let fire_missile_amount = i32::try_from(self.fire_missiles.len()).unwrap_or(i32::MAX);
        payload.extend_from_slice(&fire_missile_amount.to_le_bytes());
        payload.extend_from_slice(&self.killer_ref_id.to_le_bytes());
        payload.push(u8::from(self.destroy_object));
        payload.push(u8::from(self.do_fire_death));
        payload.push(u8::from(self.do_missile_death));
        for missile in &self.fire_missiles {
            payload.extend_from_slice(&missile.missile_offset_time.to_le_bytes());
            payload.extend_from_slice(&missile.missile_x.to_le_bytes());
            payload.extend_from_slice(&missile.missile_y.to_le_bytes());
        }
        payload
    }
}

impl DestroyObjectPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() < 15 {
            return None;
        }
        let ref_id = i32::from_le_bytes(payload[0..4].try_into().ok()?);
        let fire_missile_amount = i32::from_le_bytes(payload[4..8].try_into().ok()?);
        if fire_missile_amount < 0 {
            return None;
        }
        let fire_missile_amount = usize::try_from(fire_missile_amount).ok()?;
        let expected_len = 15 + fire_missile_amount * 16;
        if payload.len() != expected_len {
            return None;
        }

        let mut fire_missiles = Vec::with_capacity(fire_missile_amount);
        for index in 0..fire_missile_amount {
            let start = 15 + index * 16;
            fire_missiles.push(DestroyObjectMissileInfo {
                missile_offset_time: f64::from_le_bytes(payload[start..start + 8].try_into().ok()?),
                missile_x: i32::from_le_bytes(payload[start + 8..start + 12].try_into().ok()?),
                missile_y: i32::from_le_bytes(payload[start + 12..start + 16].try_into().ok()?),
            });
        }

        Some(Self {
            ref_id,
            killer_ref_id: i32::from_le_bytes(payload[8..12].try_into().ok()?),
            destroy_object: decode_bool_byte(payload[12])?,
            do_fire_death: decode_bool_byte(payload[13])?,
            do_missile_death: decode_bool_byte(payload[14])?,
            fire_missiles,
        })
    }
}

impl CommandPayload for ObjectGrenadeAmountPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetGrenadeAmount;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&self.grenade_amount.to_le_bytes());
        payload
    }
}

impl ObjectGrenadeAmountPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 8 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            grenade_amount: i32::from_le_bytes(payload[4..8].try_into().ok()?),
        })
    }
}

impl CommandPayload for PickupGrenadeAnimationPacket {
    const EVENT_ID: TcpEventId = TcpEventId::PickupGrenadeAnimation;

    fn encode_payload(&self) -> Vec<u8> {
        self.ref_id.to_le_bytes().to_vec()
    }
}

impl PickupGrenadeAnimationPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 4 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload.try_into().ok()?),
        })
    }
}

impl CommandPayload for SetLidOpenPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetLidOpen;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(5);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.push(u8::from(self.lid_open));
        payload
    }
}

impl SetLidOpenPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 5 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            lid_open: decode_bool_byte(payload[4])?,
        })
    }
}

impl CommandPayload for EjectVehiclePacket {
    const EVENT_ID: TcpEventId = TcpEventId::EjectVehicle;

    fn encode_payload(&self) -> Vec<u8> {
        self.ref_id.to_le_bytes().to_vec()
    }
}

impl EjectVehiclePacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 4 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload.try_into().ok()?),
        })
    }
}

impl CommandPayload for CraneAnimPacket {
    const EVENT_ID: TcpEventId = TcpEventId::DoCraneAnim;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(9);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&self.repair_ref_id.to_le_bytes());
        payload.push(u8::from(self.on));
        payload
    }
}

impl CraneAnimPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 9 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            repair_ref_id: i32::from_le_bytes(payload[4..8].try_into().ok()?),
            on: decode_bool_byte(payload[8])?,
        })
    }
}

impl CommandPayload for RepairBuildingAnimPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetRepairAnim;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(14);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.push(u8::from(self.on));
        payload.extend_from_slice(&self.remaining_time.to_le_bytes());
        payload.push(u8::from(self.play_sound));
        payload
    }
}

impl RepairBuildingAnimPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 14 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            on: decode_bool_byte(payload[4])?,
            remaining_time: f64::from_le_bytes(payload[5..13].try_into().ok()?),
            play_sound: decode_bool_byte(payload[13])?,
        })
    }
}

impl CommandPayload for SnipeObjectPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SnipeObject;

    fn encode_payload(&self) -> Vec<u8> {
        self.ref_id.to_le_bytes().to_vec()
    }
}

impl SnipeObjectPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 4 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload.try_into().ok()?),
        })
    }
}

impl CommandPayload for DriverHitEffectPacket {
    const EVENT_ID: TcpEventId = TcpEventId::DriverHitEffect;

    fn encode_payload(&self) -> Vec<u8> {
        self.ref_id.to_le_bytes().to_vec()
    }
}

impl DriverHitEffectPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 4 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload.try_into().ok()?),
        })
    }
}

impl CommandPayload for ComputerMessagePacket {
    const EVENT_ID: TcpEventId = TcpEventId::ComputerMessage;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&self.sound.to_le_bytes());
        payload
    }
}

impl ComputerMessagePacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 8 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            sound: i32::from_le_bytes(payload[4..8].try_into().ok()?),
        })
    }
}

impl CommandPayload for DoPortraitAnimPacket {
    const EVENT_ID: TcpEventId = TcpEventId::DoPortraitAnim;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&self.anim_id.to_le_bytes());
        payload
    }
}

impl DoPortraitAnimPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 8 {
            return None;
        }
        Some(Self {
            ref_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            anim_id: i32::from_le_bytes(payload[4..8].try_into().ok()?),
        })
    }
}

impl CommandPayload for RequestSelectableMapListCommand {
    const EVENT_ID: TcpEventId = TcpEventId::RequestSelectableMapList;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl RequestSelectableMapListCommand {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        payload.is_empty().then_some(Self)
    }
}

impl CommandPayload for GiveSelectableMapListPacket {
    const EVENT_ID: TcpEventId = TcpEventId::GiveSelectableMapList;

    fn encode_payload(&self) -> Vec<u8> {
        if self.maps.is_empty() {
            return Vec::new();
        }

        let send_str = self.maps.join(",");
        let mut payload = Vec::with_capacity(send_str.len() + 1);
        payload.extend_from_slice(send_str.as_bytes());
        payload.push(0);
        payload
    }
}

impl GiveSelectableMapListPacket {
    pub(crate) fn new(maps: impl IntoIterator<Item = impl Into<String>>) -> Option<Self> {
        let maps = maps.into_iter().map(Into::into).collect::<Vec<String>>();
        if maps
            .iter()
            .any(|map| map.as_bytes().contains(&0) || map.as_bytes().contains(&b','))
        {
            return None;
        }
        Some(Self { maps })
    }

    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() <= 1 {
            return Some(Self { maps: Vec::new() });
        }
        if payload.last().copied()? != 0 {
            return None;
        }

        let body = &payload[..payload.len() - 1];
        if body.contains(&0) {
            return None;
        }

        let mut maps = Vec::new();
        let mut start = 0;
        while start < body.len() {
            let end = body[start..]
                .iter()
                .position(|byte| *byte == b',')
                .map_or(body.len(), |offset| start + offset);
            let segment = &body[start..end];
            let segment = &segment[..segment.len().min(499)];
            maps.push(std::str::from_utf8(segment).ok()?.to_owned());
            start = if end < body.len() { end + 1 } else { end };
        }

        Some(Self { maps })
    }
}

impl CommandPayload for ClearPlayerListPacket {
    const EVENT_ID: TcpEventId = TcpEventId::ClearPlayerList;

    fn encode_payload(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl ClearPlayerListPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        payload.is_empty().then_some(Self)
    }
}

impl CommandPayload for PlayerIdPacket {
    const EVENT_ID: TcpEventId = TcpEventId::GivePlayerId;

    fn encode_payload(&self) -> Vec<u8> {
        self.player_id.to_le_bytes().to_vec()
    }
}

impl PlayerIdPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 4 {
            return None;
        }
        Some(Self {
            player_id: i32::from_le_bytes(payload.try_into().ok()?),
        })
    }
}

impl CommandPayload for AddLocalPlayerPacket {
    const EVENT_ID: TcpEventId = TcpEventId::AddLocalPlayer;

    fn encode_payload(&self) -> Vec<u8> {
        self.player_id.to_le_bytes().to_vec()
    }
}

impl AddLocalPlayerPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 4 {
            return None;
        }
        Some(Self {
            player_id: i32::from_le_bytes(payload.try_into().ok()?),
        })
    }
}

impl CommandPayload for DeleteLocalPlayerPacket {
    const EVENT_ID: TcpEventId = TcpEventId::DeleteLocalPlayer;

    fn encode_payload(&self) -> Vec<u8> {
        self.player_id.to_le_bytes().to_vec()
    }
}

impl DeleteLocalPlayerPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 4 {
            return None;
        }
        Some(Self {
            player_id: i32::from_le_bytes(payload.try_into().ok()?),
        })
    }
}

impl CommandPayload for SetLocalPlayerNamePacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetLocalPlayerName;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(5 + self.name.len());
        payload.extend_from_slice(&self.player_id.to_le_bytes());
        payload.extend_from_slice(self.name.as_bytes());
        payload.push(0);
        payload
    }
}

impl SetLocalPlayerNamePacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() < 5 || payload.last().copied()? != 0 {
            return None;
        }
        let player_id = i32::from_le_bytes(payload[0..4].try_into().ok()?);
        let name_bytes = &payload[4..];
        let nul_index = name_bytes.iter().position(|byte| *byte == 0)?;
        Some(Self {
            player_id,
            name: std::str::from_utf8(&name_bytes[..nul_index])
                .ok()?
                .to_owned(),
        })
    }
}

impl CommandPayload for SetLocalPlayerTeamPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetLocalPlayerTeam;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&self.player_id.to_le_bytes());
        payload.extend_from_slice(&self.team.to_le_bytes());
        payload
    }
}

impl SetLocalPlayerTeamPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 8 {
            return None;
        }
        Some(Self {
            player_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            team: i32::from_le_bytes(payload[4..8].try_into().ok()?),
        })
    }
}

impl CommandPayload for SetLocalPlayerModePacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetLocalPlayerMode;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&self.player_id.to_le_bytes());
        payload.extend_from_slice(&self.mode.to_le_bytes());
        payload
    }
}

impl SetLocalPlayerModePacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 8 {
            return None;
        }
        Some(Self {
            player_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            mode: i32::from_le_bytes(payload[4..8].try_into().ok()?),
        })
    }
}

impl CommandPayload for SetLocalPlayerIgnoredPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetLocalPlayerIgnored;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&self.player_id.to_le_bytes());
        payload.extend_from_slice(&self.ignored.to_le_bytes());
        payload
    }
}

impl SetLocalPlayerIgnoredPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 8 {
            return None;
        }
        Some(Self {
            player_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            ignored: i32::from_le_bytes(payload[4..8].try_into().ok()?),
        })
    }
}

impl CommandPayload for SetLocalPlayerLogInfoPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetLocalPlayerLogInfo;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(19);
        payload.extend_from_slice(&self.player_id.to_le_bytes());
        payload.extend_from_slice(&self.db_id.to_le_bytes());
        payload.extend_from_slice(&self.voting_power.to_le_bytes());
        payload.extend_from_slice(&self.total_games.to_le_bytes());
        payload.push(u8::from(self.activated));
        payload.push(u8::from(self.logged_in));
        payload.push(u8::from(self.bot_logged_in));
        payload
    }
}

impl SetLocalPlayerLogInfoPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 19 {
            return None;
        }
        Some(Self {
            player_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            db_id: i32::from_le_bytes(payload[4..8].try_into().ok()?),
            voting_power: i32::from_le_bytes(payload[8..12].try_into().ok()?),
            total_games: i32::from_le_bytes(payload[12..16].try_into().ok()?),
            activated: decode_bool_byte(payload[16])?,
            logged_in: decode_bool_byte(payload[17])?,
            bot_logged_in: decode_bool_byte(payload[18])?,
        })
    }
}

impl CommandPayload for SetLocalPlayerVoteInfoPacket {
    const EVENT_ID: TcpEventId = TcpEventId::SetLocalPlayerVoteInfo;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&self.player_id.to_le_bytes());
        payload.extend_from_slice(&self.vote_choice.to_le_bytes());
        payload
    }
}

impl SetLocalPlayerVoteInfoPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 8 {
            return None;
        }
        Some(Self {
            player_id: i32::from_le_bytes(payload[0..4].try_into().ok()?),
            vote_choice: i32::from_le_bytes(payload[4..8].try_into().ok()?),
        })
    }
}

impl CommandPayload for VoteInfoPacket {
    const EVENT_ID: TcpEventId = TcpEventId::VoteInfo;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(9);
        payload.push(u8::from(self.in_progress));
        payload.extend_from_slice(&self.vote_type.to_le_bytes());
        payload.extend_from_slice(&self.value.to_le_bytes());
        payload
    }
}

impl VoteInfoPacket {
    pub(crate) fn decode_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() != 9 {
            return None;
        }
        let in_progress = decode_bool_byte(payload[0])?;
        let vote_type = i32::from_le_bytes(payload[1..5].try_into().ok()?);
        let value = i32::from_le_bytes(payload[5..9].try_into().ok()?);
        Some(Self {
            in_progress,
            vote_type,
            value,
        })
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlaceCannonCommand {
    pub ref_id: i32,
    pub tx: i32,
    pub ty: i32,
    pub cannon_id: u8,
}

#[cfg(test)]
impl PlaceCannonCommand {
    pub fn new(ref_id: i32, tx: i32, ty: i32, kind: ObjectKind) -> Option<Self> {
        let ObjectKind::Cannon(cannon) = kind else {
            return None;
        };

        Some(Self {
            ref_id,
            tx,
            ty,
            cannon_id: cannon as u8,
        })
    }
}

#[cfg(test)]
impl CommandPayload for PlaceCannonCommand {
    const EVENT_ID: TcpEventId = TcpEventId::PlaceCannon;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(13);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&self.tx.to_le_bytes());
        payload.extend_from_slice(&self.ty.to_le_bytes());
        payload.push(self.cannon_id);
        payload
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AddBuildingQueueCommand {
    pub ref_id: i32,
    pub object_type: u8,
    pub object_id: u8,
}

#[cfg(test)]
impl AddBuildingQueueCommand {
    pub fn new(ref_id: i32, kind: ObjectKind) -> Option<Self> {
        let (object_type, object_id) = object_kind_wire_parts(kind)?;
        Some(Self {
            ref_id,
            object_type,
            object_id,
        })
    }
}

#[cfg(test)]
impl CommandPayload for AddBuildingQueueCommand {
    const EVENT_ID: TcpEventId = TcpEventId::AddBuildingQueue;

    fn encode_payload(&self) -> Vec<u8> {
        encode_start_like_payload(self.ref_id, self.object_type, self.object_id)
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CancelBuildingQueueCommand {
    pub ref_id: i32,
    pub list_index: i32,
    pub object_type: u8,
    pub object_id: u8,
}

#[cfg(test)]
impl CancelBuildingQueueCommand {
    pub fn new(ref_id: i32, list_index: i32, kind: ObjectKind) -> Option<Self> {
        let (object_type, object_id) = object_kind_wire_parts(kind)?;
        Some(Self {
            ref_id,
            list_index,
            object_type,
            object_id,
        })
    }
}

#[cfg(test)]
impl CommandPayload for CancelBuildingQueueCommand {
    const EVENT_ID: TcpEventId = TcpEventId::CancelBuildingQueue;

    fn encode_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(10);
        payload.extend_from_slice(&self.ref_id.to_le_bytes());
        payload.extend_from_slice(&self.list_index.to_le_bytes());
        payload.push(self.object_type);
        payload.push(self.object_id);
        payload
    }
}

pub(crate) fn encode_packet(event_id: TcpEventId, payload: &[u8]) -> Vec<u8> {
    let payload_len = i32::try_from(payload.len()).expect("payload too large for C int length");
    let mut packet = Vec::with_capacity(8 + payload.len());
    packet.extend_from_slice(&payload_len.to_le_bytes());
    packet.extend_from_slice(&event_id.wire_id().to_le_bytes());
    packet.extend_from_slice(payload);
    packet
}

#[cfg(test)]
pub fn object_kind_wire_parts(kind: ObjectKind) -> Option<(u8, u8)> {
    match kind {
        ObjectKind::Rock => Some((MapObjectType::MapItem as u8, ItemType::Rock as u8)),
        ObjectKind::Bridge(building) => Some((MapObjectType::Bridge as u8, building as u8)),
        ObjectKind::Building(building) => Some((MapObjectType::Building as u8, building as u8)),
        ObjectKind::Cannon(cannon) => Some((MapObjectType::Cannon as u8, cannon as u8)),
        ObjectKind::Vehicle(vehicle) => Some((MapObjectType::Vehicle as u8, vehicle as u8)),
        ObjectKind::Robot(robot) => Some((MapObjectType::Robot as u8, robot as u8)),
        ObjectKind::Animal(id) => Some((MapObjectType::Animal as u8, id)),
        ObjectKind::MapItem(id) => Some((MapObjectType::MapItem as u8, id)),
    }
}

#[cfg(test)]
fn encode_start_like_payload(ref_id: i32, object_type: u8, object_id: u8) -> Vec<u8> {
    let mut payload = Vec::with_capacity(6);
    payload.extend_from_slice(&ref_id.to_le_bytes());
    payload.push(object_type);
    payload.push(object_id);
    payload
}

fn encode_game_paused_payload(game_paused: bool) -> Vec<u8> {
    vec![u8::from(game_paused)]
}

fn decode_game_paused_payload(payload: &[u8]) -> Option<bool> {
    match payload {
        [0] => Some(false),
        [1] => Some(true),
        _ => None,
    }
}

fn decode_bool_byte(value: u8) -> Option<bool> {
    match value {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::original::objects::{BuildingType, CannonType, RobotType, VehicleType};

    #[test]
    fn tcp_event_ids_match_c_enum_positions() {
        assert_eq!(TcpEventId::RequestMap.wire_id(), 1);
        assert_eq!(TcpEventId::StoreMap.wire_id(), 3);
        assert_eq!(TcpEventId::RequestObjects.wire_id(), 4);
        assert_eq!(TcpEventId::RequestZones.wire_id(), 5);
        assert_eq!(TcpEventId::AddNewObject.wire_id(), 6);
        assert_eq!(TcpEventId::SetZoneInfo.wire_id(), 7);
        assert_eq!(TcpEventId::SetName.wire_id(), 8);
        assert_eq!(TcpEventId::SetTeam.wire_id(), 9);
        assert_eq!(TcpEventId::NewsEvent.wire_id(), 10);
        assert_eq!(TcpEventId::SendWaypoints.wire_id(), 11);
        assert_eq!(TcpEventId::SendRallypoints.wire_id(), 12);
        assert_eq!(TcpEventId::SendLoc.wire_id(), 13);
        assert_eq!(TcpEventId::SetObjectTeam.wire_id(), 14);
        assert_eq!(TcpEventId::SetAttackObject.wire_id(), 15);
        assert_eq!(TcpEventId::UpdateHealth.wire_id(), 17);
        assert_eq!(TcpEventId::EndGame.wire_id(), 18);
        assert_eq!(TcpEventId::ResetGame.wire_id(), 19);
        assert_eq!(TcpEventId::DestroyObject.wire_id(), 21);
        assert_eq!(TcpEventId::StartBuilding.wire_id(), 22);
        assert_eq!(TcpEventId::StopBuilding.wire_id(), 23);
        assert_eq!(TcpEventId::SetBuildingState.wire_id(), 24);
        assert_eq!(TcpEventId::SetBuiltCannonAmount.wire_id(), 25);
        assert_eq!(TcpEventId::PlaceCannon.wire_id(), 26);
        assert_eq!(TcpEventId::SendChat.wire_id(), 27);
        assert_eq!(TcpEventId::ComputerMessage.wire_id(), 28);
        assert_eq!(TcpEventId::ObjectGroupInfo.wire_id(), 29);
        assert_eq!(TcpEventId::EjectVehicle.wire_id(), 30);
        assert_eq!(TcpEventId::DoCraneAnim.wire_id(), 31);
        assert_eq!(TcpEventId::SetRepairAnim.wire_id(), 32);
        assert_eq!(TcpEventId::RequestSettings.wire_id(), 33);
        assert_eq!(TcpEventId::SetSettings.wire_id(), 34);
        assert_eq!(TcpEventId::SetLidOpen.wire_id(), 35);
        assert_eq!(TcpEventId::SnipeObject.wire_id(), 36);
        assert_eq!(TcpEventId::DriverHitEffect.wire_id(), 37);
        assert_eq!(TcpEventId::SetPlayerMode.wire_id(), 38);
        assert_eq!(TcpEventId::RequestPlayerList.wire_id(), 39);
        assert_eq!(TcpEventId::ClearPlayerList.wire_id(), 40);
        assert_eq!(TcpEventId::AddLocalPlayer.wire_id(), 41);
        assert_eq!(TcpEventId::DeleteLocalPlayer.wire_id(), 42);
        assert_eq!(TcpEventId::SetLocalPlayerName.wire_id(), 43);
        assert_eq!(TcpEventId::SetLocalPlayerTeam.wire_id(), 44);
        assert_eq!(TcpEventId::SetLocalPlayerMode.wire_id(), 45);
        assert_eq!(TcpEventId::SetLocalPlayerIgnored.wire_id(), 46);
        assert_eq!(TcpEventId::SetLocalPlayerLogInfo.wire_id(), 47);
        assert_eq!(TcpEventId::SetLocalPlayerVoteInfo.wire_id(), 48);
        assert_eq!(TcpEventId::UpdateGamePaused.wire_id(), 50);
        assert_eq!(TcpEventId::GetGamePaused.wire_id(), 51);
        assert_eq!(TcpEventId::SetGamePaused.wire_id(), 52);
        assert_eq!(TcpEventId::StartVote.wire_id(), 53);
        assert_eq!(TcpEventId::VoteYes.wire_id(), 54);
        assert_eq!(TcpEventId::VoteNo.wire_id(), 55);
        assert_eq!(TcpEventId::VotePass.wire_id(), 56);
        assert_eq!(TcpEventId::VoteInfo.wire_id(), 57);
        assert_eq!(TcpEventId::GivePlayerId.wire_id(), 58);
        assert_eq!(TcpEventId::RequestPlayerId.wire_id(), 59);
        assert_eq!(TcpEventId::RequestSelectableMapList.wire_id(), 60);
        assert_eq!(TcpEventId::GiveSelectableMapList.wire_id(), 61);
        assert_eq!(TcpEventId::SendLogin.wire_id(), 62);
        assert_eq!(TcpEventId::RequestLoginOff.wire_id(), 63);
        assert_eq!(TcpEventId::GiveLoginOff.wire_id(), 64);
        assert_eq!(TcpEventId::CreateUser.wire_id(), 65);
        assert_eq!(TcpEventId::SetGrenadeAmount.wire_id(), 66);
        assert_eq!(TcpEventId::PickupGrenadeAnimation.wire_id(), 67);
        assert_eq!(TcpEventId::DoPortraitAnim.wire_id(), 68);
        assert_eq!(TcpEventId::TeamEnded.wire_id(), 69);
        assert_eq!(TcpEventId::PollBuyRegistrationKey.wire_id(), 70);
        assert_eq!(TcpEventId::BuyRegistrationKey.wire_id(), 71);
        assert_eq!(TcpEventId::ReturnRegistrationKey.wire_id(), 72);
        assert_eq!(TcpEventId::GetGameSpeed.wire_id(), 73);
        assert_eq!(TcpEventId::SetGameSpeed.wire_id(), 74);
        assert_eq!(TcpEventId::UpdateGameSpeed.wire_id(), 75);
        assert_eq!(TcpEventId::AddBuildingQueue.wire_id(), 76);
        assert_eq!(TcpEventId::SetBuildingQueueList.wire_id(), 77);
        assert_eq!(TcpEventId::CancelBuildingQueue.wire_id(), 78);
        assert_eq!(TcpEventId::ReshuffleTeams.wire_id(), 79);
        assert_eq!(TcpEventId::StartBot.wire_id(), 80);
        assert_eq!(TcpEventId::StopBot.wire_id(), 81);
        assert_eq!(TcpEventId::SelectMap.wire_id(), 82);
        assert_eq!(TcpEventId::ResetMap.wire_id(), 83);
        assert_eq!(TcpEventId::RequestVersion.wire_id(), 84);
        assert_eq!(TcpEventId::GiveVersion.wire_id(), 85);
    }

    #[test]
    fn non_pause_vote_commands_match_source_int_and_empty_packet_layouts() {
        let start = StartBotCommand { team: 4 };
        assert_eq!(start.encode_payload(), 4_i32.to_le_bytes());
        assert_eq!(
            StartBotCommand::decode_payload(&start.encode_payload()),
            Some(start)
        );

        let stop = StopBotCommand { team: 2 };
        assert_eq!(stop.encode_payload(), 2_i32.to_le_bytes());
        assert_eq!(
            StopBotCommand::decode_payload(&stop.encode_payload()),
            Some(stop)
        );

        let map = SelectMapCommand { map_num: 17 };
        assert_eq!(map.encode_payload(), 17_i32.to_le_bytes());
        assert_eq!(
            SelectMapCommand::decode_payload(&map.encode_payload()),
            Some(map)
        );

        assert_eq!(ReshuffleTeamsCommand.encode_payload(), Vec::<u8>::new());
        assert_eq!(
            ReshuffleTeamsCommand::decode_payload(&[]),
            Some(ReshuffleTeamsCommand)
        );
        assert_eq!(ResetMapCommand.encode_payload(), Vec::<u8>::new());
        assert_eq!(ResetMapCommand::decode_payload(&[]), Some(ResetMapCommand));
        assert_eq!(StartBotCommand::decode_payload(&[1, 0, 0]), None);
        assert_eq!(ResetMapCommand::decode_payload(&[0]), None);
    }

    #[test]
    fn reset_game_packet_matches_source_empty_event() {
        assert_eq!(
            ResetGamePacket.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x13, 0x00, 0x00, 0x00, // RESET_GAME
            ]
        );
        assert_eq!(ResetGamePacket::decode_payload(&[]), Some(ResetGamePacket));
        assert_eq!(ResetGamePacket::decode_payload(&[0]), None);
    }

    #[test]
    fn end_game_and_team_ended_packets_match_source_layouts() {
        assert_eq!(
            EndGamePacket.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x12, 0x00, 0x00, 0x00, // END_GAME
            ]
        );
        assert_eq!(EndGamePacket::decode_payload(&[]), Some(EndGamePacket));
        assert_eq!(EndGamePacket::decode_payload(&[0]), None);

        let won = TeamEndedPacket { team: 2, won: true };
        assert_eq!(
            won.encode_packet(),
            vec![
                0x08, 0x00, 0x00, 0x00, // payload length
                0x45, 0x00, 0x00, 0x00, // TEAM_ENDED
                0x02, 0x00, 0x00, 0x00, // team
                0x01, 0x00, 0x00, 0x00, // won + native padding
            ]
        );
        assert_eq!(
            TeamEndedPacket::decode_payload(&won.encode_payload()),
            Some(won)
        );
        assert_eq!(TeamEndedPacket::decode_payload(&[0; 7]), None);
        assert_eq!(TeamEndedPacket::decode_payload(&[0; 9]), None);
        assert_eq!(
            TeamEndedPacket::decode_payload(&[2, 0, 0, 0, 2, 0, 0, 0]),
            None
        );
    }

    #[test]
    fn account_commands_match_source_ascii_and_packed_bool_layouts() {
        let login = SendLoginCommand {
            login_name: "alice".to_string(),
            password: "secret".to_string(),
        };
        assert_eq!(login.encode_payload(), b"alice,secret\0");
        assert_eq!(
            SendLoginCommand::decode_payload(&login.encode_payload()),
            Some(login)
        );

        let create = CreateUserCommand {
            user_name: "Alice".to_string(),
            login_name: "alice".to_string(),
            password: "secret".to_string(),
            email: "alice@example.test".to_string(),
        };
        assert_eq!(
            CreateUserCommand::decode_payload(&create.encode_payload()),
            Some(create)
        );
        assert_eq!(RequestLoginOffCommand.encode_payload(), Vec::<u8>::new());
        assert_eq!(
            RequestLoginOffCommand::decode_payload(&[]),
            Some(RequestLoginOffCommand)
        );
        assert_eq!(
            GiveLoginOffPacket::decode_payload(&[1]),
            Some(GiveLoginOffPacket { show_login: true })
        );
        assert_eq!(GiveLoginOffPacket::decode_payload(&[2]), None);
    }

    #[test]
    fn registration_packets_are_empty_poll_and_exact_sixteen_byte_blocks() {
        assert_eq!(
            PollBuyRegistrationKeyPacket::decode_payload(&[]),
            Some(PollBuyRegistrationKeyPacket)
        );
        assert_eq!(PollBuyRegistrationKeyPacket::decode_payload(&[0]), None);

        let device_id = *b"zod-rust-client!";
        let buy = BuyRegistrationKeyCommand { device_id };
        assert_eq!(buy.encode_payload(), device_id);
        assert_eq!(
            BuyRegistrationKeyCommand::decode_payload(&buy.encode_payload()),
            Some(buy)
        );

        let returned = ReturnRegistrationKeyPacket {
            encrypted_key: [0xA5; 16],
        };
        assert_eq!(
            ReturnRegistrationKeyPacket::decode_payload(&returned.encrypted_key),
            Some(returned)
        );
        assert_eq!(BuyRegistrationKeyCommand::decode_payload(&[0; 15]), None);
    }

    #[test]
    fn object_kind_mapping_matches_original_type_ids() {
        assert_eq!(
            object_kind_wire_parts(ObjectKind::Building(BuildingType::RobotFactory)),
            Some((2, 4))
        );
        assert_eq!(
            object_kind_wire_parts(ObjectKind::Cannon(CannonType::MissileCannon)),
            Some((3, 3))
        );
        assert_eq!(
            object_kind_wire_parts(ObjectKind::Vehicle(VehicleType::Crane)),
            Some((4, 6))
        );
        assert_eq!(
            object_kind_wire_parts(ObjectKind::Robot(RobotType::Laser)),
            Some((5, 5))
        );
        assert_eq!(object_kind_wire_parts(ObjectKind::Rock), Some((7, 1)));
    }

    #[test]
    fn encodes_start_building_packet_layout() {
        let command = StartBuildingCommand::new(
            0x0102_0304,
            ObjectKind::Building(BuildingType::RobotFactory),
        )
        .unwrap();

        assert_eq!(
            command.encode_packet(),
            vec![
                0x06, 0x00, 0x00, 0x00, // payload length
                0x16, 0x00, 0x00, 0x00, // START_BUILDING
                0x04, 0x03, 0x02, 0x01, // ref_id
                0x02, 0x04, // object type, object id
            ]
        );
    }

    #[test]
    fn encodes_stop_building_packet_layout() {
        let command = StopBuildingCommand { ref_id: -2 };

        assert_eq!(
            command.encode_packet(),
            vec![
                0x04, 0x00, 0x00, 0x00, // payload length
                0x17, 0x00, 0x00, 0x00, // STOP_BUILDING
                0xfe, 0xff, 0xff, 0xff, // ref_id
            ]
        );
    }

    #[test]
    fn encodes_player_info_command_packets_like_source_connect_path() {
        let name = SetNameCommand::new("Player").unwrap();
        let team = SetTeamCommand { team: 1 };
        let mode = SetPlayerModeCommand { mode: 1 };

        assert_eq!(
            name.encode_packet(),
            vec![
                0x07, 0x00, 0x00, 0x00, // payload length
                0x08, 0x00, 0x00, 0x00, // SET_NAME
                b'P', b'l', b'a', b'y', b'e', b'r', 0x00,
            ]
        );
        assert_eq!(
            team.encode_packet(),
            vec![
                0x04, 0x00, 0x00, 0x00, // payload length
                0x09, 0x00, 0x00, 0x00, // SET_TEAM
                0x01, 0x00, 0x00, 0x00, // red
            ]
        );
        assert_eq!(
            mode.encode_packet(),
            vec![
                0x01, 0x00, 0x00, 0x00, // payload length
                0x26, 0x00, 0x00, 0x00, // SET_PLAYER_MODE
                0x01, // PLAYER_MODE
            ]
        );
        assert_eq!(
            SetNameCommand::decode_payload(&name.encode_payload()),
            Some(name)
        );
        assert_eq!(
            SetTeamCommand::decode_payload(&team.encode_payload()),
            Some(team)
        );
        assert_eq!(
            SetPlayerModeCommand::decode_payload(&mode.encode_payload()),
            Some(mode)
        );
    }

    #[test]
    fn encodes_set_local_player_name_team_and_mode_packets() {
        let name = SetLocalPlayerNamePacket {
            player_id: 7,
            name: "Alice".to_string(),
        };
        let team = SetLocalPlayerTeamPacket {
            player_id: 7,
            team: 2,
        };
        let mode = SetLocalPlayerModePacket {
            player_id: 7,
            mode: 1,
        };

        assert_eq!(
            name.encode_packet(),
            vec![
                0x0a, 0x00, 0x00, 0x00, // payload length
                0x2b, 0x00, 0x00, 0x00, // SET_LPLAYER_NAME
                0x07, 0x00, 0x00, 0x00, // p_id
                b'A', b'l', b'i', b'c', b'e', 0x00,
            ]
        );
        assert_eq!(
            team.encode_packet(),
            vec![
                0x08, 0x00, 0x00, 0x00, // payload length
                0x2c, 0x00, 0x00, 0x00, // SET_LPLAYER_TEAM
                0x07, 0x00, 0x00, 0x00, // p_id
                0x02, 0x00, 0x00, 0x00, // blue
            ]
        );
        assert_eq!(
            mode.encode_packet(),
            vec![
                0x08, 0x00, 0x00, 0x00, // payload length
                0x2d, 0x00, 0x00, 0x00, // SET_LPLAYER_MODE
                0x07, 0x00, 0x00, 0x00, // p_id
                0x01, 0x00, 0x00, 0x00, // PLAYER_MODE
            ]
        );
        assert_eq!(
            SetLocalPlayerNamePacket::decode_payload(&name.encode_payload()),
            Some(name)
        );
        assert_eq!(
            SetLocalPlayerTeamPacket::decode_payload(&team.encode_payload()),
            Some(team)
        );
        assert_eq!(
            SetLocalPlayerModePacket::decode_payload(&mode.encode_payload()),
            Some(mode)
        );
    }

    #[test]
    fn encodes_request_player_id_and_list_packets_like_source_connect_path() {
        assert_eq!(
            RequestMapCommand.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x01, 0x00, 0x00, 0x00, // REQUEST_MAP
            ]
        );
        assert_eq!(
            RequestMapCommand::decode_payload(&[]),
            Some(RequestMapCommand)
        );
        assert_eq!(RequestMapCommand::decode_payload(&[0]), None);
        assert_eq!(
            RequestObjectsCommand.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x04, 0x00, 0x00, 0x00, // REQUEST_OBJECTS
            ]
        );
        assert_eq!(
            RequestObjectsCommand::decode_payload(&[]),
            Some(RequestObjectsCommand)
        );
        assert_eq!(RequestObjectsCommand::decode_payload(&[0]), None);
        assert_eq!(
            RequestSettingsCommand.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x21, 0x00, 0x00, 0x00, // REQUEST_SETTINGS
            ]
        );
        assert_eq!(
            RequestSettingsCommand::decode_payload(&[]),
            Some(RequestSettingsCommand)
        );
        assert_eq!(RequestSettingsCommand::decode_payload(&[0]), None);
        assert_eq!(
            RequestZonesCommand.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x05, 0x00, 0x00, 0x00, // REQUEST_ZONES
            ]
        );
        assert_eq!(
            RequestZonesCommand::decode_payload(&[]),
            Some(RequestZonesCommand)
        );
        assert_eq!(RequestZonesCommand::decode_payload(&[0]), None);
        assert_eq!(
            RequestPlayerIdCommand.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x3b, 0x00, 0x00, 0x00, // REQUEST_PLAYER_ID
            ]
        );
        assert_eq!(
            RequestPlayerListCommand.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x27, 0x00, 0x00, 0x00, // REQUEST_PLAYER_LIST
            ]
        );
        assert_eq!(
            RequestSelectableMapListCommand.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x3c, 0x00, 0x00, 0x00, // REQUEST_SELECTABLE_MAP_LIST
            ]
        );
        assert_eq!(
            RequestSelectableMapListCommand::decode_payload(&[]),
            Some(RequestSelectableMapListCommand)
        );
        assert_eq!(RequestSelectableMapListCommand::decode_payload(&[0]), None);
    }

    #[test]
    fn encodes_store_map_packet_with_source_chunk_header() {
        let packet = StoreMapPacket {
            packet_number: 2,
            bytes: vec![b'm', b'a', b'p'],
        };
        let final_packet = StoreMapPacket {
            packet_number: -1,
            bytes: Vec::new(),
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x07, 0x00, 0x00, 0x00, // payload length
                0x03, 0x00, 0x00, 0x00, // STORE_MAP
                0x02, 0x00, 0x00, 0x00, // pack_num
                b'm', b'a', b'p',
            ]
        );
        assert_eq!(
            StoreMapPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(
            final_packet.encode_packet(),
            vec![
                0x04, 0x00, 0x00, 0x00, // payload length
                0x03, 0x00, 0x00, 0x00, // STORE_MAP
                0xff, 0xff, 0xff, 0xff, // final pack_num
            ]
        );
        assert_eq!(StoreMapPacket::decode_payload(&[0; 3]), None);
    }

    #[test]
    fn encodes_set_settings_raw_zsettings_payload() {
        let packet = SetSettingsPacket::new(vec![7; SOURCE_ZSETTINGS_PACKET_SIZE]).unwrap();
        let wire_packet = packet.encode_packet();

        assert_eq!(
            &wire_packet[..8],
            &[
                0x8c, 0x05, 0x00, 0x00, // payload length 1420
                0x22, 0x00, 0x00, 0x00, // SET_SETTINGS
            ]
        );
        assert!(wire_packet[8..].iter().all(|byte| *byte == 7));
        assert_eq!(
            SetSettingsPacket::decode_payload(&wire_packet[8..]),
            Some(packet)
        );
        assert_eq!(
            SetSettingsPacket::new(vec![0; SOURCE_ZSETTINGS_PACKET_SIZE - 1]),
            None
        );
        assert_eq!(
            SetSettingsPacket::decode_payload(&[0; SOURCE_ZSETTINGS_PACKET_SIZE - 1]),
            None
        );
        assert_eq!(
            SetSettingsPacket::decode_payload(&[0; SOURCE_ZSETTINGS_PACKET_SIZE + 1]),
            None
        );
    }

    #[test]
    fn encodes_set_zone_info_packet_layout() {
        let packet = SetZoneInfoPacket {
            zone_number: 3,
            owner: 2,
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x05, 0x00, 0x00, 0x00, // payload length
                0x07, 0x00, 0x00, 0x00, // SET_ZONE_INFO
                0x03, 0x00, 0x00, 0x00, // zone_number
                0x02, // owner
            ]
        );
        assert_eq!(
            SetZoneInfoPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(SetZoneInfoPacket::decode_payload(&[0; 4]), None);
        assert_eq!(SetZoneInfoPacket::decode_payload(&[0; 6]), None);
    }

    #[test]
    fn encodes_add_new_object_packet_layout() {
        let packet = ObjectInitPacket {
            x: 32,
            y: 48,
            ref_id: 7,
            owner: 1,
            object_type: 5,
            object_id: 3,
            building_level: -1,
            extra_links: 0x0102,
            health: 75,
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x16, 0x00, 0x00, 0x00, // payload length
                0x06, 0x00, 0x00, 0x00, // ADD_NEW_OBJECT
                0x20, 0x00, 0x00, 0x00, // x
                0x30, 0x00, 0x00, 0x00, // y
                0x07, 0x00, 0x00, 0x00, // ref_id
                0x01, // owner
                0x05, // object_type
                0x03, // object_id
                0xff, // blevel
                0x02, 0x01, // extra_links
                0x4b, 0x00, 0x00, 0x00, // health
            ]
        );
        assert_eq!(
            ObjectInitPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(ObjectInitPacket::decode_payload(&[0; 21]), None);
        assert_eq!(ObjectInitPacket::decode_payload(&[0; 23]), None);
    }

    #[test]
    fn encodes_set_built_cannon_amount_variable_packet_layout() {
        let packet = BuiltCannonListPacket {
            ref_id: 12,
            cannon_ids: vec![0, 3],
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x0a, 0x00, 0x00, 0x00, // payload length
                0x19, 0x00, 0x00, 0x00, // SET_BUILT_CANNON_AMOUNT
                0x0c, 0x00, 0x00, 0x00, // ref_id
                0x02, 0x00, 0x00, 0x00, // cannon amount
                0x00, 0x03, // cannon ids
            ]
        );
        assert_eq!(
            BuiltCannonListPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(BuiltCannonListPacket::decode_payload(&[0; 7]), None);
        let mut bad_amount = vec![0; 9];
        bad_amount[4..8].copy_from_slice(&2_i32.to_le_bytes());
        assert_eq!(BuiltCannonListPacket::decode_payload(&bad_amount), None);
    }

    #[test]
    fn encodes_set_object_team_packet_layout() {
        let packet = ObjectTeamPacket {
            ref_id: 7,
            owner: 1,
            driver_type: 0,
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
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x1f, 0x00, 0x00, 0x00, // payload length
                0x0e, 0x00, 0x00, 0x00, // SET_OBJECT_TEAM
                0x07, 0x00, 0x00, 0x00, // ref_id
                0x01, // owner
                0x00, // driver_type
                0x02, // driver_amount
                0x39, 0x04, 0x00, 0x00, // driver_health
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0x3f, // next_attack_time
                0xf4, 0x01, 0x00, 0x00, // driver_health
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // next_attack_time
            ]
        );
        assert_eq!(
            ObjectTeamPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(ObjectTeamPacket::decode_payload(&[0; 6]), None);
        assert_eq!(ObjectTeamPacket::decode_payload(&[0; 8]), None);
        let mut bad_amount = vec![0; 7];
        bad_amount[6] = 0xff;
        assert_eq!(ObjectTeamPacket::decode_payload(&bad_amount), None);
    }

    #[test]
    fn encodes_set_attack_object_packet_layout() {
        let packet = AttackObjectPacket {
            ref_id: 7,
            attack_object_ref_id: 9,
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x08, 0x00, 0x00, 0x00, // payload length
                0x0f, 0x00, 0x00, 0x00, // SET_ATTACK_OBJECT
                0x07, 0x00, 0x00, 0x00, // ref_id
                0x09, 0x00, 0x00, 0x00, // attack_object_ref_id
            ]
        );
        assert_eq!(
            AttackObjectPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(
            AttackObjectPacket::decode_payload(&[0x07, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,]),
            Some(AttackObjectPacket {
                ref_id: 7,
                attack_object_ref_id: -1,
            })
        );
        assert_eq!(AttackObjectPacket::decode_payload(&[0; 7]), None);
        assert_eq!(AttackObjectPacket::decode_payload(&[0; 9]), None);
    }

    #[test]
    fn encodes_send_waypoints_packet_layout() {
        let packet = SendWaypointsPacket {
            ref_id: 7,
            waypoints: vec![
                SourceWaypoint {
                    mode: SourceWaypointMode::Move,
                    ref_id: -1,
                    x: 32,
                    y: -48,
                    attack_to: true,
                    player_given: true,
                },
                SourceWaypoint {
                    mode: SourceWaypointMode::Attack,
                    ref_id: 9,
                    x: 100,
                    y: -116,
                    attack_to: true,
                    player_given: true,
                },
            ],
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x26, 0x00, 0x00, 0x00, // payload length
                0x0b, 0x00, 0x00, 0x00, // SEND_WAYPOINTS
                0x07, 0x00, 0x00, 0x00, // ref_id
                0x02, 0x00, 0x00, 0x00, // waypoint count
                0x00, // MOVE_WP
                0xff, 0xff, 0xff, 0xff, // waypoint ref_id
                0x20, 0x00, 0x00, 0x00, // x
                0xd0, 0xff, 0xff, 0xff, // y
                0x01, // attack_to
                0x01, // player_given
                0x02, // ATTACK_WP
                0x09, 0x00, 0x00, 0x00, // waypoint ref_id
                0x64, 0x00, 0x00, 0x00, // x
                0x8c, 0xff, 0xff, 0xff, // y
                0x01, // attack_to
                0x01, // player_given
            ]
        );
        assert_eq!(
            SendWaypointsPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
    }

    #[test]
    fn encodes_object_location_packet_layout() {
        let packet = ObjectLocationPacket {
            ref_id: 7,
            x: 32,
            y: 48,
            dx: 1.5,
            dy: -2.25,
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x14, 0x00, 0x00, 0x00, // payload length
                0x0d, 0x00, 0x00, 0x00, // SEND_LOC
                0x07, 0x00, 0x00, 0x00, // ref_id
                0x20, 0x00, 0x00, 0x00, // x
                0x30, 0x00, 0x00, 0x00, // y
                0x00, 0x00, 0xc0, 0x3f, // dx
                0x00, 0x00, 0x10, 0xc0, // dy
            ]
        );
        assert_eq!(
            ObjectLocationPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(ObjectLocationPacket::decode_payload(&[0; 19]), None);
    }

    #[test]
    fn encodes_send_rallypoints_packet_layout() {
        let packet = SendRallypointsPacket {
            ref_id: 12,
            waypoints: vec![SourceWaypoint {
                mode: SourceWaypointMode::Move,
                ref_id: -1,
                x: 64,
                y: -80,
                attack_to: true,
                player_given: true,
            }],
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x17, 0x00, 0x00, 0x00, // payload length
                0x0c, 0x00, 0x00, 0x00, // SEND_RALLYPOINTS
                0x0c, 0x00, 0x00, 0x00, // ref_id
                0x01, 0x00, 0x00, 0x00, // waypoint count
                0x00, // MOVE_WP
                0xff, 0xff, 0xff, 0xff, // waypoint ref_id
                0x40, 0x00, 0x00, 0x00, // x
                0xb0, 0xff, 0xff, 0xff, // y
                0x01, // attack_to
                0x01, // player_given
            ]
        );
        assert_eq!(
            SendRallypointsPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
    }

    #[test]
    fn rejects_invalid_send_waypoints_payloads() {
        assert_eq!(SendWaypointsPacket::decode_payload(&[0; 7]), None);
        assert_eq!(SendWaypointsPacket::decode_payload(&[0; 9]), None);

        let mut negative_count = Vec::new();
        negative_count.extend_from_slice(&7_i32.to_le_bytes());
        negative_count.extend_from_slice(&(-1_i32).to_le_bytes());
        assert_eq!(SendWaypointsPacket::decode_payload(&negative_count), None);

        let mut bad_mode = SendWaypointsPacket {
            ref_id: 7,
            waypoints: vec![SourceWaypoint {
                mode: SourceWaypointMode::Move,
                ref_id: -1,
                x: 0,
                y: 0,
                attack_to: false,
                player_given: false,
            }],
        }
        .encode_payload();
        bad_mode[8] = 42;
        assert_eq!(SendWaypointsPacket::decode_payload(&bad_mode), None);

        let mut bad_bool = SendWaypointsPacket {
            ref_id: 7,
            waypoints: vec![SourceWaypoint {
                mode: SourceWaypointMode::Move,
                ref_id: -1,
                x: 0,
                y: 0,
                attack_to: false,
                player_given: false,
            }],
        }
        .encode_payload();
        bad_bool[21] = 2;
        assert_eq!(SendWaypointsPacket::decode_payload(&bad_bool), None);
        assert_eq!(SendRallypointsPacket::decode_payload(&bad_bool), None);
    }

    #[test]
    fn encodes_object_group_info_packet_layout() {
        let packet = ObjectGroupInfoPacket {
            ref_id: 10,
            leader_ref_id: -1,
            minion_refs: vec![11, 12],
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x14, 0x00, 0x00, 0x00, // payload length
                0x1d, 0x00, 0x00, 0x00, // OBJECT_GROUP_INFO
                0x0a, 0x00, 0x00, 0x00, // ref_id
                0xff, 0xff, 0xff, 0xff, // leader_ref_id
                0x02, 0x00, 0x00, 0x00, // minions
                0x0b, 0x00, 0x00, 0x00, // minion ref
                0x0c, 0x00, 0x00, 0x00, // minion ref
            ]
        );
        assert_eq!(
            ObjectGroupInfoPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(ObjectGroupInfoPacket::decode_payload(&[0; 11]), None);
        assert_eq!(ObjectGroupInfoPacket::decode_payload(&[0; 13]), None);
        let mut bad_count = vec![0; 12];
        bad_count[8..12].copy_from_slice(&(-1_i32).to_le_bytes());
        assert_eq!(ObjectGroupInfoPacket::decode_payload(&bad_count), None);
    }

    #[test]
    fn encodes_update_health_packet_layout() {
        let packet = ObjectHealthPacket {
            ref_id: 7,
            health: 1081,
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x08, 0x00, 0x00, 0x00, // payload length
                0x11, 0x00, 0x00, 0x00, // UPDATE_HEALTH
                0x07, 0x00, 0x00, 0x00, // ref_id
                0x39, 0x04, 0x00, 0x00, // health
            ]
        );
        assert_eq!(
            ObjectHealthPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(ObjectHealthPacket::decode_payload(&[0; 7]), None);
        assert_eq!(ObjectHealthPacket::decode_payload(&[0; 9]), None);
    }

    #[test]
    fn encodes_building_state_packet_layout() {
        let packet = BuildingStatePacket {
            ref_id: 7,
            state: 2,
            init_offset: -1.25,
            production_time: 36.0,
            object_type: 5,
            object_id: 0,
        };

        let encoded = packet.encode_packet();
        assert_eq!(&encoded[0..4], &26_i32.to_le_bytes());
        assert_eq!(&encoded[4..8], &24_i32.to_le_bytes());
        assert_eq!(encoded.len(), 34);
        assert_eq!(
            BuildingStatePacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(BuildingStatePacket::decode_payload(&[0; 25]), None);
        assert_eq!(BuildingStatePacket::decode_payload(&[0; 27]), None);
    }

    #[test]
    fn encodes_building_queue_packet_layout() {
        let packet = BuildingQueuePacket {
            ref_id: 9,
            units: vec![
                BuildingQueueUnit {
                    object_type: 5,
                    object_id: 0,
                },
                BuildingQueueUnit {
                    object_type: 4,
                    object_id: 2,
                },
            ],
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                12, 0, 0, 0, // payload length
                77, 0, 0, 0, // SET_BUILDING_QUEUE_LIST
                9, 0, 0, 0, // ref_id
                2, 0, 0, 0, // amount
                5, 0, 4, 2, // packed production units
            ]
        );
        assert_eq!(
            BuildingQueuePacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(BuildingQueuePacket::decode_payload(&[0; 7]), None);
        let mut bad_count = vec![0; 8];
        bad_count[4..8].copy_from_slice(&(-1_i32).to_le_bytes());
        assert_eq!(BuildingQueuePacket::decode_payload(&bad_count), None);
    }

    #[test]
    fn encodes_delete_object_packet_layout() {
        let packet = DeleteObjectPacket { ref_id: 7 };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x04, 0x00, 0x00, 0x00, // payload length
                0x10, 0x00, 0x00, 0x00, // DELETE_OBJECT
                0x07, 0x00, 0x00, 0x00, // ref_id
            ]
        );
        assert_eq!(
            DeleteObjectPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(DeleteObjectPacket::decode_payload(&[0; 3]), None);
        assert_eq!(DeleteObjectPacket::decode_payload(&[0; 5]), None);
    }

    #[test]
    fn encodes_set_grenade_amount_packet_layout() {
        let packet = ObjectGrenadeAmountPacket {
            ref_id: 7,
            grenade_amount: 3,
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x08, 0x00, 0x00, 0x00, // payload length
                0x42, 0x00, 0x00, 0x00, // SET_GRENADE_AMOUNT
                0x07, 0x00, 0x00, 0x00, // ref_id
                0x03, 0x00, 0x00, 0x00, // grenade_amount
            ]
        );
        assert_eq!(
            ObjectGrenadeAmountPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(
            ObjectGrenadeAmountPacket::decode_payload(&[
                0x07, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
            ]),
            Some(ObjectGrenadeAmountPacket {
                ref_id: 7,
                grenade_amount: -1,
            })
        );
        assert_eq!(ObjectGrenadeAmountPacket::decode_payload(&[0; 7]), None);
        assert_eq!(ObjectGrenadeAmountPacket::decode_payload(&[0; 9]), None);
    }

    #[test]
    fn encodes_pickup_grenade_animation_packet_layout() {
        let packet = PickupGrenadeAnimationPacket { ref_id: 7 };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x04, 0x00, 0x00, 0x00, // payload length
                0x43, 0x00, 0x00, 0x00, // PICKUP_GRENADE_ANIM
                0x07, 0x00, 0x00, 0x00, // ref_id
            ]
        );
        assert_eq!(
            PickupGrenadeAnimationPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(
            PickupGrenadeAnimationPacket::decode_payload(&[0xff, 0xff, 0xff, 0xff]),
            Some(PickupGrenadeAnimationPacket { ref_id: -1 })
        );
        assert_eq!(PickupGrenadeAnimationPacket::decode_payload(&[0; 3]), None);
        assert_eq!(PickupGrenadeAnimationPacket::decode_payload(&[0; 5]), None);
    }

    #[test]
    fn encodes_set_lid_open_packet_layout() {
        let packet = SetLidOpenPacket {
            ref_id: 7,
            lid_open: true,
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x05, 0x00, 0x00, 0x00, // payload length
                0x23, 0x00, 0x00, 0x00, // SET_LID_OPEN
                0x07, 0x00, 0x00, 0x00, // ref_id
                0x01, // lid_open
            ]
        );
        assert_eq!(
            SetLidOpenPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(
            SetLidOpenPacket::decode_payload(&[7, 0, 0, 0, 0]),
            Some(SetLidOpenPacket {
                ref_id: 7,
                lid_open: false,
            })
        );
        assert_eq!(SetLidOpenPacket::decode_payload(&[0; 4]), None);
        assert_eq!(SetLidOpenPacket::decode_payload(&[0; 6]), None);
        assert_eq!(SetLidOpenPacket::decode_payload(&[7, 0, 0, 0, 2]), None);
    }

    #[test]
    fn encodes_eject_vehicle_packet_layout() {
        let packet = EjectVehiclePacket { ref_id: 7 };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x04, 0x00, 0x00, 0x00, // payload length
                0x1e, 0x00, 0x00, 0x00, // EJECT_VEHICLE
                0x07, 0x00, 0x00, 0x00, // ref_id
            ]
        );
        assert_eq!(
            EjectVehiclePacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(EjectVehiclePacket::decode_payload(&[0; 3]), None);
        assert_eq!(EjectVehiclePacket::decode_payload(&[0; 5]), None);
    }

    #[test]
    fn encodes_crane_anim_packet_layout() {
        let packet = CraneAnimPacket {
            ref_id: 7,
            repair_ref_id: 9,
            on: true,
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x09, 0x00, 0x00, 0x00, // payload length
                0x1f, 0x00, 0x00, 0x00, // DO_CRANE_ANIM
                0x07, 0x00, 0x00, 0x00, // ref_id
                0x09, 0x00, 0x00, 0x00, // repair_ref_id
                0x01, // on
            ]
        );
        assert_eq!(
            CraneAnimPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(
            CraneAnimPacket::decode_payload(&[7, 0, 0, 0, 9, 0, 0, 0, 0]),
            Some(CraneAnimPacket {
                ref_id: 7,
                repair_ref_id: 9,
                on: false,
            })
        );
        assert_eq!(CraneAnimPacket::decode_payload(&[0; 8]), None);
        assert_eq!(CraneAnimPacket::decode_payload(&[0; 10]), None);
        assert_eq!(
            CraneAnimPacket::decode_payload(&[7, 0, 0, 0, 9, 0, 0, 0, 2]),
            None
        );
    }

    #[test]
    fn encodes_repair_building_anim_packet_layout() {
        let packet = RepairBuildingAnimPacket {
            ref_id: 7,
            on: true,
            remaining_time: 2.5,
            play_sound: true,
        };
        let mut expected = vec![
            0x0e, 0x00, 0x00, 0x00, // payload length
            0x20, 0x00, 0x00, 0x00, // SET_REPAIR_ANIM
            0x07, 0x00, 0x00, 0x00, // ref_id
            0x01, // on
        ];
        expected.extend_from_slice(&2.5f64.to_le_bytes());
        expected.push(0x01); // play_sound

        assert_eq!(packet.encode_packet(), expected);
        assert_eq!(
            RepairBuildingAnimPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(
            RepairBuildingAnimPacket::decode_payload(&[7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Some(RepairBuildingAnimPacket {
                ref_id: 7,
                on: false,
                remaining_time: 0.0,
                play_sound: false,
            })
        );
        assert_eq!(RepairBuildingAnimPacket::decode_payload(&[0; 13]), None);
        assert_eq!(RepairBuildingAnimPacket::decode_payload(&[0; 15]), None);
        let mut invalid_bool = packet.encode_payload();
        invalid_bool[4] = 2;
        assert_eq!(
            RepairBuildingAnimPacket::decode_payload(&invalid_bool),
            None
        );
        let mut invalid_sound_bool = packet.encode_payload();
        invalid_sound_bool[13] = 2;
        assert_eq!(
            RepairBuildingAnimPacket::decode_payload(&invalid_sound_bool),
            None
        );
    }

    #[test]
    fn encodes_destroy_object_packet_layout() {
        let packet = DestroyObjectPacket {
            ref_id: 7,
            killer_ref_id: -1,
            destroy_object: true,
            do_fire_death: false,
            do_missile_death: true,
            fire_missiles: Vec::new(),
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x0f, 0x00, 0x00, 0x00, // payload length
                0x15, 0x00, 0x00, 0x00, // DESTROY_OBJECT
                0x07, 0x00, 0x00, 0x00, // ref_id
                0x00, 0x00, 0x00, 0x00, // fire_missile_amount
                0xff, 0xff, 0xff, 0xff, // killer_ref_id
                0x01, // destroy_object
                0x00, // do_fire_death
                0x01, // do_missile_death
            ]
        );
        assert_eq!(
            DestroyObjectPacket::decode_payload(&packet.encode_payload()),
            Some(packet.clone())
        );
        assert_eq!(DestroyObjectPacket::decode_payload(&[0; 14]), None);
        assert_eq!(DestroyObjectPacket::decode_payload(&[0; 16]), None);

        let mut bad_bool = packet.encode_payload();
        bad_bool[12] = 2;
        assert_eq!(DestroyObjectPacket::decode_payload(&bad_bool), None);
    }

    #[test]
    fn decodes_destroy_object_packet_with_turrent_missiles() {
        let packet = DestroyObjectPacket {
            ref_id: 8,
            killer_ref_id: 3,
            destroy_object: false,
            do_fire_death: true,
            do_missile_death: false,
            fire_missiles: vec![DestroyObjectMissileInfo {
                missile_offset_time: 3.25,
                missile_x: 44,
                missile_y: -12,
            }],
        };

        assert_eq!(
            DestroyObjectPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
    }

    #[test]
    fn encodes_snipe_object_packet_layout() {
        let packet = SnipeObjectPacket { ref_id: 7 };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x04, 0x00, 0x00, 0x00, // payload length
                0x24, 0x00, 0x00, 0x00, // SNIPE_OBJECT
                0x07, 0x00, 0x00, 0x00, // ref_id
            ]
        );
        assert_eq!(
            SnipeObjectPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(SnipeObjectPacket::decode_payload(&[0; 3]), None);
        assert_eq!(SnipeObjectPacket::decode_payload(&[0; 5]), None);
    }

    #[test]
    fn encodes_driver_hit_effect_packet_layout() {
        let packet = DriverHitEffectPacket { ref_id: 7 };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x04, 0x00, 0x00, 0x00, // payload length
                0x25, 0x00, 0x00, 0x00, // DRIVER_HIT_EFFECT
                0x07, 0x00, 0x00, 0x00, // ref_id
            ]
        );
        assert_eq!(
            DriverHitEffectPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(DriverHitEffectPacket::decode_payload(&[0; 3]), None);
        assert_eq!(DriverHitEffectPacket::decode_payload(&[0; 5]), None);
    }

    #[test]
    fn encodes_computer_message_packet_layout() {
        let packet = ComputerMessagePacket {
            ref_id: 7,
            sound: 19,
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x08, 0x00, 0x00, 0x00, // payload length
                0x1c, 0x00, 0x00, 0x00, // COMP_MSG
                0x07, 0x00, 0x00, 0x00, // ref_id
                0x13, 0x00, 0x00, 0x00, // COMP_VEHICLE_SND
            ]
        );
        assert_eq!(
            ComputerMessagePacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(ComputerMessagePacket::decode_payload(&[0; 7]), None);
        assert_eq!(ComputerMessagePacket::decode_payload(&[0; 9]), None);
    }

    #[test]
    fn encodes_do_portrait_anim_packet_layout() {
        let packet = DoPortraitAnimPacket {
            ref_id: 7,
            anim_id: 61,
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x08, 0x00, 0x00, 0x00, // payload length
                0x44, 0x00, 0x00, 0x00, // DO_PORTRAIT_ANIM
                0x07, 0x00, 0x00, 0x00, // ref_id
                0x3d, 0x00, 0x00, 0x00, // VEHICLE_CAPTURED_ANIM
            ]
        );
        assert_eq!(
            DoPortraitAnimPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(DoPortraitAnimPacket::decode_payload(&[0; 7]), None);
        assert_eq!(DoPortraitAnimPacket::decode_payload(&[0; 9]), None);
    }

    #[test]
    fn encodes_selectable_map_list_packet_like_source_server() {
        let packet =
            GiveSelectableMapListPacket::new(["alpha.map", "beta.map", "gamma.map"]).unwrap();

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x1d, 0x00, 0x00, 0x00, // payload length
                0x3d, 0x00, 0x00, 0x00, // GIVE_SELECTABLE_MAP_LIST
                b'a', b'l', b'p', b'h', b'a', b'.', b'm', b'a', b'p', b',', b'b', b'e', b't', b'a',
                b'.', b'm', b'a', b'p', b',', b'g', b'a', b'm', b'm', b'a', b'.', b'm', b'a', b'p',
                0x00,
            ]
        );
        assert_eq!(
            GiveSelectableMapListPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(
            GiveSelectableMapListPacket { maps: Vec::new() }.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x3d, 0x00, 0x00, 0x00, // GIVE_SELECTABLE_MAP_LIST
            ]
        );
        assert_eq!(
            GiveSelectableMapListPacket::decode_payload(&[]),
            Some(GiveSelectableMapListPacket { maps: Vec::new() })
        );
        assert_eq!(
            GiveSelectableMapListPacket::decode_payload(&[0]),
            Some(GiveSelectableMapListPacket { maps: Vec::new() })
        );
    }

    #[test]
    fn decodes_selectable_map_list_like_source_split_loop() {
        assert_eq!(
            GiveSelectableMapListPacket::decode_payload(b",a,,b,\0"),
            Some(GiveSelectableMapListPacket {
                maps: vec![
                    String::new(),
                    "a".to_string(),
                    String::new(),
                    "b".to_string(),
                ]
            })
        );
        assert_eq!(GiveSelectableMapListPacket::new(["bad,map"]), None);
        assert_eq!(GiveSelectableMapListPacket::new(["bad\0map"]), None);
        assert_eq!(GiveSelectableMapListPacket::decode_payload(b"a,b"), None);
        assert_eq!(
            GiveSelectableMapListPacket::decode_payload(b"a\0,b\0"),
            None
        );

        let long = vec![b'x'; 520];
        let mut payload = long;
        payload.push(0);
        assert_eq!(
            GiveSelectableMapListPacket::decode_payload(&payload)
                .unwrap()
                .maps[0]
                .len(),
            499
        );
    }

    #[test]
    fn encodes_player_id_clear_and_add_player_packets() {
        let player_id = PlayerIdPacket { player_id: 7 };
        let add = AddLocalPlayerPacket { player_id: 7 };
        let delete = DeleteLocalPlayerPacket { player_id: 7 };

        assert_eq!(
            player_id.encode_packet(),
            vec![
                0x04, 0x00, 0x00, 0x00, // payload length
                0x3a, 0x00, 0x00, 0x00, // GIVE_PLAYER_ID
                0x07, 0x00, 0x00, 0x00, // p_id
            ]
        );
        assert_eq!(
            ClearPlayerListPacket.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x28, 0x00, 0x00, 0x00, // CLEAR_PLAYER_LIST
            ]
        );
        assert_eq!(
            add.encode_packet(),
            vec![
                0x04, 0x00, 0x00, 0x00, // payload length
                0x29, 0x00, 0x00, 0x00, // ADD_LPLAYER
                0x07, 0x00, 0x00, 0x00, // p_id
            ]
        );
        assert_eq!(
            delete.encode_packet(),
            vec![
                0x04, 0x00, 0x00, 0x00, // payload length
                0x2a, 0x00, 0x00, 0x00, // DELETE_LPLAYER
                0x07, 0x00, 0x00, 0x00, // p_id
            ]
        );
        assert_eq!(
            PlayerIdPacket::decode_payload(&player_id.encode_payload()),
            Some(player_id)
        );
        assert_eq!(
            ClearPlayerListPacket::decode_payload(&ClearPlayerListPacket.encode_payload()),
            Some(ClearPlayerListPacket)
        );
        assert_eq!(
            AddLocalPlayerPacket::decode_payload(&add.encode_payload()),
            Some(add)
        );
        assert_eq!(
            DeleteLocalPlayerPacket::decode_payload(&delete.encode_payload()),
            Some(delete)
        );
        assert_eq!(PlayerIdPacket::decode_payload(&[]), None);
        assert_eq!(ClearPlayerListPacket::decode_payload(&[0]), None);
        assert_eq!(AddLocalPlayerPacket::decode_payload(&[0; 3]), None);
        assert_eq!(DeleteLocalPlayerPacket::decode_payload(&[0; 3]), None);
    }

    #[test]
    fn encodes_set_local_player_ignored_and_loginfo_packets() {
        let ignored = SetLocalPlayerIgnoredPacket {
            player_id: 7,
            ignored: 1,
        };
        let loginfo = SetLocalPlayerLogInfoPacket {
            player_id: 7,
            db_id: -1,
            voting_power: 3,
            total_games: 10,
            activated: true,
            logged_in: true,
            bot_logged_in: false,
        };

        assert_eq!(
            ignored.encode_packet(),
            vec![
                0x08, 0x00, 0x00, 0x00, // payload length
                0x2e, 0x00, 0x00, 0x00, // SET_LPLAYER_IGNORED
                0x07, 0x00, 0x00, 0x00, // p_id
                0x01, 0x00, 0x00, 0x00, // ignored
            ]
        );
        assert_eq!(
            loginfo.encode_packet(),
            vec![
                0x13, 0x00, 0x00, 0x00, // payload length
                0x2f, 0x00, 0x00, 0x00, // SET_LPLAYER_LOGINFO
                0x07, 0x00, 0x00, 0x00, // p_id
                0xff, 0xff, 0xff, 0xff, // db_id
                0x03, 0x00, 0x00, 0x00, // voting_power
                0x0a, 0x00, 0x00, 0x00, // total_games
                0x01, 0x01, 0x00, // activated, logged_in, bot_logged_in
            ]
        );
        assert_eq!(
            SetLocalPlayerIgnoredPacket::decode_payload(&ignored.encode_payload()),
            Some(ignored)
        );
        assert_eq!(
            SetLocalPlayerLogInfoPacket::decode_payload(&loginfo.encode_payload()),
            Some(loginfo)
        );
        assert_eq!(SetLocalPlayerIgnoredPacket::decode_payload(&[0; 7]), None);
        assert_eq!(SetLocalPlayerLogInfoPacket::decode_payload(&[0; 18]), None);
        let mut bad_bool = loginfo.encode_payload();
        bad_bool[16] = 2;
        assert_eq!(SetLocalPlayerLogInfoPacket::decode_payload(&bad_bool), None);
    }

    #[test]
    fn encodes_set_game_paused_packet_layout() {
        let pause = SetGamePausedCommand { game_paused: true };
        let resume = SetGamePausedCommand { game_paused: false };

        assert_eq!(
            pause.encode_packet(),
            vec![
                0x01, 0x00, 0x00, 0x00, // payload length
                0x34, 0x00, 0x00, 0x00, // SET_GAME_PAUSED
                0x01, // game_paused
            ]
        );
        assert_eq!(
            resume.encode_packet(),
            vec![
                0x01, 0x00, 0x00, 0x00, // payload length
                0x34, 0x00, 0x00, 0x00, // SET_GAME_PAUSED
                0x00, // game_paused
            ]
        );
    }

    #[test]
    fn decodes_update_game_paused_payload_layout() {
        assert_eq!(
            SetGamePausedCommand::decode_payload(&[1]),
            Some(SetGamePausedCommand { game_paused: true })
        );
        assert_eq!(
            SetGamePausedCommand::decode_payload(&[0]),
            Some(SetGamePausedCommand { game_paused: false })
        );
        assert_eq!(
            UpdateGamePausedPacket::decode_payload(&[1]),
            Some(UpdateGamePausedPacket { game_paused: true })
        );
        assert_eq!(
            UpdateGamePausedPacket::decode_payload(&[0]),
            Some(UpdateGamePausedPacket { game_paused: false })
        );
        assert_eq!(UpdateGamePausedPacket::decode_payload(&[]), None);
        assert_eq!(UpdateGamePausedPacket::decode_payload(&[1, 0]), None);
        assert_eq!(UpdateGamePausedPacket::decode_payload(&[2]), None);
        assert_eq!(
            UpdateGamePausedPacket { game_paused: true }.encode_payload(),
            vec![1]
        );
    }

    #[test]
    fn encodes_news_event_packet_layout_with_source_string_payload() {
        let packet = NewsEventPacket::new("ok", 0, 4, 5).unwrap();

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x06, 0x00, 0x00, 0x00, // payload length
                0x0a, 0x00, 0x00, 0x00, // NEWS_EVENT
                0x00, 0x04, 0x05, // r, g, b
                b'o', b'k', 0x00, // nul-terminated message
            ]
        );
        assert_eq!(
            NewsEventPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
    }

    #[test]
    fn decodes_news_event_payload_like_source_client_guard() {
        assert_eq!(NewsEventPacket::new("", 1, 2, 3), None);
        assert_eq!(NewsEventPacket::new("bad\0message", 1, 2, 3), None);
        assert_eq!(NewsEventPacket::decode_payload(&[1, 2, 3, b'x', 0]), None);
        assert_eq!(
            NewsEventPacket::decode_payload(&[1, 2, 3, b'o', b'k']),
            None
        );
        assert_eq!(
            NewsEventPacket::decode_payload(&[1, 2, 3, b'o', 0, b'x']),
            None
        );
    }

    #[test]
    fn encodes_send_chat_ascii_packet_layout() {
        let command = SendChatCommand::new("hello").unwrap();

        assert_eq!(
            command.encode_packet(),
            vec![
                0x06, 0x00, 0x00, 0x00, // payload length
                0x1b, 0x00, 0x00, 0x00, // SEND_CHAT
                b'h', b'e', b'l', b'l', b'o', 0x00,
            ]
        );
        assert_eq!(
            SendChatCommand::decode_payload(&command.encode_payload()),
            Some(command)
        );
    }

    #[test]
    fn decodes_send_chat_like_source_server_guard() {
        assert_eq!(SendChatCommand::new(""), None);
        assert_eq!(SendChatCommand::new("bad\0message"), None);
        assert_eq!(SendChatCommand::decode_payload(&[]), None);
        assert_eq!(SendChatCommand::decode_payload(&[0]), None);
        assert_eq!(SendChatCommand::decode_payload(&[b'o', b'k']), None);
        assert_eq!(SendChatCommand::decode_payload(&[b'o', 0, b'x', 0]), None);
    }

    #[test]
    fn encodes_give_version_packet_with_source_fixed_char_array() {
        let packet = GiveVersionPacket::source_current();
        let wire_packet = packet.encode_packet();

        assert_eq!(&wire_packet[..8], &[0x32, 0, 0, 0, 0x55, 0, 0, 0]);
        assert_eq!(&wire_packet[8..18], b"2011-09-06");
        assert_eq!(wire_packet[18], 0);
        assert!(wire_packet[19..].iter().all(|byte| *byte == 0));
        assert_eq!(
            GiveVersionPacket::decode_payload(&wire_packet[8..]),
            Some(packet)
        );
    }

    #[test]
    fn decodes_give_version_like_source_client_guard() {
        assert_eq!(GiveVersionPacket::decode_payload(&[0; 49]), None);
        assert_eq!(GiveVersionPacket::decode_payload(&[0; 51]), None);
        assert_eq!(GiveVersionPacket::new("x".repeat(49)), None);
        assert_eq!(GiveVersionPacket::new("bad\0version"), None);

        let mut no_nul = [b'x'; 50];
        assert_eq!(
            GiveVersionPacket::decode_payload(&no_nul),
            Some(GiveVersionPacket {
                version: "x".repeat(49)
            })
        );
        no_nul[3] = 0;
        assert_eq!(
            GiveVersionPacket::decode_payload(&no_nul),
            Some(GiveVersionPacket {
                version: "xxx".to_string()
            })
        );
    }

    #[test]
    fn encodes_update_game_speed_packet_with_source_float_layout() {
        let packet = UpdateGameSpeedPacket { game_speed: 1.5 };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x04, 0x00, 0x00, 0x00, // payload length
                0x4b, 0x00, 0x00, 0x00, // UPDATE_GAME_SPEED
                0x00, 0x00, 0xc0, 0x3f, // f32 1.5
            ]
        );
        assert_eq!(
            UpdateGameSpeedPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(UpdateGameSpeedPacket::decode_payload(&[]), None);
        assert_eq!(UpdateGameSpeedPacket::decode_payload(&[0; 3]), None);
        assert_eq!(UpdateGameSpeedPacket::decode_payload(&[0; 5]), None);
    }

    #[test]
    fn encodes_set_game_speed_command_with_source_float_layout() {
        let command = SetGameSpeedCommand { game_speed: 0.5 };

        assert_eq!(
            command.encode_packet(),
            vec![
                0x04, 0x00, 0x00, 0x00, // payload length
                0x4a, 0x00, 0x00, 0x00, // SET_GAME_SPEED
                0x00, 0x00, 0x00, 0x3f, // f32 0.5
            ]
        );
        assert_eq!(
            SetGameSpeedCommand::decode_payload(&command.encode_payload()),
            Some(command)
        );
        assert_eq!(SetGameSpeedCommand::decode_payload(&[]), None);
        assert_eq!(SetGameSpeedCommand::decode_payload(&[0; 3]), None);
        assert_eq!(SetGameSpeedCommand::decode_payload(&[0; 5]), None);
    }

    #[test]
    fn encodes_vote_info_packet_with_source_packed_layout() {
        let packet = VoteInfoPacket {
            in_progress: true,
            vote_type: 0,
            value: -1,
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x09, 0x00, 0x00, 0x00, // payload length
                0x39, 0x00, 0x00, 0x00, // VOTE_INFO
                0x01, // in_progress
                0x00, 0x00, 0x00, 0x00, // PAUSE_VOTE
                0xff, 0xff, 0xff, 0xff, // value
            ]
        );
        assert_eq!(
            VoteInfoPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(VoteInfoPacket::decode_payload(&[1, 0, 0, 0]), None);
        assert_eq!(
            VoteInfoPacket::decode_payload(&[2, 0, 0, 0, 0, 0, 0, 0, 0]),
            None
        );
    }

    #[test]
    fn encodes_set_local_player_vote_info_packet_layout() {
        let packet = SetLocalPlayerVoteInfoPacket {
            player_id: 7,
            vote_choice: 1,
        };

        assert_eq!(
            packet.encode_packet(),
            vec![
                0x08, 0x00, 0x00, 0x00, // payload length
                0x30, 0x00, 0x00, 0x00, // SET_LPLAYER_VOTEINFO
                0x07, 0x00, 0x00, 0x00, // p_id
                0x01, 0x00, 0x00, 0x00, // P_YES_VOTE
            ]
        );
        assert_eq!(
            SetLocalPlayerVoteInfoPacket::decode_payload(&packet.encode_payload()),
            Some(packet)
        );
        assert_eq!(SetLocalPlayerVoteInfoPacket::decode_payload(&[]), None);
        assert_eq!(SetLocalPlayerVoteInfoPacket::decode_payload(&[0; 9]), None);
    }

    #[test]
    fn encodes_vote_choice_commands_as_empty_payloads() {
        assert_eq!(
            VoteYesCommand.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x36, 0x00, 0x00, 0x00, // VOTE_YES
            ]
        );
        assert_eq!(
            VoteNoCommand.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x37, 0x00, 0x00, 0x00, // VOTE_NO
            ]
        );
        assert_eq!(
            VotePassCommand.encode_packet(),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x38, 0x00, 0x00, 0x00, // VOTE_PASS
            ]
        );
    }

    #[test]
    fn encodes_place_cannon_packet_layout() {
        let command =
            PlaceCannonCommand::new(7, 12, 34, ObjectKind::Cannon(CannonType::Howitzer)).unwrap();

        assert_eq!(
            command.encode_packet(),
            vec![
                0x0d, 0x00, 0x00, 0x00, // payload length
                0x1a, 0x00, 0x00, 0x00, // PLACE_CANNON
                0x07, 0x00, 0x00, 0x00, // ref_id
                0x0c, 0x00, 0x00, 0x00, // tx
                0x22, 0x00, 0x00, 0x00, // ty
                0x02, // oid
            ]
        );
    }

    #[test]
    fn encodes_add_building_queue_packet_layout() {
        let command =
            AddBuildingQueueCommand::new(9, ObjectKind::Robot(RobotType::Psycho)).unwrap();

        assert_eq!(
            command.encode_packet(),
            vec![
                0x06, 0x00, 0x00, 0x00, // payload length
                0x4c, 0x00, 0x00, 0x00, // ADD_BUILDING_QUEUE
                0x09, 0x00, 0x00, 0x00, // ref_id
                0x05, 0x01, // object type, object id
            ]
        );
    }

    #[test]
    fn encodes_cancel_building_queue_packet_layout() {
        let command =
            CancelBuildingQueueCommand::new(11, 3, ObjectKind::Vehicle(VehicleType::Light))
                .unwrap();

        assert_eq!(
            command.encode_packet(),
            vec![
                0x0a, 0x00, 0x00, 0x00, // payload length
                0x4e, 0x00, 0x00, 0x00, // CANCEL_BUILDING_QUEUE
                0x0b, 0x00, 0x00, 0x00, // ref_id
                0x03, 0x00, 0x00, 0x00, // list_i
                0x04, 0x01, // object type, object id
            ]
        );
    }

    #[test]
    fn encode_packet_keeps_empty_payload_header_only() {
        assert_eq!(
            encode_packet(TcpEventId::StopBuilding, &[]),
            vec![
                0x00, 0x00, 0x00, 0x00, // payload length
                0x17, 0x00, 0x00, 0x00, // STOP_BUILDING
            ]
        );
    }

    #[test]
    fn place_cannon_constructor_rejects_non_cannon_kind() {
        assert_eq!(
            PlaceCannonCommand::new(1, 2, 3, ObjectKind::Robot(RobotType::Grunt)),
            None
        );
    }
}
