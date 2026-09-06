use html5ever::tendril::StrTendril;
use markup5ever_rcdom::{Handle, NodeData};
use mlua::{Function, Lua};

use crate::html::parse_html;

use super::builtin::BuiltinFunction;

pub struct SlackHtmlToMarkdown;

impl BuiltinFunction for SlackHtmlToMarkdown {
    fn get_name(&self) -> &str {
        "slack_html_to_markdown"
    }

    fn get_function(&self, lua: &Lua) -> Function {
        lua.create_function(|_, html: String| Ok(slack_html_to_markdown(&html)))
            .unwrap()
    }
}

pub fn slack_html_to_markdown(html: &str) -> String {
    let dom = parse_html(&html.to_string());
    normalize_markdown(&render_node(&dom.document, &RenderContext::default()))
}

#[derive(Debug, Clone, Default)]
struct RenderContext {
    inline_code: bool,
    preformatted: bool,
    ordered_depth: usize,
}

fn render_node(handle: &Handle, context: &RenderContext) -> String {
    match handle.data {
        NodeData::Document => render_children(handle, context),
        NodeData::Text { ref contents } => render_text(&contents.borrow(), context),
        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            let tag_name = name.local.as_ref();
            match tag_name {
                "br" => "\n".to_string(),
                "html" | "body" | "span" => render_children(handle, context),
                "p" | "div" | "section" => render_block(handle, context),
                "strong" | "b" => wrap_inline("**", render_children(handle, context)),
                "em" | "i" => wrap_inline("*", render_children(handle, context)),
                "s" | "strike" | "del" => wrap_inline("~~", render_children(handle, context)),
                "code" => {
                    if context.preformatted {
                        render_raw_text(handle)
                    } else {
                        let mut child_context = context.clone();
                        child_context.inline_code = true;
                        format!(
                            "`{}`",
                            escape_inline_code(&render_children(handle, &child_context))
                        )
                    }
                }
                "pre" => {
                    format!(
                        "\n```\n{}\n```\n",
                        render_raw_text(handle).trim_matches('\n')
                    )
                }
                "a" => {
                    let href = get_attr(handle, "href");
                    let label = render_children(handle, context).trim().to_string();
                    match (href, label.is_empty()) {
                        (Some(href), false) if href != label => {
                            format!("[{}]({})", escape_link_label(&label), href)
                        }
                        (Some(href), _) => format!("<{}>", href),
                        (None, _) => label,
                    }
                }
                "blockquote" => {
                    let inner = render_children(handle, context);
                    format!("\n{}\n", quote_block(&inner))
                }
                "ul" => render_list(handle, context, false),
                "ol" => render_list(handle, context, true),
                "li" => render_children(handle, context),
                _ => {
                    let _ = attrs;
                    render_children(handle, context)
                }
            }
        }
        _ => String::new(),
    }
}

fn render_children(handle: &Handle, context: &RenderContext) -> String {
    handle
        .children
        .borrow()
        .iter()
        .map(|child| render_node(child, context))
        .collect::<Vec<_>>()
        .join("")
}

fn render_text(text: &StrTendril, context: &RenderContext) -> String {
    let raw = text.as_ref();
    if context.preformatted || context.inline_code {
        return raw.to_string();
    }

    collapse_whitespace(raw)
}

fn render_raw_text(handle: &Handle) -> String {
    match handle.data {
        NodeData::Text { ref contents } => contents.borrow().to_string(),
        _ => handle
            .children
            .borrow()
            .iter()
            .map(render_raw_text)
            .collect::<Vec<_>>()
            .join(""),
    }
}

fn render_block(handle: &Handle, context: &RenderContext) -> String {
    let inner = render_children(handle, context);
    if inner.trim().is_empty() {
        return String::new();
    }

    format!("{}\n", inner.trim())
}

fn render_list(handle: &Handle, context: &RenderContext, ordered: bool) -> String {
    let mut result = String::new();
    let mut index = 1;

    let mut child_context = context.clone();
    if ordered {
        child_context.ordered_depth += 1;
    }

    for child in handle.children.borrow().iter() {
        if !is_element(child, "li") {
            continue;
        }

        let item = render_children(child, &child_context);
        let item = item.trim();
        if item.is_empty() {
            continue;
        }

        let marker = if ordered {
            format!("{}.", index)
        } else {
            "-".to_string()
        };
        result.push_str(&format!("{} {}\n", marker, indent_continuation(item)));
        index += 1;
    }

    result.push('\n');
    result
}

fn wrap_inline(marker: &str, value: String) -> String {
    let value = value.trim();
    if value.is_empty() {
        String::new()
    } else {
        format!("{}{}{}", marker, value, marker)
    }
}

fn get_attr(handle: &Handle, name: &str) -> Option<String> {
    if let NodeData::Element { ref attrs, .. } = handle.data {
        for attr in attrs.borrow().iter() {
            if &*attr.name.local == name {
                return Some(attr.value.to_string());
            }
        }
    }

    None
}

fn is_element(handle: &Handle, tag_name: &str) -> bool {
    if let NodeData::Element { ref name, .. } = handle.data {
        return &*name.local == tag_name;
    }

    false
}

fn collapse_whitespace(value: &str) -> String {
    let mut result = String::new();
    let mut previous_was_space = false;

    for ch in value.chars() {
        if ch.is_whitespace() {
            if !previous_was_space {
                result.push(' ');
            }
            previous_was_space = true;
        } else {
            result.push(ch);
            previous_was_space = false;
        }
    }

    result
}

fn escape_inline_code(value: &str) -> String {
    value.replace('`', "\\`")
}

fn escape_link_label(value: &str) -> String {
    value.replace('[', "\\[").replace(']', "\\]")
}

fn quote_block(value: &str) -> String {
    value
        .trim()
        .lines()
        .map(|line| format!("> {}", line.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn indent_continuation(value: &str) -> String {
    value
        .lines()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                line.trim().to_string()
            } else {
                format!("  {}", line.trim())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_markdown(value: &str) -> String {
    let mut result = String::new();
    let mut blank_lines = 0;

    for line in value.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            blank_lines += 1;
            if blank_lines <= 1 {
                result.push('\n');
            }
            continue;
        }

        blank_lines = 0;
        result.push_str(line.trim_start());
        result.push('\n');
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::builtin::BuiltinFunction;

    #[test]
    fn test_slack_html_to_markdown() {
        let html = r#"
            <div>
                <span>Hello </span><b>world</b><br>
                <a href="https://example.com/">Example</a>
                <code>foo()</code>
            </div>
            <blockquote>quoted<br>line</blockquote>
            <pre>let x = 1;</pre>
            <ul><li>one</li><li>two</li></ul>
        "#;

        let actual = slack_html_to_markdown(html);
        let expected = r#"Hello **world**
[Example](https://example.com/) `foo()`

> quoted
> line

```
let x = 1;
```
- one
- two"#;

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_slack_html_to_markdown_from_lua() {
        let lua = Lua::new();
        SlackHtmlToMarkdown {}.set_function(&lua).unwrap();
        lua.globals()
            .set(
                "html",
                r#"<div><b>Slack</b> <a href="https://example.com/">link</a></div>"#,
            )
            .unwrap();
        lua.load("result = slack_html_to_markdown(html)")
            .exec()
            .unwrap();

        let actual = lua.globals().get::<String>("result").unwrap();
        assert_eq!("**Slack** [link](https://example.com/)", actual);
    }
}
