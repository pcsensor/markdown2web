use std::collections::HashMap;

use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapterBuilder, Options, Plugins,
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
    let headings = extract_headings_from_html(&html);
    Ok((html, headings))
}

fn extract_headings_from_html(html: &str) -> Vec<Heading> {
    let re = Regex::new(r#"<h([1-6])>.+?id="([^"]+)".+?</h[1-6]>"#).expect("valid heading regex");
    re.captures_iter(html)
        .map(|caps| {
            let level = caps
                .get(1)
                .map(|m| m.as_str().parse().unwrap_or(1))
                .unwrap_or(1);
            let id = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let title = strip_html_tags(caps.get(0).map(|m| m.as_str()).unwrap_or_default());
            Heading { level, id, title }
        })
        .collect()
}

fn strip_html_tags(input: &str) -> String {
    Regex::new(r"<[^>]*>")
        .expect("valid tag regex")
        .replace_all(input, "")
        .to_string()
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

    #[test]
    fn toc_headings_match_rendered_html_ids() {
        let markdown = r#"# First Heading

Some content.

## Second Heading

More content.

### Second Heading

Duplicate heading test.
"#;
        let (html, headings) = render_markdown(markdown).unwrap();

        for heading in &headings {
            assert!(
                html.contains(&format!("id=\"{}\"", heading.id)),
                "TOC heading id '{}' not found in rendered HTML",
                heading.id
            );
        }
    }
}
