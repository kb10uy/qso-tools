use mlua::prelude::*;
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

use crate::schope::engine::tableop::{TableType, check_table_type};

pub fn lua_to_json(lua_value: LuaValue) -> LuaResult<JsonValue> {
    match lua_value {
        LuaNil => Ok(JsonValue::Null),
        LuaValue::Boolean(b) => Ok(JsonValue::Bool(b)),
        LuaValue::Integer(i) => Ok(JsonValue::Number(JsonNumber::from(i))),
        LuaValue::Number(n) => JsonNumber::from_f64(n)
            .map(JsonValue::Number)
            .ok_or_else(|| LuaError::runtime(format!("non-finite number {n} is not serializable"))),
        LuaValue::String(s) => Ok(JsonValue::String(s.to_str()?.to_string())),
        LuaValue::Table(table) => match check_table_type(&table)? {
            TableType::List => table
                .sequence_values()
                .map(|v| lua_to_json(v?))
                .collect::<LuaResult<Vec<_>>>()
                .map(JsonValue::Array),
            TableType::Map => table
                .pairs::<String, LuaValue>()
                .map(|p| {
                    let (k, v) = p?;
                    Ok((k, lua_to_json(v)?))
                })
                .collect::<LuaResult<JsonMap<_, _>>>()
                .map(JsonValue::Object),
        },
        other => Err(LuaError::runtime(format!(
            "unsupported type {}",
            other.type_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schope::engine::tableop::{ensure_list, ensure_map};
    use serde_json::json;

    fn eval(lua: &Lua, code: &str) -> LuaValue {
        lua.load(code).eval().expect("valid Lua")
    }

    #[test]
    fn converts_scalars() {
        let lua = Lua::new();
        assert_eq!(lua_to_json(LuaNil).unwrap(), JsonValue::Null);
        assert_eq!(lua_to_json(eval(&lua, "true")).unwrap(), json!(true));
        assert_eq!(lua_to_json(eval(&lua, "42")).unwrap(), json!(42));
        assert_eq!(lua_to_json(eval(&lua, "1.5")).unwrap(), json!(1.5));
        assert_eq!(lua_to_json(eval(&lua, "'a'")).unwrap(), json!("a"));
    }

    #[test]
    fn rejects_non_finite_and_unsupported() {
        let lua = Lua::new();
        assert!(lua_to_json(eval(&lua, "0/0")).is_err());
        assert!(lua_to_json(eval(&lua, "1/0")).is_err());
        assert!(lua_to_json(eval(&lua, "function() end")).is_err());
    }

    #[test]
    fn infers_table_type() {
        let lua = Lua::new();
        assert_eq!(
            lua_to_json(eval(&lua, "{1, 'b', {c = 3}}")).unwrap(),
            json!([1, "b", {"c": 3}])
        );
        assert_eq!(
            lua_to_json(eval(&lua, "{a = 1, b = {2, 3}}")).unwrap(),
            json!({"a": 1, "b": [2, 3]})
        );
        assert_eq!(lua_to_json(eval(&lua, "{}")).unwrap(), json!({}));
    }

    #[test]
    fn respects_explicit_table_type() {
        let lua = Lua::new();
        let empty: LuaTable = lua.create_table().unwrap();
        let list = ensure_list(&lua, empty).unwrap();
        assert_eq!(lua_to_json(LuaValue::Table(list)).unwrap(), json!([]));

        let sequence: LuaTable = lua.load("{1, 2}").eval().unwrap();
        let map = ensure_map(&lua, sequence).unwrap();
        assert_eq!(
            lua_to_json(LuaValue::Table(map)).unwrap(),
            json!({"1": 1, "2": 2})
        );
    }
}
