use crate::{BoxResult, VERSION, plugins, sdk, templates};
use askama::Template;
use inquire::{InquireError, Select};
use serde::{Deserialize, Serialize};
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{collections::HashMap, fmt::Display, fs::File, path::PathBuf};
use std::{fmt, fs};

pub const PROJECT_FILE: &str = "sm-pkg.yaml";

// https://wiki.alliedmods.net/Required_Versions_%28SourceMod%29
// https://github.com/alliedmodders/sourcemod/tree/master/gamedata/sdktools.games
#[derive(clap::ValueEnum, Clone, Debug, Serialize, Default, Deserialize)]
pub enum Game {
    #[default]
    TF,
}

impl Game {
    pub fn mod_folder(&self) -> PathBuf {
        match self {
            Game::TF => PathBuf::from("tf"),
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Game::TF => write!(f, "Team Fortress 2"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Package {
    pub game: Game,
    pub create_startup_script: Option<bool>,
    pub startup_opts: Option<templates::StartSh>,
    pub branch: sdk::Branch,
    pub plugins: Vec<String>,
    pub templates: Option<TemplateSet>,
    pub raw_configs: Option<Vec<SimpleConfig>>,
}

#[derive(Serialize, Deserialize)]
pub struct SimpleConfig {
    pub path: PathBuf,
    pub options: HashMap<String, String>,
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
pub struct Project {
    /// Root directory of the project.
    pub root: PathBuf,
    pub package: Option<Package>,
}

impl Project {
    pub fn new(root: PathBuf) -> BoxResult<Self> {
        println!("🏗️ Using project root {:?}", root);
        Ok(Project {
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
        let options: Vec<Game> = vec![Game::TF];
        let game: Result<Game, InquireError> = Select::new("👇 Select a game", options).prompt();
        self.package = match game {
            Ok(choice) => Some(Package {
                branch: branch?,
                game: choice,
                plugins: Vec::new(),
                templates: None,
                raw_configs: None,
                create_startup_script: None,
                startup_opts: None,
            }),
            Err(_) => return Err("❗ Failed to select a game".into()),
        };
        self.save_package_config()?;
        Ok(())
    }

    pub fn write_configs(&self) -> BoxResult {
        let pkg = match &self.package {
            None => return Err("No package loaded".into()),
            Some(pkg) => pkg,
        };
        if let Some(configs) = &pkg.templates {
            self.write_sourcemod_cfg(&configs.sourcemod_cfg)?;
            self.write_core_cfg(&configs.core_cfg)?;
            self.write_databases_cfg(&configs.databases_cfg)?;
            self.write_maplists_cfg(&configs.maplists_cfg)?;
            self.write_admins_cfg(&configs.admins_cfg)?;
            self.write_admin_groups_cfg(&configs.admin_groups_cfg)?;
            self.write_admin_overrides_cfg(&configs.admin_overrides_cfg)?;
            self.write_admins_simple_ini(&configs.admins_simple_ini)?;
        };

        if let Some(raw_configs) = &pkg.raw_configs {
            self.write_raw_configs(raw_configs)?;
        }

        if let Some(create) = pkg.create_startup_script
            && create
        {
            self.write_startup_script(&pkg.startup_opts)?
        }

        Ok(())
    }

    fn write_startup_script(&self, config: &Option<templates::StartSh>) -> BoxResult {
        let script_path = self.root.join("start.sh");
        match &config {
            None => Err("No startup_opts definition found".into()),
            Some(template) => match write_cfg(&TagFormat::Shell, &script_path, template) {
                Err(e) => Err(e),
                Ok(()) => {
                    let mut perms = fs::metadata(&script_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&script_path, perms)?;
                    Ok(())
                }
            },
        }
    }

    fn write_raw_configs(&self, raw_configs: &Vec<SimpleConfig>) -> BoxResult {
        for raw_config in raw_configs {
            let out_path = self.root.join(&raw_config.path);
            let mut file = File::create(&out_path)?;
            write_tag(&TagFormat::Ini, &mut file)?;
            for (key, value) in &raw_config.options {
                writeln!(file, "{} \"{}\"", key, value)?;
            }
            println!("📝 Created {}", out_path.display());
        }

        Ok(())
    }

    fn write_sourcemod_cfg(&self, config: &Option<templates::SourcemodCfg>) -> BoxResult {
        if let Some(template) = &config {
            write_cfg(
                &TagFormat::Ini,
                &self.root.join("tf/cfg/sourcemod/sourcemod.cfg"),
                template,
            )?;
        }

        Ok(())
    }

    fn write_core_cfg(&self, config: &Option<templates::CoreCfg>) -> BoxResult {
        if let Some(template) = &config {
            write_cfg(
                &TagFormat::Ini,
                &self.root.join("tf/addons/sourcemod/configs/core.cfg"),
                template,
            )?;
        }

        Ok(())
    }

    fn write_databases_cfg(&self, config: &Option<templates::DatabasesCfg>) -> BoxResult {
        if let Some(template) = &config {
            write_cfg(
                &TagFormat::Ini,
                &self.root.join("tf/addons/sourcemod/configs/databases.cfg"),
                template,
            )?
        }

        Ok(())
    }

    fn write_maplists_cfg(&self, config: &Option<templates::MaplistsCfg>) -> BoxResult {
        if let Some(template) = &config {
            write_cfg(
                &TagFormat::Ini,
                &self.root.join("tf/addons/sourcemod/configs/maplists.cfg"),
                template,
            )?;
        }

        Ok(())
    }

    fn write_admins_simple_ini(&self, config: &Option<templates::AdminsSimpleIni>) -> BoxResult {
        if let Some(template) = &config {
            write_cfg(
                &TagFormat::Ini,
                &self
                    .root
                    .join("tf/addons/sourcemod/configs/admins_simple.ini"),
                template,
            )?
        }

        Ok(())
    }

    fn write_admins_cfg(&self, config: &Option<templates::AdminsCfg>) -> BoxResult {
        if let Some(template) = &config {
            write_cfg(
                &TagFormat::Ini,
                &self.root.join("tf/addons/sourcemod/configs/admins.cfg"),
                template,
            )?
        }

        Ok(())
    }

    fn write_admin_groups_cfg(&self, config: &Option<templates::AdminGroupsCfg>) -> BoxResult {
        if let Some(template) = &config {
            write_cfg(
                &TagFormat::Ini,
                &self
                    .root
                    .join("tf/addons/sourcemod/configs/admin_groups.cfg"),
                template,
            )?
        }

        Ok(())
    }

    fn write_admin_overrides_cfg(
        &self,
        config: &Option<templates::AdminOverridesCfg>,
    ) -> BoxResult {
        if let Some(template) = &config {
            write_cfg(
                &TagFormat::Ini,
                &self
                    .root
                    .join("tf/addons/sourcemod/configs/admin_overrides.cfg"),
                template,
            )?
        }

        Ok(())
    }
}

enum TagFormat {
    Shell,
    Ini,
}

fn write_tag(format: &TagFormat, fp: &mut impl std::io::Write) -> BoxResult {
    let ts = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
    let body = match format {
        TagFormat::Ini => format!(
            "// Generated by sm-pkg-{} - {}\n// DO NOT EDIT THIS FILE MANUALLY\n",
            VERSION, ts
        ),
        TagFormat::Shell => format!(
            "# Generated by sm-pkg-{} - {}\n# DO NOT EDIT THIS FILE MANUALLY\n",
            VERSION, ts
        ),
    };
    fp.write_all(body.as_bytes())?;
    Ok(())
}

fn write_cfg(format: &TagFormat, path: &Path, template: impl Template) -> BoxResult {
    let mut fp = File::create(path)?;
    write_tag(format, &mut fp)?;
    match Template::write_into(&template, &mut fp) {
        Ok(()) => {
            println!("📝 Created {}", path.display());
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}
