//! Markdown to HTML, with syntax highlighting and LaTeX math rendered to MathML at build time.
//! Footnotes become Tufte-style sidenotes on the site and stay ordinary footnotes in the feed.

use std::collections::HashMap;
use std::str::FromStr;

use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use pulldown_latex::config::DisplayMode;
use serde::Deserialize;
use syntect::highlighting::{Color, FontStyle, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSettings};
use syntect::html::{ClassStyle, ClassedHTMLGenerator, css_for_theme_with_class_style};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

const CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed { prefix: "hl-" };
const LIGHT_THEME: &str = "themes/github-light.json";
const DARK_THEME: &str = "themes/github-dark.json";

pub struct Rendered {
    pub html: String,
    /// Same content with plain footnotes, for feed readers.
    pub feed_html: String,
    pub has_math: bool,
    pub has_code: bool,
}

pub struct Markdown {
    syntaxes: SyntaxSet,
}

impl Markdown {
    pub fn new() -> Self {
        Self {
            syntaxes: SyntaxSet::load_defaults_newlines(),
        }
    }

    pub fn render(&self, source: &str) -> Result<Rendered, String> {
        let options = Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_SMART_PUNCTUATION
            | Options::ENABLE_MATH;
        let mut events: Vec<Event> = Vec::new();
        let mut has_math = false;
        let mut has_code = false;
        let mut code: Option<(String, String)> = None; // (language, source) of the open code block

        for event in Parser::new_ext(source, options) {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    let lang = match kind {
                        CodeBlockKind::Fenced(info) => info.split_whitespace().next().unwrap_or("").to_owned(),
                        CodeBlockKind::Indented => String::new(),
                    };
                    code = Some((lang, String::new()));
                }
                Event::Text(text) if code.is_some() => code.as_mut().unwrap().1.push_str(&text),
                Event::End(TagEnd::CodeBlock) => {
                    let (lang, src) = code.take().unwrap();
                    events.push(Event::Html(self.highlight(&lang, &src)?.into()));
                    has_code = true;
                }
                Event::InlineMath(tex) => {
                    events.push(Event::InlineHtml(math(&tex, DisplayMode::Inline)?.into()));
                    has_math = true;
                }
                Event::DisplayMath(tex) => {
                    let mathml = math(&tex, DisplayMode::Block)?;
                    // Display math sits inside a paragraph, so wrap it in a span styled as a block.
                    events.push(Event::InlineHtml(
                        format!("<span class=\"math-block\">{mathml}</span>").into(),
                    ));
                    has_math = true;
                }
                event => events.push(event),
            }
        }

        add_heading_ids(&mut events);
        let mut feed_html = String::new();
        pulldown_cmark::html::push_html(&mut feed_html, events.iter().cloned());
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, sidenotes(events)?.into_iter());
        Ok(Rendered {
            html,
            feed_html,
            has_math,
            has_code,
        })
    }

    fn highlight(&self, lang: &str, src: &str) -> Result<String, String> {
        let syntax = self
            .syntaxes
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| self.syntaxes.find_syntax_plain_text());
        let mut generator = ClassedHTMLGenerator::new_with_class_style(syntax, &self.syntaxes, CLASS_STYLE);
        for line in LinesWithEndings::from(src) {
            generator
                .parse_html_for_line_which_includes_newline(line)
                .map_err(|e| format!("highlighting {lang}: {e}"))?;
        }
        let lang: String = lang
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '+')
            .collect();
        Ok(format!(
            "<pre class=\"code\" data-lang=\"{lang}\"><code>{}</code></pre>\n",
            generator.finalize()
        ))
    }
}

/// Stylesheet for highlighted code: the light theme by default, the dark one under `.dark`.
pub fn syntax_css() -> Result<String, String> {
    let css = |path: &str| {
        css_for_theme_with_class_style(&load_theme(path)?, CLASS_STYLE).map_err(|e| format!("{path}: {e}"))
    };
    Ok(format!("{}\n.dark {{\n{}\n}}\n", css(LIGHT_THEME)?, css(DARK_THEME)?))
}

/// The parts of a VS Code color theme that matter for highlighting. Its `tokenColors` rules have
/// the same shape as TextMate theme rules, so they convert directly into a syntect `Theme`.
#[derive(Deserialize)]
struct VsCodeTheme {
    name: String,
    colors: HashMap<String, String>,
    #[serde(rename = "tokenColors")]
    token_colors: Vec<TokenColor>,
}

#[derive(Deserialize)]
struct TokenColor {
    scope: Option<Scopes>,
    settings: TokenSettings,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Scopes {
    One(String),
    Many(Vec<String>),
}

#[derive(Deserialize)]
struct TokenSettings {
    foreground: Option<String>,
    background: Option<String>,
    #[serde(rename = "fontStyle")]
    font_style: Option<String>,
}

fn load_theme(path: &str) -> Result<Theme, String> {
    let source = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let theme: VsCodeTheme = serde_json::from_str(&source).map_err(|e| format!("{path}: {e}"))?;
    let color = |key: &str| theme.colors.get(key).map(|c| parse_color(c)).transpose();
    let mut scopes = Vec::new();
    for rule in &theme.token_colors {
        let selector = match &rule.scope {
            Some(Scopes::One(s)) => s.clone(),
            Some(Scopes::Many(list)) => list.join(", "),
            None => continue, // a rule without a scope only styles the editor itself
        };
        let style = &rule.settings;
        scopes.push(ThemeItem {
            scope: ScopeSelectors::from_str(&selector).map_err(|e| format!("{path}: scope `{selector}`: {e}"))?,
            style: StyleModifier {
                foreground: style.foreground.as_deref().map(parse_color).transpose()?,
                background: style.background.as_deref().map(parse_color).transpose()?,
                font_style: style.font_style.as_deref().map(parse_font_style),
            },
        });
    }
    Ok(Theme {
        name: Some(theme.name.clone()),
        author: None,
        settings: ThemeSettings {
            foreground: color("editor.foreground")?,
            background: color("editor.background")?,
            ..ThemeSettings::default()
        },
        scopes,
    })
}

/// `#rgb`, `#rrggbb`, or `#rrggbbaa`.
fn parse_color(hex: &str) -> Result<Color, String> {
    let digits = hex.trim_start_matches('#');
    let full: String = match digits.len() {
        3 => digits.chars().flat_map(|c| [c, c]).collect(),
        6 | 8 => digits.to_owned(),
        _ => return Err(format!("unsupported color `{hex}`")),
    };
    let byte = |i: usize| u8::from_str_radix(&full[i..i + 2], 16).map_err(|_| format!("bad color `{hex}`"));
    Ok(Color {
        r: byte(0)?,
        g: byte(2)?,
        b: byte(4)?,
        a: if full.len() == 8 { byte(6)? } else { 0xff },
    })
}

fn parse_font_style(style: &str) -> FontStyle {
    style
        .split_whitespace()
        .fold(FontStyle::empty(), |acc, word| match word {
            "bold" => acc | FontStyle::BOLD,
            "italic" => acc | FontStyle::ITALIC,
            "underline" => acc | FontStyle::UNDERLINE,
            _ => acc,
        })
}

fn math(tex: &str, display_mode: DisplayMode) -> Result<String, String> {
    let storage = pulldown_latex::Storage::new();
    let parser = pulldown_latex::Parser::new(tex, &storage);
    let config = pulldown_latex::RenderConfig {
        display_mode,
        annotation: Some(tex),
        ..Default::default()
    };
    let mut out = String::new();
    pulldown_latex::push_mathml(&mut out, parser, config).map_err(|e| format!("math `{tex}`: {e}"))?;
    Ok(out)
}

/// Moves each footnote definition to where it's referenced, as a sidenote.
///
/// Markup follows Tufte CSS: wide screens show the note in the right margin; on narrow screens
/// the number is a label for a hidden checkbox that expands the note inline, so no JavaScript.
fn sidenotes(events: Vec<Event>) -> Result<Vec<Event>, String> {
    let mut definitions: HashMap<String, Vec<Event>> = HashMap::new();
    let mut body = Vec::new();
    let mut open: Option<(String, Vec<Event>)> = None;
    for event in events {
        match event {
            Event::Start(Tag::FootnoteDefinition(label)) => open = Some((label.to_string(), Vec::new())),
            Event::End(TagEnd::FootnoteDefinition) => {
                let (label, content) = open.take().unwrap();
                definitions.insert(label, content);
            }
            event => match &mut open {
                Some((_, content)) => content.push(event),
                None => body.push(event),
            },
        }
    }

    let mut numbers: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::with_capacity(body.len());
    let mut body = body.into_iter().peekable();
    while let Some(event) = body.next() {
        let Event::FootnoteReference(label) = event else {
            out.push(event);
            continue;
        };
        if let Some(n) = numbers.get(label.as_ref()) {
            // A repeated reference points back at the note instead of duplicating it.
            out.push(Event::InlineHtml(
                format!("<sup class=\"sidenote-ref\">{n}</sup>").into(),
            ));
            continue;
        }
        let n = numbers.len() + 1;
        numbers.insert(label.to_string(), n);
        let content = definitions
            .remove(label.as_ref())
            .ok_or_else(|| format!("footnote [^{label}] has no definition"))?;
        // The note sits inside a paragraph, so its own paragraphs become line breaks.
        let mut note = String::new();
        let mut first = true;
        let inline = content.into_iter().filter_map(|e| match e {
            Event::Start(Tag::Paragraph) if first => {
                first = false;
                None
            }
            Event::Start(Tag::Paragraph) => Some(Event::InlineHtml("<br>".into())),
            Event::End(TagEnd::Paragraph) => None,
            e => Some(e),
        });
        pulldown_cmark::html::push_html(&mut note, inline);
        out.push(Event::InlineHtml(
            format!(
                "<input type=\"checkbox\" id=\"sn-{n}\" class=\"sidenote-toggle\" aria-label=\"Show note {n}\">\
                 <label for=\"sn-{n}\" class=\"sidenote-ref\">{n}</label>"
            )
            .into(),
        ));
        // Keep punctuation right after the reference ahead of the note, so an expanded note on a
        // narrow screen doesn't strand a period at the start of the next line.
        let mut rest = None;
        if let Some(Event::Text(text)) = body.peek() {
            let split = text
                .find(|c: char| !".,;:!?)\u{201d}\u{2019}".contains(c))
                .unwrap_or(text.len());
            if split > 0 {
                out.push(Event::Text(text[..split].to_owned().into()));
                rest = Some(text[split..].to_owned());
                body.next();
            }
        }
        out.push(Event::InlineHtml(
            format!(
                "<span class=\"sidenote\"><span class=\"sidenote-num\">{n}</span>{}</span>",
                note.trim()
            )
            .into(),
        ));
        if let Some(rest) = rest.filter(|r| !r.is_empty()) {
            out.push(Event::Text(rest.into()));
        }
    }
    Ok(out)
}

/// Give h2/h3 headings an `id` so sections can be linked to.
fn add_heading_ids(events: &mut [Event]) {
    let mut i = 0;
    while i < events.len() {
        if let Event::Start(Tag::Heading {
            level: HeadingLevel::H2 | HeadingLevel::H3,
            id: None,
            ..
        }) = &events[i]
        {
            let mut text = String::new();
            for event in &events[i + 1..] {
                match event {
                    Event::End(TagEnd::Heading(_)) => break,
                    Event::Text(t) | Event::Code(t) => text.push_str(t),
                    _ => {}
                }
            }
            if let Event::Start(Tag::Heading { id, .. }) = &mut events[i] {
                *id = Some(CowStr::from(slugify(&text)));
            }
        }
        i += 1;
    }
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    for c in text.to_lowercase().chars() {
        if c.is_alphanumeric() {
            slug.push(c);
        } else if !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
    }
    slug.trim_end_matches('-').to_owned()
}
