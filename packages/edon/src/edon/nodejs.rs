use std::path::Path;

use super::internal;
use crate::sys;

pub struct Nodejs {}

impl Nodejs {
  /// Load libnode.so by path
  pub fn load<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
    let _ = sys::library::load(path);
    Ok(Self {})
  }

  /// Look for libnode.so from
  /// env:EDON_LIBNODE_PATH
  /// <exe_path>/libnode.so
  /// <exe_path>/../lib/libnode.so
  /// <exe_path>/../share/libnode.so
  pub fn load_auto() -> crate::Result<Self> {
    let _ = sys::library::load_auto();
    Ok(Self {})
  }

  /// Start Nodejs
  pub fn start_blocking<Args: AsRef<str>>(
    &self,
    argv: &[Args],
  ) -> crate::Result<()> {
    internal::start_blocking(argv)
  }

  /// Evaluate block of JavaScript
  pub fn eval_blocking<Code: AsRef<str>>(
    &self,
    code: Code,
  ) -> crate::Result<()> {
    internal::eval_blocking(code)
  }

  /// Register native module
  /// Accessible via "process._linkedBinding("my_native_extension")"
  pub fn napi_module_register<
    S: AsRef<str>,
    F: 'static + Sync + Send + Fn(sys::napi::napi_env, sys::napi::napi_value) -> sys::napi::napi_value,
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
