use crate::builtins::builtin::BuiltinFunction;
use mlua::Lua;

pub fn init(lua: &Lua) -> mlua::Result<()> {
    {
        use crate::builtins::json::PrettierJson;
        let _ = PrettierJson {}.set_function(lua);
    }

    {
        use crate::builtins::exec::Exec;
        let _ = Exec {}.set_function(lua);
    }

    Ok(())
}
