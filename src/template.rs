use minijinja::Environment;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Write, path};

type ConfigValues = HashMap<String, String>;

pub fn read_templates<'a>() -> Result<Environment<'a>, minijinja::Error> {
    let mut env = Environment::new();
    minijinja_embed::load_templates!(&mut env);

    Ok(env)
}

#[derive(Serialize, Deserialize)]
enum Format {
    CFG,
    KV,
}

#[derive(Serialize, Deserialize)]
pub struct FileConfig {
    format: Format,
    path: path::PathBuf,
}

pub fn render_cfg(mut writer: impl Write, values: ConfigValues) {
    for (key, value) in values {
        writeln!(writer, "{} = {}", key, value).unwrap();
    }
}

#[test]
fn test_read_config() -> Result<(), Box<dyn std::error::Error>> {
    let env = read_templates()?;
    assert_eq!(9, env.templates().count());
    Ok(())
}
