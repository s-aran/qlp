//! Include external file command
//!
//! # Example
//! ```lua
//! content = include("other_file.txt")
//! ```

use core::panic;
use std::{fs, path::PathBuf};

use mlua::Lua;

use super::builtin::*;

pub struct Include;

impl BuiltinFunction for Include {
    fn get_name(&self) -> &str {
        "include"
    }

    fn get_function(&self, lua: &Lua) -> mlua::Function {
        let lua_ref = lua.clone();
        lua_ref
            .clone()
            .create_function(move |_, path: PathBuf| {
                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(e) => {
                        panic!("Error reading file: {}", e);
                    }
                };

                return Ok(content);
            })
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::Mll;

    #[test]
    fn test_include() {
        let template = r#"{{content}}"#;
        let script = r#"
            content = include("LICENSE")
        "#;

        let mut mll = Mll::new();
        mll.set_template(template.to_string());

        let render = mll.render_with_lua(script);

        let expected = fs::read_to_string("LICENSE").unwrap();

        assert_eq!(expected, render.unwrap());
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[should_panic(expected = "No such file or directory (os error 2)")]
    fn test_include_panic() {
        let template = r#"{{content}}"#;
        let script = r#"
            content = include("LICENSE1")
        "#;

        let mut mll = Mll::new();
        mll.set_template(template.to_string());

        let _ = mll.render_with_lua(script);
    }
}
