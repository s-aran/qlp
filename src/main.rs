mod clip;
mod error;
mod global_memory;
mod html;
mod win_clipboard;

use std::{fs::read_to_string, io::Read, path::PathBuf, rc::Rc};

use clap::Parser;
use clip::{Clip, ClipboardFormat, clipboard::Clipboard};
use html::{parse_html, rc_dom_to_lua_table};

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
    let format = clip.determine_format().unwrap();

    match format {
        ClipboardFormat::Html(_) => {
            let table = lua.create_table().unwrap();

            let html = clip.get_data(&format).unwrap().to_string();
            let html = Clipboard::get_html(&html);

            table.set("html", html.clone()).unwrap();

            let dom = parse_html(&html);
            let parsed_table = rc_dom_to_lua_table(&lua, dom);
            table.set("parsed", parsed_table).unwrap();

            lua.globals().set("qlp", table).unwrap();
        }
        ClipboardFormat::Text(_) => {
            let text = clip.get_data(&format).unwrap().to_string();
            lua.globals().set("qlp", text).unwrap();
        }
    }

    lua.load(script).exec().unwrap();
}
