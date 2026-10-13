//! User-agent defaults for every HTML element.

pub const ALL_ELEMENTS: &[&str] = &[
    "a","abbr","address","area","article","aside","audio","b","base","bdi","bdo","blockquote","body","br","button",
    "canvas","caption","cite","code","col","colgroup","data","datalist","dd","del","details","dfn","dialog","div",
    "dl","dt","em","embed","fieldset","figcaption","figure","footer","form","h1","h2","h3","h4","h5","h6","head",
    "header","hgroup","hr","html","i","iframe","img","input","ins","kbd","label","legend","li","link","main","map",
    "mark","menu","meta","meter","nav","noscript","object","ol","optgroup","option","output","p","param","picture",
    "pre","progress","q","rp","rt","ruby","s","samp","script","search","section","select","slot","small","source",
    "span","strong","style","sub","summary","sup","table","tbody","td","template","textarea","tfoot","th","thead",
    "time","title","tr","track","u","ul","var","video","wbr",
];

const HIDDEN: &[&str] = &["head","script","style","meta","link","title","template","base","noscript","param","source","track","datalist","area"];
const INLINE: &[&str] = &[
    "a","abbr","b","bdi","bdo","br","cite","code","data","del","dfn","em","i","ins","kbd","label","mark","q","rp","rt",
    "ruby","s","samp","small","span","strong","sub","sup","time","u","var","wbr","img","input","button","select",
    "textarea","output","meter","progress","audio","video","canvas","iframe","embed","object","picture","map",
];

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DefaultDisplay { None, Block, Inline }

pub fn default_display(tag: &str) -> DefaultDisplay {
    if HIDDEN.contains(&tag) { DefaultDisplay::None }
    else if INLINE.contains(&tag) { DefaultDisplay::Inline }
    else { DefaultDisplay::Block }
}

/// UA stylesheet declarations (inline CSS text) per tag.
pub fn ua_style(tag: &str) -> &'static str {
    match tag {
        "h1" => "font-size:32px;font-weight:bold;margin-top:21px;margin-bottom:21px",
        "h2" => "font-size:24px;font-weight:bold;margin-top:20px;margin-bottom:20px",
        "h3" => "font-size:19px;font-weight:bold;margin-top:19px;margin-bottom:19px",
        "h4" => "font-size:16px;font-weight:bold;margin-top:21px;margin-bottom:21px",
        "h5" => "font-size:13px;font-weight:bold;margin-top:22px;margin-bottom:22px",
        "h6" => "font-size:11px;font-weight:bold;margin-top:24px;margin-bottom:24px",
        "p" | "dl" | "pre" | "figure" => "margin-top:16px;margin-bottom:16px",
        "ul" | "ol" => "margin-top:16px;margin-bottom:16px;padding-left:40px",
        "blockquote" => "margin:16px 40px",
        "hr" => "height:1px;background-color:gray;margin-top:8px;margin-bottom:8px",
        "b" | "strong" | "th" => "font-weight:bold",
        "button" => "padding:4px 8px;background-color:#efefef;border-radius:3px",
        "input" | "textarea" | "select" => "padding:2px 4px;background-color:white;border-width:1px;border-color:#767676;min-width:120px;min-height:20px",
        "body" => "margin:8px",
        _ => "",
    }
}
