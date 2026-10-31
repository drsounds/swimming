//! Runs the same behavioural checks against every JS engine backend compiled in.
use swimming_bridge::{Bridge, JsEngine};

fn conformance<E: JsEngine>() {
    let b = Bridge::<E>::new(200.0, 100.0).unwrap();

    // Values round-trip through native functions.
    let id = r#"
        var N = __SWIMMING_NATIVE__;
        var root = N.createNode("div", '{"width":"200px","height":"100px"}', "{}");
        var text = N.createText("héllo");
        if (typeof root !== "number" || typeof text !== "number") throw new Error("ids must be numbers");
        N.appendChild(root, text);
        N.commit();
    "#;
    b.eval(id).unwrap();

    // Promise jobs are drained after eval.
    b.eval("var resolved = false; Promise.resolve().then(function () { resolved = true; });").unwrap();
    b.eval("if (!resolved) throw new Error('microtasks were not run')").unwrap();

    // Exceptions surface with their message.
    let error = b.eval("throw new TypeError('boom')").unwrap_err();
    assert!(error.0.contains("boom"), "{} error lacks message: {error}", E::NAME);

    // Missing hooks are not an error.
    b.tick(16).unwrap();
}

#[cfg(feature = "v8")]
#[test]
fn v8_conformance() { conformance::<swimming_bridge::engine::V8Engine>(); }

#[cfg(feature = "quickjs")]
#[test]
fn quickjs_conformance() { conformance::<swimming_bridge::engine::QuickJsEngine>(); }
