use std::{error::Error, path::PathBuf, process::Command};

use which::which;

use crate::repo;

const COMPILER_BIN: &str = "spcomp64";

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
pub fn compile(
    args: &CompilerArgs,
    plugin_def: &repo::PluginDefinition,
) -> Result<(), Box<dyn Error>> {
    if let Some(inputs) = &plugin_def.inputs {
        for input in inputs {
            let mut command = build_command(args);
            command.arg(input);

            let output = command.output().expect("Failed to execute spcomp64");
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            println!("Standard Output: {}", stdout);
            println!("Error Output: {}", stderr);
        }
    }

    Ok(())
}

// build_command 
fn build_command(args: &CompilerArgs) -> Command {
    let mut command = Command::new(COMPILER_BIN);

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

impl Default for CompilerArgs {
    fn default() -> Self {
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

        match default_include_path() {
            Ok(include) => args.include(include),
            Err(e) => eprintln!("Warn, cannot find default include path: {}", e),
        }

        args
    }
}

fn default_compiler_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(which(COMPILER_BIN)?)
}

fn default_include_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let bin_path = match default_compiler_path() {
        Ok(path) => path.parent().unwrap().to_path_buf(),
        Err(e) => return Err(e),
    };

    let include_path = bin_path.join("include");
    if !include_path.exists() {
        return Err("Include directory not found".into());
    }

    Ok(include_path)
}
