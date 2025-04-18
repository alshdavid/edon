use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_acquire_threadsafe_function".as_bytes();
type SIGNATURE = fn(func: napi_threadsafe_function) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_acquire_threadsafe_function(func: napi_threadsafe_function) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL).unwrap())(func)
}
