use crate::plugins;
use inquire::{InquireError, Select};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::Display,
    fs::File,
    path::{self, PathBuf},
};

pub const PROJECT_FILE: &str = "sm-pkg.json";

// https://wiki.alliedmods.net/Required_Versions_%28SourceMod%29
pub enum Game {
    TF2,
    CSGO,
    L4D2,
    OTHER,
}

impl Serialize for Game {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Game {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "tf2" => Ok(Game::TF2),
            "csgo" => Ok(Game::CSGO),
            "l4d2" => Ok(Game::L4D2),
            _ => Ok(Game::OTHER),
        }
    }
}
impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Game::TF2 => write!(f, "tf2"),
            Game::CSGO => write!(f, "csgo"),
            Game::L4D2 => write!(f, "l4d2"),
            _ => write!(f, "other"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub game: Game,
    pub plugins: Vec<String>,
}

pub struct Manager {
    pub root: path::PathBuf,
    pub config: Option<Project>,
}

impl Manager {
    pub fn new(root: path::PathBuf) -> Self {
        Manager { root, config: None }
    }

    pub fn open_or_new(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.project_file_path().exists() {
            true => Some(self.existing_project()?),
            false => Some(self.create_package_config()?),
        };
        println!(
            "🟢 Loaded package config {:?}",
            self.root.join(PROJECT_FILE)
        );
        Ok(())
    }

    pub fn open(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.project_file_path().exists() {
            true => Some(self.existing_project()?),
            false => {
                return Err(format!(
                    "❗No {} found, has the project been initialized?",
                    PROJECT_FILE,
                )
                .into());
            }
        };
        println!(
            "📂 Loaded package config {:?}",
            self.root.join(PROJECT_FILE)
        );
        Ok(())
    }

    pub fn project_file_path(&self) -> PathBuf {
        PROJECT_FILE.into()
    }

    pub fn save_package_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = match self.config {
            Some(ref config) => config,
            None => return Err("❗ No config?".into()),
        };
        let file = File::create(self.project_file_path())?;
        serde_json::to_writer_pretty(file, &config)?;

        Ok(())
    }

    fn has_plugin(&self, plugin_name: &str) -> bool {
        match self.config {
            None => false,
            Some(ref config) => config
                .plugins
                .contains(&plugin_name.to_string().to_lowercase()),
        }
    }

    pub fn add_plugin(
        &mut self,
        plugin: plugins::Definition,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.has_plugin(&plugin.name) {
            return Err("❗ Plugin already exists".into());
        }
        match &mut self.config {
            Some(config) => {
                config.plugins.push(plugin.name.to_lowercase());
                Ok(())
            }
            None => Err("❗ No config?".into()),
        }
    }

    fn existing_project(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(self.project_file_path())?;
        let existing_config: Project = serde_json::from_reader(file)?;
        println!(
            "🔎 Existing project found! (game: {:?})",
            existing_config.game.to_string()
        );
        self.config = Some(existing_config);
        Ok(())
    }

    fn create_package_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let options: Vec<Game> = vec![Game::TF2, Game::L4D2, Game::CSGO, Game::OTHER];
        let game: Result<Game, InquireError> = Select::new("👇 Select a game", options).prompt();
        self.config = match game {
            Ok(choice) => Some(Project {
                game: choice,
                plugins: Vec::new(),
            }),
            Err(_) => return Err("❗ Failed to select a game".into()),
        };
        self.save_package_config()?;
        Ok(())
    }
}
