use std::{collections::HashMap, fmt, str::FromStr};

use anyhow::Result;
use serde::Deserialize;
use thiserror::Error as ThisError;

use crate::config::{Config, read_items_from_tomls};

const COUNTIES_FILENAME: &str = "japan-jcx.toml";
const TOWNS_FILENAME: &str = "japan-jcx-town.toml";

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

#[derive(Debug, ThisError)]
#[error("invalid JCC/JCG code: {0}")]
pub struct InvalidJcxCode(String);

/// JCC/JCG code with optional HAMLOG town suffix like `15006C`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JcxCode {
    county: String,
    town: Option<String>,
}

impl JcxCode {
    pub fn county(&self) -> &str {
        &self.county
    }

    pub fn town(&self) -> Option<&str> {
        self.town.as_deref()
    }

    /// Returns the code lengths of the prefecture and the city containing this county.
    fn ancestors(&self) -> &[usize] {
        match self.county.len() {
            6 => &[2, 4],
            4 | 5 => &[2],
            _ => &[],
        }
    }
}

impl FromStr for JcxCode {
    type Err = InvalidJcxCode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let code = s.trim().to_ascii_uppercase();
        let (county, town) = match code.char_indices().last() {
            Some((i, c)) if c.is_ascii_alphabetic() => (&code[..i], Some(&code[i..])),
            _ => (code.as_str(), None),
        };
        if county.is_empty() || !county.bytes().all(|b| b.is_ascii_digit()) {
            return Err(InvalidJcxCode(s.to_string()));
        }
        Ok(JcxCode {
            county: county.to_string(),
            town: town.map(str::to_string),
        })
    }
}

impl fmt::Display for JcxCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.county,
            self.town.as_deref().unwrap_or_default()
        )
    }
}

/// JCC/JCG definitions and HAMLOG town codes.
#[derive(Debug, Clone, Default)]
pub struct Jcx {
    counties: HashMap<String, County>,
    towns: HashMap<String, HashMap<String, Town>>,
}

impl Jcx {
    pub fn new(
        counties: HashMap<String, County>,
        towns: HashMap<String, HashMap<String, Town>>,
    ) -> Jcx {
        Jcx {
            counties: counties
                .into_iter()
                .map(|(k, v)| (k.to_ascii_uppercase(), v))
                .collect(),
            towns: towns
                .into_iter()
                .map(|(k, v)| {
                    let v = v
                        .into_iter()
                        .map(|(k, v)| (k.to_ascii_uppercase(), v))
                        .collect();
                    (k.to_ascii_uppercase(), v)
                })
                .collect(),
        }
    }

    /// Loads `japan-jcx.toml` and `japan-jcx-town.toml` next to the config file.
    pub fn load(config: &Config) -> Result<Jcx> {
        Ok(Jcx::new(
            read_items_from_tomls(config.sibling_file(COUNTIES_FILENAME))?,
            read_items_from_tomls(config.sibling_file(TOWNS_FILENAME))?,
        ))
    }

    pub fn is_empty(&self) -> bool {
        self.counties.is_empty()
    }

    pub fn county(&self, code: &JcxCode) -> Option<&County> {
        self.counties.get(code.county())
    }

    pub fn town(&self, code: &JcxCode) -> Option<&Town> {
        self.towns.get(code.county())?.get(code.town()?)
    }

    /// Builds the full Japanese name from the prefecture down to the HAMLOG town.
    /// Returns `None` if the county or the given town is unknown.
    pub fn full_name_ja(&self, code: &JcxCode) -> Option<String> {
        let county = self.county(code)?;
        let town = match code.town() {
            Some(_) => Some(self.town(code)?),
            None => None,
        };

        let ancestors = code
            .ancestors()
            .iter()
            .filter_map(|&len| self.counties.get(&code.county()[..len]));
        let name = ancestors
            .chain([county])
            .filter_map(|c| c.name_ja.as_deref())
            .chain(town.and_then(|t| t.name_ja.as_deref()))
            .collect();
        Some(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn county(kind: &str, name_ja: &str) -> County {
        County {
            kind: Some(kind.to_string()),
            name_ja: Some(name_ja.to_string()),
            name_en: None,
        }
    }

    fn jcx() -> Jcx {
        Jcx::new(
            HashMap::from([
                ("01".to_string(), county("prefecture", "北海道")),
                ("0101".to_string(), county("city", "札幌市")),
                ("010101".to_string(), county("ward", "中央区")),
                ("01006".to_string(), county("gun", "虻田郡")),
                ("10".to_string(), county("prefecture", "東京都")),
                ("100101".to_string(), county("ward", "千代田区")),
            ]),
            HashMap::from([(
                "01006".to_string(),
                HashMap::from([(
                    "a".to_string(),
                    Town {
                        name_ja: Some("京極町".to_string()),
                    },
                )]),
            )]),
        )
    }

    #[test]
    fn parses_code() {
        let code: JcxCode = " 01006a ".parse().unwrap();
        assert_eq!(code.county(), "01006");
        assert_eq!(code.town(), Some("A"));
        assert_eq!(code.to_string(), "01006A");

        let code: JcxCode = "0101".parse().unwrap();
        assert_eq!(code.county(), "0101");
        assert_eq!(code.town(), None);

        assert!("".parse::<JcxCode>().is_err());
        assert!("A".parse::<JcxCode>().is_err());
        assert!("01X06".parse::<JcxCode>().is_err());
        assert!("MA,Middlesex".parse::<JcxCode>().is_err());
    }

    #[test]
    fn builds_full_name() {
        let jcx = jcx();
        let full_name = |code: &str| jcx.full_name_ja(&code.parse().unwrap());

        assert_eq!(full_name("01").as_deref(), Some("北海道"));
        assert_eq!(full_name("0101").as_deref(), Some("北海道札幌市"));
        assert_eq!(full_name("010101").as_deref(), Some("北海道札幌市中央区"));
        assert_eq!(full_name("01006").as_deref(), Some("北海道虻田郡"));
        assert_eq!(full_name("01006A").as_deref(), Some("北海道虻田郡京極町"));
        assert_eq!(full_name("100101").as_deref(), Some("東京都千代田区"));
        assert_eq!(full_name("01006B"), None);
        assert_eq!(full_name("9999"), None);
    }
}
