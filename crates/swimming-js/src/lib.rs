//! swimming-js: the engine-neutral JavaScript interface used by swimming-bridge.
//! Backends (`swimming-js-v8`, `swimming-js-quickjs`, ...) implement [`JsEngine`].
use std::fmt;

/// Engine-neutral value crossing the Rust/JS boundary. Anything else (objects, functions, ...)
/// is seen as `Undefined` on the Rust side.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum JsValue {
    #[default]
    Undefined,
    Bool(bool),
    Number(f64),
    String(String),
}

impl JsValue {
    pub fn as_f64(&self) -> f64 {
        match self {
            JsValue::Number(n) => *n,
            _ => f64::NAN,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            JsValue::String(s) => s,
            _ => "",
        }
    }

    pub fn is_true(&self) -> bool { matches!(self, JsValue::Bool(true)) }
}

/// A Rust function exposed to JS; receives the call's arguments.
pub type NativeFunction = Box<dyn Fn(&[JsValue]) -> JsValue>;

/// A JS exception or engine failure, carrying the engine's rendering of the error.
#[derive(Clone, Debug, PartialEq)]
pub struct JsError(pub String);

impl fmt::Display for JsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
}

impl std::error::Error for JsError {}

impl From<String> for JsError {
    fn from(message: String) -> Self { JsError(message) }
}

/// What the bridge needs from a JavaScript engine. Engines have no event loop of their own:
/// the bridge drives timers through `__SWIMMING_TICK__` and drains promise jobs via [`run_jobs`](Self::run_jobs).
pub trait JsEngine: Sized {
    /// Human-readable engine name, e.g. for diagnostics.
    const NAME: &'static str;

    fn new() -> Result<Self, JsError>;

    /// Defines `globalThis[name]` as an object whose methods call the given native functions.
    fn register_object(&self, name: &str, functions: Vec<(&'static str, NativeFunction)>) -> Result<(), JsError>;

    /// Evaluates a classic (non-module) script in the global scope.
    fn eval(&self, source: &str) -> Result<(), JsError>;

    /// Calls `globalThis[name](...args)`; returns `Ok(None)` when it is not a function.
    fn call_global(&self, name: &str, args: &[JsValue]) -> Result<Option<JsValue>, JsError>;

    /// Runs all pending promise jobs (microtasks).
    fn run_jobs(&self);
}
