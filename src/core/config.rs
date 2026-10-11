use std::{
    collections::HashMap,
    fs::read_to_string,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use etcetera::{BaseStrategy, choose_base_strategy};
use serde::{Deserialize, Deserializer, de::DeserializeOwned, de::Error as DeError};
use time::{Date, Month};
use toml::value::Datetime;

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

    #[serde(default)]
    pub contest: Option<ContestOperatorConfig>,
}

/// Operator information for summary sheets of contests.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContestOperatorConfig {
    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub address: Option<String>,

    #[serde(default)]
    pub tel: Option<String>,

    #[serde(default)]
    pub email: Option<String>,

    #[serde(default, deserialize_with = "deserialize_date")]
    pub birthday: Option<Date>,

    #[serde(default, deserialize_with = "deserialize_date")]
    pub licensed_at: Option<Date>,
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

/// Deserializes TOML local date like `1998-06-03`.
fn deserialize_date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Date>, D::Error> {
    let datetime = Datetime::deserialize(deserializer)?;
    let (Some(date), None, None) = (datetime.date, datetime.time, datetime.offset) else {
        return Err(D::Error::custom(format!(
            "expected local date, found {datetime}"
        )));
    };
    let month = Month::try_from(date.month).map_err(D::Error::custom)?;
    Date::from_calendar_date(date.year.into(), month, date.day)
        .map(Some)
        .map_err(D::Error::custom)
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

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    #[test]
    fn parses_operator() {
        let operator: OperatorConfig = toml::from_str(
            "name = \"No Call\"
[contest]
name = \"Contest Name\"
address = \"Tokyo\"
birthday = 1998-06-03
",
        )
        .unwrap();
        let contest = operator.contest.unwrap();
        assert_eq!(contest.name.as_deref(), Some("Contest Name"));
        assert_eq!(contest.address.as_deref(), Some("Tokyo"));
        assert_eq!(contest.birthday, Some(date!(1998 - 06 - 03)));
        assert_eq!(contest.licensed_at, None);

        let operator: OperatorConfig = toml::from_str("name = \"No Call\"").unwrap();
        assert_eq!(operator.contest, None);

        let invalid = toml::from_str::<OperatorConfig>(
            "name = \"No Call\"
[contest]
birthday = 1998-06-03T00:00:00
",
        );
        assert!(invalid.is_err());
        let misplaced = toml::from_str::<OperatorConfig>(
            "name = \"No Call\"
address = \"Tokyo\"
",
        );
        assert!(misplaced.is_err());
    }
}
