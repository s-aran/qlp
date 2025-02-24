use std::{
    ffi::{OsStr, OsString},
    process::Command,
};

use mlua::{Function, IntoLua, Lua};

use super::builtin::BuiltinFunction;

pub struct Exec;

impl BuiltinFunction for Exec {
    fn get_name(&self) -> &str {
        "exec"
    }

    fn get_function(&self, lua: &Lua) -> Function {
        lua.create_function(|_, (param, args): (String, Vec<String>)| Ok(system(param, args)))
            .unwrap()
    }
}

pub struct ExecResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl IntoLua for ExecResult {
    fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
        let table = lua.create_table()?;

        table.set("code", self.code)?;
        table.set("stdout", self.stdout)?;
        table.set("stderr", self.stderr)?;

        Ok(table.into_lua(lua).unwrap())
    }
}

fn system(program: String, args: Vec<String>) -> ExecResult {
    let mut command = Command::new(program.as_str());
    let result = command.args(args);
    let output = result.output().expect("failed to execute process");

    ExecResult {
        code: output.status.code().unwrap(),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}
