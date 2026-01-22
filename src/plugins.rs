use std::{
    fmt::Display,
    fs::{self, create_dir_all},
    path::PathBuf,
};

use serde::Deserialize;

use crate::{fsutil, repo::Repository, sdk};

/// Definition of a plugin.
#[derive(Debug, Deserialize, Clone)]
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
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
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

pub fn install(
    _app_root: &PathBuf,
    sdk_env: &sdk::Environment,
    repo: &Repository,
    output_path: &PathBuf,
    plugins: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !fs::exists(output_path)? {
        return Err("target path does not exist".into());
    };

    for plugin in repo.find_plugin_definitions(&plugins)? {
        install_plugin(&sdk_env, &plugin, &output_path)?
    }
    Ok(())
}

fn install_plugin(
    _builder: &sdk::Environment,
    _plugin: &Definition,
    _output_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // let source_dir = builder.build(&plugin)?;
    // println!("Installing {}", plugin.name);
    // fsutil::copy_dir_all(source_dir, output_path)?;
    Ok(())
}
