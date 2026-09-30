use callfind::grid_locator::GridLocator;
use compact_str::CompactString;
use mlua::prelude::*;

use crate::core::schope::data::{exchange::Exchange, record::Record};

#[derive(Debug, Clone, PartialEq)]
pub struct QslCardEntry {
    pub qso: Record,
    pub exchange: Exchange,
    pub station: QslStation,
    pub operator: QslOperator,
    pub instrument: QslInstrument,
    pub qsl: QslRouting,
}

impl IntoLua for QslCardEntry {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("qso", self.qso)?;
        table.set("exchange", self.exchange)?;
        table.set("station", self.station)?;
        table.set("operator", self.operator)?;
        table.set("instrument", self.instrument)?;
        table.set("qsl", self.qsl)?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QslStation {
    pub callsign: Option<CompactString>,
    pub location: QslLocation,
    pub references: QslReferences,
}

impl IntoLua for QslStation {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("callsign", self.callsign.as_deref())?;
        table.set("location", self.location)?;
        table.set("references", self.references)?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QslLocation {
    pub grid: Option<GridLocator>,
    pub city: Option<CompactString>,
    pub county: Option<QslCounty>,
    pub state: Option<QslState>,
    pub country: Option<CompactString>,
}

impl IntoLua for QslLocation {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("grid", self.grid.map(|g| g.to_string()))?;
        table.set("city", self.city.as_deref())?;
        table.set("county", self.county)?;
        table.set("state", self.state)?;
        table.set("country", self.country.as_deref())?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QslState {
    pub code: CompactString,
    pub name: Option<CompactString>,
}

impl IntoLua for QslState {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("code", self.code.as_str())?;
        table.set("name", self.name.as_deref())?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QslCounty {
    pub code: CompactString,
    pub kind: Option<CompactString>,
    pub name_ja: Option<CompactString>,
    pub name_en: Option<CompactString>,
    pub town_ja: Option<CompactString>,
}

impl IntoLua for QslCounty {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("code", self.code.as_str())?;
        table.set("kind", self.kind.as_deref())?;
        table.set("name_ja", self.name_ja.as_deref())?;
        table.set("name_en", self.name_en.as_deref())?;
        table.set("town_ja", self.town_ja.as_deref())?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QslReferences {
    pub pota: Vec<QslPark>,
    pub sota: Option<CompactString>,
    pub wwff: Option<CompactString>,
    pub iota: Option<CompactString>,
    pub sig: Option<CompactString>,
    pub sig_info: Option<CompactString>,
}

impl IntoLua for QslReferences {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("pota", self.pota)?;
        table.set("sota", self.sota.as_deref())?;
        table.set("wwff", self.wwff.as_deref())?;
        table.set("iota", self.iota.as_deref())?;
        table.set("sig", self.sig.as_deref())?;
        table.set("sig_info", self.sig_info.as_deref())?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QslPark {
    pub reference: CompactString,
    pub location: Option<CompactString>,
    pub name_en: Option<CompactString>,
    pub name_ja: Option<CompactString>,
}

impl IntoLua for QslPark {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("reference", self.reference.as_str())?;
        table.set("location", self.location.as_deref())?;
        table.set("name_en", self.name_en.as_deref())?;
        table.set("name_ja", self.name_ja.as_deref())?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QslOperator {
    pub callsign: Option<CompactString>,
    pub name: Option<CompactString>,
}

impl IntoLua for QslOperator {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("callsign", self.callsign.as_deref())?;
        table.set("name", self.name.as_deref())?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QslInstrument {
    pub antenna: Option<CompactString>,
    pub rig: Option<CompactString>,
    pub power: Option<f64>,
}

impl IntoLua for QslInstrument {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("antenna", self.antenna.as_deref())?;
        table.set("rig", self.rig.as_deref())?;
        table.set("power", self.power)?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QslRouting {
    pub should_send: bool,
    pub received: bool,
    pub via: Option<CompactString>,
    pub sent_via: Option<CompactString>,
}

impl IntoLua for QslRouting {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("should_send", self.should_send)?;
        table.set("received", self.received)?;
        table.set("via", self.via.as_deref())?;
        table.set("sent_via", self.sent_via.as_deref())?;

        Ok(LuaValue::Table(table))
    }
}
