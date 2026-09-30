use std::fs::read_to_string;

use anyhow::{Context, Result};
use callfind::cty::CtyDatabase;

use crate::core::config::Config;

pub const CTY_FILENAME: &str = "cty.dat";

/// Loads `cty.dat` next to the config file.
/// Returns `None` if the file does not exist.
pub fn load_cty(config: &Config) -> Result<Option<CtyDatabase>> {
    let Some(path) = config.sibling_file(CTY_FILENAME) else {
        return Ok(None);
    };
    let text =
        read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let cty =
        CtyDatabase::parse(&text).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(Some(cty))
}
