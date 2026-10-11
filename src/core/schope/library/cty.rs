use std::sync::Arc;

use callfind::cty::{CtyDatabase, Scope};
use mlua::prelude::*;

/// DXCC entity lookup backed by `cty.dat`.
pub struct CtyModule {
    cty: Option<Arc<CtyDatabase>>,
}

impl CtyModule {
    pub fn new(cty: Option<Arc<CtyDatabase>>) -> CtyModule {
        CtyModule { cty }
    }

    pub fn create_module_table(self, lua: &Lua) -> LuaResult<LuaTable> {
        let t = lua.create_table()?;
        let this = Arc::new(self);

        t.set(
            "entity",
            lua.create_function(move |lua, call: String| this.entity(lua, &call))?,
        )?;

        Ok(t)
    }

    fn entity(&self, lua: &Lua, call: &str) -> LuaResult<LuaValue> {
        let Some(cty) = &self.cty else {
            return Err(LuaError::runtime("cty.dat not found next to config file"));
        };
        let Some(found) = cty.lookup_in(&call.trim().to_ascii_uppercase(), Scope::Dxcc) else {
            return Ok(LuaNil);
        };

        let table = lua.create_table()?;
        table.set("name", found.entity.name.as_str())?;
        table.set("prefix", found.entity.primary_prefix.as_str())?;
        table.set("continent", found.location.continent.to_string())?;
        table.set("cq_zone", found.location.cq_zone)?;
        table.set("itu_zone", found.location.itu_zone)?;
        Ok(LuaValue::Table(table))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CTY: &str = "\
Japan:                    25:  45:  AS:   36.40:  -138.38:    -9.0:  JA:
    JA,JE,JF,JG,JH,JI,JJ,JK,JL,JM,JN,JO,JP,JQ,JR,JS,7J,7K,7L,7M,7N,8J,8K,8L,8M,8N;
United States:            05:  08:  NA:   37.53:    91.67:     5.0:  K:
    AA,K,N,W;
";

    #[test]
    fn looks_up_entity() {
        let lua = Lua::new();
        let cty = CtyDatabase::parse(CTY).unwrap();
        let module = CtyModule::new(Some(Arc::new(cty)))
            .create_module_table(&lua)
            .unwrap();
        lua.globals().set("cty", module).unwrap();

        let (prefix, continent, cq_zone): (String, String, u8) = lua
            .load("local e = cty.entity('jl1his'); return e.prefix, e.continent, e.cq_zone")
            .eval()
            .unwrap();
        assert_eq!(
            (prefix.as_str(), continent.as_str(), cq_zone),
            ("JA", "AS", 25)
        );
        let unknown: LuaValue = lua.load("return cty.entity('3D2AG')").eval().unwrap();
        assert!(unknown.is_nil());

        let empty = CtyModule::new(None).create_module_table(&lua).unwrap();
        lua.globals().set("cty", empty).unwrap();
        assert!(lua.load("return cty.entity('JL1HIS')").exec().is_err());
    }
}
