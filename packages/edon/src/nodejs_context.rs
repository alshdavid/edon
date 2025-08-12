use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use crate::internal::NodejsWorkerEvent;
use crate::internal::NodejsMainEvent;
use crate::Env;
use crate::NodejsOptions;
use crate::NODEJS_CONTEXT_COUNT;

pub struct NodejsWorker {
  id: String,
  tx_main: Sender<NodejsMainEvent>,
  tx_wrk: Sender<NodejsWorkerEvent>,
}

impl NodejsWorker {
  pub(crate) fn start(
    options: &NodejsOptions,
    tx_main: Sender<NodejsMainEvent>,
  ) -> crate::Result<Self> {
    NODEJS_CONTEXT_COUNT.fetch_add(1, Ordering::AcqRel);
    let (tx, rx) = channel();
    let (tx_wrk, rx_wrk) = channel::<NodejsWorkerEvent>();

    tx_main
      .send(NodejsMainEvent::StartCommonjsWorker {
        rx_wrk,
        argv: options.as_argv(),
        resolve: tx,
      })
      .ok();

    let id = rx.recv().unwrap();

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
      .send(NodejsWorkerEvent::Eval { code, resolve: tx })
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
      .send(NodejsWorkerEvent::EvalModule { code, resolve: tx })
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
      .send(NodejsWorkerEvent::Exec {
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
      .send(NodejsWorkerEvent::Require {
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
      .send(NodejsWorkerEvent::Import {
        specifier: specifier.as_ref().to_string(),
        resolve: tx,
      })
      .ok();

    rx.recv().unwrap()
  }
}

impl Drop for NodejsWorker {
  fn drop(&mut self) {
    let (tx, rx) = channel();
    self
      .tx_main
      .send(NodejsMainEvent::StopCommonjsWorker {
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
        .send(NodejsMainEvent::StopMain { resolve: tx })
        .unwrap();
      rx.recv().unwrap();
    }
  }
}
