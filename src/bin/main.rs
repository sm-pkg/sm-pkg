#![feature(str_as_str)]

use clap::{Parser, Subcommand};
use resolve_path::PathResolveExt;
use smpkg::{
    compiler, plugins,
    repo::{self, Repository},
    sdk::Manager,
};
use std::path::{Path, PathBuf};

const DEFAULT_ROOT: &str = "~/.smpkg";
const DEFAULT_BRANCH: &str = "1.12";
const DEFAULT_OUTPUT: &str = "./output";
const UPDATE_URL: &str =
    "https://raw.githubusercontent.com/leighmacdonald/smpkg-repo/refs/heads/master/index.json";

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = DEFAULT_ROOT)]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Install one or more packages", arg_required_else_help = true)]
    Install {
        #[arg(short,long, default_value = DEFAULT_OUTPUT)]
        target: PathBuf,
        #[arg(required = true)]
        plugins: Vec<String>,
    },

    #[command(about = "Search package cache", arg_required_else_help = true)]
    Search {
        #[arg(required = true)]
        query: String,
    },

    #[command(about = "Update package cache")]
    Update {},

    #[command(about = "Build a plugin")]
    Build {
        #[arg(required = true)]
        plugin: String,
    },

    #[command(about = "Download and install sourcemod")]
    SDKInstall {
        #[arg(default_value = DEFAULT_BRANCH)]
        branch: String,
    },

    #[command(about = "List installed sourcemod versions")]
    SDKList {},

    #[command(about = "Fetches the latest version of sourcemod for a branch")]
    SDKLatest {
        #[arg(default_value = DEFAULT_BRANCH)]
        branch: String,
    },
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 smpkg - sourcemod package manager");
    let args = Cli::parse();
    let root = args.root.expect("No root path specified");
    let root_path = root.try_resolve()?;

    println!("🪸 Using root: {:?}", root_path);

    if !root_path.exists() {
        std::fs::create_dir_all(&root_path)?;
    }

    match args.command {
        Commands::SDKInstall { branch } => sdk_install(&root_path, branch).await,
        Commands::SDKLatest { branch } => sdk_latest(&root_path, branch).await,
        Commands::SDKList {} => sdk_list(&root_path).await,
        Commands::Install { target, plugins } => install(&root_path, target, plugins).await,
        Commands::Search { query } => search(&root_path, query).await,
        Commands::Update {} => update(&root_path).await,
        Commands::Build { plugin } => build(&root_path, plugin).await,
    }
}

async fn build(root_path: &Path, plugin: String) -> Result<(), Box<dyn std::error::Error>> {
    let args = compiler::CompilerArgs::default();
    let repo = Repository::new(root_path, UPDATE_URL);
    let plugin_def = repo.find_plugin_definition(&plugin)?;
    compiler::compile(&args, &plugin_def)?;
    Ok(())
}

async fn install(
    root_path: &Path,
    target: PathBuf,
    plugins: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::new(root_path, UPDATE_URL);

    plugins::install(repo, &target, plugins)?;
    Ok(())
}

async fn search(root_path: &Path, query: String) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::new(root_path, UPDATE_URL);
    let matches: Vec<repo::PluginDefinition> = repo.search(&query)?;
    matches
        .into_iter()
        .for_each(|p| println!("{} - {} - {}", p.name, p.version, p.description));
    Ok(())
}

async fn update(root_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let repo = repo::Repository::new(root_path, UPDATE_URL);
    repo.update().await?;
    println!("Updated local package cache");
    Ok(())
}

async fn sdk_list(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let sdk = Manager::new(root);
    println!("🛠️  Currently installed sourcemod SDKs:\n");
    let sdks = sdk.get_installed_sdks();
    for sdk in sdks {
        println!("🏷️  {}", sdk);
    }
    Ok(())
}

async fn sdk_latest(root: &Path, branch: String) -> Result<(), Box<dyn std::error::Error>> {
    let result = Manager::new(root).fetch_latest_version(&branch).await;
    match result {
        Ok(version) => {
            println!("🕓 Latest version: {version}");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

async fn sdk_install(root: &Path, branch: String) -> Result<(), Box<dyn std::error::Error>> {
    let sdk = Manager::new(root);
    sdk.fetch_version(branch).await
}
