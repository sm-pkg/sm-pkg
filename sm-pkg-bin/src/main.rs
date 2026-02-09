use clap::{CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete;
use resolve_path::PathResolveExt;
use sm_pkg::{
    BoxResult, DEFAULT_ROOT, VERSION, fsutil,
    plugins::{self, create_build_root},
    project,
    repo::{self},
    sdk::{self, Branch, Runtime},
};
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

//const DEFAULT_OUTPUT: &str = "./output";

#[derive(Parser, Debug)]
#[command(name = "completion-derive")]
struct Cli {
    #[arg(short, long, default_value = DEFAULT_ROOT)]
    app_root: Option<PathBuf>,

    #[arg(long = "generate", value_enum)]
    generator: Option<clap_complete::Shell>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Initialize a new project")]
    Init {
        #[arg(short, long, default_value = ".", value_hint = ValueHint::DirPath)]
        project_root: PathBuf,
    },
    #[command(about = "Install all project dependencies")]
    Install {
        #[arg(short, long, default_value = ".", value_hint = ValueHint::DirPath)]
        project_root: PathBuf,
    },
    #[command(about = "Add one or more plugins to a project")]
    Add {
        #[arg(short, long, default_value = ".", value_hint = ValueHint::DirPath)]
        project_root: PathBuf,

        #[arg(required = true, value_hint = ValueHint::Unknown)]
        plugins: Vec<String>,
    },
    #[command(about = "Remove one or more plugins from a project")]
    Remove {
        #[arg(short, long, default_value = ".", value_hint = ValueHint::DirPath)]
        project_root: PathBuf,

        #[arg(required = true, value_hint = ValueHint::Unknown)]
        plugins: Vec<String>,
    },

    #[command(about = "Generate configuration files")]
    Config {
        #[arg(short, long, default_value = ".", value_hint = ValueHint::DirPath)]
        project_root: PathBuf,
    },

    #[command(about = "List configured project pacakges")]
    List {
        #[arg(short, long, default_value = ".", value_hint = ValueHint::DirPath)]
        project_root: PathBuf,
    },
    #[command(about = "Search package cache", arg_required_else_help = true)]
    Search {
        #[arg(required = true, value_hint = ValueHint::Unknown)]
        query: String,
    },

    #[command(about = "Build one or more plugins", arg_required_else_help = true)]
    Build {
        #[arg(required = true, value_hint = ValueHint::Unknown)]
        plugins: Vec<String>,

        #[arg(short, long, default_value_t, value_enum, value_hint = ValueHint::Unknown)]
        branch: Branch,

        #[arg(short('r'), long, value_hint = ValueHint::DirPath)]
        build_root: Option<PathBuf>,
    },

    #[command(about = "Update package cache")]
    Update {},

    #[command(about = "Download and install sourcemod")]
    SDKInstall {
        #[arg(short, long, default_value_t, value_enum, value_hint = ValueHint::Other)]
        branch: Branch,

        #[arg(short, long, default_value_t, value_enum, value_hint = ValueHint::Other)]
        runtime: Runtime,
    },

    #[command(about = "List installed sourcemod versions")]
    SDKList {},

    #[command(about = "Fetches the latest version of sourcemod for a branch")]
    SDKLatest {
        #[arg(short, long, default_value_t, value_enum, value_hint = ValueHint::Other)]
        branch: Branch,

        #[arg(short, long, default_value_t, value_enum, value_hint = ValueHint::Other)]
        runtime: Runtime,
    },
    #[command(about = "Rebuild the package index in the local directory")]
    BuildIndex {},
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> BoxResult {
    run().await
}

async fn run() -> BoxResult {
    println!("📦 sm-pkg - sourcemod package manager - {}", VERSION);
    let args = Cli::parse();
    if let Some(generator) = args.generator {
        let mut cmd = Cli::command();
        print_completions(generator, &mut cmd);
        return Ok(());
    }
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
        Commands::Init { project_root } => project_init(&app_root_resolved, &project_root).await,
        Commands::Config { project_root } => {
            project_config(&app_root_resolved, &project_root).await
        }
        Commands::Build {
            plugins,
            branch,
            build_root,
        } => plugin_build(&app_root_resolved, &plugins, &branch, build_root).await,
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
        Commands::BuildIndex {} => build_index().await,
    }
}

async fn build_index() -> BoxResult {
    let mut specs: Vec<plugins::Definition> = vec![];
    for name in fs::read_dir(".")? {
        let fp = match name {
            Err(_) => continue,
            Ok(p) => p.path().join(plugins::PLUGIN_DEFINITION_FILE),
        };
        if !fp.exists() {
            continue;
        }
        let definition: plugins::Definition = serde_yaml::from_reader(File::open(fp)?)?;
        specs.push(definition);
    }

    let mut output = File::create(repo::INDEX_FILE)?;
    serde_yaml::to_writer(&mut output, &specs)?;

    println!(
        "✅ Package index built successfully. Found {} packages.",
        specs.len()
    );

    Ok(())
}

async fn plugin_build(
    app_root: &Path,
    plugins: &Vec<String>,
    branch: &Branch,
    build_root_option: Option<PathBuf>,
) -> BoxResult {
    let build_root = match build_root_option {
        Some(build_root) => build_root,
        None => create_build_root(app_root)?,
    };

    let sdk_manager = sdk::Manager::new(app_root);
    let repo = repo::LocalRepo::new(app_root);
    let sdk_env = sdk_manager.get_sdk_env(branch)?;
    match plugins::build(app_root, &sdk_env, &build_root, &repo, plugins) {
        Err(e) => return Err(format!("❌ Failed to build plugins: {}", e).into()),
        Ok(_) => {
            println!("✅ Plugins built successfully: {}", build_root.display());
        }
    };

    Ok(())
}

async fn plugin_add(app_root: &Path, project_root: &Path, plugins: Vec<String>) -> BoxResult {
    let repo = repo::LocalRepo::new(app_root);
    let mut project_manager = project::Project::new(&project_root, &repo)?;
    project_manager.open_or_new()?;

    for plugin in plugins {
        project_manager.add_plugin(repo.find_plugin_definition(&plugin)?)?;
    }

    project_manager.save_package_config()
}

async fn package_list(app_root: &Path, project_root: &Path) -> BoxResult {
    let repo = repo::LocalRepo::new(app_root);
    let mut pm = project::Project::new(&project_root, &repo)?;
    pm.open()?;
    match pm.package {
        None => return Err("❗ No package config found".into()),
        Some(config) => {
            if config.plugins.is_empty() {
                println!("🤷‍♂️ No plugins added");
            } else {
                println!("📋 Plugins added to package:");
                for plugin in config.plugins {
                    println!("- {}", plugin);
                }
            }
        }
    }
    Ok(())
}

async fn package_remove(app_root: &Path, project_root: &Path, plugins: Vec<String>) -> BoxResult {
    let repo = repo::LocalRepo::new(app_root);
    let mut project_manager = project::Project::new(&project_root, &repo)?;
    project_manager.open()?;

    for plugin in plugins {
        project_manager.remove_plugin(repo.find_plugin_definition(&plugin)?)?;
    }

    Ok(())
}

async fn project_init(app_root: &Path, project_root: &Path) -> BoxResult {
    let repo = repo::LocalRepo::new(app_root);
    let mut project_manager = project::Project::new(&project_root, &repo)?;
    project_manager.open_or_new()
}

async fn project_config(app_root: &Path, project_root: &Path) -> BoxResult {
    let repo = repo::LocalRepo::new(app_root);
    let mut project_manager = project::Project::new(&project_root, &repo)?;
    project_manager.open()?;
    project_manager.write_configs()
}

async fn package_install(app_root: &Path, project_root: &Path) -> BoxResult {
    let repo = repo::LocalRepo::new(app_root);
    let mut project_manager = project::Project::new(&project_root, &repo)?;
    project_manager.open()?;

    let project_config = project_manager.package.as_ref().expect("No package found?");
    let sdk_manager = sdk::Manager::new(app_root);
    let build_root = create_build_root(app_root)?;
    let outputs = plugins::build(
        app_root,
        &sdk_manager.get_sdk_env(&project_config.branch)?,
        &build_root,
        &repo::LocalRepo::new(app_root),
        &project_config.plugins,
    )?;

    let mod_folder = project_root.join(project_config.game.mod_folder());
    if !mod_folder.exists() {
        return Err(format!("Mod folder does not exist: {}", mod_folder.display()).into());
    }
    sdk_manager
        .install_metamod(&project_config.branch, &mod_folder)
        .await?;
    sdk_manager
        .install_sourcemod(&project_config.branch, &mod_folder)
        .await?;

    let sm_root = mod_folder.join("addons").join("sourcemod");

    for build_root in outputs {
        println!("📀 Installing {:?} -> {:?}", &build_root, &sm_root);
        fsutil::copy_dir_all(&build_root, &sm_root)?;
    }

    project_manager.write_configs()
}

async fn search(root_path: &Path, query: String) -> BoxResult {
    let repo = repo::LocalRepo::new(root_path);
    let matches: Vec<plugins::Definition> = repo.search(&query)?;
    matches
        .into_iter()
        .for_each(|p| println!("{} - {} - {}", p.name, p.version, p.description));
    Ok(())
}

async fn update(root_path: &Path) -> BoxResult {
    let repo = repo::LocalRepo::new(root_path);
    repo.update().await?;
    println!("✅ Updated local package cache");
    Ok(())
}

async fn sdk_list(root: &Path) -> BoxResult {
    let sdk_manager = sdk::Manager::new(root);
    println!("🛠️  Currently installed sourcemod SDKs:\n");
    let sdks = sdk_manager.get_installed_sdks();
    for sdk in sdks {
        println!("🏷️  {}", sdk);
    }
    Ok(())
}

async fn sdk_latest(root: &Path, runtime: &Runtime, branch: &Branch) -> BoxResult {
    let manager = sdk::Manager::new(root);
    let version = match runtime {
        Runtime::Metamod => manager.fetch_latest_metamod_build(branch).await?,
        Runtime::Sourcemod => manager.fetch_latest_sourcemod_build(branch).await?,
    };

    println!("🕓 Latest version: {version}");
    Ok(())
}

async fn sdk_install(root: &Path, runtime: &Runtime, branch: &Branch) -> BoxResult {
    let sdk_manager = sdk::Manager::new(root);
    sdk_manager.install_sdk(runtime, branch).await
}

fn print_completions<G: clap_complete::Generator>(generator: G, cmd: &mut clap::Command) {
    clap_complete::generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
}
