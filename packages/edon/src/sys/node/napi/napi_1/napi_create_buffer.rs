use std::ffi::c_void;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_create_buffer".as_bytes();
type SIGNATURE =
  fn(env: napi_env, length: usize, data: *mut *mut c_void, result: *mut napi_value) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_create_buffer(
  env: napi_env,
  length: usize,
  data: *mut *mut c_void,
  result: *mut napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(env, length, data, result)
}
