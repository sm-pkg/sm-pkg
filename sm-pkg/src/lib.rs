#[macro_use]
extern crate log;

pub mod config;
pub mod fsutil;
pub mod plugins;
pub mod project;
pub mod repo;
pub mod sdk;
pub mod templates;
const CONFIG_FILE_PATH: &str = "~/.sm-pkg/config.yaml";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_ROOT: &str = "~/.sm-pkg";
pub const PROJECT_FILE: &str = "sm-pkg.yaml";
pub const DL_CACHE: &str = "dl_cache";
pub const REPO_URL: &str = "https://github.com/sm-pkg/plugins/archive/refs/heads/master.zip";
pub const UPDATE_URL: &str =
    "https://raw.githubusercontent.com/sm-pkg/plugins/refs/heads/master/index.yaml";
pub const INDEX_FILE: &str = "index.yaml";
pub const PLUGIN_DEFINITION_FILE: &str = "plugin.yaml";

pub type BoxResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
