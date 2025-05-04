use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use super::internal;
use super::NodejsWorker;
use crate::internal::constants::LIB_NAME;
use crate::internal::NodejsEvent;
use crate::internal::PathExt;

pub struct Nodejs {
  tx_eval: Sender<NodejsEvent>,
}

impl Nodejs {
  /// Load libnode.so by path
  pub fn load<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
    let _ = libnode_sys::load::cdylib(path);
    let tx_eval = internal::start_node_instance();
    Ok(Self { tx_eval })
  }

  /// Look for libnode.so from
  /// env:EDON_LIBNODE_PATH
  /// <exe_path>/libnode.so
  /// <exe_path>/lib/libnode.so
  /// <exe_path>/share/libnode.so
  /// <exe_path>/../lib/libnode.so
  /// <exe_path>/../share/libnode.so
  pub fn load_auto() -> crate::Result<Self> {
    if let Ok(path) = std::env::var("EDON_LIBNODE_PATH") {
      let _ = libnode_sys::load::cdylib(&path);
    } else {
      let dirname = std::env::current_exe()?.try_parent()?;

      let paths = vec![
        dirname.join(LIB_NAME),
        dirname.join("lib").join(LIB_NAME),
        dirname.join("share").join(LIB_NAME),
        dirname.join("..").join("lib").join(LIB_NAME),
        dirname.join("..").join("share").join(LIB_NAME),
      ];

      for path in paths {
        if std::fs::exists(&path)? {
          let _ = libnode_sys::load::cdylib(&path);
          break;
        }
      }
    };

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
    F: 'static
      + Sync
      + Send
      + Fn(libnode_sys::napi_env, libnode_sys::napi_value) -> libnode_sys::napi_value,
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
    self
      .tx_eval
      .send(NodejsEvent::Eval {
        code: code.as_ref().to_string(),
        resolve: tx,
      })
      .ok();
    rx.recv().unwrap();
    Ok(())
  }

  pub fn env<F: 'static + FnOnce(libnode_sys::napi_env)>(
    &self,
    callback: F,
  ) {
    self
      .tx_eval
      .send(NodejsEvent::Env {
        callback: Box::new(callback),
      })
      .ok();
  }

  /// Evaluate block of JavaScript
  pub fn spawn_worker(&self) -> crate::Result<NodejsWorker> {
    // let (tx, rx) = channel();
    // self.tx_eval.send((code.as_ref().to_string(), tx)).unwrap();
    // rx.recv().unwrap();
    Ok(NodejsWorker {})
  }
}
