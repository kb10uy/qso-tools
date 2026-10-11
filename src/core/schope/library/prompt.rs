use std::sync::Arc;

use inquire::{Confirm, CustomType, Select, Text};
use mlua::prelude::*;

use crate::core::schope::library::{SchopeModule, create_module_method};

pub struct PromptModule;

impl PromptModule {
    pub fn text(&self, prompt: String, default: Option<String>) -> LuaResult<String> {
        let mut input = Text::new(&prompt);
        if let Some(default_value) = &default {
            input = input.with_default(default_value);
        }
        input.prompt().map_err(LuaError::external)
    }

    pub fn integer(&self, prompt: String, default: Option<i64>) -> LuaResult<i64> {
        let mut input = CustomType::new(&prompt);
        if let Some(default_value) = default {
            input = input.with_default(default_value);
        }
        input.prompt().map_err(LuaError::external)
    }

    pub fn float(&self, prompt: String, default: Option<f64>) -> LuaResult<f64> {
        let mut input = CustomType::new(&prompt);
        if let Some(default_value) = default {
            input = input.with_default(default_value);
        }
        input.prompt().map_err(LuaError::external)
    }

    pub fn choice(
        &self,
        prompt: String,
        choices: Vec<String>,
        default: Option<usize>,
    ) -> LuaResult<String> {
        let mut select = Select::new(&prompt, choices);
        if let Some(default_value) = default {
            select = select.with_starting_cursor(default_value);
        }
        select.prompt().map_err(LuaError::external)
    }

    pub fn confirm(&self, prompt: String, default: Option<bool>) -> LuaResult<bool> {
        let mut confirm = Confirm::new(&prompt);
        if let Some(default_value) = default {
            confirm = confirm.with_default(default_value);
        }
        confirm.prompt().map_err(LuaError::external)
    }
}

impl SchopeModule for PromptModule {
    fn create_module_table(lua: &Lua, _: LuaMultiValue) -> Result<LuaTable, LuaError> {
        let t = lua.create_table()?;

        let module = Arc::new(PromptModule);
        t.set(
            "text",
            create_module_method(lua, &module, |this, (prompt, default)| {
                this.text(prompt, default)
            })?,
        )?;
        t.set(
            "integer",
            create_module_method(lua, &module, |this, (prompt, default)| {
                this.integer(prompt, default)
            })?,
        )?;
        t.set(
            "float",
            create_module_method(lua, &module, |this, (prompt, default)| {
                this.float(prompt, default)
            })?,
        )?;
        t.set(
            "choice",
            create_module_method(lua, &module, |this, (prompt, choices, default)| {
                this.choice(prompt, choices, default)
            })?,
        )?;
        t.set(
            "confirm",
            create_module_method(lua, &module, |this, (prompt, default)| {
                this.confirm(prompt, default)
            })?,
        )?;

        Ok(t)
    }
}
