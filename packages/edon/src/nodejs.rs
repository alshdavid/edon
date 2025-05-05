use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::OnceLock;

use super::internal;
use super::NodejsWorker;
use crate::internal::constants::LIB_NAME;
use crate::internal::NodejsEvent;
use crate::internal::PathExt;
use crate::napi::JsObject;
use crate::Env;

// Due to a quirk of v8, only one instance of Nodejs can be used per process.
// The current C FFI does not allow spawning multiple contexts so to get around
// this for now, we store the Nodejs instance as a static and inject
// a JavaScript prelude that creates "vm" instances to act as contexts.
//
// The consumer can also spawn and interact with Nodejs worker threads.
static NODEJS: OnceLock<crate::Result<NodejsRef>> = OnceLock::new();
pub type NodejsRef = Arc<Nodejs>;

pub struct Nodejs {
  tx_eval: Sender<NodejsEvent>,
}

impl Nodejs {
  /// Load libnode by path
  /// ```
  /// Windows:  "libnode.dll"
  /// MacOS:    "libnode.dylib"
  /// Linux:    "libnode.so"
  /// ```
  pub fn load<P: AsRef<Path>>(path: P) -> crate::Result<NodejsRef> {
    let nodejs = NODEJS.get_or_init(move || {
      let _ = libnode_sys::load::cdylib(path);
      let tx_eval = internal::start_node_instance()?;
      Ok(Arc::new(Self { tx_eval }))
    });

    match nodejs {
      Ok(nodejs) => Ok(nodejs.clone()),
      Err(err) => Err(err.clone()),
    }
  }

  /// Look for libnode.so from
  ///
  /// * $EDON_LIBNODE_PATH
  /// * <exe_path>/libnode.so
  /// * <exe_path>/lib/libnode.so
  /// * <exe_path>/share/libnode.so
  /// * <exe_path>/../lib/libnode.so
  /// * <exe_path>/../share/libnode.so
  pub fn load_auto() -> crate::Result<NodejsRef> {
    if let Ok(path) = std::env::var("EDON_LIBNODE_PATH") {
      Self::load(&path)
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
          return Self::load(&path);
        }
      }

      Err(crate::Error::LibnodeNotFound)
    }
  }

  /// Register native module
  ///
  /// This runs once per main/worker thread and is accessible
  /// in JavaScript via `importNative("my_native_extension")`
  pub fn napi_module_register<
    S: AsRef<str>,
    F: 'static + Sync + Send + Fn(Env, JsObject) -> crate::Result<JsObject>,
  >(
    &self,
    module_name: S,
    register_function: F,
  ) -> crate::Result<()> {
    internal::napi_module_register(module_name, register_function)
  }

  /// Evaluate Block of Commonjs JavaScript
  ///
  /// The last line of the script will be returned
  pub fn eval_script<Code: AsRef<str>>(
    &self,
    code: Code,
  ) -> crate::Result<()> {
    let (tx, rx) = channel();
    let tx_eval = self.tx_eval.clone();
    let code = code.as_ref().to_string();

    tx_eval
      .send(NodejsEvent::EvalScript { code, resolve: tx })
      .ok();

    rx.recv().unwrap();
    Ok(())
  }

  /// Evaluate Block of ESM JavaScript
  ///
  /// The last line of the script will be returned
  pub fn eval_module<Code: AsRef<str>>(
    &self,
    code: Code,
  ) -> crate::Result<()> {
    let (tx, rx) = channel();
    let tx_eval = self.tx_eval.clone();
    let code = code.as_ref().to_string();

    tx_eval
      .send(NodejsEvent::EvalModule { code, resolve: tx })
      .ok();

    rx.recv().unwrap();
    Ok(())
  }

  /// Evaluate Native JavaScript
  ///
  /// This will provide a Nodejs Env and allow execution of
  /// native code in the JavaScript context
  pub fn exec<F: 'static + Send + FnOnce(Env) -> crate::Result<()>>(
    &self,
    callback: F,
  ) -> crate::Result<()> {
    let (tx, rx) = channel();

    self
      .tx_eval
      .send(NodejsEvent::Env {
        callback: Box::new(callback),
        resolve: tx,
      })
      .ok();

    rx.recv().unwrap()
  }

  /// Spawn a Nodejs worker thread
  pub fn spawn_worker(&self) -> crate::Result<NodejsWorker> {
    // let (tx, rx) = channel();
    // self.tx_eval.send((code.as_ref().to_string(), tx)).unwrap();
    // rx.recv().unwrap();
    Ok(NodejsWorker {})
  }
}
