use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::OnceLock;

use super::internal;
use super::NodejsContext;
use crate::internal::constants::LIB_NAME;
use crate::internal::NodejsContextEvent;
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


  /// Spawn a Nodejs worker thread
  pub fn spawn_context(&self) -> crate::Result<NodejsContext> {
    let (tx, rx) = channel();
    let (tx_wrk, rx_wrk) = channel::<NodejsContextEvent>();

    self
      .tx_eval
      .send(NodejsEvent::StartCommonjsWorker { rx_wrk, resolve: tx })
      .ok();

    rx.recv().unwrap();
    NodejsContext::start(tx_wrk)
  }
}
