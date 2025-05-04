use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

static RUNNING: AtomicBool = AtomicBool::new(false);

pub fn is_running() -> bool {
  RUNNING.load(Ordering::Acquire)
}

pub(crate) fn mark_running() -> crate::Result<()> {
  match RUNNING.compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire) {
    Ok(_) => Ok(()),
    Err(_) => Err(crate::Error::NodejsAlreadyRunning),
  }
}

pub(crate) fn mark_stopped() -> crate::Result<()> {
  match RUNNING.compare_exchange(true, false, Ordering::Acquire, Ordering::Acquire) {
    Ok(_) => Ok(()),
    Err(_) => Err(crate::Error::NodejsNotRunning),
  }
}
