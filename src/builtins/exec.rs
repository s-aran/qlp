use std::process::Command;

use encoding_rs::SHIFT_JIS;
use mlua::{FromLua, Function, IntoLua, Lua};

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

impl FromLua for ExecResult {
    fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        let table = value.as_table().unwrap();

        Ok(ExecResult {
            code: table.get("code")?,
            stdout: table.get("stdout")?,
            stderr: table.get("stderr")?,
        })
    }
}

fn system(program: String, args: Vec<String>) -> ExecResult {
    let mut command = Command::new(program.as_str());
    let result = command.args(args);
    let output = result.output().expect("failed to execute process");

    ExecResult {
        code: output.status.code().unwrap(),
        stdout: decode_process_output(&output.stdout),
        stderr: decode_process_output(&output.stderr),
    }
}

fn decode_process_output(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_owned(),
        Err(_) => {
            let (s, _, _) = SHIFT_JIS.decode(bytes);
            s.into_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::exec::system;

    #[test]
    fn test_system() {
        let (program, args, expected) = echo_command();
        let args = args.iter().map(|e| (*e).into()).collect::<Vec<String>>();
        let result = system(program, args);

        assert_eq!(0, result.code);
        assert_eq!(expected, result.stdout);
        assert_eq!("", result.stderr);
    }

    #[test]
    fn test_exec_by_lua() {
        let lua = Lua::new();

        let _ = Exec {}.set_function(&lua);
        let (program, args, expected) = echo_command();
        lua.globals().set("program", program).unwrap();
        lua.globals().set("args", args).unwrap();
        lua.load(r#"result = exec(program, args)"#).exec().unwrap();

        let result = lua.globals().get::<ExecResult>("result").unwrap();

        assert_eq!(0, result.code);
        assert_eq!(expected, result.stdout);
        assert_eq!("", result.stderr);
    }

    #[cfg(target_os = "windows")]
    fn echo_command() -> (String, Vec<&'static str>, &'static str) {
        (
            "cmd".to_string(),
            vec!["/C", "echo foo bar baz qux"],
            "foo bar baz qux\r\n",
        )
    }

    #[cfg(not(target_os = "windows"))]
    fn echo_command() -> (String, Vec<&'static str>, &'static str) {
        (
            "echo".to_string(),
            vec!["foo", "bar", "baz", "qux"],
            "foo bar baz qux\n",
        )
    }
}
