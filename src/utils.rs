use mlua::{Lua, Result, Table, Value};
use serde_json::{Map, Value as JsonValue};

pub fn json_str_to_lua_table(lua: &Lua, json_str: &str) -> Result<Table> {
    let json_value: JsonValue = serde_json::from_str(json_str)
        .map_err(|e| mlua::Error::RuntimeError(format!("JSON parse error: {}", e)))?;

    let lua_value = json_to_lua(lua, &json_value)?;
    match lua_value {
        Value::Table(table) => Ok(table),
        _ => Err(mlua::Error::RuntimeError(
            "Expected JSON to be a table".into(),
        )),
    }
}

pub fn lua_table_to_json_str(lua: &Lua, table: Table) -> Result<String> {
    let json_value = lua_to_json(Value::Table(table))?;
    serde_json::to_string(&json_value).map_err(|e| mlua::Error::RuntimeError(e.to_string()))
}

pub fn json_to_lua(lua: &Lua, json: &JsonValue) -> Result<Value> {
    match json {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(b) => Ok(Value::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Err(mlua::Error::RuntimeError("Invalid number".into()))
            }
        }
        JsonValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
        JsonValue::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
        JsonValue::Object(obj) => {
            let table = lua.create_table()?;
            for (k, v) in obj.iter() {
                table.set(k.as_str(), json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

pub fn lua_to_json(value: Value) -> Result<JsonValue> {
    match value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Bool(b)),
        Value::Integer(i) => Ok(JsonValue::Number(i.into())),
        Value::Number(n) => Ok(JsonValue::Number(
            serde_json::Number::from_f64(n)
                .ok_or_else(|| mlua::Error::RuntimeError("Invalid number".into()))?,
        )),
        Value::String(s) => Ok(JsonValue::String(s.to_str()?.to_string())),
        Value::Table(table) => {
            if is_array(&table)? {
                let mut arr = Vec::new();
                for pair in table.pairs::<i64, Value>() {
                    let (_, v) = pair?;
                    arr.push(lua_to_json(v)?);
                }
                Ok(JsonValue::Array(arr))
            } else {
                let mut obj = Map::new();
                for pair in table.pairs::<String, Value>() {
                    let (k, v) = pair?;
                    obj.insert(k, lua_to_json(v)?);
                }
                Ok(JsonValue::Object(obj))
            }
        }
        _ => Err(mlua::Error::RuntimeError("Unsupported Lua value".into())),
    }
}

fn is_array(table: &Table) -> Result<bool> {
    let mut expected = 1;
    for pair in table.pairs::<Value, Value>() {
        let (k, _) = pair?;
        if let Value::Integer(i) = k {
            if i != expected {
                return Ok(false);
            }
            expected += 1;
        } else {
            return Ok(false);
        }
    }
    Ok(true)
}
