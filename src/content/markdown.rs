use std::collections::HashMap;

use comrak::{
    Options, Plugins, markdown_to_html_with_plugins, plugins::syntect::SyntectAdapterBuilder,
};
use regex::Regex;

use crate::{content::Heading, error::AppResult};

pub fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in input.chars() {
        let normalized = match ch {
            'A'..='Z' => ch.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' => ch,
            '\u{4e00}'..='\u{9fff}' => ch,
            _ => '-',
        };
        if normalized == '-' {
            if !last_dash && !slug.is_empty() {
                slug.push('-');
            }
            last_dash = true;
        } else {
            slug.push(normalized);
            last_dash = false;
        }
    }
    slug.trim_matches('-').to_string()
}

pub fn extract_headings(markdown: &str) -> Vec<Heading> {
    let heading_re = Regex::new(r"^(#{1,6})\s+(.*)$").expect("valid regex");
    let mut seen = HashMap::<String, usize>::new();
    let mut headings = Vec::new();
    for line in markdown.lines() {
        if let Some(caps) = heading_re.captures(line) {
            let level = caps.get(1).map(|m| m.as_str().len()).unwrap_or(1) as u8;
            let title = caps.get(2).map(|m| m.as_str().trim()).unwrap_or_default();
            let base = slugify(title);
            let count = seen.entry(base.clone()).or_insert(0);
            *count += 1;
            let suffix = if *count > 1 {
                format!("-{}", *count)
            } else {
                String::new()
            };
            headings.push(Heading {
                level,
                id: format!("section-{}{}", base, suffix),
                title: title.to_string(),
            });
        }
    }
    headings
}

pub fn render_markdown(markdown: &str) -> AppResult<(String, Vec<Heading>)> {
    let headings = extract_headings(markdown);
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.extension.math_dollars = true;
    options.extension.math_code = true;
    options.extension.header_ids = Some("section-".to_string());

    let adapter = SyntectAdapterBuilder::new()
        .theme("base16-ocean.dark")
        .build();
    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let html = markdown_to_html_with_plugins(markdown, &options, &plugins);
    Ok((html, headings))
}

pub fn word_count(markdown: &str) -> usize {
    markdown.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_basic_text() {
        assert_eq!(slugify("Hello Rust World"), "hello-rust-world");
        assert_eq!(slugify("中文 标题"), "中文-标题");
    }

    #[test]
    fn renders_math_nodes_for_frontend_typesetting() {
        let (html, _) = render_markdown("$$x^2$$").unwrap();
        assert!(html.contains("data-math-style=\"display\""));
        assert!(html.contains("x^2"));
    }

    #[test]
    fn renders_math_code_blocks() {
        let (html, _) = render_markdown("```math\nx^2 + y^2 = z^2\n```").unwrap();
        assert!(html.contains("data-math-style=\"display\""));
        assert!(html.contains("language-math"));
    }

    #[test]
    fn highlights_code_fences_with_language_markup() {
        let (html, _) = render_markdown(
            r"```rust
fn main() {}
```",
        )
        .unwrap();
        assert!(html.contains("language-rust"));
    }
}
