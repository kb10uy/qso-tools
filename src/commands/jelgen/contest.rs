use compact_str::CompactString;
use mlua::prelude::*;

use crate::core::schope::data::{exchange::Exchange, record::Record};

/// Contest definition returned by `initialize()` of the script.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContestConfig {
    pub contest_name: String,
    pub category_code: String,
    pub callsign: Option<String>,
    pub op_callsign: Option<String>,
    pub operator: Option<String>,
    pub power: Option<f64>,
    pub fd_coeff: Option<i64>,
    pub op_place: Option<String>,
    pub power_supply: Option<String>,
    pub comments: Option<String>,
    pub multi_op_list: Option<String>,
    pub reg_club_number: Option<String>,
    pub address: Option<String>,
    pub name: Option<String>,
    pub tel: Option<String>,
    pub email: Option<String>,
    pub license_date: Option<String>,
    pub age: Option<u32>,
    pub date: Option<String>,
    pub offset: Option<String>,
    pub log_type: Option<String>,
}

impl FromLua for ContestConfig {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        let LuaValue::Table(table) = value else {
            return Err(LuaError::runtime(format!(
                "initialize() must return table, got {}",
                value.type_name()
            )));
        };

        Ok(ContestConfig {
            contest_name: table.get("contest_name")?,
            category_code: table.get("category_code")?,
            callsign: table.get("callsign")?,
            op_callsign: table.get("op_callsign")?,
            operator: table.get("operator")?,
            power: table.get("power")?,
            fd_coeff: table.get("fd_coeff")?,
            op_place: table.get("op_place")?,
            power_supply: table.get("power_supply")?,
            comments: table.get("comments")?,
            multi_op_list: table.get("multi_op_list")?,
            reg_club_number: table.get("reg_club_number")?,
            address: table.get("address")?,
            name: table.get("name")?,
            tel: table.get("tel")?,
            email: table.get("email")?,
            license_date: table.get("license_date")?,
            age: table.get("age")?,
            date: table.get("date")?,
            offset: table.get("offset")?,
            log_type: table.get("log_type")?,
        })
    }
}

/// QSO passed to `process_qso()` of the script.
#[derive(Debug, Clone, PartialEq)]
pub struct ContestEntry {
    pub qso: Record,
    pub exchange: Exchange,
    pub station_callsign: Option<CompactString>,
    pub operator: Option<CompactString>,
    pub power: Option<f64>,
    pub contest_id: Option<CompactString>,
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

        Ok(LuaValue::Table(table))
    }
}

/// Scoring of a QSO returned by `process_qso()` of the script.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct QsoScore {
    pub key: String,
    pub point: i64,
    pub multiplier: Option<String>,
    pub multiplier_key: Option<String>,
    pub band: Option<String>,
    pub mode: Option<String>,
    pub tx_report: Option<String>,
    pub tx_number: Option<String>,
    pub rx_report: Option<String>,
    pub rx_number: Option<String>,
}

impl FromLua for QsoScore {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        let LuaValue::Table(table) = value else {
            return Err(LuaError::runtime(format!(
                "process_qso() must return table or nil, got {}",
                value.type_name()
            )));
        };

        Ok(QsoScore {
            key: table.get("key")?,
            point: table.get::<Option<i64>>("point")?.unwrap_or_default(),
            multiplier: table.get("multiplier")?,
            multiplier_key: table.get("multiplier_key")?,
            band: table.get("band")?,
            mode: table.get("mode")?,
            tx_report: table.get("tx_report")?,
            tx_number: table.get("tx_number")?,
            rx_report: table.get("rx_report")?,
            rx_number: table.get("rx_number")?,
        })
    }
}

impl QsoScore {
    /// Returns the key to count the multiplier only once.
    pub fn multiplier_key(&self) -> Option<&str> {
        self.multiplier_key
            .as_deref()
            .or(self.multiplier.as_deref())
    }
}

/// Tally of processed QSOs passed to `summarize()` of the script.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ContestSummary {
    pub qsos: u64,
    pub dupes: u64,
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
            .load("{ key = '7-CW-JA1RL', point = 2, multiplier = '10', multiplier_key = '7-10' }")
            .eval()
            .unwrap();
        assert_eq!(score.point, 2);
        assert_eq!(score.multiplier_key(), Some("7-10"));

        let score: QsoScore = lua
            .load("{ key = 'JA1RL', multiplier = '10' }")
            .eval()
            .unwrap();
        assert_eq!(score.point, 0);
        assert_eq!(score.multiplier_key(), Some("10"));

        assert!(lua.load("{ point = 1 }").eval::<QsoScore>().is_err());
    }

    #[test]
    fn reads_contest_config() {
        let lua = Lua::new();
        let config: ContestConfig = lua
            .load("{ contest_name = 'ALL JA1', category_code = 'XMA', power = 25, age = 28 }")
            .eval()
            .unwrap();
        assert_eq!(config.contest_name, "ALL JA1");
        assert_eq!(config.power, Some(25.0));
        assert_eq!(config.age, Some(28));

        assert!(
            lua.load("{ contest_name = 'ALL JA1' }")
                .eval::<ContestConfig>()
                .is_err()
        );
    }

    #[test]
    fn calculates_default_total() {
        let summary = ContestSummary {
            qsos: 10,
            dupes: 1,
            points: 12,
            multipliers: 5,
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
