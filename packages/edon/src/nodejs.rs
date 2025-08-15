use std::path::Path;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

use napi::JsObject;
use napi::JsUnknown;
use napi::Env;

use super::internal;
use super::NodejsWorker;
use crate::internal::NodejsMainEvent;
use crate::NodejsOptions;

// Due to a quirk of v8, only one instance of Nodejs can be used per process.
// The current C FFI does not allow spawning multiple contexts so to get around
// this for now, we store the Nodejs instance as a static and inject
// a JavaScript prelude that creates "vm" instances to act as contexts.
//
// The consumer can also spawn and interact with Nodejs worker threads.
static NODEJS: OnceLock<crate::Result<NodejsRef>> = OnceLock::new();
pub(crate) static NODEJS_CONTEXT_COUNT: AtomicU32 = AtomicU32::new(0);

pub type NodejsRef = Sender<NodejsMainEvent>;

pub struct Nodejs {
  tx_main: NodejsRef,
}

impl Nodejs {
  /// Load libnode by path
  /// ```
  /// Windows:  "libnode.dll"
  /// MacOS:    "libnode.dylib"
  /// Linux:    "libnode.so"
  /// ```
  pub fn load_with_args<P: AsRef<Path>, Args: AsRef<str>>(
    path: P,
    args: &[Args],
  ) -> crate::Result<Nodejs> {
    NODEJS_CONTEXT_COUNT.fetch_add(1, Ordering::AcqRel);

    let nodejs = NODEJS.get_or_init(move || {
      let lib = libnode_sys::load::cdylib(path.as_ref()).unwrap();
      napi::setup(path.as_ref());

      let tx_main = internal::start_node_instance(args)?;
      Ok(tx_main)
    });

    match nodejs {
      Ok(nodejs) => Ok(Self {
        tx_main: nodejs.clone(),
      }),
      Err(err) => Err(err.clone()),
    }
  }

  /// Load libnode by path
  /// ```
  /// Windows:  "libnode.dll"
  /// MacOS:    "libnode.dylib"
  /// Linux:    "libnode.so"
  /// ```
  pub fn load_default<P: AsRef<Path>>(path: P) -> crate::Result<Nodejs> {
    Self::load_with_args(path, &[] as &[&str])
  }

  /// Load libnode by path
  /// ```
  /// Windows:  "libnode.dll"
  /// MacOS:    "libnode.dylib"
  /// Linux:    "libnode.so"
  /// ```
  pub fn load(options: NodejsOptions) -> crate::Result<Nodejs> {
    Self::load_with_args(options.libnode_path.clone(), &options.as_argv())
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
  pub fn spawn_worker_thread(&self) -> crate::Result<NodejsWorker> {
    self.spawn_worker_thread_with_options(&NodejsOptions::default())
  }

  pub fn spawn_worker_thread_with_options(
    &self,
    options: &NodejsOptions,
  ) -> crate::Result<NodejsWorker> {
    NodejsWorker::start(options, self.tx_main.clone())
  }

  pub fn eval<Code: AsRef<str>>(
    &self,
    code: Code,
    callback: impl 'static + Send + FnOnce(Env, JsUnknown),
  ) -> crate::Result<()> {
    self
      .tx_main
      .send(NodejsMainEvent::Eval {
        code: code.as_ref().to_string(),
        callback: Box::new(callback),
      })
      .unwrap();

    Ok(())
  }

  /// Evaluate Block of Commonjs JavaScript
  ///
  /// The last line of the script will be returned
  pub fn eval_blocking<Code: AsRef<str>>(
    &self,
    code: Code,
  ) -> crate::Result<()> {
    let (tx, rx) = channel();

    self
      .tx_main
      .send(NodejsMainEvent::Eval {
        code: code.as_ref().to_string(),
        callback: Box::new(move |_env, _val| {
          tx.send(Ok(())).unwrap();
        }),
      })
      .ok();

    rx.recv().unwrap()
  }

  /// Evaluate Block of ESM JavaScript
  pub fn eval_typescript<Code: AsRef<str>>(
    &self,
    code: Code,
    callback: impl 'static + Send + FnOnce(Env, JsUnknown),
  ) -> crate::Result<()> {
    self
      .tx_main
      .send(NodejsMainEvent::EvalTypeScript {
        code: code.as_ref().to_string(),
        callback: Box::new(callback),
      })
      .unwrap();

    Ok(())
  }

  /// Evaluate Block of ESM JavaScript
  pub fn eval_typescript_blocking<Code: AsRef<str>>(
    &self,
    code: Code,
  ) -> crate::Result<()> {
    let (tx, rx) = channel();

    self
      .tx_main
      .send(NodejsMainEvent::EvalTypeScript {
        code: code.as_ref().to_string(),
        callback: Box::new(move |_env, _val| {
          tx.send(Ok(())).unwrap();
        }),
      })
      .ok();

    rx.recv().unwrap()
  }

  /// Evaluate Native JavaScript
  ///
  /// This will provide a Nodejs Env and allow execution of
  /// native code in the JavaScript context
  pub fn exec_blocking<F: 'static + Send + FnOnce(Env) -> crate::Result<()>>(
    &self,
    callback: F,
  ) -> crate::Result<()> {
    let (tx, rx) = channel();

    self
      .tx_main
      .send(NodejsMainEvent::Exec {
        callback: Box::new(move |env| {
          let result = callback(env);
          tx.send(Ok(())).unwrap();
          result
        }),
      })
      .ok();

    rx.recv().unwrap()
  }

  /// Evaluate Native JavaScript
  ///
  /// This will provide a Nodejs Env and allow execution of
  /// native code in the JavaScript context
  pub fn exec<F: 'static + Send + FnOnce(Env) -> crate::Result<()>>(
    &self,
    callback: F,
  ) -> crate::Result<()> {
    self
      .tx_main
      .send(NodejsMainEvent::Exec {
        callback: Box::new(callback),
      })
      .unwrap();
    
    Ok(())
  }

  /// Call Nodejs's require() function to import code
  pub fn require<Specifier: AsRef<str>>(
    &self,
    specifier: Specifier,
  ) -> crate::Result<()> {
    let (tx, rx) = channel();

    self
      .tx_main
      .send(NodejsMainEvent::Require {
        specifier: specifier.as_ref().to_string(),
        resolve: tx,
      })
      .ok();

    rx.recv().unwrap()
  }

  /// Call Nodejs's await import() to import code
  pub fn import<Specifier: AsRef<str>>(
    &self,
    specifier: Specifier,
  ) -> crate::Result<()> {
    let (tx, rx) = channel();

    self
      .tx_main
      .send(NodejsMainEvent::Import {
        specifier: specifier.as_ref().to_string(),
        resolve: tx,
      })
      .ok();

    rx.recv().unwrap()
  }
}

impl Drop for Nodejs {
  fn drop(&mut self) {
    let context_count = NODEJS_CONTEXT_COUNT.fetch_sub(1, Ordering::AcqRel);
    if context_count == 1 {
      let (tx, rx) = channel();
      self
        .tx_main
        .send(NodejsMainEvent::StopMain { resolve: tx })
        .unwrap();
      rx.recv().unwrap();
    }
  }
}
