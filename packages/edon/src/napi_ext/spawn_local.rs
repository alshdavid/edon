use futures::Future;

use super::runtime;
use crate::napi::Env;
use crate::napi::JsObject;
use crate::napi::NapiValue;

pub fn spawn_local<Fut>(
  env: &Env,
  future: Fut,
) -> crate::napi::Result<()>
where
  Fut: Future<Output = crate::napi::Result<()>> + 'static,
{
  runtime::spawn_local_fut(*env, async move {
    if let Err(error) = future.await {
      eprintln!("Uncaught Napi Error: {}", error);
    };
  })?;

  Ok(())
}

pub fn spawn_local_promise<R, Fut>(
  env: &Env,
  future: Fut,
) -> crate::napi::Result<JsObject>
where
  R: NapiValue + 'static,
  Fut: Future<Output = crate::napi::Result<R>> + 'static,
{
  env.create_promise(Box::new(move |env, resolve_func, reject_func| {
    runtime::spawn_local_fut(env, async move {
      match future.await {
        Ok(result) => resolve_func(result),
        Err(error) => reject_func(error),
      };
    })
  }))
}
