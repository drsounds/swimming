use swimming_core::{SwimmingEngine, ROOT};

fn div(e: &mut SwimmingEngine, style: &str) -> u64 { e.create_node("div", style, "{}") }

#[test]
fn lays_out_css_with_lightningcss_and_taffy() {
    let mut e = SwimmingEngine::new();
    let row = div(&mut e, "display:flex;flex-direction:row;padding:10px");
    let a = div(&mut e, "width:50px;height:20px");
    let b = div(&mut e, "width:30px;height:20px;margin-left:5px");
    e.append_child(row, a);
    e.append_child(row, b);
    e.append_child(ROOT, row);
    e.compute_layout(200.0, 100.0);
    let r = &e.layout().unwrap().root.children[0];
    assert_eq!((r.children[0].x, r.children[0].y, r.children[0].w, r.children[0].h), (10.0, 10.0, 50.0, 20.0));
    assert_eq!(r.children[1].x, 65.0);
}

#[test]
fn every_html_element_renders_and_head_is_hidden() {
    let mut e = SwimmingEngine::new();
    let main = e.create_node("main", "", "{}");
    let h1 = e.create_node("h1", "", "{}");
    let t = e.create_text("Hi");
    e.append_child(h1, t);
    e.append_child(main, h1);
    for tag in ["script", "table", "video"] {
        let n = e.create_node(tag, "", "{}");
        e.append_child(main, n);
    }
    e.append_child(ROOT, main);
    e.compute_layout(100.0, 100.0);
    let kids = &e.layout().unwrap().root.children[0].children;
    let tags: Vec<_> = kids.iter().map(|k| k.tag.as_str()).collect();
    assert_eq!(tags, ["h1", "table", "#anon"]);
    assert!(kids[0].h > 30.0);
}

#[test]
fn insert_before_remove_and_hit_test() {
    let mut e = SwimmingEngine::new();
    let a = div(&mut e, "width:50px;height:20px");
    let b = div(&mut e, "width:50px;height:20px");
    e.append_child(ROOT, b);
    e.insert_before(ROOT, a, Some(b));
    e.compute_layout(200.0, 100.0);
    assert_eq!(e.hit_test(5.0, 5.0), Some(a));
    assert_eq!(e.hit_test(5.0, 25.0), Some(b));
    assert_eq!(e.hit_test(150.0, 90.0), None);
    let before = e.node_count();
    let inner = div(&mut e, "");
    e.append_child(a, inner);
    e.remove_child(ROOT, a);
    assert_eq!(e.node_count(), before - 1);
    assert_eq!(e.parent(b), Some(ROOT));
}
