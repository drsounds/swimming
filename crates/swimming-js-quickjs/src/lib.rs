//! swimming-js-quickjs: QuickJS backend (via rquickjs) for swimming-js.
use swimming_js::{JsEngine, JsError, JsValue, NativeFunction};
use rquickjs::{Context, Ctx, Exception, FromJs, Function, IntoJs, Object, Runtime, Value, function::Rest};

pub struct QuickJsEngine {
    // Field order matters: the context must drop before the runtime.
    context: Context,
    runtime: Runtime,
}

/// Local wrapper so rquickjs's conversion traits can be implemented for `JsValue`.
struct Val(JsValue);

impl<'js> FromJs<'js> for Val {
    fn from_js(_ctx: &Ctx<'js>, value: Value<'js>) -> rquickjs::Result<Self> {
        Ok(Val(if let Some(b) = value.as_bool() {
            JsValue::Bool(b)
        } else if let Some(n) = value.as_number() {
            JsValue::Number(n)
        } else if let Some(s) = value.as_string() {
            JsValue::String(s.to_string()?)
        } else {
            JsValue::Undefined
        }))
    }
}

impl<'js> IntoJs<'js> for Val {
    fn into_js(self, ctx: &Ctx<'js>) -> rquickjs::Result<Value<'js>> {
        Ok(match self.0 {
            JsValue::Undefined => Value::new_undefined(ctx.clone()),
            JsValue::Bool(b) => Value::new_bool(ctx.clone(), b),
            JsValue::Number(n) => Value::new_number(ctx.clone(), n),
            JsValue::String(s) => rquickjs::String::from_str(ctx.clone(), &s)?.into_value(),
        })
    }
}

/// Renders the pending exception (with stack) rather than rquickjs's generic "Exception generated".
fn describe(ctx: &Ctx<'_>, error: rquickjs::Error) -> JsError {
    JsError(
        ctx.catch()
            .into_object()
            .and_then(Exception::from_object)
            .map(|exception| exception.to_string())
            .unwrap_or_else(|| error.to_string()),
    )
}

impl JsEngine for QuickJsEngine {
    const NAME: &'static str = "QuickJS";

    fn new() -> Result<Self, JsError> {
        let runtime = Runtime::new().map_err(|e| JsError(e.to_string()))?;
        let context = Context::full(&runtime).map_err(|e| JsError(e.to_string()))?;
        Ok(QuickJsEngine { context, runtime })
    }

    fn register_object(&self, name: &str, functions: Vec<(&'static str, NativeFunction)>) -> Result<(), JsError> {
        self.context.with(|ctx| {
            let result = (|| {
                let object = Object::new(ctx.clone())?;
                for (key, native) in functions {
                    object.set(key, Function::new(ctx.clone(), move |args: Rest<Val>| {
                        let args: Vec<JsValue> = args.0.into_iter().map(|v| v.0).collect();
                        Val(native(&args))
                    })?)?;
                }
                ctx.globals().set(name, object)
            })();
            result.map_err(|error| describe(&ctx, error))
        })
    }

    fn eval(&self, source: &str) -> Result<(), JsError> {
        self.context.with(|ctx| ctx.eval::<(), _>(source).map_err(|error| describe(&ctx, error)))
    }

    fn call_global(&self, name: &str, args: &[JsValue]) -> Result<Option<JsValue>, JsError> {
        self.context.with(|ctx| {
            let Ok(function) = ctx.globals().get::<_, Function>(name) else { return Ok(None) };
            function
                .call::<_, Val>((Rest(args.iter().cloned().map(Val).collect()),))
                .map(|value| Some(value.0))
                .map_err(|error| describe(&ctx, error))
        })
    }

    fn run_jobs(&self) {
        while self.runtime.execute_pending_job().unwrap_or(false) {}
    }
}
