use std::sync::mpsc::{channel, Sender};

use crate::{internal::NodejsContextEvent, Env};



pub struct NodejsContext {
  tx_wrk: Sender<NodejsContextEvent>,
}

impl NodejsContext {
  pub (crate) fn start(tx_wrk: Sender<NodejsContextEvent>) -> crate::Result<Self> {
    return Ok(Self { tx_wrk })
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

  pub fn require<Specifier: AsRef<str>>(&self, specifier: Specifier) -> crate::Result<()> {
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

  pub fn import<Specifier: AsRef<str>>(&self, specifier: Specifier) -> crate::Result<()> {
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
