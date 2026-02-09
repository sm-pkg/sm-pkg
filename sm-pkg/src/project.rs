use crate::{BoxResult, VERSION, plugins, repo, sdk, templates};
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
    AG2,
    ALIENSWARM,
    AOC,
    BG2,
    BMS,
    CSPROMOD,
    CSTRIKE,
    DINODDAY,
    DOD,
    DOI,
    DYSTOPIA,
    EMPIRES,
    ESMOD,
    FAS,
    FF,
    FOF,
    GESOURCE,
    GMOD9,
    HIDDEN,
    HL1MP,
    HL2CTF,
    HL2MP,
    INSURGENCY,
    IOS,
    KZ,
    LEFT4DEAD2,
    MODULARCOMBAT,
    NEOTOKYO,
    NMRIH,
    NUCLEARDAWN,
    OBSIDIAN,
    OPENFORTRESS,
    PF2,
    PVKILL,
    REACTIVEDROP,
    MKBETA,
    SHIP,
    SOURCEFORTS,
    SYNERGY,
    #[default]
    #[serde(alias = "tf", alias = "Tf")]
    TF,
    TF2CLASSIC,
    TF2CLASSIFIED,
    TREASON,
    ZM,
    ZPANIC,
}

impl Game {
    pub fn mod_folder(&self) -> &Path {
        match self {
            Game::AG2 => todo!(),
            Game::ALIENSWARM => todo!(),
            Game::AOC => todo!(),
            Game::BG2 => todo!(),
            Game::BMS => todo!(),
            Game::CSPROMOD => todo!(),
            Game::CSTRIKE => todo!(),
            Game::DINODDAY => todo!(),
            Game::DOD => todo!(),
            Game::DOI => todo!(),
            Game::DYSTOPIA => todo!(),
            Game::EMPIRES => todo!(),
            Game::ESMOD => todo!(),
            Game::FAS => todo!(),
            Game::FF => todo!(),
            Game::FOF => todo!(),
            Game::GESOURCE => todo!(),
            Game::GMOD9 => todo!(),
            Game::HIDDEN => todo!(),
            Game::HL1MP => todo!(),
            Game::HL2CTF => todo!(),
            Game::HL2MP => todo!(),
            Game::INSURGENCY => todo!(),
            Game::IOS => todo!(),
            Game::KZ => todo!(),
            Game::LEFT4DEAD2 => todo!(),
            Game::MODULARCOMBAT => todo!(),
            Game::NEOTOKYO => todo!(),
            Game::NMRIH => todo!(),
            Game::NUCLEARDAWN => todo!(),
            Game::OBSIDIAN => todo!(),
            Game::OPENFORTRESS => todo!(),
            Game::PF2 => todo!(),
            Game::PVKILL => todo!(),
            Game::REACTIVEDROP => todo!(),
            Game::MKBETA => todo!(),
            Game::SHIP => todo!(),
            Game::SOURCEFORTS => todo!(),
            Game::SYNERGY => todo!(),
            Game::TF => Path::new("tf"),
            Game::TF2CLASSIC => todo!(),
            Game::TF2CLASSIFIED => todo!(),
            Game::TREASON => todo!(),
            Game::ZM => todo!(),
            Game::ZPANIC => todo!(),
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Game::AG2 => todo!(),
            Game::ALIENSWARM => todo!(),
            Game::AOC => todo!(),
            Game::BG2 => todo!(),
            Game::BMS => todo!(),
            Game::CSPROMOD => todo!(),
            Game::CSTRIKE => todo!(),
            Game::DINODDAY => todo!(),
            Game::DOD => todo!(),
            Game::DOI => todo!(),
            Game::DYSTOPIA => todo!(),
            Game::EMPIRES => todo!(),
            Game::ESMOD => todo!(),
            Game::FAS => todo!(),
            Game::FF => todo!(),
            Game::FOF => todo!(),
            Game::GESOURCE => todo!(),
            Game::GMOD9 => todo!(),
            Game::HIDDEN => todo!(),
            Game::HL1MP => todo!(),
            Game::HL2CTF => todo!(),
            Game::HL2MP => todo!(),
            Game::INSURGENCY => todo!(),
            Game::IOS => todo!(),
            Game::KZ => todo!(),
            Game::LEFT4DEAD2 => todo!(),
            Game::MODULARCOMBAT => todo!(),
            Game::NEOTOKYO => todo!(),
            Game::NMRIH => todo!(),
            Game::NUCLEARDAWN => todo!(),
            Game::OBSIDIAN => todo!(),
            Game::OPENFORTRESS => todo!(),
            Game::PF2 => todo!(),
            Game::PVKILL => todo!(),
            Game::REACTIVEDROP => todo!(),
            Game::MKBETA => todo!(),
            Game::SHIP => todo!(),
            Game::SOURCEFORTS => todo!(),
            Game::SYNERGY => todo!(),
            Game::TF => write!(f, "Team Fortress 2"),
            Game::TF2CLASSIC => todo!(),
            Game::TF2CLASSIFIED => todo!(),
            Game::TREASON => todo!(),
            Game::ZM => todo!(),
            Game::ZPANIC => todo!(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Package {
    pub game: Game,
    pub branch: sdk::Branch,
    pub plugins: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_startup_script: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_opts: Option<templates::StartSh>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub templates: Option<TemplateSet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_configs: Option<Vec<SimpleConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_configs: Option<Vec<SimpleConfig>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleConfig {
    pub path: PathBuf,
    pub options: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct TemplateSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sourcemod_cfg: Option<templates::SourcemodCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maplists_cfg: Option<templates::MaplistsCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub databases_cfg: Option<templates::DatabasesCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core_cfg: Option<templates::CoreCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admins_simple_ini: Option<templates::AdminsSimpleIni>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admins_cfg: Option<templates::AdminsCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_groups_cfg: Option<templates::AdminGroupsCfg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_overrides_cfg: Option<templates::AdminOverridesCfg>,
}

/// Manager is responsible for loading and managing a project using its package configuration file, sm-pkg.yaml.
pub struct Project<'p> {
    /// Root directory of the project.
    project_root: &'p Path,
    pub package: Option<Package>,
    repo: &'p repo::LocalRepo<'p>,
}

impl<'p> Project<'p> {
    pub fn new(project_root: &'p &Path, repo: &'p repo::LocalRepo) -> BoxResult<Self> {
        println!("🏗️ Using project root {:?}", project_root);
        Ok(Project {
            project_root,
            repo,
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
        self.project_root.join(PROJECT_FILE)
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

    pub fn remove_plugin(&mut self, plugin: plugins::Definition) -> BoxResult {
        if !self.has_plugin(&plugin.name) {
            return Err("❗ Plugin doesnt exists in project".into());
        }

        match &mut self.package {
            Some(config) => {
                config.plugins.retain(|p| p != &plugin.name);
                println!("✅ Plugin Removed {}", plugin.name);
                Ok(())
            }
            None => Err("❗ No plugin found?".into()),
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
                description: None,
                raw_configs: None,
                create_startup_script: None,
                startup_opts: None,
                plugin_configs: None,
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

        for plugin in &pkg.plugins {
            let def = self.repo.find_plugin_definition(plugin)?;
            self.write_plugin_config(&def)?;
        }

        Ok(())
    }

    fn write_plugin_config(&self, def: &plugins::Definition) -> BoxResult {
        let mut base_configs = match &def.configs {
            Some(configs) => configs.clone(),
            None => return Ok(()),
        };

        let local_configs = match &self.package {
            Some(p) => match &p.plugin_configs {
                None => &Vec::new(),
                Some(configs) => configs,
            },
            None => return Ok(()),
        };

        for base_config in base_configs.iter_mut() {
            if let Some(local_config) = local_configs.iter().find(|c| c.path == base_config.path) {
                for (k, v) in local_config.options.iter() {
                    base_config.options.insert(k.clone(), v.clone());
                }
            }

            self.write_raw_configs(&vec![base_config.clone()])?;
        }

        Ok(())
    }

    fn write_startup_script(&self, config: &Option<templates::StartSh>) -> BoxResult {
        let script_path = self.project_root.join("start.sh");
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
            let out_path = self.project_root.join(&raw_config.path);
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
                &self.project_root.join("tf/cfg/sourcemod/sourcemod.cfg"),
                template,
            )?;
        }

        Ok(())
    }

    fn write_core_cfg(&self, config: &Option<templates::CoreCfg>) -> BoxResult {
        if let Some(template) = &config {
            write_cfg(
                &TagFormat::Ini,
                &self
                    .project_root
                    .join("tf/addons/sourcemod/configs/core.cfg"),
                template,
            )?;
        }

        Ok(())
    }

    fn write_databases_cfg(&self, config: &Option<templates::DatabasesCfg>) -> BoxResult {
        if let Some(template) = &config {
            write_cfg(
                &TagFormat::Ini,
                &self
                    .project_root
                    .join("tf/addons/sourcemod/configs/databases.cfg"),
                template,
            )?
        }

        Ok(())
    }

    fn write_maplists_cfg(&self, config: &Option<templates::MaplistsCfg>) -> BoxResult {
        if let Some(template) = &config {
            write_cfg(
                &TagFormat::Ini,
                &self
                    .project_root
                    .join("tf/addons/sourcemod/configs/maplists.cfg"),
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
                    .project_root
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
                &self
                    .project_root
                    .join("tf/addons/sourcemod/configs/admins.cfg"),
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
                    .project_root
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
                    .project_root
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
