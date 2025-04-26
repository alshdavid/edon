use std::path::Path;

use crate::sys;

use super::internal;

pub struct Nodejs {}

impl Nodejs {
  pub fn load<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
    let _ = sys::library::load(path);
    Ok(Self{})
  }

  pub fn load_auto() -> crate::Result<Self> {
    let _ = sys::library::load_auto();
    Ok(Self{})
  }

  pub fn start_blocking<Args: AsRef<str>>(&self, argv: &[Args]) -> crate::Result<()> {
    internal::start_blocking(argv)
  }

  pub fn eval_blocking<Code: AsRef<str>>(&self, code: Code) -> crate::Result<()> {
    internal::eval_blocking(code)
  }

  pub fn napi_module_register<
    S: AsRef<str>,
    F: 'static
      + Sync
      + Send
      + Fn(sys::napi::napi_env, sys::napi::napi_value) -> sys::napi::napi_value,
  >(
    &self,
    module_name: S,
    register_function: F,
  ) {
    internal::napi_module_register(module_name, register_function)
  }

  pub fn is_running() -> bool {
    internal::is_running()
  }
}