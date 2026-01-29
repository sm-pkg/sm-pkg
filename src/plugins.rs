use crate::{BoxResult, fsutil, repo::Repository, sdk};
use serde::{Deserialize, Serialize};
use std::{collections, fmt::Display, fs::create_dir_all, path::PathBuf};

pub const PLUGIN_DEFINITION_FILE: &str = "plugin.yaml";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConfigFile {
    pub name: String,
    pub values: collections::HashMap<String, String>,
}

/// Definition of a plugin.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Definition {
    /// Name of the plugin. Currently this must match the parent directory name in the repository.
    pub name: String,
    pub description: String,
    pub version: String,
    /// Which plugin scripts to compile from the plugin's directory.
    pub inputs: Option<Vec<PathBuf>>,
    pub url: Option<String>,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
    pub dependencies: Option<Vec<String>>,
    /// The full path to the plugin's directory
    pub path: Option<PathBuf>,

    pub configs: Option<Vec<ConfigFile>>,
}

impl Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn build(
    app_root: &PathBuf,
    sdk_env: &sdk::Environment,
    repo: &Repository,
    plugins: &Vec<String>,
) -> BoxResult<Vec<PathBuf>> {
    let mut outputs = Vec::new();
    for plugin in repo.find_plugin_definitions(&plugins)? {
        let src_tree = app_root.join("repo").join(&plugin.name).join("src");
        let build_dir = app_root.join("build").join(&plugin.name);
        create_dir_all(&build_dir)?;

        fsutil::copy_dir_all(src_tree, &build_dir)?;

        let mut args = sdk_env.args();
        args.active_dir = Some(build_dir.join("scripting"));
        sdk_env.compile(&mut args, &plugin)?;
        outputs.push(build_dir);
    }

    Ok(outputs)
}
