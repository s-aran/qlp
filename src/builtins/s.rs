//! Shift-JIS string conversion command
//!
//! # Example
//! ```lua
//! print(s("あいうえお"))  -- あいうえお
//! ```

use mlua::{Function, Lua};

use super::builtin::BuiltinFunction;
use crate::utils::lua_string_to_shift_jis;

pub struct ShiftJis;

impl BuiltinFunction for ShiftJis {
    fn get_name(&self) -> &str {
        "s"
    }

    fn get_function(&self, lua: &Lua) -> Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |l: &Lua, string: mlua::String| {
                let s = lua_string_to_shift_jis(&l, string);
                Ok(s.clone())
            })
            .unwrap()
    }
}
