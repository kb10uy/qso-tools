use mlua::prelude::*;

const SCHOPE_TYPE_KEY: &str = "__schope_type";
const SCHOPE_TYPE_LIST: &str = "list";
const SCHOPE_TYPE_MAP: &str = "map";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableType {
    List,
    Map,
}

impl TableType {
    fn marker(self) -> &'static str {
        match self {
            TableType::List => SCHOPE_TYPE_LIST,
            TableType::Map => SCHOPE_TYPE_MAP,
        }
    }

    fn from_marker(marker: &str) -> Option<TableType> {
        match marker {
            SCHOPE_TYPE_LIST => Some(TableType::List),
            SCHOPE_TYPE_MAP => Some(TableType::Map),
            _ => None,
        }
    }
}

pub fn ensure_list(lua: &Lua, table: LuaTable) -> LuaResult<LuaTable> {
    mark_table_type(lua, table, TableType::List)
}

pub fn ensure_map(lua: &Lua, table: LuaTable) -> LuaResult<LuaTable> {
    mark_table_type(lua, table, TableType::Map)
}

fn mark_table_type(lua: &Lua, table: LuaTable, table_type: TableType) -> LuaResult<LuaTable> {
    let metatable = match table.metatable() {
        Some(mt) => mt,
        None => {
            let mt = lua.create_table()?;
            table.set_metatable(Some(mt.clone()))?;
            mt
        }
    };
    metatable.set(SCHOPE_TYPE_KEY, table_type.marker())?;
    Ok(table)
}

pub fn check_table_type(table: &LuaTable) -> LuaResult<TableType> {
    if let Some(mt) = table.metatable() {
        let marker: Option<String> = mt.get(SCHOPE_TYPE_KEY)?;
        if let Some(table_type) = marker.as_deref().and_then(TableType::from_marker) {
            return Ok(table_type);
        }
    }

    let len = table.raw_len();
    if len == 0 {
        return Ok(TableType::Map);
    }
    for i in 1..=len {
        if !table.contains_key(i)? {
            return Ok(TableType::Map);
        }
    }
    Ok(TableType::List)
}
