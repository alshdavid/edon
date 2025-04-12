use std::ffi::c_void;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_get_threadsafe_function_context".as_bytes();
type SIGNATURE = fn(func: napi_threadsafe_function, result: *mut *mut c_void) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_get_threadsafe_function_context(
  func: napi_threadsafe_function,
  result: *mut *mut c_void,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(func, result)
}
