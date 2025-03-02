mod builtin;
mod builtins;
mod clip;
mod error;
#[cfg(target_os = "windows")]
mod global_memory;
mod html;
mod utils;
#[cfg(target_os = "windows")]
mod win_clipboard;

use std::{fs::read_to_string, io::Read, path::PathBuf};

use clap::Parser;
use clip::{Clip, Clipboard, ClipboardFormat};
use html::{
    create_html_for_clipboard, html_handle_to_string, lua_table_to_html_list,
    lua_table_to_html_table, parse_html, rc_dom_to_lua_table,
};
use mlua::Value;

#[derive(Debug, Parser, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "FILE")]
    file: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    // determine script
    let may_path = &args.file;
    let script = if may_path.is_none() {
        let stdin = std::io::stdin();
        let mut buffer = String::new();
        let mut lock = stdin.lock();
        match lock.read_to_string(&mut buffer) {
            Ok(_) => buffer,
            Err(e) => panic!("Error reading from stdin: {}", e),
        }
    } else {
        let path = may_path.clone().unwrap();
        if path.exists() {
            read_to_string(path).unwrap()
        } else {
            panic!("File not found");
        }
    };

    let mut clip = Clipboard::new();

    let lua = mlua::Lua::new();
    let _ = builtin::init(&lua).unwrap();

    let format = clip.determine_format().unwrap();

    match format {
        ClipboardFormat::Html(_) => {
            let table = lua.create_table().unwrap();

            let html = clip.get_data(&format).unwrap().to_string();
            table.set("raw", html.clone()).unwrap();

            let text = clip
                .get_data(&ClipboardFormat::Text("".to_string()))
                .unwrap()
                .to_string();
            table.set("text", text.clone()).unwrap();

            let html = Clipboard::get_html(&html);
            table.set("html", html.clone()).unwrap();

            let dom = parse_html(&html);
            let parsed_table = rc_dom_to_lua_table(&lua, dom);
            table.set("parsed", parsed_table).unwrap();

            lua.globals().set("qlp", table).unwrap();
        }
        ClipboardFormat::Text(_) => {
            let table = lua.create_table().unwrap();

            let text = clip.get_data(&format).unwrap().to_string();
            table.set("raw", text.clone()).unwrap();

            table.set("text", text.clone()).unwrap();

            lua.globals().set("qlp", table).unwrap();
        }
    }

    // execute lua script
    match lua.load(script).exec() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
        }
    }

    // set clipboard
    let current_table = lua.globals().get::<mlua::Table>("qlp").unwrap();

    match current_table.get::<Value>("result") {
        Ok(value) => match value {
            Value::String(s) => clip
                .set_data(&ClipboardFormat::Text(s.to_string_lossy()))
                .unwrap(),
            Value::Nil => {
                // NOP
            }
            Value::Boolean(b) => clip
                .set_data(&ClipboardFormat::Text(b.to_string()))
                .unwrap(),
            Value::Integer(n) => clip
                .set_data(&ClipboardFormat::Text(n.to_string()))
                .unwrap(),
            Value::Number(n) => clip
                .set_data(&ClipboardFormat::Text(n.to_string()))
                .unwrap(),
            Value::Table(t) => {
                // TODO: list
                let handles = vec![lua_table_to_html_table(&lua, &t)];
                // let handles = lua_table_to_html_list(&lua, &t);
                let html_handle = create_html_for_clipboard(handles);
                let html = html_handle_to_string(&html_handle);
                clip.set_data(&ClipboardFormat::Html(html)).unwrap();
            }
            _ => {
                // NOP
            }
        },
        Err(_) => {}
    }

    match current_table.get::<Value>("result_html_raw") {
        Ok(value) => match value {
            Value::String(s) => {
                let parsed = parse_html(&s.to_string_lossy());
                let html_handle = create_html_for_clipboard(vec![parsed.document]);
                let html_str = html_handle_to_string(&html_handle);
                clip.set_data(&ClipboardFormat::Html(html_str)).unwrap();
            }
            _ => {
                // NOP
            }
        },
        Err(_) => {}
    }
}
