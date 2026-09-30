use std::{collections::HashMap, fs::read_to_string, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, de::DeserializeOwned};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Instrument {
    pub rig: String,
    pub antenna: String,
    pub default_power: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Park {
    pub name_en: Option<String>,
    pub name_ja: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct County {
    pub kind: Option<String>,
    pub name_ja: Option<String>,
    pub name_en: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Town {
    pub name_ja: Option<String>,
}

pub fn read_items_from_tomls<T: DeserializeOwned>(
    files: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<HashMap<String, T>> {
    let mut items = HashMap::new();

    for file in files {
        let file = file.as_ref();
        let toml =
            read_to_string(file).with_context(|| format!("failed to read {}", file.display()))?;
        let file_items: HashMap<_, T> =
            toml::from_str(&toml).with_context(|| format!("failed to parse {}", file.display()))?;
        items.extend(file_items);
    }
    Ok(items)
}
