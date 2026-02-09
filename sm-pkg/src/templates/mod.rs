mod intbool;
mod onoffbool;
mod yesnobool;

use askama::Template;
use serde::{Deserialize, Serialize};

use intbool::*;
use onoffbool::*;
use yesnobool::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Format {
    CFG,
    KV,
    TEMPLATE,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ReserveType {
    Public = 0,
    DropHighLatency = 1,
    DropHighLatencyLimited = 2,
}

impl std::fmt::Display for ReserveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReserveType::Public => write!(f, "0"),
            ReserveType::DropHighLatency => write!(f, "1"),
            ReserveType::DropHighLatencyLimited => write!(f, "2"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ImmunityMode {
    Ignore = 0,
    ProtectLowerAccessOnly = 1,
    ProtectEqualOrLowerAccess = 2,
    ProtectEqualOrLowerAccessNoAdminImmunity = 3,
}

impl std::fmt::Display for ImmunityMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImmunityMode::Ignore => write!(f, "0"),
            ImmunityMode::ProtectLowerAccessOnly => write!(f, "1"),
            ImmunityMode::ProtectEqualOrLowerAccess => write!(f, "2"),
            ImmunityMode::ProtectEqualOrLowerAccessNoAdminImmunity => write!(f, "3"),
        }
    }
}

#[derive(Template, Serialize, Deserialize, Debug)]
#[template(
    path = "cfg/sourcemod/sourcemod.cfg.jinja2",
    ext = "txt",
    whitespace = "suppress"
)]
pub struct SourcemodCfg {
    pub sm_show_activity: Option<u8>,
    pub sm_menu_sounds: Option<IntBool>,
    pub sm_vote_delay: Option<u32>,
    pub sm_datetime_format: Option<String>,
    pub sm_immunity_mode: Option<ImmunityMode>,
    pub sm_time_adjustment: Option<u32>,
    pub sm_flood_time: Option<f32>,
    pub sm_reserve_type: Option<ReserveType>,
    pub sm_reserved_slots: Option<u8>,
    pub sm_hide_slots: Option<IntBool>,
    pub sm_chat_mode: Option<IntBool>,
    pub sm_timeleft_interval: Option<u32>,
    pub sm_trigger_show: Option<IntBool>,
    pub sm_vote_progress_hintbox: Option<IntBool>,
    pub sm_vote_progress_chat: Option<IntBool>,
    pub sm_vote_progress_console: Option<IntBool>,
    pub sm_vote_progress_client_console: Option<IntBool>,
}

#[derive(Template, Serialize, Deserialize, Debug)]
#[template(
    path = "addons/sourcemod/configs/core.cfg.jinja2",
    ext = "txt",
    whitespace = "suppress"
)]
pub struct CoreCfg {
    pub logging: Option<OnOffBool>,
    pub log_mode: Option<String>,
    pub log_time_format: Option<String>,
    pub server_lang: Option<String>,
    pub public_chat_trigger: Option<String>,
    pub silent_chat_trigger: Option<String>,
    pub silent_fail_suppress: Option<YesNoBool>,
    pub pass_info_var: Option<String>,
    pub allow_cl_language_var: Option<OnOffBool>,
    pub disable_auto_update: Option<YesNoBool>,
    pub force_restart_after_update: Option<YesNoBool>,
    pub auto_update_url: Option<String>,
    pub debug_spew: Option<YesNoBool>,
    pub steam_authstring_validation: Option<YesNoBool>,
    pub block_bad_plugins: Option<YesNoBool>,
    pub slow_script_timeout: Option<u32>,
    pub follow_csgo_server_guidelines: Option<YesNoBool>,
    pub jit_metadata: Option<String>,
    pub enable_line_debugging: Option<YesNoBool>,
}

#[derive(Serialize, Deserialize)]
pub struct SourcemodDatabase {
    pub name: String,
    pub database: String,
    pub driver: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub pass: Option<String>,
    pub timeout: Option<u32>,
}

#[derive(Template, Serialize, Deserialize)]
#[template(
    path = "addons/sourcemod/configs/databases.cfg.jinja2",
    ext = "txt",
    whitespace = "suppress"
)]
pub struct DatabasesCfg {
    pub driver_default: Option<String>,
    pub databases: Option<Vec<SourcemodDatabase>>,
}

#[derive(Template, Serialize, Deserialize)]
#[template(
    path = "addons/sourcemod/configs/maplists.cfg.jinja2",
    ext = "txt",
    whitespace = "suppress"
)]
pub struct MaplistsCfg {
    pub default_target: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SourcemodSimpleAdmin {
    pub identity: String,
    pub password: Option<String>,
    pub flags: String,
    pub immunity: Option<u8>,
}

#[derive(Template, Serialize, Deserialize)]
#[template(
    path = "addons/sourcemod/configs/admins_simple.ini.jinja2",
    ext = "txt",
    whitespace = "suppress"
)]
pub struct AdminsSimpleIni {
    pub users: Option<Vec<SourcemodSimpleAdmin>>,
}

#[derive(Serialize, Deserialize)]
pub struct SourcemodAdmin {
    pub name: String,
    pub auth: String,
    pub identity: String,
    pub password: Option<String>,
    pub group: Option<String>,
    pub flags: Option<String>,
    pub immunity: Option<u8>,
}

#[derive(Template, Serialize, Deserialize)]
#[template(
    path = "addons/sourcemod/configs/admins.cfg.jinja2",
    ext = "txt",
    whitespace = "suppress"
)]
pub struct AdminsCfg {
    pub users: Option<Vec<SourcemodAdmin>>,
}

#[derive(Serialize, Deserialize)]
pub struct Override {
    pub command: String,
    pub action: String,
}

#[derive(Serialize, Deserialize)]
pub struct AdminGroup {
    pub name: String,
    pub flags: Option<String>,
    pub immunity: Option<u8>,
    pub overrides: Option<Vec<Override>>,
}

#[derive(Template, Serialize, Deserialize)]
#[template(
    path = "addons/sourcemod/configs/admin_groups.cfg.jinja2",
    ext = "txt",
    whitespace = "suppress"
)]
pub struct AdminGroupsCfg {
    pub default_immunity: Option<String>,
    pub groups: Option<Vec<AdminGroup>>,
}

#[derive(Serialize, Deserialize)]
pub struct AdminOverride {
    pub command: String,
    pub flags: String,
}

#[derive(Template, Serialize, Deserialize)]
#[template(
    path = "addons/sourcemod/configs/admin_overrides.cfg.jinja2",
    ext = "txt",
    whitespace = "suppress"
)]
pub struct AdminOverridesCfg {
    pub overrides: Option<Vec<AdminOverride>>,
}

#[derive(Template, Serialize, Deserialize)]
#[template(path = "start.sh.jinja2", ext = "txt")]
pub struct StartSh {
    pub use_64bit: Option<bool>,
    pub mod_folder: String,
    pub port: Option<u16>,
    pub tv_port: Option<u16>,
    pub client_port: Option<u16>,
    pub max_players: Option<u8>,
    pub unrestricted_max_players: Option<bool>,
    pub map: Option<String>,
    pub gslt: Option<String>,
    pub rcon_password: Option<String>,
    pub sv_password: Option<String>,
    pub region: Option<u8>,
    pub ip: Option<String>,
    pub sdr_enable: Option<bool>,
    pub workshop_authkey: Option<String>,
}
