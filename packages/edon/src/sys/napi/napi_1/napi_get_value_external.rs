use std::ffi::c_void;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_get_value_external".as_bytes();
type SIGNATURE = fn(env: napi_env, value: napi_value, result: *mut *mut c_void) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_get_value_external(
  env: napi_env,
  value: napi_value,
  result: *mut *mut c_void,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, value, result)
}
