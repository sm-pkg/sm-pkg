use std::{
    fmt::Display,
    fs::File,
    path::{self, PathBuf},
};

use inquire::{InquireError, Select};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::plugins;

pub const PROJECT_FILE: &str = "sm-pkg.json";

// https://wiki.alliedmods.net/Required_Versions_%28SourceMod%29
enum Game {
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
    game: Game,
    plugins: Option<Vec<String>>,
}

pub struct Manager {
    pub root: path::PathBuf,
    pub config: Option<Project>,
}

impl Manager {
    pub fn new(root: path::PathBuf) -> Self {
        Manager { root, config: None }
    }

    pub fn open(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let project_file = self.project_file_path();
        if project_file.exists() {
            self.config = Some(self.existing_project()?);
        } else {
            self.config = Some(self.create()?);
        }
        Ok(())
    }

    pub fn project_file_path(&self) -> PathBuf {
        self.root.join(PROJECT_FILE)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = match self.config {
            Some(ref config) => config,
            None => return Err("No config?".into()),
        };
        let file = File::create(self.project_file_path())?;
        serde_json::to_writer_pretty(file, &config)?;

        Ok(())
    }

    pub fn add_plugin(
        &mut self,
        plugin: plugins::Definition,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.config {
            Some(config) => {
                if let Some(ref mut plugins) = config.plugins {
                    plugins.push(plugin.name);
                } else {
                    config.plugins = Some(vec![plugin.name]);
                }

                Ok(())
            }
            None => Err("No config?".into()),
        }
    }

    fn existing_project(&self) -> Result<Project, Box<dyn std::error::Error>> {
        let file = File::open(self.project_file_path())?;
        let existing_config: Project = serde_json::from_reader(file)?;
        println!(
            "🔴 Existing project found! (game: {:?})",
            existing_config.game.to_string()
        );
        Ok(existing_config)
    }

    fn create(&self) -> Result<Project, Box<dyn std::error::Error>> {
        let options: Vec<Game> = vec![Game::TF2, Game::L4D2, Game::CSGO, Game::OTHER];
        let game: Result<Game, InquireError> = Select::new("👇 Select a game", options).prompt();
        match game {
            Ok(choice) => Ok(Project {
                game: choice,
                plugins: Some(Vec::new()),
            }),
            Err(_) => Err("❗ Failed to select a game".into()),
        }
    }
}
