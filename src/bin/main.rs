#![feature(str_as_str)]

use clap::{ArgMatches, Command, arg};
use resolve_path::PathResolveExt;
use smpkg::CommandHandler;
use smpkg::sdk::Manager;
use smpkg::sdk::commands::{SDKInstaller, SDKVersion};
use std::path::Path;
use tokio;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = "~/.smpkg".try_resolve()?;
    let root_path = root.as_path();
    run(root_path).await
}

pub async fn run(root_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !root_path.exists() {
        std::fs::create_dir_all(root_path)?;
    }

    if let Some(("sourcemod", sourcemod_matches)) = commands().subcommand() {
        let subcommand = sourcemod_matches
            .subcommand()
            .unwrap_or(("latest-version", sourcemod_matches));

        match subcommand {
            ("latest-version", sourcemod_matches) => {
                SDKVersion {}.execute(root_path, sourcemod_matches).await
            }
            ("install", sourcemod_matches) => {
                SDKInstaller {}.execute(root_path, sourcemod_matches).await
            }
            ("ls", _) => sourcemo_list_handler(root_path).await,
            (_, _) => Ok(()),
        }
    } else {
        Ok(())
    }
}

pub fn commands() -> ArgMatches {
    return Command::new("smpkg")
        .about("A simple package manager")
        // .version(crate_version!)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(commands_sourcemod())
        .get_matches();
}

pub fn sourcemod_latest_version() -> Command {
    Command::new("latest-version")
        .short_flag('v')
        .about("Fetches the latest version of Sourcemod")
        .arg(arg!(<branch> "The remote branch"))
}

pub fn sourcemod_install() -> Command {
    Command::new("install")
        .short_flag('i')
        .about("Download and install sourcemod")
        .arg(arg!(<branch> "The remote branch"))
}

pub fn sourcemo_list() -> Command {
    Command::new("ls").about("List installed sourcemod versions")
}

pub fn commands_sourcemod() -> Command {
    Command::new("sourcemod")
        .about("Sourcemod distribution management commands")
        .alias("sm")
        .arg_required_else_help(true)
        .subcommand(sourcemod_latest_version())
        .subcommand(sourcemod_install())
}

async fn sourcemo_list_handler(root: &Path) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let sdk = Manager::new(root);
    println!("🛠️ Currently installed sourcemod SDKs:");
    let sdks = sdk.get_installed_sdks();
    for sdk in sdks {
        println!("{}", sdk);
    }
    Ok(())
}
