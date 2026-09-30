use std::{
    collections::HashMap,
    fs::read_to_string,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use etcetera::{BaseStrategy, choose_base_strategy};
use serde::Deserialize;

const APP_NAME: &str = "qso-tools";
const CONFIG_FILENAME: &str = "config.toml";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub operators: HashMap<String, OperatorConfig>,

    #[serde(skip)]
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorConfig {
    pub name: String,
}

impl Config {
    pub fn default_path() -> Result<PathBuf> {
        let strategy = choose_base_strategy().context("failed to determine home directory")?;
        Ok(strategy.config_dir().join(APP_NAME).join(CONFIG_FILENAME))
    }

    pub fn load(explicit_path: Option<&Path>) -> Result<Config> {
        let path = match explicit_path {
            Some(path) => path.to_path_buf(),
            None => Config::default_path()?,
        };

        let text = match read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == ErrorKind::NotFound && explicit_path.is_none() => {
                return Ok(Config {
                    path,
                    ..Default::default()
                });
            }
            Err(e) => {
                return Err(e).with_context(|| format!("failed to read {}", path.display()));
            }
        };
        let config: Config =
            toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(Config { path, ..config })
    }

    pub fn sibling_file(&self, filename: &str) -> Option<PathBuf> {
        let path = self.path.parent()?.join(filename);
        path.is_file().then_some(path)
    }
}
