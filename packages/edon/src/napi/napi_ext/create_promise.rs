use std::cell::Cell;

use super::js_rc::JsRc;
use crate::napi::Env;
use crate::napi::JsFunction;
use crate::napi::JsObject;
use crate::napi::NapiValue;

const SYM_PROMISE: &str = "Promise";
const SYM_PROMISE_EXECUTOR: &str = "napi::promise::executor";

pub type PromiseExecutor<Res> = Box<
  dyn FnOnce(Env, Box<dyn Fn(Res)>, Box<dyn Fn(crate::napi::Error)>) -> crate::napi::Result<()>,
>;

pub fn create_promise<Res>(
  env: &Env,
  executor: PromiseExecutor<Res>,
) -> crate::napi::Result<JsObject>
where
  Res: NapiValue + 'static,
{
  let promise_key = env.create_string(SYM_PROMISE)?;
  let promise_ctor: JsFunction = env.get_global()?.get_property_unchecked(promise_key)?;

  let executor = {
    let cell = Cell::new(Some(executor));
    move |env, res, rej| {
      cell
        .take()
        .expect("This function should not be called more than once")(env, res, rej)
    }
  };

  let executor = env.create_function_from_closure(SYM_PROMISE_EXECUTOR, move |ctx| {
    let resolve_func_js: JsFunction = ctx.get(0)?;
    let reject_func: JsFunction = ctx.get(1)?;

    let resolve_func = JsRc::new(ctx.env, resolve_func_js)?;
    let reject_func = JsRc::new(ctx.env, reject_func)?;

    executor(
      ctx.env.to_owned(),
      Box::new({
        move |r| {
          resolve_func.call(None, &[r]).unwrap();
        }
      }),
      Box::new({
        let env = *ctx.env;
        move |e| {
          let error = (env).create_error(e).unwrap();
          reject_func.call(None, &[error]).unwrap();
        }
      }),
    )?;

    Ok(())
  })?;

  let promise = promise_ctor.new_instance(&[executor])?;
  Ok(promise)
}
