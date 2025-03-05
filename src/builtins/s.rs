use mlua::{Function, Lua};

use encoding_rs;
use encoding_rs::SHIFT_JIS;

use super::builtin::BuiltinFunction;

pub struct ShiftJis;

impl BuiltinFunction for ShiftJis {
    fn get_name(&self) -> &str {
        "s"
    }

    fn get_function(&self, lua: &Lua) -> Function {
        lua.create_function(|l: &Lua, string: mlua::String| {
            let ls = string.to_str().unwrap();
            let (s, _, _) = SHIFT_JIS.encode(&ls);
            Ok(l.create_string(&s))
        })
        .unwrap()
    }
}
