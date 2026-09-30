use std::sync::Arc;

use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use mlua::prelude::*;

use crate::schope::library::{SchopeModule, create_module_method};

pub struct PromptModule {
    theme: ColorfulTheme,
}

impl PromptModule {
    pub fn text(&self, prompt: String, default: Option<String>) -> LuaResult<String> {
        let mut input = Input::with_theme(&self.theme).with_prompt(prompt);
        if let Some(default_value) = default {
            input = input.default(default_value);
        }
        input.interact_text().map_err(LuaError::external)
    }

    pub fn integer(&self, prompt: String, default: Option<i64>) -> LuaResult<i64> {
        let mut input = Input::with_theme(&self.theme).with_prompt(prompt);
        if let Some(default_value) = default {
            input = input.default(default_value);
        }
        input.interact_text().map_err(LuaError::external)
    }

    pub fn float(&self, prompt: String, default: Option<f64>) -> LuaResult<f64> {
        let mut input = Input::with_theme(&self.theme).with_prompt(prompt);
        if let Some(default_value) = default {
            input = input.default(default_value);
        }
        input.interact_text().map_err(LuaError::external)
    }

    pub fn choice(
        &self,
        prompt: String,
        mut choices: Vec<String>,
        default: Option<usize>,
    ) -> LuaResult<String> {
        let mut select = Select::with_theme(&self.theme)
            .with_prompt(prompt)
            .items(&choices);
        if let Some(default_value) = default {
            select = select.default(default_value);
        }
        let index = select.interact().map_err(LuaError::external)?;
        Ok(choices.swap_remove(index))
    }

    pub fn confirm(&self, prompt: String, default: Option<bool>) -> LuaResult<bool> {
        let mut confirm = Confirm::with_theme(&self.theme).with_prompt(prompt);
        if let Some(default_value) = default {
            confirm = confirm.default(default_value);
        }
        confirm.interact().map_err(LuaError::external)
    }
}

impl SchopeModule for PromptModule {
    fn create_module_table(lua: &Lua, _: LuaMultiValue) -> Result<LuaTable, LuaError> {
        let t = lua.create_table()?;

        let module = Arc::new(PromptModule {
            theme: ColorfulTheme::default(),
        });
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
