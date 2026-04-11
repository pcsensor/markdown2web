use std::collections::HashMap;

use comrak::{
    Options, Plugins, markdown_to_html_with_plugins, plugins::syntect::SyntectAdapterBuilder,
};
use regex::Regex;

use crate::{content::Heading, error::AppResult};

#[derive(Debug)]
struct AudioEmbed {
    token: String,
    label: String,
    src: String,
    mime_type: &'static str,
}

#[derive(Debug)]
struct VideoEmbed {
    token: String,
    label: String,
    src: String,
    mime_type: &'static str,
}

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
    let (markdown, audio_embeds) = replace_audio_blocks_with_tokens(markdown);
    let (markdown, video_embeds) = replace_video_blocks_with_tokens(&markdown);

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

    let html = markdown_to_html_with_plugins(&markdown, &options, &plugins);
    let html = restore_audio_blocks(&html, &audio_embeds);
    let html = restore_video_blocks(&html, &video_embeds);
    let headings = extract_headings_from_html(&html);
    Ok((html, headings))
}

fn replace_audio_blocks_with_tokens(markdown: &str) -> (String, Vec<AudioEmbed>) {
    let re = Regex::new(r"#\[([^\]]*)\]\(([^)]+\.(?:mp3|wav))\)").expect("valid audio regex");
    let mut audio_embeds = Vec::new();
    let markdown = re
        .replace_all(markdown, |caps: &regex::Captures| {
            let index = audio_embeds.len();
            let token = format!("M2W_AUDIO_EMBED_{}", index);
            let src = caps[2].to_string();
            audio_embeds.push(AudioEmbed {
                token: token.clone(),
                label: caps[1].to_string(),
                mime_type: if src.ends_with(".wav") {
                    "audio/wav"
                } else {
                    "audio/mpeg"
                },
                src,
            });
            token
        })
        .to_string();
    (markdown, audio_embeds)
}

fn replace_video_blocks_with_tokens(markdown: &str) -> (String, Vec<VideoEmbed>) {
    let re = Regex::new(r"@\[([^\]]*)\]\(([^)]+\.(?:mp4|webm|ogv|ogg|mov|m4v))\)")
        .expect("valid video regex");
    let mut video_embeds = Vec::new();
    let markdown = re
        .replace_all(markdown, |caps: &regex::Captures| {
            let index = video_embeds.len();
            let token = format!("M2W_VIDEO_EMBED_{}", index);
            let src = caps[2].to_string();
            video_embeds.push(VideoEmbed {
                token: token.clone(),
                label: caps[1].to_string(),
                mime_type: video_mime_type(&src),
                src,
            });
            token
        })
        .to_string();
    (markdown, video_embeds)
}

fn video_mime_type(src: &str) -> &'static str {
    if src.ends_with(".webm") {
        "video/webm"
    } else if src.ends_with(".ogv") || src.ends_with(".ogg") {
        "video/ogg"
    } else if src.ends_with(".mov") {
        "video/quicktime"
    } else {
        "video/mp4"
    }
}

fn restore_audio_blocks(html: &str, audio_embeds: &[AudioEmbed]) -> String {
    let mut output = html.to_string();
    for embed in audio_embeds {
        let player = format!(
            r#"<div class="audio-player" data-audio-player>
                <div class="audio-player-inner">
                    <button type="button" class="audio-play-btn" data-audio-play-btn aria-label="播放">
                        <svg viewBox="0 0 20 20" fill="none" class="audio-icon-play" aria-hidden="true">
                            <path d="M7 5.5v9l7-4.5-7-4.5Z" />
                        </svg>
                        <svg viewBox="0 0 20 20" fill="none" class="audio-icon-pause" aria-hidden="true">
                            <rect x="6" y="5.5" width="2.6" height="9" rx="0.8" />
                            <rect x="11.4" y="5.5" width="2.6" height="9" rx="0.8" />
                        </svg>
                    </button>
                    <div class="audio-info">
                        <span class="audio-label">{}</span>
                        <span class="audio-time" data-audio-time>00:00/00:00</span>
                    </div>
                    <div class="audio-progress-wrap" data-audio-progress-wrap>
                        <div class="audio-progress-bar" data-audio-progress-bar></div>
                    </div>
                    <audio preload="metadata" data-audio>
                        <source src="{}" type="{}">
                    </audio>
                </div>
            </div>"#,
            embed.label, embed.src, embed.mime_type
        );
        output = output.replace(&format!("<p>{}</p>\n", embed.token), &player);
        output = output.replace(&format!("<p>{}</p>", embed.token), &player);
        output = output.replace(&embed.token, &player);
    }
    output
}

fn restore_video_blocks(html: &str, video_embeds: &[VideoEmbed]) -> String {
    let mut output = html.to_string();
    for embed in video_embeds {
        let player = format!(
            r#"<figure class="video-player" data-video-player>
                <div class="video-player-frame">
                    <video class="video-player-media" controls preload="none" playsinline data-video-src="{}" data-video-type="{}">
                        <source data-src="{}" type="{}">
                        无法播放视频：{}
                    </video>
                    <button type="button" class="video-load-button" data-video-load data-static-button>播放视频</button>
                </div>
            </figure>"#,
            embed.src, embed.mime_type, embed.src, embed.mime_type, embed.label
        );
        output = output.replace(&format!("<p>{}</p>\n", embed.token), &player);
        output = output.replace(&format!("<p>{}</p>", embed.token), &player);
        output = output.replace(&embed.token, &player);
    }
    output
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
    fn renders_audio_embed_blocks_after_markdown_render() {
        let (html, _) = render_markdown("#[红尘客栈](/assets/hckz.mp3)").unwrap();
        assert!(html.contains("data-audio-player"));
        assert!(html.contains("audio-label\">红尘客栈"));
        assert!(html.contains("source src=\"/assets/hckz.mp3\" type=\"audio/mpeg\""));
        assert!(!html.contains("M2W_AUDIO_EMBED_0"));
    }

    #[test]
    fn renders_video_embed_blocks_after_markdown_render() {
        let (html, _) = render_markdown("@[Clion开发STM32](/assets/Clion-STM32.mp4)").unwrap();
        assert!(html.contains("data-video-player"));
        assert!(html.contains("video-player-frame"));
        assert!(html.contains("controls preload=\"none\" playsinline"));
        assert!(html.contains("data-video-load"));
        assert!(html.contains("无法播放视频：Clion开发STM32"));
        assert!(html.contains("data-video-src=\"/assets/Clion-STM32.mp4\""));
        assert!(html.contains("source data-src=\"/assets/Clion-STM32.mp4\" type=\"video/mp4\""));
        assert!(!html.contains("video-label"));
        assert!(!html.contains("M2W_VIDEO_EMBED_0"));
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
