pub mod local_path;
pub mod overlay;

use crate::{BoxResult, INDEX_FILE, REPO_URL, plugins, repo::overlay::PluginOverlays};
use archive::{ArchiveExtractor, ArchiveFormat};
use std::{
    fs::{File, create_dir_all, remove_dir_all, write},
    path::{Path, PathBuf},
};

pub trait PluginDefinitionProvider<'a> {
    fn find_plugin_definitions(&self, plugins: &Vec<String>)
    -> BoxResult<Vec<plugins::Definition>>;
    fn find_plugin_definition(&self, plugin: &String) -> BoxResult<plugins::Definition>;
}

pub struct LocalRepo<'a> {
    root: &'a Path,
}

impl<'a> LocalRepo<'a> {
    pub fn new(root: &'a Path) -> Self {
        LocalRepo { root }
    }

    pub async fn update(&self) -> BoxResult {
        self.checkout_repo().await?;

        Ok(())
    }

    pub fn root_dir(&self) -> &Path {
        self.root
    }

    fn read_index(&self) -> Result<Vec<plugins::Definition>, Box<dyn std::error::Error>> {
        let index = match File::open(self.root.join("repo").join(INDEX_FILE)) {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to find index.json, maybe you need to run update?");
                return Err(e.into());
            }
        };
        let results: Vec<plugins::Definition> = serde_yaml::from_reader(index)?;
        Ok(results)
    }

    pub fn search(&self, query: &str) -> BoxResult<Vec<plugins::Definition>> {
        let mut packages = self.read_index()?;
        packages.retain(|p| {
            p.name.to_lowercase().contains(query) || p.description.to_lowercase().contains(query)
        });
        if packages.is_empty() {
            Err("No matches found".into())
        } else {
            Ok(packages)
        }
    }

    // pub async fn build(name: String) {}
    // git clone --no-checkout --depth=1 --filter=tree:0 git@github.com:sm-pkg/plugins plugins
    // git sparse-checkout set --no-cone /connect
    // git checkout
    pub async fn checkout_repo(&self) -> BoxResult {
        let out_root = self.root_dir().join("repo");
        info!("📢 Downloading repo snapshot");
        let body = reqwest::get(REPO_URL).await?.bytes().await?;
        info!("✅ Successfully downloaded");

        if out_root.exists() {
            remove_dir_all(&out_root)?;
        }

        create_dir_all(&out_root)?;

        let extractor = ArchiveExtractor::new();
        let files = extractor.extract(&body, ArchiveFormat::Zip)?;
        for file in files {
            let archive_path = PathBuf::from(&file.path);
            let dest_path = out_root.join(archive_path.strip_prefix("plugins-master")?);
            if file.is_directory {
                create_dir_all(&dest_path)?;
                debug!("📝 {}", dest_path.display());
            } else {
                write(&dest_path, &file.data)?;
            }
        }

        Ok(())
    }

    pub fn plugins(&self) -> BoxResult<Vec<plugins::Definition>> {
        let mut valid_definitions: Vec<plugins::Definition> = Vec::new();
        for known_plugin in self.read_index()? {
            let mut plugin = known_plugin.clone();
            plugin.path = Some(
                self.root
                    .join(format!("repo/{}/src/scripting", &plugin.name))
                    .to_path_buf(),
            );
            valid_definitions.push(plugin);
        }

        Ok(valid_definitions)
    }
}

impl<'a> PluginDefinitionProvider<'a> for LocalRepo<'a> {
    fn find_plugin_definitions(
        &self,
        plugins: &Vec<String>,
    ) -> BoxResult<Vec<plugins::Definition>> {
        let packages = self.read_index()?;
        let mut valid_definitions: Vec<plugins::Definition> = Vec::new();
        for plugin in plugins {
            let mut found = false;
            for package in &packages {
                if package.name == *plugin {
                    let mut plugin_def = package.clone();
                    plugin_def.path = Some(
                        self.root
                            .join(format!("repo/{}/src/scripting", plugin_def.name))
                            .to_path_buf(),
                    );
                    valid_definitions.push(plugin_def);
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(format!("Plugin not found: {}", plugin).into());
            }
        }

        Ok(valid_definitions)
    }

    fn find_plugin_definition(&self, plugin: &String) -> BoxResult<plugins::Definition> {
        match self
            .read_index()?
            .iter()
            .find(|d| d.name == *plugin)
            .and_then(|f| {
                let mut plugin = f.clone();
                plugin.path = Some(
                    self.root
                        .join(format!("repo/{}/src/scripting", &plugin.name))
                        .to_path_buf(),
                );
                Some(plugin)
            }) {
            None => Err(format!("Plugin not found: {}", plugin).into()),
            Some(plugin) => Ok(plugin),
        }
    }
}

pub fn open_default_overlays(root_path: &'static Path) -> BoxResult<PluginOverlays<'static>> {
    let mut overlays = PluginOverlays::default();

    let local: Box<dyn PluginDefinitionProvider> = Box::new(LocalRepo::new(root_path));
    overlays.add_overlay(local);

    Err("".into())
}
