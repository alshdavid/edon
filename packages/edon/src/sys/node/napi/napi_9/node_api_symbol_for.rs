use std::ffi::c_char;
use std::sync::OnceLock;

use super::super::*;

const SYMBOL: &[u8] = "node_api_symbol_for".as_bytes();
type SIGNATURE =
  fn(env: napi_env, utf8name: *const c_char, length: isize, result: *mut napi_value) -> napi_status;
static CACHE: OnceLock<super::super::super::libnode::DynSymbol<SIGNATURE>> = OnceLock::new();

pub unsafe fn node_api_symbol_for(
  env: napi_env,
  utf8name: *const c_char,
  length: isize,
  result: *mut napi_value,
) -> napi_status {
  CACHE.get_or_init(|| super::super::super::libnode::libnode_sym(SYMBOL))(
    env, utf8name, length, result,
  )
}
