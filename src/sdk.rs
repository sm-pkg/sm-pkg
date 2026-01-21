use flate2::read::GzDecoder;
use reqwest::Error;
use serde::Serialize;
use std::{
    fmt::Display,
    fs::{File, create_dir_all, remove_file},
    io::Write,
    os::unix::fs,
    path::{self, PathBuf},
};
use tar::Archive;

const DL_CACHE: &str = "dl_cache";

#[derive(clap::ValueEnum, Clone, Debug, Serialize, Default)]
pub enum Runtime {
    #[default]
    Sourcemod,
    Metamod,
}

#[derive(clap::ValueEnum, Clone, Debug, Serialize, Default)]
pub enum Branch {
    #[default]
    Stable,
    Dev,
}

impl Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Branch::Stable => write!(f, "1.12"),
            Branch::Dev => write!(f, "1.13"),
        }
    }
}

pub struct Manager<'a> {
    /// The root directory 
    root: &'a PathBuf,
}

impl<'a> Manager<'a> {
    pub fn new(root: &'a PathBuf) -> Self {
        Manager { root }
    }

    pub async fn install_game_dir(
        &self,
        runtime: &Runtime,
        branch: &Branch,
        game_dir: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match runtime {
            Runtime::Sourcemod => self.install_sourcemod(branch, &game_dir).await,
            Runtime::Metamod => self.install_metamod(branch, &game_dir).await,
        }
    }

    pub async fn install_sdk(
        &self,
        runtime: &Runtime,
        branch: &Branch,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let out_path = self.root.join(format!("sdks/sourcemod-{}", branch));
        let cache_path = self.root.join(DL_CACHE);
        if !cache_path.exists() {
            create_dir_all(&cache_path)?;
        }
        match runtime {
            Runtime::Sourcemod => self.install_sourcemod(branch, &out_path).await,
            Runtime::Metamod => self.install_metamod(branch, &out_path).await,
        }
    }

    pub async fn fetch_latest_sourcemod_version(&self, branch: &Branch) -> Result<String, Error> {
        let target = format!("https://sm.alliedmods.net/smdrop/{branch}/sourcemod-latest-linux");
        reqwest::get(target).await?.text().await
    }

    pub async fn fetch_latest_metamod_version(&self, branch: &Branch) -> Result<String, Error> {
        let target = format!("https://mms.alliedmods.net/mmsdrop/{branch}/mmsource-latest-linux");
        reqwest::get(target).await?.text().await
    }

    async fn fetch_archive(
        &self,
        url: String,
        of: &mut File,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let body = reqwest::get(url).await?.bytes().await?;
        of.write_all(&body[..])?;
        Ok(())
    }

    async fn install_sourcemod(
        &self,
        branch: &Branch,
        target_dir: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("⏳ Fetching latest version... ");
        let latest_version = Self::fetch_latest_sourcemod_version(self, &branch).await?;
        println!("🔎 Found: {latest_version}");
        let archive_path = self.root.join(DL_CACHE).join(&latest_version);
        if !archive_path.exists() {
            let target = format!(
                "https://sm.alliedmods.net/smdrop/{branch}/{}",
                &latest_version
            );
            println!("💾 Downlading sdk: {target}...");
            let mut of = File::create(&archive_path)?;
            self.fetch_archive(target, &mut of).await?;
        }

        self.extract_archive(&archive_path, &target_dir)?;
        self.activate_sdk(branch)?;

        Ok(())
    }

    async fn install_metamod(
        &self,
        branch: &Branch,
        target_dir: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("⏳ Fetching latest version... ");
        let version = Self::fetch_latest_sourcemod_version(self, branch).await?;
        println!("🔎 Found: {version}");
        let target = format!("https://sm.alliedmods.net/smdrop/{branch}/{version}");
        println!("💾 Downlading sdk: {target}");
        let archive_path = self.root.join(DL_CACHE).join(version);
        let mut of = File::create(&archive_path)?;
        self.fetch_archive(target, &mut of).await?;

        self.extract_archive(&archive_path, &target_dir)?;

        Ok(())
    }

    fn extract_archive(
        &self,
        archive_path: &PathBuf,
        out_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("📤 Extracting into: {:?}...", out_path);
        let input_archive = File::open(archive_path)?;
        let mut archive = Archive::new(GzDecoder::new(&input_archive));
        archive.unpack(out_path)?;
        Ok(())
    }

    pub fn get_installed_sdks(&self) -> Vec<String> {
        let mut sdks = Vec::new();
        if let Ok(entries) = std::fs::read_dir(self.root.join("sdks")) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("sourcemod-") {
                        sdks.push(name.to_string());
                    }
                }
            }
        }
        sdks
    }

    pub fn activate_sdk(&self, branch: &Branch) -> Result<(), Box<dyn std::error::Error>> {
        let wanted = self.root.join(format!("sdks/sourcemod-{branch}"));
        let sdks = self.get_installed_sdks();
        if sdks.is_empty() {
            Err("No SDKs installed, try: sourcemod install".into())
        } else {
            let wanted_sdk = sdks
                .iter()
                .find(|p| wanted == self.root.join("sdks").join(path::Path::new(p)));
            match wanted_sdk {
                Some(latest_sdk) => {
                    let sm_root = self.root.join("sdks").join(path::Path::new(latest_sdk));
                    let current_root = self.root.join("sdks/current");
                    println!("⭐ Activating {latest_sdk} @ {current_root:?}");

                    if current_root.exists() {
                        remove_file(&current_root)?;
                    }

                    fs::symlink(sm_root, &current_root)?;
                    println!("✅ SDK activated successfully");
                    println!(
                        "🚨 You probably want to add {:?} to your $PATH if you have not already",
                        current_root.join("addons/sourcemod/scripting")
                    );
                    Ok(())
                }
                None => Err("❗❗❗ No SDK found for branch".into()),
            }
        }
    }
}
