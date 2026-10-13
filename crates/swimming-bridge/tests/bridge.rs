use swimming_bridge::JsBridge;

#[test]
fn js_builds_tree_and_native_events_reach_js() {
    let b = JsBridge::new(200.0, 100.0).unwrap();
    b.eval(r#"
        const n = __SWIMMING_NATIVE__;
        globalThis.clicked = [];
        const box = n.createNode("div", "width:50px;height:20px;background-color:red", "{}");
        n.appendChild(0, box);
        n.commit();
        globalThis.__SWIMMING_DISPATCH__ = (id, ev) => clicked.push(id + ":" + ev);
    "#).unwrap();
    assert_eq!(b.pointer("click", 5.0, 5.0).unwrap(), Some(1));
    assert_eq!(b.pointer("click", 150.0, 80.0).unwrap(), None);
    b.eval("if (clicked.join() !== '1:click') throw new Error('bad dispatch: ' + clicked)").unwrap();
    let px = b.render().unwrap().pixel(2, 2).unwrap();
    assert_eq!((px.red(), px.green(), px.blue()), (255, 0, 0));
}
