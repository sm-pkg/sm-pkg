use std::{
    fmt::Display,
    fs::{self, create_dir_all, remove_dir_all},
    path::PathBuf,
};

use serde::Deserialize;

use crate::{compiler, fsutil, repo::Repository};

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

pub struct Builder {
    root: PathBuf,
}

impl Builder {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn build(&self, plugin_def: Definition) -> Result<(), Box<dyn std::error::Error>> {
        let build_root = self.setup_build_root(&plugin_def)?;
        let mut args = compiler::CompilerArgs::default();
        args.active_dir = Some(build_root.join("src/scripting"));

        println!("🔨 Build root {:?}", args.active_dir);
        compiler::compile(&mut args, &plugin_def)?;
        Ok(())
    }

    // fn install(
    //     &self,
    //     plugin_def: &Definition,
    //     target: &PathBuf,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     Ok(())
    // }

    /// Setup a new clean build root containing the plugin sources. This wipes all
    /// existing build artifacts from the plugins build root if it previous existed.
    fn setup_build_root(
        &self,
        plugin_def: &Definition,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let build_root = self.build_root(&plugin_def);
        if build_root.exists() {
            remove_dir_all(&build_root)?;
        }
        create_dir_all(&build_root)?;
        let src = self.root.join("repo").join(&plugin_def.name);
        if !src.exists() {
            return Err("plugin source directory does not exist?".into());
        }
        fsutil::copy_dir_all(src, &build_root)?;
        Ok(build_root)
    }

    fn build_root(&self, plugin_def: &Definition) -> PathBuf {
        self.root.join("build").join(plugin_def.name.to_lowercase())
    }
}
