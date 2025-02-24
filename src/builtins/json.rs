use mlua::{Function, Lua};

use super::builtin::BuiltinFunction;

pub struct PrettierJson;

impl BuiltinFunction for PrettierJson {
    fn get_name(&self) -> &str {
        "prettier_json"
    }

    fn get_function(&self, lua: &Lua) -> Function {
        lua.create_function(|_, json: String| Ok(prettier_json(&json)))
            .unwrap()
    }
}

fn prettier_json(json: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    serde_json::to_string_pretty(&value).unwrap()
}
