use crate::{BoxResult, plugins};
use archive::{ArchiveExtractor, ArchiveFormat};
use std::{
    fs::{File, create_dir_all, remove_dir_all, write},
    path::{Path, PathBuf},
};

const REPO_URL: &str = "https://github.com/sm-pkg/plugins/archive/refs/heads/master.zip";
pub const UPDATE_URL: &str =
    "https://raw.githubusercontent.com/sm-pkg/plugins/refs/heads/master/index.yaml";
pub const INDEX_FILE: &str = "index.yaml";

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
                println!("Failed to find index.json, maybe you need to run update?");
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
        println!("📢 Downloading repo snapshot");
        let body = reqwest::get(REPO_URL).await?.bytes().await?;
        println!("✅ Successfully downloaded");

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
                println!("📝 {}", dest_path.display());
            } else {
                write(&dest_path, &file.data)?;
            }
        }

        Ok(())
    }

    pub fn find_plugin_definitions(
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

    pub fn find_plugin_definition(&self, plugin: &String) -> BoxResult<plugins::Definition> {
        for known_plugin in self.read_index()? {
            if known_plugin.name == *plugin {
                let mut plugin = known_plugin.clone();
                plugin.path = Some(
                    self.root
                        .join(format!("repo/{}/src/scripting", &plugin.name))
                        .to_path_buf(),
                );
                return Ok(plugin);
            }
        }

        Err(format!("Plugin not found: {}", plugin).into())
    }
}
