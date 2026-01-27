use git2;
use std::{fs::File, io::Write, path::Path};

use crate::{BoxResult, plugins};

const REPO_URL: &str = "https://github.com/sm-pkg/plugins";

pub struct Repository<'a> {
    url: &'a str,
    root: &'a Path,
}

impl<'a> Repository<'a> {
    pub fn new(root: &'a Path, url: &'a str) -> Self {
        Repository { url, root }
    }

    pub async fn update(&self) -> BoxResult {
        let body = reqwest::get(self.url).await?.bytes().await?;
        let mut file = File::create(self.root.join("index.yaml"))?;
        file.write_all(&body[..])?;
        self.checkout_repo()?;

        Ok(())
    }

    pub fn root_dir(&self) -> &Path {
        self.root
    }

    fn read_index(&self) -> Result<Vec<plugins::Definition>, Box<dyn std::error::Error>> {
        let index = match File::open(self.root.join("index.yaml")) {
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
    pub fn checkout_repo(&self) -> BoxResult {
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

        println!("📥 Checking out into {}", path.display());

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
