use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use crate::internal::NodejsContextEvent;
use crate::internal::NodejsEvent;
use crate::Env;
use crate::NodeOptions;
use crate::NODEJS_CONTEXT_COUNT;

pub struct NodejsContext {
  id: String,
  tx_main: Sender<NodejsEvent>,
  tx_wrk: Sender<NodejsContextEvent>,
}

impl NodejsContext {
  pub(crate) fn start(
    id: String,
    _options: &NodeOptions,
    tx_main: Sender<NodejsEvent>,
    tx_wrk: Sender<NodejsContextEvent>,
  ) -> crate::Result<Self> {
    return Ok(Self {
      id,
      tx_main,
      tx_wrk,
    });
  }

  /// Evaluate Block of Commonjs JavaScript
  ///
  /// The last line of the script will be returned
  pub fn eval<Code: AsRef<str>>(
    &self,
    code: Code,
  ) -> crate::Result<()> {
    let (tx, rx) = channel();
    let tx_eval = self.tx_wrk.clone();
    let code = code.as_ref().to_string();

    tx_eval
      .send(NodejsContextEvent::Eval { code, resolve: tx })
      .ok();

    rx.recv().unwrap()
  }

  /// Evaluate Block of ESM JavaScript
  pub fn eval_module<Code: AsRef<str>>(
    &self,
    code: Code,
  ) -> crate::Result<()> {
    let (tx, rx) = channel();
    let tx_eval = self.tx_wrk.clone();
    let code = code.as_ref().to_string();

    tx_eval
      .send(NodejsContextEvent::EvalModule { code, resolve: tx })
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
    let (tx, rx) = channel();

    self
      .tx_wrk
      .send(NodejsContextEvent::Exec {
        callback: Box::new(callback),
        resolve: tx,
      })
      .ok();

    rx.recv().unwrap()
  }

  pub fn require<Specifier: AsRef<str>>(
    &self,
    specifier: Specifier,
  ) -> crate::Result<()> {
    let (tx, rx) = channel();

    self
      .tx_wrk
      .send(NodejsContextEvent::Require {
        specifier: specifier.as_ref().to_string(),
        resolve: tx,
      })
      .ok();

    rx.recv().unwrap()
  }

  pub fn import<Specifier: AsRef<str>>(
    &self,
    specifier: Specifier,
  ) -> crate::Result<()> {
    let (tx, rx) = channel();

    self
      .tx_wrk
      .send(NodejsContextEvent::Import {
        specifier: specifier.as_ref().to_string(),
        resolve: tx,
      })
      .ok();

    rx.recv().unwrap()
  }
}

impl Drop for NodejsContext {
  fn drop(&mut self) {
    let (tx, rx) = channel();
    self
      .tx_main
      .send(NodejsEvent::StopCommonjsWorker {
        id: self.id.clone(),
        resolve: tx,
      })
      .unwrap();
    rx.recv().unwrap();

    let context_count = NODEJS_CONTEXT_COUNT.fetch_sub(1, Ordering::AcqRel);
    if context_count == 1 {
      let (tx, rx) = channel();
      self
        .tx_main
        .send(NodejsEvent::StopMain { resolve: tx })
        .unwrap();
      rx.recv().unwrap();
    }
  }
}
