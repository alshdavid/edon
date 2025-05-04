#[doc(hidden)]
#[macro_export]
macro_rules! _check_status_or_throw {
  ($env:expr, $code:expr, $($msg:tt)*) => {
    if let Err(e) = $crate::napi::check_status!($code, $($msg)*) {
      #[allow(unused_unsafe)]
      unsafe { $crate::napi::JsError::from(e).throw_into($env) };
    }
  };
}
pub use _check_status_or_throw as check_status_or_throw;