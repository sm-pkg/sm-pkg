pub mod config;
pub mod fsutil;
pub mod plugins;
pub mod project;
pub mod repo;
pub mod sdk;
pub mod templates;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_ROOT: &str = "~/.sm-pkg";

pub type BoxResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
