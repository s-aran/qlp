use crate::builtins::builtin::BuiltinFunction;
use mlua::Lua;

pub fn init(lua: &Lua) -> mlua::Result<()> {
    {
        use crate::builtins::json::JsonToTable;
        use crate::builtins::json::MinifyJson;
        use crate::builtins::json::PrettierJson;

        let _ = PrettierJson {}.set_function(lua);
        let _ = MinifyJson {}.set_function(lua);
        let _ = JsonToTable {}.set_function(lua);
    }

    {
        use crate::builtins::exec::Exec;
        let _ = Exec {}.set_function(lua);
    }

    {
        use crate::builtins::s::ShiftJis;
        let _ = ShiftJis {}.set_function(lua);
    }

    {
        use crate::builtins::include::Include;
        let _ = Include {}.set_function(lua);
    }

    Ok(())
}

pub fn get_engine_version(lua: &Lua) -> u32 {
    lua.globals().get::<u32>("ENGINE_VERSION").unwrap_or(0)
}

pub fn set_engine_version(lua: &Lua, version: u32) {
    //
}
