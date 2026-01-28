use crate::{BoxResult, plugins, sdk, templates};
use askama::Template;
use inquire::{InquireError, Select};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::File,
    path::{self, PathBuf},
};

pub const PROJECT_FILE: &str = "sm-pkg.yaml";

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
    pub branch: sdk::Branch,
    pub plugins: Vec<String>,
    pub templates: Option<TemplateSet>,
}

#[derive(Serialize, Deserialize)]
pub struct TemplateSet {
    pub sourcemod_cfg: Option<templates::SourcemodCfg>,
    pub maplists_cfg: Option<templates::MaplistsCfg>,
    pub databases_cfg: Option<templates::DatabasesCfg>,
    pub core_cfg: Option<templates::CoreCfg>,
    pub admins_simple_ini: Option<templates::AdminsSimpleIni>,
    pub admins_cfg: Option<templates::AdminsCfg>,
    pub admin_groups_cfg: Option<templates::AdminGroupsCfg>,
    pub admin_overrides_cfg: Option<templates::AdminOverridesCfg>,
}

/// Manager is responsible for loading and managing a project using its package configuration file, sm-pkg.yaml.
pub struct Manager {
    /// Root directory of the project.
    pub root: path::PathBuf,
    pub package: Option<Package>,
}

impl Manager {
    pub fn new(root: path::PathBuf) -> BoxResult<Self> {
        println!("🏗️ Using project root {:?}", root);
        Ok(Manager {
            root,
            package: None,
        })
    }

    pub fn open_or_new(&mut self) -> BoxResult {
        match self.project_file_path().exists() {
            true => Some(self.existing_project()?),
            false => Some(self.create_package_config()?),
        };
        println!("🟢 Loaded package config {:?}", self.project_file_path());
        Ok(())
    }

    pub fn open(&mut self) -> BoxResult {
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

    pub fn save_package_config(&self) -> BoxResult {
        let config = match self.package {
            Some(ref config) => config,
            None => return Err("❗ No config?".into()),
        };
        let file = File::create(self.project_file_path())?;
        serde_yaml::to_writer(file, &config)?;

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

    pub fn add_plugin(&mut self, plugin: plugins::Definition) -> BoxResult {
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

    fn existing_project(&mut self) -> BoxResult {
        let file = File::open(self.project_file_path())?;
        let existing_config: Package = serde_yaml::from_reader(file)?;
        println!(
            "🔎 Existing project found! (game: {:?})",
            existing_config.game.to_string()
        );
        self.package = Some(existing_config);
        Ok(())
    }

    fn create_package_config(&mut self) -> BoxResult {
        let branch_opts = vec![sdk::Branch::Stable, sdk::Branch::Dev];
        let branch: Result<sdk::Branch, InquireError> =
            Select::new("👇 Select a metamod/sourcemod branch", branch_opts).prompt();
        let options: Vec<Game> = vec![Game::TF, Game::HL2];
        let game: Result<Game, InquireError> = Select::new("👇 Select a game", options).prompt();
        self.package = match game {
            Ok(choice) => Some(Package {
                branch: branch?,
                game: choice,
                plugins: Vec::new(),
                templates: None,
            }),
            Err(_) => return Err("❗ Failed to select a game".into()),
        };
        self.save_package_config()?;
        Ok(())
    }

    pub fn write_configs(&self) -> BoxResult {
        let configs = match &self.package {
            Some(config) => match &config.templates {
                Some(configs) => configs,
                None => return Ok(()),
            },
            None => {
                println!("⚠️ No configs were found, this is probably a mistake");
                return Ok(());
            }
        };

        self.write_sourcemod_cfg(&configs.sourcemod_cfg)?;
        self.write_core_cfg(&configs.core_cfg)?;
        self.write_databases_cfg(&configs.databases_cfg)?;
        self.write_maplists_cfg(&configs.maplists_cfg)?;
        self.write_admins_cfg(&configs.admins_cfg)?;
        self.write_admin_groups_cfg(&configs.admin_groups_cfg)?;
        self.write_admin_overrides_cfg(&configs.admin_overrides_cfg)?;
        self.write_admins_simple_ini(&configs.admins_simple_ini)?;

        Ok(())
    }

    fn write_sourcemod_cfg(&self, config: &Option<templates::SourcemodCfg>) -> BoxResult {
        if let Some(template) = &config {
            let path = self.root.join("tf/cfg/sourcemod/sourcemod.cfg");
            template.write_into(&mut File::create(&path)?)?;
            println!("📝 Created {}", path.display());
        }

        Ok(())
    }

    fn write_core_cfg(&self, config: &Option<templates::CoreCfg>) -> BoxResult {
        if let Some(template) = &config {
            let path = self.root.join("tf/addons/sourcemod/configs/core.cfg");
            template.write_into(&mut File::create(&path)?)?;
            println!("📝 Created {}", path.display());
        }

        Ok(())
    }

    fn write_databases_cfg(&self, config: &Option<templates::DatabasesCfg>) -> BoxResult {
        if let Some(template) = &config {
            let path = self.root.join("tf/addons/sourcemod/configs/databases.cfg");
            template.write_into(&mut File::create(&path)?)?;
            println!("📝 Created {}", path.display());
        }

        Ok(())
    }

    fn write_maplists_cfg(&self, config: &Option<templates::MaplistsCfg>) -> BoxResult {
        if let Some(template) = &config {
            let path = self.root.join("tf/addons/sourcemod/configs/maplists.cfg");
            template.write_into(&mut File::create(&path)?)?;
            println!("📝 Created {}", path.display());
        }

        Ok(())
    }

    fn write_admins_simple_ini(&self, config: &Option<templates::AdminsSimpleIni>) -> BoxResult {
        if let Some(template) = &config {
            let path = self
                .root
                .join("tf/addons/sourcemod/configs/admins_simple.ini");
            template.write_into(&mut File::create(&path)?)?;
            println!("📝 Created {}", path.display());
        }

        Ok(())
    }

    fn write_admins_cfg(&self, config: &Option<templates::AdminsCfg>) -> BoxResult {
        if let Some(template) = &config {
            let path = self.root.join("tf/addons/sourcemod/configs/admins.cfg");
            template.write_into(&mut File::create(&path)?)?;
            println!("📝 Created {}", path.display());
        }

        Ok(())
    }

    fn write_admin_groups_cfg(&self, config: &Option<templates::AdminGroupsCfg>) -> BoxResult {
        if let Some(template) = &config {
            let path = self
                .root
                .join("tf/addons/sourcemod/configs/admin_groups.cfg");
            template.write_into(&mut File::create(&path)?)?;
            println!("📝 Created {}", path.display());
        }

        Ok(())
    }

    fn write_admin_overrides_cfg(
        &self,
        config: &Option<templates::AdminOverridesCfg>,
    ) -> BoxResult {
        if let Some(template) = &config {
            let path = self
                .root
                .join("tf/addons/sourcemod/configs/admin_overrides.cfg");
            template.write_into(&mut File::create(&path)?)?;
            println!("📝 Created {}", path.display());
        }

        Ok(())
    }

    // fn handle_template_cfg(&self, fc: &FileConfig, output_file: &mut File) -> BoxResult {
    //     // Write out raw section first, explicit options should override anything in there.
    //     match &fc.raw {
    //         Some(content) => output_file.write_all(content.as_bytes())?,
    //         None => (),
    //     };

    //     match &fc.options {
    //         Some(v) => {
    //             for (key, value) in v {
    //                 write!(output_file, "{} \"{}\"\n", key, value)?;
    //             }
    //             Ok(())
    //         }
    //         None => Ok(()),
    //     }
    // }
}
