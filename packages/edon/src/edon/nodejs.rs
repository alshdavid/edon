use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use super::internal;
use super::NodejsWorker;
use crate::sys::{self};

pub struct Nodejs {
  tx_eval: Sender<(String, Sender<()>)>,
}

impl Nodejs {
  /// Load libnode.so by path
  pub fn load<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
    let _ = sys::library::load(path);
    let tx_eval = internal::start_node_instance();
    Ok(Self { tx_eval })
  }

  /// Look for libnode.so from
  /// env:EDON_LIBNODE_PATH
  /// <exe_path>/libnode.so
  /// <exe_path>/../lib/libnode.so
  /// <exe_path>/../share/libnode.so
  pub fn load_auto() -> crate::Result<Self> {
    let _ = sys::library::load_auto();
    let tx_eval = internal::start_node_instance();
    Ok(Self { tx_eval })
  }

  pub fn is_running() -> bool {
    internal::is_running()
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

  /// Evaluate block of JavaScript
  pub fn eval<Code: AsRef<str>>(
    &self,
    code: Code,
  ) -> crate::Result<()> {
    let (tx, rx) = channel();
    self.tx_eval.send((code.as_ref().to_string(), tx)).unwrap();
    rx.recv().unwrap();
    Ok(())
  }

  /// Evaluate block of JavaScript
  pub fn new_worker(&self) -> crate::Result<NodejsWorker> {
    // let (tx, rx) = channel();
    // self.tx_eval.send((code.as_ref().to_string(), tx)).unwrap();
    // rx.recv().unwrap();
    Ok(NodejsWorker {})
  }
}
