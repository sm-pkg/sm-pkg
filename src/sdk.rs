use flate2::read::GzDecoder;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::{File, create_dir_all, remove_file},
    io::Write,
    os::unix::fs::symlink,
    path::{self, PathBuf},
    process::Command,
};
use tar::Archive;

use crate::{BoxResult, plugins};

const DL_CACHE: &str = "dl_cache";

#[derive(clap::ValueEnum, Clone, Debug, Serialize, Default)]
pub enum Runtime {
    #[default]
    Sourcemod,
    Metamod,
}

#[derive(clap::ValueEnum, Clone, Debug, Serialize, Default, Deserialize)]
pub enum Branch {
    #[default]
    Stable,
    Dev,
}

impl Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Branch::Stable => write!(f, "stable"),
            Branch::Dev => write!(f, "dev"),
        }
    }
}

pub struct Manager<'a> {
    /// The root directory
    app_root: &'a PathBuf,
}

impl<'a> Manager<'a> {
    pub fn new(app_root: &'a PathBuf) -> Self {
        Manager { app_root }
    }

    pub async fn install_game_dir(
        &self,
        runtime: &Runtime,
        branch: &Branch,
        game_dir: &PathBuf,
    ) -> BoxResult {
        match runtime {
            Runtime::Sourcemod => self.install_sourcemod(branch, &game_dir).await,
            Runtime::Metamod => self.install_metamod(branch, &game_dir).await,
        }
    }

    pub async fn install_sdk(&self, runtime: &Runtime, branch: &Branch) -> BoxResult {
        let out_path = self.app_root.join(format!(
            "{}/sdks/sourcemod-{}",
            self.app_root.display(),
            self.get_sdk_branch_version(runtime, branch)
        ));
        let cache_path = self.app_root.join(DL_CACHE);
        if !cache_path.exists() {
            create_dir_all(&cache_path)?;
        }
        match runtime {
            Runtime::Sourcemod => {
                self.install_sourcemod(branch, &out_path).await?;
                self.activate_sdk(branch)
            }
            Runtime::Metamod => self.install_metamod(branch, &out_path).await,
        }
    }

    fn get_sdk_branch_version(&self, runtime: &Runtime, branch: &Branch) -> &str {
        match runtime {
            Runtime::Sourcemod => match branch {
                Branch::Stable => "1.12",
                Branch::Dev => "1.13",
            },
            Runtime::Metamod => match branch {
                Branch::Stable => "1.12",
                Branch::Dev => "2.0",
            },
        }
    }

    pub async fn fetch_latest_sourcemod_build(&self, branch: &Branch) -> Result<String, Error> {
        let target = format!(
            "https://sm.alliedmods.net/smdrop/{}/sourcemod-latest-linux",
            self.get_sdk_branch_version(&Runtime::Sourcemod, branch)
        );
        reqwest::get(target).await?.text().await
    }

    pub async fn fetch_latest_metamod_build(&self, branch: &Branch) -> Result<String, Error> {
        let target = format!(
            "https://mms.alliedmods.net/mmsdrop/{}/mmsource-latest-linux",
            self.get_sdk_branch_version(&Runtime::Metamod, branch)
        );
        reqwest::get(target).await?.text().await
    }

    async fn fetch_archive(&self, url: String, of: &mut File) -> BoxResult {
        let body = reqwest::get(url).await?.bytes().await?;
        of.write_all(&body[..])?;
        Ok(())
    }

    fn ensure_cache_dir(&self) -> BoxResult<PathBuf> {
        let cache_path = self.app_root.join(DL_CACHE);
        if !cache_path.exists() {
            std::fs::create_dir_all(&cache_path)?;
        }
        Ok(cache_path)
    }

    pub async fn install_sourcemod(&self, branch: &Branch, target_dir: &PathBuf) -> BoxResult {
        println!("⏳ Fetching latest version... ");
        let latest_version = Self::fetch_latest_sourcemod_build(self, &branch).await?;
        println!("🔎 Found: {latest_version}");
        let archive_path = self.ensure_cache_dir()?.join(&latest_version);
        if !archive_path.exists() {
            let target = format!(
                "https://sm.alliedmods.net/smdrop/{}/{}",
                self.get_sdk_branch_version(&Runtime::Sourcemod, branch),
                &latest_version
            );
            println!("💾 Downlading sourcemod sdk: {target}...");
            let mut of = File::create(&archive_path)?;
            self.fetch_archive(target, &mut of).await?;
        }

        self.extract_archive(&archive_path, &target_dir)?;

        Ok(())
    }

    pub async fn install_metamod(&self, branch: &Branch, target_dir: &PathBuf) -> BoxResult {
        println!("⏳ Fetching latest version... ");
        let latest_version = Self::fetch_latest_metamod_build(self, &branch).await?;
        println!("🔎 Found: {latest_version}");
        let archive_path = self.ensure_cache_dir()?.join(&latest_version);
        if !archive_path.exists() {
            let target = format!(
                "https://mms.alliedmods.net/mmsdrop/{}/{}",
                self.get_sdk_branch_version(&Runtime::Metamod, branch),
                &latest_version
            );
            println!("💾 Downlading metamod sdk: {target}...");
            let mut of = File::create(&archive_path)?;
            self.fetch_archive(target, &mut of).await?;
        }

        self.extract_archive(&archive_path, &target_dir)?;

        Ok(())
    }

    fn extract_archive(&self, archive_path: &PathBuf, out_path: &PathBuf) -> BoxResult {
        println!("📤 Extracting into: {:?}...", out_path);
        let input_archive = File::open(archive_path)?;
        let mut archive = Archive::new(GzDecoder::new(&input_archive));
        archive.unpack(out_path)?;
        Ok(())
    }

    pub fn get_installed_sdks(&self) -> Vec<String> {
        let mut sdks = Vec::new();
        if let Ok(entries) = std::fs::read_dir(self.app_root.join("sdks")) {
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

    pub fn activate_sdk(&self, branch: &Branch) -> BoxResult {
        let wanted = self.app_root.join(format!(
            "{}/sdks/sourcemod-{}",
            self.app_root.display(),
            self.get_sdk_branch_version(&Runtime::Sourcemod, branch)
        ));
        let sdks = self.get_installed_sdks();
        if sdks.is_empty() {
            Err("No SDKs installed, try: sourcemod install".into())
        } else {
            let wanted_sdk = sdks
                .iter()
                .find(|p| wanted == self.app_root.join("sdks").join(path::Path::new(p)));
            match wanted_sdk {
                Some(latest_sdk) => {
                    let sm_root = self.app_root.join("sdks").join(path::Path::new(latest_sdk));
                    let current_root = self.app_root.join("sdks/current");
                    println!("⭐ Activating {latest_sdk} @ {current_root:?}");

                    if current_root.exists() {
                        remove_file(&current_root)?;
                    }

                    symlink(sm_root, &current_root)?;
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

    pub fn get_sdk_env(&self, branch: &Branch) -> BoxResult<Environment> {
        let wanted = self.app_root.join(format!(
            "{}/sdks/sourcemod-{}",
            self.app_root.display(),
            self.get_sdk_branch_version(&Runtime::Sourcemod, branch)
        ));

        let sdks = self.get_installed_sdks();
        if sdks.is_empty() {
            return Err("No SDKs installed, try: sourcemod install".into());
        } else {
            let wanted_sdk = sdks
                .iter()
                .find(|p| wanted == self.app_root.join("sdks").join(path::Path::new(p)));
            match wanted_sdk {
                Some(path) => Ok(Environment::new(PathBuf::from(
                    self.app_root.join("sdks").join(path),
                ))),
                None => Err("No SDK found for branch".into()),
            }
        }
    }
}

#[cfg(target_pointer_width = "64")]
const COMPILER_BIN: &str = "spcomp64";

#[cfg(target_pointer_width = "32")]
// Maybe still people using this since 32bit srcds is still everywhere?
const COMPILER_BIN: &str = "spcomp";

#[derive(Debug)]
pub struct Environment {
    sdk_root: PathBuf,
}

impl Environment {
    pub fn new(sdk_root: PathBuf) -> Self {
        Environment { sdk_root }
    }

    pub fn args(&self) -> CompilerArgs {
        let mut args = CompilerArgs::new(&self.sdk_root);
        args.include(
            self.sdk_root
                .join("addons")
                .join("sourcemod")
                .join("scripting")
                .join("include"),
        );
        args
    }

    // Usage: spcomp64 [options] <filename> [filename...]
    // optional arguments:
    //   -D                        Active directory path
    //   --active-dir=ACTIVE_DIR
    //   -e                        Error file path
    //   --error-file=ERROR_FILE
    //   -E, --warnings-as-errors  Treat warnings as errors
    //   -h, --show-includes       Show included file paths
    //   -z
    //   --compress-level=COMPRESS_LEVEL
    //                             Compression level, default 9 (0=none, 1=worst,
    //                             9=best)
    //   -t, --tabsize=TABSIZE     TAB indent size (in character positions,
    //                             default=8)
    //   -v, --verbose=VERBOSE     Verbosity level; 0=quiet, 1=normal, 2=verbose
    //   -p, --prefix=PREFIX       Set name of "prefix" file
    //   -o, --output=OUTPUT       Set base name of (P-code) output file
    //   -O, --opt-level=OPT_LEVEL
    //                             Deprecated; has no effect
    //   -i, --include=INCLUDE     Path for include files
    //   -w, --warning=WARNING     Disable a specific warning by its number.
    //   -;, --require-semicolons  Require a semicolon to end each statement.
    //   --syntax-only             Perform a dry-run (No file output) on the input
    //   --use-stderr              Use stderr instead of stdout for error messages.
    //   --no-verify               Disable opcode verification (for debugging).
    //   --show-stats              Show compiler statistics on exit.
    //   sym=val                   Define macro "sym" with value "val".
    //   sym=                      Define macro "sym" with value 0.
    pub fn compile(&self, args: &mut CompilerArgs, plugin_def: &plugins::Definition) -> BoxResult {
        if let Some(inputs) = &plugin_def.inputs {
            for input in inputs {
                let mut out_bin = input.clone();
                out_bin.set_extension("smx");

                args.output = match &args.active_dir {
                    Some(dir) => {
                        let out_dir = dir.join("..").join("plugins");
                        if !out_dir.exists() {
                            create_dir_all(&out_dir)?;
                        }

                        Some(out_dir.join(&out_bin))
                    }
                    None => Some(PathBuf::from(&out_bin)),
                };

                let mut command = self.build_command(args);
                command.arg(input);
                // println!("Calling: {:?}", command);
                print!("🔨 Compiling {:?} -> ", input);
                match &args.output {
                    Some(out) => println!("into {:?}", out),
                    None => println!("into ."),
                }

                let output = command.output().expect("Failed to execute spcomp64");
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                match args.verbose.unwrap_or(0) {
                    0 => (),
                    _ => print!("{}", stdout),
                }
                if !stderr.is_empty() {
                    print!("{}", stderr);
                }
            }
        }

        Ok(())
    }

    // build_command
    fn build_command(&self, args: &CompilerArgs) -> Command {
        let mut command = Command::new(
            self.sdk_root
                .join("addons")
                .join("sourcemod")
                .join("scripting")
                .join(COMPILER_BIN),
        );

        if let Some(compress_level) = args.compress_level
            && (compress_level >= 1 && compress_level <= 9)
        {
            command.arg("-z").arg(format!("{:?}", compress_level));
        }
        if let Some(tabsize) = args.tabsize {
            command.arg("-t").arg(format!("{:?}", tabsize));
        }
        if let Some(error_file) = &args.error_file {
            command.arg("-e").arg(error_file);
        }
        if let Some(verbose) = args.verbose
            && (verbose == 1 || verbose == 2)
        {
            command.arg("-v").arg(format!("{:?}", verbose));
        }
        let mut tmp = args.active_dir.iter();
        while let Some(active_dir) = tmp.next() {
            command.arg("-D").arg(active_dir);
        }
        if let Some(prefix) = &args.prefix {
            command.arg("-p").arg(prefix);
        }
        if let Some(output) = &args.output {
            command.arg("-o").arg(output);
        }
        for include in &args.includes {
            command.arg("-i").arg(include);
        }
        if args.warnings_as_error.unwrap_or(false) {
            command.arg("-E");
        }
        if let Some(warnings) = &args.warnings {
            for warning in warnings {
                command.arg("-w").arg(warning);
            }
        };
        if args.require_semicolons.unwrap_or(true) {
            command.arg("--require-semicolons");
        }
        if args.syntax_only.unwrap_or(false) {
            command.arg("--syntax-only");
        }
        if args.use_stderr.unwrap_or(false) {
            command.arg("--use-stderr");
        }
        if args.no_verify.unwrap_or(false) {
            command.arg("--no-verify");
        }
        if args.show_stats.unwrap_or(false) {
            command.arg("--show-stats");
        }
        if args.show_includes.unwrap_or(false) {
            command.arg("--show-includes");
        }

        command
    }
}
pub struct CompilerArgs {
    pub includes: Vec<PathBuf>,
    pub warnings: Option<Vec<String>>,
    pub use_stderr: Option<bool>,
    pub show_stats: Option<bool>,
    pub macro_defs: Vec<String>,
    pub require_semicolons: Option<bool>,
    pub syntax_only: Option<bool>,
    pub no_verify: Option<bool>,
    pub error_file: Option<PathBuf>,
    pub active_dir: Option<PathBuf>,
    pub warnings_as_error: Option<bool>,
    pub show_includes: Option<bool>,
    pub compress_level: Option<u8>,
    pub output: Option<PathBuf>,
    pub prefix: Option<String>,
    pub tabsize: Option<u8>,
    pub verbose: Option<u8>,
}

impl CompilerArgs {
    pub fn include(&mut self, path: PathBuf) {
        if !self.includes.contains(&path) {
            self.includes.push(path.clone());
        }
    }
}

impl CompilerArgs {
    pub fn new(sdk_root: &PathBuf) -> Self {
        let mut args = CompilerArgs {
            includes: Vec::new(),
            warnings: Some(Vec::new()),
            use_stderr: Some(true),
            show_stats: Some(true),
            macro_defs: Vec::new(),
            require_semicolons: Some(true),
            syntax_only: Some(false),
            no_verify: Some(false),
            error_file: None,
            active_dir: None,
            warnings_as_error: Some(true),
            show_includes: Some(false),
            compress_level: Some(9),
            output: None,
            prefix: None,
            tabsize: Some(4),
            verbose: Some(0),
        };

        match default_include_path(&sdk_root) {
            Ok(include) => args.include(include),
            Err(e) => eprintln!("Warn, cannot find default include path: {}", e),
        }

        args
    }
}

fn default_include_path(sdk_path: &PathBuf) -> BoxResult<PathBuf> {
    let include_path = sdk_path
        .join("addons")
        .join("sourcemod")
        .join("scripting")
        .join("include");
    if !include_path.exists() {
        return Err("Include directory not found".into());
    }

    Ok(include_path)
}
