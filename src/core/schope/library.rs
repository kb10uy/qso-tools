pub mod datetime;
pub mod prompt;

use std::sync::Arc;

use mlua::{MaybeSend, prelude::*};

pub trait SchopeModule {
    fn create_module_table(lua: &Lua, args: LuaMultiValue) -> Result<LuaTable, LuaError>;
}

fn create_module_method<M, P, R>(
    lua: &Lua,
    module: &Arc<M>,
    func: impl Fn(&M, P) -> LuaResult<R> + MaybeSend + 'static,
) -> Result<LuaFunction, LuaError>
where
    M: SchopeModule + MaybeSend + Sync + 'static,
    P: FromLuaMulti,
    R: IntoLuaMulti,
{
    let this = module.clone();
    lua.create_function(move |_, params| func(&this, params))
}
