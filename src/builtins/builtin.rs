use mlua::Lua;

pub trait BuiltinFunction {
    fn get_name(&self) -> &str;
    fn get_function(&self, lua: &Lua) -> mlua::Function;

    fn set_function(&self, lua: &Lua) -> mlua::Result<()> {
        let globals = lua.globals();
        let name = self.get_name();
        let func = self.get_function(lua);
        globals.set(name, func)?;
        Ok(())
    }
}
