use std::{collections::HashMap, str::FromStr};

use mlua::prelude::*;

use crate::core::qso::record::QsoRecord;

const DEFAULT_MODE_GROUPS: &[(&str, &[&str])] = &[
    ("CW", &["CW"]),
    ("PH", &["SSB", "AM", "FM", "DIGITALVOICE"]),
];
const DEFAULT_SCOPE: &[Dimension] = &[Dimension::Band];
const CABRILLO_CATEGORY_ORDER: &[&str] = &[
    "operator",
    "assisted",
    "band",
    "mode",
    "power",
    "station",
    "transmitter",
    "time",
    "overlay",
];

/// Attribute of QSO used to separate duplicate checks and multipliers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dimension {
    Band,
    BandGroup,
    Mode,
    ModeGroup,
}

impl FromStr for Dimension {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "band" => Ok(Dimension::Band),
            "band_group" => Ok(Dimension::BandGroup),
            "mode" => Ok(Dimension::Mode),
            "mode_group" => Ok(Dimension::ModeGroup),
            otherwise => Err(format!("unknown scope dimension {otherwise}")),
        }
    }
}

/// Contest definition returned by the script.
#[derive(Debug, Clone)]
pub struct ContestDefinition {
    pub name: String,
    pub contest_id: Option<String>,
    pub offset: Option<String>,
    pub categories: Vec<Category>,
    pub mode_groups: HashMap<String, String>,
    pub band_groups: HashMap<String, String>,
    pub dupe_scope: Vec<Dimension>,
    pub multiplier_scope: Vec<Dimension>,
    pub score_qso: LuaFunction,
    pub dupe_key: Option<LuaFunction>,
    pub multiplier_key: Option<LuaFunction>,
    pub total: Option<LuaFunction>,
}

/// Category of the contest.
#[derive(Debug, Clone)]
pub struct Category {
    pub code: String,
    pub name: String,
    pub modes: Option<Vec<String>>,
    pub bands: Option<Vec<String>>,
    pub exclude_bands: Vec<String>,
    pub requires: Vec<String>,
    pub cabrillo: Vec<(String, String)>,
    pub table: LuaTable,
}

impl FromLua for ContestDefinition {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        let LuaValue::Table(table) = value else {
            return Err(LuaError::runtime(format!(
                "script must return table, got {}",
                value.type_name()
            )));
        };

        let mut categories = vec![];
        let category_tables: LuaTable = table.get("categories")?;
        for pair in category_tables.pairs::<String, LuaTable>() {
            let (code, category_table) = pair?;
            categories.push(Category::new(code, category_table)?);
        }
        if categories.is_empty() {
            return Err(LuaError::runtime("categories must not be empty"));
        }
        categories.sort_by(|a, b| a.code.cmp(&b.code));

        let mode_groups = match table.get::<Option<HashMap<String, Vec<String>>>>("mode_groups")? {
            Some(groups) => invert_groups(groups, |m| m.to_ascii_uppercase()),
            None => DEFAULT_MODE_GROUPS
                .iter()
                .flat_map(|(g, ms)| ms.iter().map(|m| (m.to_string(), g.to_string())))
                .collect(),
        };
        let band_groups = table
            .get::<Option<HashMap<String, Vec<String>>>>("band_groups")?
            .map(|groups| invert_groups(groups, |b| b.to_ascii_lowercase()))
            .unwrap_or_default();

        Ok(ContestDefinition {
            name: table.get("name")?,
            contest_id: table.get("contest_id")?,
            offset: table.get("offset")?,
            categories,
            mode_groups,
            band_groups,
            dupe_scope: read_scope(&table, "dupe_scope")?,
            multiplier_scope: read_scope(&table, "multiplier_scope")?,
            score_qso: table.get("score_qso")?,
            dupe_key: table.get("dupe_key")?,
            multiplier_key: table.get("multiplier_key")?,
            total: table.get("total")?,
        })
    }
}

impl Category {
    fn new(code: String, table: LuaTable) -> LuaResult<Category> {
        table.set("code", code.as_str())?;
        let lowercase = |bands: Vec<String>| -> Vec<String> {
            bands.into_iter().map(|b| b.to_ascii_lowercase()).collect()
        };

        Ok(Category {
            name: table.get("name")?,
            modes: table.get("modes")?,
            bands: table.get::<Option<Vec<String>>>("bands")?.map(lowercase),
            exclude_bands: lowercase(table.get::<Option<_>>("exclude_bands")?.unwrap_or_default()),
            requires: table.get::<Option<_>>("requires")?.unwrap_or_default(),
            cabrillo: read_cabrillo_categories(&table)?,
            table,
            code,
        })
    }
}

impl ContestDefinition {
    pub fn category(&self, code: &str) -> Option<&Category> {
        self.categories.iter().find(|c| c.code == code)
    }

    pub fn mode_group(&self, qso: &QsoRecord) -> String {
        let mode = qso.mode.as_str().to_ascii_uppercase();
        let submode = qso.submode.as_deref().map(str::to_ascii_uppercase);
        submode
            .and_then(|s| self.mode_groups.get(&s))
            .or_else(|| self.mode_groups.get(&mode))
            .cloned()
            .unwrap_or(mode)
    }

    pub fn band_group(&self, qso: &QsoRecord) -> String {
        let band = qso.band.to_string();
        self.band_groups.get(&band).cloned().unwrap_or(band)
    }

    /// Builds the key of QSO separated by the dimensions.
    pub fn scope_key(&self, scope: &[Dimension], qso: &QsoRecord, value: &str) -> String {
        let mut parts: Vec<_> = scope
            .iter()
            .map(|dimension| match dimension {
                Dimension::Band => qso.band.to_string(),
                Dimension::BandGroup => self.band_group(qso),
                Dimension::Mode => qso.mode.as_str().to_ascii_uppercase(),
                Dimension::ModeGroup => self.mode_group(qso),
            })
            .collect();
        parts.push(value.to_string());
        parts.join("/")
    }

    /// Checks whether QSO is in bands and modes of the category.
    pub fn accepts(&self, category: &Category, qso: &QsoRecord) -> bool {
        let band = qso.band.to_string();
        let in_bands = category.bands.as_ref().is_none_or(|bs| bs.contains(&band));
        let in_modes = category
            .modes
            .as_ref()
            .is_none_or(|ms| ms.contains(&self.mode_group(qso)));
        in_bands && in_modes && !category.exclude_bands.contains(&band)
    }
}

/// Reads `CATEGORY-*` values of Cabrillo in the order of the specification.
fn read_cabrillo_categories(table: &LuaTable) -> LuaResult<Vec<(String, String)>> {
    let values: HashMap<String, String> = table.get::<Option<_>>("cabrillo")?.unwrap_or_default();
    let mut values: Vec<_> = values
        .into_iter()
        .map(|(k, v)| (k.to_ascii_lowercase(), v))
        .collect();
    values.sort_by_key(|(k, _)| {
        let rank = CABRILLO_CATEGORY_ORDER.iter().position(|o| o == k);
        (rank.unwrap_or(CABRILLO_CATEGORY_ORDER.len()), k.clone())
    });
    Ok(values)
}

fn read_scope(table: &LuaTable, key: &str) -> LuaResult<Vec<Dimension>> {
    match table.get::<Option<Vec<String>>>(key)? {
        Some(dimensions) => dimensions
            .iter()
            .map(|d| d.parse().map_err(LuaError::runtime))
            .collect(),
        None => Ok(DEFAULT_SCOPE.to_vec()),
    }
}

fn invert_groups(
    groups: HashMap<String, Vec<String>>,
    normalize: impl Fn(&str) -> String,
) -> HashMap<String, String> {
    groups
        .into_iter()
        .flat_map(|(group, members)| {
            members
                .into_iter()
                .map(move |m| (m, group.clone()))
                .collect::<Vec<_>>()
        })
        .map(|(member, group)| (normalize(&member), group))
        .collect()
}

#[cfg(test)]
mod tests {
    use adif_manip::{LengthMode, read_adi};
    use time::UtcOffset;

    use super::*;

    fn definition(lua: &Lua, extra: &str) -> ContestDefinition {
        lua.load(format!(
            "return {{
                name = 'Test',
                categories = {{
                    XA = {{ name = 'Mixed all band', cabrillo = {{ power = 'LOW', Operator = 'SINGLE-OP', band = 'ALL' }} }},
                    PA = {{ name = 'Phone', modes = {{ 'PH' }}, exclude_bands = {{ '20M' }} }},
                    C7 = {{ name = 'CW 7MHz', modes = {{ 'CW' }}, bands = {{ '40m' }} }},
                }},
                score_qso = function() end,
                {extra}
            }}"
        ))
        .eval()
        .unwrap()
    }

    fn qso(fields: &str) -> QsoRecord {
        let document = read_adi(
            &format!("Test\n<EOH>\n<QSO_DATE:8>20260606<TIME_ON:4>1200<FREQ:5>7.010<CALL:5>JA1RL{fields}<EOR>\n"),
            LengthMode::Bytes,
        )
        .unwrap();
        QsoRecord::new(&document.records()[0], UtcOffset::UTC).unwrap()
    }

    #[test]
    fn reads_definition() {
        let lua = Lua::new();
        let definition = definition(&lua, "");
        let codes: Vec<_> = definition
            .categories
            .iter()
            .map(|c| c.code.as_str())
            .collect();
        assert_eq!(codes, ["C7", "PA", "XA"]);
        assert_eq!(definition.dupe_scope, [Dimension::Band]);
        let cabrillo: Vec<_> = definition
            .category("XA")
            .unwrap()
            .cabrillo
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(
            cabrillo,
            [("operator", "SINGLE-OP"), ("band", "ALL"), ("power", "LOW")]
        );
        assert_eq!(
            definition
                .category("PA")
                .unwrap()
                .table
                .get::<String>("code")
                .unwrap(),
            "PA"
        );

        let invalid = lua
            .load("return { name = 'Test', categories = { XA = { name = 'X' } }, dupe_scope = { 'call' }, score_qso = function() end }")
            .eval::<ContestDefinition>();
        assert!(invalid.is_err());
    }

    #[test]
    fn groups_modes_and_bands() {
        let lua = Lua::new();
        let default = definition(&lua, "");
        assert_eq!(default.mode_group(&qso("<BAND:3>40m<MODE:3>SSB")), "PH");
        assert_eq!(default.mode_group(&qso("<BAND:3>40m<MODE:2>CW")), "CW");
        assert_eq!(default.mode_group(&qso("<BAND:3>40m<MODE:3>FT8")), "FT8");

        let custom = definition(
            &lua,
            "mode_groups = { DG = { 'FT8', 'FT4' } }, band_groups = { HF = { '40M', '20m' } },",
        );
        assert_eq!(
            custom.mode_group(&qso("<BAND:3>40m<MODE:4>MFSK<SUBMODE:3>FT4")),
            "DG"
        );
        assert_eq!(custom.mode_group(&qso("<BAND:3>40m<MODE:3>SSB")), "SSB");
        assert_eq!(custom.band_group(&qso("<BAND:3>40m<MODE:2>CW")), "HF");
        assert_eq!(custom.band_group(&qso("<BAND:2>6m<MODE:2>CW")), "6m");
    }

    #[test]
    fn builds_scope_key() {
        let lua = Lua::new();
        let definition = definition(&lua, "");
        let qso = qso("<BAND:3>40m<MODE:3>SSB");
        let scope = [Dimension::Band, Dimension::ModeGroup];
        assert_eq!(definition.scope_key(&scope, &qso, "JA1RL"), "40m/PH/JA1RL");
        assert_eq!(definition.scope_key(&[], &qso, "JA1RL"), "JA1RL");
    }

    #[test]
    fn accepts_qsos_of_category() {
        let lua = Lua::new();
        let definition = definition(&lua, "");
        let category = |code| definition.category(code).unwrap();
        let ssb_40m = qso("<BAND:3>40m<MODE:3>SSB");
        let ssb_20m = qso("<BAND:3>20m<MODE:3>SSB");
        let cw_40m = qso("<BAND:3>40m<MODE:2>CW");

        assert!(definition.accepts(category("XA"), &ssb_20m));
        assert!(definition.accepts(category("PA"), &ssb_40m));
        assert!(!definition.accepts(category("PA"), &ssb_20m));
        assert!(!definition.accepts(category("PA"), &cw_40m));
        assert!(definition.accepts(category("C7"), &cw_40m));
        assert!(!definition.accepts(category("C7"), &ssb_40m));
    }
}
