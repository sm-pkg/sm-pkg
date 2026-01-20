use std::{fmt::Display, fs, path::PathBuf};

use serde::Deserialize;

use crate::repo::Repository;

/// Definition of a plugin.
#[derive(Debug, Deserialize, Clone)]
pub struct Definition {
    /// Name of the plugin. Currently this must match the parent directory name in the repository.
    pub name: String,
    pub description: String,
    pub version: String,
    /// Which plugin scripts to compile from the plugin's directory.
    pub inputs: Option<Vec<String>>,
    pub url: Option<String>,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
    pub dependencies: Option<Vec<String>>,
    /// The full path to the plugin's directory
    pub path: Option<PathBuf>,
}

impl Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn install(
    repo: Repository,
    target: &PathBuf,
    plugins: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !fs::exists(target)? {
        return Err("target path does not exist".into());
    };

    for plugin in repo.find_plugin_definitions(&plugins)? {
        perform_install(&repo, &target, &plugin)?
    }
    Ok(())
}

fn perform_install(
    repo: &Repository,
    repo_root: &PathBuf,
    _plugin: &Definition,
) -> Result<(), Box<dyn std::error::Error>> {
    if !fs::exists(repo_root)? {
        return Err("target path does not exist".into());
    }

    repo.checkout_repo()?;

    //fsutil::copy_dir_all(plugin.path(), repo_root)?;
    Ok(())
}
