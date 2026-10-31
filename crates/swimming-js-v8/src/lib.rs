//! swimming-js-v8: V8 backend (via rusty_v8) for swimming-js.
use swimming_js::{JsEngine, JsError, JsValue, NativeFunction};
use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::Once;

pub struct V8Engine {
    // Field order matters: the context handle must drop before the isolate,
    // and the natives (referenced from JS functions) after it.
    context: v8::Global<v8::Context>,
    isolate: RefCell<v8::OwnedIsolate>,
    natives: RefCell<Vec<Box<NativeFunction>>>,
}

fn init_platform() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

fn from_v8(scope: &v8::PinScope, value: v8::Local<v8::Value>) -> JsValue {
    if value.is_boolean() {
        JsValue::Bool(value.boolean_value(scope))
    } else if value.is_number() {
        JsValue::Number(value.number_value(scope).unwrap_or(f64::NAN))
    } else if value.is_string() {
        JsValue::String(value.to_rust_string_lossy(scope))
    } else {
        JsValue::Undefined
    }
}

fn to_v8<'s>(scope: &v8::PinScope<'s, '_>, value: &JsValue) -> v8::Local<'s, v8::Value> {
    match value {
        JsValue::Undefined => v8::undefined(scope).into(),
        JsValue::Bool(b) => v8::Boolean::new(scope, *b).into(),
        JsValue::Number(n) => v8::Number::new(scope, *n).into(),
        JsValue::String(s) => v8::String::new(scope, s).map_or_else(|| v8::undefined(scope).into(), Into::into),
    }
}

fn key<'s>(scope: &v8::PinScope<'s, '_>, name: &str) -> Result<v8::Local<'s, v8::Value>, JsError> {
    v8::String::new(scope, name).map(Into::into).ok_or_else(|| JsError(format!("invalid property name '{name}'")))
}

/// Prefers the stack trace (which includes the message) over the bare exception string.
fn describe(scope: &v8::PinnedRef<v8::TryCatch<v8::HandleScope>>) -> JsError {
    let Some(exception) = scope.exception() else { return JsError("execution terminated".to_owned()) };
    let stack = scope.stack_trace().filter(|s| s.is_string()).map(|s| s.to_rust_string_lossy(scope));
    JsError(stack.unwrap_or_else(|| exception.to_rust_string_lossy(scope)))
}

/// Shared callback for all native functions; the `NativeFunction` is passed as the function's data.
fn trampoline(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    let Ok(external) = v8::Local::<v8::External>::try_from(args.data()) else { return };
    // SAFETY: points into `V8Engine::natives`, which outlives the isolate and every function in it.
    let native = unsafe { &*(external.value() as *const NativeFunction) };
    let values: Vec<JsValue> = (0..args.length()).map(|i| from_v8(scope, args.get(i))).collect();
    rv.set(to_v8(scope, &native(&values)));
}

impl JsEngine for V8Engine {
    const NAME: &'static str = "V8";

    fn new() -> Result<Self, JsError> {
        init_platform();
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        // Match the other engines: promise jobs only run when the bridge drains them.
        isolate.set_microtasks_policy(v8::MicrotasksPolicy::Explicit);
        let context = {
            v8::scope!(let scope, &mut isolate);
            let context = v8::Context::new(scope, Default::default());
            v8::Global::new(scope, context)
        };
        Ok(V8Engine { context, isolate: RefCell::new(isolate), natives: RefCell::new(Vec::new()) })
    }

    fn register_object(&self, name: &str, functions: Vec<(&'static str, NativeFunction)>) -> Result<(), JsError> {
        let isolate = &mut *self.isolate.borrow_mut();
        v8::scope_with_context!(let scope, isolate, &self.context);
        let object = v8::Object::new(scope);
        let mut natives = self.natives.borrow_mut();
        for (name, native) in functions {
            let native = Box::new(native);
            let external = v8::External::new(scope, &*native as *const NativeFunction as *mut c_void);
            natives.push(native);
            let function = v8::Function::builder(trampoline)
                .data(external.into())
                .build(scope)
                .ok_or_else(|| JsError(format!("cannot create native function '{name}'")))?;
            object.set(scope, key(scope, name)?, function.into());
        }
        let global = scope.get_current_context().global(scope);
        global.set(scope, key(scope, name)?, object.into());
        Ok(())
    }

    fn eval(&self, source: &str) -> Result<(), JsError> {
        let isolate = &mut *self.isolate.borrow_mut();
        v8::scope_with_context!(let scope, isolate, &self.context);
        v8::tc_scope!(let scope, scope);
        let code = v8::String::new(scope, source).ok_or_else(|| JsError("script source is too large".to_owned()))?;
        match v8::Script::compile(scope, code, None).and_then(|script| script.run(scope)) {
            Some(_) => Ok(()),
            None => Err(describe(scope)),
        }
    }

    fn call_global(&self, name: &str, args: &[JsValue]) -> Result<Option<JsValue>, JsError> {
        let isolate = &mut *self.isolate.borrow_mut();
        v8::scope_with_context!(let scope, isolate, &self.context);
        v8::tc_scope!(let scope, scope);
        let global = scope.get_current_context().global(scope);
        let Some(function) = global
            .get(scope, key(scope, name)?)
            .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        else {
            return Ok(None);
        };
        let args: Vec<_> = args.iter().map(|arg| to_v8(scope, arg)).collect();
        let receiver = v8::undefined(scope).into();
        match function.call(scope, receiver, &args) {
            Some(value) => Ok(Some(from_v8(scope, value))),
            None => Err(describe(scope)),
        }
    }

    fn run_jobs(&self) {
        self.isolate.borrow_mut().perform_microtask_checkpoint();
    }
}
