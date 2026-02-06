use crate::{
    BoxResult, fsutil,
    project::{Game, SimpleConfig},
    repo::LocalRepo,
    sdk,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    fs::create_dir_all,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const PLUGIN_DEFINITION_FILE: &str = "plugin.yaml";

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
    pub configs: Option<Vec<SimpleConfig>>,
    pub games: Option<Vec<Game>>,
}

impl Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn build(
    app_root: &Path,
    sdk_env: &sdk::Environment,
    build_root: &Path,
    repo: &LocalRepo,
    plugins: &Vec<String>,
) -> BoxResult<Vec<PathBuf>> {
    let mut outputs = Vec::new();
    for plugin in repo.find_plugin_definitions(plugins)? {
        let src_tree = app_root.join("repo").join(&plugin.name).join("src");
        let build_dir = build_root.join(&plugin.name);
        create_dir_all(&build_dir)?;
        fsutil::copy_dir_all(src_tree, &build_dir)?;

        if let Some(deps) = &plugin.dependencies {
            let include_dir = build_dir.join("include");
            create_dir_all(&include_dir)?;
            for dep in deps {
                let inc_tree = app_root
                    .join("repo")
                    .join(dep)
                    .join("src/scripting/include");
                if !inc_tree.exists() {
                    return Err(
                        format!("Dependency include directory not found: {:?}", inc_tree).into(),
                    );
                }
                println!("➕ Adding {} includes", dep);
                fsutil::copy_dir_all(inc_tree, &include_dir)?;
            }
        }

        let mut args = sdk_env.args();
        args.active_dir = Some(build_dir.join("scripting"));
        // The path must be a full absolute path
        args.include(build_dir.clone().join("include").canonicalize()?);
        sdk_env.compile(&mut args, &plugin)?;
        outputs.push(build_dir);
    }

    Ok(outputs)
}

/// Create an empty, new build context to build plugins under.
pub fn create_build_root(app_root: &Path) -> BoxResult<PathBuf> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let build_root = app_root.join("build").join(format!("bld-{}", now));
    create_dir_all(&build_root)?;
    Ok(build_root)
}
