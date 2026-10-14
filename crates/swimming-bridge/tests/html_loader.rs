use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};
use swimming_bridge::{JsBridge, loader::run_source};

#[test]
fn loads_html_tree_and_external_script_from_http() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 1024];
            let length = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..length]);
            let (content_type, body) = if request.starts_with("GET /app.js ") {
                ("application/javascript", "globalThis.externalLoaded = 42;")
            } else {
                (
                    "text/html",
                    "<!doctype html><html><body><main style=\"width:120px;height:40px;background-color:red\"><p>hello</p></main><script src=\"/app.js\"></script><script>globalThis.inlineLoaded = true;</script></body></html>",
                )
            };
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        }
    });

    let bridge = JsBridge::new(200.0, 100.0).unwrap();
    run_source(&bridge, &format!("http://{address}/")).unwrap();
    bridge
        .eval("if (!inlineLoaded || externalLoaded !== 42) throw new Error('script tags were not run');")
        .unwrap();
    let engine = bridge.engine();
    let engine = engine.lock().unwrap();
    let layout = engine.layout().unwrap();
    assert_eq!(layout.root.children[0].tag, "main");
    assert_eq!(layout.root.children[0].children[0].tag, "p");
    assert_eq!(
        layout.root.children[0].computed.background,
        Some([255, 0, 0, 255])
    );
    drop(engine);
    server.join().unwrap();
}
