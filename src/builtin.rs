use crate::builtins::builtin::BuiltinFunction;
use mlua::Lua;

pub fn init(lua: &Lua) -> mlua::Result<()> {
    {
        use crate::builtins::json::PrettierJson;
        let _ = PrettierJson {}.set_function(lua);
    }

    Ok(())
}
