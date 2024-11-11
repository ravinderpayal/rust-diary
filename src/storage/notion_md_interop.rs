//storage/notion_md_interop.rs
use notion::models::block::Block;
use notion::models::block::*;
use notion::models::text;
use notion::models::text::*;

use notion::models::block::Text;
use regex::Regex;

pub trait ToMarkdown {
    fn to_markdown(&self) -> String;
}

impl ToMarkdown for Block {
    fn to_markdown(&self) -> String {
        match self {
            Block::Paragraph { paragraph, .. } => paragraph_to_markdown(paragraph),
            Block::Heading1 { heading_1, .. } => format!("# {}\n", text_to_markdown(heading_1)),
            Block::Heading2 { heading_2, .. } => format!("## {}\n", text_to_markdown(heading_2)),
            Block::Heading3 { heading_3, .. } => format!("### {}\n", text_to_markdown(heading_3)),
            Block::BulletedListItem { bulleted_list_item, .. } => {
                format!("- {}\n", paragraph_to_markdown(bulleted_list_item))
            }
            Block::NumberedListItem { numbered_list_item, .. } => {
                format!("1. {}\n", paragraph_to_markdown(numbered_list_item))
            }
            Block::ToDo { to_do, .. } => todo_to_markdown(to_do),
            Block::Toggle { toggle, .. } => toggle_to_markdown(toggle),
            Block::Code { code, .. } => code_to_markdown(code),
            Block::Quote { quote, .. } => format!("> {}\n\n", paragraph_to_markdown(quote)),
            Block::Callout { callout, .. } => callout_to_markdown(callout),
            Block::Divider { .. } => "---\n".to_string(),
            // Add more block types as needed
            _ => String::new(), // Placeholder for unsupported block types
        }
    }
}

fn text_to_markdown(text: &notion::models::block::Text) -> String {
    text.rich_text.iter().map(rich_text_to_markdown).collect()
}

fn _nested_text_to_markdown(text: &notion::models::text::Text) -> String {
    text.content.clone() // rich_text.iter().map(rich_text_to_markdown).collect()
}

fn rich_text_to_markdown(rich_text_ref: &RichText) -> String {
    match rich_text_ref {
        RichText::Text { rich_text, text } => {
            format_rich_text(&rich_text, &text.content, text.link.as_ref())
        }
        RichText::Mention { rich_text, .. } => {
            // For mentions, we'll just use the plain text for now
            rich_text.plain_text.clone()
        }
        RichText::Equation { rich_text } => {
            format!("${}$", rich_text.plain_text)
        }
    }
}

fn vec_rich_text_to_markdown(rich_text_vec: &Vec<RichText>) -> String {
    rich_text_vec.iter().map(rich_text_to_markdown).collect()
}

fn format_rich_text(common: &RichTextCommon, content: &str, link: Option<&Link>) -> String {
    let mut formatted = content.to_string();

    if let Some(annotations) = &common.annotations {
        if annotations.bold.unwrap_or(false) {
            formatted = format!("**{}**", formatted);
        }
        if annotations.italic.unwrap_or(false) {
            formatted = format!("*{}*", formatted);
        }
        if annotations.strikethrough.unwrap_or(false) {
            formatted = format!("~~{}~~", formatted);
        }
        if annotations.code.unwrap_or(false) {
            formatted = format!("`{}`", formatted);
        }
        // Note: underline is not standard in Markdown, so we're skipping it
        // Color could be handled with custom HTML if needed
    }

    if let Some(link) = link {
        formatted = format!("[{}]({})", formatted, link.url);
    } else if let Some(href) = &common.href {
        formatted = format!("[{}]({})", formatted, href);
    }

    formatted
}

fn paragraph_to_markdown(paragraph: &TextAndChildren) -> String {
    let text = vec_rich_text_to_markdown(&paragraph.rich_text);
    format!("{}\n", text)
}

fn todo_to_markdown(todo: &ToDoFields) -> String {
    let checkbox = if todo.checked { "- [x]" } else { "- [ ]" };
    format!(
        "{} {}\n",
        checkbox,
        vec_rich_text_to_markdown(&todo.rich_text)
    )
}

fn toggle_to_markdown(toggle: &TextAndChildren) -> String {
    format!(
        "<details><summary>{}</summary>\n\n{}</details>\n\n",
        vec_rich_text_to_markdown(&toggle.rich_text),
        children_to_markdown(&toggle.children)
    )
}

fn code_to_markdown(code: &CodeFields) -> String {
    let language = &code.language;
    let code_text = vec_rich_text_to_markdown(&code.rich_text);
    format!("```{:?}\n{}\n```\n\n", language, code_text)
}

fn callout_to_markdown(callout: &Callout) -> String {
    let icon = match &callout.icon {
        notion::models::block::FileOrEmojiObject::Emoji { emoji } => emoji,
        _ => "",
    };
    format!(
        "> {} {}\n\n",
        icon,
        vec_rich_text_to_markdown(&callout.rich_text)
    )
}

fn children_to_markdown(children: &Option<Vec<Block>>) -> String {
    children.as_ref().map_or(String::new(), |blocks| {
        blocks.iter().map(|block| block.to_markdown()).collect()
    })
}

pub fn blocks_to_markdown(blocks: &[Block]) -> String {
    blocks.iter().map(|block| block.to_markdown()).collect()
}

// Mark-down to Notion Blocks

pub trait MarkdownToNotionBlocks {
    fn to_notion_blocks(&self) -> Vec<CreateBlock>;
}

impl MarkdownToNotionBlocks for str {
    fn to_notion_blocks(&self) -> Vec<CreateBlock> {
        let lines: Vec<&str> = self.lines().collect();
        let mut blocks = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let (block, lines_consumed) = process_lines(&lines[i..]);
            blocks.push(block);
            i += lines_consumed;
        }
        blocks
    }
}

fn process_lines(lines: &[&str]) -> (CreateBlock, usize) {
    let line = lines[0];
    /* let common = BlockCommon {
        id: Uuid::new_v4().to_string(),
        last_edited_time: DateTime::naive_local(&self),
        has_children: todo!(),
        created_by: UserCommoni,
        last_edited_by: todo!(),
        created_time: todo!(),
        // ..Default::default()
    };*/

    if line.starts_with("# ") {
        (create_heading(line, 1), 1)
    } else if line.starts_with("## ") {
        (create_heading(line, 2), 1)
    } else if line.starts_with("### ") {
        (create_heading(line, 3), 1)
    } else if line.starts_with("- [ ] ") || line.starts_with("- [x] ") {
        (create_todo(line), 1)
    } else if line.starts_with("- ") {
        (create_bulleted_list_item(line), 1)
    } else if line.starts_with(|c: char| c.is_digit(10)) && line.contains(". ") {
        (create_numbered_list_item(line), 1)
    } else if line.starts_with("> ") {
        (create_quote(line), 1)
    } else if line.starts_with("```") {
        create_code_block(lines)
    } else if line.starts_with("---") {
        (CreateBlock::Divider {}, 1)
    } else if line.starts_with("|") && lines.len() > 2 && lines[1].starts_with("|") {
        create_table(lines)
    } else if line.starts_with("!")
        && line.contains("[")
        && line.contains("]")
        && line.contains("(")
        && line.contains(")")
    {
        (create_image(line), 1)
    } else if line.starts_with("[") && line.contains("]:") {
        (create_bookmark(line), 1)
    } else if line.trim().starts_with("$") && line.trim().ends_with("$") {
        (create_equation(line), 1)
    } else {
        (create_paragraph(line), 1)
    }
}

fn create_rich_text(content: &str) -> RichText {
    RichText::Text {
        rich_text: RichTextCommon {
            plain_text: content.to_string(),
            href: None,
            annotations: None,
        },
        text: text::Text { content: content.to_string(), link: None },
    }
}

fn create_heading(line: &str, level: u8) -> CreateBlock {
    let content = line.trim_start_matches(|c| c == '#' || c == ' ');
    let rich_text = vec![create_rich_text(content)];
    match level {
        1 => CreateBlock::Heading1 { heading_1: Text { rich_text } },
        2 => CreateBlock::Heading2 { heading_2: Text { rich_text } },
        3 => CreateBlock::Heading3 { heading_3: Text { rich_text } },
        _ => unreachable!(),
    }
}

fn create_paragraph(line: &str) -> CreateBlock {
    CreateBlock::Paragraph {
        paragraph: TextAndChildren {
            rich_text: vec![create_rich_text(line)],
            children: None,
            color: TextColor::Default,
        },
    }
}

fn create_bulleted_list_item(line: &str) -> CreateBlock {
    CreateBlock::BulletedListItem {
        bulleted_list_item: TextAndChildren {
            rich_text: vec![create_rich_text(line.trim_start_matches("- "))],
            children: None,
            color: TextColor::Default,
        },
    }
}

fn create_numbered_list_item(line: &str) -> CreateBlock {
    CreateBlock::NumberedListItem {
        numbered_list_item: TextAndChildren {
            rich_text: vec![create_rich_text(
                line.trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ' '),
            )],
            children: None,
            color: TextColor::Default,
        },
    }
}

fn create_todo(line: &str) -> CreateBlock {
    let checked = line.starts_with("- [x] ");
    let content =
        line.trim_start_matches(|c| c == '-' || c == '[' || c == ']' || c == ' ' || c == 'x');
    CreateBlock::ToDo {
        to_do: ToDoFields {
            rich_text: vec![create_rich_text(content)],
            checked,
            color: TextColor::Default,
            children: None,
        },
    }
}

fn create_quote(line: &str) -> CreateBlock {
    CreateBlock::Quote {
        quote: TextAndChildren {
            rich_text: vec![create_rich_text(line.trim_start_matches("> "))],
            children: None,
            color: TextColor::Default,
        },
    }
}

fn create_code_block(lines: &[&str]) -> (CreateBlock, usize) {
    let mut code_lines = Vec::new();
    let mut i = 1;
    while i < lines.len() && !lines[i].starts_with("```") {
        code_lines.push(lines[i]);
        i += 1;
    }
    let language = lines[0].trim_start_matches("```").trim();
    (
        CreateBlock::Code {
            code: CodeFields {
                rich_text: code_lines.into_iter().map(create_rich_text).collect(),
                language: language.to_code_language(),
                caption: vec![],
            },
        },
        i + 1,
    )
}

fn create_image(line: &str) -> CreateBlock {
    create_bookmark(line)
    /*
    let re = Regex::new(r"!\[(.*?)\]\((.*?)\)").unwrap();
    if let Some(caps) = re.captures(line) {
        // let url = caps.get(2).map_or("", |m| m.as_str());
        /*CreateBlock::Image {
            image: FileObject::External {
                external: new notion::models::block::ExternalFileObject{url: url.to_string()},
            },
        }*/

        create_bookmark(line)
    } else {
        CreateBlock::Unsupported {}
    }*/
}

fn create_bookmark(line: &str) -> CreateBlock {
    let re = Regex::new(r"\[(.*?)\]:\s*(.*)").unwrap();
    if let Some(caps) = re.captures(line) {
        let url = caps.get(2).map_or("", |m| m.as_str());
        CreateBlock::Bookmark {
            bookmark: BookmarkFields { url: url.to_string(), caption: vec![] },
        }
    } else {
        CreateBlock::Unsupported {}
    }
}

fn create_equation(line: &str) -> CreateBlock {
    let equation = line.trim().trim_matches('$');
    CreateBlock::Equation { equation: Equation { expression: equation.to_string() } }
}

fn create_table(lines: &[&str]) -> (CreateBlock, usize) {
    let mut rows = Vec::new();
    let mut i = 0;
    while i < lines.len() && lines[i].starts_with("|") {
        let cells: Vec<String> = lines[i]
            .split('|')
            .filter(|&cell| !cell.trim().is_empty())
            .map(|cell| cell.trim().to_string())
            .collect();
        rows.push(cells);
        i += 1;
    }
    let has_column_header = true; // Assuming the first row is always a header in Markdown tables
    (
        CreateBlock::Table {
            table: TableFields {
                table_width: rows[0].len() as u64,
                has_column_header,
                has_row_header: false,
                children: vec![],
            },
        },
        i,
    )
}

pub trait ToCodeLanguage {
    fn to_code_language(&self) -> CodeLanguage;
}

impl ToCodeLanguage for str {
    fn to_code_language(&self) -> CodeLanguage {
        match self.to_lowercase().as_str() {
            "abap" => CodeLanguage::Abap,
            "arduino" => CodeLanguage::Arduino,
            "bash" => CodeLanguage::Bash,
            "basic" => CodeLanguage::Basic,
            "c" => CodeLanguage::C,
            "clojure" => CodeLanguage::Clojure,
            "coffeescript" => CodeLanguage::Coffeescript,
            "c++" => CodeLanguage::CPlusPlus,
            "csharp" => CodeLanguage::CSharp,
            "css" => CodeLanguage::Css,
            "dart" => CodeLanguage::Dart,
            "diff" => CodeLanguage::Diff,
            "docker" => CodeLanguage::Docker,
            "elixir" => CodeLanguage::Elixir,
            "elm" => CodeLanguage::Elm,
            "erlang" => CodeLanguage::Erlang,
            "flow" => CodeLanguage::Flow,
            "fortran" => CodeLanguage::Fortran,
            "fsharp" => CodeLanguage::FSharp,
            "gherkin" => CodeLanguage::Gherkin,
            "glsl" => CodeLanguage::Glsl,
            "go" => CodeLanguage::Go,
            "graphql" => CodeLanguage::Graphql,
            "groovy" => CodeLanguage::Groovy,
            "haskell" => CodeLanguage::Haskell,
            "html" => CodeLanguage::Html,
            "java" => CodeLanguage::Java,
            "javascript" => CodeLanguage::Javascript,
            "json" => CodeLanguage::Json,
            "julia" => CodeLanguage::Julia,
            "kotlin" => CodeLanguage::Kotlin,
            "latex" => CodeLanguage::Latex,
            "less" => CodeLanguage::Less,
            "lisp" => CodeLanguage::Lisp,
            "livescript" => CodeLanguage::Livescript,
            "lua" => CodeLanguage::Lua,
            "makefile" => CodeLanguage::Makefile,
            "markdown" => CodeLanguage::Markdown,
            "markup" => CodeLanguage::Markup,
            "matlab" => CodeLanguage::Matlab,
            "mermaid" => CodeLanguage::Mermaid,
            "nix" => CodeLanguage::Nix,
            "objective-c" => CodeLanguage::ObjectiveC,
            "ocaml" => CodeLanguage::Ocaml,
            "pascal" => CodeLanguage::Pascal,
            "perl" => CodeLanguage::Perl,
            "php" => CodeLanguage::Php,
            "plain text" => CodeLanguage::PlainText,
            "powershell" => CodeLanguage::Powershell,
            "prolog" => CodeLanguage::Prolog,
            "protobuf" => CodeLanguage::Protobuf,
            "python" => CodeLanguage::Python,
            "r" => CodeLanguage::R,
            "reason" => CodeLanguage::Reason,
            "ruby" => CodeLanguage::Ruby,
            "rust" => CodeLanguage::Rust,
            "sass" => CodeLanguage::Sass,
            "scala" => CodeLanguage::Scala,
            "scheme" => CodeLanguage::Scheme,
            "scss" => CodeLanguage::Scss,
            "shell" => CodeLanguage::Shell,
            "sql" => CodeLanguage::Sql,
            "swift" => CodeLanguage::Swift,
            "typescript" => CodeLanguage::Typescript,
            "vbnet" => CodeLanguage::VbNet,
            "verilog" => CodeLanguage::Verilog,
            "vhdl" => CodeLanguage::Vhdl,
            "visual basic" => CodeLanguage::VisualBasic,
            "webassembly" => CodeLanguage::Webassembly,
            "xml" => CodeLanguage::Xml,
            "yaml" => CodeLanguage::Yaml,
            _ => CodeLanguage::PlainText, // Default to plain text if language is not recognized
        }
    }
}
