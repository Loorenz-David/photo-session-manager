//! The page reader (R8·U3): a plain HTTP download from the Mac, for every site (evidence doc
//! "Owner picks" 2), and what a downloaded page offers without running scripts: its title, its
//! visible text and its main images, by the routes R1 found (`og:title` / `og:image`, JSON-LD
//! `Product`, then the largest content images). HTML is read with html5ever's tokenizer (0.39);
//! no script is ever run.

pub mod http;

use std::cell::RefCell;

use html5ever::{
    buffer_queue::BufferQueue,
    tendril::StrTendril,
    tokenizer::{
        Tag, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts, states::RawKind,
    },
};
use serde_json::Value as Json;

pub use http::HttpPageReader;

/// The visible text is cut to this many characters (it goes into a model request).
pub const MAX_TEXT_CHARS: usize = 30_000;

/// What a page offers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HtmlPage {
    pub title: String,
    pub text: String,
    /// Candidate image URLs as written in the page, best first (not yet resolved).
    pub images: Vec<String>,
}

#[derive(Default)]
struct Collected {
    title: String,
    in_title: bool,
    og_title: Option<String>,
    og_image: Option<String>,
    json_ld: Vec<String>,
    in_json_ld: bool,
    /// Depth inside elements whose text is not visible (script, style, head, …).
    hidden: u32,
    text: String,
    /// `(declared width, src)` of each `<img>`.
    imgs: Vec<(u32, String)>,
}

struct Sink(RefCell<Collected>);

fn attr(tag: &Tag, name: &str) -> Option<String> {
    tag.attrs
        .iter()
        .find(|a| &*a.name.local == name)
        .map(|a| a.value.to_string())
}

/// The largest width an `<img>` declares: its `width` attribute or a `srcset` `w` descriptor.
fn declared_width(tag: &Tag) -> u32 {
    let width = attr(tag, "width")
        .and_then(|w| w.trim().parse::<u32>().ok())
        .unwrap_or(0);
    let srcset = attr(tag, "srcset")
        .map(|set| {
            set.split(',')
                .filter_map(|entry| {
                    entry
                        .split_whitespace()
                        .nth(1)?
                        .strip_suffix('w')?
                        .parse::<u32>()
                        .ok()
                })
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    width.max(srcset)
}

const HIDDEN: [&str; 8] = [
    "script", "style", "noscript", "template", "head", "svg", "iframe", "select",
];

impl TokenSink for Sink {
    type Handle = ();

    fn process_token(&self, token: Token, _line: u64) -> TokenSinkResult<()> {
        let mut c = self.0.borrow_mut();
        match token {
            Token::TagToken(tag) => {
                let name: &str = &tag.name;
                match tag.kind {
                    TagKind::StartTag => {
                        if name == "meta" {
                            let key = attr(&tag, "property").or_else(|| attr(&tag, "name"));
                            let content = attr(&tag, "content");
                            match (key.as_deref(), content) {
                                (Some("og:title"), Some(v)) if c.og_title.is_none() => {
                                    c.og_title = Some(v)
                                }
                                (Some("og:image"), Some(v)) if c.og_image.is_none() => {
                                    c.og_image = Some(v)
                                }
                                _ => {}
                            }
                        }
                        if name == "img"
                            && let Some(src) = attr(&tag, "src")
                        {
                            let width = declared_width(&tag);
                            c.imgs.push((width, src));
                        }
                        if name == "br" || name == "p" || name == "li" || name == "div" {
                            c.text.push(' ');
                        }
                        if name == "body" {
                            // A page whose `</head>` is missing still shows its body.
                            c.hidden = 0;
                        }
                        if tag.self_closing {
                            return TokenSinkResult::Continue;
                        }
                        if HIDDEN.contains(&name) {
                            c.hidden += 1;
                        }
                        match name {
                            "title" => {
                                c.in_title = true;
                                TokenSinkResult::RawData(RawKind::Rcdata)
                            }
                            "script" => {
                                c.in_json_ld = attr(&tag, "type")
                                    .is_some_and(|t| t.eq_ignore_ascii_case("application/ld+json"));
                                if c.in_json_ld {
                                    c.json_ld.push(String::new());
                                }
                                TokenSinkResult::RawData(RawKind::ScriptData)
                            }
                            "style" | "noscript" | "iframe" | "xmp" | "noembed" | "noframes" => {
                                TokenSinkResult::RawData(RawKind::Rawtext)
                            }
                            "textarea" => TokenSinkResult::RawData(RawKind::Rcdata),
                            "plaintext" => TokenSinkResult::Plaintext,
                            _ => TokenSinkResult::Continue,
                        }
                    }
                    TagKind::EndTag => {
                        if HIDDEN.contains(&name) {
                            c.hidden = c.hidden.saturating_sub(1);
                        }
                        match name {
                            "title" => c.in_title = false,
                            "script" => c.in_json_ld = false,
                            _ => c.text.push(' '),
                        }
                        TokenSinkResult::Continue
                    }
                }
            }
            Token::CharacterTokens(text) => {
                if c.in_title {
                    c.title.push_str(&text);
                } else if c.in_json_ld {
                    if let Some(last) = c.json_ld.last_mut() {
                        last.push_str(&text);
                    }
                } else if c.hidden == 0 {
                    c.text.push_str(&text);
                }
                TokenSinkResult::Continue
            }
            _ => TokenSinkResult::Continue,
        }
    }
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every object of a JSON-LD block (top level, arrays and `@graph`).
fn ld_objects(value: &Json, out: &mut Vec<Json>) {
    match value {
        Json::Array(items) => items.iter().for_each(|i| ld_objects(i, out)),
        Json::Object(map) => {
            out.push(value.clone());
            if let Some(graph) = map.get("@graph") {
                ld_objects(graph, out);
            }
        }
        _ => {}
    }
}

fn is_product(object: &Json) -> bool {
    match &object["@type"] {
        Json::String(t) => t == "Product",
        Json::Array(types) => types.iter().any(|t| t == "Product"),
        _ => false,
    }
}

fn ld_images(value: &Json, out: &mut Vec<String>) {
    match value {
        Json::String(url) => out.push(url.clone()),
        Json::Array(items) => items.iter().for_each(|i| ld_images(i, out)),
        Json::Object(map) => {
            if let Some(url) = map.get("url").or_else(|| map.get("contentUrl")) {
                ld_images(url, out);
            }
        }
        _ => {}
    }
}

/// The title, visible text and candidate images of an HTML document.
pub fn read_html(html: &str) -> HtmlPage {
    let sink = Sink(RefCell::new(Collected::default()));
    let tokenizer = Tokenizer::new(sink, TokenizerOpts::default());
    let input = BufferQueue::default();
    input.push_back(StrTendril::from_slice(html));
    let _ = tokenizer.feed(&input);
    tokenizer.end();
    let c = tokenizer.sink.0.into_inner();
    let mut product: Option<Json> = None;
    for block in &c.json_ld {
        let Ok(value) = serde_json::from_str::<Json>(block.trim()) else {
            continue;
        };
        let mut objects = Vec::new();
        ld_objects(&value, &mut objects);
        if let Some(found) = objects.into_iter().find(is_product) {
            product = Some(found);
            break;
        }
    }
    let title = c
        .og_title
        .map(|t| collapse(&t))
        .filter(|t| !t.is_empty())
        .or_else(|| {
            product
                .as_ref()
                .and_then(|p| p["name"].as_str())
                .map(collapse)
                .filter(|t| !t.is_empty())
        })
        .unwrap_or_else(|| collapse(&c.title));
    let mut images: Vec<String> = Vec::new();
    images.extend(c.og_image);
    if let Some(product) = &product {
        ld_images(&product["image"], &mut images);
    }
    let mut imgs = c.imgs;
    // Largest declared width first; equal widths keep document order.
    imgs.sort_by_key(|img| std::cmp::Reverse(img.0));
    images.extend(imgs.into_iter().map(|(_, src)| src));
    let text: String = collapse(&c.text).chars().take(MAX_TEXT_CHARS).collect();
    HtmlPage {
        title,
        text,
        images,
    }
}
