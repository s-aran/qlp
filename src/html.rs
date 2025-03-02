use std::io::Cursor;

use html5ever::{
    Attribute, parse_document,
    serialize::{HtmlSerializer, Serialize, SerializeOpts, TraversalScope},
};
use markup5ever::{QualName, local_name, namespace_url, ns};
use markup5ever_rcdom::{Handle, Node, NodeData, RcDom, SerializableHandle};

use mlua::{Lua, Table, Value};
use xml5ever::tendril::{Tendril, TendrilSink};

#[derive(Debug)]
struct Working {
    table_stack: Vec<Handle>,
    body_topnode: Option<Handle>,
}

impl Default for Working {
    fn default() -> Self {
        Self {
            table_stack: Default::default(),
            body_topnode: Default::default(),
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
        if working.body_topnode.is_none() {
            working.body_topnode = Some(child.clone());
        }

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

    if working.table_stack.len() <= 0 {
        let cell = working.body_topnode.unwrap();
        let row_table = lua.create_table().unwrap();
        let data_table = lua.create_table().unwrap();
        data_table.set("text", get_text(&cell)).unwrap();
        if let Some(href) = get_anchor_href(&cell) {
            data_table.set("href", href).unwrap();
        }

        // lua to start arrays with index 1
        row_table.set(1, data_table).unwrap();
        table.set(1, row_table).unwrap();
    }

    while working.table_stack.len() > 0 {
        let table_handle = working.table_stack.pop().unwrap();
        for (row_i, row) in get_rows(&table_handle).iter().enumerate() {
            let row_table = lua.create_table().unwrap();

            for (column_n, cell) in get_data(&row).iter().enumerate() {
                // Table
                //   +-- row_n : (table)
                //   |            +-- column_n: (table)
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
        print!("{:indent$}", " ", indent = (indent * 2 + 1) as usize);
        if value.is_table() {
            println!("{:?}:", key);
            print_table(value.as_table().unwrap(), indent + 1);
        } else {
            println!("{:?}: {:?}", key, value);
        }
    }
}

fn create_html() -> Handle {
    Node::new(NodeData::Element {
        name: QualName::new(None, ns!(html), local_name!("html")),
        attrs: vec![].into(),
        template_contents: None.into(),
        mathml_annotation_xml_integration_point: false,
    })
}

fn create_body() -> Handle {
    Node::new(NodeData::Element {
        name: QualName::new(None, ns!(html), local_name!("body")),
        attrs: vec![].into(),
        template_contents: None.into(),
        mathml_annotation_xml_integration_point: false,
    })
}

fn create_meta<T>(key: T, value: T) -> Handle
where
    T: ToString,
{
    Node::new(NodeData::Element {
        name: QualName::new(None, ns!(html), local_name!("meta")),
        attrs: vec![Attribute {
            name: QualName::new(None, ns!(), key.to_string().into()),
            value: value.to_string().into(),
        }]
        .into(),
        template_contents: None.into(),
        mathml_annotation_xml_integration_point: false,
    })
}

fn create_comment<T>(text: T) -> Handle
where
    T: ToString,
{
    Node::new(NodeData::Comment {
        contents: text.to_string().into(),
    })
}

fn create_table() -> Handle {
    Node::new(NodeData::Element {
        name: QualName::new(None, ns!(html), local_name!("table")),
        attrs: vec![].into(),
        template_contents: None.into(),
        mathml_annotation_xml_integration_point: false,
    })
}

fn create_tr() -> Handle {
    Node::new(NodeData::Element {
        name: QualName::new(None, ns!(html), local_name!("tr")),
        attrs: vec![].into(),
        template_contents: None.into(),
        mathml_annotation_xml_integration_point: false,
    })
}

fn create_td() -> Handle {
    Node::new(NodeData::Element {
        name: QualName::new(None, ns!(html), local_name!("td")),
        attrs: vec![].into(),
        template_contents: None.into(),
        mathml_annotation_xml_integration_point: false,
    })
}

fn create_a(href: String) -> Handle {
    Node::new(NodeData::Element {
        name: QualName::new(None, ns!(html), local_name!("a")),
        attrs: vec![Attribute {
            name: QualName::new(None, ns!(), local_name!("href")),
            value: href.into(),
        }]
        .into(),
        template_contents: None.into(),
        mathml_annotation_xml_integration_point: false,
    })
}

fn create_text<T>(text: T) -> Handle
where
    T: ToString,
{
    Node::new(NodeData::Text {
        contents: Tendril::from(text.to_string()).into(),
    })
}

fn create_ul() -> Handle {
    Node::new(NodeData::Element {
        name: QualName::new(None, ns!(html), local_name!("ul")),
        attrs: vec![].into(),
        template_contents: None.into(),
        mathml_annotation_xml_integration_point: false,
    })
}

fn create_ol() -> Handle {
    Node::new(NodeData::Element {
        name: QualName::new(None, ns!(html), local_name!("ol")),
        attrs: vec![].into(),
        template_contents: None.into(),
        mathml_annotation_xml_integration_point: false,
    })
}

fn create_li() -> Handle {
    Node::new(NodeData::Element {
        name: QualName::new(None, ns!(html), local_name!("li")),
        attrs: vec![].into(),
        template_contents: None.into(),
        mathml_annotation_xml_integration_point: false,
    })
}

pub fn lua_table_to_html_table(_: &Lua, value: &Table) -> Handle {
    let table = create_table();

    for row_kv in value.pairs::<Value, Value>() {
        let (row_key, row_value) = row_kv.unwrap();
        let row_table = row_value.as_table().unwrap();

        let tr = create_tr();

        for cell_kv in row_table.pairs::<Value, Value>() {
            let (column_key, column_value) = cell_kv.unwrap();
            println!("{:?} {:?}", column_key, column_value);
            let column_table = column_value.as_table().unwrap();

            let td = create_td();

            let text = column_table.get::<String>("text").unwrap();
            let href = column_table.get::<String>("href");

            if href.is_ok() {
                let a = create_a(href.unwrap());
                a.children.borrow_mut().push(create_text(text));

                td.children.borrow_mut().push(a);
            } else {
                td.children.borrow_mut().push(create_text(text));
            }

            tr.children.borrow_mut().push(td);
        }

        table.children.borrow_mut().push(tr);
    }

    return table;
}

fn lua_table_to_ul(lua: &Lua, value: &Table) -> Handle {
    let ul = create_ul();

    if value.contains_key("text").unwrap() {
        let li = create_li();
        if value.contains_key("href").unwrap() {
            let a = create_a(value.get::<String>("href").unwrap());
            let text = create_text(value.get::<String>("text").unwrap());
            a.children.borrow_mut().push(text);
            li.children.borrow_mut().push(a);
        } else {
            let text = create_text(value.get::<String>("text").unwrap());
            li.children.borrow_mut().push(text);
        }
        ul.children.borrow_mut().push(li);
    }

    for cell_kv in value.pairs::<Value, Value>() {
        let (column_key, column_value) = cell_kv.unwrap();

        if !column_value.is_table() {
            continue;
        }

        let column_table = column_value.as_table().unwrap();

        let created_ul = lua_table_to_ul(lua, column_table);
        let children = ul.children.borrow().clone();
        let found_ul = children.iter().find(|e| {
            match e.data {
                NodeData::Element { ref name, .. } => {
                    if name.local.as_ref() == "ul" {
                        return true;
                    }
                }
                _ => {
                    return false;
                }
            }

            return false;
        });

        if found_ul.is_some() {
            let mut ul_children = found_ul.unwrap().children.borrow_mut();
            ul_children.append(&mut created_ul.children.borrow_mut());
        } else {
            ul.children.borrow_mut().push(created_ul);
        }
    }

    return ul;
}

pub fn lua_table_to_html_list(lua: &Lua, value: &Table) -> Handle {
    print_table(value, 0);
    println!("--------------------------------------------------------------------------------");

    let mut result: Option<Handle> = None;

    for row_kv in value.pairs::<Value, Value>() {
        let (row_key, row_value) = row_kv.unwrap();
        if result.is_some() {
            let ul = Some(lua_table_to_ul(lua, row_value.as_table().unwrap()));
            result
                .clone()
                .unwrap()
                .children
                .borrow_mut()
                .append(&mut ul.unwrap().children.borrow_mut());
        } else {
            result = Some(lua_table_to_ul(lua, row_value.as_table().unwrap()));
        }
    }

    return result.unwrap();
}

pub fn create_html_for_clipboard(contents: Vec<Handle>) -> Handle {
    let html = create_html();
    let body = create_body();
    let meta_charset = create_meta("charset", "utf-8");

    let comment_start_fragment = create_comment("StartFragment");
    let comment_end_fragment = create_comment("EndFragment");

    // like Google Chrome
    let mut content = vec![];
    content.push(comment_start_fragment);
    content.push(meta_charset);
    content.extend(contents);
    content.push(comment_end_fragment);

    body.children.borrow_mut().append(&mut content);
    html.children.borrow_mut().push(body);

    html
}

pub fn html_handle_to_string(handle: &Handle) -> String {
    let mut buf = vec![];
    let mut serializer = HtmlSerializer::new(
        &mut buf,
        SerializeOpts {
            create_missing_parent: true,
            ..Default::default()
        },
    );
    let serializable = SerializableHandle::from(handle.clone());
    serializable
        .serialize(&mut serializer, TraversalScope::IncludeNode)
        .unwrap();
    String::from_utf8(buf).unwrap()
}

#[cfg(test)]
mod tests {

    use mlua::Table;

    use super::*;

    #[test]
    fn test_parse_html() {
        let html = r#"<html>
<body>
<!--StartFragment--><google-sheets-html-origin><style type="text/css"><!--td {border: 1px solid #cccccc;}br {mso-data-placement:same-cell;}--></style><table xmlns="http://www.w3.org/1999/xhtml" cellspacing="0" cellpadding="0" dir="ltr" border="1" style="table-layout:fixed;font-size:10pt;font-family:Arial;width:0px;border-collapse:collapse;border:none" data-sheets-root="1" data-sheets-baot="1"><colgroup><col width="100"/><col width="100"/><col width="100"/></colgroup><tbody><tr style="height:21px;"><td style="border-top:1px solid #000000;border-right:1px solid #000000;border-bottom:1px solid #000000;border-left:1px solid #000000;overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;">aa</td><td style="border-top:1px solid #000000;border-right:1px solid #000000;border-bottom:1px solid #000000;overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;font-weight:bold;">bb</td><td style="overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-decoration:underline;color:#1155cc;"><a class="in-cell-link" href="https://google.com/" target="_blank">cc</a></td></tr></tbody></table><!--EndFragment-->
</body>
</html>"#;

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
        let html = r#"<html>
<body>
<!--StartFragment--><google-sheets-html-origin><style type="text/css"><!--td {border: 1px solid #cccccc;}br {mso-data-placement:same-cell;}--></style><table xmlns="http://www.w3.org/1999/xhtml" cellspacing="0" cellpadding="0" dir="ltr" border="1" style="table-layout:fixed;font-size:10pt;font-family:Arial;width:0px;border-collapse:collapse;border:none" data-sheets-root="1" data-sheets-baot="1"><colgroup><col width="100"/><col width="100"/></colgroup><tbody><tr style="height:21px;"><td style="border-left:1px solid #000000;overflow:hidden;padding:2px 3px 2px 3px;vertical-align:bottom;text-decoration:underline;color:#1155cc;"><a class="in-cell-link" href="https://google.com/" target="_blank">うう</a></td><td style="border-right:1px solid transparent;overflow:visible;padding:2px 0px 2px 0px;vertical-align:bottom;"><div style="white-space:nowrap;overflow:hidden;position:relative;width:297px;left:3px;"><div style="float:left;"><span style="font-size:10pt;font-family:Arial;font-style:normal;text-decoration:underline;text-decoration-skip-ink:none;-webkit-text-decoration-skip:none;color:#1155cc;"><a class="in-cell-link" target="_blank" href="https://example.com/">Example Domain</a></span><span style="font-size:10pt;font-family:Arial;font-style:normal;">tps://example.com/</span></div></div></td></tr></tbody></table><!--EndFragment-->
</body>
</html>"#;

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

    #[test]
    fn test_parse_html3() {
        let html = r#"<html>
<body>
<!--StartFragment--><style type="text/css"><!--td {border: 1px solid #cccccc;}br {mso-data-placement:same-cell;}--></style><span style="font-size:10pt;font-family:Arial;font-style:normal;" data-sheets-root="1"><span style="font-size:10pt;font-family:Arial;font-style:normal;text-decoration:underline;text-decoration-skip-ink:none;-webkit-text-decoration-skip:none;color:#1155cc;"><a class="in-cell-link" target="_blank" href="https://example.com/">Example Domain</a></span><span style="font-size:10pt;font-family:Arial;font-style:normal;">tps://example.com/</span></span><!--EndFragment-->
</body>
</html"#;

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
        assert_eq!(
            "Example Domaintps://example.com/",
            actual_cells.get::<String>("text").unwrap()
        );
        assert_eq!(
            "https://example.com/",
            actual_cells.get::<String>("href").unwrap()
        );
    }

    #[test]
    fn test_to_html_table() {
        let lua = mlua::Lua::new();

        let table = lua.create_table().unwrap();

        let row1 = lua.create_table().unwrap();
        let cell1_1 = lua.create_table().unwrap();
        cell1_1.set("text", "aa").unwrap();

        row1.set(1, cell1_1).unwrap();

        let row2 = lua.create_table().unwrap();
        let cell2_1 = lua.create_table().unwrap();
        cell2_1.set("text", "bb").unwrap();

        row2.set(1, cell2_1).unwrap();

        let row3 = lua.create_table().unwrap();
        let cell3_1 = lua.create_table().unwrap();
        cell3_1.set("text", "cc").unwrap();
        cell3_1.set("href", "https://example.com/").unwrap();

        row3.set(1, cell3_1).unwrap();

        table.set(1, row1).unwrap();
        table.set(2, row2).unwrap();
        table.set(3, row3).unwrap();

        let actual_table = lua_table_to_html_table(&lua, &table);
        let actual_html = create_html_for_clipboard(vec![actual_table]);

        println!("{:?}", actual_html);

        println!("{}", html_handle_to_string(&actual_html));

        assert!(false)
    }

    #[test]
    fn test_list() {
        let html = r#"<html>
<body>
<!--StartFragment--><meta charset="utf-8"><b style="font-weight:normal;" id="docs-internal-guid-81b70dd7-7fff-f183-d406-52450c91b0e8"><ul style="margin-top:0;margin-bottom:0;padding-inline-start:48px;"><li dir="ltr" style="list-style-type:disc;font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;" aria-level="1"><p dir="ltr" style="line-height:1.38;margin-top:0pt;margin-bottom:0pt;" role="presentation"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">hoge</span></p></li><ul style="margin-top:0;margin-bottom:0;padding-inline-start:48px;"><li dir="ltr" style="list-style-type:circle;font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;" aria-level="2"><p dir="ltr" style="line-height:1.38;margin-top:0pt;margin-bottom:0pt;" role="presentation"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">piyo</span></p></li><ul style="margin-top:0;margin-bottom:0;padding-inline-start:48px;"><li dir="ltr" style="list-style-type:square;font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;" aria-level="3"><p dir="ltr" style="line-height:1.38;margin-top:0pt;margin-bottom:0pt;" role="presentation"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">fuga</span></p></li></ul><li dir="ltr" style="list-style-type:circle;font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;" aria-level="2"><p dir="ltr" style="line-height:1.38;margin-top:0pt;margin-bottom:0pt;" role="presentation"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">moge</span></p></li></ul><li dir="ltr" style="list-style-type:disc;font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;" aria-level="1"><p dir="ltr" style="line-height:1.38;margin-top:0pt;margin-bottom:0pt;" role="presentation"><span style="font-size:11pt;font-family:Arial,sans-serif;color:#000000;background-color:transparent;font-weight:400;font-style:normal;font-variant:normal;text-decoration:none;vertical-align:baseline;white-space:pre;white-space:pre-wrap;">mogera</span></p></li></ul><br /></b><!--EndFragment-->
</body>
</html>"#;
    }

    #[test]
    fn test_list_2() {
        let lua = mlua::Lua::new();

        let table = lua.create_table().unwrap();

        let row1 = lua.create_table().unwrap();
        row1.set("text", "foo").unwrap();

        let cell1_1 = lua.create_table().unwrap();
        cell1_1.set("text", "aa").unwrap();

        row1.set(1, cell1_1).unwrap();

        let cell1_2 = lua.create_table().unwrap();
        cell1_2.set("text", "cc").unwrap();

        row1.set(2, cell1_2).unwrap();

        let row2 = lua.create_table().unwrap();
        row2.set("text", "bar").unwrap();

        let cell2_1 = lua.create_table().unwrap();
        cell2_1.set("text", "bb").unwrap();

        row2.set(1, cell2_1).unwrap();

        let cell2_2 = lua.create_table().unwrap();
        cell2_2.set("text", "dd").unwrap();

        row2.set(2, cell2_2).unwrap();

        table.set(1, row1).unwrap();
        table.set(2, row2).unwrap();

        // let actual_html = merge_ul(lua_table_to_html_list(&lua, &table));
        let actual_html = lua_table_to_html_list(&lua, &table);
        let actual = html_handle_to_string(&actual_html);
        println!("{:?}", actual);

        let expected = "<ul><li>foo</li><ul><li>aa</li><li>cc</li></ul><li>bar</li><ul><li>bb</li><li>dd</li></ul></ul>";

        assert_eq!(expected, actual);
    }
}
