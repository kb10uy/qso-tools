use std::collections::{BTreeMap, HashMap};

use compact_str::CompactString;
use mlua::prelude::*;

use crate::core::schope::data::{exchange::Exchange, record::Record};

/// Kind of the multiplier given as `multiplier`.
pub const DEFAULT_MULTIPLIER_KIND: &str = "default";

/// QSO passed to functions of the script.
#[derive(Debug, Clone, PartialEq)]
pub struct ContestEntry {
    pub qso: Record,
    pub exchange: Exchange,
    pub station_callsign: Option<CompactString>,
    pub operator: Option<CompactString>,
    pub power: Option<f64>,
    pub contest_id: Option<CompactString>,
    pub name: Option<CompactString>,
}

impl IntoLua for ContestEntry {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("qso", self.qso)?;
        table.set("exchange", self.exchange)?;
        table.set("station_callsign", self.station_callsign.as_deref())?;
        table.set("operator", self.operator.as_deref())?;
        table.set("power", self.power)?;
        table.set("contest_id", self.contest_id.as_deref())?;
        table.set("name", self.name.as_deref())?;

        Ok(LuaValue::Table(table))
    }
}

/// Scoring of a QSO returned by `score_qso()` of the script.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct QsoScore {
    pub point: i64,
    pub multipliers: Vec<(String, String)>,
    pub tx_report: Option<String>,
    pub tx_number: Option<String>,
    pub rx_report: Option<String>,
    pub rx_number: Option<String>,
}

impl FromLua for QsoScore {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        let LuaValue::Table(table) = value else {
            return Err(LuaError::runtime(format!(
                "score_qso() must return table or nil, got {}",
                value.type_name()
            )));
        };

        let mut multipliers: Vec<_> = table
            .get::<Option<HashMap<String, String>>>("multipliers")?
            .unwrap_or_default()
            .into_iter()
            .collect();
        if let Some(multiplier) = table.get::<Option<String>>("multiplier")? {
            multipliers.push((DEFAULT_MULTIPLIER_KIND.to_string(), multiplier));
        }
        multipliers.sort();

        Ok(QsoScore {
            point: table.get::<Option<i64>>("point")?.unwrap_or_default(),
            multipliers,
            tx_report: table.get("tx_report")?,
            tx_number: table.get("tx_number")?,
            rx_report: table.get("rx_report")?,
            rx_number: table.get("rx_number")?,
        })
    }
}

/// Tally of QSOs passed to `total()` of the script.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContestSummary {
    pub qsos: u64,
    pub dupes: u64,
    pub points: i64,
    pub multipliers: u64,
    pub multipliers_by_kind: BTreeMap<String, u64>,
    pub bands: BTreeMap<String, BandSummary>,
}

/// Tally of QSOs in a band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BandSummary {
    pub qsos: u64,
    pub points: i64,
    pub multipliers: u64,
}

impl ContestSummary {
    /// Calculates total score as points times multipliers, or points if there is no multiplier.
    pub fn default_total(&self) -> i64 {
        match self.multipliers {
            0 => self.points,
            m => self.points * m as i64,
        }
    }
}

impl IntoLua for ContestSummary {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("qsos", self.qsos)?;
        table.set("dupes", self.dupes)?;
        table.set("points", self.points)?;
        table.set("multipliers", self.multipliers)?;
        table.set("total", self.default_total())?;
        table.set("multipliers_by_kind", self.multipliers_by_kind)?;
        table.set("bands", self.bands)?;

        Ok(LuaValue::Table(table))
    }
}

impl IntoLua for BandSummary {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("qsos", self.qsos)?;
        table.set("points", self.points)?;
        table.set("multipliers", self.multipliers)?;

        Ok(LuaValue::Table(table))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_qso_score() {
        let lua = Lua::new();
        let score: QsoScore = lua
            .load("{ point = 2, multiplier = '100110', multipliers = { area = '1' } }")
            .eval()
            .unwrap();
        assert_eq!(score.point, 2);
        assert_eq!(
            score.multipliers,
            [
                ("area".to_string(), "1".to_string()),
                ("default".to_string(), "100110".to_string()),
            ]
        );

        let score: QsoScore = lua.load("{}").eval().unwrap();
        assert_eq!(score, QsoScore::default());

        assert!(lua.load("'invalid'").eval::<QsoScore>().is_err());
    }

    #[test]
    fn calculates_default_total() {
        let summary = ContestSummary {
            points: 12,
            multipliers: 5,
            ..Default::default()
        };
        assert_eq!(summary.default_total(), 60);
        assert_eq!(
            ContestSummary {
                multipliers: 0,
                ..summary
            }
            .default_total(),
            12
        );
    }
}
