use scraper::{Html, Selector, node::Node};
use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use swimming_core::{ROOT, SwimmingEngine};
use url::Url;

type LoadResult<T> = Result<T, Box<dyn Error>>;

pub fn run_source(bridge: &crate::JsBridge, source: &str) -> LoadResult<()> {
    if let Ok(url) = Url::parse(source) {
        if matches!(url.scheme(), "http" | "https") {
            let content = fetch(url.as_str())?;
            if is_html(&url, &content) {
                mount_html(&bridge.engine(), &content);
                for script in html_scripts(&content, Some(&url), None)? {
                    bridge.eval_script(&script).map_err(std::io::Error::other)?;
                }
            } else {
                bridge
                    .eval_script(&content)
                    .map_err(std::io::Error::other)?;
            }
            return Ok(());
        }
    }

    let path = Path::new(source);
    let content = std::fs::read_to_string(path)?;
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm"))
    {
        mount_html(&bridge.engine(), &content);
        for script in html_scripts(&content, None, path.parent())? {
            bridge.eval_script(&script).map_err(std::io::Error::other)?;
        }
    } else {
        bridge
            .eval_script(&content)
            .map_err(std::io::Error::other)?;
    }
    Ok(())
}

fn fetch(url: &str) -> LoadResult<String> {
    Ok(ureq::get(url).call()?.body_mut().read_to_string()?)
}

fn is_html(url: &Url, content: &str) -> bool {
    url.path().ends_with(".html")
        || url.path().ends_with(".htm")
        || content
            .trim_start()
            .to_ascii_lowercase()
            .starts_with("<!doctype html")
        || content
            .trim_start()
            .to_ascii_lowercase()
            .starts_with("<html")
}

fn html_scripts(
    html: &str,
    base_url: Option<&Url>,
    base_path: Option<&Path>,
) -> LoadResult<Vec<String>> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("script").expect("static selector is valid");
    let mut scripts = Vec::new();
    for element in document.select(&selector) {
        let kind = element
            .value()
            .attr("type")
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        if kind == "module" {
            continue;
        }
        if !kind.is_empty()
            && !matches!(
                kind.as_str(),
                "text/javascript"
                    | "application/javascript"
                    | "text/ecmascript"
                    | "application/ecmascript"
            )
        {
            continue;
        }
        if let Some(src) = element.value().attr("src") {
            let script = if let Some(base) = base_url {
                fetch(base.join(src)?.as_str())?
            } else {
                let path = PathBuf::from(src);
                let path = if path.is_absolute() {
                    base_path
                        .unwrap_or_else(|| Path::new("."))
                        .join(path.strip_prefix("/").unwrap_or(&path))
                } else {
                    base_path.unwrap_or_else(|| Path::new(".")).join(path)
                };
                std::fs::read_to_string(path)?
            };
            scripts.push(script);
        } else {
            let script = element.text().collect::<String>();
            if !script.trim().is_empty() {
                scripts.push(script);
            }
        }
    }
    Ok(scripts)
}

fn mount_html(engine: &Arc<Mutex<SwimmingEngine>>, html: &str) {
    let document = Html::parse_document(html);
    let selector = Selector::parse("body").expect("static selector is valid");
    let mut engine = engine.lock().unwrap();
    if let Some(body) = document.select(&selector).next() {
        append_html_node(&mut engine, body, ROOT);
    }
    let (width, height) = (engine.width, engine.height);
    engine.compute_layout(width, height);
}

fn append_html_node(engine: &mut SwimmingEngine, node: scraper::ElementRef<'_>, parent: u64) {
    for child in node.children() {
        match child.value() {
            Node::Element(element) if element.name() != "script" && element.name() != "style" => {
                let style = element.attr("style").unwrap_or("");
                let mut props = serde_json::Map::new();
                for (key, value) in element.attrs() {
                    props.insert(key.to_owned(), serde_json::Value::String(value.to_owned()));
                }
                let id = engine.create_node(
                    element.name(),
                    style,
                    &serde_json::Value::Object(props).to_string(),
                );
                engine.append_child(parent, id);
                if let Some(child_element) = scraper::ElementRef::wrap(child) {
                    append_html_node(engine, child_element, id);
                }
            }
            Node::Text(text) if !text.trim().is_empty() => {
                let id = engine.create_text(text);
                engine.append_child(parent, id);
            }
            _ => {}
        }
    }
}
