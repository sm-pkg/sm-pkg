use crate::{
    BoxResult, plugins,
    sdk::Branch,
    tmpl::{self, FileConfig},
};
use inquire::{InquireError, Select};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::File,
    io::Write,
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
    pub branch: Branch,
    pub plugins: Vec<String>,
    pub configs: Option<Vec<tmpl::FileConfig>>,
}

pub struct Manager {
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
                configs: None,
            }),
            Err(_) => return Err("❗ Failed to select a game".into()),
        };
        self.save_package_config()?;
        Ok(())
    }

    pub fn write_configs(&self) -> BoxResult {
        let configs = match &self.package {
            Some(config) => match &config.configs {
                Some(configs) => configs,
                None => &Vec::new(),
            },
            None => return Err("❗ No config?".into()),
        };

        for file_config in configs {
            self.handle_template(&file_config)?;
        }

        Ok(())
    }

    fn handle_template(&self, file_config: &FileConfig) -> BoxResult {
        let out_path = self.root.join(&file_config.path);
        println!("Outpath: {}", out_path.to_str().unwrap());
        let mut output_file = File::create(out_path)?;
        match file_config.format {
            tmpl::Format::CFG => self.handle_template_cfg(file_config, &mut output_file),
            tmpl::Format::KV => self.handle_template_kv(file_config, &mut output_file),
            tmpl::Format::TEMPLATE => self.handle_template_template(file_config, &mut output_file),
        }
    }

    fn handle_template_cfg(&self, fc: &FileConfig, output_file: &mut File) -> BoxResult {
        // Write out raw section first, explicit options should override anything in there.
        match &fc.raw {
            Some(content) => output_file.write_all(content.as_bytes())?,
            None => (),
        };

        match &fc.options {
            Some(v) => {
                for (key, value) in v {
                    write!(output_file, "{} \"{}\"\n", key, value)?;
                }
                Ok(())
            }
            None => Ok(()),
        }
    }

    fn handle_template_kv(&self, _fc: &FileConfig, _output_file: &mut File) -> BoxResult {
        Ok(())
    }

    fn handle_template_template(&self, fc: &FileConfig, _output_file: &mut File) -> BoxResult {
        let mut out_path = fc.path.clone();
        out_path.add_extension("jinja2");
        println!("Template: {:?}", out_path);
        let out_path_buf = out_path.to_path_buf();
        let _template_path = match out_path_buf.to_str() {
            None => return Err("invalid  template path".into()),
            Some(p) => p,
        };
        // println!("Template Path: {}", template_path);
        // let tmpl = match self.env.get_template(template_path) {
        //     Err(_) => return Err("Failed to load template".into()),
        //     Ok(t) => t,
        // };

        // println!("{:?}", fc.options);

        // tmpl.render_to_write(fc.options.clone().unwrap(), output_file)?;

        // println!(
        //     "💾 Wrote template: {}",
        //     fc.path.to_str().unwrap_or("unknown")
        // );

        Ok(())
    }
}
