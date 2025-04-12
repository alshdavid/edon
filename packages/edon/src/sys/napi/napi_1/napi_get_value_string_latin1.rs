use std::ffi::c_char;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "napi_get_value_string_latin1".as_bytes();
type SIGNATURE = fn(
  env: napi_env,
  value: napi_value,
  buf: *mut c_char,
  bufsize: usize,
  result: *mut usize,
) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn napi_get_value_string_latin1(
  env: napi_env,
  value: napi_value,
  buf: *mut c_char,
  bufsize: usize,
  result: *mut usize,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env, value, buf, bufsize, result,
  )
}
