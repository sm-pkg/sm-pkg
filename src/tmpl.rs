use askama::Template;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Write, path};

type ConfigValues = HashMap<String, String>;

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

#[derive(Template)]
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
