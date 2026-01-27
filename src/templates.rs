use askama::Template;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Write, path};

type ConfigValues = HashMap<String, String>;

#[derive(Serialize, Deserialize)]
pub enum Toggle {
    Off = 0,
    On = 1,
}

impl std::fmt::Display for Toggle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Toggle::Off => write!(f, "0"),
            Toggle::On => write!(f, "1"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Format {
    CFG,
    KV,
    TEMPLATE,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileConfig {
    pub format: Format,
    pub path: path::PathBuf,
    pub raw: Option<String>,
    pub options: Option<HashMap<String, String>>,
}

pub fn render_cfg(mut writer: impl Write, values: ConfigValues) {
    for (key, value) in values {
        writeln!(writer, "{} = {}", key, value).unwrap();
    }
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Template, Serialize, Deserialize)]
#[template(path = "tf/cfg/sourcemod/sourcemod.cfg.jinja2", ext = "txt")]
pub struct SourcemodCfg {
    pub sm_show_activity: u8,
    pub sm_menu_sounds: u8,
    pub sm_vote_delay: u32,
    pub sm_datetime_format: String,
    pub sm_immunity_mode: ImmunityMode,
    pub sm_time_adjustment: u32,
    pub sm_flood_time: f32,
    pub sm_reserve_type: ReserveType,
    pub sm_reserved_slots: u8,
    pub sm_hide_slots: Toggle,
    pub sm_chat_mode: Toggle,
    pub sm_timeleft_interval: u32,
    pub sm_trigger_show: Toggle,
    pub sm_vote_progress_hintbox: Toggle,
    pub sm_vote_progress_chat: Toggle,
    pub sm_vote_progress_console: Toggle,
    pub sm_vote_progress_client_console: Toggle,
}

impl Default for SourcemodCfg {
    fn default() -> Self {
        Self {
            sm_show_activity: 13,
            sm_menu_sounds: 1,
            sm_vote_delay: 30,
            sm_datetime_format: "%m/%d/%Y - %H:%M:%S".to_string(),
            sm_immunity_mode: ImmunityMode::ProtectLowerAccessOnly,
            sm_time_adjustment: 0,
            sm_flood_time: 0.75,
            sm_reserve_type: ReserveType::Public,
            sm_reserved_slots: 0,
            sm_hide_slots: Toggle::Off,
            sm_chat_mode: Toggle::Off,
            sm_timeleft_interval: 0,
            sm_trigger_show: Toggle::Off,
            sm_vote_progress_hintbox: Toggle::Off,
            sm_vote_progress_chat: Toggle::Off,
            sm_vote_progress_console: Toggle::Off,
            sm_vote_progress_client_console: Toggle::Off,
        }
    }
}

#[derive(Template)]
#[template(path = "tf/addons/sourcemod/configs/core.cfg.jinja2", ext = "txt")]
pub struct CoreCfg {
    pub core_logging: Option<String>,
    pub core_log_mode: Option<String>,
    pub core_log_time_format: Option<String>,
    pub core_server_lang: Option<String>,
    pub core_public_chat_trigger: Option<String>,
    pub core_silent_chat_trigger: Option<String>,
    pub core_silent_fail_suppress: Option<String>,
    pub core_pass_info_var: Option<String>,
    pub core_allow_cl_language_var: Option<String>,
    pub core_disable_auto_update: Option<String>,
    pub core_force_restart_after_update: Option<String>,
    pub core_auto_update_url: Option<String>,
    pub core_debug_spew: Option<String>,
    pub core_steam_authstring_validation: Option<String>,
    pub core_block_bad_plugins: Option<String>,
    pub core_slow_script_timeout: Option<String>,
    pub core_follow_csgo_server_guidelines: Option<String>,
    pub core_jit_metadata: Option<String>,
    pub core_enable_line_debugging: Option<String>,
}

impl Default for CoreCfg {
    fn default() -> Self {
        Self {
            core_logging: Some(String::from("on")),
            core_log_mode: Some(String::from("daily")),
            core_log_time_format: Some(String::from("default")),
            core_server_lang: Some(String::from("en")),
            core_public_chat_trigger: Some(String::from("!")),
            core_silent_chat_trigger: Some(String::from("/")),
            core_silent_fail_suppress: Some(String::from("no")),
            core_pass_info_var: Some(String::from("_password")),
            core_allow_cl_language_var: Some(String::from("on")),
            core_disable_auto_update: Some(String::from("no")),
            core_force_restart_after_update: Some(String::from("no")),
            core_auto_update_url: Some(String::from("http://update.sourcemod.net/update/")),
            core_debug_spew: Some(String::from("no")),
            core_steam_authstring_validation: Some(String::from("yes")),
            core_block_bad_plugins: Some(String::from("yes")),
            core_slow_script_timeout: Some(String::from("8")),
            core_follow_csgo_server_guidelines: Some(String::from("yes")),
            core_jit_metadata: Some(String::from("default")),
            core_enable_line_debugging: Some(String::from("no")),
        }
    }
}

pub struct SourcemodDatabase {
    pub name: String,
    pub database: String,
    pub driver: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub pass: Option<String>,
    pub timeout: Option<String>,
}

#[derive(Template)]
#[template(path = "tf/addons/sourcemod/configs/databases.cfg.jinja2", ext = "txt")]
pub struct DatabasesCfg {
    pub driver_default: Option<String>,
    pub databases: Option<Vec<SourcemodDatabase>>,
}

impl Default for DatabasesCfg {
    fn default() -> Self {
        Self {
            driver_default: None,
            databases: None,
        }
    }
}

#[derive(Template)]
#[template(path = "tf/addons/sourcemod/configs/maplists.cfg.jinja2", ext = "txt")]
pub struct MaplistsCfg {
    pub default_target: Option<String>,
}

impl Default for MaplistsCfg {
    fn default() -> Self {
        Self {
            default_target: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SourcemodSimpleAdmin {
    pub identity: String,
    pub password: Option<String>,
    pub flags: String,
    pub immunity: Option<String>,
}

#[derive(Template)]
#[template(
    path = "tf/addons/sourcemod/configs/admins_simple.ini.jinja2",
    ext = "txt"
)]
pub struct AdminsSimpleIni {
    pub users: Option<Vec<SourcemodSimpleAdmin>>,
}

impl Default for AdminsSimpleIni {
    fn default() -> Self {
        Self { users: None }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SourcemodAdmin {
    pub auth: String,
    pub identity: String,
    pub password: Option<String>,
    pub group: Option<String>,
    pub flags: Option<String>,
    pub immunity: Option<String>,
}

#[derive(Template, Serialize, Deserialize)]
#[template(
    path = "tf/addons/sourcemod/configs/admins.cfg.jinja2",
    ext = "txt",
    escape = "none"
)]
pub struct AdminsCfg {
    pub users: Option<Vec<SourcemodAdmin>>,
}

impl Default for AdminsCfg {
    fn default() -> Self {
        Self { users: None }
    }
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
    pub immunity: Option<String>,
    pub overrides: Option<Vec<Override>>,
}

#[derive(Template)]
#[template(
    path = "tf/addons/sourcemod/configs/admin_groups.cfg.jinja2",
    ext = "txt"
)]
pub struct AdminGroupsCfg {
    pub default_immunity: Option<String>,
    pub groups: Option<Vec<AdminGroup>>,
}

impl Default for AdminGroupsCfg {
    fn default() -> Self {
        Self {
            default_immunity: None,
            groups: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AdminOverride {
    pub command: String,
    pub flags: String,
}

#[derive(Template)]
#[template(
    path = "tf/addons/sourcemod/configs/admin_overrides.cfg.jinja2",
    ext = "txt"
)]
pub struct AdminOverridesCfg {
    pub overrides: Option<Vec<AdminOverride>>,
}

impl Default for AdminOverridesCfg {
    fn default() -> Self {
        Self { overrides: None }
    }
}
