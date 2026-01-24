use crate::{plugins, sdk::Branch, template};
use inquire::{InquireError, Select};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::File,
    path::{self, PathBuf},
};

pub const PROJECT_FILE: &str = "sm-pkg.json";

// https://wiki.alliedmods.net/Required_Versions_%28SourceMod%29
// https://github.com/alliedmodders/sourcemod/tree/master/gamedata/sdktools.games
#[derive(clap::ValueEnum, Clone, Debug, Serialize, Default, Deserialize)]
pub enum Game {
    #[default]
    TF,
    HL2,
}

impl Game {
    pub fn mod_folder(&self) -> PathBuf {
        match self {
            Game::TF => PathBuf::from("tf"),
            Game::HL2 => PathBuf::from("hl2"),
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Game::TF => write!(f, "Team Fortress 2"),
            Game::HL2 => write!(f, "Half-Life 2"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Package {
    pub game: Game,
    pub branch: Branch,
    pub plugins: Vec<String>,
    pub configs: Vec<template::FileConfig>,
}

pub struct Manager {
    pub root: path::PathBuf,
    pub package: Option<Package>,
}

impl Manager {
    pub fn new(root: path::PathBuf) -> Self {
        println!("🏗️ Using project root {:?}", root);
        Manager {
            root,
            package: None,
        }
    }

    pub fn open_or_new(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.project_file_path().exists() {
            true => Some(self.existing_project()?),
            false => Some(self.create_package_config()?),
        };
        println!("🟢 Loaded package config {:?}", self.project_file_path());
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
        println!("📂 Loaded package config {:?}", self.project_file_path());
        Ok(())
    }

    pub fn project_file_path(&self) -> PathBuf {
        self.root.join(PROJECT_FILE)
    }

    pub fn save_package_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = match self.package {
            Some(ref config) => config,
            None => return Err("❗ No config?".into()),
        };
        let file = File::create(self.project_file_path())?;
        serde_json::to_writer_pretty(file, &config)?;

        Ok(())
    }

    fn has_plugin(&self, plugin_name: &str) -> bool {
        match self.package {
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
        match &mut self.package {
            Some(config) => {
                config.plugins.push(plugin.name.to_lowercase());
                Ok(())
            }
            None => Err("❗ No config?".into()),
        }
    }

    fn existing_project(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(self.project_file_path())?;
        let existing_config: Package = serde_json::from_reader(file)?;
        println!(
            "🔎 Existing project found! (game: {:?})",
            existing_config.game.to_string()
        );
        self.package = Some(existing_config);
        Ok(())
    }

    fn create_package_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let branch_opts = vec![Branch::Stable, Branch::Dev];
        let branch: Result<Branch, InquireError> =
            Select::new("👇 Select a metamod/sourcemod branch", branch_opts).prompt();

        let options: Vec<Game> = vec![Game::TF, Game::HL2];
        let game: Result<Game, InquireError> = Select::new("👇 Select a game", options).prompt();
        self.package = match game {
            Ok(choice) => Some(Package {
                branch: branch?,
                game: choice,
                plugins: Vec::new(),
                configs: Vec::new(),
            }),
            Err(_) => return Err("❗ Failed to select a game".into()),
        };
        self.save_package_config()?;
        Ok(())
    }

    pub fn write_configs() -> Result<(), Box<dyn std::error::Error>> {
        //let sm_config = template::create_sourcemod_cfg(overrides);

        Ok(())
    }
}
