use std::path::Path;

use clap::ArgMatches;

use crate::{CommandHandler, sdk::Manager};

pub struct SDKVersion {}

impl CommandHandler for SDKVersion {
    async fn execute(
        &self,
        root: &Path,
        latest_version_matches: &ArgMatches,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let branch = latest_version_matches
            .get_one::<String>("branch")
            .expect("Invalid branch");
        let sdk = Manager::new(root);
        let result = sdk.fetch_latest_version(branch).await;
        match result {
            Ok(version) => {
                println!("Latest version: {version}");
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
}

pub struct SDKInstaller {}

impl CommandHandler for SDKInstaller {
    async fn execute(
        &self,
        root: &Path,
        latest_version_matches: &ArgMatches,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let branch = latest_version_matches
            .get_one::<String>("branch")
            .expect("Invalid branch")
            .clone();
        let sdk = Manager::new(root);
        sdk.fetch_version(branch).await
    }
}
