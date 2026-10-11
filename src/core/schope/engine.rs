mod serde;
mod tableop;

use std::{fs::read_to_string, path::Path, sync::Arc};

use anyhow::{Context, Result, bail};
use callfind::cty::CtyDatabase;
use mlua::prelude::*;
use tracing::debug;

use crate::core::{
    jcx::Jcx,
    schope::{
        engine::tableop::{ensure_list, ensure_map},
        library::{
            SchopeModule, cty::CtyModule, datetime::DateTimeModule, jarl::JarlModule,
            prompt::PromptModule,
        },
    },
};

pub use serde::lua_to_json;

pub fn initialize_lua(script_base: &Path) -> Result<Lua, LuaError> {
    let lua = Lua::new();

    replace_print(&lua)?;
    set_package_path(&lua, script_base)?;
    register_provided_features(&lua)?;

    Ok(lua)
}

/// Loads the script file that returns a table of functions.
pub fn load_script(script_path: &Path) -> Result<(Lua, LuaTable)> {
    load_script_with(script_path, |_| Ok(()))
}

/// Loads the script file after `setup` registers additional features.
pub fn load_script_with(
    script_path: &Path,
    setup: impl FnOnce(&Lua) -> LuaResult<()>,
) -> Result<(Lua, LuaTable)> {
    let script_text = read_to_string(script_path)
        .with_context(|| format!("failed to read {}", script_path.display()))?;
    let Some(script_base) = script_path.parent() else {
        bail!("script path is invalid");
    };

    let lua = initialize_lua(script_base)?;
    setup(&lua)?;
    let script_table: LuaTable = lua
        .load(script_text)
        .set_name(format!("@{}", script_path.display()))
        .eval()?;
    Ok((lua, script_table))
}

/// Registers `jarl` and `cty` modules backed by reference data.
pub fn register_reference_modules(
    lua: &Lua,
    jcx: Arc<Jcx>,
    cty: Option<Arc<CtyDatabase>>,
) -> LuaResult<()> {
    let jarl = JarlModule::new(jcx).create_module_table(lua)?;
    let cty = CtyModule::new(cty).create_module_table(lua)?;

    let package: LuaTable = lua.globals().get("package")?;
    let package_preload: LuaTable = package.get("preload")?;
    package_preload.set(
        "jarl",
        lua.create_function(move |_, _: LuaMultiValue| Ok(jarl.clone()))?,
    )?;
    package_preload.set(
        "cty",
        lua.create_function(move |_, _: LuaMultiValue| Ok(cty.clone()))?,
    )?;

    Ok(())
}

/// Replaces Lua's `print()` with `debug!()`.
fn replace_print(lua: &Lua) -> Result<(), LuaError> {
    let globals = lua.globals();

    let original: LuaFunction = globals.get("print")?;
    let hooked = lua.create_function(|_, args: LuaMultiValue| {
        let arg_texts: Result<Vec<_>, _> = args.iter().map(|v| v.to_string()).collect();
        debug!("{}", arg_texts?.join("\t"));
        Ok(())
    })?;

    globals.set("_print", original)?;
    globals.set("print", hooked)?;

    Ok(())
}

/// Sets Lua package path including `base_path`.
fn set_package_path(lua: &Lua, base_path: &Path) -> Result<(), LuaError> {
    let root = base_path.to_string_lossy();
    let globals = lua.globals();
    let package: LuaTable = globals.get("package")?;

    let package_config: String = package.get("config")?;
    let sep = package_config
        .lines()
        .next()
        .expect("package.config must exist");
    let package_path = format!("{root}{sep}?.lua;{root}{sep}?{sep}init.lua");
    package.set("path", package_path)?;

    Ok(())
}

fn register_provided_features(lua: &Lua) -> Result<(), LuaError> {
    let globals = lua.globals();

    globals.set("list", lua.create_function(ensure_list)?)?;
    globals.set("map", lua.create_function(ensure_map)?)?;

    let package: LuaTable = globals.get("package")?;
    let package_preload: LuaTable = package.get("preload")?;
    package_preload.set(
        "datetime",
        lua.create_function(DateTimeModule::create_module_table)?,
    )?;
    package_preload.set(
        "prompt",
        lua.create_function(PromptModule::create_module_table)?,
    )?;

    Ok(())
}
