use mlua::{Function, Lua};

use crate::utils::json_str_to_lua_table;

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

pub struct MinifyJson;
impl BuiltinFunction for MinifyJson {
    fn get_name(&self) -> &str {
        "minify_json"
    }

    fn get_function(&self, lua: &Lua) -> Function {
        lua.create_function(|_, json: String| Ok(minify_json(&json)))
            .unwrap()
    }
}
fn minify_json(json: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    serde_json::to_string(&value).unwrap()
}

pub struct JsonToTable;
impl BuiltinFunction for JsonToTable {
    fn get_name(&self) -> &str {
        "json_to_table"
    }

    fn get_function(&self, lua: &Lua) -> Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, json: String| {
                Ok(json_str_to_lua_table(&lua_ref, json.as_str()).unwrap())
            })
            .unwrap()
    }
}
