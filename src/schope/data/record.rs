use compact_str::{CompactString, ToCompactString};
use mlua::prelude::*;
use time::OffsetDateTime;

use crate::{qso::record::QsoRecord, schope::library::datetime::SchopeDateTime};

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub datetime: SchopeDateTime,
    pub band: CompactString,
    pub freq: f64,
    pub freq_str: CompactString,
    pub mode: CompactString,
    pub call: CompactString,
}

impl IntoLua for Record {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("datetime", self.datetime)?;
        table.set("band", self.band.as_str())?;
        table.set("freq", self.freq)?;
        table.set("freq_str", self.freq_str.as_str())?;
        table.set("mode", self.mode.as_str())?;
        table.set("call", self.call.as_str())?;

        Ok(LuaValue::Table(table))
    }
}

impl From<QsoRecord> for Record {
    fn from(value: QsoRecord) -> Self {
        Record {
            datetime: OffsetDateTime::from(value.datetime).into(),
            band: value.band.to_compact_string(),
            freq: value.frequency,
            freq_str: value.frequency_str,
            mode: value.mode,
            call: value.call,
        }
    }
}
