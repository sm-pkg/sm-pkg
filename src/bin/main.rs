// #![feature(str_as_str)]

use clap::{Parser, Subcommand};
use resolve_path::PathResolveExt;
use sm_pkg::{plugins, project, repo, sdk};
use std::path::{Path, PathBuf};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_ROOT: &str = "~/.smpkg";
const DEFAULT_BRANCH: &str = "1.12";
//const DEFAULT_OUTPUT: &str = "./output";
const UPDATE_URL: &str =
    "https://raw.githubusercontent.com/sm-pkg/plugins/refs/heads/master/index.json";

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = DEFAULT_ROOT)]
    app_root: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Initialize a new project")]
    Init {
        #[arg(short, long, default_value = ".")]
        project_root: PathBuf,
    },
    #[command(about = "Add one or more plugins to a project")]
    Add {
        #[arg(required = true)]
        plugins: Vec<String>,
    },
    #[command(about = "Remove one or more plugins from a project")]
    Remove {
        #[arg(required = true)]
        plugins: Vec<String>,
    },
    #[command(about = "List configured project pacakges")]
    List {},
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
    println!("📦 smpkg - sourcemod package manager - {}", VERSION);
    let args = Cli::parse();
    let app_root = args.app_root.expect("No app_root path specified");
    let app_root_resolved = app_root.try_resolve()?.to_path_buf();

    println!("🪸 Using app root: {:?}", app_root_resolved);

    if !app_root_resolved.exists() {
        std::fs::create_dir_all(&app_root_resolved)?;
    }

    match args.command {
        Commands::SDKInstall { branch } => sdk_install(&app_root_resolved, branch).await,
        Commands::SDKLatest { branch } => sdk_latest(&app_root_resolved, branch).await,
        Commands::SDKList {} => sdk_list(&app_root_resolved).await,
        Commands::Search { query } => search(&app_root_resolved, query).await,
        Commands::Update {} => update(&app_root_resolved).await,
        Commands::Build { plugin } => build(&app_root_resolved, plugin).await,
        Commands::Init { project_root } => project_init(&project_root).await,
        Commands::Add { plugins } => plugin_add(&app_root_resolved, plugins).await,
        Commands::Remove { plugins } => plugin_remove(&app_root_resolved, plugins).await,
        Commands::List {} => plugin_list(&app_root_resolved).await,
    }
}

async fn plugin_add(
    app_root: &PathBuf,
    plugins: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = repo::Repository::new(app_root, UPDATE_URL);
    let mut project_manager = project::Manager::new(app_root.to_path_buf());
    project_manager.open_or_new()?;

    for plugin in plugins {
        let plugin_def = repo.find_plugin_definition(&plugin)?;
        project_manager.add_plugin(plugin_def)?;
    }

    project_manager.save_package_config()?;

    Ok(())
}

async fn plugin_list(app_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut pm = project::Manager::new(app_root.to_path_buf());
    pm.open()?;
    match pm.config {
        None => return Err("❗ No package config found".into()),
        Some(config) => {
            println!("📋 Plugins added to package:");
            match config.plugins.is_empty() {
                true => println!("🤷‍♂️ No plugins added"),
                false => {
                    for plugin in config.plugins {
                        println!("- {}", plugin);
                    }
                }
            }
        }
    }
    Ok(())
}

async fn plugin_remove(
    _app_root: &Path,
    _plugins: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

async fn project_init(project_root: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut project_manager = project::Manager::new(project_root.to_path_buf());
    project_manager.open_or_new()?;

    Ok(())
}

async fn build(root_path: &PathBuf, plugin: String) -> Result<(), Box<dyn std::error::Error>> {
    let repo = repo::Repository::new(root_path, UPDATE_URL);
    let plugin_def = repo.find_plugin_definition(&plugin)?;
    let builder = plugins::Builder::new(root_path.clone());
    builder.build(plugin_def)?;

    Ok(())
}

async fn search(root_path: &Path, query: String) -> Result<(), Box<dyn std::error::Error>> {
    let repo = repo::Repository::new(root_path, UPDATE_URL);
    let matches: Vec<plugins::Definition> = repo.search(&query)?;
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
    let sdk = sdk::Manager::new(root);
    println!("🛠️  Currently installed sourcemod SDKs:\n");
    let sdks = sdk.get_installed_sdks();
    for sdk in sdks {
        println!("🏷️  {}", sdk);
    }
    Ok(())
}

async fn sdk_latest(root: &Path, branch: String) -> Result<(), Box<dyn std::error::Error>> {
    let result = sdk::Manager::new(root).fetch_latest_version(&branch).await;
    match result {
        Ok(version) => {
            println!("🕓 Latest version: {version}");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

async fn sdk_install(root: &Path, branch: String) -> Result<(), Box<dyn std::error::Error>> {
    let sdk = sdk::Manager::new(root);
    sdk.fetch_version(branch).await
}
