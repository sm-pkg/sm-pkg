use std::{fs, path::PathBuf};

use crate::repo::{PluginDefinition, Repository};

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
    plugin: &PluginDefinition,
) -> Result<(), Box<dyn std::error::Error>> {
    if !fs::exists(repo_root)? {
        return Err("target path does not exist".into());
    }

    repo.checkout_repo(plugin)?;

    //fsutil::copy_dir_all(plugin.path(), repo_root)?;
    Ok(())
}
