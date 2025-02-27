use std::io::Cursor;

use html5ever::parse_document;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use xml5ever::tendril::TendrilSink;

#[derive(Debug)]
struct Working {
    table_stack: Vec<Handle>,
}

impl Default for Working {
    fn default() -> Self {
        Self {
            table_stack: Default::default(),
        }
    }
}

pub fn parse_html(html: &String) -> RcDom {
    let b = html.clone().into_bytes();
    let mut c = Cursor::new(b);
    let rcdom_sink = RcDom::default();
    let opts = Default::default();

    let dom = parse_document(rcdom_sink, opts)
        .from_utf8()
        .read_from(&mut c)
        .unwrap();

    dom
}

fn walk(handle: &Handle, working: &mut Working) {
    if let NodeData::Element { ref name, .. } = handle.data {
        match name.local.as_ref() {
            "table" => {
                working.table_stack.push(handle.clone());
            }
            "tr" => {}
            "th" => {}
            "td" => {}
            _ => {}
        }
    }

    let children = handle.children.borrow();
    for child in children.iter() {
        walk(child, working);
    }
}

fn get_rows(table: &Handle) -> Vec<Handle> {
    let mut rows = Vec::new();

    let children = table.children.borrow();
    for child in children.iter() {
        if let NodeData::Element { ref name, .. } = child.data {
            if name.local.as_ref() == "tr" {
                rows.push(child.clone());
            } else if name.local.as_ref() == "thead" || name.local.as_ref() == "tbody" {
                rows.extend(get_rows(child));
            }
        }
    }

    rows
}

fn get_header(row: &Handle) -> Vec<Handle> {
    let mut items = Vec::new();

    let children = row.children.borrow();
    for child in children.iter() {
        if let NodeData::Element { ref name, .. } = child.data {
            if name.local.as_ref() == "th" {
                items.push(child.clone());
            }
        }
    }

    items
}

fn get_data(row: &Handle) -> Vec<Handle> {
    let mut items = Vec::new();

    let children = row.children.borrow();
    for child in children.iter() {
        if let NodeData::Element { ref name, .. } = child.data {
            if name.local.as_ref() == "td" {
                items.push(child.clone());
            }
        }
    }

    items
}

fn get_anchor_href(handle: &Handle) -> Option<String> {
    let children = handle.children.borrow();

    for child in children.iter() {
        if let NodeData::Element {
            ref name,
            ref attrs,
            ..
        } = child.data
        {
            if name.local.as_ref() == "a" {
                for attr in attrs.borrow().iter() {
                    if attr.name.local.as_ref() == "href" {
                        return Some(attr.value.to_string());
                    }
                }
            } else {
                return get_anchor_href(child);
            }
        }
    }

    None
}

fn get_text(handle: &Handle) -> String {
    let mut text = String::new();

    let children = handle.children.borrow();
    for child in children.iter() {
        if let NodeData::Text { ref contents } = child.data {
            text.push_str(contents.borrow().as_ref());
        } else {
            text.push_str(&get_text(child));
        }
    }

    text
}

pub fn rc_dom_to_lua_table(lua: &mlua::Lua, dom: RcDom) -> mlua::Table {
    let mut working = Working::default();
    walk(&dom.document, &mut working);

    let table = lua.create_table().unwrap();

    while working.table_stack.len() > 0 {
        let table_handle = working.table_stack.pop().unwrap();
        for (row_i, row) in get_rows(&table_handle).iter().enumerate() {
            let row_table = lua.create_table().unwrap();

            for (column_n, cell) in get_data(&row).iter().enumerate() {
                // Table
                //   +-- row_n : ...
                //   |            +-- column_n: ...
                //   |            |              +-- (if anchor) href: url
                //   |            |              `-- text: text
                //   |            +-- :
                //   |            +-- :
                //   |            `-- :
                //   +-- :
                //   +-- :
                //   `-- :

                let data_table = lua.create_table().unwrap();
                data_table.set("text", get_text(cell)).unwrap();
                if let Some(href) = get_anchor_href(cell) {
                    data_table.set("href", href).unwrap();
                }

                // lua to start arrays with index 1
                row_table.set(column_n + 1, data_table).unwrap();
            }

            // lua to start arrays with index 1
            table.set(row_i + 1, row_table).unwrap();
        }
    }

    table
}

fn print_table(table: &mlua::Table, indent: u32) {
    for pair in table.pairs::<mlua::Value, mlua::Value>() {
        let (key, value) = pair.unwrap();
        print!("{:indent$}", " ", indent = (indent * 2) as usize);
        if value.is_table() {
            println!("{:?}:", key);
            print_table(value.as_table().unwrap(), indent + 1);
        } else {
            println!("{:?}: {:?}", key, value);
        }
    }
}

// test
#[cfg(test)]
mod tests {
    use mlua::Table;

    use crate::clip::Clip;
    use crate::clip::Clipboard;

    use super::*;

    #[test]
    fn test_parse_html() {
        let clipboard = r#"Version:0.9
StartHTML:0000000105
EndHTML:0000001269
StartFragment:0000000141
EndFragment:0000001233
<html>
<body>
<!--StartFragment--><google-sheets-html-origin><style type="text/css"><!--td {border: 1px solid #cccccc;}br {mso-data-placement:same-cell;}--></style><table xmlns="http://www.w3.org/1999/xhtml" cellspacing="0" cellpadding="0" dir="ltr" border="1" style="table-layout:fixed;font-size:10pt;font-family:Arial;width:0px;border-collapse:collapse;border:none" data-sheets-root="1" data-sheets-baot="1"><colgroup><col width="100"/><col width="100"/><col width="100"/></colgroup><tbody><tr style="height:21px;"><td style="border-top:1px solid #000000;border-right:1px solid #000000;border-bottom:1px solid #000000;border-left:1px solid #000000;overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;">aa</td><td style="border-top:1px solid #000000;border-right:1px solid #000000;border-bottom:1px solid #000000;overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;font-weight:bold;">bb</td><td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-decoration:underline;color:#1155cc;"><a class="in-cell-link" href="https://google.com/" target="_blank">cc</a></td></tr></tbody></table><!--EndFragment-->
</body>
</html>"#;
        let html = Clipboard::get_html(&clipboard);

        let dom = parse_html(&html.to_string());
        assert_eq!(dom.document.children.borrow().len(), 1);

        println!("{:?}", dom.document);
        println!("--------------------");

        let lua = mlua::Lua::new();

        let table = rc_dom_to_lua_table(&lua, dom);
        print_table(&table, 0);

        // assert table length
        let actual_table = table;
        assert_eq!(1, actual_table.len().unwrap());

        // assert row length
        let actual_rows = &actual_table.get::<Table>(1).unwrap();
        assert_eq!(3, actual_rows.len().unwrap());

        // assert cell 1
        let actual_cells = &actual_rows.get::<Table>(1).unwrap();
        assert_eq!("aa", actual_cells.get::<String>("text").unwrap());

        // assert cell 2
        let actual_cells = &actual_rows.get::<Table>(2).unwrap();
        assert_eq!("bb", actual_cells.get::<String>("text").unwrap());

        // assert cell 3
        let actual_cells = &actual_rows.get::<Table>(3).unwrap();
        assert_eq!("cc", actual_cells.get::<String>("text").unwrap());
        assert_eq!(
            "https://google.com/",
            actual_cells.get::<String>("href").unwrap()
        );
    }

    #[test]
    fn test_parse_html2() {
        let clipboard = r#"Version:0.9
StartHTML:0000000105
EndHTML:0000001485
StartFragment:0000000141
EndFragment:0000001449
<html>
<body>
<!--StartFragment--><google-sheets-html-origin><style type="text/css"><!--td {border: 1px solid #cccccc;}br {mso-data-placement:same-cell;}--></style><table xmlns="http://www.w3.org/1999/xhtml" cellspacing="0" cellpadding="0" dir="ltr" border="1" style="table-layout:fixed;font-size:10pt;font-family:Arial;width:0px;border-collapse:collapse;border:none" data-sheets-root="1" data-sheets-baot="1"><colgroup><col width="100"/><col width="100"/></colgroup><tbody><tr style="height:21px;"><td style="border-left:1px solid #000000;overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-decoration:underline;color:#1155cc;"><a class="in-cell-link" href="https://google.com/" target="_blank">うう</a></td><td style="border-right:1px solid transparent;overflow:visible;padding:2px 0px 2px 0px;vertical-align:bottom;"><div style="white-space:nowrap;overflow:hidden;position:relative;width:297px;left:3px;"><div style="float:left;"><span style="font-size:10pt;font-family:Arial;font-style:normal;text-decoration:underline;text-decoration-skip-ink:none;-webkit-text-decoration-skip:none;color:#1155cc;"><a class="in-cell-link" target="_blank" href="https://example.com/">Example Domain</a></span><span style="font-size:10pt;font-family:Arial;font-style:normal;">tps://example.com/</span></div></div></td></tr></tbody></table><!--EndFragment-->
</body>
</html>"#;
        let html = Clipboard::get_html(&clipboard);

        let dom = parse_html(&html.to_string());
        assert_eq!(dom.document.children.borrow().len(), 1);

        println!("{:?}", dom.document);
        println!("--------------------");

        let lua = mlua::Lua::new();

        let table = rc_dom_to_lua_table(&lua, dom);
        print_table(&table, 0);

        // assert table length
        let actual_table = table;
        assert_eq!(1, actual_table.len().unwrap());

        // assert row length
        let actual_rows = &actual_table.get::<Table>(1).unwrap();
        assert_eq!(2, actual_rows.len().unwrap());

        // assert cell 1
        let actual_cells = &actual_rows.get::<Table>(1).unwrap();
        assert_eq!("うう", actual_cells.get::<String>("text").unwrap());
        assert_eq!(
            "https://google.com/",
            actual_cells.get::<String>("href").unwrap()
        );

        // assert cell 2
        let actual_cells = &actual_rows.get::<Table>(2).unwrap();
        assert_eq!(
            "Example Domaintps://example.com/",
            actual_cells.get::<String>("text").unwrap()
        );
        assert_eq!(
            "https://example.com/",
            actual_cells.get::<String>("href").unwrap()
        );
    }
}
