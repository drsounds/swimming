//! Compile-time JavaScript engine selection. Each backend is its own crate, pulled in by a Cargo
//! feature (`v8`, `quickjs`); [`DefaultEngine`] is the one `JsBridge` uses.
pub use swimming_js::*;

#[cfg(feature = "quickjs")]
pub use swimming_js_quickjs::QuickJsEngine;
#[cfg(feature = "v8")]
pub use swimming_js_v8::V8Engine;

/// When several backends are compiled in, the explicitly opted-in `quickjs` wins over the default `v8`.
#[cfg(feature = "quickjs")]
pub type DefaultEngine = QuickJsEngine;
#[cfg(all(feature = "v8", not(feature = "quickjs")))]
pub type DefaultEngine = V8Engine;

#[cfg(not(any(feature = "v8", feature = "quickjs")))]
compile_error!("swimming-bridge needs a JavaScript engine: enable the `v8` or `quickjs` feature");
