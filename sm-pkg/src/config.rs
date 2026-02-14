use crate::{BoxResult, CONFIG_FILE_PATH, sdk};
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::{fs::File, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub compiler_args: sdk::CompilerArgs,
}

impl Config {
    pub fn open_or_default() -> BoxResult<Self> {
        match config_path() {
            Some(p) => {
                let reader = File::open(p)?;
                let config: Config = serde_yaml::from_reader(reader)?;
                Ok(config)
            }
            None => Ok(Config {
                compiler_args: sdk::CompilerArgs::default(),
            }),
        }
    }
}

fn config_path() -> Option<PathBuf> {
    match PathBuf::from(CONFIG_FILE_PATH).try_resolve() {
        Ok(path) => {
            if path.exists() {
                Some(path.to_path_buf())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
