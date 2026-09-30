use compact_str::CompactString;
use mlua::prelude::*;

use crate::qso::exchange::QsoExchanges;

#[derive(Debug, Clone, PartialEq)]
pub struct Exchange {
    pub tx_report: Option<CompactString>,
    pub tx_number: Option<CompactString>,
    pub rx_report: Option<CompactString>,
    pub rx_number: Option<CompactString>,
}

impl IntoLua for Exchange {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("tx_report", self.tx_report.as_deref())?;
        table.set("tx_number", self.tx_number.as_deref())?;
        table.set("rx_report", self.rx_report.as_deref())?;
        table.set("rx_number", self.rx_number.as_deref())?;

        Ok(LuaValue::Table(table))
    }
}

impl From<QsoExchanges> for Exchange {
    fn from(value: QsoExchanges) -> Self {
        Exchange {
            tx_report: value.sent.report,
            tx_number: value.sent.number,
            rx_report: value.received.report,
            rx_number: value.received.number,
        }
    }
}
