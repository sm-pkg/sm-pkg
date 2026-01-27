pub mod fsutil;
pub mod plugins;
pub mod project;
pub mod repo;
pub mod sdk;
pub mod templates;

pub type BoxResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
