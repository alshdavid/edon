use crate::napi::bindgen_runtime::ToNapiValue;
use crate::napi::bindgen_runtime::TypeName;
use crate::napi::Env;
use crate::napi::Error;
use crate::napi::Result;

pub trait Task: Send + Sized {
  type Output: Send + Sized + 'static;
  type JsValue: ToNapiValue + TypeName;

  /// Compute logic in libuv thread
  fn compute(&mut self) -> Result<Self::Output>;

  /// Into this method if `compute` return `Ok`
  fn resolve(
    &mut self,
    env: Env,
    output: Self::Output,
  ) -> Result<Self::JsValue>;

  /// Into this method if `compute` return `Err`
  fn reject(
    &mut self,
    _env: Env,
    err: Error,
  ) -> Result<Self::JsValue> {
    Err(err)
  }

  // after resolve or reject
  fn finally(
    &mut self,
    _env: Env,
  ) -> Result<()> {
    Ok(())
  }
}
