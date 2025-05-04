pub struct NodejsWorker {
  // tx_eval: Sender<(String, Sender<()>)>,
}

impl NodejsWorker {
  /// Evaluate block of JavaScript
  pub fn eval<Code: AsRef<str>>(
    &self,
    _code: Code,
  ) -> crate::Result<()> {
    // let (tx, rx) = channel();
    // self.tx_eval.send((code.as_ref().to_string(), tx)).unwrap();
    // rx.recv().unwrap();
    Ok(())
  }
}
