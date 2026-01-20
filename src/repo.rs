use git2;
use serde::Deserialize;
use std::fmt::Display;
use std::{fs::File, io::Write, path::Path};

const REPO_URL: &str = "https://github.com/leighmacdonald/smpkg-repo";

#[derive(Debug, Deserialize, Clone)]
pub struct PluginDefinition {
    pub name: String,
    pub description: String,
    pub version: String,
    pub inputs: Option<Vec<String>>,
    pub url: Option<String>,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
    pub dependencies: Option<Vec<String>>,
}

impl Display for PluginDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct Repository<'a> {
    url: &'a str,
    root: &'a Path,
}

impl<'a> Repository<'a> {
    pub fn new(root: &'a Path, url: &'a str) -> Self {
        Repository { url, root }
    }

    pub async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
        let body = reqwest::get(self.url).await?.bytes().await?;
        let mut file = File::create(self.root.join("index.json"))?;
        file.write_all(&body[..])?;

        Ok(())
    }

    pub fn root_dir(&self) -> &Path {
        self.root
    }

    fn read_index(&self) -> Result<Vec<PluginDefinition>, Box<dyn std::error::Error>> {
        let index = match File::open(self.root.join("index.json")) {
            Ok(file) => file,
            Err(e) => {
                println!("Failed to find index.json, maybe you need to run update?");
                return Err(e.into());
            }
        };
        let results: Vec<PluginDefinition> = serde_json::from_reader(index)?;
        Ok(results)
    }

    pub fn search(&self, query: &str) -> Result<Vec<PluginDefinition>, Box<dyn std::error::Error>> {
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

    //pub async fn build(name: String) {}
    // git clone --no-checkout --depth=1 --filter=tree:0 git@github.com:leighmacdonald/smpkg-repo test-repo
    // git sparse-checkout set --no-cone /connect
    // git checkout
    pub fn checkout_repo(
        &self,
        plugin: &PluginDefinition,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let build_root = self.root_dir().join("repo");
        let path = build_root.as_path();

        if !path.exists() {
            match git2::Repository::clone(REPO_URL, path) {
                Ok(repo) => repo,
                Err(e) => panic!("failed to clone: {}", e),
            };
        } else {
            let repo = match git2::Repository::open(path) {
                Ok(repo) => repo,
                Err(e) => panic!("failed to open: {}", e),
            };

            let remote_name = "origin";
            let mut remote = repo
                .find_remote(remote_name)
                .expect("Failed to find remote");

            // Configure fetch options and callbacks (e.g., for authentication or progress)
            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(|_url, _username_from_url, _allowed_types| git2::Cred::default());

            let mut fetch_options = Some(git2::FetchOptions::new());
            fetch_options.as_mut().unwrap().remote_callbacks(callbacks);

            // Fetch all remote branches
            remote
                .fetch(
                    &["refs/heads/*:refs/remotes/origin/*"],
                    fetch_options.as_mut(),
                    None,
                )
                .expect("Failed to fetch");

            let oid = repo.refname_to_id("refs/remotes/origin/master")?;
            let object = repo.find_object(oid, None).unwrap();
            repo.reset(&object, git2::ResetType::Hard, None)?;
            println!("✅ Repo update complete");
        }

        println!("📥 Checking out {} into {}", &plugin.name, path.display());

        Ok(())
    }

    pub fn find_plugin_definitions(
        &self,
        plugins: &Vec<String>,
    ) -> Result<Vec<PluginDefinition>, Box<dyn std::error::Error>> {
        let packages = self.read_index()?;
        let mut valid_definitions: Vec<PluginDefinition> = Vec::new();
        for plugin in plugins {
            let mut found = false;
            for package in &packages {
                if package.name == *plugin {
                    valid_definitions.push(package.clone());
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

    pub fn find_plugin_definition(
        &self,
        plugin: &String,
    ) -> Result<PluginDefinition, Box<dyn std::error::Error>> {
        for known_plugin in self.read_index()? {
            if known_plugin.name == *plugin {
                return Ok(known_plugin);
            }
        }

        Err(format!("Plugin not found: {}", plugin).into())
    }
}
