use swimming_native::{Engine, Message};

fn tree(json: &str) -> Message {
    serde_json::from_str(&format!(r#"{{"type":"tree","root":{json}}}"#)).unwrap()
}

#[test]
fn lays_out_css_with_lightningcss_and_taffy() {
    let mut e = Engine::new(200, 100);
    let r = e.handle(tree(r#"{"id":1,"type":"div","style":"display:flex;flex-direction:row;padding:10px","children":[
        {"id":2,"type":"span","style":"width:50px;height:20px"},
        {"id":3,"type":"span","style":"width:30px;height:20px;margin-left:5px"}]}"#));
    assert!(r.frame);
    let kids = &e.laid_out.as_ref().unwrap().root.children;
    assert_eq!((kids[0].x, kids[0].y, kids[0].w, kids[0].h), (10.0, 10.0, 50.0, 20.0));
    assert_eq!(kids[1].x, 65.0);
}

#[test]
fn every_html_element_renders_and_head_is_hidden() {
    let mut e = Engine::new(100, 100);
    e.handle(tree(r##"{"id":1,"type":"main","children":[
        {"id":2,"type":"h1","children":[{"id":3,"type":"#text","text":"Hi"}]},
        {"id":4,"type":"script"},{"id":5,"type":"table"},{"id":6,"type":"video"}]}"##));
    let kids = &e.laid_out.as_ref().unwrap().root.children;
    let tags: Vec<_> = kids.iter().map(|k| k.tag.as_str()).collect();
    assert_eq!(tags, ["h1", "table", "#anon"]); // script hidden, video inline
    assert!(kids[0].h > 30.0); // UA h1 font-size applied
}

#[test]
fn hit_testing_reports_deepest_element() {
    let mut e = Engine::new(200, 100);
    e.handle(tree(r#"{"id":1,"type":"div","children":[{"id":2,"type":"button","style":"width:50px;height:20px"}]}"#));
    let r = e.handle(Message::Pointer { event: "click".into(), x: 5.0, y: 5.0 });
    assert_eq!(r.json["target"], 2);
}

#[test]
fn paints_background() {
    let mut e = Engine::new(20, 20);
    e.handle(tree(r#"{"id":1,"type":"div","style":"background-color:red;width:10px;height:10px"}"#));
    let px = e.render().unwrap().pixel(2, 2).unwrap();
    assert_eq!((px.red(), px.green(), px.blue()), (255, 0, 0));
}
