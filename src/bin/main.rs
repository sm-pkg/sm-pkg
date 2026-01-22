use clap::{Parser, Subcommand};
use resolve_path::PathResolveExt;
use sm_pkg::{
    fsutil, plugins, project, repo,
    sdk::{self, Branch, Runtime},
};
use std::path::{Path, PathBuf};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_ROOT: &str = "~/.sm-pkg";
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
    #[command(about = "Install all project dependencies")]
    Install {
        #[arg(short, long, default_value = ".")]
        project_root: PathBuf,
    },
    #[command(about = "Add one or more plugins to a project")]
    Add {
        #[arg(short, long, default_value = ".")]
        project_root: PathBuf,

        #[arg(required = true)]
        plugins: Vec<String>,
    },
    #[command(about = "Remove one or more plugins from a project")]
    Remove {
        #[arg(short, long, default_value = ".")]
        project_root: PathBuf,

        #[arg(required = true)]
        plugins: Vec<String>,
    },
    #[command(about = "List configured project pacakges")]
    List {
        #[arg(short, long, default_value = ".")]
        project_root: PathBuf,
    },
    #[command(about = "Search package cache", arg_required_else_help = true)]
    Search {
        #[arg(required = true)]
        query: String,
    },

    #[command(about = "Update package cache")]
    Update {},

    #[command(about = "Download and install sourcemod")]
    SDKInstall {
        #[arg(short, long, default_value_t, value_enum)]
        branch: Branch,

        #[arg(short, long, default_value_t, value_enum)]
        runtime: Runtime,
    },

    #[command(about = "List installed sourcemod versions")]
    SDKList {},

    #[command(about = "Fetches the latest version of sourcemod for a branch")]
    SDKLatest {
        #[arg(short, long, default_value_t, value_enum)]
        branch: Branch,

        #[arg(short, long, default_value_t, value_enum)]
        runtime: Runtime,
    },
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 sm-pkg - sourcemod package manager - {}", VERSION);
    let args = Cli::parse();
    let app_root = args.app_root.expect("No app_root path specified");
    let app_root_resolved = app_root.try_resolve()?.to_path_buf();

    println!("🪸 Using app root: {:?}", app_root_resolved);

    if !app_root_resolved.exists() {
        std::fs::create_dir_all(&app_root_resolved)?;
    }

    match args.command {
        Commands::SDKInstall { branch, runtime } => {
            sdk_install(&app_root_resolved, &runtime, &branch).await
        }
        Commands::SDKLatest { branch, runtime } => {
            sdk_latest(&app_root_resolved, &runtime, &branch).await
        }
        Commands::SDKList {} => sdk_list(&app_root_resolved).await,
        Commands::Search { query } => search(&app_root_resolved, query).await,
        Commands::Update {} => update(&app_root_resolved).await,
        Commands::Init { project_root } => project_init(&project_root).await,
        Commands::Add {
            plugins,
            project_root,
        } => plugin_add(&app_root_resolved, &project_root, plugins).await,
        Commands::Remove {
            plugins,
            project_root,
        } => package_remove(&app_root_resolved, &project_root, plugins).await,
        Commands::List { project_root } => package_list(&app_root_resolved, &project_root).await,
        Commands::Install { project_root } => {
            package_install(&app_root_resolved, &project_root).await
        }
    }
}

async fn plugin_add(
    app_root: &PathBuf,
    project_root: &PathBuf,
    plugins: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = repo::Repository::new(app_root, UPDATE_URL);
    let mut project_manager = project::Manager::new(project_root.clone());
    project_manager.open_or_new()?;

    for plugin in plugins {
        let plugin_def = repo.find_plugin_definition(&plugin)?;
        project_manager.add_plugin(plugin_def)?;
    }

    project_manager.save_package_config()?;

    Ok(())
}

async fn package_list(
    _app_root: &Path,
    project_root: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pm = project::Manager::new(project_root.clone());
    pm.open()?;
    match pm.package {
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

async fn package_remove(
    _app_root: &Path,
    _project_root: &PathBuf,
    _plugins: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

async fn project_init(project_root: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut project_manager = project::Manager::new(project_root.to_path_buf());
    project_manager.open_or_new()?;

    Ok(())
}
async fn package_install(
    app_root: &PathBuf,
    project_root: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut project_manager = project::Manager::new(project_root.clone());
    project_manager.open()?;
    let repo = repo::Repository::new(&app_root, UPDATE_URL);
    let package = project_manager.package.expect("No package found?");
    let sdk_manager = sdk::Manager::new(&app_root);
    let sdk_env = sdk_manager.get_sdk_env(&package.branch)?;
    let outputs = plugins::build(&app_root, &sdk_env, &repo, &package.plugins)?;

    let mod_folder = project_root.join(&package.game.mod_folder());
    if !mod_folder.exists() {
        return Err(format!("Mod folder does not exist: {}", mod_folder.display()).into());
    }
    sdk_manager
        .install_metamod(&package.branch, &mod_folder)
        .await?;
    sdk_manager
        .install_sourcemod(&package.branch, &mod_folder)
        .await?;

    let sm_root = mod_folder.join("addons").join("sourcemod");

    for build_root in outputs {
        println!("📀 Installing {:?} -> {:?}", &build_root, &sm_root);
        fsutil::copy_dir_all(&build_root, &sm_root)?;
    }

    // plugins::install(&app_root, &sdk_env, &repo, &sm_root, package.plugins)?;

    //project_manager.install().await?;
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

async fn sdk_list(root: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let sdk = sdk::Manager::new(&root);
    println!("🛠️  Currently installed sourcemod SDKs:\n");
    let sdks = sdk.get_installed_sdks();
    for sdk in sdks {
        println!("🏷️  {}", sdk);
    }
    Ok(())
}

async fn sdk_latest(
    root: &PathBuf,
    runtime: &Runtime,
    branch: &Branch,
) -> Result<(), Box<dyn std::error::Error>> {
    let manager = sdk::Manager::new(&root);
    let version = match runtime {
        Runtime::Metamod => manager.fetch_latest_metamod_build(&branch).await?,
        Runtime::Sourcemod => manager.fetch_latest_sourcemod_build(&branch).await?,
    };

    println!("🕓 Latest version: {version}");
    Ok(())
}

async fn sdk_install(
    root: &PathBuf,
    runtime: &Runtime,
    branch: &Branch,
) -> Result<(), Box<dyn std::error::Error>> {
    sdk::Manager::new(&root).install_sdk(runtime, branch).await
}
