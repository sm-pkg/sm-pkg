use flate2::read::GzDecoder;
use reqwest::Error;
use std::{fs::remove_file, os::unix::fs, path::Path};
use tar::Archive;

pub struct Manager<'a> {
    root: &'a Path,
}

impl<'a> Manager<'a> {
    pub fn new(root: &'a Path) -> Self {
        Manager { root }
    }

    pub async fn fetch_latest_version(&self, branch: &String) -> Result<String, Error> {
        let target = format!("https://sm.alliedmods.net/smdrop/{branch}/sourcemod-latest-linux");
        reqwest::get(target).await?.text().await
    }

    pub async fn fetch_version(&self, branch: String) -> Result<(), Box<dyn std::error::Error>> {
        println!("⏳ Fetching latest version... ");
        let version = Self::fetch_latest_version(self, &branch).await?;
        println!("🔎 Found: {version}");
        let target = format!("https://sm.alliedmods.net/smdrop/{branch}/{version}");
        println!("💾 Downlading sdk: {target}");
        let body = reqwest::get(target).await?.bytes().await?;
        let mut archive = Archive::new(GzDecoder::new(&body[..]));
        let out_path = self.root.join(format!("sdks/sourcemod-{}", branch));
        println!("📤 Extracting into: {:?}...", out_path);
        archive.unpack(out_path)?;

        self.activate_sdk(branch)?;

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

    pub fn activate_sdk(&self, branch: String) -> Result<(), Box<dyn std::error::Error>> {
        let wanted = self.root.join(format!("sdks/sourcemod-{branch}"));
        let sdks = self.get_installed_sdks();
        if sdks.is_empty() {
            Err("No SDKs installed, try: sourcemod install".into())
        } else {
            let wanted_sdk = sdks
                .iter()
                .find(|p| wanted == self.root.join("sdks").join(Path::new(p)));
            match wanted_sdk {
                Some(latest_sdk) => {
                    let sm_root = self.root.join("sdks").join(Path::new(latest_sdk));
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
